//! 資源存取權限檢查（防範 IDOR）
//! 此模組依賴 CurrentUser（middleware 層別）與 AppError，屬於業務邏輯，置於 services/ 層。

use std::marker::PhantomData;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, middleware::CurrentUser, Result};

/// 檢查使用者是否有權存取資源
/// - 資源擁有者（resource_owner_id == current_user.id）可存取
/// - 具備 admin_permission 權限者可存取（如 hr.leave.view_all）
/// - 否則回傳 403 Forbidden
pub fn check_resource_access(
    current_user: &CurrentUser,
    resource_owner_id: Uuid,
    admin_permission: &str,
) -> Result<()> {
    if current_user.id == resource_owner_id || current_user.has_permission(admin_permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此資源".into()))
    }
}

/// SEC-SoD 職權分離守衛：申請 / 登錄 / 建立者不得核准自己送出的紀錄。
///
/// 原本只存在於 `services/equipment/mod.rs`（設備報廢 / 閒置 / 維護驗收三處），
/// R97-1 上移到本模組供 ERP 單據核准共用——同一套系統裡「設備報廢要兩個人、
/// 庫存報廢一個人就行」的落差沒有道理，稽核時也解釋不通。
///
/// 呼叫端負責決定「哪些情境要套用」；本函式只做比對，不含業務判斷。
pub fn assert_not_self_approval(
    submitter_id: Uuid,
    current_user_id: Uuid,
    msg: &str,
) -> Result<()> {
    if submitter_id == current_user_id {
        return Err(AppError::Forbidden(msg.into()));
    }
    Ok(())
}

// ============================================
// 計畫書存取權限檢查
// ============================================

/// 角色列表：具有 view_all 權限的角色
const VIEW_ALL_ROLES: &[&str] = &[
    crate::constants::ROLE_IACUC_CHAIR,
    crate::constants::ROLE_IACUC_STAFF,
    crate::constants::ROLE_VET,
    crate::constants::ROLE_REVIEWER,
];

