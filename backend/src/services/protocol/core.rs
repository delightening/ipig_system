use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use validator::Validate;

use super::history::event_type_for;
use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        CreatePartnerRequest, CreateProtocolRequest, ImportApprovedProtocolRequest, PartnerType,
        Protocol, ProtocolActivityType, ProtocolListItem, ProtocolQuery, ProtocolResponse,
        ProtocolRole, ProtocolStatus, UpdateProtocolRequest,
    },
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, PartnerService,
    },
    AppError, Result,
};

const CONFLICT_MSG: &str = "此記錄已被其他人修改，請重新載入後再試。";

/// SD 變更專用的 focused audit diff（裁定 21）。
///
/// 存在理由：`DataDiff::compute` 吃 `Serialize + AuditRedact`，直接餵整個
/// `Protocol` 會得到「整份快照」——`changed_fields` 裡雖然有
/// `study_director_user_id`，但同時還有其他一起變動的欄位，
/// 於是無法用它區分「這次改了 SD」與「這次改了一堆東西、SD 剛好也在裡面」。
/// 只放這一個欄位，`changed_fields` 就恰好等於 `["study_director_user_id"]`。
///
/// 不含姓名、只存 UUID：稽核要查是誰，join `users` 即可；
/// 把姓名複製進不可竄改的 audit row 等於在裡面固化個資。
#[derive(serde::Serialize)]
struct SdChangeAudit {
    study_director_user_id: Option<Uuid>,
}

impl AuditRedact for SdChangeAudit {}

/// 驗證匯入里程碑日期依時序遞增
/// （申請→預審→獸醫→委員一審→補件→委員二審→核准）。只檢查有填的里程碑。
fn validate_import_milestone_order(
    req: &ImportApprovedProtocolRequest,
    effective_approved_at: Option<NaiveDate>,
) -> Result<()> {
    let ordered = [
        req.submitted_at,
        req.pre_review_at,
        req.vet_review_at,
        req.committee_first_review_at,
        req.revision_required_at,
        req.committee_second_review_at,
        effective_approved_at,
    ];
    let mut last: Option<NaiveDate> = None;
    for d in ordered.into_iter().flatten() {
        if let Some(prev) = last {
            if d < prev {
                return Err(AppError::Validation(
                    "審查里程碑日期必須依時序遞增（申請→預審→獸醫→委員一審→補件→委員二審→核准）"
                        .into(),
                ));
            }
        }
        last = Some(d);
    }
    Ok(())
}

/// 從 `working_content` 取出使用者填的 GLP 勾選值。
///
/// 回 `None` 代表「這次沒有表態」（沒帶 working_content、沒有 basic 區段、
/// 沒有 is_glp 欄位，或值不是可辨識的布林表示），呼叫端自行決定要沿用舊值
/// 還是視為 false。
///
/// ⚠️ 這個函式**不是**判定「這份計畫是不是 GLP」的來源——那是
/// `protocols.is_glp` 欄位（migration 006 / 裁定 14）。這裡取的是使用者在表單上
/// 的輸入，只在寫入欄位時當作素材。混淆兩者正是裁定 14 要消除的問題。
///
/// 🔴 **字串 `"true"` / `"false"` 必須跟 JSON 布林一樣認**（CodeRabbit #25 指出）。
/// migration 006 的回填用 `working_content -> 'basic' ->> 'is_glp' = 'true'`，
/// 而 `->>` 回傳 text——JSON 布林 `true` 與字串 `"true"` 都得到 `'true'`。
/// 若這裡只認 `as_bool()`，同一份內容在兩邊會得到相反結論：
///
/// | | 字串 `"true"` |
/// |---|---|
/// | migration `->>` | GLP |
/// | `as_bool()` → `None` → `unwrap_or(false)` | 非 GLP |
/// | 前端 `basic.is_glp ? …`（JS truthiness） | GLP |
///
/// 前端型別是 `is_glp: boolean`、送的一定是布林，但 `working_content` 在後端是
/// 未驗證的 `serde_json::Value`——直接打 API 就能送字串進來，結果是欄位說非 GLP、
/// 畫面說 GLP。那正是裁定 14 要消除的「JSON 與欄位打架」。
///
/// 只認這兩個字串、不認 `"TRUE"` / `"1"` / `"yes"`：因為判準是**與 migration 對齊**，
/// 不是「盡量寬容」。migration 的 `= 'true'` 也不認那些，多認就又製造新的分歧。
fn glp_flag_in_content(content: Option<&serde_json::Value>) -> Option<bool> {
    let value = content
        .and_then(|c| c.get("basic"))
        .and_then(|b| b.get("is_glp"))?;
    match value {
        serde_json::Value::Bool(b) => Some(*b),
        serde_json::Value::String(s) if s == "true" => Some(true),
        serde_json::Value::String(s) if s == "false" => Some(false),
        _ => None,
    }
}

impl ProtocolService {
    /// 生成計畫編號
    /// 格式：Pre-{民國年}-{序號:03}
    /// 例如：Pre-114-001, Pre-114-002
    async fn generate_protocol_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;

        // 查詢該民國年的所有計畫編號
        let prefix = format!("Pre-{}-", roc_year);
        let protocol_nos: Vec<String> =
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE protocol_no LIKE $1")
                .bind(format!("{}%", prefix))
                .fetch_all(pool)
                .await?;

