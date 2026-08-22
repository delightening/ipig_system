//! R97-2：庫存與帳務三本帳的 DB 層 immutability triggers（migration 151）。
//!
//! 背景：migration 041 已為 audit log 與電子簽章補上 DB 層防線，但
//! `stock_ledger` / `journal_entries` / `journal_entry_lines` 這三張
//! append-only 真相源至今只有 index、沒有 trigger，僅靠應用層自律
//! （盤點確認 backend/src 對三表零 UPDATE、零 DELETE）。
//!
//! 本檔鎖住的是 **trigger 存在且真的會擋**。少了它，將來有人 down 掉 151、
//! 或改 schema 時漏建 trigger，不會有任何東西發現——應用層本來就不做這些操作，
//! 既有測試自然全綠。
//!
//! 每個 case 都同時驗「被擋」與「沒有副作用」：擋下的操作不得改動任何列。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    let pool = test_db::connect_disposable(5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 造一筆最小可用的 stock_ledger 列（連同其依賴的倉庫／產品／單據），回傳 ledger id。
async fn seed_ledger_row(pool: &PgPool) -> Uuid {
    let s = Uuid::new_v4().simple().to_string();
    let wh = Uuid::new_v4();
    let product = Uuid::new_v4();
    let doc = Uuid::new_v4();
    let ledger = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, '不可竄改測試倉')")
        .bind(wh)
        .bind(format!("WH-IMM-{}", &s[..8]))
        .execute(pool)
        .await
        .expect("seed warehouse");
    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, '不可竄改測試品')")
        .bind(product)
        .bind(format!("SKU-IMM-{}", &s[..8]))
        .execute(pool)
        .await
        .expect("seed product");
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, \
                 (SELECT id FROM users ORDER BY created_at LIMIT 1))",
    )
    .bind(doc)
    .bind(format!("GRN-IMM-{}", &s[..8]))
    .bind(wh)
    .execute(pool)
    .await
    .expect("seed document");
    sqlx::query(
        "INSERT INTO stock_ledger (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', 10)",
    )
    .bind(ledger)
    .bind(wh)
    .bind(product)
    .bind(doc)
    .bind(format!("GRN-IMM-{}", &s[..8]))
    .execute(pool)
    .await
    .expect("seed stock_ledger row（INSERT 必須不受 trigger 影響）");

    ledger
}

/// 造一張傳票 + 一行分錄，回傳 (entry_id, line_id)。
async fn seed_journal(pool: &PgPool) -> (Uuid, Uuid) {
    let s = Uuid::new_v4().simple().to_string();
    let entry: Uuid = sqlx::query(
        "INSERT INTO journal_entries (entry_no, entry_date, description, created_by) \
         VALUES ($1, CURRENT_DATE, '不可竄改測試傳票', \
                 (SELECT id FROM users ORDER BY created_at LIMIT 1)) RETURNING id",
    )
    .bind(format!("JE-IMM-{}", &s[..8]))
    .fetch_one(pool)
    .await
    .expect("seed journal_entry")
    .get("id");

    let line: Uuid = sqlx::query(
        "INSERT INTO journal_entry_lines (journal_entry_id, line_no, account_id, debit_amount, credit_amount) \
         VALUES ($1, 1, (SELECT id FROM chart_of_accounts ORDER BY code LIMIT 1), 100, 0) RETURNING id",
    )
    .bind(entry)
    .fetch_one(pool)
    .await
    .expect("seed journal_entry_line")
    .get("id");

    (entry, line)
}