/// 使用者是否為計畫成員（PI / 委託人 CLIENT）或計劃負責人 SD。
///
/// SD（`study_director_user_id`）為 CO_EDITOR 拆除後的內部協作者後繼，故沿用 CO_EDITOR
/// 原有的「計畫關係人」操作權（如 AI 審查）。
pub async fn is_pi_sd_or_client_member(
    pool: &PgPool,
    protocol_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_protocols
            WHERE protocol_id = $1 AND user_id = $2
              AND role_in_protocol IN ('PI', 'CLIENT')
            UNION
            SELECT 1 FROM protocols
            WHERE id = $1 AND study_director_user_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫的指派審查委員
pub async fn is_assigned_reviewer(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM review_assignments
            WHERE protocol_id = $1 AND reviewer_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫的指派獸醫
pub async fn is_assigned_vet(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM vet_review_assignments
            WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否與計畫有任何關聯（PI / **SD** / Co-Editor / 審查委員 / 獸醫）
///
/// ⚠️ **SD 是 2026-08-25 才補進來的**（CodeRabbit 於 PR #23 指出）。在那之前這個
/// 查詢只看四張表，漏了 `protocols.study_director_user_id`——而 SD 是計畫負責人，
/// 說他「與計畫無關」在語意上就是錯的。
///
/// 為什麼一直沒被發現：`require_protocol_related_access` 的第一行是
/// `has_protocol_view_all` 短路，而 SD 必為 `EXPERIMENT_STAFF`、該角色具
/// `aup.protocol.view_all`，所以實際上走不到這個查詢。**是被掩蓋，不是不存在。**
/// 實測（`tests/api_sd_close_access_guard.rs`）：拿掉 view_all 之後，
/// 只透過 SD 欄位關聯的人立刻拿到 403。
///
/// 補上之後**不會有任何人獲得新權限**（2026-08-25 實測正式庫）：
/// 6 位 SD 中 5 位具 view_all（本來就能存取），剩下 1 位同時是該計畫的 PI
/// （透過 `pi_user_id` 已有關聯）。這個改動移除的是隱性依賴，不是放寬授權。
///
/// 這也讓本函式與 `access.rs` 其餘 13 處已認 SD 的授權檢查一致
/// （`is_protocol_sd`／`can_sign_notice`／補登授權…）——先前只有這一個漏掉。
pub async fn has_any_protocol_role(
    pool: &PgPool,
    protocol_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols
              WHERE id = $1 AND (pi_user_id = $2 OR study_director_user_id = $2)
            UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            UNION SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
            UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 回傳使用者「可存取」的計畫 id 集合（PI / **SD** / 共編成員 / 指派審查委員 / 指派獸醫）。
///
/// 用途：列表 / 報表端點對「無 view_all 權限」的使用者做資料邊界收斂——把此集合
/// 強制 AND 進查詢，避免送空 filter 即讀到跨計畫全量資料（IDOR）。
/// view_all 角色不需呼叫（直接全量）。回傳空 vec 代表使用者不關聯任何計畫。
///
/// ⚠️ SD 於 2026-08-25 一併補上，理由與 [`has_any_protocol_role`] 相同——
/// 兩者是同一個漏洞的兩個面向（一個管「單筆能不能存取」、一個管「列表看得到哪些」）。
/// 只補其中一個會造成「能開單筆、卻不在自己的列表裡」這種更難解釋的狀態。
pub async fn accessible_protocol_ids(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>> {
    let ids: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id AS protocol_id FROM protocols WHERE pi_user_id = $1
        UNION SELECT id AS protocol_id FROM protocols WHERE study_director_user_id = $1
        UNION SELECT protocol_id FROM user_protocols WHERE user_id = $1
        UNION SELECT protocol_id FROM review_assignments WHERE reviewer_id = $1
        UNION SELECT protocol_id FROM vet_review_assignments WHERE vet_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

/// 使用者是否為計畫的 PI（`protocols.pi_user_id` 直接關聯，或 `user_protocols` role='PI'）。
///
/// 不含 CLIENT / CO_EDITOR。用於 amendment PI 寫入授權，以及草稿編輯 / 送出收緊
/// （`can_edit_protocol` / `submit_protocol`，對齊原始 spec §4.1「編輯·提交僅 PI ✓」）。
/// 納入 `pi_user_id` FK 以涵蓋「未建 user_protocols 成員列」的計畫（如匯入流程）。
pub async fn is_protocol_pi(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols WHERE id = $2 AND pi_user_id = $1
            UNION ALL
            SELECT 1 FROM user_protocols
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        )"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫負責人 SD（`study_director_user_id`）。
pub async fn is_study_director(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1 AND study_director_user_id = $2)",
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 是否為任一「已核准」計畫的計畫層級 SD（`protocols.study_director_user_id`）。
///
/// 用於無計畫（外部客戶）銷貨單開立授權（2026-07-20 裁定放寬）：限定已核准狀態，
/// 避免只掛過草稿 / 已結案計畫的人長期保有開單權。
pub async fn is_study_director_of_any_approved(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM protocols WHERE study_director_user_id = $1 \
         AND status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS'))",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 須知簽署授權（PR-B）：計畫 PI（`pi_user_id` 或 `user_protocols` PI 角色）或 SD。
pub async fn can_sign_notice(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols
            WHERE id = $1 AND (pi_user_id = $2 OR study_director_user_id = $2)
            UNION
            SELECT 1 FROM user_protocols
            WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol = 'PI'
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否與計畫有關聯（any role in user_protocols）
pub async fn has_protocol_membership(
    pool: &PgPool,
    protocol_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_protocols
            WHERE user_id = $1 AND protocol_id = $2
        )"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫審查委員或獸醫（用於 review comment 權限）
pub async fn is_reviewer_or_vet(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
            UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為 amendment 的指派審查員
pub async fn is_amendment_reviewer(
    pool: &PgPool,
    amendment_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM amendment_review_assignments
            WHERE amendment_id = $1 AND reviewer_id = $2
        )"#,
    )
    .bind(amendment_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否有 view_all 計畫權限（含角色檢查）
pub fn has_protocol_view_all(current_user: &CurrentUser) -> bool {
    current_user.has_permission("aup.protocol.view_all")
        || current_user
            .roles
            .iter()
            .any(|r| VIEW_ALL_ROLES.contains(&r.as_str()))
}

/// 檢查計畫查看權限（view_all 或有任何計畫角色），失敗回傳 403
///
/// HIGH-03: 原本最多執行 3 次獨立 DB 查詢，改為單一 4-way UNION EXISTS 查詢。
pub async fn require_protocol_view_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
    pi_user_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) || current_user.id == pi_user_id {
        return Ok(());
    }
    let (has_access,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
            UNION
            SELECT 1 FROM user_protocols
            WHERE protocol_id = $1 AND user_id = $2
              AND role_in_protocol IN ('PI', 'CLIENT')
            UNION
            SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
            UNION
            SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(current_user.id)
    .fetch_one(pool)
    .await?;

    if has_access {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You don't have permission to view this protocol".into(),
        ))
    }
}

/// 檢查計畫查看權限（不查 pi_user_id，用於不知道 PI 的情境）
pub async fn require_protocol_related_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) {
        return Ok(());
    }
    if has_any_protocol_role(pool, protocol_id, current_user.id).await? {
        return Ok(());
    }
    Err(AppError::Forbidden(
        "You don't have permission to access this protocol".into(),
    ))
}

// ============================================
// R75-P4 Phase 2：typed scope wrapper（編譯期強制物件層授權）
// ============================================

/// 計畫資源 id 的型別標記（見 `Scoped`）。
pub struct ProtocolId;

/// 已通過物件層授權的資源 id 證明。
///
/// 唯一建構路徑（如 `Scoped::<ProtocolId>::authorize`）會先跑對應的授權檢查，
/// 故「持有 `Scoped<T>` ⟺ 呼叫端已授權」在編譯期成立。下游服務函式改吃
/// `Scoped<T>`（而非裸 `Uuid`）即可強制呼叫端先授權——漏檢查直接編譯不過。
/// 欄位私有且無其他建構子，無法繞過授權產生證明。
pub struct Scoped<T> {
    id: Uuid,
    _marker: PhantomData<T>,
}

impl<T> Scoped<T> {
    /// 取出已授權的資源 id（供下游查詢使用）。
    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl Scoped<ProtocolId> {
    /// 唯一建構路徑：跑 `require_protocol_related_access`，通過才產出證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<Self> {
        require_protocol_related_access(pool, current_user, protocol_id).await?;
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }
}

/// 計畫「檢視」授權的型別標記（見 `Scoped`）。對應 `require_protocol_view_access`
/// （4-way：PI 短路 + pi_user_id / user_protocols PI·CLIENT·CO_EDITOR / review·vet 指派），
/// 語意比 `ProtocolId`（related）更貼近「可檢視單筆計畫」。
pub struct ProtocolView;

impl Scoped<ProtocolView> {
    /// 唯一建構路徑：先取 `pi_user_id`（兼存在性檢查 → 404），再跑
    /// `require_protocol_view_access`，通過才產出檢視證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<Self> {
        let pi_user_id: Uuid = sqlx::query_scalar("SELECT pi_user_id FROM protocols WHERE id = $1")
            .bind(protocol_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;
        require_protocol_view_access(pool, current_user, protocol_id, pi_user_id).await?;
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }
}

/// 計畫「編輯」授權的型別標記（見 `Scoped`）。對應 `require_protocol_edit`
/// （`can_edit_protocol`：`aup.protocol.edit` + 計畫關聯 / PI·co-editor / 補登中建立者·SD·管理者）。
pub struct ProtocolEdit;

/// `can_edit_protocol` 的 Result 版守衛：不可編輯時回 `Forbidden`（供 `Scoped<ProtocolEdit>` 與
/// handler 共用，避免散落的 `if !can_edit { Forbidden }` 樣板）。
pub async fn require_protocol_edit(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if can_edit_protocol(pool, current_user, protocol_id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You don't have permission to edit this protocol".into(),
        ))
    }
}

/// 純 SD 指派授權（`update_protocol` 不含內容變更時）。
///
/// 執秘（`IACUC_STAFF`）/ 管理員可指派他人為 SD（協調動作，對齊 `validate_and_authorize_sd`），
/// 或本身即可編輯者（PI / SD）。SD 值本身「誰可被指派 / 誰可指派」由 service 端
/// `validate_and_authorize_sd` 進一步把關（含 self-assign 與 EXPERIMENT_STAFF 合法性）。
/// 此守衛僅確保「無內容編輯權的執秘 / admin」能通過外層 `Scoped<ProtocolEdit>` 閘以完成 SD 指派，
/// 而內容編輯（標題 / 表單 / 日期）仍須 `can_edit_protocol`（PI / SD / admin）。
pub async fn require_protocol_sd_assign(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if current_user.is_admin()
        || current_user
            .roles
            .iter()
            .any(|r| r == crate::constants::ROLE_IACUC_STAFF)
        || can_edit_protocol(pool, current_user, protocol_id).await?
    {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You don't have permission to assign the study director of this protocol".into(),
        ))
    }
}

