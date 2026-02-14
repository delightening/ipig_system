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
        let (access_token, expires_in) = Self::generate_access_token(config, &user, &roles, &permissions, None)?;

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
                entry_date: user.entry_date,
                position: user.position.clone(),
                aup_roles: user.aup_roles.clone(),
                years_experience: user.years_experience,
                trainings: user.trainings.0.clone(),
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
        let (access_token, expires_in) = Self::generate_access_token(config, &user, &roles, &permissions, None)?;
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
                entry_date: user.entry_date,
                position: user.position.clone(),
                aup_roles: user.aup_roles.clone(),
                years_experience: user.years_experience,
                trainings: user.trainings.0.clone(),
                roles,
                permissions,
            },
            must_change_password: user.must_change_password,
        })
    }

    /// 模擬登入（管理員專用）
    pub async fn impersonate(
        pool: &PgPool,
        config: &Config,
        admin_user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<LoginResponse> {
        // 查詢目標用戶
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(target_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Target user not found or inactive".to_string()))?;

        // 獲取角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成 JWT（包含 impersonated_by 資訊，有效期受限）
        let (access_token, expires_in) = Self::generate_access_token(
            config, &user, &roles, &permissions, Some(admin_user_id)
        )?;

        // 生成 Refresh Token
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        // 審計日誌：記錄模擬登入行為（SEC-11）
        tracing::warn!(
            "[Security] 模擬登入 - 管理員 {} 模擬登入為用戶 {} ({})",
            admin_user_id, user.email, user.id
        );

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
                entry_date: user.entry_date,
                position: user.position.clone(),
                aup_roles: user.aup_roles.clone(),
                years_experience: user.years_experience,
                trainings: user.trainings.0.clone(),
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
        impersonated_by: Option<Uuid>,
    ) -> Result<(String, i64)> {
        let now = Utc::now();
        // 模擬登入時限制有效期為 30 分鐘（SEC-11）
        let expires_in = if impersonated_by.is_some() {
            1800 // 30 分鐘
        } else {
            config.jwt_expiration_hours * 3600
        };
        let exp = now + Duration::seconds(expires_in);

        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            roles: roles.to_vec(),
            permissions: permissions.to_vec(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            impersonated_by,
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

    /// Hash token for storage（使用 SHA-256 密碼學雜湊）
    fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 驗證密碼強度（SEC-10）
    /// 至少 8 字元，需包含大寫、小寫字母與數字
    pub fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AppError::Validation("密碼長度至少需要 8 個字元".to_string()));
        }
        if !password.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(AppError::Validation("密碼需包含至少一個大寫英文字母".to_string()));
        }
        if !password.chars().any(|c| c.is_ascii_lowercase()) {
            return Err(AppError::Validation("密碼需包含至少一個小寫英文字母".to_string()));
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AppError::Validation("密碼需包含至少一個數字".to_string()));
        }
        Ok(())
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

        // 驗證新密碼強度
        Self::validate_password_strength(new_password)?;

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

    /// 透過用戶 ID 驗證密碼（用於電子簽章等操作）
    pub async fn verify_password_by_id(
        pool: &PgPool,
        user_id: Uuid,
        password: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !Self::verify_password(password, &user.password_hash)? {
            return Err(AppError::Unauthorized);
        }

        Ok(user)
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

        // 驗證新密碼強度
        Self::validate_password_strength(new_password)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // validate_password_strength 測試
    // ==========================================

    #[test]
    fn test_password_valid() {
        // 符合所有條件：8 字元以上 + 大寫 + 小寫 + 數字
        assert!(AuthService::validate_password_strength("Abcd1234").is_ok());
    }

    #[test]
    fn test_password_too_short() {
        assert!(AuthService::validate_password_strength("Ab1").is_err());
        assert!(AuthService::validate_password_strength("Abc1234").is_err()); // 7 字元
    }

    #[test]
    fn test_password_no_uppercase() {
        assert!(AuthService::validate_password_strength("abcd1234").is_err());
    }

    #[test]
    fn test_password_no_lowercase() {
        assert!(AuthService::validate_password_strength("ABCD1234").is_err());
    }

    #[test]
    fn test_password_no_digit() {
        assert!(AuthService::validate_password_strength("Abcdefgh").is_err());
    }

    #[test]
    fn test_password_empty() {
        assert!(AuthService::validate_password_strength("").is_err());
    }

    #[test]
    fn test_password_with_special_chars() {
        // 特殊字元不影響通過
        assert!(AuthService::validate_password_strength("Abcd123!@#").is_ok());
    }

    #[test]
    fn test_password_unicode() {
        // 含中文字元仍需滿足大小寫與數字
        assert!(AuthService::validate_password_strength("Ab12345中文").is_ok());
    }

    #[test]
    fn test_password_exact_8_chars() {
        // 恰好 8 字元
        assert!(AuthService::validate_password_strength("Abcdef1!").is_ok());
    }

    // ==========================================
    // hash_token 測試
    // ==========================================

    #[test]
    fn test_hash_token_deterministic() {
        let hash1 = AuthService::hash_token("test_token");
        let hash2 = AuthService::hash_token("test_token");
        assert_eq!(hash1, hash2, "相同輸入應產生相同 hash");
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = AuthService::hash_token("token_a");
        let hash2 = AuthService::hash_token("token_b");
        assert_ne!(hash1, hash2, "不同輸入應產生不同 hash");
    }

    #[test]
    fn test_hash_token_length() {
        // SHA-256 hex 輸出固定 64 字元
        let hash = AuthService::hash_token("any_token");
        assert_eq!(hash.len(), 64, "SHA-256 hex 應為 64 字元");
    }

    // ==========================================
    // hash_password / verify_password 測試
    // ==========================================

    #[test]
    fn test_hash_and_verify_password() {
        let password = "TestPass123";
        let hash = AuthService::hash_password(password).expect("hash 應成功");
        let is_valid = AuthService::verify_password(password, &hash).expect("verify 應成功");
        assert!(is_valid, "正確密碼應通過驗證");
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "TestPass123";
        let hash = AuthService::hash_password(password).expect("hash 應成功");
        let is_valid = AuthService::verify_password("WrongPass123", &hash).expect("verify 應成功");
        assert!(!is_valid, "錯誤密碼不應通過驗證");
    }

    #[test]
    fn test_hash_password_unique_salts() {
        let password = "SamePassword1";
        let hash1 = AuthService::hash_password(password).expect("hash 應成功");
        let hash2 = AuthService::hash_password(password).expect("hash 應成功");
        assert_ne!(hash1, hash2, "不同 salt 應產生不同 hash");
    }
}
