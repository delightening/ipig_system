//! R62-1 Storage Inventory Drift Reconciliation Tool
//!
//! 對比 `stock_ledger` 累計運算結果 vs `storage_location_inventory.on_hand_qty`，
//! 找出 drift（不一致）並輸出 CSV 報告。**read-only，完全不寫 DB**。
//!
//! ## 觸發條件
//! - 庫存對不上 / 倉管回報差異
//! - GLP 稽核前的資料健康檢查
//! - Migration / 大型 refactor 後驗證
//!
//! ## 使用方式
//! ```bash
//! # 預設輸出到 stdout（含 summary）
//! cargo run --bin reconcile_storage_inventory
//!
//! # 寫到 CSV
//! cargo run --bin reconcile_storage_inventory -- --output drift_report.csv
//!
//! # 只顯示有 drift 的 row（隱藏 drift=0 的乾淨資料）
//! cargo run --bin reconcile_storage_inventory -- --output drift.csv --only-drift
//! ```
//!
//! ## 演算
//! 1. 對每筆 stock_ledger row 依 direction 換算 signed_qty（in/transfer_in/adjust_in 為正，其餘負）
//! 2. 按 (storage_location_id, product_id, batch_no, expiry_date) group + sum
//! 3. LEFT JOIN storage_location_inventory，計算 drift = current - recalculated
//! 4. 同時抓「ledger 有但 SLI 沒有」的 orphan 累計（疑似漏建 SLI 列）
//!
//! ## 注意
//! - 不影響 prod 可用性：只跑 SELECT
//! - Ledger 中 `storage_location_id IS NULL` 的列代表 migration 069 前的 warehouse-level
//!   只記錄；本工具忽略，因為無法歸屬到具體 location
//! - 若有 drift，請走 R62-2 流程：人工 review CSV → 寫 ops migration 070 修正

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};

const SQL_RECONCILIATION: &str = r#"
WITH ledger_agg AS (
    SELECT
        storage_location_id,
        product_id,
        COALESCE(batch_no, '') AS batch_key,
        COALESCE(expiry_date, '1900-01-01'::date) AS expiry_key,
        SUM(
            CASE direction
                WHEN 'in'           THEN qty_base
                WHEN 'transfer_in'  THEN qty_base
                WHEN 'adjust_in'    THEN qty_base
                WHEN 'out'          THEN -qty_base
                WHEN 'transfer_out' THEN -qty_base
                WHEN 'adjust_out'   THEN -qty_base
                ELSE 0
            END
        ) AS recalc_qty
    FROM stock_ledger
    WHERE storage_location_id IS NOT NULL
    GROUP BY storage_location_id, product_id, batch_key, expiry_key
),
unknown_dirs AS (
    SELECT DISTINCT direction
    FROM stock_ledger
    WHERE storage_location_id IS NOT NULL
      AND direction NOT IN ('in', 'transfer_in', 'adjust_in', 'out', 'transfer_out', 'adjust_out')
),
sli_side AS (
    SELECT
        sli.storage_location_id,
        sl.code  AS storage_location_code,
        sl.name  AS storage_location_name,
        sli.product_id,
        p.sku    AS product_sku,
        p.name   AS product_name,
        sli.batch_no,
        sli.expiry_date,
        sli.on_hand_qty AS current_qty,
        COALESCE(la.recalc_qty, 0) AS recalc_qty,
        sli.on_hand_qty - COALESCE(la.recalc_qty, 0) AS drift,
        'sli_exists' AS source
    FROM storage_location_inventory sli
    JOIN storage_locations sl ON sl.id = sli.storage_location_id
    JOIN products p ON p.id = sli.product_id
    LEFT JOIN ledger_agg la
        ON la.storage_location_id = sli.storage_location_id
       AND la.product_id          = sli.product_id
       AND la.batch_key           = COALESCE(sli.batch_no, '')
       AND la.expiry_key          = COALESCE(sli.expiry_date, '1900-01-01'::date)
),
orphan_side AS (
    SELECT
        la.storage_location_id,
        sl.code  AS storage_location_code,
        sl.name  AS storage_location_name,
        la.product_id,
        p.sku    AS product_sku,
        p.name   AS product_name,
        NULLIF(la.batch_key, '') AS batch_no,
        NULLIF(la.expiry_key, '1900-01-01'::date) AS expiry_date,
        NULL::numeric AS current_qty,
        la.recalc_qty,
        -la.recalc_qty AS drift,
        'orphan_ledger' AS source
    FROM ledger_agg la
    JOIN storage_locations sl ON sl.id = la.storage_location_id
    JOIN products p ON p.id = la.product_id
    LEFT JOIN storage_location_inventory sli
        ON sli.storage_location_id = la.storage_location_id
       AND sli.product_id          = la.product_id
       AND COALESCE(sli.batch_no, '') = la.batch_key
       AND COALESCE(sli.expiry_date, '1900-01-01'::date) = la.expiry_key
    WHERE sli.id IS NULL
      AND la.recalc_qty != 0
)
SELECT * FROM (
    SELECT * FROM sli_side
    UNION ALL
    SELECT * FROM orphan_side
) combined
ORDER BY ABS(combined.drift) DESC, combined.storage_location_code, combined.product_sku
"#;