impl Scoped<ProtocolEdit> {
    /// 唯一建構路徑：跑 `require_protocol_edit`，通過才產出編輯證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<Self> {
        require_protocol_edit(pool, current_user, protocol_id).await?;
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }

    /// `update_protocol` 專用的欄位感知授權：
    /// - `touches_content == true`（改標題 / 表單 / 日期）→ 須 `can_edit_protocol`（PI / SD / admin）。
    /// - `touches_content == false`（純 SD 指派 / 僅 version）→ 允許執秘 / admin 協調指派 SD。
    pub async fn authorize_update(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
        touches_content: bool,
    ) -> Result<Self> {
        if touches_content {
            require_protocol_edit(pool, current_user, protocol_id).await?;
        } else {
            require_protocol_sd_assign(pool, current_user, protocol_id).await?;
        }
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }
}

/// 動物「讀取」授權的型別標記（見 `Scoped`）。對應 `require_animal_read_access`
/// （放寬到具 `animal.animal.view_all` 的內部試驗人員可跨計畫讀取）。
pub struct AnimalRead;

/// 動物「寫入 / 計畫綁定」授權的型別標記（見 `Scoped`）。對應 `require_animal_access`
/// （限動物所屬計畫成員 + view_all 角色，較讀取嚴格）。
pub struct AnimalWrite;

impl Scoped<AnimalRead> {
    /// 唯一建構路徑：跑 `require_animal_read_access`，通過才產出讀取證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        animal_id: Uuid,
    ) -> Result<Self> {
        require_animal_read_access(pool, current_user, animal_id).await?;
        Ok(Self {
            id: animal_id,
            _marker: PhantomData,
        })
    }

    /// 由**觀察紀錄 ID** 建立讀取證明（R94-4）。
    ///
    /// 存在理由：呼叫端過去必須自己跑「`get_observation_animal_id` 解析 → 再授權」兩步，
    /// **漏掉第二步會造成跨計畫讀取（IDOR）且編譯得過**。本方法把兩步封在同一個入口內。
    ///
    /// ⚠️ **這是慣例保證，不是型別保證**：`get_observation_animal_id` 目前仍是 `pub`
    /// （`create_care_record` 的歸屬比對需要它），所以「解析完忘了授權」仍然寫得出來也編譯得過。
    /// 要升級成真正的結構性保證，得把 resolver 收私有、另開一支歸屬比對專用 helper。
    ///
    /// ⚠️ **呼叫時務必顯式寫出 turbofish**（`Scoped::<AnimalRead>::from_observation`）。
    /// 本型別與 `Scoped<AnimalWrite>` 的同名方法簽章完全相同，只差型別參數；若省略
    /// turbofish，型別推論會從**期望型別**倒推變體——當下游 service 收 `Scoped<AnimalRead>`
    /// 時就會自動選到較弱的讀取授權。
    ///
    /// 回傳的 `id()` 仍是 **animal_id**（與 `authorize` 一致），故下游吃 `Scoped<AnimalRead>`
    /// 的 service 不需改動。
    pub async fn from_observation(
        pool: &PgPool,
        current_user: &CurrentUser,
        observation_id: Uuid,
    ) -> Result<Self> {
        let animal_id = get_observation_animal_id(pool, observation_id).await?;
        Self::authorize(pool, current_user, animal_id).await
    }
}

impl Scoped<AnimalWrite> {
    /// 唯一建構路徑：跑 `require_animal_access`，通過才產出寫入證明。
    /// 與 `Scoped<AnimalRead>` 為不同型別 → 讀取證明無法傳入吃寫入證明的函式，
    /// 防止唯讀使用者觸發寫入路徑（型別層阻擋權限提升）。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        animal_id: Uuid,
    ) -> Result<Self> {
        require_animal_access(pool, current_user, animal_id).await?;
        Ok(Self {
            id: animal_id,
            _marker: PhantomData,
        })
    }

    /// 由**觀察紀錄 ID** 建立寫入證明（R94-4）。理由同
    /// `Scoped::<AnimalRead>::from_observation`，但走較嚴格的 `require_animal_access`
    /// （限動物所屬計畫成員 + view_all 角色）。
    ///
    /// ⚠️ 讀與寫刻意各留一條入口、不合併：兩者的授權判準不同
    /// （讀放行具 `animal.animal.view_all` 的跨計畫人員，寫不放行），
    /// 合併成單一入口會讓呼叫端有機會挑錯強度。
    pub async fn from_observation(
        pool: &PgPool,
        current_user: &CurrentUser,
        observation_id: Uuid,
    ) -> Result<Self> {
        let animal_id = get_observation_animal_id(pool, observation_id).await?;
        Self::authorize(pool, current_user, animal_id).await
    }
}

