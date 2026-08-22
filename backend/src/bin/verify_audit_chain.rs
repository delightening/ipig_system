//! 一次性稽核 HMAC 鏈完整性驗證工具（read-only）。
//!
//! 用途：在 `AUDIT_CHAIN_VERIFY_ACTIVE=false`（每日 cron 未啟用）的環境下，
//! 手動對「全部歷史」跑一次 `AuditService::verify_chain_range`，取得權威的
//! intact / broken 報告。只做 SELECT + HMAC 重算，不寫入任何資料。
//!
//! 執行（本機連 prod DB）：
//! ```bash
//! DATABASE_URL=postgres://postgres:<pw>@127.0.0.1:5433/ipig_db \
//! AUDIT_HMAC_KEY_FILE=./secrets/audit_hmac_key.txt \
//! cargo run --release --bin verify_audit_chain
//! ```
//! 也可改用 `AUDIT_HMAC_KEY=<key>` 直接提供金鑰（需與寫入端相同）。

use std::env;

use chrono::{TimeZone, Utc};
use sqlx::postgres::PgPoolOptions;

use erp_backend::services::AuditService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db_url = env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL 必須設定"))?;

    // HMAC 金鑰：優先 env，其次讀檔（與 prod secret 檔一致）
    let key = match env::var("AUDIT_HMAC_KEY") {
        Ok(k) => k,
        Err(_) => {
            let path = env::var("AUDIT_HMAC_KEY_FILE")
                .map_err(|_| anyhow::anyhow!("需設定 AUDIT_HMAC_KEY 或 AUDIT_HMAC_KEY_FILE"))?;
            std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("讀取 AUDIT_HMAC_KEY_FILE={path} 失敗: {e}"))?
                .trim()
                .to_string()
        }
    };
    if key.len() < 44 {
        return Err(anyhow::anyhow!(
            "HMAC 金鑰長度 {} < 44，與寫入端不符，驗證會全面 false positive；中止",
            key.len()
        ));
    }
    AuditService::init_hmac_key(Some(key));

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await?;

    // 全史範圍：2000-01-01 ~ 明天（涵蓋所有既有 row）
    let from = Utc
        .with_ymd_and_hms(2000, 1, 1, 0, 0, 0)
        .single()
        .expect("valid genesis ts");
    let to = Utc::now() + chrono::Duration::days(1);

    let report = AuditService::verify_chain_range(&pool, from, to).await?;

    println!("=== Audit HMAC Chain Verification (full history) ===");
    println!(
        "range            : {} ~ {}",
        report.range_from, report.range_to
    );
    println!("total_rows       : {}", report.total_rows);
    println!("verified_rows    : {}", report.verified_rows);
    println!(
        "skipped_no_hash  : {} (SECURITY 類不入鏈，合法)",
        report.skipped_no_hash
    );
    println!(
        "acknowledged     : {} (audit_chain_known_breaks 白名單，歷史非竄改)",
        report.acknowledged_breaks
    );
    println!(
        "broken_links     : {} (未確認，觸發告警)",
        report.broken_links.len()
    );
    println!(
        "RESULT           : {}",
        if report.is_intact() {
            "✅ CHAIN INTACT"
        } else {
            "❌ BROKEN LINKS DETECTED"
        }
    );

    if !report.is_intact() {
        println!("--- broken rows (up to 50) ---");
        for b in report.broken_links.iter().take(50) {
            println!("  id={} created_at={}", b.id, b.created_at);
        }
        std::process::exit(1);
    }

    Ok(())
}