const SQL_UNKNOWN_DIRECTIONS: &str = r#"
SELECT DISTINCT direction
FROM stock_ledger
WHERE storage_location_id IS NOT NULL
  AND direction NOT IN ('in', 'transfer_in', 'adjust_in', 'out', 'transfer_out', 'adjust_out')
"#;

#[derive(Debug)]
struct Args {
    output: Option<String>,
    only_drift: bool,
}

fn parse_args() -> Result<Args> {
    let mut output: Option<String> = None;
    let mut only_drift = false;

    // CLI tool 為手動 invoke，args() 是標準慣例 (參考 audit_archive.rs)
    let mut iter = std::env::args().skip(1); // nosemgrep: rust.lang.security.args.args
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--output" | "-o" => {
                output = Some(iter.next().context("--output 需要檔名")?);
            }
            "--only-drift" => only_drift = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => anyhow::bail!("未知參數: {other}（用 --help 查看用法）"),
        }
    }

    Ok(Args { output, only_drift })
}

fn print_help() {
    println!(
        r#"R62-1 Storage Inventory Drift Reconciliation Tool

USAGE:
    cargo run --bin reconcile_storage_inventory [OPTIONS]

OPTIONS:
    -o, --output <PATH>    輸出 CSV 到指定路徑（預設輸出到 stdout）
        --only-drift       只顯示 drift != 0 的列（過濾乾淨資料）
    -h, --help             顯示說明

範例:
    cargo run --bin reconcile_storage_inventory -- -o drift_report.csv --only-drift
"#
    );
}

#[derive(Debug, sqlx::FromRow)]
struct DriftRow {
    storage_location_id: Option<uuid::Uuid>,
    storage_location_code: Option<String>,
    storage_location_name: Option<String>,
    product_id: Option<uuid::Uuid>,
    product_sku: Option<String>,
    product_name: Option<String>,
    batch_no: Option<String>,
    expiry_date: Option<chrono::NaiveDate>,
    current_qty: Option<Decimal>,
    recalc_qty: Decimal,
    drift: Decimal,
    source: String, // "sli_exists" | "orphan_ledger"
}

async fn warn_unknown_directions(pool: &PgPool) -> Result<()> {
    let dirs: Vec<(String,)> = sqlx::query_as(SQL_UNKNOWN_DIRECTIONS)
        .fetch_all(pool)
        .await
        .context("查詢 unknown directions 失敗")?;
    for (d,) in &dirs {
        eprintln!("⚠️  stock_ledger 存在未知 direction: \"{d}\"（已被 ELSE 0 忽略，請檢查）");
    }
    Ok(())
}

async fn run_reconciliation(pool: &PgPool) -> Result<Vec<DriftRow>> {
    let rows: Vec<DriftRow> = sqlx::query_as(SQL_RECONCILIATION)
        .fetch_all(pool)
        .await
        .context("執行 reconciliation query 失敗")?;
    Ok(rows)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            if c == '"' {
                out.push('"');
            }
            out.push(c);
        }
        out.push('"');
        out
    } else {
        s.to_owned()
    }
}