/// 須知簽署授權的型別標記（見 `Scoped`）。對應 `can_sign_notice`（計畫 PI / SD）。
pub struct NoticeSign;

/// `can_sign_notice` 的 Result 版守衛：無權時回 `Forbidden`（供 `Scoped<NoticeSign>` 與
/// handler 共用，避免散落的 `if !can_sign_notice { Forbidden }` 樣板）。
pub async fn require_sign_notice(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if can_sign_notice(pool, protocol_id, current_user.id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "僅計畫 PI 或計劃負責人(SD) 可簽署申請須知".to_string(),
        ))
    }
}

impl Scoped<NoticeSign> {
    /// 唯一建構路徑：跑 `require_sign_notice`，通過才產出簽署證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<Self> {
        require_sign_notice(pool, current_user, protocol_id).await?;
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }
}

/// 變更申請「PI 寫入」授權的型別標記（見 `Scoped`）。對應 amendment 建立 / 更新 / 提交
/// 守衛：管理者短路，否則須為計畫 PI（`user_protocols` PI 角色）。
pub struct AmendmentWrite;

/// 是否可建立 / 更新 / 提交此計畫的變更申請：admin 或計畫 PI（沿用 `is_protocol_pi`，
/// 含 `user_protocols` 成員 PI）。與 `can_edit_protocol` 不同——不含 SD／補登管理者，
/// 修正案寫入權收得比一般編輯更緊。供 `require_amendment_write` 與
/// `ProtocolResponse.can_write_amendment`（前端按鈕 gating）共用同一權威判斷。
pub async fn can_write_amendment(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<bool> {
    if current_user.is_admin() {
        return Ok(true);
    }
    is_protocol_pi(pool, protocol_id, current_user.id).await
}

/// `can_write_amendment` 的 Result 版守衛：不可寫入時回 `Forbidden`（供
/// `Scoped<AmendmentWrite>` 使用）。
pub async fn require_amendment_write(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if can_write_amendment(pool, current_user, protocol_id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "僅計畫主持人 (PI) 可建立 / 更新 / 提交變更申請".to_string(),
        ))
    }
}

impl Scoped<AmendmentWrite> {
    /// 唯一建構路徑：跑 `require_amendment_write`，通過才產出寫入證明。
    pub async fn authorize(
        pool: &PgPool,
        current_user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<Self> {
        require_amendment_write(pool, current_user, protocol_id).await?;
        Ok(Self {
            id: protocol_id,
            _marker: PhantomData,
        })
    }
}

/// 計畫內容編輯授權（防 IDOR）。
///
/// 草稿編輯權收緊為「計畫關係人」，對齊原始 spec §4.1（編輯草稿僅 PI ✓、CLIENT/執秘 ✗）。
/// 允許條件（任一）：
/// 1. 系統管理員。
/// 2. 計畫 PI（`pi_user_id` 或 `user_protocols` role='PI'）。
/// 3. 計劃負責人 SD（`study_director_user_id`）。
/// 4. 補登中計劃的 建立者 / 負責人(SD) / 管理者 / `import_approved` 權限者。
///
/// 不再因 `aup.protocol.view_all` / `aup.protocol.edit` 權限或 CLIENT·CO_EDITOR 成員身分放行
/// → 執秘（`IACUC_STAFF`）與 CLIENT 對草稿唯讀。
pub async fn can_edit_protocol(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<bool> {
    if current_user.is_admin() {
        return Ok(true);
    }
    if is_protocol_pi(pool, protocol_id, current_user.id).await? {
        return Ok(true);
    }
    // §5.4：SD（計劃負責人）對自己負責的計劃取得編輯權（staff 新建計劃 = 只設 SD）。
    if is_study_director(pool, protocol_id, current_user.id).await? {
        return Ok(true);
    }
    can_manage_import_pending(pool, protocol_id, current_user).await
}

// ============================================
// 動物層級存取控制（C2: 防範動物醫療記錄 IDOR）
// ============================================

/// 取得動物所屬的 protocol_id。
///
/// `animals` 表並無 `protocol_id` 欄位，動物與計畫的關聯一律以 `iacuc_no` 表示。
/// 此處經 `animals.iacuc_no = protocols.iacuc_no` 解析出 `protocols.id`
/// （與 `services/animal/core/query.rs` 既有的 join pattern 一致；`protocols.iacuc_no`
/// 為 UNIQUE，故至多一筆）。
///
/// 動物不存在、未指派計畫（`iacuc_no` 為 NULL）或查無對應 protocol 時回傳 `NotFound`，
/// 使 `require_animal_access` 對非 view_all 使用者 fail-closed 拒絕存取。
pub async fn get_animal_protocol_id(pool: &PgPool, animal_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT pr.id FROM animals a \
         JOIN protocols pr ON a.iacuc_no = pr.iacuc_no \
         WHERE a.id = $1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Animal not found".into()))
}

/// 動物存在性檢查：不存在（或已軟刪除）一律回 `NotFound("Animal not found")`。
///
/// 供 `require_animal_access` / `require_animal_read_access` 在「全場可存取」分支先行呼叫，
/// 使「查無此豬」對所有角色（含 view_all）一致回 404，而非 view_all 短路後由下游查詢回
/// 空集合 / 200（R70 follow-up：補齊 view_all 角色對不存在動物的 404 行為）。
///
/// 訊息刻意與非 view_all 分支的 `get_animal_protocol_id`（同回 `"Animal not found"`）對齊，
/// 避免「同一端點因角色不同回不同錯誤字串」成為角色探測 / IDOR probe 訊號；同時與本模組
/// 其餘 NotFound（`"Surgery not found"` 等）的英文慣例一致。
async fn require_animal_exists(pool: &PgPool, animal_id: Uuid) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(AppError::NotFound("Animal not found".into()))
    }
}

