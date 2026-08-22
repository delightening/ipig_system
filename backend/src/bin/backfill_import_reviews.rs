//! 一次性：補登已匯入計畫的「執秘意見 + 獸醫師評比」審查文件（委員意見先 park）。
//! 讀 {iacuc_no: ImportReviewsRequest} JSON → 逐筆呼叫 ProtocolService::record_import_reviews。
//! ⚠️ record_import_reviews 為「全量取代」：本次只帶執秘+獸醫、committee 空；日後補委員時
//! 須連執秘+獸醫一起帶（否則會被清掉）。僅作用於 import_pending=true 計畫。
//!
//! ## Usage
//! ```bash
//! DATABASE_URL_FILE=../secrets/db_url_host.txt AUDIT_HMAC_KEY_FILE=../secrets/audit_hmac_key.txt \
//!   cargo run --bin backfill_import_reviews -- \
//!     --file "<workspace-root>/_import-artifacts/protocol-reviews-backfill-<批次>.json" --dry-run
//! ```

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::collections::BTreeMap;
use uuid::Uuid;

use erp_backend::config::read_secret;
use erp_backend::models::ImportReviewsRequest;
use erp_backend::services::{AuditService, ProtocolService};
use erp_backend::ActorContext;

fn arg_value(flag: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == flag)
        .and_then(|i| a.get(i + 1).cloned())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    // R85-7：中間 JSON 不進版控、住在 repo 外，故不給預設路徑（見 docs/design/README.md）。
    let path = arg_value("--file").context("必須以 --file <path> 指定審查補登 JSON 路徑")?;
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {path}"))?;
    let map: BTreeMap<String, ImportReviewsRequest> =
        serde_json::from_str(&raw).context("reviews backfill JSON 解析失敗")?;
    println!("讀到 {} 筆審查補登（{}）", map.len(), path);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(
            &read_secret("DATABASE_URL")
                .context("DATABASE_URL（或 DATABASE_URL_FILE）must be set")?,
        )
        .await
        .context("connect db")?;

    if !dry_run {
        // audit-integrity 硬化（CodeRabbit）：AUDIT_HMAC_KEY 存在但長度不足時，不再靜默
        // 降級成無 HMAC（會讓 backfill 的稽核寫入缺 integrity_hash），改 fail-loud 拒絕執行，
        // 避免誤設短金鑰時無聲弱化稽核完整性。
        let hmac_key = read_secret("AUDIT_HMAC_KEY");
        if let Some(k) = hmac_key.as_deref() {
            anyhow::ensure!(
                k.len() >= 44,
                "AUDIT_HMAC_KEY 已設定但長度不足（{} < 44 字元）；拒絕以無效金鑰執行 backfill（避免靜默弱化稽核完整性）",
                k.len()
            );
        }
        AuditService::init_hmac_key(hmac_key);
    }
    let actor = ActorContext::System {
        reason: "legacy_review_backfill_exec_vet",
    };
    let (mut ok, mut skip, mut err) = (0u32, 0u32, 0u32);
    for (iacuc, req) in &map {
        let row: Option<(Uuid, bool)> = sqlx::query_as(
            "SELECT id, import_pending FROM protocols WHERE iacuc_no = $1 AND status <> 'DELETED'",
        )
        .bind(iacuc)
        .fetch_optional(&pool)
        .await?;
        let Some((id, pending)) = row else {
            skip += 1;
            eprintln!("SKIP {iacuc} | 查無計畫");
            continue;
        };
        if !pending {
            skip += 1;
            eprintln!("SKIP {iacuc} | 非 import_pending");
            continue;
        }
        let sec = req.secretary_comments.len();
        let vet = req.vet_review.as_ref().map(|v| v.items.len()).unwrap_or(0);
        if dry_run {
            ok += 1;
            println!("[dry-run] {iacuc} ({id}) → 執秘 {sec} 筆 / 獸醫 {vet} 項 / 委員 0（park）");
            continue;
        }
        match ProtocolService::record_import_reviews(&pool, &actor, id, req).await {
            Ok(()) => {
                ok += 1;
                println!("OK {iacuc} ({id}) → 執秘 {sec} / 獸醫 {vet}");
            }
            Err(e) => {
                err += 1;
                eprintln!("ERR {iacuc}: {e}");
            }
        }
    }
    println!(
        "\n完成：補登 {ok}，跳過 {skip}，失敗 {err}（共 {} 筆）",
        map.len()
    );
    Ok(())
}
