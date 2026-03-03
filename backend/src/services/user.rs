use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{AuditAction, CreateUserRequest, PaginationParams, UpdateUserRequest, User, UserResponse},
    services::{AuditService, AuthService},
    AppError, Result,
};

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
                id, email, password_hash, display_name, phone, organization,
                entry_date, position, aup_roles, years_experience, trainings,
                is_internal, is_active, must_change_password, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, true, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.organization)
        .bind(req.entry_date)
        .bind(&req.position)
        .bind(&req.aup_roles)
        .bind(req.years_experience)
        .bind(sqlx::types::Json(&req.trainings))
        .bind(req.is_internal)
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
    pub async fn list(pool: &PgPool, keyword: Option<&str>, pagination: &PaginationParams) -> Result<Vec<UserResponse>> {
        let suffix = pagination.sql_suffix();
        let users = if let Some(kw) = keyword {
            let pattern = format!("%{}%", kw);
            let sql = [
                "SELECT * FROM users WHERE email ILIKE $1 OR display_name ILIKE $1 ORDER BY created_at DESC",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, User>(&sql)
                .bind(&pattern)
                .fetch_all(pool)
                .await?
        } else {
            let sql =
                ["SELECT * FROM users ORDER BY created_at DESC", suffix.as_str()].concat();
            sqlx::query_as::<_, User>(&sql)
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
                   ORDER BY r.code"#
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
                   ORDER BY p.code"#
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
                organization = COALESCE($4, organization),
                entry_date = COALESCE($5, entry_date),
                position = COALESCE($6, position),
                aup_roles = COALESCE($7, aup_roles),
                years_experience = COALESCE($8, years_experience),
                trainings = COALESCE($9, trainings),
                is_internal = COALESCE($10, is_internal),
                is_active = COALESCE($11, is_active),
                updated_at = NOW()
            WHERE id = $12
            RETURNING *
            "#,
        )
        .bind(&req.email)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.organization)
        .bind(req.entry_date)
        .bind(&req.position)
        .bind(&req.aup_roles)
        .bind(req.years_experience)
        .bind(trainings_json)
        .bind(req.is_internal)
        .bind(req.is_active)
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
        let result = sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// 刪除用戶（硬刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        // 先刪除用戶的角色關聯
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        // 刪除用戶的 refresh tokens
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        // 刪除用戶
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}
