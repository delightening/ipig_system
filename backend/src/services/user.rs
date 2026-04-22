use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        CreateUserRequest, PaginationParams, UpdateUserRequest, User, UserResponse,
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
    pub async fn list(
        pool: &PgPool,
        keyword: Option<&str>,
        pagination: &PaginationParams,
    ) -> Result<Vec<UserResponse>> {
        let suffix = pagination.sql_suffix();
        let users = if let Some(kw) = keyword {
            let pattern = user_search_pattern(kw);
            let sql = [
                "SELECT * FROM users WHERE is_active = true AND (email ILIKE $1 OR display_name ILIKE $1) ORDER BY created_at DESC",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, User>(&sql)
                .bind(&pattern)
                .fetch_all(pool)
                .await?
        } else {
            let sql = [
                "SELECT * FROM users WHERE is_active = true ORDER BY created_at DESC",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, User>(&sql).fetch_all(pool).await?
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
                updated_at = NOW()
            WHERE id = $14
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
        .fetch_one(&mut *tx)
        .await?;

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

            // SEC-PRIV: 驗證指派的角色 ID 確實存在且為有效角色，防止靜默失敗
            if !role_ids.is_empty() {
                let valid_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM roles WHERE id = ANY($1) AND is_active = true",
                )
                .bind(role_ids)
                .fetch_one(&mut *tx)
                .await?;

                if valid_count != role_ids.len() as i64 {
                    return Err(AppError::Validation(
                        "部分角色 ID 無效或已停用".to_string(),
                    ));
                }

                // SEC-PRIV: 檢查是否包含 SYSTEM_ADMIN 角色 — 僅允許現有 SYSTEM_ADMIN 指派
                let has_system_admin: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM roles WHERE id = ANY($1) AND code = $2)",
                )
                .bind(role_ids)
                .bind(crate::constants::ROLE_SYSTEM_ADMIN)
                .fetch_one(&mut *tx)
                .await?;

                if has_system_admin {
                    // System actor 受信任（seed / 自動化 provisioning）直接放行；
                    // User actor 必須擁有 SYSTEM_ADMIN 角色才能指派 SYSTEM_ADMIN 給他人。
                    if let Some(uid) = actor_user_id {
                        let actor_is_system_admin: bool = sqlx::query_scalar(
                            r#"SELECT EXISTS(
                                SELECT 1 FROM user_roles ur
                                INNER JOIN roles r ON ur.role_id = r.id
                                WHERE ur.user_id = $1 AND r.code = $2
                            )"#,
                        )
                        .bind(uid)
                        .bind(crate::constants::ROLE_SYSTEM_ADMIN)
                        .fetch_one(&mut *tx)
                        .await?;

                        if !actor_is_system_admin {
                            return Err(AppError::Forbidden(
                                "僅 SYSTEM_ADMIN 可指派 SYSTEM_ADMIN 角色".to_string(),
                            ));
                        }
                    }
                    // None = System actor：略過角色檢查
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
                let before_snap = RoleAssignmentSnapshot {
                    roles: before_roles,
                };
                let after_snap = RoleAssignmentSnapshot {
                    roles: after_roles,
                };
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

        let after = sqlx::query_as::<_, User>(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
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

        // Soft delete: 停用帳號 + 匿名化個人資料（允許對已停用帳號執行）
        let after = sqlx::query_as::<_, User>(
            r#"UPDATE users SET
                is_active = false,
                email = 'deleted_' || id::text || '@deleted.local',
                updated_at = NOW()
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id)
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
