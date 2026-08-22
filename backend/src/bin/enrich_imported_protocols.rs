//! 一次性：把計畫書詳細內容（從申請表 docx 結構化抽取）深度合併進已匯入計畫的 working_content。
//! 僅作用於 import_pending=true 的計畫（補登期可編輯）。保留匯入時的 basic.pi/sponsor，疊加
//! purpose/design/surgery/animals/guidelines 等。
//!
//! ## Usage
//! ```bash
//! DATABASE_URL_FILE=../secrets/db_url_host.txt cargo run --bin enrich_imported_protocols -- \
//!     --file "<workspace-root>/_import-artifacts/protocol-content-enrich-<批次>.json" --dry-run
//! ```

use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::collections::BTreeMap;

use erp_backend::config::read_secret_strict;

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

/// 深度合併：overlay 疊加進 base（物件遞迴、其餘 overlay 覆蓋）。base 既有但 overlay 沒有的鍵保留。
fn deep_merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                deep_merge(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (b, o) => *b = o.clone(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    // R85-7：中間 JSON 不進版控、住在 repo 外，故不給預設路徑（見 docs/design/README.md）。
    let json_path = arg_value("--file").context("必須以 --file <path> 指定 enrich JSON 路徑")?;

    let raw = std::fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read {json_path}"))?;
    let content: BTreeMap<String, Value> =
        serde_json::from_str(&raw).context("enrich JSON 解析失敗")?;
    println!("讀到 {} 筆待補內容（{}）", content.len(), json_path);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&read_secret_strict("DATABASE_URL")?)
        .await
        .context("connect db")?;

    let (mut ok, mut skip, mut err) = (0u32, 0u32, 0u32);
    for (iacuc, extracted) in &content {
        let row: Option<(uuid::Uuid, Option<Value>, bool)> = sqlx::query_as(
            "SELECT id, working_content, import_pending FROM protocols WHERE iacuc_no = $1 AND status <> 'DELETED'",
        )
        .bind(iacuc)
        .fetch_optional(&pool)
        .await?;
        let Some((id, existing, pending)) = row else {
            skip += 1;
            eprintln!("SKIP {iacuc} | 查無此計畫");
            continue;
        };
        if !pending {
            skip += 1;
            eprintln!("SKIP {iacuc} | 非 import_pending（已 finalize），不動");
            continue;
        }
        let mut merged = existing.unwrap_or_else(|| Value::Object(Default::default()));
        deep_merge(&mut merged, extracted);
        let keys = merged.as_object().map(|o| o.len()).unwrap_or(0);
        if dry_run {
            ok += 1;
            println!("[dry-run] {iacuc} ({id}) → 合併後 working_content 頂層 {keys} 鍵");
            continue;
        }
        let affected = sqlx::query(
            "UPDATE protocols SET working_content = $1, updated_at = now() WHERE id = $2 AND import_pending = true",
        )
        .bind(&merged)
        .bind(id)
        .execute(&pool)
        .await;
        match affected {
            Ok(r) if r.rows_affected() == 1 => {
                ok += 1;
                println!("OK {iacuc} ({id}) → working_content 已補（頂層 {keys} 鍵）");
            }
            Ok(_) => {
                skip += 1;
                eprintln!("SKIP {iacuc} | 未更新（import_pending 變動?）");
            }
            Err(e) => {
                err += 1;
                eprintln!("ERR {iacuc}: {e}");
            }
        }
    }
    println!(
        "\n完成：補登 {ok}，跳過 {skip}，失敗 {err}（共 {} 筆）",
        content.len()
    );
    Ok(())
}