/// 是否可「跨計畫讀取」動物資料：真 view_all 角色（VET/REVIEWER/IACUC）
/// 或具 `animal.animal.view_all` 權限者（含 EXPERIMENT_STAFF / INTERN）。
///
/// 與 `has_protocol_view_all` 的差異：多納入「具 animal view_all 權限但非 view_all 角色」
/// 的內部試驗人員，使其可**讀取**全場動物紀錄（R70 follow-up）；寫入仍走
/// `require_animal_access`（限自己計畫），故跨計畫放寬僅及於讀取。
fn has_animal_view_all(current_user: &CurrentUser) -> bool {
    has_protocol_view_all(current_user) || current_user.has_permission("animal.animal.view_all")
}

/// 動物「寫入 / 計畫綁定」存取守衛（防 IDOR）。
///
/// 先驗動物存在（查無此豬 → 404）；view_all 角色（IACUC Chair/Staff/Vet/Reviewer）放行；
/// 其餘須為動物所屬計畫成員。用於計畫綁定紀錄的寫入（手術 / 犧牲 / 病理 / 照護 / 獸醫單 /
/// 轉讓等）與動物本體編輯 / 刪除。EXPERIMENT_STAFF 雖可跨計畫讀取，寫入仍限自己計畫。
pub async fn require_animal_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    animal_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) {
        return require_animal_exists(pool, animal_id).await;
    }
    let protocol_id = get_animal_protocol_id(pool, animal_id).await?;
    require_protocol_related_access(pool, current_user, protocol_id).await
}

/// 動物「讀取」存取守衛（防 IDOR）。
///
/// 與 `require_animal_access` 的差異：放行範圍多納入具 `animal.animal.view_all` 權限的
/// 內部試驗人員（EXPERIMENT_STAFF / INTERN），使其可跨計畫**讀取**動物紀錄（含基礎紀錄
/// 與計畫綁定紀錄的讀取端點）；PI / CLIENT 等僅 `view_project` 者仍限自己計畫（防跨計畫讀取）。
///
/// 一律先驗動物存在（查無此豬 → 404）。亦用於基礎紀錄（體重 / 疫苗 / 血檢 / 觀察 / 猝死）的
/// 寫入端點——EXPERIMENT_STAFF 具 `animal.animal.view_all`，故對未指派計畫的動物亦可登錄
/// 基礎紀錄（R70-3 免計畫紀錄放寬）。呼叫前需先通過 `require_permission!()` 把關角色讀寫權。
pub async fn require_animal_read_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    animal_id: Uuid,
) -> Result<()> {
    if has_animal_view_all(current_user) {
        return require_animal_exists(pool, animal_id).await;
    }
    let protocol_id = get_animal_protocol_id(pool, animal_id).await?;
    require_protocol_related_access(pool, current_user, protocol_id).await
}

/// 獸醫巡場報告（vet patrol）讀取授權（R75-5）。
///
/// 巡場為**全場內部福利文件**（`vet_patrol_reports` 無 protocol 欄、報告本質跨多動物，
/// 無法逐筆 scope），故以「角色身分」把關而非物件擁有權：限內部監督 / 實驗人員
/// （具 animal view_all：VET / REVIEWER / IACUC_CHAIR / EXPERIMENT_STAFF / INTERN / QAU）。
///
/// 排除僅 `animal.animal.view_project` 的外部 CLIENT / PI——原以 `animal.record.view`
/// 把關時，外部 CLIENT/PI 亦持該權，可跨客戶讀全場巡場觀察 + 醫療照片（R75-5 IDOR）。
///
/// 原本另放行全域 `STUDY_DIRECTOR` 角色，已隨該角色一併移除：SD 只存在於
/// 「該計畫的 SD」（`protocols.study_director_user_id`），而 SD 一律自
/// `EXPERIMENT_STAFF` 指派（`validate_and_authorize_sd`），本來就走
/// `has_animal_view_all` 通過，不需要另一條旁路。
pub fn require_vet_patrol_view(current_user: &CurrentUser) -> Result<()> {
    if has_animal_view_all(current_user) {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權檢視獸醫巡場報告".into()))
    }
}

