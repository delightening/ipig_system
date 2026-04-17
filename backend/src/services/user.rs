use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{
        AuditAction, CreateUserRequest, PaginationParams, UpdateUserRequest, User, UserResponse,
    },
    services::{AuditService, AuthService},
    AppError, Result,
};

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
    /// 建立用戶（私域註冊 - 只有管理員可以建立）
    pub async fn create(pool: &PgPool, req: &CreateUserRequest) -> Result<User> {
        // 檢查 email 是否已存在
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(&req.email)
                .fetch_one(pool)
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
        .fetch_one(pool)
        .await?;

        if !req.role_ids.is_empty() {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
            )
            .bind(user.id)
            .bind(&req.role_ids)
            .execute(pool)
            .await?;
        }

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

    /// 更新用戶
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        actor_user_id: Uuid,
        req: &UpdateUserRequest,
    ) -> Result<UserResponse> {
        // 檢查用戶是否存在
        let before_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 如果要更新 email，檢查是否已被使用
        if let Some(ref new_email) = req.email {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
            )
            .bind(new_email)
            .bind(id)
            .fetch_one(pool)
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
        .fetch_one(pool)
        .await?;

        // 記錄稽核日誌
        AuditService::log(
            pool,
            actor_user_id,
            AuditAction::Update,
            "user",
            id,
            Some(serde_json::to_value(&before_user).unwrap_or(serde_json::Value::Null)),
            Some(serde_json::to_value(&updated_user).unwrap_or(serde_json::Value::Null)),
        )
        .await?;

        // 如果要更新角色
        if let Some(ref role_ids) = req.role_ids {
            // SEC-PRIV: 驗證指派的角色 ID 確實存在且為有效角色，防止靜默失敗
            if !role_ids.is_empty() {
                let valid_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM roles WHERE id = ANY($1) AND is_active = true",
                )
                .bind(role_ids)
                .fetch_one(pool)
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
                .fetch_one(pool)
                .await?;

                if has_system_admin {
                    // 檢查操作者是否為 SYSTEM_ADMIN（而非 legacy admin）
                    let actor_is_system_admin: bool = sqlx::query_scalar(
                        r#"SELECT EXISTS(
                            SELECT 1 FROM user_roles ur
                            INNER JOIN roles r ON ur.role_id = r.id
                            WHERE ur.user_id = $1 AND r.code = $2
                        )"#,
                    )
                    .bind(actor_user_id)
                    .bind(crate::constants::ROLE_SYSTEM_ADMIN)
                    .fetch_one(pool)
                    .await?;

                    if !actor_is_system_admin {
                        return Err(AppError::Forbidden(
                            "僅 SYSTEM_ADMIN 可指派 SYSTEM_ADMIN 角色".to_string(),
                        ));
                    }
                }
            }

            // 刪除現有角色
            sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            if !role_ids.is_empty() {
                sqlx::query(
                    "INSERT INTO user_roles (user_id, role_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
                )
                .bind(id)
                .bind(role_ids)
                .execute(pool)
                .await?;
            }
        }

        let (roles, permissions) =
            AuthService::get_user_roles_permissions(pool, updated_user.id).await?;

        Ok(UserResponse::from_user(&updated_user, roles, permissions))
    }

    /// GDPR：自帳號停用（軟刪除，is_active=false）
    pub async fn deactivate_self(pool: &PgPool, id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// 刪除用戶（改為 soft delete + 停用，避免 CASCADE 級聯刪除 16+ 關聯表）
    /// H3: 原為硬刪除，會導致 HR、認證、計畫書等所有關聯資料被靜默刪除
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        // 撤銷所有 refresh tokens（登出）
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        // Soft delete: 停用帳號 + 匿名化個人資料（允許對已停用帳號執行）
        let result = sqlx::query(
            r#"UPDATE users SET
                is_active = false,
                email = 'deleted_' || id::text || '@deleted.local',
                updated_at = NOW()
            WHERE id = $1"#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

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
