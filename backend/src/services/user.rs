use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        AssignableUser, CommitteeMember, CreateUserRequest, DepartmentMember, PaginationParams,
        UpdateUserRequest, User, UserResponse,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, AuthService,
    },
    AppError, Result,
};

/// Audit-only 輔助型別：記錄角色指派的 before/after（用於 USER_ROLE_CHANGE 事件）。
/// 角色清單無敏感欄位，空 AuditRedact impl 即可。
#[derive(Serialize)]
struct RoleAssignmentSnapshot {
    roles: Vec<String>,
}
impl AuditRedact for RoleAssignmentSnapshot {}

/// Audit-only 輔助型別：強制移除角色時，記下被釋出的未結清事項清單
/// （用於 SECURITY 類 ROLE_REMOVAL_FORCED 事件）。內容為申請人姓名與請假日期，
/// 與該事件本就可見的層級相同，空 AuditRedact impl 即可。
#[derive(Serialize)]
struct UnsettledItemsSnapshot {
    items: Vec<String>,
}
impl AuditRedact for UnsettledItemsSnapshot {}

/// 產生使用者列表搜尋用的 ILIKE pattern（前後加 %、trim），供 list 與單元測試使用。
pub fn user_search_pattern(keyword: &str) -> String {
    format!("%{}%", keyword.trim())
}

#[derive(sqlx::FromRow)]
struct UserRoleRow {
    user_id: Uuid,
    code: String,
}

#[derive(sqlx::FromRow)]
struct UserPermRow {
    user_id: Uuid,
    code: String,
}

pub struct UserService;

impl UserService {
    /// 建立用戶（私域註冊 - 管理員 / 系統自動化初始化）— Service-driven audit
    ///
    /// Actor 政策：User（管理員操作）與 System（seed / 自動化 provisioning）均允許；
    /// Anonymous 拒絕。
    /// SEC-PRIV (CSO-r2 #1): 驗證角色指派授權。
    ///
    /// 1. 角色 ID 必須存在且為 active（防靜默失敗）。
    /// 2. 指派「管理員層級」或 DIRECTOR 角色須由具同層級的 actor 執行，防止持有
    ///    `admin.user.create/edit` 的非管理員（如 ADMIN_STAFF）自我提權：
    ///    - 指派 SYSTEM_ADMIN：actor 必須本身為 SYSTEM_ADMIN（維持既有不變式）。
    ///    - 指派 legacy `admin`（= 全權限 admin，見 `middleware::Claims::is_admin`）：
    ///      actor 必須為管理員層級（SYSTEM_ADMIN 或 legacy admin）。
    ///    - 指派 DIRECTOR：actor 必須為管理員層級，或本身已持有 DIRECTOR（2026-08-13 補，
    ///      見 `access::require_authority_to_assign_roles` 的同步說明）。
    ///
    /// `actor_user_id == None` 代表 System actor（seed / provisioning），受信任放行。
    async fn validate_role_assignment(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor_user_id: Option<Uuid>,
        role_ids: &[Uuid],
    ) -> Result<()> {
        if role_ids.is_empty() {
            return Ok(());
        }

        // 1) 角色存在且 active。以「去重後的 ID 數」比對：role_ids 可能含重複，
        //    而 COUNT(*) 只計到資料表中相異的角色列，直接比 role_ids.len() 會把
        //    帶重複 ID 的合法請求誤判為「部分角色無效」（插入端 ON CONFLICT 已容忍重複）。
        let distinct_role_count = role_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        let valid_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roles WHERE id = ANY($1) AND is_active = true",
        )
        .bind(role_ids)
        .fetch_one(&mut **tx)
        .await?;
        if valid_count != distinct_role_count as i64 {
            return Err(AppError::Validation("部分角色 ID 無效或已停用".to_string()));
        }