/// 透過觀察記錄 ID 取得 animal_id。
///
/// ⚠️ **刻意不加 `AND deleted_at IS NULL`**——本函式是 **IDOR 守衛**（反查歸屬以驗證
/// 計畫存取權），不是存在性檢查。已軟刪除紀錄的稽核軌跡仍須可讀：
/// `get_observation_versions` 正是先呼叫本函式授權、再查 `record_versions` 表，
/// 而後者（`services/animal/core/query.rs`）不受 `animal_observations.deleted_at` 影響。
/// 加上過濾會讓「已刪紀錄的版本歷史」變成 404，等於砍掉軟刪除的稽核用途。
///
/// 這與 surgery 一致：`get_surgery_versions` 走 `AnimalSurgeryService::get_by_id`
/// （`SELECT * FROM animal_surgeries WHERE id = $1`，同樣不過濾），已刪手術紀錄的版本
/// 歷史也查得到。**需要擋掉已刪紀錄的地方由 service 層各自負責**——例如
/// `AnimalObservationService::get_by_id` 就有過濾，故 `GET /observations/{id}` 仍回 404。
///
/// 📌 2026-08-11 曾一度加上此過濾（誤判為「七個 resolver 中唯一漏掉的不一致」），
/// 經 CodeAnt review 指出「改變了每個呼叫端的可觀察契約」後查證撤銷。
/// 教訓：改共用守衛前要走過**所有**呼叫端的下游，不能用單一路徑的觀察推論全部。
pub async fn get_observation_animal_id(pool: &PgPool, observation_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT animal_id FROM animal_observations WHERE id = $1")
        .bind(observation_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Observation not found".into()))
}

/// 透過照護紀錄 ID 取得 animal_id（care_medication_records → observation/surgery → animal）
pub async fn get_care_record_animal_id(pool: &PgPool, care_record_id: Uuid) -> Result<Uuid> {
    let animal_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(
            (SELECT animal_id FROM animal_observations WHERE id = c.record_id LIMIT 1),
            (SELECT animal_id FROM animal_surgeries WHERE id = c.record_id LIMIT 1)
        )
        FROM care_medication_records c
        WHERE c.id = $1
        "#,
    )
    .bind(care_record_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    animal_id.ok_or_else(|| AppError::NotFound("Care record not found".into()))
}

/// 透過獸醫建議紀錄 ID 取得 animal_id（R26-11 service-layer authz 必備）
pub async fn get_vet_advice_record_animal_id(
    pool: &PgPool,
    vet_advice_record_id: Uuid,
) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT animal_id FROM animal_vet_advice_records WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vet_advice_record_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Vet advice record not found".into()))
}

