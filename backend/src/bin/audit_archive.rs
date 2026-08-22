//! R41-3 Audit Archive CLI — 將 `user_activity_logs` 超過保留期的舊資料匯出至加密 tar.gz
//! 並從運作中 DB 刪除，以符合 `docs/security/DATA_RETENTION_POLICY.md` §6 容量上限政策。
//!
//! ## 觸發條件
//! 觸發點由 Prometheus alert `AuditLogTableSizeWarning` 通知 ops 後手動執行：
//!   - 表 row count > 5,000,000，或
//!   - 表 size > 5 GB，或
//!   - 最舊一筆 created_at > 2 年
//!
//! ## 使用方式
//! ```bash
//! # dry-run（不實際刪除，只顯示會匯出/刪除多少筆）
//! cargo run --bin audit_archive -- --before "2024-05-11" --dry-run
//!
//! # 實際執行（會寫加密 tar.gz 到 $BACKUP_DIR，然後 DELETE）
//! cargo run --bin audit_archive -- --before "2024-05-11" --output /backups/audit_archive
//! ```
//!
//! ## 設計重點
//! 1. **歸檔事件本身寫入 audit log**（type: `AUDIT_ARCHIVE_EXECUTED`，含 actor / row 範圍）
//! 2. **加密**：呼叫 gpg 或 age 對 tar.gz 加密（鍵由 ops 持有）
//! 3. **HMAC chain 保留**：歸檔的是 `created_at < cutoff` 的舊紀錄；新紀錄的 chain
//!    head 不變（DELETE 不會影響 chain 鏈接，因為新紀錄的 prev_hash 鎖定到當時的前一筆）
//! 4. **冪等**：以 created_at 為界，重複執行不會重複歸檔（已刪除的就不在了）
//!
//! ## 狀態：skeleton（R41-3 Phase A）
//! 本檔目前為**設計骨架**，僅包含 CLI 介面與設計意圖。實際 export / encrypt / delete
//! 邏輯待首次觸發 Prometheus alert 時再實作（預期 1–2 年內不會達到容量上限）。

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug)]
struct Args {
    before: DateTime<Utc>,
    output: Option<String>,
    dry_run: bool,
}

fn parse_args() -> Result<Args> {
    let mut before: Option<DateTime<Utc>> = None;
    let mut output: Option<String> = None;
    let mut dry_run = false;

    // Rule rationale: std::env::args() can be tampered by parent process before exec.
    // Exception here: this is a CLI tool intended for operator-invoked one-shot use
    // (cron / manual `cargo run --bin audit_archive`); args() is the standard idiom
    // and not used for security-sensitive trust decisions.
    let mut iter = std::env::args().skip(1); // nosemgrep: rust.lang.security.args.args
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--before" => {
                let v = iter.next().context("--before requires YYYY-MM-DD value")?;
                let date = NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                    .context("--before: invalid date format, expected YYYY-MM-DD")?;
                before = Some(date.and_hms_opt(0, 0, 0).context("invalid date")?.and_utc());
            }
            "--output" => {
                output = Some(iter.next().context("--output requires path")?);
            }
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            other => anyhow::bail!("Unknown argument: {other}\n\n{}", usage()),
        }
    }

    Ok(Args {
        before: before.context("--before is required")?,
        output,
        dry_run,
    })
}

fn usage() -> &'static str {
    "Usage: audit_archive --before YYYY-MM-DD [--output PATH] [--dry-run]

Archives user_activity_logs rows with created_at < --before to an encrypted
tarball, then DELETEs them from the operational DB.

Required:
  --before DATE   cutoff date (UTC); rows older than this will be archived

Optional:
  --output PATH   output directory (default: $BACKUP_DIR or ./audit_archive)
  --dry-run       compute counts only, no export or delete
  -h, --help      show this message"
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;

    println!("audit_archive: cutoff = {}", args.before);
    println!("audit_archive: dry_run = {}", args.dry_run);
    println!("audit_archive: output = {:?}", args.output);
    println!();
    println!("⚠️  R41-3 SKELETON — export/encrypt/delete logic not yet implemented.");
    println!();
    println!("This binary's purpose is documented in:");
    println!("  - docs/security/DATA_RETENTION_POLICY.md §6");
    println!("  - docs/plans/r41_nics_compliance.md (R41-3)");
    println!();
    println!("Implementation will be triggered when Prometheus alert");
    println!("`AuditLogTableSizeWarning` fires (expected 1–2 years from now).");

    // Non-dry-run invocations must NOT exit 0 — cron / ops would mistake skeleton
    // for a successful archive. Only --dry-run currently succeeds (no destructive op).
    if !args.dry_run {
        anyhow::bail!(
            "audit_archive 尚未實作 export/encrypt/delete；目前僅支援 --dry-run。\n\
             實作觸發點：Prometheus alert AuditLogTableSizeWarning 或 AuditLogTableRowsWarning"
        );
    }

    Ok(())
}
