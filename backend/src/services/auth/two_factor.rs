use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    config::Config,
    constants::TWO_FA_TEMP_EXPIRES_SECS,
    middleware::ActorContext,
    models::{audit_diff::DataDiff, User},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// R66-B2：把 TOTP secret（base32）以 AEAD 加密成信封字串存 DB（AAD = user_id）。
    fn encrypt_totp_secret(config: &Config, user_id: Uuid, secret_b32: &str) -> Result<String> {
        let key = config.encryption_key.as_ref().ok_or_else(|| {
            AppError::Internal(
                "ENCRYPTION_KEY 未設定，無法加密 TOTP secret（請設定金鑰後再啟用 2FA）".into(),
            )
        })?;
        key.encrypt(secret_b32.as_bytes(), user_id.as_bytes())
    }

    /// R66-B2：解出 DB 中的 TOTP secret（base32）。過渡期相容 legacy 明文（無 `:` 前綴）。
    fn decrypt_totp_secret(
        config: &Config,
        user_id: Uuid,
        stored: &str,
    ) -> Result<Zeroizing<String>> {
        if !crate::utils::crypto::is_encrypted_envelope(stored) {
            // backfill 前的 legacy 明文 base32：直接回傳（backfill 完成後可移除此分支）。
            return Ok(Zeroizing::new(stored.to_string()));
        }
        let key = config.encryption_key.as_ref().ok_or_else(|| {
            AppError::Internal("ENCRYPTION_KEY 未設定，無法解密 TOTP secret".into())
        })?;
        let plaintext = key.decrypt(stored, user_id.as_bytes())?;
        // 借用驗證 UTF-8、立即把唯一複本包進 Zeroizing<String>：不留未清除的明文 secret 複本
        // （plaintext 為 Zeroizing<Vec<u8>>，drop 時自動清零）。
        let s = std::str::from_utf8(&plaintext)
            .map_err(|_| AppError::Internal("TOTP secret 解密結果非 UTF-8".into()))?;
        Ok(Zeroizing::new(s.to_string()))
    }

    /// 產生 TOTP secret 並存入 DB（尚未啟用，等待 confirm）
    pub async fn generate_totp_setup(
        pool: &PgPool,
        config: &Config,
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

        // 明文 base32 secret 以 Zeroizing 持有 → drop 時清零，不殘留 heap。
        let secret_b32 = Zeroizing::new(secret.to_encoded().to_string());

        // 產生 10 組備用碼
        let backup_codes: Vec<String> = (0..10)
            .map(|_| {
                use rand::RngExt;
                let mut rng = rand::rng();
                format!("{:08}", rng.random_range(10_000_000u32..99_999_999u32))
            })
            .collect();

        // hash 備用碼存入 DB
        let hashed_codes: Vec<String> = backup_codes
            .iter()
            .map(|c| Self::hash_backup_code(c))
            .collect();

        // R66-B2：AEAD 加密後存入（取代原明文 base32），AAD 綁 user_id。
        let stored_secret = Self::encrypt_totp_secret(config, user_id, &secret_b32)?;

        // 存入 DB（totp_enabled 仍為 false）
        sqlx::query(
            r#"UPDATE users SET totp_secret_encrypted = $1, totp_backup_codes = $2, updated_at = NOW() WHERE id = $3"#,
        )
        .bind(&stored_secret)
        .bind(&hashed_codes)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok((otpauth_uri, backup_codes))
    }

    /// 驗證 TOTP code 並正式啟用 2FA — Service-driven audit (SECURITY)
    pub async fn confirm_totp_setup(
        pool: &PgPool,
        config: &Config,
        actor: &ActorContext,
        user_id: Uuid,
        code: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let stored = before
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::BusinessRule("尚未產生 2FA secret，請先呼叫 setup".into()))?;
        let secret = Self::decrypt_totp_secret(config, user_id, stored)?;

        // 啟用流程：驗證即可（step 防重放僅登入端需要）
        let _ = Self::verify_totp_code(&secret, code)?;

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
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
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
        config: &Config,
        actor: &ActorContext,
        user_id: Uuid,
        code: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !before.totp_enabled {
            return Err(AppError::BusinessRule("2FA 未啟用".into()));
        }

        let stored = before
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::Internal("2FA enabled but no secret".into()))?;

        // 驗證 TOTP 或備用碼。
        //
        // Gemini PR #168 critical 指出：外層 tx 已持 users row FOR UPDATE，呼叫
        // `verify_backup_code(pool, ...)` 會 deadlock（helper 用 pool 新連線 UPDATE
        // 同一 row）。disable 流程下方 UPDATE 即將把 `totp_backup_codes` 設為 NULL，
        // 無需額外消耗單一備用碼；改用純 in-memory 比對。
        //
        // TOTP 需解密（解密失敗不致命）；備用碼不需解密 → 金鑰壞時仍可用備用碼停用 2FA。
        let totp_ok = match Self::decrypt_totp_secret(config, user_id, stored) {
            Ok(secret) => Self::verify_totp_code(&secret, code).is_ok(),
            Err(_) => false,
        };
        if !totp_ok && Self::find_matching_backup_code(&before.totp_backup_codes, code).is_none() {
            return Err(AppError::Validation("驗證碼錯誤".into()));
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
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
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

        let data = decode::<serde_json::Value>(token, &config.jwt_keys.decoding, &validation)
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

    /// 驗證 2FA 登入（temp token + TOTP/backup code + 防重放消耗），回傳已驗證的 `User`。
    ///
    /// High-3 (#207)：本函式**不**簽發 token。token 簽發改由 handler 在
    /// `create_session` / `end_excess_sessions`（SEC-28 併發上限）之後執行
    /// （session-before-token），與密碼登入路徑一致，避免 session 建立失敗時留孤兒 token。
    pub async fn verify_2fa_and_load_user(
        pool: &PgPool,
        config: &Config,
        temp_token: &str,
        code: &str,
        ip: Option<&str>,
    ) -> Result<User> {
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

        let stored = user
            .totp_secret_encrypted
            .as_deref()
            .ok_or_else(|| AppError::Internal("2FA enabled but no secret".into()))?;

        // 嘗試 TOTP code；失敗（**含解密失敗**）則嘗試 backup code。backup code 僅需 hash 列、
        // 不需解密 → 金鑰壞 / 信封損毀時仍可用 backup code 救援登入（不被解密失敗的 `?` 擋住）。
        // CSO #3 防重放：TOTP 成功時取得匹配的 time-step，拒絕 step <= 上次已用者
        // （同一 code 在 ±1 窗 ≈90s 內不可重複使用）。backup code 為一次性、走原消耗邏輯。
        let totp_step: Option<u64> = match Self::decrypt_totp_secret(config, user.id, stored) {
            Ok(secret) => Self::verify_totp_code(&secret, code).ok(),
            Err(_) => None,
        };
        let mut consumed_totp_step: Option<i64> = None;
        let verify_result = match totp_step {
            Some(step) => {
                let step = step as i64;
                if user.last_totp_step.is_some_and(|last| step <= last) {
                    // 已用過的 step → 視為重放，拒絕（不消耗、不更新）
                    Err(AppError::Validation("驗證碼錯誤或已過期".into()))
                } else {
                    consumed_totp_step = Some(step);
                    Ok(())
                }
            }
            None => Self::verify_backup_code(pool, user_id, &user.totp_backup_codes, code).await,
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

        // 更新最後登入時間；TOTP 登入時一併記錄已用 step（防重放，CSO #3）。
        // CSO #3b（Gemini PR #524）：原為「讀→檢查→寫」三步無鎖，並發下兩請求用同一 code
        // 皆能通過上方 step<=last 檢查 → 防重放被繞過（TOCTOU）。改用樂觀鎖：把單調遞增條件
        // 下放到 UPDATE 的 WHERE（last_totp_step < $2），DB 層原子保證單一 step 只成功一次。
        // - consumed_totp_step = Some(step)（TOTP 登入）：WHERE 需 step 嚴格大於現值；
        //   rows_affected == 0 代表被並發搶先（重放）→ 驗證失敗。
        // - consumed_totp_step = None（backup-code 登入）：$2 IS NULL 分支照常更新 last_login_at，
        //   不動 last_totp_step、不受並發影響。
        let rows_affected = sqlx::query(
            "UPDATE users SET last_login_at = NOW(), \
                 last_totp_step = COALESCE($2, last_totp_step) \
             WHERE id = $1 \
               AND ($2::BIGINT IS NULL OR last_totp_step IS NULL OR last_totp_step < $2)",
        )
        .bind(user.id)
        .bind(consumed_totp_step)
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            // 並發請求已搶先消耗此 TOTP step → 視為重放，拒絕（記一筆 2FA 失敗）
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

        Ok(user)
    }

    /// TOTP 時間步長（秒）。與 TOTP::new 第 4 參數一致。
    const TOTP_PERIOD_SECS: u64 = 30;

    /// 驗證 TOTP code（允許 ±1 時間步長），回傳「匹配的時間步」(unix_time / period)。
    ///
    /// 回傳 step 供呼叫端做防重放：登入端記錄最後成功 step，拒絕 step <= 已記錄者，
    /// 避免同一 code 在 ±1 窗（≈90s）內被重複使用（CSO #3）。
    fn verify_totp_code(secret_b32: &str, code: &str) -> Result<u64> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use totp_rs::{Algorithm, Secret, TOTP};

        let secret = Secret::Encoded(secret_b32.to_string());
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            Self::TOTP_PERIOD_SECS,
            secret
                .to_bytes()
                .map_err(|e| AppError::Internal(format!("TOTP secret decode: {e}")))?,
            None,
            String::new(),
        )
        .map_err(|e| AppError::Internal(format!("TOTP init: {e}")))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Internal(format!("TOTP time: {e}")))?
            .as_secs();
        let current_step = now / Self::TOTP_PERIOD_SECS;

        // 保留原 ±1 step 容忍（時鐘漂移），但用 generate 對「每個 step 精確比對」。
        // 不能用 totp.check(code, t)——它本身帶 ±1 skew，會讓 current code 在 current+1
        // 也誤判為真，回傳錯誤的較高 step。回傳精確匹配的 step。
        for step in [
            current_step.saturating_sub(1),
            current_step,
            current_step + 1,
        ] {
            if totp.generate(step * Self::TOTP_PERIOD_SECS) == code {
                return Ok(step);
            }
        }
        Err(AppError::Validation("驗證碼錯誤或已過期".into()))
    }

    /// 純 in-memory 驗證一次性備用碼，回傳匹配位置（若符合）。
    /// 不 touch DB，供 Service-driven tx 內避免死鎖使用。
    ///
    /// Gemini PR #168 指出：`verify_backup_code` 在 tx 內被呼叫時會 deadlock
    /// （外層 tx 已持 users row FOR UPDATE，該 helper 用 pool 新連線 UPDATE
    /// 同一 row → 永久等待）。本 helper 僅做密碼比對，不寫 DB。
    fn find_matching_backup_code(stored_codes: &Option<Vec<String>>, code: &str) -> Option<usize> {
        let codes = stored_codes.as_ref()?;
        codes.iter().position(|stored| {
            if stored.starts_with("$argon2") {
                use argon2::{
                    password_hash::{PasswordHash, PasswordVerifier},
                    Argon2,
                };
                PasswordHash::new(stored)
                    .ok()
                    .and_then(|h| Argon2::default().verify_password(code.as_bytes(), &h).ok())
                    .is_some()
            } else {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(code.as_bytes());
                let legacy_hash = hex::encode(hasher.finalize());
                stored == &legacy_hash
            }
        })
    }

    /// 驗證並消耗一次性備用碼（pool-based：供 2FA login 路徑使用）
    ///
    /// 呼叫端必須在 tx 外使用；disable_totp 路徑改用 `find_matching_backup_code`
    /// 純驗證 + 同 tx 內把 `totp_backup_codes` 設為 NULL（見 `disable_totp`）。
    async fn verify_backup_code(
        pool: &PgPool,
        user_id: Uuid,
        stored_codes: &Option<Vec<String>>,
        code: &str,
    ) -> Result<()> {
        if stored_codes.as_ref().is_none() {
            return Err(AppError::Validation("驗證碼錯誤".into()));
        }
        let idx = Self::find_matching_backup_code(stored_codes, code);
        let codes = stored_codes
            .as_ref()
            .ok_or_else(|| AppError::Validation("驗證碼錯誤".into()))?;

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
                hex::encode(hasher.finalize())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use totp_rs::{Algorithm, Secret, TOTP};

    /// 產生一個與 verify_totp_code 同參數的 TOTP，回傳 (secret_b32, 當前 code, 當前 step)。
    fn make_totp_now() -> (String, String, u64) {
        let secret = Secret::generate_secret();
        let b32 = secret.to_encoded().to_string();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            AuthService::TOTP_PERIOD_SECS,
            secret.to_bytes().expect("secret bytes"),
            None,
            String::new(),
        )
        .expect("totp");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs();
        let code = totp.generate(now);
        let step = now / AuthService::TOTP_PERIOD_SECS;
        (b32, code, step)
    }

    #[test]
    fn verify_totp_code_returns_matching_step() {
        let (secret, code, step) = make_totp_now();
        let got = AuthService::verify_totp_code(&secret, &code).expect("valid code");
        // 匹配 step 應為當前步（±1 容忍下，generate(now) 必落在 current）
        assert_eq!(got, step, "回傳 step 應等於當前 time-step");
    }

    #[test]
    fn verify_totp_code_rejects_wrong_code() {
        let (secret, _code, _step) = make_totp_now();
        assert!(
            AuthService::verify_totp_code(&secret, "000000").is_err()
                || AuthService::verify_totp_code(&secret, "999999").is_err(),
            "錯誤碼應被拒"
        );
    }

    /// CSO #3 防重放核心不變式：呼叫端用 step <= last_totp_step 判定重放。
    /// 此處驗證 step 單調性語意（同 code 取得同 step；故 step <= last 即可擋重放）。
    #[test]
    fn totp_step_is_stable_for_same_code_enabling_replay_guard() {
        let (secret, code, _step) = make_totp_now();
        let s1 = AuthService::verify_totp_code(&secret, &code).expect("first");
        let s2 = AuthService::verify_totp_code(&secret, &code).expect("second");
        assert_eq!(s1, s2, "同一 code 兩次驗證回傳相同 step");
        // 呼叫端邏輯：若 last_totp_step = s1，則第二次 s2 <= s1 → 判定重放拒絕
        let last_totp_step: Option<i64> = Some(s1 as i64);
        let replay_rejected = last_totp_step.is_some_and(|last| (s2 as i64) <= last);
        assert!(replay_rejected, "step <= last 應觸發重放拒絕");
    }
}
