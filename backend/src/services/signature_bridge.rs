//! R30-27c：桌機 ↔ 手機簽名 bridge service
//!
//! 流程：
//! 1. 桌機 admin 觸發 mutation → service::start() 開 session（5min TTL）→
//!    回傳 `(session_id, mobile_token)`；桌機 dialog 把 token 編入 QR 給手機掃。
//! 2. 手機開 `/sign/:session_id?token=...`（公開 page）→ 完成簽名 →
//!    POST submit → service::submit() 驗 token、寫 payload、status='COMPLETED'。
//! 3. 桌機輪詢 GET /signing-bridge/:id/status →
//!    status=COMPLETED → service::consume() 取 payload + status='CONSUMED'，
//!    payload 串入 `mutation_signature` 送出 mutation。
//!
//! 安全設計：
//! - mobile_token 為 64 字元 crypto-random base64url；DB 存 SHA-256 hash（與
//!   `AuthService::hash_token` 同一 pattern；短命 + 單次使用，非長期 password）。
//! - status 由 PENDING → COMPLETED → CONSUMED 嚴格單向；submit 不能覆寫已 COMPLETED。
//! - consume 用 SELECT FOR UPDATE + status='COMPLETED' guard，避免 race condition。
//! - status 端點僅 owner 可讀，避免 mobile_token 洩漏即拿到別人 payload。

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    config::Config,
    services::AuthService,
    utils::crypto::{is_encrypted_envelope, EncryptionKey},
    AppError, Result,
};

/// session TTL — 短到讓 QR 截圖回放窗口很小，但足以給 admin 拿手機掃碼
pub const SIGNING_BRIDGE_TTL_MINUTES: i64 = 5;

/// COMPLETED 但未被桌機 consume 的 session 寬限期（分鐘）。
/// 桌機正常輪詢秒級即取走；超過此寬限視為棄單，由 GC 連 payload 明文一起刪除。
const COMPLETED_GRACE_MINUTES: i64 = 60;

/// purpose 常數（audit / debug 用，DB 約定 1-50 字元）
pub const PURPOSE_ROLE_CREATE: &str = "role.create";
pub const PURPOSE_ROLE_UPDATE: &str = "role.update";
pub const PURPOSE_ROLE_DELETE: &str = "role.delete";

/// service::start() 的回傳：session_id 與 plaintext mobile_token（後者只此一次回給桌機）
pub struct StartedSession {
    pub session_id: Uuid,
    pub mobile_token: String,
    pub expires_at: DateTime<Utc>,
}

/// service::consume() 取走的 payload
pub struct BridgePayload {
    pub payload: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
}

pub struct SignatureBridgeService;

