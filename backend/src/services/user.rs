use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreateUserRequest, UpdateUserRequest, User, UserResponse},
    services::AuthService,
    AppError, Result,
};

pub struct UserService;

impl UserService {
    /// 建立用戶（私域註冊 - 只有管理員可以建立）
    pub async fn create(pool: &PgPool, req: &CreateUserRequest) -> Result<User> {
        // 檢查 email 是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
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
                is_internal, is_active, must_change_password, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.organization)
        .bind(req.is_internal)
        .fetch_one(pool)
        .await?;

        // 指派角色
        for role_id in &req.role_ids {
            sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user.id)
                .bind(role_id)
                .execute(pool)
                .await?;
        }

        Ok(user)
    }

    /// 取得用戶列表
    pub async fn list(pool: &PgPool, keyword: Option<&str>) -> Result<Vec<UserResponse>> {
        let users = if let Some(kw) = keyword {
            let pattern = format!("%{}%", kw);
            sqlx::query_as::<_, User>(
                r#"
                SELECT * FROM users 
                WHERE email ILIKE $1 OR display_name ILIKE $1
                ORDER BY created_at DESC
                "#
            )
            .bind(&pattern)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?
        };

        let mut result = Vec::new();
        for user in users {
            let (roles, permissions) = AuthService::get_user_roles_permissions(pool, user.id).await?;
            result.push(UserResponse {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
                phone: user.phone,
                organization: user.organization,
                is_internal: user.is_internal,
                is_active: user.is_active,
                must_change_password: user.must_change_password,
                theme_preference: user.theme_preference,
                language_preference: user.language_preference,
                last_login_at: user.last_login_at,
                roles,
                permissions,
            });
        }

        Ok(result)
    }

    /// 取得單一用戶
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<UserResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let (roles, permissions) = AuthService::get_user_roles_permissions(pool, user.id).await?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            phone: user.phone,
            organization: user.organization,
            is_internal: user.is_internal,
            is_active: user.is_active,
            must_change_password: user.must_change_password,
            theme_preference: user.theme_preference,
            language_preference: user.language_preference,
            last_login_at: user.last_login_at,
            roles,
            permissions,
        })
    }

    /// 更新用戶
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateUserRequest) -> Result<UserResponse> {
        // 檢查用戶是否存在
        let _user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 如果要更新 email，檢查是否已被使用
        if let Some(ref new_email) = req.email {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)"
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
        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET
                email = COALESCE($1, email),
                display_name = COALESCE($2, display_name),
                phone = COALESCE($3, phone),
                organization = COALESCE($4, organization),
                is_internal = COALESCE($5, is_internal),
                is_active = COALESCE($6, is_active),
                updated_at = NOW()
            WHERE id = $7
            RETURNING *
            "#
        )
        .bind(&req.email)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.organization)
        .bind(req.is_internal)
        .bind(req.is_active)
        .bind(id)
        .fetch_one(pool)
        .await?;

        // 如果要更新角色
        if let Some(ref role_ids) = req.role_ids {
            // 刪除現有角色
            sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            // 指派新角色
            for role_id in role_ids {
                sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
                    .bind(id)
                    .bind(role_id)
                    .execute(pool)
                    .await?;
            }
        }

        let (roles, permissions) = AuthService::get_user_roles_permissions(pool, updated_user.id).await?;

        Ok(UserResponse {
            id: updated_user.id,
            email: updated_user.email,
            display_name: updated_user.display_name,
            phone: updated_user.phone,
            organization: updated_user.organization,
            is_internal: updated_user.is_internal,
            is_active: updated_user.is_active,
            must_change_password: updated_user.must_change_password,
            theme_preference: updated_user.theme_preference,
            language_preference: updated_user.language_preference,
            last_login_at: updated_user.last_login_at,
            roles,
            permissions,
        })
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
