use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    middleware::Claims,
    models::{LoginResponse, RefreshToken, User, UserResponse},
    AppError, Result,
};

use super::AuthService;

impl AuthService {
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
            user: UserResponse::from_user(&user, roles, permissions),
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
            user: UserResponse::from_user(&user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// 停止模擬登入，恢復管理員身分
    /// 查詢管理員帳號並簽發正常 token（不含 impersonated_by）
    pub async fn impersonate_restore(
        pool: &PgPool,
        config: &Config,
        admin_user_id: Uuid,
    ) -> Result<LoginResponse> {
        // 查詢管理員帳號
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(admin_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Admin user not found or inactive".to_string()))?;

        // 獲取管理員的角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成正常的 JWT（不含 impersonated_by，使用正常有效期）
        let (access_token, expires_in) = Self::generate_access_token(
            config, &user, &roles, &permissions, None
        )?;

        // 生成 Refresh Token
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_user(&user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// 生成 Access Token
    pub(super) fn generate_access_token(
        config: &Config,
        user: &User,
        roles: &[String],
        permissions: &[String],
        impersonated_by: Option<Uuid>,
    ) -> Result<(String, i64)> {
        let now = Utc::now();
        // 模擬登入時限制有效期為 30 分鐘（SEC-11）
        // 否則使用 jwt_expiration_seconds（SEC-25，預設 15 分鐘）
        let expires_in = if impersonated_by.is_some() {
            1800 // 30 分鐘
        } else {
            config.jwt_expiration_seconds
        };
        let exp = now + Duration::seconds(expires_in);

        // Cookie 限制：Set-Cookie 的 name+value 須 ≤ 4096 字元。Admin 擁有全部權限時
        // JWT 會過大，導致瀏覽器忽略 cookie、登入失敗。Admin 的 has_permission 會直接
        // 回傳 true，故可省略 permissions 以縮小 JWT。
        let is_admin = roles
            .iter()
            .any(|r| r == "SYSTEM_ADMIN" || r == "admin");
        let claims_permissions: Vec<String> = if is_admin {
            vec![]
        } else {
            permissions.to_vec()
        };

        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            roles: roles.to_vec(),
            permissions: claims_permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
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
    pub(super) async fn generate_refresh_token(
        pool: &PgPool,
        user_id: Uuid,
        config: &Config,
    ) -> Result<String> {
        // SEC-34: 使用 CSPRNG 產生 256-bit 高熵 Refresh Token
        use rand::RngCore;
        let mut token_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut token_bytes);
        let token = token_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
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
    pub(super) fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 撤銷用戶的所有有效 refresh tokens（密碼變更/重設後強制重新登入）
    pub(super) async fn revoke_user_refresh_tokens(pool: &PgPool, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 更新用戶密碼 hash（含 must_change_password 標記）
    pub(super) async fn update_password_in_db(
        pool: &PgPool,
        user_id: Uuid,
        password_hash: &str,
        must_change_password: bool,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE users SET password_hash = $1, must_change_password = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(password_hash)
        .bind(must_change_password)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
