use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{LoginRequest, LoginResponse, User, UserResponse},
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// 驗證 email + 密碼（不產生 token，供 handler 決定是否走 2FA）
    ///
    /// CRIT-01 + MED-02: 失敗事件在 advisory lock 事務內原子性寫入，
    /// 防止並發請求繞過帳號鎖定計數。tx.commit() 移至密碼驗證之後，
    /// 確保 advisory lock 在整個驗證過程中持續保持。
    pub async fn validate_credentials(
        pool: &PgPool,
        config: &Config,
        req: &LoginRequest,
        ip: Option<&str>,
    ) -> Result<User> {
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
                    req.email,
                    fail_count
                );
                return Err(AppError::Validation(format!(
                    "帳號已暫時鎖定，請 {} 分鐘後再試",
                    config.account_lockout_duration_minutes
                )));
            }
        }

        let user_opt =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND is_active = true")
                .bind(&req.email)
                .fetch_optional(&mut *tx)
                .await?;

        let user = match user_opt {
            Some(u) => u,
            None => {
                // SEC-AUDIT-002: 執行虛擬 Argon2 驗證，確保「使用者不存在」與「密碼錯誤」
                // 的回應時間一致，防止透過 timing side-channel 枚舉有效 email
                let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$dGhpc2lzYWR1bW15c2FsdA$ZKhM8GZ8MJD3E5VNrp0MWuVIL+rCBjKNHdXMaGW4+A8";
                use argon2::{
                    password_hash::PasswordHash, password_hash::PasswordVerifier, Argon2,
                };
                if let Ok(parsed) = PasswordHash::new(dummy_hash) {
                    let _ = Argon2::default().verify_password(req.password.as_bytes(), &parsed);
                }

                // CRIT-01: 原子性記錄失敗（使用者不存在），確保與 fail_count 讀取在同一事務
                Self::insert_failure_event_in_tx(&mut tx, None, &req.email, ip)
                    .await
                    .ok();
                tx.commit().await.ok();
                return Err(AppError::InvalidCredentials(
                    "Invalid email or password".to_string(),
                ));
            }
        };

        use argon2::{password_hash::PasswordHash, password_hash::PasswordVerifier, Argon2};

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;

        // MED-02: 密碼驗證完成前 advisory lock 持續保持（tx 尚未 commit）
        let verify_result =
            Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash);

        if verify_result.is_err() {
            // CRIT-01: 原子性記錄失敗（密碼錯誤），與 fail_count 讀取在同一 advisory lock 範圍內
            Self::insert_failure_event_in_tx(&mut tx, Some(user.id), &req.email, ip)
                .await
                .ok();
            tx.commit().await?;
            return Err(AppError::InvalidCredentials(
                "Invalid email or password".to_string(),
            ));
        }

        tx.commit().await?;

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

    /// 在現有事務中插入登入失敗事件（極簡版本）
    /// 供帳號鎖定計數的原子性記錄使用，完整的 GeoIP/異常偵測
    /// 由 LoginTracker::log_failure 在 handler 層非同步處理。
    async fn insert_failure_event_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Option<Uuid>,
        email: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO login_events (id, user_id, email, event_type, ip_address, created_at)
               VALUES ($1, $2, $3, 'login_failure', $4::INET, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .execute(&mut **tx)
        .await?;
        Ok(())
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
        let (access_token, expires_in) =
            Self::generate_access_token(config, user, &roles, &permissions, None)?;
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
        let user = Self::validate_credentials(pool, config, req, None).await?;
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
            "#,
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
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok((roles, permissions))
    }
}
