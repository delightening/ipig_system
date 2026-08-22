use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    constants::{
        LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR, LOGIN_EVENT_LOCKOUT_RESET, REAUTH_EXPIRES_SECS,
        STEP_UP_LOCKOUT_MAX_ATTEMPTS, STEP_UP_LOCKOUT_WINDOW_MINS,
    },
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
            return Err(AppError::Validation("密碼需包含至少一個數字".to_string()));
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

    /// 修改自己的密碼（需驗證舊密碼）— Service-driven audit (SECURITY)
    /// 密碼更新後重新簽發 tokens，讓用戶不需要重新登入。
    ///
    /// Audit 語意：actor 即 target user（自行變更），`AuditRedact` 會遮蔽
    /// password_hash / totp_secret / backup_codes 等敏感欄位。
    pub async fn change_own_password(
        pool: &PgPool,
        config: &Config,
        actor: &crate::middleware::ActorContext,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<LoginResponse> {
        actor.require_user()?;
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !Self::verify_password(current_password, &user.password_hash)? {
            return Err(AppError::Validation(
                "Current password is incorrect".to_string(),
            ));
        }

        Self::validate_password_strength(new_password)?;
        let new_password_hash = Self::hash_password(new_password)?;

        // password hash 更新 + refresh token 撤銷 + SECURITY audit 全部同 tx
        let mut tx = pool.begin().await?;
        Self::update_password_in_db_tx(&mut tx, user_id, &new_password_hash, false).await?;
        Self::revoke_user_refresh_tokens_tx(&mut tx, user_id).await?;
        // CSO-r3 #3：撤銷此使用者所有未過期 access token（下方重發的新 token iat ≥ NOW 不受影響）
        sqlx::query("UPDATE users SET tokens_valid_after = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        // 密碼已變更 → 解除以舊密碼失敗次數累積的鎖定（同 tx，與密碼一起生效或一起回滾）
        Self::insert_lockout_reset_event_tx(&mut tx, user_id).await?;

        let after = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

        let display = format!("{} <{}>", after.display_name, after.email);
        crate::services::AuditService::log_activity_tx(
            &mut tx,
            actor,
            crate::services::audit::ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "PASSWORD_SELF_CHANGE",
                entity: Some(crate::services::audit::AuditEntity::new(
                    "user", after.id, &display,
                )),
                data_diff: Some(crate::models::audit_diff::DataDiff::compute(
                    Some(&user),
                    Some(&after),
                )),
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
            },
        )
        .await?;
        tx.commit().await?;

        // 重新簽發 tokens（保持登入狀態）
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;
        let (access_token, expires_in) =
            Self::generate_access_token(config, &user, &roles, &permissions, None)?;
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

    /// 透過用戶 ID 驗證密碼（用於電子簽章作廢等敏感 step-up 操作）
    ///
    /// CSO #4：密碼驗證失敗時寫一筆 `reauth_failure` security 事件，供監控 / 告警偵測
    /// 對 step-up 端點（如簽章作廢）的暴力嘗試（該路由僅 write-tier 限流 + admin 權限，
    /// 失敗本身原先無痕）。
    pub async fn verify_password_by_id(
        pool: &PgPool,
        user_id: Uuid,
        password: &str,
    ) -> Result<User> {
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // R66-B3: step-up 暴力破解防護 — 獨立計數器，count→verify→write 在單一
        // advisory-locked 事務內原子化（比照 `validate_credentials` 帳號鎖定的
        // CRIT-01/MED-02），確保同一 user 的並發請求序列化，不會都在 count<門檻時
        // 闖過而突破「最多 N 次」保證。只計 `reauth_failure`（不讀/不寫
        // `login_failure`），故鎖定 step-up 不影響登入、避免攻擊者用 step-up 失敗
        // 把使用者鎖出登入（DoS）。達門檻時即使密碼正確也擋下。
        let mut tx = pool.begin().await?;

        // 串行化同一 user 的並發 step-up 嘗試（key space 用 user_id，與登入的
        // email-hash 鎖獨立 — 兩者保護不同計數器、不互相阻塞）。
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
            .bind(user.id.to_string())
            .execute(&mut *tx)
            .await?;

        // 與登入鎖定一致：只計最後一筆解鎖標記之後的失敗。密碼一換，先前對舊密碼的
        // 猜測值即失效，不該再擋住合法使用者的敏感操作；管理員手動解鎖亦同。
        let recent_reauth_failures: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM login_events
               WHERE user_id = $1
                 AND event_type = 'reauth_failure'
                 AND created_at > NOW() - make_interval(mins => $2::integer)
                 AND created_at > COALESCE(
                       (SELECT MAX(created_at) FROM login_events
                         WHERE user_id = $1 AND event_type IN ($3, $4)),
                       '-infinity'::timestamptz)"#,
        )
        .bind(user.id)
        .bind(STEP_UP_LOCKOUT_WINDOW_MINS as i32)
        .bind(LOGIN_EVENT_LOCKOUT_RESET)
        .bind(LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR)
        .fetch_one(&mut *tx)
        .await?;

        if recent_reauth_failures >= STEP_UP_LOCKOUT_MAX_ATTEMPTS {
            // 尚未寫入任何資料，rollback 即釋放 advisory lock。
            tx.rollback().await.ok();
            tracing::warn!(
                "[Auth] step-up 密碼重驗鎖定 user={} reauth_failures={} window={}min",
                user.id,
                recent_reauth_failures,
                STEP_UP_LOCKOUT_WINDOW_MINS
            );
            return Err(AppError::Validation(format!(
                "密碼驗證失敗次數過多，請 {} 分鐘後再試",
                STEP_UP_LOCKOUT_WINDOW_MINS
            )));
        }

        if !Self::verify_password(password, &user.password_hash)? {
            // 失敗事件在同一 advisory-locked tx 內寫入，確保與 count 觀測一致；
            // best-effort 記錄，寫入失敗只告警、不改變「密碼錯誤」的回應。
            if let Err(e) = sqlx::query(
                r#"INSERT INTO login_events (id, user_id, email, event_type, failure_reason, created_at)
                   VALUES ($1, $2, $3, 'reauth_failure', $4, NOW())"#,
            )
            .bind(Uuid::new_v4())
            .bind(user.id)
            .bind(&user.email)
            .bind("password re-verification failed (sensitive step-up)")
            .execute(&mut *tx)
            .await
            {
                tracing::warn!("reauth_failure 事件寫入失敗 user={}: {e}", user.id);
            }
            if let Err(e) = tx.commit().await {
                tracing::warn!("step-up reauth tx commit 失敗 user={}: {e}", user.id);
            }
            return Err(AppError::Unauthorized);
        }

        tx.commit().await?;
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
            &Header::new(Algorithm::ES256),
            &claims,
            &config.jwt_keys.encoding,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create reauth token: {}", e)))?;
        Ok((token, REAUTH_EXPIRES_SECS))
    }

    /// SEC-33：驗證 reauth token 是否有效且屬於當前使用者
    pub fn verify_reauth_token(config: &Config, token: &str, expected_user_id: Uuid) -> Result<()> {
        // SEC-AUDIT-003: 明確指定 ES256 演算法，防止 algorithm substitution 攻擊
        // SEC-UPG: 升級至 ES256（ECDSA P-256）非對稱簽章
        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = true;
        // reauth token 不含 aud/iss，跳過這些驗證
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["exp", "sub"]);
        let token_data = decode::<ReauthClaims>(token, &config.jwt_keys.decoding, &validation)
            .map_err(|_| AppError::BusinessRule("請重新輸入密碼以確認敏感操作".to_string()))?;
        if token_data.claims.purpose != "reauth" || token_data.claims.sub != expected_user_id {
            return Err(AppError::BusinessRule(
                "請重新輸入密碼以確認敏感操作".to_string(),
            ));
        }
        Ok(())
    }

    /// Admin 重設他人密碼（不需驗證舊密碼）— Service-driven audit (SECURITY)
    ///
    /// Audit 關鍵：actor = 管理員，target = 被重設者；AuditRedact 會遮蔽 password_hash。
    pub async fn reset_user_password(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        target_user_id: Uuid,
        new_password: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        actor.require_user()?;

        // Gemini PR #168 HIGH：Argon2 hash 移到 tx 之外。
        // Argon2 刻意設計為 CPU-intensive（幾百 ms），若在 tx 內（且 FOR UPDATE
        // 持鎖狀態）執行，會拉長鎖持有時間 × 並發請求數，造成嚴重 throughput 問題。
        Self::validate_password_strength(new_password)?;
        let new_password_hash = Self::hash_password(new_password)?;

        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 FOR UPDATE")
            .bind(target_user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Self::update_password_in_db_tx(&mut tx, target_user_id, &new_password_hash, true).await?;
        Self::revoke_user_refresh_tokens_tx(&mut tx, target_user_id).await?;
        // CSO-r3 #3：撤銷被重設者所有未過期 access token
        sqlx::query("UPDATE users SET tokens_valid_after = NOW() WHERE id = $1")
            .bind(target_user_id)
            .execute(&mut *tx)
            .await?;
        // 密碼已變更 → 解除以舊密碼失敗次數累積的鎖定（管理員代為重設同樣適用）
        Self::insert_lockout_reset_event_tx(&mut tx, target_user_id).await?;

        let after = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(target_user_id)
            .fetch_one(&mut *tx)
            .await?;

        let display = format!("{} <{}>", after.display_name, after.email);
        crate::services::AuditService::log_activity_tx(
            &mut tx,
            actor,
            crate::services::audit::ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "PASSWORD_ADMIN_RESET",
                entity: Some(crate::services::audit::AuditEntity::new(
                    "user", after.id, &display,
                )),
                data_diff: Some(crate::models::audit_diff::DataDiff::compute(
                    Some(&before),
                    Some(&after),
                )),
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// 忘記密碼 - 產生重設 token
    pub async fn forgot_password(pool: &PgPool, email: &str) -> Result<Option<(Uuid, String)>> {
        // 查詢用戶
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND is_active = true")
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

        // M6: 使用 256-bit CSPRNG token 取代 UUID v4（UUID 僅 122 bit 有效熵）
        let token = {
            use rand::Rng;
            let mut bytes = [0u8; 32];
            rand::rng().fill_bytes(&mut bytes);
            use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
            URL_SAFE_NO_PAD.encode(bytes)
        };
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(1); // 1 小時內有效

        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
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

        // 先驗證 token（未鎖）取得 user_id；無效則直接回，避免對無效 token 做 Argon2（DoS 防護）
        let token_record = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT * FROM password_reset_tokens
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid or expired reset token".to_string()))?;

        Self::validate_password_strength(new_password)?;
        // Argon2 雜湊在 tx / 鎖之前計算，避免持鎖期間執行（gemini #168 規範）
        let new_password_hash = Self::hash_password(new_password)?;

        // 所有寫入 + 稽核同一 tx，確保原子性（密碼更新 / token 標記 / token 撤銷 /
        // tokens_valid_after / audit 任一失敗即整體回滾，gemini review）。
        let mut tx = pool.begin().await?;
        // 重新鎖定 token 並確認仍未使用，防同 token 並發重複重設
        let still_valid: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM password_reset_tokens WHERE id = $1 AND used_at IS NULL FOR UPDATE",
        )
        .bind(token_record.id)
        .fetch_optional(&mut *tx)
        .await?;
        if still_valid.is_none() {
            return Err(AppError::Validation(
                "Invalid or expired reset token".to_string(),
            ));
        }

        Self::update_password_in_db_tx(&mut tx, token_record.user_id, &new_password_hash, false)
            .await?;
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(token_record.id)
            .execute(&mut *tx)
            .await?;
        Self::revoke_user_refresh_tokens_tx(&mut tx, token_record.user_id).await?;
        // CSO-r3 #3：撤銷此使用者所有未過期 access token
        sqlx::query("UPDATE users SET tokens_valid_after = NOW() WHERE id = $1")
            .bind(token_record.user_id)
            .execute(&mut *tx)
            .await?;
        // 密碼已變更 → 解除以舊密碼失敗次數累積的鎖定。忘記密碼是被鎖使用者最常
        // 走的自救路徑，不解鎖等於重設完仍被擋滿整個視窗（2026-07-29 使用者回報）
        Self::insert_lockout_reset_event_tx(&mut tx, token_record.user_id).await?;

        // H4 + gemini：SECURITY 稽核。忘記密碼 token 無登入 actor → Anonymous（HMAC chain
        // actor 端 fallback SYSTEM_USER_ID）；entity 綁定 target user，讓 entity_id 可被
        // 稽核查詢（避免 entity_id NULL 查不到），與既有 PASSWORD_* 事件一致走 log_activity_tx。
        crate::services::AuditService::log_activity_tx(
            &mut tx,
            &crate::middleware::ActorContext::Anonymous,
            crate::services::audit::ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "PASSWORD_TOKEN_RESET",
                entity: Some(crate::services::audit::AuditEntity::new(
                    "user",
                    token_record.user_id,
                    "",
                )),
                data_diff: Some({
                    let mut dd = crate::models::audit_diff::DataDiff::empty();
                    dd.after = Some(serde_json::json!({ "target_user_id": token_record.user_id }));
                    dd
                }),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
