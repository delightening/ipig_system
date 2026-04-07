use sqlx::PgPool;

use crate::{
    config::Config,
    models::{LoginRequest, LoginResponse, User, UserResponse},
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// 驗證 email + 密碼（不產生 token，供 handler 決定是否走 2FA）
    pub async fn validate_credentials(pool: &PgPool, config: &Config, req: &LoginRequest) -> Result<User> {
        // H4: 使用 pg_advisory_xact_lock 串行化相同帳號的並發登入嘗試，防止 TOCTOU 繞過帳號鎖定
        let mut tx = pool.begin().await?;

        // SEC-20: 帳號鎖定檢查（在 advisory lock 保護下執行，確保 count 原子性）
        if !config.disable_account_lockout {
            sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
                .bind(&req.email)
                .execute(&mut *tx)
                .await?;

            let (fail_count,): (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM login_events
                WHERE email = $1
                  AND event_type = 'login_failure'
                  AND created_at > NOW() - make_interval(mins => $2::integer)
                "#,
            )
            .bind(&req.email)
            .bind(config.account_lockout_duration_minutes)
            .fetch_one(&mut *tx)
            .await?;

            if fail_count >= config.account_lockout_max_attempts {
                tx.commit().await.ok();
                tracing::warn!(
                    "[Auth] 帳號 {} 因連續失敗 {} 次被暫時鎖定",
                    req.email, fail_count
                );
                return Err(AppError::Validation(
                    format!("帳號已暫時鎖定，請 {} 分鐘後再試", config.account_lockout_duration_minutes),
                ));
            }
        }

        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(&req.email)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::InvalidCredentials("Invalid email or password".to_string()))?;

        use argon2::{password_hash::PasswordHash, Argon2, password_hash::PasswordVerifier};

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;

        tx.commit().await?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::InvalidCredentials("Invalid email or password".to_string()))?;

        // 帳號到期日檢查
        if let Some(expires_at) = user.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(AppError::Validation(
                    "帳號已過期，請聯繫系統管理員".to_string(),
                ));
            }
        }

        Ok(user)
    }

    /// 密碼驗證通過後，產生完整的 LoginResponse
    pub async fn issue_login_tokens(
        pool: &PgPool,
        config: &Config,
        user: &User,
    ) -> Result<LoginResponse> {
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(pool)
            .await?;

        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;
        let (access_token, expires_in) = Self::generate_access_token(config, user, &roles, &permissions, None)?;
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_user(user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// 登入（向後相容：validate_credentials + issue_login_tokens + 2FA 檢查留給 handler）
    pub async fn login(
        pool: &PgPool,
        config: &Config,
        req: &LoginRequest,
    ) -> Result<LoginResponse> {
        let user = Self::validate_credentials(pool, config, req).await?;
        Self::issue_login_tokens(pool, config, &user).await
    }

    /// 登出（撤銷 refresh token）
    pub async fn logout(pool: &PgPool, user_id: uuid::Uuid) -> Result<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 獲取用戶的角色和權限
    pub async fn get_user_roles_permissions(
        pool: &PgPool,
        user_id: uuid::Uuid,
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
}