        // 2) 管理員層級 + DIRECTOR 提權防護（單次查詢取出指派集合內的受保護角色碼）
        //
        // ⚠️ 2026-08-13 補 DIRECTOR：`ADMIN_STAFF` 持有 `admin.user.edit`，若不擋
        // DIRECTOR 指派，等於任何行政人員都能把終審簽核權指派給任意使用者、包括自己。
        // 與 `access::require_authority_to_assign_roles` 保持同一份判準（該函式服務
        // 「不經 UserService」的角色指派路徑，如 invitation），兩處各自查詢是既有結構
        // （一個吃 `tx`、一個吃 `pool`，未強行合併），但條件必須同步修改。
        let guarded_codes = &[
            crate::constants::ROLE_SYSTEM_ADMIN,
            crate::constants::ROLE_ADMIN_LEGACY,
            crate::constants::ROLE_DIRECTOR,
        ][..];
        let assigned_guarded_codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM roles WHERE id = ANY($1) AND code = ANY($2)")
                .bind(role_ids)
                .bind(guarded_codes)
                .fetch_all(&mut **tx)
                .await?;
        let assigns_system_admin = assigned_guarded_codes
            .iter()
            // SYSTEM_ADMIN-ONLY：刻意只認 SYSTEM_ADMIN——語意是「只有 X 能指派 X」，
            // 不是漏 legacy fallback；加上它等於把這條授權規則降級。
            .any(|c| c == crate::constants::ROLE_SYSTEM_ADMIN);
        let assigns_legacy_admin = assigned_guarded_codes
            .iter()
            .any(|c| c == crate::constants::ROLE_ADMIN_LEGACY);
        let assigns_director = assigned_guarded_codes
            .iter()
            .any(|c| c == crate::constants::ROLE_DIRECTOR);

