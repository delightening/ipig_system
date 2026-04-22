use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    constants::TWO_FA_TEMP_EXPIRES_SECS,
    middleware::ActorContext,
    models::{audit_diff::DataDiff, LoginResponse, User, UserResponse},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// 產生 TOTP secret 並存入 DB（尚未啟用，等待 confirm）
    pub async fn generate_totp_setup(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
    ) -> Result<(String, Vec<String>)> {
        use totp_rs::{Algorithm, Secret, TOTP};

        let secret = Secret::generate_secret();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret
                .to_bytes()
                .map_err(|e| AppError::Internal(format!("TOTP secret error: {e}")))?,
            Some("iPig System".to_string()),
            email.to_string(),
        )
        .map_err(|e| AppError::Internal(format!("TOTP init error: {e}")))?;

        let otpauth_uri = totp.get_url();

        let secret_b32 = secret.to_encoded().to_string();

        // 產生 10 組備用碼
        let backup_codes: Vec<String> = (0..10)
            .map(|_| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                format!("{:08}", rng.gen_range(10_000_000u32..99_999_999u32))
            })
            .collect();

        // hash 備用碼存入 DB
        let hashed_codes: Vec<String> = backup_codes
            .iter()
            .map(|c| Self::hash_backup_code(c))
            .collect();

        // 存入 DB（totp_enabled 仍為 false）
        sqlx::query(
            r#"UPDATE users SET totp_secret_encrypted = $1, totp_backup_codes = $2, updated_at = NOW() WHERE id = $3"#,
        )
        .bind(&secret_b32)
        .bind(&hashed_codes)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok((otpauth_uri, backup_codes))
    }

    /// 驗證 TOTP code 並正式啟用 2FA — Service-driven audit (SECURITY)
    pub async fn confirm_totp_setup(
        pool: &PgPool,
        actor: &ActorContext,
        user_id: Uuid,
        code: &str,
    ) -> Result<()> {
        let _actor_user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let secret = before
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::BusinessRule("尚未產生 2FA secret，請先呼叫 setup".into()))?;

        Self::verify_totp_code(secret, code)?;

        let after = sqlx::query_as::<_, User>(
            "UPDATE users SET totp_enabled = true, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        // SECURITY audit：2FA 啟用（User 的 AuditRedact 已遮蔽 totp_secret / backup_codes）
        let display = format!("{} <{}>", after.display_name, after.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "TWO_FACTOR_ENABLED",
                entity: Some(AuditEntity::new("user", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        tracing::info!("[2FA] User {} enabled TOTP", user_id);
        Ok(())
    }

    /// 停用 2FA — Service-driven audit (SECURITY)
    pub async fn disable_totp(
        pool: &PgPool,
        actor: &ActorContext,
        user_id: Uuid,
        code: &str,
    ) -> Result<()> {
        let _actor_user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !before.totp_enabled {
            return Err(AppError::BusinessRule("2FA 未啟用".into()));
        }

        let secret = before
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::Internal("2FA enabled but no secret".into()))?;

        // 驗證 TOTP 或備用碼（verify_backup_code 接 pool，但只做 SELECT 與原子 UPDATE
        // 無關，此處仍在 tx 外驗證可接受；後續 R26-X 若要全 tx 化再處理）
        if Self::verify_totp_code(secret, code).is_err() {
            Self::verify_backup_code(pool, user_id, &before.totp_backup_codes, code).await?;
        }

        let after = sqlx::query_as::<_, User>(
            "UPDATE users SET totp_enabled = false, totp_secret_encrypted = NULL, totp_backup_codes = NULL, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} <{}>", after.display_name, after.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "TWO_FACTOR_DISABLED",
                entity: Some(AuditEntity::new("user", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        tracing::info!("[2FA] User {} disabled TOTP", user_id);
        Ok(())
    }

    /// 產生 2FA pending temp JWT（密碼已驗證，但 2FA 未完成）
    pub fn generate_2fa_temp_token(config: &Config, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = (now + Duration::seconds(TWO_FA_TEMP_EXPIRES_SECS)).timestamp() as usize;

        let claims = serde_json::json!({
            "sub": user_id.to_string(),
            "purpose": "2fa_pending",
            "exp": exp,
            "iat": now.timestamp(),
        });

        encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &config.jwt_keys.encoding,
        )
        .map_err(|e| AppError::Internal(format!("JWT encode error: {e}")))
    }

    /// 解析 2FA temp token 取得 user_id
    fn decode_2fa_temp_token(config: &Config, token: &str) -> Result<Uuid> {
        // SEC-AUDIT-003: 明確指定 ES256 演算法，防止 algorithm substitution 攻擊
        // SEC-UPG: 升級至 ES256（ECDSA P-256）非對稱簽章
        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = true;
        // 2FA temp token 不含標準 aud/iss，跳過這些驗證
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["exp"]);

        let data = decode::<serde_json::Value>(
            token,
            &config.jwt_keys.decoding,
            &validation,
        )
        .map_err(|_| AppError::Validation("2FA 驗證碼已過期或無效".into()))?;

        let purpose = data.claims.get("purpose").and_then(|v| v.as_str());
        if purpose != Some("2fa_pending") {
            return Err(AppError::Validation("無效的 2FA token".into()));
        }

        let sub = data
            .claims
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Token 缺少 sub".into()))?;

        Uuid::parse_str(sub).map_err(|_| AppError::Validation("Token sub 無效".into()))
    }

    /// 使用 temp_token + TOTP code 完成 2FA 登入，回傳正式 LoginResponse
    pub async fn complete_2fa_login(
        pool: &PgPool,
        config: &Config,
        temp_token: &str,
        code: &str,
        ip: Option<&str>,
    ) -> Result<LoginResponse> {
        let user_id = Self::decode_2fa_temp_token(config, temp_token)?;

        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::Validation("使用者不存在或已停用".into()))?;

        // 應用層 2FA 嘗試次數限制：5 分鐘內失敗 >= 5 次即拒絕
        let fail_count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM login_events
               WHERE email = $1
                 AND event_type = '2fa_failure'
                 AND created_at > NOW() - INTERVAL '5 minutes'"#,
        )
        .bind(&user.email)
        .fetch_one(pool)
        .await?;

        if fail_count >= 5 {
            return Err(AppError::TooManyRequests(
                "2FA 驗證失敗次數過多，請重新登入後再試".to_string(),
            ));
        }

        let secret = user
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::Internal("2FA enabled but no secret".into()))?;

        // 嘗試 TOTP code；若失敗則嘗試 backup code
        let verify_result = if Self::verify_totp_code(secret, code).is_ok() {
            Ok(())
        } else {
            Self::verify_backup_code(pool, user_id, &user.totp_backup_codes, code).await
        };

        if verify_result.is_err() {
            let _ = sqlx::query(
                r#"INSERT INTO login_events (id, user_id, email, event_type, ip_address, created_at)
                   VALUES ($1, $2, $3, '2fa_failure', $4::INET, NOW())"#,
            )
            .bind(Uuid::new_v4())
            .bind(user.id)
            .bind(&user.email)
            .bind(ip)
            .execute(pool)
            .await;
            return Err(AppError::Validation("驗證碼錯誤或已過期".into()));
        }

        // 更新最後登入時間
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(pool)
            .await?;

        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;
        let (access_token, expires_in) =
            Self::generate_access_token(config, &user, &roles, &permissions, None)?;
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

    /// 驗證 TOTP code（允許 ±1 時間步長）
    fn verify_totp_code(secret_b32: &str, code: &str) -> Result<()> {
        use totp_rs::{Algorithm, Secret, TOTP};

        let secret = Secret::Encoded(secret_b32.to_string());
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret
                .to_bytes()
                .map_err(|e| AppError::Internal(format!("TOTP secret decode: {e}")))?,
            None,
            String::new(),
        )
        .map_err(|e| AppError::Internal(format!("TOTP init: {e}")))?;

        if totp
            .check_current(code)
            .map_err(|e| AppError::Internal(format!("TOTP check: {e}")))?
        {
            Ok(())
        } else {
            Err(AppError::Validation("驗證碼錯誤或已過期".into()))
        }
    }

    /// 驗證並消耗一次性備用碼
    async fn verify_backup_code(
        pool: &PgPool,
        user_id: Uuid,
        stored_codes: &Option<Vec<String>>,
        code: &str,
    ) -> Result<()> {
        let codes = stored_codes
            .as_ref()
            .ok_or_else(|| AppError::Validation("驗證碼錯誤".into()))?;

        // M5: 支援新（Argon2）與舊（SHA-256 hex）格式，逐一比對
        let idx = codes.iter().position(|stored| {
            if stored.starts_with("$argon2") {
                // Argon2 格式
                use argon2::{
                    password_hash::{PasswordHash, PasswordVerifier},
                    Argon2,
                };
                PasswordHash::new(stored)
                    .ok()
                    .and_then(|h| Argon2::default().verify_password(code.as_bytes(), &h).ok())
                    .is_some()
            } else {
                // 舊 SHA-256 格式（向後相容，下次重設 2FA 會自動升級）
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(code.as_bytes());
                let legacy_hash = format!("{:x}", hasher.finalize());
                stored == &legacy_hash
            }
        });

        match idx {
            Some(i) => {
                let mut remaining = codes.clone();
                remaining.remove(i);
                sqlx::query("UPDATE users SET totp_backup_codes = $1 WHERE id = $2")
                    .bind(&remaining)
                    .bind(user_id)
                    .execute(pool)
                    .await?;
                tracing::info!(
                    "[2FA] User {} used backup code (remaining: {})",
                    user_id,
                    remaining.len()
                );
                Ok(())
            }
            None => Err(AppError::Validation("驗證碼錯誤".into())),
        }
    }

    /// M5: 使用 Argon2id 加鹽雜湊備用碼（防止預算彩虹表）
    fn hash_backup_code(code: &str) -> String {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(code.as_bytes(), &salt)
            .map(|h| h.to_string())
            .unwrap_or_else(|_| {
                // 極端情況 fallback（不應發生）：使用 SHA-256
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(code.as_bytes());
                format!("{:x}", hasher.finalize())
            })
    }
}