        // 解析序號並找出最大值
        let max_seq = protocol_nos
            .iter()
            .filter_map(|no| {
                // 格式：Pre-114-001，提取最後的數字部分
                let parts: Vec<&str> = no.split('-').collect();
                if parts.len() >= 3 {
                    parts[2].parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 建立計畫
    ///
    /// R30-29：INSERT + user_protocols + audit-in-tx 全 tx 原子。
    /// 失敗整 tx rollback，不留半成品。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateProtocolRequest,
        created_by: Uuid,
    ) -> Result<Protocol> {
        let protocol_no = Self::generate_protocol_no(pool).await?;
        let pi_user_id = req.pi_user_id.unwrap_or(created_by);

        let mut tx = pool.begin().await?;

        // 計劃負責人（SD，選填）：客戶/PI 建立時通常留空，由執行秘書事後指派。
        // 有指定時驗證 + 授權（僅執秘/admin 可指派他人，其餘限本人）。
        //
        // 🔴 判準必須與 `update` 一致（CodeRabbit #26 第 2 輪指出）。
        //
        // 第一版直接傳 `req.pi_user_id`，理由是「PI 留空時退回 `created_by` 是佔位，
        // 拿它比對 SD 會誤擋執秘自任 SD（裁定 10／11）」。那個理由對，但**不完整**：
        // `req.pi_user_id` 為 `None` 時，`created_by` **不一定**是佔位——
        // 他也可能是真的 PI 在建自己的計畫。
        //
        // 漏掉的攻擊路徑：同時具 `PI` 與 `EXPERIMENT_STAFF` 的人，建立計畫時
        // 不填 pi_user_id、把自己設成 SD → 存進去的 `pi_user_id` 與
        // `study_director_user_id` 都是他本人 → **裁定 16 被繞過**。
        // 而 `update` 那邊的啟發式抓不到它（那人有 PI 角色，不算佔位），
        // 只會擋住之後的變更——但違規狀態已經在建立時就寫進去了。
        //
        // 2026-08-26 實測正式庫：**沒有**使用者同時具那兩個角色，
        // 所以目前**無法觸發**。
        // 仍然修，因為角色指派是例行管理動作，而且 create 與 update 判準不一致
        // 本身就是遲早會咬人的東西。
        let effective_pi_for_check = match req.pi_user_id {
            Some(explicit) => Some(explicit),
            // PI 留空：只有在建立者「沒有 PI 角色」時才視為外部 PI 的佔位
            None => {
                let creator_is_pi = sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(
                         SELECT 1 FROM user_roles ur
                         JOIN roles r ON r.id = ur.role_id
                         WHERE ur.user_id = $1 AND r.code = 'PI'
                       )"#,
                )
                .bind(created_by)
                .fetch_one(&mut *tx)
                .await?;
                if creator_is_pi {
                    Some(created_by)
                } else {
                    None
                }
            }
        };
        if let Some(sd_id) = req.study_director_user_id {
            Self::validate_and_authorize_sd(&mut tx, actor, sd_id, effective_pi_for_check).await?;
        }

        // 把上面判斷「PI 是不是外部人員的佔位值」的結果**寫死**進去（migration 009），
        // 不要事後用「pi_user_id == created_by 現查角色」回推——那套啟發式的結果
        // 會隨事後的角色異動漂移，`update` 之後只讀這個欄位。
        let pi_is_external = effective_pi_for_check.is_none();

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, protocol_no, title, status, pi_user_id, study_director_user_id,
                working_content, start_date, end_date, created_by, is_glp, pi_is_external,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&protocol_no)
        .bind(&req.title)
        .bind(ProtocolStatus::Draft)
        .bind(pi_user_id)
        .bind(req.study_director_user_id)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(created_by)
        // 建立當下就把表單上的 GLP 勾選寫進權威欄位——否則新計畫的欄位
        // 一律是 false，與 working_content 立刻不一致。
        .bind(glp_flag_in_content(req.working_content.as_ref()).unwrap_or(false))
        .bind(pi_is_external)
        .fetch_one(&mut *tx)
        .await?;

        // user_protocols：PI 連結
        sqlx::query(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (user_id, protocol_id) DO NOTHING
            "#
        )
        .bind(pi_user_id)
        .bind(protocol.id)
        .bind(ProtocolRole::Pi)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        // protocol_activities + user_activity_logs（同 tx）
        Self::record_activity_tx(
            &mut tx,
            actor,
            protocol.id,
            ProtocolActivityType::Created,
            None,
            Some(protocol.status.as_str().to_string()),
            None,
            None,
            None,
        )
        .await?;

        // Service-driven audit：完整 create 快照進 HMAC chain
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_CREATE",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&protocol)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(protocol)
    }

