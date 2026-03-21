use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    constants::REAUTH_EXPIRES_SECS,
    middleware::ReauthClaims,
    models::{LoginResponse, PasswordResetToken, User, UserResponse},
    AppError, Result,
};

use super::AuthService;

/// 常見弱密碼黑名單（至少 30 組），比對時統一轉為小寫
const COMMON_WEAK_PASSWORDS: &[&str] = &[
    "123456",
    "password",
    "qwerty",
    "12345678",
    "abc123",
    "password1",
    "admin",
    "letmein",
    "welcome",
    "monkey",
    "1234567890",
    "qwerty123",
    "iloveyou",
    "admin123",
    "password123",
    "changeme",
    "changeme123",
    "p@ssw0rd",
    "passw0rd",
    "123456789",
    "111111",
    "sunshine",
    "princess",
    "football",
    "shadow",
    "master",
    "dragon",
    "login",
    "baseball",
    "trustno1",
];

/// 檢查是否為常見弱密碼（大小寫不敏感）
fn is_common_weak_password(password: &str) -> bool {
    let lower = password.to_lowercase();
    COMMON_WEAK_PASSWORDS.iter().any(|&weak| weak == lower)
}

impl AuthService {
    /// 驗證密碼強度（SEC-10）
    /// 至少 10 字元，需包含大寫、小寫字母與數字，不得使用常見弱密碼
    pub fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 10 {
            return Err(AppError::Validation(
                "密碼長度至少需要 10 個字元".to_string(),
            ));
        }
        if !password.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(AppError::Validation(
                "密碼需包含至少一個大寫英文字母".to_string(),
            ));
        }
        if !password.chars().any(|c| c.is_ascii_lowercase()) {
            return Err(AppError::Validation(
                "密碼需包含至少一個小寫英文字母".to_string(),
            ));
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AppError::Validation(
                "密碼需包含至少一個數字".to_string(),
            ));
        }
        if is_common_weak_password(password) {
            return Err(AppError::Validation(
                "此密碼過於簡單，請使用更複雜的密碼".to_string(),
            ));
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
    /// 密碼更新後重新簽發 tokens，讓用戶不需要重新登入
    pub async fn change_own_password(
        pool: &PgPool,
        config: &Config,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<LoginResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !Self::verify_password(current_password, &user.password_hash)? {
            return Err(AppError::Validation("Current password is incorrect".to_string()));
        }

        Self::validate_password_strength(new_password)?;
        let new_password_hash = Self::hash_password(new_password)?;
        Self::update_password_in_db(pool, user_id, &new_password_hash, false).await?;
        Self::revoke_user_refresh_tokens(pool, user_id).await?;

        // 重新簽發 tokens（保持登入狀態）
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;
        let (access_token, expires_in) = Self::generate_access_token(config, &user, &roles, &permissions, None)?;
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: {
                let mut resp = UserResponse::from_user(&user, roles, permissions);
                resp.must_change_password = false;
                resp
            },
            must_change_password: false,
        })
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

    /// SEC-33：產生敏感操作二級認證用短期 token（密碼驗證後呼叫）
    pub fn generate_reauth_token(config: &Config, user_id: Uuid) -> Result<(String, i64)> {
        let now = Utc::now();
        let exp = now + Duration::seconds(REAUTH_EXPIRES_SECS);
        let claims = ReauthClaims {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            purpose: "reauth".to_string(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to create reauth token: {}", e)))?;
        Ok((token, REAUTH_EXPIRES_SECS))
    }

    /// SEC-33：驗證 reauth token 是否有效且屬於當前使用者
    pub fn verify_reauth_token(
        config: &Config,
        token: &str,
        expected_user_id: Uuid,
    ) -> Result<()> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        let token_data = decode::<ReauthClaims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AppError::Forbidden("請重新輸入密碼以確認敏感操作".to_string()))?;
        if token_data.claims.purpose != "reauth" || token_data.claims.sub != expected_user_id {
            return Err(AppError::Forbidden("請重新輸入密碼以確認敏感操作".to_string()));
        }
        Ok(())
    }

    /// Admin 重設他人密碼（不需驗證舊密碼）
    pub async fn reset_user_password(
        pool: &PgPool,
        target_user_id: Uuid,
        new_password: &str,
    ) -> Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
        )
        .bind(target_user_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Self::validate_password_strength(new_password)?;
        let new_password_hash = Self::hash_password(new_password)?;
        Self::update_password_in_db(pool, target_user_id, &new_password_hash, true).await?;
        Self::revoke_user_refresh_tokens(pool, target_user_id).await?;
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

        let new_password_hash = Self::hash_password(new_password)?;
        Self::update_password_in_db(pool, token_record.user_id, &new_password_hash, false).await?;

        // 標記 token 已使用
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(token_record.id)
            .execute(pool)
            .await?;

        Self::revoke_user_refresh_tokens(pool, token_record.user_id).await?;
        Ok(())
    }
}