fn csv_row(r: &DriftRow) -> String {
    let mut buf = String::with_capacity(256);
    let _ = write!(
        buf,
        "{},{},{},{},{},{},{},{},{},{},{},{}",
        escape_csv(&r.source),
        r.storage_location_id
            .map(|u| u.to_string())
            .unwrap_or_default(),
        escape_csv(r.storage_location_code.as_deref().unwrap_or("")),
        escape_csv(r.storage_location_name.as_deref().unwrap_or("")),
        r.product_id.map(|u| u.to_string()).unwrap_or_default(),
        escape_csv(r.product_sku.as_deref().unwrap_or("")),
        escape_csv(r.product_name.as_deref().unwrap_or("")),
        escape_csv(r.batch_no.as_deref().unwrap_or("")),
        r.expiry_date.map(|d| d.to_string()).unwrap_or_default(),
        r.current_qty.map(|q| q.to_string()).unwrap_or_default(),
        r.recalc_qty,
        r.drift,
    );
    buf
}

fn write_csv<W: Write>(writer: &mut W, rows: &[DriftRow], only_drift: bool) -> Result<()> {
    writeln!(
        writer,
        "source,location_id,location_code,location_name,product_id,product_sku,product_name,batch_no,expiry_date,current_qty,recalc_qty,drift"
    )?;
    for r in rows {
        if only_drift && r.drift == Decimal::ZERO {
            continue;
        }
        writeln!(writer, "{}", csv_row(r))?;
    }
    Ok(())
}

fn print_summary(rows: &[DriftRow]) {
    let total = rows.len();
    let drift_rows: Vec<&DriftRow> = rows.iter().filter(|r| r.drift != Decimal::ZERO).collect();
    let orphan: Vec<&DriftRow> = rows
        .iter()
        .filter(|r| r.source == "orphan_ledger")
        .collect();
    let pos_drift = drift_rows
        .iter()
        .filter(|r| r.drift > Decimal::ZERO)
        .count();
    let neg_drift = drift_rows
        .iter()
        .filter(|r| r.drift < Decimal::ZERO)
        .count();
    let drift_total: Decimal = drift_rows.iter().map(|r| r.drift.abs()).sum();

    eprintln!("\n=== R62-1 Storage Drift Reconciliation Summary ===");
    eprintln!("Total rows scanned:        {total}");
    eprintln!(
        "Drift rows:                {} ({} clean)",
        drift_rows.len(),
        total - drift_rows.len()
    );
    eprintln!("  └ SLI 多算 (drift > 0):  {pos_drift}");
    eprintln!("  └ SLI 少算 (drift < 0):  {neg_drift}");
    eprintln!(
        "  └ Orphan ledger:         {} (有 ledger 累計但 SLI 漏建)",
        orphan.len()
    );
    eprintln!("Σ |drift|:                 {drift_total}");
    eprintln!("===================================================\n");

    if !drift_rows.is_empty() {
        eprintln!(
            "⚠️  發現 drift — 請 review CSV 並走 R62-2 流程（人工 sign-off + ops migration 070）"
        );
    } else {
        eprintln!("✅ 庫存乾淨：ledger 累計與 SLI 完全一致");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;

    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL 環境變數未設定（範例: postgres://user:pass@localhost/db）")?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("DB 連線失敗")?;

    eprintln!("R62-1 開始 reconciliation（read-only）...");
    warn_unknown_directions(&pool).await?;
    let rows = run_reconciliation(&pool).await?;
    eprintln!("Query 完成，{} 列。", rows.len());

    match args.output.as_deref() {
        Some(path) => {
            let mut f = File::create(path).with_context(|| format!("無法建立 {path}"))?;
            write_csv(&mut f, &rows, args.only_drift)?;
            eprintln!("CSV 寫入: {path}");
        }
        None => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write_csv(&mut handle, &rows, args.only_drift)?;
        }
    }

    print_summary(&rows);
    Ok(())
}
