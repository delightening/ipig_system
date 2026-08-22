//! 一次性：批次匯入 legacy 已核准計劃。讀 payloads JSON（由整理工具產出）→ 逐筆呼叫
//! `ProtocolService::import_approved`。每筆建 APPROVED + import_pending 計劃（imported_at 標記）。
//!
//! 外部 PI（payload `pi_user_id=null`）→ DB `pi_user_id` 記為匯入者本人（`--created-by`），
//! 真名存於 `working_content.basic.pi.name`；之後由 `provision_legacy_pi_accounts` relink。
//!
//! ## Usage
//! ```bash
//! # 預覽（不寫入；驗 iacuc 撞號、SD 是否合法、印計畫）：
//! IMPORT_ACTOR_USER_ID=<importer-uuid> cargo run --bin import_legacy_protocols -- \
//!     --file "<workspace-root>/_import-artifacts/protocol-import-payloads-<批次>.json" --dry-run
//! # 執行（寫入 DATABASE_URL 指向之 DB；test DB 先驗證，再換 prod）：
//! IMPORT_ACTOR_USER_ID=<uuid> cargo run --bin import_legacy_protocols -- --file <path>
//! ```
//! ⚠️ import_approved 內部自管 transaction（無 dry-run rollback）；故先對 test DB 實跑驗證。

use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use erp_backend::config::{read_audit_hmac_key_strict, read_secret_strict};
use erp_backend::models::ImportApprovedProtocolRequest;
use erp_backend::services::{AuditService, ProtocolService};
use erp_backend::ActorContext;

#[derive(Deserialize)]
struct PayloadFile {
    payloads: Vec<PayloadEntry>,
}

#[derive(Deserialize)]
struct PayloadEntry {
    no: String,
    payload: ImportApprovedProtocolRequest,
}

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    // R85-7：不給 repo 內的預設路徑。中間 JSON 依裁定不進版控、住在 repo 外，
    // 留著預設值只會指向不存在（或過期）的檔案，錯得比直接報錯還晚。
    let json_path = arg_value("--file")
        .or_else(|| std::env::var("IMPORT_PAYLOADS_FILE").ok())
        .context("必須以 --file <path> 或 IMPORT_PAYLOADS_FILE 指定 payload JSON 路徑")?;
    let created_by: Uuid = std::env::var("IMPORT_ACTOR_USER_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(s.trim()).ok())
        .context("IMPORT_ACTOR_USER_ID（匯入者 uuid，內部員工）must be set")?;

    let raw = std::fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read payloads file {json_path}"))?;
    let file: PayloadFile = serde_json::from_str(&raw).context("payloads JSON 解析失敗")?;
    println!("讀到 {} 筆 payload（{}）", file.payloads.len(), json_path);

    let database_url = read_secret_strict("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // 載入 prod 既有 iacuc_no，用於「只補沒有的」跳過判斷（去重鍵 = IACUC No.，非標題）
    let existing_iacuc: std::collections::HashSet<String> = sqlx::query_scalar(
        "SELECT iacuc_no FROM protocols WHERE iacuc_no IS NOT NULL AND status <> 'DELETED'",
    )
    .fetch_all(&pool)
    .await
    .context("load existing iacuc_no")?
    .into_iter()
    .collect();
    // 回傳 Some(skip 原因) 或 None(可匯入)：僅依 iacuc_no 判重
    let skip_reason = |p: &ImportApprovedProtocolRequest| -> Option<String> {
        let iacuc = p.iacuc_no.trim();
        if existing_iacuc.contains(iacuc) {
            Some(format!("iacuc {iacuc} 已存在"))
        } else {
            None
        }
    };

    if dry_run {
        let (mut new_n, mut skip_n) = (0u32, 0u32);
        for e in &file.payloads {
            let sd_ok: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(SELECT 1 FROM users u JOIN user_roles ur ON ur.user_id=u.id
                   JOIN roles r ON r.id=ur.role_id WHERE u.id=$1 AND u.is_active AND u.is_internal
                   AND u.deleted_at IS NULL AND r.code='EXPERIMENT_STAFF')"#,
            )
            .bind(e.payload.study_director_user_id)
            .fetch_one(&pool)
            .await?;
            match skip_reason(&e.payload) {
                Some(r) => {
                    skip_n += 1;
                    println!("SKIP {} | {}", e.no, r);
                }
                None => {
                    new_n += 1;
                    println!(
                        "NEW  {} | iacuc={} | SD合法={} | {}",
                        e.no,
                        e.payload.iacuc_no,
                        sd_ok,
                        e.payload.title.chars().take(24).collect::<String>()
                    );
                }
            }
        }
        println!(
            "\n[dry-run] 可匯入(NEW) {new_n} 筆 / 跳過(SKIP) {skip_n} 筆 / 共 {}（未寫入）",
            file.payloads.len()
        );
        return Ok(());
    }

    AuditService::init_hmac_key(read_audit_hmac_key_strict()?);
    let actor = ActorContext::System {
        reason: "legacy_protocol_batch_import",
    };
    let (mut ok, mut skip, mut err) = (0u32, 0u32, 0u32);
    for e in &file.payloads {
        if let Some(r) = skip_reason(&e.payload) {
            skip += 1;
            println!("SKIP {} | {}", e.no, r);
            continue;
        }
        match ProtocolService::import_approved(&pool, &actor, &e.payload, created_by).await {
            Ok(p) => {
                ok += 1;
                println!("OK {} → {} ({})", e.no, p.protocol_no, p.id);
            }
            Err(err_msg) => {
                err += 1;
                eprintln!("ERR {}: {}", e.no, err_msg);
            }
        }
    }
    println!(
        "\n完成：匯入 {ok}，跳過 {skip}，失敗 {err}（共 {} 筆）",
        file.payloads.len()
    );
    Ok(())
}
