//! R66-B2: Backfill — 把 `users.totp_secret_encrypted` 內的 legacy 明文 base32
//! 加密為 AEAD 信封（XChaCha20-Poly1305，AAD = user_id）。
//!
//! Idempotent：已是信封（`is_encrypted_envelope`）的 row 直接略過，可重複執行。
//! `users` 表無 immutability trigger，UPDATE 安全。
//!
//! ## Usage
//! ```bash
//! cargo run --bin backfill_totp_encryption
//! cargo run --bin backfill_totp_encryption -- --dry-run   # 僅預覽、不寫入
//! ```

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use erp_backend::config::read_secret;
use erp_backend::utils::crypto::{is_encrypted_envelope, EncryptionKey};

#[derive(Debug, sqlx::FromRow)]
struct UserSecret {
    id: Uuid,
    totp_secret_encrypted: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let dry_run = std::env::args().any(|a| a == "--dry-run");

    // 用 config 同款 read_secret → 支援 {KEY}_FILE（Docker Secrets）+ {KEY} env fallback。
    let database_url =
        read_secret("DATABASE_URL").context("DATABASE_URL (or _FILE) must be set")?;
    let key_b64 = read_secret("ENCRYPTION_KEY").context("ENCRYPTION_KEY (or _FILE) must be set")?;
    let key = EncryptionKey::from_base64(&key_b64)
        .map_err(|e| anyhow::anyhow!("invalid ENCRYPTION_KEY: {e:?}"))?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let rows: Vec<UserSecret> = sqlx::query_as(
        "SELECT id, totp_secret_encrypted FROM users WHERE totp_secret_encrypted IS NOT NULL",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to query users with totp_secret_encrypted")?;

    println!("Found {} users with a TOTP secret", rows.len());

    let mut encrypted = 0u32;
    let mut already = 0u32;

    for row in &rows {
        if is_encrypted_envelope(&row.totp_secret_encrypted) {
            already += 1;
            continue;
        }
        // legacy 明文 base32 → AEAD 信封（AAD = user_id，與讀路徑一致）。
        let envelope = key
            .encrypt(row.totp_secret_encrypted.as_bytes(), row.id.as_bytes())
            .map_err(|e| anyhow::anyhow!("encrypt user {}: {e:?}", row.id))?;

        if !dry_run {
            sqlx::query(
                "UPDATE users SET totp_secret_encrypted = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&envelope)
            .bind(row.id)
            .execute(&pool)
            .await
            .with_context(|| format!("Failed to update user {}", row.id))?;
        }
        encrypted += 1;
    }

    println!("\n=== TOTP secret 加密 Backfill Summary ===");
    println!("already encrypted (skipped): {already}");
    println!("legacy plaintext encrypted:  {encrypted}");
    if dry_run {
        println!("(dry-run mode — no rows updated)");
    } else {
        println!("Updated {encrypted} rows total.");
    }

    Ok(())
}