/// 斷言錯誤確實來自本 migration 的擋板，而不是「碰巧也失敗了」。
///
/// 比對 SQLSTATE `P0001`（trigger 內 `RAISE EXCEPTION ... USING ERRCODE`）而非訊息字串：
/// 字串會因文案調整或 client 語系而變，SQLSTATE 不會。訊息只當輔助檢查
/// （CodeRabbit 於 PR #140 指出原本只比對字串的脆弱性）。
fn assert_append_only_violation(err: &sqlx::Error, ctx: &str) {
    let db_err = err
        .as_database_error()
        .unwrap_or_else(|| panic!("{ctx}：應為資料庫錯誤，實際：{err}"));
    assert_eq!(
        db_err.code().as_deref(),
        Some("P0001"),
        "{ctx}：應為 trigger 擋下的 P0001，實際 code={:?} msg={}",
        db_err.code(),
        db_err.message()
    );
    assert!(
        db_err.message().contains("append-only"),
        "{ctx}：訊息應點明 append-only，實際：{}",
        db_err.message()
    );
}

// ── stock_ledger ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn stock_ledger_rejects_update_and_keeps_row_intact() {
    let pool = setup_pool().await;
    let id = seed_ledger_row(&pool).await;

    let err = sqlx::query("UPDATE stock_ledger SET qty_base = qty_base + 1 WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect_err("stock_ledger 應拒絕 UPDATE");
    assert_append_only_violation(&err, "stock_ledger UPDATE");

    let qty: Decimal = sqlx::query("SELECT qty_base FROM stock_ledger WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("re-read ledger row")
        .get("qty_base");
    assert_eq!(qty, Decimal::from(10), "被擋下的 UPDATE 不得留下任何副作用");
}

#[tokio::test]
#[serial]
async fn stock_ledger_rejects_delete_and_keeps_row_intact() {
    let pool = setup_pool().await;
    let id = seed_ledger_row(&pool).await;

    let err = sqlx::query("DELETE FROM stock_ledger WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect_err("stock_ledger 應拒絕 DELETE");
    assert_append_only_violation(&err, "stock_ledger DELETE");

    let still_there: i64 = sqlx::query("SELECT count(*) AS n FROM stock_ledger WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("count ledger row")
        .get("n");
    assert_eq!(still_there, 1, "被擋下的 DELETE 不得刪掉任何列");
}

// ── journal_entries / journal_entry_lines ─────────────────────────

#[tokio::test]
#[serial]
async fn journal_entries_reject_update_and_delete() {
    let pool = setup_pool().await;
    let (entry, _line) = seed_journal(&pool).await;

    let upd = sqlx::query("UPDATE journal_entries SET description = 'tampered' WHERE id = $1")
        .bind(entry)
        .execute(&pool)
        .await
        .expect_err("journal_entries 應拒絕 UPDATE");
    assert_append_only_violation(&upd, "journal_entries UPDATE");

    let del = sqlx::query("DELETE FROM journal_entries WHERE id = $1")
        .bind(entry)
        .execute(&pool)
        .await
        .expect_err("journal_entries 應拒絕 DELETE");
    assert_append_only_violation(&del, "journal_entries DELETE");

    let desc: String = sqlx::query("SELECT description FROM journal_entries WHERE id = $1")
        .bind(entry)
        .fetch_one(&pool)
        .await
        .expect("re-read entry")
        .get("description");
    assert_eq!(desc, "不可竄改測試傳票", "被擋下的操作不得改動 description");
}

#[tokio::test]
#[serial]
async fn journal_entry_lines_reject_update_and_delete() {
    let pool = setup_pool().await;
    let (_entry, line) = seed_journal(&pool).await;

    let upd = sqlx::query("UPDATE journal_entry_lines SET debit_amount = 999 WHERE id = $1")
        .bind(line)
        .execute(&pool)
        .await
        .expect_err("journal_entry_lines 應拒絕 UPDATE");
    assert_append_only_violation(&upd, "journal_entry_lines UPDATE");

    let del = sqlx::query("DELETE FROM journal_entry_lines WHERE id = $1")
        .bind(line)
        .execute(&pool)
        .await
        .expect_err("journal_entry_lines 應拒絕 DELETE");
    assert_append_only_violation(&del, "journal_entry_lines DELETE");

    let debit: Decimal = sqlx::query("SELECT debit_amount FROM journal_entry_lines WHERE id = $1")
        .bind(line)
        .fetch_one(&pool)
        .await
        .expect("re-read line")
        .get("debit_amount");
    assert_eq!(debit, Decimal::from(100), "被擋下的操作不得改動金額");
}

/// 父表被擋 → `ON DELETE CASCADE` 不會發生，子表分錄必須原封不動。
///
/// 這條單獨存在的理由：`journal_entry_lines.journal_entry_id` 帶
/// `ON DELETE CASCADE`，若只在子表加 trigger 而漏了父表，刪父表就會連帶
/// 清掉分錄；反之若只擋父表，直接刪子表仍會漏。兩邊都要，本測試鎖住這件事。
#[tokio::test]
#[serial]
async fn deleting_journal_entry_does_not_cascade_away_its_lines() {
    let pool = setup_pool().await;
    let (entry, _line) = seed_journal(&pool).await;

    let _ = sqlx::query("DELETE FROM journal_entries WHERE id = $1")
        .bind(entry)
        .execute(&pool)
        .await
        .expect_err("父表 DELETE 應被擋");

    let lines: i64 =
        sqlx::query("SELECT count(*) AS n FROM journal_entry_lines WHERE journal_entry_id = $1")
            .bind(entry)
            .fetch_one(&pool)
            .await
            .expect("count lines")
            .get("n");
    assert_eq!(lines, 1, "父表 DELETE 被擋下，cascade 不得清掉子表分錄");
}

// ── TRUNCATE：row-level trigger 擋不到的那條路 ─────────────────────
//
// PostgreSQL 的 TRUNCATE **不觸發** BEFORE UPDATE / BEFORE DELETE 的 row-level
// trigger。R97-2 初版只加了 UPDATE/DELETE 擋板，實測 `TRUNCATE stock_ledger CASCADE`
// 照樣把整張表清空且不報錯——等於「不可竄改」有一個單字就能繞過的後門
// （CodeRabbit 於 PR #140 指出）。以下三項鎖住 STATEMENT 層的 TRUNCATE 擋板。

#[tokio::test]
#[serial]
async fn stock_ledger_rejects_truncate() {
    let pool = setup_pool().await;
    let id = seed_ledger_row(&pool).await;

    let err = sqlx::query("TRUNCATE stock_ledger")
        .execute(&pool)
        .await
        .expect_err("stock_ledger 應拒絕 TRUNCATE");
    assert_append_only_violation(&err, "stock_ledger TRUNCATE");

    let still_there: i64 = sqlx::query("SELECT count(*) AS n FROM stock_ledger WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("count ledger row")
        .get("n");
    assert_eq!(still_there, 1, "被擋下的 TRUNCATE 不得清空任何列");
}

#[tokio::test]
#[serial]
async fn journal_tables_reject_truncate() {
    let pool = setup_pool().await;
    let (entry, line) = seed_journal(&pool).await;

    let e1 = sqlx::query("TRUNCATE journal_entry_lines")
        .execute(&pool)
        .await
        .expect_err("journal_entry_lines 應拒絕 TRUNCATE");
    assert_append_only_violation(&e1, "journal_entry_lines TRUNCATE");

    // 父表 TRUNCATE 必須指定 CASCADE（子表有 FK），兩張表的擋板都要生效。
    let e2 = sqlx::query("TRUNCATE journal_entries CASCADE")
        .execute(&pool)
        .await
        .expect_err("journal_entries 應拒絕 TRUNCATE CASCADE");
    assert_append_only_violation(&e2, "journal_entries TRUNCATE CASCADE");

    let remaining: i64 = sqlx::query(
        "SELECT (SELECT count(*) FROM journal_entries WHERE id = $1) \
              + (SELECT count(*) FROM journal_entry_lines WHERE id = $2) AS n",
    )
    .bind(entry)
    .bind(line)
    .fetch_one(&pool)
    .await
    .expect("count survivors")
    .get("n");
    assert_eq!(remaining, 2, "被擋下的 TRUNCATE 不得清空傳票或分錄");
}