impl SignatureBridgeService {
    /// 桌機開 session — 產 mobile_token、INSERT row、回傳給桌機編入 QR。
    pub async fn start(pool: &PgPool, user_id: Uuid, purpose: &str) -> Result<StartedSession> {
        if purpose.is_empty() || purpose.len() > 50 {
            return Err(AppError::Validation(
                "bridge purpose must be 1-50 chars".into(),
            ));
        }

        let mobile_token = generate_mobile_token();
        let token_hash = AuthService::hash_token(&mobile_token);
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(SIGNING_BRIDGE_TTL_MINUTES);

        sqlx::query(
            r#"INSERT INTO signature_bridge_sessions
                 (id, user_id, mobile_token_hash, purpose, expires_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(purpose)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(StartedSession {
            session_id,
            mobile_token,
            expires_at,
        })
    }

    /// 手機 submit — 驗 token、寫 payload、status='COMPLETED'。
    /// 公開端點：caller (handler) 不需 auth；token-bearer 即驗證身分。
    pub async fn submit(
        pool: &PgPool,
        config: &Config,
        session_id: Uuid,
        mobile_token: &str,
        payload: serde_json::Value,
    ) -> Result<()> {
        if mobile_token.is_empty() {
            return Err(AppError::Validation("mobile_token required".into()));
        }
        let token_hash = AuthService::hash_token(mobile_token);

        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE 鎖 row，避免 submit 與 consume race 出兩次。
        // 一併取 user_id 作為 R66-C6 加密 AAD（綁定 payload 到該 session/owner）。
        let row: Option<(String, DateTime<Utc>, Uuid)> = sqlx::query_as(
            r#"SELECT status, expires_at, user_id FROM signature_bridge_sessions
               WHERE id = $1 AND mobile_token_hash = $2 FOR UPDATE"#,
        )
        .bind(session_id)
        .bind(&token_hash)
        .fetch_optional(&mut *tx)
        .await?;

        let (status, expires_at, user_id) = row.ok_or_else(|| {
            AppError::NotFound("signature bridge session not found or token invalid".into())
        })?;

        if Utc::now() > expires_at {
            // lazy 標記 EXPIRED（cron 也會清；加 status='PENDING' guard 避免覆寫
            // 並行 transaction 已轉成 COMPLETED/CONSUMED 的 row — gemini review #298）
            sqlx::query(
                "UPDATE signature_bridge_sessions SET status = 'EXPIRED' \
                 WHERE id = $1 AND status = 'PENDING'",
            )
            .bind(session_id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Err(AppError::BadRequest(
                "signature bridge session expired".into(),
            ));
        }

        if status != "PENDING" {
            return Err(AppError::Conflict(format!(
                "signature bridge session already in status {}",
                status
            )));
        }

        // R66-C6：AEAD 加密 payload（含明文密碼 + 手寫筆跡）後才寫 DB；欄位現為 TEXT 信封。
        let envelope = encrypt_payload(
            config.encryption_key.as_ref(),
            session_id,
            user_id,
            &payload,
        )?;

        sqlx::query(
            r#"UPDATE signature_bridge_sessions
               SET payload = $2, status = 'COMPLETED', submitted_at = NOW()
               WHERE id = $1"#,
        )
        .bind(session_id)
        .bind(&envelope)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 桌機輪詢 status — 不取 payload，只回 status 給 dialog 決定是否該 consume。
    /// owner-only：caller 已驗 user_id 對得上 session.user_id。
    pub async fn get_status(pool: &PgPool, session_id: Uuid, user_id: Uuid) -> Result<String> {
        let row: Option<(String, DateTime<Utc>)> = sqlx::query_as(
            r#"SELECT status, expires_at FROM signature_bridge_sessions
               WHERE id = $1 AND user_id = $2"#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let (status, expires_at) =
            row.ok_or_else(|| AppError::NotFound("signature bridge session not found".into()))?;

        // PENDING 但已過期 → EXPIRED；不 update DB 此處（讓 cron 統一清）
        if status == "PENDING" && Utc::now() > expires_at {
            return Ok("EXPIRED".into());
        }
        Ok(status)
    }

    /// 桌機 dialog 取走 payload（status 從 COMPLETED → CONSUMED，單次使用）。
    /// owner-only。
    pub async fn consume(
        pool: &PgPool,
        config: &Config,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<BridgePayload> {
        let mut tx = pool.begin().await?;

        // R66-C6：payload 欄現為 TEXT（AEAD 信封字串）。
        let row: Option<(String, Option<String>, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"SELECT status, payload, submitted_at
               FROM signature_bridge_sessions
               WHERE id = $1 AND user_id = $2 FOR UPDATE"#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (status, stored_payload, submitted_at) =
            row.ok_or_else(|| AppError::NotFound("signature bridge session not found".into()))?;

        if status != "COMPLETED" {
            return Err(AppError::Conflict(format!(
                "session not ready to consume (status={})",
                status
            )));
        }

        let stored_payload = stored_payload
            .ok_or_else(|| AppError::Internal("session COMPLETED but payload null".into()))?;
        // R66-C6：解密信封（過渡期相容 legacy 明文 JSON）。
        let payload = decrypt_payload(
            config.encryption_key.as_ref(),
            session_id,
            user_id,
            &stored_payload,
        )?;
        let submitted_at = submitted_at
            .ok_or_else(|| AppError::Internal("session COMPLETED but submitted_at null".into()))?;

        // payload 已 AEAD 加密（R66-C6）；consume 取走後仍立即把欄位清為 NULL，縮短
        // 信封 at-rest 殘留並讓 GC 更快回收（CONSUMED row 不留任何 payload）。
        sqlx::query(
            r#"UPDATE signature_bridge_sessions
               SET status = 'CONSUMED', consumed_at = NOW(), payload = NULL
               WHERE id = $1"#,
        )
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(BridgePayload {
            payload,
            submitted_at,
        })
    }

    /// GC：清掉已無用 / 棄置的 bridge session，縮短明文 payload 的 at-rest 殘留。
    ///
    /// payload 含明文密碼 + 手寫筆跡（migration 049 註記待加 column 級加密）。
    /// `consume()` 取走後即 `payload=NULL`，但下列三類 row 會殘留：
    /// - **CONSUMED / EXPIRED**：已無用 → 直接刪（payload 早已 NULL 或從未寫入）。
    /// - **COMPLETED 但未 consume**（手機簽完、桌機沒取走）：payload 明文無限期殘留 →
    ///   `submitted_at` 超過 grace（預設 1 小時，遠大於桌機正常輪詢秒級窗口）即視為棄單，
    ///   連 payload 一起刪。
    /// - **PENDING 但已過 TTL**（沒人 submit）：無 payload，刪掉避免表膨脹。
    ///
    /// 安全性：TTL 僅 5 分鐘，任何 PENDING 超過 `expires_at` 必然已失效；COMPLETED 的
    /// 1 小時 grace 遠超正常取用窗口，不會誤刪使用中 session。並行 consume 有 `FOR UPDATE`
    /// 保護，不受本 GC 的無鎖 DELETE 影響（刪除條件互斥於 consume 的時間範圍）。
    ///
    /// 規模：本表僅 admin 改權限時產生 session（TTL 5min + 每日 GC），穩態每次刪除為
    /// 個位數～數十列，故單條 DELETE 即可，與專案其他 GC（messaging/vet_patrol/session）
    /// 一致。若未來量級/並發大幅上升（如多 instance、上 AWS），再改 `ORDER BY created_at
    /// LIMIT N FOR UPDATE SKIP LOCKED` 分批處理避免長鎖（Gemini PR #518 建議）。
    ///
    /// 回傳刪除的 row 數。
    pub async fn cleanup_stale_sessions(pool: &PgPool) -> Result<u64> {
        let now = Utc::now();
        let completed_cutoff = now - Duration::minutes(COMPLETED_GRACE_MINUTES);

        let result = sqlx::query(
            r#"DELETE FROM signature_bridge_sessions
               WHERE status IN ('CONSUMED', 'EXPIRED')
                  OR (status = 'PENDING' AND expires_at < $1)
                  OR (status = 'COMPLETED' AND submitted_at IS NOT NULL AND submitted_at < $2)"#,
        )
        .bind(now)
        .bind(completed_cutoff)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// R66-C6：加密 AAD = session_id ‖ user_id（32 bytes），綁定 payload 到該 session/owner，
/// 防 ciphertext 跨列移植；submit 與 consume 須用相同值。
fn payload_aad(session_id: Uuid, user_id: Uuid) -> [u8; 32] {
    let mut aad = [0u8; 32];
    aad[..16].copy_from_slice(session_id.as_bytes());
    aad[16..].copy_from_slice(user_id.as_bytes());
    aad
}

/// R66-C6：序列化 payload → AEAD 加密成信封字串。無金鑰 → fail-closed 拒絕（不存明文）。
fn encrypt_payload(
    key: Option<&EncryptionKey>,
    session_id: Uuid,
    user_id: Uuid,
    payload: &serde_json::Value,
) -> Result<String> {
    let key = key
        .ok_or_else(|| AppError::Internal("ENCRYPTION_KEY 未設定，無法加密簽章 payload".into()))?;
    // 序列化後的明文（含密碼 + 手寫筆跡）以 Zeroizing 持有 → drop 時清零，不殘留 heap。
    let plaintext = Zeroizing::new(
        serde_json::to_vec(payload)
            .map_err(|e| AppError::Internal(format!("payload 序列化失敗：{e}")))?,
    );
    key.encrypt(&plaintext, &payload_aad(session_id, user_id))
}

/// R66-C6：解出 payload。過渡期相容 migration 104 前 / in-flight 的 legacy 明文 JSON
/// （無信封前綴 → 直接解析）。
fn decrypt_payload(
    key: Option<&EncryptionKey>,
    session_id: Uuid,
    user_id: Uuid,
    stored: &str,
) -> Result<serde_json::Value> {
    if !is_encrypted_envelope(stored) {
        return serde_json::from_str(stored)
            .map_err(|e| AppError::Internal(format!("legacy payload 解析失敗：{e}")));
    }
    let key = key
        .ok_or_else(|| AppError::Internal("ENCRYPTION_KEY 未設定，無法解密簽章 payload".into()))?;
    let plaintext = key.decrypt(stored, &payload_aad(session_id, user_id))?;
    serde_json::from_slice(&plaintext)
        .map_err(|e| AppError::Internal(format!("payload 解密後解析失敗：{e}")))
}

/// 64 字元 crypto-random base64url token（與 invitation token 同套路）
fn generate_mobile_token() -> String {
    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mobile_token_unique_and_long() {
        let a = generate_mobile_token();
        let b = generate_mobile_token();
        assert_ne!(a, b);
        // 48 bytes → 64 chars base64url
        assert_eq!(a.len(), 64);
        // base64url 字符集
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    // ── R66-C6: payload AEAD 加解密（離線確定性）─────────────────────────────
    fn test_key() -> EncryptionKey {
        use base64::{engine::general_purpose::STANDARD, Engine};
        EncryptionKey::from_base64(&STANDARD.encode([5u8; 32])).expect("test key")
    }

    #[test]
    fn payload_encrypt_decrypt_roundtrip() {
        let k = test_key();
        let (sid, uid) = (Uuid::new_v4(), Uuid::new_v4());
        let v = serde_json::json!({"password": "p@ss", "handwriting_svg": "<svg/>"});
        let env = encrypt_payload(Some(&k), sid, uid, &v).expect("encrypt");
        assert!(is_encrypted_envelope(&env), "輸出應為信封");
        let back = decrypt_payload(Some(&k), sid, uid, &env).expect("decrypt");
        assert_eq!(back, v);
    }

    #[test]
    fn payload_decrypt_rejects_wrong_session_or_owner_aad() {
        let k = test_key();
        let uid = Uuid::new_v4();
        let v = serde_json::json!({"x": 1});
        let env = encrypt_payload(Some(&k), Uuid::new_v4(), uid, &v).expect("encrypt");
        // 換 session_id → AAD 不符 → 解密失敗（防跨列移植）
        decrypt_payload(Some(&k), Uuid::new_v4(), uid, &env).expect_err("session AAD 不符應失敗");
        // 換 user_id → 同理
        let env2 = encrypt_payload(Some(&k), uid, uid, &v).expect("encrypt2");
        decrypt_payload(Some(&k), uid, Uuid::new_v4(), &env2).expect_err("owner AAD 不符應失敗");
    }

    #[test]
    fn payload_decrypt_passes_through_legacy_plaintext_json() {
        let k = test_key();
        let legacy = r#"{"password":"p","handwriting_svg":"<svg/>"}"#;
        let v = decrypt_payload(Some(&k), Uuid::new_v4(), Uuid::new_v4(), legacy)
            .expect("legacy plaintext 應可解析");
        assert_eq!(v["password"], "p");
    }

    #[test]
    fn payload_encrypt_fail_closed_without_key() {
        encrypt_payload(None, Uuid::new_v4(), Uuid::new_v4(), &serde_json::json!({}))
            .expect_err("無金鑰應 fail-closed");
    }
}