    /// 驗證並授權「計劃負責人（SD）」指派：SD 必須是啟用中、本公司內部、具
    /// EXPERIMENT_STAFF 角色者，且**不得與該計畫的 PI 為同一人**（裁定 16）；
    /// 授權上僅 IACUC_STAFF / 管理員可指派「他人」為 SD，
    /// 其餘登入者只能指派自己。System actor（維運/種子）不受限。
    /// 共用於 import_approved / create / update（DRY，CLAUDE.md §7 權限集中）。
    async fn validate_and_authorize_sd(
        conn: &mut sqlx::PgConnection,
        actor: &ActorContext,
        sd_id: Uuid,
        pi_user_id: Option<Uuid>,
    ) -> Result<()> {
        // 裁定 16：PI 不可兼任同一計畫的 SD。
        //
        // ⚠️ 這不是形式上的職稱分離，而是**結案雙簽的正確性前提**：
        // 裁定 9 要求 PI 與 SD 各簽一次才能結案，同一人的話兩張簽章的
        // signer_id 會是同一個 UUID——稽核鏈上看起來是「雙方認可」，
        // 實際是自簽自證，正是 21 CFR §11.10(g) 職權分離要防的東西。
        //
        // 放在最前面而不是最後：先擋掉語意上不合法的組合，再談那個人有沒有
        // 資格當 SD。錯誤訊息也因此更貼近使用者實際做錯的事——PI 通常是外部
        // 人員、本來就沒有 EXPERIMENT_STAFF 角色，若讓角色檢查先跑，
        // 使用者收到的會是「此人不具試驗工作人員角色」，完全看不出真正的問題。
        //
        // ⚠️ 為什麼參數是 `Option`：`create` 與 `import_approved` 都用
        // `req.pi_user_id.unwrap_or(created_by)`——**外部 PI（沒有系統帳號）時，
        // `protocols.pi_user_id` 記的是匯入者本人**，通常就是執行祕書。
        // 拿那個佔位值做職責分離檢查在語意上是錯的：它不代表任何人是 PI，
        // 而裁定 10／11 明確允許執秘自任 SD。所以只在 PI **被明確指定**
        // （`req.pi_user_id.is_some()`）時才比對。
        //
        // 2026-08-25 實測正式庫：存在 `pi_user_id = created_by` 的計畫，
        // 其中同時 PI=SD 的是少數（即裁定 16 要處理的存量），
        // 所以這個放寬目前不會漏掉任何真實的違規案例。
        if let Some(pi_id) = pi_user_id {
            if sd_id == pi_id {
                return Err(AppError::Validation(
                    "計畫主持人（PI）不可兼任同一計畫的計劃負責人（Study Director）；請改指派其他試驗工作人員"
                        .into(),
                ));
            }
        }

        // 🔴 **與「停用帳號」序列化**（CodeRabbit #27 第 3 輪指出，2026-08-27）。
        //
        // 沒有這道鎖時的競態：
        //   T1 指派 SD → 讀到該使用者 is_active = true（無鎖）
        //   T2 停用該使用者 → 對 users 列 FOR UPDATE → 檢查「有沒有未結案 GLP 案
        //      以他為 SD」→ 此刻 T1 還沒寫入 → 通過 → 停用
        //   T1 寫入 protocols.study_director_user_id → commit
        //   結果：**已停用的人成為未結案 GLP 計畫的 SD，而兩邊的閘都通過了**。
        //
        // 為什麼 FOR SHARE 就夠、而且不會死鎖（2026-08-27 實測確認前提）：
        //   指派端鎖序 = protocols FOR UPDATE → users FOR SHARE
        //   停用端鎖序 = users FOR UPDATE → 讀 protocols（**不加鎖**，
        //     MVCC 下純 SELECT 不會等 FOR UPDATE）
        //   停用端從不等待 protocols 上的鎖，所以沒有循環等待。
        //
        //   兩種先後都能擋住（isolation 為預設的 read committed）：
        //   - 停用端先拿到 users 鎖 → 指派端卡在 FOR SHARE →
        //     停用 commit 後指派端才讀 is_active → 讀到 false → 擋下。
        //   - 指派端先拿到 FOR SHARE → 停用端卡在 FOR UPDATE →
        //     指派 commit 後停用端才跑 GLP 檢查，那是**新的一句 SQL**，
        //     read committed 會取新快照 → 看得到剛寫入的 SD → 擋下。
        //
        // ⚠️ 鎖獨立成一句，不併進下面的 EXISTS：帶 join 的 EXISTS 子查詢加鎖定子句
        // 在 Postgres 有限制（且鎖到哪張表不明顯）。分開寫也讓「鎖的是 users 這一列」
        // 這件事直接看得出來。
        let sd_exists: Option<bool> =
            sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1 FOR SHARE")
                .bind(sd_id)
                .fetch_optional(&mut *conn)
                .await?;
        // 不存在就讓下面的資格查詢回同一句錯誤訊息，不另外分歧。
        let _ = sd_exists;

        let sd_is_valid: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                 SELECT 1 FROM users u
                 JOIN user_roles ur ON ur.user_id = u.id
                 JOIN roles r ON r.id = ur.role_id
                 WHERE u.id = $1 AND u.is_active = true AND u.deleted_at IS NULL
                   AND u.is_internal = true
                   AND r.code = $2
               )"#,
        )
        .bind(sd_id)
        .bind(crate::constants::ROLE_EXPERIMENT_STAFF)
        .fetch_one(&mut *conn)
        .await?;
        if !sd_is_valid {
            return Err(AppError::Validation(
                "計劃負責人（Study Director）必須是啟用中且具試驗工作人員（EXPERIMENT_STAFF）角色的員工".into(),
            ));
        }
        // 授權：僅執行秘書(IACUC_STAFF) / 管理員可指派「他人」；其餘登入者只能設自己。
        if let ActorContext::User(u) = actor {
            let can_assign_any_sd = u.is_admin()
                || u.roles
                    .iter()
                    .any(|r| r == crate::constants::ROLE_IACUC_STAFF);
            if !can_assign_any_sd && sd_id != u.id {
                return Err(AppError::Forbidden(
                    "僅執行秘書或管理員可指派他人為計劃負責人；請將自己設為計劃負責人".into(),
                ));
            }
        }
        Ok(())
    }

    /// 匯入「已核准計劃」：場內既有、已通過審查的計劃直接進系統成 APPROVED，
    /// **跳過 IACUC 審查 state machine**（不需 review_assignments / 委員評論 / 獸醫審查）。
    ///
    /// 合規軌跡：audit event_type=`PROTOCOL_IMPORT_APPROVED`（與系統內審查通過區隔）。
    /// 會計接點：依 iacuc_no 自動建立 customer partner（同正常核准流程 status.rs）。
    /// 全 tx 原子；失敗整 tx rollback。
    pub async fn import_approved(
        pool: &PgPool,
        actor: &ActorContext,
        req: &ImportApprovedProtocolRequest,
        created_by: Uuid,
    ) -> Result<Protocol> {
        // Service 層拒絕 Anonymous 觸發 mutation（CLAUDE.md §ActorContext::Anonymous 規範 2）
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "匯入已核准計劃須由已登入使用者或系統觸發".into(),
                ));
            }
        }
        let iacuc_no = req.iacuc_no.trim();
        if iacuc_no.is_empty() {
            return Err(AppError::Validation("IACUC 編號為必填".into()));
        }
        // 日期區間驗證：兩者皆有時，結束日不可早於起始日
        if let (Some(start), Some(end)) = (req.start_date, req.end_date) {
            if end < start {
                return Err(AppError::Validation("計畫結束日不可早於起始日".into()));
            }
        }
        // 審查里程碑日期需依時序遞增（使用實際會寫入的 approved_at 做驗證）
        let effective_approved_at = req.approved_at.or_else(|| Some(Utc::now().date_naive()));
        validate_import_milestone_order(req, effective_approved_at)?;
        let protocol_no = Self::generate_protocol_no(pool).await?;

        let mut tx = pool.begin().await?;

        // iacuc_no 唯一性：避免與既有計劃撞號
        let dup: Option<Uuid> = sqlx::query_scalar("SELECT id FROM protocols WHERE iacuc_no = $1")
            .bind(iacuc_no)
            .fetch_optional(&mut *tx)
            .await?;
        if dup.is_some() {
            return Err(AppError::BusinessRule(format!(
                "IACUC 編號 {iacuc_no} 已存在，無法重複匯入"
            )));
        }

        // PI 解析：選系統內 PI 時用其 id（並收緊驗證）；外部 PI（None）記為匯入者本人，
        // 真實 PI 身分存於 working_content.basic.pi / sponsor（比照 /protocols/new）。
        let effective_pi_user_id = req.pi_user_id.unwrap_or(created_by);

        // 外部 PI（無系統帳號）必須於 working_content.basic.pi.name 提供姓名，
        // 否則計畫無可識別的主持人（前端已擋，後端再守一層 API contract）。
        if req.pi_user_id.is_none() {
            let pi_name = req
                .working_content
                .as_ref()
                .and_then(|c| c.get("basic"))
                .and_then(|b| b.get("pi"))
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .map(str::trim)
                .unwrap_or("");
            if pi_name.is_empty() {
                return Err(AppError::Validation(
                    "外部計畫主持人必須於 working_content.basic.pi.name 提供姓名".into(),
                ));
            }
        }

        // 匯入授權收緊（CSO follow-up）：選定的系統內 PI 必須是啟用中、且具 `PI` 角色。
        // 外部 PI 無系統帳號，不適用此檢查。
        //
        // 原本白名單另含全域 `STUDY_DIRECTOR` 角色，已隨該角色一併移除——SD 與 PI
        // 是兩種不同的身分，把 SD 混進「誰可以當 PI」的白名單本身就是角色概念混淆。
        if let Some(pi_id) = req.pi_user_id {
            let pi_is_valid: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(
                     SELECT 1 FROM users u
                     JOIN user_roles ur ON ur.user_id = u.id
                     JOIN roles r ON r.id = ur.role_id
                     WHERE u.id = $1 AND u.is_active = true AND u.deleted_at IS NULL
                       AND r.code = $2
                   )"#,
            )
            .bind(pi_id)
            .bind(crate::constants::ROLE_PI)
            .fetch_one(&mut *tx)
            .await?;
            if !pi_is_valid {
                return Err(AppError::Validation(
                    "指定的計畫主持人必須是啟用中且具 PI 角色的使用者".into(),
                ));
            }
        }

        // 計劃負責人（SD）驗證 + 授權（共用 helper；外部協作者即使誤掛 EXPERIMENT_STAFF 也不得任 SD）
        //
        // 同 create：傳 `req.pi_user_id` 而非 `effective_pi_user_id`。
        // 補登匯入的計畫多半是外部 PI，那時 effective 值等於匯入者本人（佔位）。
        Self::validate_and_authorize_sd(&mut tx, actor, req.study_director_user_id, req.pi_user_id)
            .await?;
        // 註：&mut tx 經 DerefMut 轉為 &mut PgConnection

        // 把「PI 是不是外部人員的佔位值」寫死進去（migration 009），道理同 create：
        // `req.pi_user_id` 為 `None` 時 `effective_pi_user_id` 就是匯入者本人的佔位，
        // 之後 `update` 只讀這個欄位，不再現查角色。
        let pi_is_external = req.pi_user_id.is_none();

        // 申請編號（選填）：trim 後空字串視為 NULL
        let application_no = req
            .application_no
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        // 版本名冊：補登「先選版本」帶入的表單版本鍵（C/D/E/F…），trim 後空字串視為 None
        let source_form_version = req
            .source_form_version
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        // import P1：匯入即標記 import_pending=true（APPROVED 但允許補登 working_content）
        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, protocol_no, iacuc_no, application_no, title, status, import_pending,
                pi_user_id, study_director_user_id, working_content,
                start_date, end_date, submitted_at, approved_at, created_by,
                source_form_version, is_glp, pi_is_external, imported_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, NOW(), NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&protocol_no)
        .bind(iacuc_no)
        .bind(application_no)
        .bind(&req.title)
        .bind(ProtocolStatus::Approved)
        .bind(true)
        .bind(effective_pi_user_id)
        .bind(req.study_director_user_id)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(req.submitted_at)
        .bind(effective_approved_at)
        .bind(created_by)
        .bind(source_form_version)
        // 補登匯入：以匯入內容為準（此時 import_pending=true，尚未鎖定）
        .bind(glp_flag_in_content(req.working_content.as_ref()).unwrap_or(false))
        .bind(pi_is_external)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (user_id, protocol_id) DO NOTHING
            "#,
        )
        .bind(effective_pi_user_id)
        .bind(protocol.id)
        .bind(ProtocolRole::Pi)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        Self::ensure_customer_partner_tx(&mut tx, iacuc_no).await?;

        // 依使用者填寫的歷史里程碑日期，backfill 完整審查時間軸至 protocol_activities
        Self::backfill_import_timeline_tx(&mut tx, actor, protocol.id, req).await?;

        // PR-E 須知簽署承接（方案 A）：提供須知版次標籤時，記一筆歷史 acknowledgement
        // （無電子簽章；signer_id = 解析後 PI/匯入者；歷史簽署日期）。匯入計劃不經 submit()，
        // 故此處僅為合規歷史紀錄，不觸發送審門檻。
        if let Some(label) = req
            .notice_version_label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let notice = crate::repositories::application_notice::ApplicationNoticeRepository::find_by_version_label(pool, label)
                .await?
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "找不到須知版次「{label}」，請先於須知版本登記建立該版本"
                    ))
                })?;
            let ack_at = req
                .notice_acknowledged_at
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc());
            crate::repositories::application_notice::NoticeAcknowledgementRepository::insert_legacy_tx(
                &mut tx,
                protocol.id,
                notice.id,
                effective_pi_user_id,
                req.notice_attachment_id,
                ack_at,
            )
            .await?;
        }

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_IMPORT_APPROVED",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&protocol)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(protocol)
    }

    /// import P1：完成補登 — 清 import_pending 旗標 + 建 v1 版本快照 + 記原始版本號。
    /// 僅對 import_pending=true 的計劃有效；完成後計劃恢復一般 APPROVED 鎖定。
    pub async fn finalize_import(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        original_version_label: Option<String>,
    ) -> Result<Protocol> {
        // Service 層拒絕 Anonymous 觸發 mutation；亦保證 protocol_versions.submitted_by
        // （NOT NULL）取得非 NULL 的 actor_user_id（CLAUDE.md §ActorContext::Anonymous 規範 2）
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "完成補登須由已登入使用者或系統觸發".into(),
                ));
            }
        }
        let mut tx = pool.begin().await?;
        let before: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if !before.import_pending {
            return Err(AppError::BusinessRule(
                "此計劃非補登中狀態，無法完成補登".to_string(),
            ));
        }

        // v1 版本快照（content_snapshot NOT NULL → working_content 為空時以 {} 代）
        let version_no = Self::get_next_version_no_tx(&mut tx, id).await?;
        let snapshot = before
            .working_content
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        sqlx::query(
            r#"
            INSERT INTO protocol_versions (id, protocol_id, version_no, content_snapshot, submitted_at, submitted_by)
            VALUES ($1, $2, $3, $4, NOW(), $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(version_no)
        .bind(&snapshot)
        .bind(actor.actor_user_id())
        .execute(&mut *tx)
        .await?;

        let label = original_version_label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let updated: Protocol = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET
                import_pending = false,
                original_version_label = COALESCE($2, original_version_label),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(label)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_IMPORT_FINALIZED",
                entity: Some(AuditEntity::new("protocol", updated.id, &updated.title)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// 刪除誤匯的匯入計劃（admin only，硬刪 + 限無下游資料）以重新匯入。R64-5c。
    /// 守衛：`imported_at` 非 NULL（僅匯入計劃）+ 無 amendment + 無 active byproduct。
    /// scaffold（user_protocols / versions / activities / 補登審查）由 FK CASCADE 連帶刪；
    /// 其餘 FK（GLP NO ACTION / byproduct RESTRICT）違反時映射為友善 BusinessRule。
    pub async fn delete_imported_protocol(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "刪除匯入計劃須由已登入使用者或系統觸發".into(),
                ));
            }
        }

        let mut tx = pool.begin().await?;
        let protocol: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // R64-5c → 擴充：admin 可硬刪「匯入計劃 / 已駁回 / 草稿」三類非執行中計畫。
        // 下游資料守衛（amendments / byproduct 樣品）與 audit 一律保留，
        // 不開放刪除執行中 / 已核准 / 有下游資料的計畫。
        let deletable = protocol.imported_at.is_some()
            || matches!(
                protocol.status,
                crate::models::ProtocolStatus::Rejected | crate::models::ProtocolStatus::Draft
            );
        if !deletable {
            return Err(AppError::BusinessRule(
                "僅匯入計劃、已駁回或草稿計畫可刪除".to_string(),
            ));
        }

        // 下游資料守衛：amendments 為 CASCADE，不擋會靜默刪 → 明確擋；byproduct 為 RESTRICT。
        let has_amendment: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM amendments WHERE protocol_id = $1)")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        if has_amendment {
            return Err(AppError::BusinessRule(
                "計劃已有變更申請，不可刪除（請先處理變更）".to_string(),
            ));
        }
        let has_byproduct: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM euthanasia_byproduct_samples \
             WHERE source_protocol_id = $1 AND deleted_at IS NULL)",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if has_byproduct {
            return Err(AppError::BusinessRule(
                "計劃已有廢棄物樣品紀錄，不可刪除".to_string(),
            ));
        }

        // audit 先寫（刪除後仍可追溯；user_activity_logs 對 entity_id 無 FK）
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_DELETED",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::compute(Some(&protocol), None)),
                request_context: None,
            },
        )
        .await?;

        // 硬刪（scaffold 由 FK CASCADE 連帶刪）；殘餘 FK 違反（23503）→ 友善錯誤
        let res = sqlx::query("DELETE FROM protocols WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await;
        match res {
            Ok(_) => {}
            Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
                return Err(AppError::BusinessRule(
                    "計劃尚有關聯資料，無法刪除".to_string(),
                ));
            }
            Err(e) => return Err(e.into()),
        }

        tx.commit().await?;
        Ok(())
    }

    /// Admin 軟刪除：將「已否決」計畫設為 DELETED（從列表隱藏，保留資料供稽核）。
    /// 不走泛型 `change_status` 以維持「終態不可離開」egress 鎖（CSO-r2 #2）。
    /// 權限（admin only）由 handler 層把關；此處要求 User/System actor 並限 REJECTED 來源。
    pub async fn soft_delete_protocol(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        // 防禦縱深：service 層自身把關 admin（不只依賴 handler 守衛；coderabbit review）。
        let is_admin_actor = match actor {
            ActorContext::System { .. } => true,
            ActorContext::User(u) => u.is_admin(),
            ActorContext::Anonymous => false,
        };
        if !is_admin_actor {
            return Err(AppError::Forbidden("僅系統管理員可軟刪除計畫".to_string()));
        }

        let mut tx = pool.begin().await?;
        let protocol: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if protocol.status != ProtocolStatus::Rejected {
            return Err(AppError::BusinessRule("僅已否決計畫可軟刪除".to_string()));
        }

        // after 快照供 audit data_diff（status：REJECTED → DELETED）
        let mut after = protocol.clone();
        after.status = ProtocolStatus::Deleted;

        // WHERE 帶當前狀態防併發覆寫（列雖已 FOR UPDATE 鎖定，仍依規範保留；gemini review）。
        sqlx::query("UPDATE protocols SET status = 'DELETED', updated_at = NOW() WHERE id = $1 AND status = $2")
            .bind(id)
            .bind(protocol.status)
            .execute(&mut *tx)
            .await?;

        // 內部活動時間軸（與其他狀態變更一致）；from 取自鎖定列的實際狀態。
        Self::record_status_change_tx(
            &mut tx,
            actor,
            id,
            Some(protocol.status),
            ProtocolStatus::Deleted,
            Some("系統管理員軟刪除計畫".to_string()),
        )
        .await?;

        // HMAC chain audit（record_status_change_tx 不寫 user_activity_logs，由此補）。
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_SOFT_DELETED",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::compute(Some(&protocol), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 依 IACUC No. 自動建立 customer partner（會計接點），複製 status.rs 核准流程行為：
    /// 已存在則跳過；建立失敗只 warn 不阻斷（與既有行為一致）。
    async fn ensure_customer_partner_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        iacuc_no: &str,
    ) -> Result<()> {
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1",
        )
        .bind(iacuc_no)
        .fetch_optional(&mut **tx)
        .await?;
        if existing.is_some() {
            return Ok(());
        }
        let create_req = CreatePartnerRequest {
            partner_type: PartnerType::Customer,
            code: Some(iacuc_no.to_string()),
            supplier_category: None,
            customer_category: None,
            name: iacuc_no.to_string(),
            tax_id: None,
            phone: None,
            phone_ext: None,
            email: None,
            address: None,
            payment_terms: None,
        };
        if create_req.validate().is_err() {
            return Ok(());
        }
        let actor = ActorContext::System {
            reason: "auto_create_customer_from_iacuc",
        };
        if let Err(e) = PartnerService::create_tx(&mut *tx, &actor, &create_req).await {
            tracing::warn!("匯入計劃：依 IACUC {} 建立客戶失敗：{}", iacuc_no, e);
        }
        Ok(())
    }

    /// 複製既有計畫建立新草稿
    /// 複製 title、working_content、start_date、end_date，新計畫狀態為 DRAFT
    ///
    /// R30-29：INSERT + user_protocols + audit-in-tx 全 tx 原子。
    pub async fn copy(
        pool: &PgPool,
        actor: &ActorContext,
        source: access::Scoped<access::ProtocolId>,
        copied_by: Uuid,
    ) -> Result<Protocol> {
        let source_id = source.id();
        let source = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(source_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("來源計畫不存在".to_string()))?;

        let new_protocol_no = Self::generate_protocol_no(pool).await?;
        let new_title = format!("（複製）{}", source.title);
        let pi_user_id = source.pi_user_id;

        let mut tx = pool.begin().await?;

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, protocol_no, title, status, pi_user_id, working_content,
                start_date, end_date, created_by, is_glp, pi_is_external, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&new_protocol_no)
        .bind(&new_title)
        .bind(ProtocolStatus::Draft)
        .bind(pi_user_id)
        .bind(&source.working_content)
        .bind(source.start_date)
        .bind(source.end_date)
        .bind(copied_by)
        // 🔴 複製直接繼承來源的**欄位**，不從 working_content 重新推導。
        //
        // 原本寫的是 `glp_flag_in_content(source.working_content)`，理由是
        // 「複製出來的是新 DRAFT、本來就還沒鎖定」。但裁定 14 之後那個理由不成立：
        // 雙向鎖對 DRAFT 一樣生效，複本的 is_glp 一設下去就再也改不了。
        // 既然如此，就該取權威來源——`protocols.is_glp` 欄位，而不是可能已經
        // 跟欄位不一致的 JSON（裁定 14 的整個重點就是「欄位才是判準」）。
        //
        // 附帶好處：來源若是 migration 006 回填來的（JSON 是字串 "true"、
        // 欄位是 true），複本直接拿 true，不必依賴上面那段字串解析也會對。
        .bind(source.is_glp)
        // 🔴 CodeRabbit #40 指出：pi_user_id 是直接複製來源的值，
        // pi_is_external 必須跟著複製，不能讓它落回欄位預設值 false。
        //
        // 複製不會改變「這個 pi_user_id 代表的是誰」這件事本身——它可能是
        // 來源計畫真正的 PI，也可能是來源建立者的外部 PI 佔位值，複製只是把
        // 同一個 pi_user_id 值搬到新計畫上，並沒有讓佔位變成真人。
        //
        // ⚠️ 若省略、讓它落回 DEFAULT false：來源若原本是外部 PI 佔位
        // （`pi_is_external = true`，`pi_user_id` = 來源建立者），複本的
        // `created_by` 是**複製者**（通常另有其人），於是複本會被誤判成
        // 「pi_user_id 是真正的 PI」，之後若複製者想自任 SD，會被 PI≠SD
        // 誤擋——而複製者根本不是那個 pi_user_id 代表的人。
        .bind(source.pi_is_external)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (user_id, protocol_id) DO NOTHING
            "#
        )
        .bind(pi_user_id)
        .bind(protocol.id)
        .bind(ProtocolRole::Pi)
        .bind(copied_by)
        .execute(&mut *tx)
        .await?;

        Self::record_activity_tx(
            &mut tx,
            actor,
            protocol.id,
            ProtocolActivityType::Created,
            None,
            Some(protocol.status.as_str().to_string()),
            None,
            Some(format!("複製自計畫 {}", source.protocol_no)),
            None,
        )
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_CREATE",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&protocol)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(protocol)
    }

    /// 查詢計畫列表（「計畫書管理」全覽用，限有 view_all 者呼叫）。
    ///
    /// - `viewer_id` / `is_admin`：計算每筆 `can_edit`（PI / SD / admin 可編輯）。
    /// - `viewer_sees_all_drafts`（執秘 / 主席 / admin）：草稿是否全可見。否則草稿僅對其
    ///   PI / SD / 成員可見，避免草稿經本端點外洩（見 PROGRESS 草稿可見性收緊條目）。
    pub async fn list(
        pool: &PgPool,
        query: &ProtocolQuery,
        viewer_id: Uuid,
        is_admin: bool,
        viewer_sees_all_drafts: bool,
        viewer: &crate::middleware::CurrentUser,
    ) -> Result<Vec<ProtocolListItem>> {
        // 固定參數：$1=viewer_id、$2=is_admin、$3=viewer_sees_all_drafts；可選過濾自 $4 起。
        let mut qb =
            sqlx::query_as::<_, ProtocolListItem>(sqlx::AssertSqlSafe(Self::build_list_sql(query)))
                .bind(viewer_id)
                .bind(is_admin)
                .bind(viewer_sees_all_drafts);
        if let Some(status) = query.status {
            if status != ProtocolStatus::Deleted {
                qb = qb.bind(status);
            }
        }
        if let Some(ref k) = query.keyword {
            qb = qb.bind(format!("%{}%", k.trim()));
        }
        if let Some(pid) = query.pi_user_id {
            qb = qb.bind(pid);
        }
        // 綁定順序須與 super::push_optional_protocol_filters 佔位符一致（status→keyword→
        // pi_user_id→start_date→end_date）。缺日期綁定會使帶日期過濾的查詢因未綁定參數噴錯。
        if let Some(start_date) = query.start_date {
            qb = qb.bind(start_date);
        }
        if let Some(end_date) = query.end_date {
            qb = qb.bind(end_date);
        }
        // 失敗一律向上拋（不再 unwrap_or_default → []，避免授權/查詢錯誤被誤判為「無計畫」）。
        let mut protocols: Vec<ProtocolListItem> = qb.fetch_all(pool).await?;
        Self::backfill_apig_nos(pool, &mut protocols).await?;

        // 「卡在誰」批次補算。目前只涵蓋行政受理 / 預審那幾關；委員會審查與需修正
        // 兩類的形狀不同（前者卡在指派委員、後者卡在申請人），解析器會回空。
        let ids: Vec<Uuid> = protocols.iter().map(|p| p.id).collect();
        if !ids.is_empty() {
            let mut owners =
                crate::services::pending_owner::resolve_for_protocols(pool, &ids, viewer).await?;
            for row in &mut protocols {
                row.pending_owner = owners.remove(&row.id);
            }
        }

        Ok(protocols)
    }

    /// 組裝 `list` 查詢 SQL（含 `can_edit` 投影與草稿可見性閘）。
    /// 固定參數 $1=viewer_id、$2=is_admin、$3=viewer_sees_all_drafts；可選過濾自 $4 起。
    fn build_list_sql(query: &ProtocolQuery) -> String {
        // PI（客人）以研究資料 basic.pi 為準（外部 PI 匯入時與 FK 匯入者不同），fallback FK；委託單位同理
        let mut sql = format!(
            r#"
            SELECT
                p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                p.pi_user_id,
                {pi_name} as pi_name,
                {pi_org} as pi_organization,
                p.start_date, p.end_date, p.created_at,
                NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number,
                p.imported_at,
                COALESCE(
                    p.pi_user_id = $1
                    OR p.study_director_user_id = $1
                    OR EXISTS (SELECT 1 FROM user_protocols up_e
                               WHERE up_e.protocol_id = p.id AND up_e.user_id = $1
                                 AND up_e.role_in_protocol = 'PI')
                    OR $2,
                    false
                ) AS can_edit
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            WHERE p.status != 'DELETED'
              AND (
                  p.status <> 'DRAFT'
                  OR $3
                  OR p.pi_user_id = $1
                  OR p.study_director_user_id = $1
                  OR EXISTS (SELECT 1 FROM user_protocols up_v
                             WHERE up_v.protocol_id = p.id AND up_v.user_id = $1)
              )
            "#,
            pi_name = crate::utils::pi_sql::pi_display_name("u.display_name"),
            pi_org = crate::utils::pi_sql::pi_sponsor_org("u.organization"),
        );
        // R7-P0-2 參數化避免 SQL injection：可選過濾（$1-$3 固定，過濾參數自 $4 起）
        // 與 get_my_protocols 共用組裝器；綁定順序見 list()。
        super::push_optional_protocol_filters(&mut sql, query, 4);
        // 可指派計畫過濾（已核准 + 有 iacuc_no）：常數狀態字串，無需綁參數。
        if query.assignable {
            sql.push_str(super::ASSIGNABLE_PROTOCOL_FILTER);
        }
        sql.push_str(" ORDER BY p.created_at DESC");
        sql
    }

    /// 批量修復缺少 APIG 編號的 Submitted / PreReview 計畫書（就地更新傳入清單）。
    async fn backfill_apig_nos(pool: &PgPool, protocols: &mut [ProtocolListItem]) -> Result<()> {
        let needs_apig_ids: Vec<Uuid> = protocols
            .iter()
            .filter(|p| {
                (p.status == ProtocolStatus::Submitted || p.status == ProtocolStatus::PreReview)
                    && p.iacuc_no
                        .as_ref()
                        .map(|no| !no.starts_with("APIG-"))
                        .unwrap_or(true)
            })
            .map(|p| p.id)
            .collect();
        if needs_apig_ids.is_empty() {
            return Ok(());
        }
        // 批量生成：一次查詢 max seq，然後在記憶體中分配 N 個連續編號
        let apig_nos = Self::generate_apig_nos_batch_pool(pool, needs_apig_ids.len()).await?;
        sqlx::query(
            r#"
            UPDATE protocols SET iacuc_no = d.apig_no, updated_at = NOW()
            FROM UNNEST($1::uuid[], $2::text[]) AS d(id, apig_no)
            WHERE protocols.id = d.id
            "#,
        )
        .bind(&needs_apig_ids)
        .bind(&apig_nos)
        .execute(pool)
        .await?;
        // 更新列表中的編號（避免重新查詢）
        for (id, apig_no) in needs_apig_ids.iter().zip(apig_nos.iter()) {
            if let Some(protocol) = protocols.iter_mut().find(|p| &p.id == id) {
                protocol.iacuc_no = Some(apig_no.clone());
            }
        }
        Ok(())
    }

    /// `get_by_id` 的 Scoped 入口（R75-P4）：handler 必須先持 `Scoped<ProtocolView>`
    /// （已過 `require_protocol_view_access`）才能取單筆，使「忘了授權」變編譯錯誤。
    /// 共用低階 `get_by_id`（仍吃裸 id）保留給已自帶 `Scoped<ProtocolId>` 的 pdf 匯出等內部呼叫。
    pub async fn get_for_view(
        pool: &PgPool,
        scope: access::Scoped<access::ProtocolView>,
    ) -> Result<ProtocolResponse> {
        Self::get_by_id(pool, scope.id()).await
    }

    /// 取得單一計畫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProtocolResponse> {
        let mut protocol = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 如果狀態是 Submitted 或 PreReview 但沒有 APIG 編號，自動生成
        // 根據規則：在計劃被提交審查與核准前，應為 APIG-{ROC}{03}
        if protocol.status == ProtocolStatus::Submitted
            || protocol.status == ProtocolStatus::PreReview
        {
            let needs_apig = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                let apig_no = Self::generate_apig_no_pool(pool).await?;
                protocol = sqlx::query_as::<_, Protocol>(
                    "UPDATE protocols SET iacuc_no = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
                )
                .bind(id)
                .bind(&apig_no)
                .fetch_one(pool)
                .await?;
            }
        }

        // 取得 PI 資訊：以研究資料 basic.pi 為準，fallback FK 使用者（同 list()／
        // my_protocols 等處，DRY）。
        //
        // 🔴 外部 PI（無系統帳號）時 `pi_user_id` 只是匯入者的佔位值——直接查
        // FK 使用者會顯示匯入者本人的姓名/email/單位，而不是真正的外部 PI。
        // 之前這裡沒走 `pi_sql` helper，是本檔唯一漏掉的一處（其餘顯示查詢都已用）。
        let pi_info: Option<(String, String, Option<String>)> = sqlx::query_as(
            sqlx::AssertSqlSafe(format!(
                "SELECT {pi_name}, {pi_email}, {pi_org} FROM protocols p JOIN users u ON u.id = p.pi_user_id WHERE p.id = $1",
                pi_name = crate::utils::pi_sql::pi_display_name("u.display_name"),
                pi_email = crate::utils::pi_sql::pi_email("u.email"),
                pi_org = crate::utils::pi_sql::pi_sponsor_org("u.organization"),
            )),
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let (pi_name, pi_email, pi_organization) = pi_info.unwrap_or_default();

        // 批次查詢計劃負責人(SD，公司內部)與建立者/匯入者顯示名，單次往返（與 PI 客人區分）
        let mut name_lookup_ids = vec![protocol.created_by];
        if let Some(sd_id) = protocol.study_director_user_id {
            name_lookup_ids.push(sd_id);
        }
        let name_rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, display_name FROM users WHERE id = ANY($1)")
                .bind(&name_lookup_ids)
                .fetch_all(pool)
                .await?;
        let sd_name: Option<String> = protocol
            .study_director_user_id
            .and_then(|sd_id| name_rows.iter().find(|r| r.0 == sd_id).map(|r| r.1.clone()));
        let created_by_name: Option<String> = name_rows
            .iter()
            .find(|r| r.0 == protocol.created_by)
            .map(|r| r.1.clone());

        // 獲取獸醫審查指派
        let vet_review = sqlx::query_as::<_, crate::models::VetReviewAssignment>(
            "SELECT * FROM vet_review_assignments WHERE protocol_id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(ProtocolResponse {
            status_display: protocol.status.display_name().to_string(),
            protocol,
            pi_name: Some(pi_name),
            pi_email: Some(pi_email),
            pi_organization,
            sd_name,
            created_by_name,
            vet_review,
            // 由 get_protocol handler 依 current_user 覆寫；其餘呼叫端預設 false。
            can_edit: false,
            can_write_amendment: false,
        })
    }

    /// 更新計畫
    ///
    /// R30-5：version optimistic lock 防 lost update。
    /// R30-29：tx + FOR UPDATE before snapshot + audit-in-tx + 完整 before/after diff。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        scope: access::Scoped<access::ProtocolEdit>,
        req: &UpdateProtocolRequest,
    ) -> Result<Protocol> {
        // R75-P4：吃 `Scoped<ProtocolEdit>`（已過 `require_protocol_edit`）→ 編譯期強制
        // 呼叫端先授權；以下沿用 `id` 的內部邏輯。
        let id = scope.id();
        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖行 + 完整 before snapshot（在同 tx 內，與後續 UPDATE 一致）
        let before: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // import P1：補登中（APPROVED + import_pending）允許編輯 working_content，
        // 不走 amendment；按「完成補登」後 import_pending 清除即恢復鎖定。
        let is_import_pending_edit =
            before.status == ProtocolStatus::Approved && before.import_pending;
        if before.status != ProtocolStatus::Draft
            && before.status != ProtocolStatus::RevisionRequired
            && before.status != ProtocolStatus::PreReviewRevisionRequired
            && before.status != ProtocolStatus::VetRevisionRequired
            && !is_import_pending_edit
        {
            return Err(AppError::BusinessRule(
                "Only draft or revision-required protocols can be edited".to_string(),
            ));
        }

        // ── GLP 屬性（裁定 14，migration 006）─────────────────────────────
        //
        // `protocols.is_glp` 是判定的權威來源。`working_content.basic.is_glp`
        // 仍然是表單內容的一部分，未鎖定時同步到欄位，鎖定後以欄位為準。
        //
        // ⚠️ 為什麼需要獨立欄位：舊寫法直接讀 working_content，而那是**設計上就
        // 可編輯**的內容。舊寫法已經防了同一 request 的繞過（取「更新後生效值」），
        // 但擋不住跨 request：
        //   request 1：只改 is_glp=false，不帶 study_director_user_id
        //              → 底下 `if let Some(sd_id)` 整段不執行，鎖根本沒被檢查
        //   request 2：改 SD（is_glp 已是 false，鎖不觸發）
        // 判定來源可被任意編輯的話，規則寫得再嚴都只是裝飾。
        //
        // **建立後即固定，雙向皆不可變更**（使用者 2026-08-25 裁定）：
        // GLP 不可改成非 GLP，非 GLP 也不可改成 GLP。
        //
        // ⚠️ 為什麼不是「核准後才鎖」——那個版本的有效範圍是空的。
        // 上面 `:1070-1081` 早就規定只有 DRAFT／三種 RevisionRequired／
        // APPROVED+import_pending 能編輯 working_content，所以「已核准且完成補登」
        // 的計畫連 update 都進不去，鎖與不鎖沒有差別。
        //
        // 而繞過路徑正好落在**可編輯**的狀態裡，實測 3 個 request 走得通：
        //   1. 只關掉 is_glp（不帶 SD）→ 下面的 GLP 鎖在 `if let Some(sd_id)` 內，不執行
        //   2. 換 SD（此時已是非 GLP，鎖不觸發）
        //   3. 把 is_glp 改回 true
        // 把判定來源從 JSONB 搬到欄位並不能堵住它——問題不在來源可竄改，
        // 而在「可編輯狀態下 is_glp 本來就能改」。雙向鎖直接斷掉第 1 步。
        //
        // 代價：起草階段勾錯要找管理員改。這是使用者權衡後接受的。
        // admin 例外（裁定 3 的「可繞過但填理由 + 寫稽核」）**尚未實作**——
        // 那需要在 UpdateProtocolRequest 加理由欄位，屬 API 契約變更。
        if let Some(requested) = glp_flag_in_content(req.working_content.as_ref()) {
            if requested != before.is_glp {
                return Err(AppError::BusinessRule(
                    "計畫的 GLP 屬性建立後不可變更（GLP 與非 GLP 皆不可互轉）。如需更正請聯繫管理員。"
                        .into(),
                ));
            }
        }

        // 計劃負責人（SD）變更（選填）：驗證 + 授權（僅執秘/admin 指派他人，其餘限本人）。
        // GLP 鎖：GLP 計劃一旦已指派 SD 即鎖定，不可變更（GLP Study Director 為法規正式角色）；
        // 尚未指派（NULL）時仍可首次指派。
        //
        // 讀 `before.is_glp`（權威欄位）而非 working_content：後者是可編輯的表單內容，
        // 拿它當規則判定來源等於沒有規則。有了上面的雙向鎖，兩者也不可能再分歧。
        if let Some(sd_id) = req.study_director_user_id {
            // ⚠️ update 這條路徑與 create／import 不同：`UpdateProtocolRequest`
            // 沒有 pi_user_id 欄位（PI 不可透過 update 變更），所以只能用
            // `before.pi_user_id`——**而那個值可能是外部 PI 的佔位**（等於建立者，
            // 因為 create 走 `req.pi_user_id.unwrap_or(created_by)`）。
            //
            // 是不是佔位，讀 `before.pi_is_external`（migration 009）—— 這是
            // `create`／`import_approved` 在建立/匯入當下就寫死的權威欄位，
            // 不是每次現查角色回推。
            //
            // 🔴 舊版曾用「`pi_user_id == created_by` 且該使用者**現在**有沒有
            // PI 角色」現查回推，角色是可變的，兩個方向都會判錯：真 PI 事後
            // 失去 PI 角色會被誤判成佔位（fail open，自任 SD 被誤放行）；
            // 佔位建立者事後取得 PI 角色會被誤判成真 PI（fail closed，合法的
            // 自任 SD 被誤擋）。CodeRabbit #26 第 3 輪指出後改為本欄位——
            // 建立/匯入當下的判斷結果一旦寫死，就不會再隨事後的角色異動漂移。
            // 見 `role_change_after_creation_does_not_reopen_real_pi_guard` /
            // `role_change_after_creation_does_not_reblock_placeholder_creator`。
            let effective_pi = if before.pi_is_external {
                None
            } else {
                Some(before.pi_user_id)
            };
            Self::validate_and_authorize_sd(&mut tx, actor, sd_id, effective_pi).await?;
            // rebase 衝突解法（2026-08-25）：本分支原本在這裡從 working_content
            // 重推 is_glp，而 #25（裁定 14）的整個重點就是**不要**那樣做——
            // 判定來源必須是權威欄位 `before.is_glp`。取 main 的版本。
            if before.is_glp
                && before.study_director_user_id.is_some()
                && before.study_director_user_id != Some(sd_id)
            {
                return Err(AppError::BusinessRule(
                    "GLP 計劃的計劃負責人（Study Director）已鎖定，不可變更".into(),
                ));
            }
        }

        // 版本名冊：重選版本 / 升級最新版時更新 source_form_version（trim 空字串→None=不變更）
        let source_form_version = req
            .source_form_version
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        // R30-5：version optimistic lock + version+1
        // $6 = NULL → 跳過版本檢查（向後相容）；命中 0 row → 409 Conflict
        // $7 = NULL → 不變更現有 SD（COALESCE 保留）；$8 = NULL → 不變更版本鍵
        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET
                title = COALESCE($2, title),
                working_content = COALESCE($3, working_content),
                start_date = COALESCE($4, start_date),
                end_date = COALESCE($5, end_date),
                study_director_user_id = COALESCE($7, study_director_user_id),
                source_form_version = COALESCE($8, source_form_version),
                -- ⚠️ 這裡**刻意沒有 is_glp**：它建立後即固定（見上方雙向鎖），
                -- 任何變更請求在到達這裡之前就已被拒。把它放進 UPDATE 只會多開
                -- 一條「有人日後改動繫結順序就悄悄能改」的路。
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1
              AND ($6::INT IS NULL OR version = $6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(req.version)
        .bind(req.study_director_user_id)
        .bind(source_form_version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        // ── SD 變更的專屬稽核事件（裁定 21）─────────────────────────────
        //
        // ⚠️ 為什麼不能只靠下面那個 PROTOCOL_UPDATE：
        // 稽核報表上看不出「這次 update 改的是 SD」——它跟改標題、改日期長得
        // 一模一樣，要靠人去比對 before/after 的 JSON 才知道。而 GLP 稽核會問
        // 「這份計畫的 SD 換過幾次、誰換的」，那個問題現在答不出來。
        //
        // 實測（2026-08-25 正式庫）：`user_activity_logs` 裡 `changed_fields`
        // 真的含 `study_director` 的只有 **1 筆**，其餘 153 筆是整份快照剛好
        // 帶到這個欄位。也就是說現況連「換過幾次」都得靠人工判讀。
        //
        // 比對 before/updated 而非 before/req：`req.study_director_user_id`
        // 為 None 時 UPDATE 走 COALESCE 保留原值，拿 req 判斷會把「沒帶」
        // 誤當成「沒變」——雖然結論相同，但一旦日後改成可清空 SD 就會出錯。
        // 比實際寫入結果最準。
        if before.study_director_user_id != updated.study_director_user_id {
            if let Some(new_sd) = updated.study_director_user_id {
                // 裁定 11：執秘可以自我指派，但稽核要看得出來。
                // 判定用 actor 而非 req——指派者就是這次操作的人。
                let self_assigned = actor.actor_user_id() == Some(new_sd);
                Self::record_activity_tx(
                    &mut tx,
                    actor,
                    id,
                    ProtocolActivityType::SdAssigned,
                    before.study_director_user_id.map(|u| u.to_string()),
                    Some(new_sd.to_string()),
                    None,
                    None,
                    Some(serde_json::json!({
                        "from_user_id": before.study_director_user_id,
                        "to_user_id": new_sd,
                        "assigned_by": actor.actor_user_id(),
                        "self_assigned": self_assigned,
                    })),
                )
                .await?;

                // 🔴 `record_activity_tx` **不寫 `user_activity_logs`**（PR #269
                // Option C，見 history.rs L91-96）——它只落 `protocol_activities`。
                // 少了下面這段，SD 變更在全域稽核軸上仍然只有一筆 PROTOCOL_UPDATE，
                // 裁定 21 等於沒做。而且 `protocol_activities` 不在 HMAC chain 上，
                // 不可竄改的那條軸看不到 SD 換人這件事。
                //
                // 這裡刻意帶 focused diff 而非 `data_diff: None`（reviewer / vet
                // 指派走 None 是因為那兩者本來就沒有 before/after）：SD 的 from→to
                // 就是稽核要問的東西，放進 HMAC 保護的 row 才有意義，
                // 且讓 `changed_fields = ["study_director_user_id"]` 可直接篩選——
                // 正是上面實測指出「整份快照剛好帶到這個欄位」所缺的那個判準。
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "AUP",
                        event_type: event_type_for(ProtocolActivityType::SdAssigned),
                        entity: Some(AuditEntity::new("protocol", id, &updated.title)),
                        data_diff: Some(DataDiff::compute(
                            Some(&SdChangeAudit {
                                study_director_user_id: before.study_director_user_id,
                            }),
                            Some(&SdChangeAudit {
                                study_director_user_id: Some(new_sd),
                            }),
                        )),
                        request_context: None,
                    },
                )
                .await?;
            }
        }

        // protocol_activities + user_activity_logs（同 tx，UPDATED 事件）
        Self::record_activity_tx(
            &mut tx,
            actor,
            id,
            ProtocolActivityType::Updated,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        // Service-driven audit：完整 before/after diff 進 HMAC chain
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_UPDATE",
                entity: Some(AuditEntity::new("protocol", id, &updated.title)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(updated)
    }
}