/// 透過手術紀錄 ID 取得 animal_id（service-layer IDOR 守衛用）
pub async fn get_surgery_animal_id(pool: &PgPool, surgery_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT animal_id FROM animal_surgeries WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(surgery_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Surgery not found".into()))
}

/// 透過體重紀錄 ID 取得 animal_id
pub async fn get_weight_animal_id(pool: &PgPool, weight_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT animal_id FROM animal_weights WHERE id = $1 AND deleted_at IS NULL")
        .bind(weight_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Weight record not found".into()))
}

/// 透過疫苗紀錄 ID 取得 animal_id
pub async fn get_vaccination_animal_id(pool: &PgPool, vaccination_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT animal_id FROM animal_vaccinations WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vaccination_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Vaccination record not found".into()))
}

/// 透過血液檢查紀錄 ID 取得 animal_id
pub async fn get_blood_test_animal_id(pool: &PgPool, blood_test_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT animal_id FROM animal_blood_tests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(blood_test_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Blood test not found".into()))
}

/// 計畫前置需求守衛：確認動物已指派計畫（iacuc_no 非 NULL）。
///
/// 用於建立「需計畫」紀錄前（手術 / 犧牲採樣 / 疼痛評估 / 病理報告 / 試驗性觀察），
/// 拒絕對尚未指派 AUP 的動物建立上述實驗性紀錄。
/// 僅擋「建立」操作，不影響既有資料讀取。
pub async fn require_animal_has_protocol(pool: &PgPool, animal_id: Uuid) -> Result<()> {
    let row: Option<Option<String>> =
        sqlx::query_scalar("SELECT iacuc_no FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_optional(pool)
            .await?;

    match row {
        None => Err(AppError::NotFound("動物不存在".into())),
        Some(None) => Err(AppError::BusinessRule(
            "此動物尚未指派計畫（AUP），無法建立手術、犧牲採樣、疼痛評估、病理報告或試驗性觀察紀錄"
                .into(),
        )),
        Some(Some(_)) => Ok(()),
    }
}

/// 檢查使用者是否有權限存取特定計畫（透過 IACUC 編號），用於 PDF 匯出等。
/// admin permission `animal.export.medical` 加上計畫成員才可通過。
pub async fn require_iacuc_protocol_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    iacuc_no: &str,
) -> Result<Uuid> {
    // CSO-r2 follow-up：依 iacuc_no 查找（原誤用 protocol_no 欄位，導致帶合法
    // iacuc_no 一律 NotFound，連既有 export-pdf 專案匯出亦受影響）。
    let protocol_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM protocols WHERE iacuc_no = $1")
            .bind(iacuc_no)
            .fetch_optional(pool)
            .await?;

    let protocol_id = protocol_id
        .ok_or_else(|| AppError::NotFound(format!("Protocol '{}' not found", iacuc_no)))?;

    if has_protocol_view_all(current_user) {
        return Ok(protocol_id);
    }
    require_protocol_related_access(pool, current_user, protocol_id).await?;
    Ok(protocol_id)
}

// ============================================
// IACUC 行政方身分判定
// ============================================

/// IACUC 行政方（執行秘書 / 主席），或管理員。
///
/// 抽出來的理由是同一組判斷在 `handlers/protocol/ai_review.rs` 重複四次
/// （`RULES_BACKEND.md` §3：同一權限檢查 ≥2 處 → `services/access.rs`），
/// 而那四處**都漏了 legacy admin**。
///
/// ⚠️ **一律用 [`CurrentUser::is_admin`] 判管理員，不要自己比對 `ROLE_SYSTEM_ADMIN`。**
/// `constants.rs` 有 `SYSTEM_ADMIN` 與 `admin` 兩個代碼，但 `roles` 表**只有 `admin`**
/// （2026-08-26 實查正式庫 16 個角色 + `backend/migrations/` 全目錄 0 命中）。
/// 單獨比對 `ROLE_SYSTEM_ADMIN` 的分支在任何從 migration 建起來的部署上都恆為 false。
pub fn is_iacuc_staff_or_chair(current_user: &CurrentUser) -> bool {
    current_user.is_admin()
        || current_user.roles.iter().any(|r| {
            [
                crate::constants::ROLE_IACUC_STAFF,
                crate::constants::ROLE_IACUC_CHAIR,
            ]
            .contains(&r.as_str())
        })
}

/// 僅 IACUC 執行秘書（**不含主席**），或管理員。
///
/// 與 [`is_iacuc_staff_or_chair`] 刻意分開：`ai_review.rs::check_protocol_owner`
/// 的原始清單就沒有主席。本次只補管理員 fallback，**不改變主席的有無**
/// ——那是行為變更，需另外裁定。
pub fn is_iacuc_staff(current_user: &CurrentUser) -> bool {
    current_user.is_admin() || current_user.has_role(crate::constants::ROLE_IACUC_STAFF)
}

// ============================================
// 設備驗收存取權限檢查
// ============================================

/// 維修保養紀錄驗收權限：`equipment.maintenance.review` 或 `equipment.manage`
/// 與 `EquipmentService::review_maintenance_record` 一致，用於 sign handler。
pub fn require_equipment_review(current_user: &CurrentUser) -> Result<()> {
    if current_user.has_permission("equipment.maintenance.review")
        || current_user.has_permission("equipment.manage")
    {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權驗收維修保養紀錄".into()))
    }
}

/// 設備管理權限：`equipment.manage`（用於 disposal applicant sign）
/// 與 `EquipmentService::create_disposal` 的 `check_manage_permission` 一致。
pub fn require_equipment_manage(current_user: &CurrentUser) -> Result<()> {
    if current_user.has_permission("equipment.manage") {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權管理設備".into()))
    }
}

/// 設備報廢核准權限：`equipment.disposal.approve` 或 `equipment.manage`
/// 與 `EquipmentService::approve_disposal` 一致，用於 sign handler。
pub fn require_equipment_disposal_approve(current_user: &CurrentUser) -> Result<()> {
    if current_user.has_permission("equipment.disposal.approve")
        || current_user.has_permission("equipment.manage")
    {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權核准設備報廢".into()))
    }
}

/// 通知路由規則管理：限系統管理員（admin only）。
/// 集中於 access 層，避免 handler 自行判斷權限（CLAUDE.md 分層規範）。
pub fn require_notification_routing_manage(current_user: &CurrentUser) -> Result<()> {
    if current_user.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden("僅限系統管理員操作通知路由".into()))
    }
}

// ============================================
// 角色指派授權（SEC-PRIV 共用守衛，CSO-r3 #2）
// ============================================

/// 驗證 actor 是否有權指派 `role_ids` 中的「管理員層級」角色（SYSTEM_ADMIN / legacy admin）
/// 或 DIRECTOR 角色。
///
/// 與 `UserService::validate_role_assignment` 的管理員提權防護一致，供「不經 UserService」
/// 的角色指派路徑（如 invitation 邀請建立 / 接受）共用，防止 SEC-PRIV (CSO-r2 #1) 守衛
/// 被旁路繞過（CSO-r3 #2）：
/// - 指派 SYSTEM_ADMIN：actor 必須本身為 SYSTEM_ADMIN。
/// - 指派 legacy `admin`（= 全權限 admin，見 `middleware::Claims::is_admin`）：
///   actor 必須為管理員層級（SYSTEM_ADMIN 或 legacy admin）。
/// - 指派 DIRECTOR：actor 必須為管理員層級，或本身已持有 DIRECTOR。
///   ⚠️ 2026-08-13 補：`ADMIN_STAFF` 持有 `admin.user.edit`，若不擋 DIRECTOR 指派，
///   等於任何行政人員都能把終審簽核權（DIRECTOR）指派給任意使用者、包括自己
///   ——這條防護補的正是這個缺口，設計討論詳見組織/人事授權設計備忘。
///   自我指派（DIRECTOR 把 DIRECTOR 指派給自己）目前**未擋**：本函式簽章沒有
///   target_user_id，擋自我指派需要另外把目標使用者 id 穿進來，屬於後續獨立變更。
///
/// `actor_user_id == None` 代表系統觸發（seed / provisioning），受信任放行。
/// 本函式僅檢查管理員層級與 DIRECTOR 授權，**不**檢查角色是否存在（由呼叫端自行驗證）。
pub async fn require_authority_to_assign_roles(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    role_ids: &[Uuid],
) -> Result<()> {
    if role_ids.is_empty() {
        return Ok(());
    }

    let admin_tier_codes = &[
        crate::constants::ROLE_SYSTEM_ADMIN,
        crate::constants::ROLE_ADMIN_LEGACY,
    ][..];
    let guarded_codes = &[
        crate::constants::ROLE_SYSTEM_ADMIN,
        crate::constants::ROLE_ADMIN_LEGACY,
        crate::constants::ROLE_DIRECTOR,
    ][..];

    let assigned_guarded_codes: Vec<String> =
        sqlx::query_scalar("SELECT code FROM roles WHERE id = ANY($1) AND code = ANY($2)")
            .bind(role_ids)
            .bind(guarded_codes)
            .fetch_all(pool)
            .await?;

    let assigns_system_admin = assigned_guarded_codes
        .iter()
        .any(|c| c == crate::constants::ROLE_SYSTEM_ADMIN);
    let assigns_legacy_admin = assigned_guarded_codes
        .iter()
        .any(|c| c == crate::constants::ROLE_ADMIN_LEGACY);
    let assigns_director = assigned_guarded_codes
        .iter()
        .any(|c| c == crate::constants::ROLE_DIRECTOR);

    if !assigns_system_admin && !assigns_legacy_admin && !assigns_director {
        return Ok(());
    }

    // System actor（None）受信任，跳過 actor 角色檢查。
    let Some(uid) = actor_user_id else {
        return Ok(());
    };

    let actor_guarded_codes: Vec<String> = sqlx::query_scalar(
        r#"SELECT r.code FROM user_roles ur
           INNER JOIN roles r ON ur.role_id = r.id
           WHERE ur.user_id = $1 AND r.code = ANY($2)"#,
    )
    .bind(uid)
    .bind(guarded_codes)
    .fetch_all(pool)
    .await?;

    let actor_is_system_admin = actor_guarded_codes
        .iter()
        .any(|c| c == crate::constants::ROLE_SYSTEM_ADMIN);
    let actor_is_admin_tier = actor_guarded_codes
        .iter()
        .any(|c| admin_tier_codes.contains(&c.as_str()));
    let actor_is_director = actor_guarded_codes
        .iter()
        .any(|c| c == crate::constants::ROLE_DIRECTOR);

    if assigns_system_admin && !actor_is_system_admin {
        return Err(AppError::Forbidden(
            "僅 SYSTEM_ADMIN 可指派 SYSTEM_ADMIN 角色".to_string(),
        ));
    }
    if assigns_legacy_admin && !actor_is_admin_tier {
        return Err(AppError::Forbidden(
            "僅管理員可指派 admin（系統管理員）角色".to_string(),
        ));
    }
    if assigns_director && !actor_is_admin_tier && !actor_is_director {
        return Err(AppError::Forbidden(
            "僅管理員或負責人（DIRECTOR）可指派 DIRECTOR 角色".to_string(),
        ));
    }
    Ok(())
}

// ============================================
// 匯入補登授權（import P1）
// ============================================

/// import P1：補登中（import_pending）計劃的「編輯內容 / 完成補登」授權。
///
/// 允許：建立者（created_by）OR 計劃負責人 SD（study_director_user_id）OR
/// 管理者（admin / 具 `aup.protocol.import_approved` 權限）。
/// 僅對 import_pending=true 的計劃為真；其餘一律 false（走正常編輯授權）。
pub async fn can_manage_import_pending(
    pool: &PgPool,
    protocol_id: Uuid,
    user: &CurrentUser,
) -> Result<bool> {
    let row: Option<(bool, Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT import_pending, created_by, study_director_user_id FROM protocols WHERE id = $1",
    )
    .bind(protocol_id)
    .fetch_optional(pool)
    .await?;

    let Some((import_pending, created_by, sd)) = row else {
        return Ok(false);
    };
    if !import_pending {
        return Ok(false);
    }
    Ok(created_by == user.id
        || sd == Some(user.id)
        || user.is_admin()
        || user.has_permission("aup.protocol.import_approved"))
}

/// 補登歷史變更授權（P6）：限計劃負責人 SD（study_director_user_id）或管理者，
/// 且計劃須為「匯入計劃」（imported_at 非 NULL）。正常計劃 / 非 SD 一律 false。
///
/// 與 `can_manage_import_pending` 的差異：(a) 不含 created_by 與一般 import_approved 權限
/// 持有者 — 使用者明定「由計劃負責人補登」；(b) 要求 imported_at 非 NULL **且**
/// import_pending=false（補登歷史變更發生在 finalize_import 之後）。
pub async fn can_backfill_historical_amendment(
    pool: &PgPool,
    protocol_id: Uuid,
    user: &CurrentUser,
) -> Result<bool> {
    let row: Option<(bool, bool, Option<Uuid>)> = sqlx::query_as(
        "SELECT imported_at IS NOT NULL, import_pending, study_director_user_id FROM protocols WHERE id = $1",
    )
    .bind(protocol_id)
    .fetch_optional(pool)
    .await?;

    let Some((is_imported, import_pending, sd)) = row else {
        return Ok(false);
    };
    // 限「已完成補登」的匯入計劃：import_pending 期間仍在補登計劃本體（import_approved 已寫
    // imported_at），尚不開放補登歷史變更，避免非 SD 匯入操作者提早進入此流程。
    if !is_imported || import_pending {
        return Ok(false);
    }
    // 對齊 P6 契約：限計劃負責人 SD 或管理者（不含一般 import_approved 權限持有者）。
    Ok(sd == Some(user.id) || user.is_admin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(perms: &[&str], roles: &[&str]) -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            email: "t@test.local".to_string(),
            roles: roles.iter().map(|s| s.to_string()).collect(),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            jti: "test-jti".to_string(),
            exp: 9999999999,
            impersonated_by: None,
        }
    }

    // R75-5：巡場讀取授權契約。
    #[test]
    fn vet_patrol_view_all_role_allowed() {
        // EXPERIMENT_STAFF 具 animal.animal.view_all
        let staff = user(&["animal.animal.view_all"], &[]);
        assert!(require_vet_patrol_view(&staff).is_ok());
    }

    /// 全域 STUDY_DIRECTOR 角色移除後，「僅 view_project 的人」一律被擋——
    /// 原本有一個 `vet_patrol_study_director_allowed` 測試鎖住相反的契約，
    /// 已隨該角色刪除。真正的計畫 SD 一律具 `EXPERIMENT_STAFF`（見
    /// `validate_and_authorize_sd`），走 `animal.animal.view_all` 通過。
    #[test]
    fn vet_patrol_view_project_only_rejected() {
        let view_project_only = user(&["animal.animal.view_project"], &[]);
        assert!(require_vet_patrol_view(&view_project_only).is_err());
    }

    #[test]
    fn vet_patrol_external_client_rejected() {
        // CLIENT / PI：僅 record.view + view_project，無 view_all、非 SD → 拒絕（R75-5 核心）
        let client = user(
            &["animal.record.view", "animal.animal.view_project"],
            &["CLIENT"],
        );
        assert!(matches!(
            require_vet_patrol_view(&client),
            Err(AppError::Forbidden(_))
        ));
        let pi = user(
            &["animal.record.view", "animal.animal.view_project"],
            &["PI"],
        );
        assert!(matches!(
            require_vet_patrol_view(&pi),
            Err(AppError::Forbidden(_))
        ));
    }
}