        // System actor（None）受信任，跳過 actor 角色檢查。
        if let Some(uid) = actor_user_id {
            if assigns_system_admin || assigns_legacy_admin || assigns_director {
                let actor_guarded_codes: Vec<String> = sqlx::query_scalar(
                    r#"SELECT r.code FROM user_roles ur
                       INNER JOIN roles r ON ur.role_id = r.id
                       WHERE ur.user_id = $1 AND r.code = ANY($2)"#,
                )
                .bind(uid)
                .bind(guarded_codes)
                .fetch_all(&mut **tx)
                .await?;
                let actor_is_system_admin = actor_guarded_codes
                    .iter()
                    // SYSTEM_ADMIN-ONLY：刻意只認 SYSTEM_ADMIN——語意是「只有 X 能指派 X」，
                    // 不是漏 legacy fallback；加上它等於把這條授權規則降級。
                    .any(|c| c == crate::constants::ROLE_SYSTEM_ADMIN);
                let actor_is_admin_tier = actor_guarded_codes.iter().any(|c| {
                    c == crate::constants::ROLE_SYSTEM_ADMIN
                        || c == crate::constants::ROLE_ADMIN_LEGACY
                });
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
            }
        }
        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateUserRequest,
    ) -> Result<User> {
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "建立使用者須由已登入管理員或系統觸發".into(),
                ));
            }
        }
        let mut tx = pool.begin().await?;

        // 檢查 email 是否已存在
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(&req.email)
                .fetch_one(&mut *tx)
                .await?;

        if exists {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        // Hash 密碼
        let password_hash = AuthService::hash_password(&req.password)?;

        // 建立用戶 - 新用戶預設需要變更密碼
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, email, password_hash, display_name, phone, phone_ext, organization,
                entry_date, position, aup_roles, years_experience, trainings,
                is_internal, is_active, must_change_password, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, true, true, $14, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.phone_ext)
        .bind(&req.organization)
        .bind(req.entry_date)
        .bind(&req.position)
        .bind(&req.aup_roles)
        .bind(req.years_experience)
        .bind(sqlx::types::Json(&req.trainings))
        .bind(req.is_internal)
        .bind(req.expires_at)
        .fetch_one(&mut *tx)
        .await?;

        if !req.role_ids.is_empty() {
            // SEC-PRIV (CSO-r2 #1): 建立使用者時亦需角色指派授權檢查（create 路徑原本零防護）。
            let actor_user_id = match actor {
                ActorContext::User(u) => Some(u.id),
                _ => None,
            };
            Self::validate_role_assignment(&mut tx, actor_user_id, &req.role_ids).await?;
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
            )
            .bind(user.id)
            .bind(&req.role_ids)
            .execute(&mut *tx)
            .await?;
        }

        let display = format!("{} <{}>", user.display_name, user.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ADMIN",
                event_type: "USER_CREATE",
                entity: Some(AuditEntity::new("user", user.id, &display)),
                data_diff: Some(DataDiff::create_only(&user)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    /// 取得用戶列表
    ///
    /// `include_inactive=false`（預設）只回在職帳號；`true` 一併回傳**停用但未刪除**的
    /// 帳號，供管理頁「顯示停用帳號」開關使用（停用者仍帶 `is_active=false`，前端據此
    /// 標記）。已軟刪除（匿名化，email 為 `…@deleted.local`）的帳號**永不回傳**，即使
    /// `include_inactive=true`——刪除者不應出現在「顯示停用」清單。停用與軟刪除皆為
    /// `is_active=false`，靠 email 網域後綴區分（見 `delete`）。
    pub async fn list(
        pool: &PgPool,
        keyword: Option<&str>,
        include_inactive: bool,
        pagination: &PaginationParams,
    ) -> Result<Vec<UserResponse>> {
        let suffix = pagination.sql_suffix();
        // is_active 過濾為布林開關（非使用者輸入），依 include_inactive 決定是否放行停用者
        let active_filter = if include_inactive {
            ""
        } else {
            "is_active = true AND "
        };
        // 已軟刪除（匿名化）帳號永不列出，即使開啟「顯示停用帳號」。以 email 網域後綴
        // 辨識（見 delete 的匿名化改寫）；停用但未刪除的帳號仍正常回傳。
        let deleted_pattern = format!("%{}", crate::constants::DELETED_USER_EMAIL_DOMAIN);
        let users = if let Some(kw) = keyword {
            let pattern = user_search_pattern(kw);
            let sql = [
                "SELECT * FROM users WHERE ",
                active_filter,
                "email NOT LIKE $2 AND (email ILIKE $1 OR display_name ILIKE $1) \
                 ORDER BY created_at DESC",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(sql))
                .bind(&pattern)
                .bind(&deleted_pattern)
                .fetch_all(pool)
                .await?
        } else {
            let sql = [
                "SELECT * FROM users WHERE ",
                active_filter,
                "email NOT LIKE $1 ORDER BY created_at DESC",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(sql))
                .bind(&deleted_pattern)
                .fetch_all(pool)
                .await?
        };

        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

        let (roles_map, perms_map) = if user_ids.is_empty() {
            (HashMap::new(), HashMap::new())
        } else {
            let role_rows = sqlx::query_as::<_, UserRoleRow>(
                r#"SELECT ur.user_id, r.code
                   FROM user_roles ur
                   INNER JOIN roles r ON ur.role_id = r.id
                   WHERE ur.user_id = ANY($1)
                   ORDER BY r.code"#,
            )
            .bind(&user_ids)
            .fetch_all(pool)
            .await?;

            let perm_rows = sqlx::query_as::<_, UserPermRow>(
                r#"SELECT DISTINCT ur.user_id, p.code
                   FROM user_roles ur
                   INNER JOIN role_permissions rp ON ur.role_id = rp.role_id
                   INNER JOIN permissions p ON rp.permission_id = p.id
                   WHERE ur.user_id = ANY($1)
                   ORDER BY p.code"#,
            )
            .bind(&user_ids)
            .fetch_all(pool)
            .await?;

            let mut rm: HashMap<Uuid, Vec<String>> = HashMap::new();
            for row in role_rows {
                rm.entry(row.user_id).or_default().push(row.code);
            }
            let mut pm: HashMap<Uuid, Vec<String>> = HashMap::new();
            for row in perm_rows {
                pm.entry(row.user_id).or_default().push(row.code);
            }
            (rm, pm)
        };

        let result = users
            .iter()
            .map(|user| {
                let roles = roles_map.get(&user.id).cloned().unwrap_or_default();
                let permissions = perms_map.get(&user.id).cloned().unwrap_or_default();
                UserResponse::from_user(user, roles, permissions)
            })
            .collect();

        Ok(result)
    }

    /// 列出所有在職使用者的精簡資料（id / display_name / email / roles），供計畫
    /// PI / Study Director 下拉指派使用。不回傳權限清單等敏感欄位，授權門檻較
    /// `list`（admin.user.view）低，由 handler 以計畫匯入權限把關。
    pub async fn list_assignable_users(pool: &PgPool) -> Result<Vec<AssignableUser>> {
        let users = sqlx::query_as::<_, AssignableUser>(
            r#"SELECT u.id, u.display_name, u.email,
                      COALESCE(ARRAY_AGG(r.code) FILTER (WHERE r.code IS NOT NULL), '{}') AS roles
               FROM users u
               LEFT JOIN user_roles ur ON ur.user_id = u.id
               LEFT JOIN roles r ON ur.role_id = r.id
               WHERE u.is_active = true
               GROUP BY u.id, u.display_name, u.email
               ORDER BY u.display_name"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(users)
    }

    /// 取得原始 User（供內部邏輯使用，如 2FA）
    pub async fn get_user_raw(pool: &PgPool, id: Uuid) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    /// 取得單一用戶
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<UserResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let (roles, permissions) = AuthService::get_user_roles_permissions(pool, user.id).await?;

        Ok(UserResponse::from_user(&user, roles, permissions))
    }

    /// 移除角色前的「未結清事項」，逐筆回傳人類可讀描述。
    ///
    /// 只查會因失去角色而**卡死**的項目：
    /// 1. 待他確認的職務代理假單（`PENDING_PROXY`）——只有被指定的代理人本人能確認
    /// 2. 待他簽核的請假單（`current_approver_id` 指向他）
    /// 3. 已核准、假期尚未結束、由他擔任代理人的假單——假期到時他已無權接手職務
    ///
    /// 加班單不列入：`overtime_records` 沒有 `current_approver_id`，審核人由角色動態
    /// 判定，換人不會卡在特定某個人身上。
    async fn find_unsettled_items(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String, String, chrono::NaiveDate, chrono::NaiveDate)> = sqlx::query_as(
            r#"
            SELECT '待他確認的職務代理' AS kind, u.display_name, l.start_date, l.end_date
            FROM leave_requests l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.proxy_user_id = $1 AND l.status = 'PENDING_PROXY'::leave_status
            UNION ALL
            SELECT '待他簽核的請假單', u.display_name, l.start_date, l.end_date
            FROM leave_requests l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.current_approver_id = $1
              AND l.status::text LIKE 'PENDING%'
              AND l.status <> 'PENDING_PROXY'::leave_status
            UNION ALL
            SELECT '假期未結束的代理指定', u.display_name, l.start_date, l.end_date
            FROM leave_requests l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.proxy_user_id = $1
              AND l.status = 'APPROVED'::leave_status
              AND l.end_date >= CURRENT_DATE
            ORDER BY 1, 3
            "#,
        )
        .bind(user_id)
        .fetch_all(&mut **tx)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(kind, applicant, start, end)| {
                if start == end {
                    format!("{kind}：{applicant} {start}")
                } else {
                    format!("{kind}：{applicant} {start}~{end}")
                }
            })
            .collect())
    }

    /// 強制移除角色時，把卡在該使用者身上的待辦釋出，避免單據跟著權限一起凍結。
    ///
    /// - 待他確認的代理假單 → 退回 `DRAFT`，申請人可重新指定代理人（保留 `proxy_user_id`
    ///   供申請人參考，與 `proxy_reject_leave` 的處置一致）
    /// - 待他簽核的請假單 → 只清 `current_approver_id`，**狀態不變**，交還該關由其他
    ///   合法審核人接手。刻意不推進關卡：那等於系統代他簽了一關，稽核上無法交代
    ///
    /// 第 3 類（已核准、假期未結束的代理指定）不動資料——假單已生效、餘額已扣，
    /// 改它會牽動餘額與通知；僅列入稽核紀錄供人工追蹤。
    async fn release_unsettled_items(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE leave_requests
               SET status = 'DRAFT'::leave_status, current_approver_id = NULL,
                   submitted_at = NULL, updated_at = NOW()
               WHERE proxy_user_id = $1 AND status = 'PENDING_PROXY'::leave_status"#,
        )
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"UPDATE leave_requests
               SET current_approver_id = NULL, updated_at = NOW()
               WHERE current_approver_id = $1
                 AND status::text LIKE 'PENDING%'
                 AND status <> 'PENDING_PROXY'::leave_status"#,
        )
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 更新用戶 — Service-driven audit
    ///
    /// 完整流程在同一 tx 內：欄位更新 + 角色重指派 + audit（包括 SECURITY 類
    /// ROLE_CHANGE / ACCOUNT_STATUS_CHANGE 事件）。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateUserRequest,
    ) -> Result<UserResponse> {
        // Actor 政策：User 或 System 均可；Anonymous 拒絕。
        // actor_user_id = Some(uid) 表示是真實 User（需角色檢查）；None 表示 System（受信任，略過）。
        let actor_user_id: Option<Uuid> = match actor {
            ActorContext::User(user) => Some(user.id),
            ActorContext::System { .. } => None,
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "更新使用者須由已登入管理員或系統觸發".into(),
                ));
            }
        };
        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE 取 before（同時檢查使用者存在）
        let before_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 如果要更新 email，檢查是否已被使用
        if let Some(ref new_email) = req.email {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
            )
            .bind(new_email)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            if exists {
                return Err(AppError::Conflict("Email already exists".to_string()));
            }
        }

        // 更新用戶
        let trainings_json = req.trainings.as_ref().map(sqlx::types::Json);
        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET
                email = COALESCE($1, email),
                display_name = COALESCE($2, display_name),
                phone = COALESCE($3, phone),
                phone_ext = COALESCE($4, phone_ext),
                organization = COALESCE($5, organization),
                entry_date = COALESCE($6, entry_date),
                position = COALESCE($7, position),
                aup_roles = COALESCE($8, aup_roles),
                years_experience = COALESCE($9, years_experience),
                trainings = COALESCE($10, trainings),
                is_internal = COALESCE($11, is_internal),
                is_active = COALESCE($12, is_active),
                expires_at = COALESCE($13, expires_at),
                version = version + 1,
                updated_at = NOW()
            WHERE id = $14
              AND ($15::INT IS NULL OR version = $15)
            RETURNING *
            "#,
        )
        .bind(&req.email)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.phone_ext)
        .bind(&req.organization)
        .bind(req.entry_date)
        .bind(&req.position)
        .bind(&req.aup_roles)
        .bind(req.years_experience)
        .bind(trainings_json)
        .bind(req.is_internal)
        .bind(req.is_active)
        .bind(req.expires_at)
        .bind(id)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("使用者資料已被其他人修改，請重新載入後再試".into()))?;

        // 基本 USER_UPDATE 事件（before/after diff 自動 redact password_hash 等）
        let display = format!("{} <{}>", updated_user.display_name, updated_user.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ADMIN",
                event_type: "USER_UPDATE",
                entity: Some(AuditEntity::new("user", updated_user.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_user), Some(&updated_user))),
                request_context: None,
            },
        )
        .await?;

        // SECURITY：帳號狀態變更另寫一筆（供監控系統快速篩選）
        if before_user.is_active != updated_user.is_active {
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "SECURITY",
                    event_type: "USER_STATUS_CHANGE",
                    entity: Some(AuditEntity::new("user", updated_user.id, &display)),
                    data_diff: Some(DataDiff::compute(Some(&before_user), Some(&updated_user))),
                    request_context: None,
                },
            )
            .await?;
        }

        // 如果要更新角色
        if let Some(ref role_ids) = req.role_ids {
            // before 角色清單只在實際要變更角色時才查（避免一般欄位更新的冗餘 DB round-trip）
            let before_roles: Vec<String> = sqlx::query_scalar(
                r#"SELECT r.code FROM user_roles ur
                   INNER JOIN roles r ON ur.role_id = r.id
                   WHERE ur.user_id = $1
                   ORDER BY r.code"#,
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            // SEC-PRIV (CSO-r2 #1): 角色指派授權檢查（存在性 + 提權防護，涵蓋
            // SYSTEM_ADMIN、legacy `admin`、DIRECTOR）。這是本函式服務的
            // `PUT /api/v1/users/{id}`（`admin.user.edit`）唯一的角色指派守衛——
            // ADMIN_STAFF 持有 `admin.user.edit` 但不持有管理員層級，2026-08-13 前
            // 這裡漏擋 DIRECTOR，等於任何行政人員都能把終審簽核權指派給任意使用者。
            Self::validate_role_assignment(&mut tx, actor_user_id, role_ids).await?;

            // 角色縮減（有 code 從清單消失）＝ 對方可能帶著卡在他身上的單據離場。
            // 先確認身上沒有未結清事項；有的話擋下並列出清單，僅系統管理員可帶
            // force_role_change 強制執行（屆時把待辦釋出，見 release_unsettled_items）。
            let after_codes: Vec<String> =
                sqlx::query_scalar("SELECT code FROM roles WHERE id = ANY($1)")
                    .bind(role_ids)
                    .fetch_all(&mut *tx)
                    .await?;
            let has_role_removal = before_roles.iter().any(|c| !after_codes.contains(c));

            if has_role_removal {
                let unsettled = Self::find_unsettled_items(&mut tx, id).await?;
                if !unsettled.is_empty() {
                    if !req.force_role_change.unwrap_or(false) {
                        return Err(AppError::UnsettledItems {
                            message: format!(
                                "此使用者身上還有 {} 件未結清事項，移除角色後將沒有人能接手。\
                                 請先請本人處理完，或由系統管理員強制移除（系統會把待他確認的\
                                 代理假單退回申請人重選代理人、待他簽核的單交還該關）。",
                                unsettled.len()
                            ),
                            items: unsettled,
                        });
                    }
                    let actor_is_admin = match actor {
                        ActorContext::User(user) => user.is_admin(),
                        ActorContext::System { .. } => true,
                        ActorContext::Anonymous => false,
                    };
                    if !actor_is_admin {
                        return Err(AppError::Forbidden(
                            "僅系統管理員可強制移除仍有未結清事項的角色".into(),
                        ));
                    }
                    Self::release_unsettled_items(&mut tx, id).await?;
                    let released = UnsettledItemsSnapshot { items: unsettled };
                    AuditService::log_activity_tx(
                        &mut tx,
                        actor,
                        ActivityLogEntry {
                            event_category: "SECURITY",
                            event_type: "ROLE_REMOVAL_FORCED",
                            entity: Some(AuditEntity::new("user", id, &display)),
                            // (Some, None) = DELETE 語意：這些待辦已從該使用者身上釋出。
                            data_diff: Some(DataDiff::compute(
                                Some(&released),
                                None::<&UnsettledItemsSnapshot>,
                            )),
                            request_context: None,
                        },
                    )
                    .await?;
                }
            }

            // 刪除現有角色
            sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            if !role_ids.is_empty() {
                sqlx::query(
                    "INSERT INTO user_roles (user_id, role_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
                )
                .bind(id)
                .bind(role_ids)
                .execute(&mut *tx)
                .await?;
            }

            // 取 after 角色清單，若有變動則寫 SECURITY 類 ROLE_CHANGE 事件
            let after_roles: Vec<String> = sqlx::query_scalar(
                r#"SELECT r.code FROM user_roles ur
                   INNER JOIN roles r ON ur.role_id = r.id
                   WHERE ur.user_id = $1
                   ORDER BY r.code"#,
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            if before_roles != after_roles {
                // CSO-r3 #4：角色實際變動 → 撤銷該使用者所有未過期 access token，
                // 使 JWT 內過時的 roles 快照即時失效（須重新登入取得新角色）。
                sqlx::query("UPDATE users SET tokens_valid_after = NOW() WHERE id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;

                let before_snap = RoleAssignmentSnapshot {
                    roles: before_roles,
                };
                let after_snap = RoleAssignmentSnapshot { roles: after_roles };
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "SECURITY",
                        event_type: "USER_ROLE_CHANGE",
                        entity: Some(AuditEntity::new("user", updated_user.id, &display)),
                        data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                        request_context: None,
                    },
                )
                .await?;
            }
        }

        tx.commit().await?;

        let (roles, permissions) =
            AuthService::get_user_roles_permissions(pool, updated_user.id).await?;

        Ok(UserResponse::from_user(&updated_user, roles, permissions))
    }

    /// GDPR：自帳號停用（軟刪除，is_active=false）— Service-driven audit
    pub async fn deactivate_self(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let user = actor.require_user()?;
        // 守門：此 API 僅允許使用者停用自己的帳號，actor.id 必須 == id
        if user.id != id {
            return Err(AppError::Forbidden(
                "deactivate_self 僅允許停用自己的帳號".into(),
            ));
        }
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // L-3（安全稽核 2026-07-04）：一併設 tokens_valid_after=NOW()，撤銷該使用者
        // 所有裝置未過期的 access token——auth middleware 的 enforce_tokens_valid_after
        // 在載入權限前先擋，故其他裝置立即 401（原本會殘留至 access token TTL）。
        let after = sqlx::query_as::<_, User>(
            "UPDATE users SET is_active = false, tokens_valid_after = NOW(), updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} <{}>", after.display_name, after.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "USER_DEACTIVATE_SELF",
                entity: Some(AuditEntity::new("user", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// 刪除用戶（改為 soft delete + 停用，避免 CASCADE 級聯刪除 16+ 關聯表）
    /// H3: 原為硬刪除，會導致 HR、認證、計畫書等所有關聯資料被靜默刪除
    /// Service-driven audit
    ///
    /// Actor 政策：User（管理員）與 System（過期帳號自動清理）均允許；Anonymous 拒絕。
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "刪除使用者須由已登入管理員或系統觸發".into(),
                ));
            }
        }
        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE 取 before（含原 email，soft-delete 後 email 會被匿名化）
        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 撤銷所有 refresh tokens（登出）
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Soft delete: 停用 + 匿名化 email/phone/phone_ext/organization/position，但
        // **保留 display_name**（R63-B6 收窄，2026-07-04）——「誰記錄的」屬 GLP 歸屬資料，
        // 須隨歷史紀錄保留（見 AUDIT_LOGGING.md / PROGRESS §9）。已刪除帳號靠 email 網域
        // 後綴 $2 = DELETED_USER_EMAIL_DOMAIN 排除於清單/選單（與 list 共用常數），不靠名字。
        let after = sqlx::query_as::<_, User>(
            r#"UPDATE users SET
                is_active = false,
                email = 'deleted_' || id::text || $2,
                phone = NULL,
                phone_ext = NULL,
                organization = NULL,
                position = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id)
        .bind(crate::constants::DELETED_USER_EMAIL_DOMAIN)
        .fetch_one(&mut *tx)
        .await?;

        // 使用 before 的原 email 供稽核查詢「誰被刪了」
        let display = format!("{} <{}>", before.display_name, before.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "USER_DELETE",
                entity: Some(AuditEntity::new("user", before.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    // ============================================
    // 部門成員指派（users.department_id）— 啟用請假 L1 單位主管關
    // ============================================

    /// 列出部門成員（在職）。`dept_id = Some` 篩單一部門；`None` 回所有已指派部門的
    /// 在職使用者（供組織圖一次載入，避免逐部門 N+1）。
    pub async fn list_department_members(
        pool: &PgPool,
        dept_id: Option<Uuid>,
    ) -> Result<Vec<DepartmentMember>> {
        let members = sqlx::query_as::<_, DepartmentMember>(
            r#"SELECT id, display_name, email, department_id, is_internal
               FROM users
               WHERE is_active = true
                 AND department_id IS NOT NULL
                 AND ($1::uuid IS NULL OR department_id = $1)
               ORDER BY display_name"#,
        )
        .bind(dept_id)
        .fetch_all(pool)
        .await?;
        Ok(members)
    }

    /// IACUC 委員名冊（在職、依角色，與部門編制無關）。
    ///
    /// 委員身分來自角色而非部門：`department_id` 是單值的人事歸屬，裝不下
    /// 「獸醫同時是審查委員」這種跨部門身分。用角色查，同一個人就能同時
    /// 出現在部門樹與委員名冊裡，各自表達不同的事實。
    ///
    /// `DISTINCT` 是必要的——一個人可能同時持有多個委員角色
    /// （例如主席兼審查委員），不去重會重複列出。
    pub async fn list_committee_members(pool: &PgPool) -> Result<Vec<CommitteeMember>> {
        let members = sqlx::query_as::<_, CommitteeMember>(
            r#"SELECT DISTINCT u.id, u.display_name, d.name AS department_name, u.is_internal
               FROM users u
               INNER JOIN user_roles ur ON ur.user_id = u.id
               INNER JOIN roles r ON r.id = ur.role_id
               LEFT JOIN departments d ON d.id = u.department_id
               WHERE u.is_active = true
                 AND u.deleted_at IS NULL
                 AND r.code = ANY($1)
               ORDER BY u.display_name"#,
        )
        .bind(
            &[
                crate::constants::ROLE_IACUC_CHAIR,
                crate::constants::ROLE_REVIEWER,
                crate::constants::ROLE_IACUC_STAFF,
            ][..],
        )
        .fetch_all(pool)
        .await?;
        Ok(members)
    }

    /// 共用：於既有 tx 內寫入 department_id 變更 + USER_DEPARTMENT_CHANGE audit。
    /// `before` 須為已 SELECT FOR UPDATE 取得的當前列，供 DataDiff 與 HMAC chain 使用。
    async fn write_department_change_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        before: &User,
        dept_id: Option<Uuid>,
    ) -> Result<User> {
        let after = sqlx::query_as::<_, User>(
            "UPDATE users SET department_id = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(dept_id)
        .bind(before.id)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} <{}>", after.display_name, after.email);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ADMIN",
                event_type: "USER_DEPARTMENT_CHANGE",
                entity: Some(AuditEntity::new("user", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(before), Some(&after))),
                request_context: None,
            },
        )
        .await?;
        Ok(after)
    }

    /// 指派使用者所屬部門（設定 users.department_id）— Service-driven audit。
    /// Actor 政策：User（管理員）/ System 允許；Anonymous 拒絕。
    /// 指派使用者到部門。
    ///
    /// `allow_external`：明知對方是外部人員（`is_internal = false`）仍要編入。
    /// 外部人員屬於某個部門是合法的（IACUC 是內部部門卻聘用外部委員），但那
    /// 必須是刻意的動作——前端的「一併顯示外部人員」開關表達這個意圖，這裡
    /// 是它的伺服器端對應。少了這道檢查，過期的前端或任何繞過該開關的呼叫端
    /// 都能靜默把外部人員塞進任一部門。
    pub async fn assign_user_department(
        pool: &PgPool,
        actor: &ActorContext,
        user_id: Uuid,
        dept_id: Uuid,
        allow_external: bool,
    ) -> Result<User> {
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "指派部門須由已登入管理員或系統觸發".into(),
                ));
            }
        }
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 已在目標部門則無需更新（免冗餘 UPDATE 與 no-op audit 記錄）。
        if before.department_id == Some(dept_id) {
            return Ok(before);
        }

        // 指派目標部門須存在且在職。FOR SHARE 鎖住該部門列直到 commit，避免與
        // 「同時停用該部門」的交易產生 TOCTOU（否則檢查通過後、UPDATE 前部門可能被停用）。
        let dept_active: Option<bool> =
            sqlx::query_scalar("SELECT is_active FROM departments WHERE id = $1 FOR SHARE")
                .bind(dept_id)
                .fetch_optional(&mut *tx)
                .await?;
        if dept_active != Some(true) {
            return Err(AppError::Validation("部門不存在或已停用".to_string()));
        }

        // 外部人員必須明示意圖才編得進來（見函式說明）。用前面已 FOR UPDATE
        // 鎖住的 `before`，不重查——避免與「同時把此人改成內部/外部」的交易
        // 產生 TOCTOU。
        if !before.is_internal && !allow_external {
            return Err(AppError::Validation(
                "此人為外部人員。外部人員可以編入部門（例如 IACUC 外聘委員），\
                 但需明確確認後再指派。"
                    .to_string(),
            ));
        }

        let after =
            Self::write_department_change_tx(&mut tx, actor, &before, Some(dept_id)).await?;
        tx.commit().await?;
        Ok(after)
    }

    /// 將使用者移出指定部門（清為 NULL）— Service-driven audit。
    /// 安全守則：僅當該使用者「目前確實屬於此部門」才清除，避免誤清已改調他部門者。
    /// 參數順序與 `assign_user_department` 一致（user_id, dept_id），避免呼叫端誤植。
    pub async fn remove_user_department(
        pool: &PgPool,
        actor: &ActorContext,
        user_id: Uuid,
        dept_id: Uuid,
    ) -> Result<User> {
        match actor {
            ActorContext::User(_) | ActorContext::System { .. } => {}
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "移除部門成員須由已登入管理員或系統觸發".into(),
                ));
            }
        }
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if before.department_id != Some(dept_id) {
            return Err(AppError::Validation(
                "此使用者不屬於指定部門，無法移除".to_string(),
            ));
        }

        let after = Self::write_department_change_tx(&mut tx, actor, &before, None).await?;
        tx.commit().await?;
        Ok(after)
    }
}

#[cfg(test)]
mod tests {
    use super::user_search_pattern;

    #[test]
    fn test_user_search_pattern_empty() {
        assert_eq!(user_search_pattern(""), "%%");
        assert_eq!(user_search_pattern("   "), "%%");
    }

    #[test]
    fn test_user_search_pattern_trim() {
        assert_eq!(user_search_pattern("  abc  "), "%abc%");
    }

    #[test]
    fn test_user_search_pattern_normal() {
        assert_eq!(user_search_pattern("john"), "%john%");
        assert_eq!(
            user_search_pattern("test@example.com"),
            "%test@example.com%"
        );
    }
}
