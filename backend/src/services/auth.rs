use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    middleware::Claims,
    models::{LoginRequest, LoginResponse, RefreshToken, User, UserResponse, PasswordResetToken},
    AppError, Result,
};

pub struct AuthService;

impl AuthService {
    /// 登入驗證
    pub async fn login(
        pool: &PgPool,
        config: &Config,
        req: &LoginRequest,
    ) -> Result<LoginResponse> {
        // 查詢用戶
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(&req.email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid email or password".to_string()))?;

        // 驗證密碼
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;
        
        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Validation("Invalid email or password".to_string()))?;

        // 更新最後登入時間
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(pool)
            .await?;

        // 獲取角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成 JWT
        let (access_token, expires_in) = Self::generate_access_token(config, &user, &roles, &permissions)?;

        // 生成 Refresh Token
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
                phone: user.phone,
                organization: user.organization,
                is_internal: user.is_internal,
                is_active: user.is_active,
                must_change_password: user.must_change_password,
                theme_preference: user.theme_preference.clone(),
                language_preference: user.language_preference.clone(),
                last_login_at: user.last_login_at,
                roles,
                permissions,
            },
            must_change_password: user.must_change_password,
        })
    }

    /// 刷新 Token
    pub async fn refresh_token(
        pool: &PgPool,
        config: &Config,
        refresh_token: &str,
    ) -> Result<LoginResponse> {
        // 計算 token hash
        let token_hash = Self::hash_token(refresh_token);

        // 查詢 refresh token
        let token_record = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens 
            WHERE token_hash = $1 
              AND revoked_at IS NULL 
              AND expires_at > NOW()
            "#
        )
        .bind(&token_hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid refresh token".to_string()))?;

        // 查詢用戶
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(token_record.user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("User not found or inactive".to_string()))?;

        // 撤銷舊的 refresh token
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
            .bind(token_record.id)
            .execute(pool)
            .await?;

        // 獲取角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成新的 tokens
        let (access_token, expires_in) = Self::generate_access_token(config, &user, &roles, &permissions)?;
        let new_refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
                phone: user.phone,
                organization: user.organization,
                is_internal: user.is_internal,
                is_active: user.is_active,
                must_change_password: user.must_change_password,
                theme_preference: user.theme_preference.clone(),
                language_preference: user.language_preference.clone(),
                last_login_at: user.last_login_at,
                roles,
                permissions,
            },
            must_change_password: user.must_change_password,
        })
    }

    /// 登出（撤銷 refresh token）
    pub async fn logout(pool: &PgPool, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 獲取用戶的角色和權限
    pub async fn get_user_roles_permissions(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let roles: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT r.code FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.is_active = true
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let permissions: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT p.code FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN user_roles ur ON rp.role_id = ur.role_id
            INNER JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.is_active = true
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok((roles, permissions))
    }

    /// 生成 Access Token
    fn generate_access_token(
        config: &Config,
        user: &User,
        roles: &[String],
        permissions: &[String],
    ) -> Result<(String, i64)> {
        let now = Utc::now();
        let expires_in = config.jwt_expiration_hours * 3600;
        let exp = now + Duration::hours(config.jwt_expiration_hours);

        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            roles: roles.to_vec(),
            permissions: permissions.to_vec(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

        Ok((token, expires_in))
    }

    /// 生成 Refresh Token
    async fn generate_refresh_token(
        pool: &PgPool,
        user_id: Uuid,
        config: &Config,
    ) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::days(config.jwt_refresh_expiration_days);

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(token)
    }

    /// Hash token for storage
    fn hash_token(token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Hash 密碼
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();
        Ok(password_hash)
    }

    /// 驗證密碼
    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// 修改自己的密碼（需驗證舊密碼）
    pub async fn change_own_password(
        pool: &PgPool,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // 查詢用戶
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 驗證舊密碼
        if !Self::verify_password(current_password, &user.password_hash)? {
            return Err(AppError::Validation("Current password is incorrect".to_string()));
        }

        // Hash 新密碼
        let new_password_hash = Self::hash_password(new_password)?;

        // 更新密碼並清除 must_change_password 標記
        sqlx::query(
            "UPDATE users SET password_hash = $1, must_change_password = false, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

        // 撤銷所有 refresh tokens（安全措施）
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Admin 重設他人密碼（不需驗證舊密碼）
    pub async fn reset_user_password(
        pool: &PgPool,
        target_user_id: Uuid,
        new_password: &str,
    ) -> Result<()> {
        // 確認目標用戶存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
        )
        .bind(target_user_id)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        // Hash 新密碼
        let new_password_hash = Self::hash_password(new_password)?;

        // 更新密碼並設置 must_change_password 標記
        sqlx::query(
            "UPDATE users SET password_hash = $1, must_change_password = true, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(target_user_id)
        .execute(pool)
        .await?;

        // 撤銷該用戶所有 refresh tokens（強制重新登入）
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(target_user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 忘記密碼 - 產生重設 token
    pub async fn forgot_password(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<(Uuid, String)>> {
        // 查詢用戶
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        // 即使用戶不存在也回傳成功（防止帳號枚舉攻擊）
        let user = match user {
            Some(u) => u,
            None => return Ok(None),
        };

        // 作廢該用戶的舊重設 tokens
        sqlx::query(
            "UPDATE password_reset_tokens SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL"
        )
        .bind(user.id)
        .execute(pool)
        .await?;

        // 產生新 token
        let token = Uuid::new_v4().to_string();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(1); // 1 小時內有效

        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(Some((user.id, token)))
    }

    /// 透過 token 重設密碼
    pub async fn reset_password_with_token(
        pool: &PgPool,
        token: &str,
        new_password: &str,
    ) -> Result<()> {
        let token_hash = Self::hash_token(token);

        // 查詢並驗證 token
        let token_record = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT * FROM password_reset_tokens 
            WHERE token_hash = $1 
              AND used_at IS NULL 
              AND expires_at > NOW()
            "#
        )
        .bind(&token_hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid or expired reset token".to_string()))?;

        // Hash 新密碼
        let new_password_hash = Self::hash_password(new_password)?;

        // 更新密碼
        sqlx::query(
            "UPDATE users SET password_hash = $1, must_change_password = false, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(token_record.user_id)
        .execute(pool)
        .await?;

        // 標記 token 已使用
        sqlx::query(
            "UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1"
        )
        .bind(token_record.id)
        .execute(pool)
        .await?;

        // 撤銷該用戶所有 refresh tokens（強制重新登入）
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(token_record.user_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
