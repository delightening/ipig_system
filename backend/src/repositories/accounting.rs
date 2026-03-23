//! 會計模組 Repository 層
//!
//! 封裝所有會計相關 SQL 查詢

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::accounting::{
    ApAgingRow, ArAgingRow, ChartOfAccount, JournalEntryLineRow, JournalEntryRow, ProfitLossRow,
    TrialBalanceRow,
};
use crate::Result;

/// 依科目代碼查詢帳戶 ID（僅啟用中科目）
pub async fn find_account_id_by_code(
    tx: &mut Transaction<'_, Postgres>,
    code: &str,
) -> Result<Option<Uuid>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM chart_of_accounts WHERE code = $1 AND is_active = true")
            .bind(code)
            .fetch_optional(&mut **tx)
            .await?;
    Ok(row.map(|r| r.0))
}

/// 列出所有啟用中的會計科目
pub async fn list_active_accounts(pool: &PgPool) -> Result<Vec<ChartOfAccount>> {
    let rows = sqlx::query_as::<_, ChartOfAccount>(
        "SELECT id, code, name, account_type::text as account_type \
         FROM chart_of_accounts WHERE is_active = true ORDER BY code",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 試算表（截至指定日期之各科目餘額）
pub async fn list_trial_balance(
    pool: &PgPool,
    as_of_date: NaiveDate,
) -> Result<Vec<TrialBalanceRow>> {
    let rows = sqlx::query_as::<_, TrialBalanceRow>(
        r#"
        WITH lines_in_range AS (
            SELECT jel.account_id, jel.debit_amount, jel.credit_amount
            FROM journal_entry_lines jel
            INNER JOIN journal_entries je ON je.id = jel.journal_entry_id AND je.entry_date <= $1
        ),
        balances AS (
            SELECT
                coa.id as account_id,
                coa.code as account_code,
                coa.name as account_name,
                coa.account_type::text as account_type,
                COALESCE(SUM(l.debit_amount), 0) - COALESCE(SUM(l.credit_amount), 0) as net
            FROM chart_of_accounts coa
            LEFT JOIN lines_in_range l ON l.account_id = coa.id
            WHERE coa.is_active = true
            GROUP BY coa.id, coa.code, coa.name, coa.account_type
        )
        SELECT
            account_id,
            account_code,
            account_name,
            account_type,
            CASE WHEN net > 0 THEN net ELSE 0 END as debit_balance,
            CASE WHEN net < 0 THEN -net ELSE 0 END as credit_balance
        FROM balances
        WHERE net != 0
        ORDER BY account_code
        "#,
    )
    .bind(as_of_date)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 查詢傳票表頭（支援日期篩選與筆數限制）
pub async fn list_journal_entries(
    pool: &PgPool,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<JournalEntryRow>> {
    let mut qb = sqlx::QueryBuilder::new(
        "SELECT id, entry_no, entry_date, description, source_entity_type, source_entity_id \
         FROM journal_entries WHERE 1=1",
    );
    if let Some(d) = date_from {
        qb.push(" AND entry_date >= ");
        qb.push_bind(d);
    }
    if let Some(d) = date_to {
        qb.push(" AND entry_date <= ");
        qb.push_bind(d);
    }
    qb.push(" ORDER BY entry_date DESC, entry_no DESC LIMIT ");
    qb.push_bind(limit);

    let entries: Vec<JournalEntryRow> = qb.build_query_as().fetch_all(pool).await?;
    Ok(entries)
}

/// 查詢指定傳票的分錄明細行
pub async fn find_journal_entry_lines(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Vec<JournalEntryLineRow>> {
    let lines = sqlx::query_as::<_, JournalEntryLineRow>(
        r#"
        SELECT jel.id as line_id, jel.line_no, coa.code as account_code, coa.name as account_name,
               jel.debit_amount, jel.credit_amount, jel.description
        FROM journal_entry_lines jel
        JOIN chart_of_accounts coa ON coa.id = jel.account_id
        WHERE jel.journal_entry_id = $1
        ORDER BY jel.line_no
        "#,
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await?;
    Ok(lines)
}

/// 應付帳款帳齡（供應商餘額）
pub async fn list_ap_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ApAgingRow>> {
    let rows = sqlx::query_as::<_, ApAgingRow>(
        r#"
        WITH ap_credits AS (
            SELECT doc.partner_id, COALESCE(SUM(jel.credit_amount - jel.debit_amount), 0) as total
            FROM journal_entries je
            JOIN journal_entry_lines jel ON jel.journal_entry_id = je.id
            JOIN chart_of_accounts coa ON coa.id = jel.account_id AND coa.code = '2100'
            JOIN documents doc ON doc.id = je.source_entity_id AND je.source_entity_type = 'document'
            WHERE je.entry_date <= $1 AND doc.doc_type IN ('GRN', 'PR')
            GROUP BY doc.partner_id
        ),
        ap_debits AS (
            SELECT ap.partner_id, COALESCE(SUM(ap.amount), 0) as total
            FROM ap_payments ap
            WHERE ap.payment_date <= $1
            GROUP BY ap.partner_id
        )
        SELECT
            p.id as partner_id,
            p.code as partner_code,
            p.name as partner_name,
            COALESCE(c.total, 0) as total_payable,
            COALESCE(d.total, 0) as total_paid,
            COALESCE(c.total, 0) - COALESCE(d.total, 0) as balance
        FROM partners p
        LEFT JOIN ap_credits c ON c.partner_id = p.id
        LEFT JOIN ap_debits d ON d.partner_id = p.id
        WHERE p.partner_type = 'supplier'
          AND (COALESCE(c.total, 0) - COALESCE(d.total, 0)) != 0
        ORDER BY balance DESC
        "#,
    )
    .bind(as_of_date)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 應收帳款帳齡（客戶餘額）
pub async fn list_ar_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ArAgingRow>> {
    let rows = sqlx::query_as::<_, ArAgingRow>(
        r#"
        WITH ar_debits AS (
            SELECT doc.partner_id, COALESCE(SUM(jel.debit_amount - jel.credit_amount), 0) as total
            FROM journal_entries je
            JOIN journal_entry_lines jel ON jel.journal_entry_id = je.id
            JOIN chart_of_accounts coa ON coa.id = jel.account_id AND coa.code = '1200'
            JOIN documents doc ON doc.id = je.source_entity_id AND je.source_entity_type = 'document'
            WHERE je.entry_date <= $1 AND doc.doc_type IN ('DO', 'SR', 'RTN')
            GROUP BY doc.partner_id
        ),
        ar_credits AS (
            SELECT ar.partner_id, COALESCE(SUM(ar.amount), 0) as total
            FROM ar_receipts ar
            WHERE ar.receipt_date <= $1
            GROUP BY ar.partner_id
        )
        SELECT
            p.id as partner_id,
            p.code as partner_code,
            p.name as partner_name,
            COALESCE(d.total, 0) as total_receivable,
            COALESCE(c.total, 0) as total_received,
            COALESCE(d.total, 0) - COALESCE(c.total, 0) as balance
        FROM partners p
        LEFT JOIN ar_debits d ON d.partner_id = p.id
        LEFT JOIN ar_credits c ON c.partner_id = p.id
        WHERE p.partner_type = 'customer'
          AND (COALESCE(d.total, 0) - COALESCE(c.total, 0)) != 0
        ORDER BY balance DESC
        "#,
    )
    .bind(as_of_date)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 取得下一組傳票編號
pub async fn next_entry_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
    let n: (i64,) = sqlx::query_as("SELECT nextval('journal_entry_no_seq')")
        .fetch_one(&mut **tx)
        .await?;
    Ok(format!("JE{:08}", n.0))
}

/// 新增傳票表頭
pub async fn insert_journal_entry(
    tx: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
    entry_no: &str,
    doc_date: NaiveDate,
    description: &str,
    source_type: &str,
    source_id: Uuid,
    created_by: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO journal_entries (id, entry_no, entry_date, description, source_entity_type, source_entity_id, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        "#,
    )
    .bind(entry_id)
    .bind(entry_no)
    .bind(doc_date)
    .bind(description)
    .bind(source_type)
    .bind(source_id)
    .bind(created_by)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// 新增傳票分錄行
pub async fn insert_entry_line(
    tx: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
    line_no: i32,
    account_id: Uuid,
    debit: Decimal,
    credit: Decimal,
    description: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(entry_id)
    .bind(line_no)
    .bind(account_id)
    .bind(debit)
    .bind(credit)
    .bind(description)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// 取得下一組 AP 付款編號
pub async fn next_ap_payment_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
    let n: (i64,) = sqlx::query_as("SELECT nextval('ap_payment_no_seq')")
        .fetch_one(&mut **tx)
        .await?;
    Ok(format!("AP{:08}", n.0))
}

/// 新增 AP 付款記錄
pub async fn insert_ap_payment(
    tx: &mut Transaction<'_, Postgres>,
    payment_id: Uuid,
    payment_no: &str,
    partner_id: Uuid,
    payment_date: NaiveDate,
    amount: Decimal,
    reference: Option<&str>,
    entry_id: Uuid,
    created_by: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO ap_payments (id, payment_no, partner_id, payment_date, amount, reference, journal_entry_id, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(payment_id)
    .bind(payment_no)
    .bind(partner_id)
    .bind(payment_date)
    .bind(amount)
    .bind(reference)
    .bind(entry_id)
    .bind(created_by)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// 取得下一組 AR 收款編號
pub async fn next_ar_receipt_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
    let n: (i64,) = sqlx::query_as("SELECT nextval('ar_receipt_no_seq')")
        .fetch_one(&mut **tx)
        .await?;
    Ok(format!("AR{:08}", n.0))
}

/// 新增 AR 收款記錄
pub async fn insert_ar_receipt(
    tx: &mut Transaction<'_, Postgres>,
    receipt_id: Uuid,
    receipt_no: &str,
    partner_id: Uuid,
    receipt_date: NaiveDate,
    amount: Decimal,
    reference: Option<&str>,
    entry_id: Uuid,
    created_by: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO ar_receipts (id, receipt_no, partner_id, receipt_date, amount, reference, journal_entry_id, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(receipt_id)
    .bind(receipt_no)
    .bind(partner_id)
    .bind(receipt_date)
    .bind(amount)
    .bind(reference)
    .bind(entry_id)
    .bind(created_by)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// 查詢品項加權平均成本
pub async fn find_avg_cost_by_product(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
) -> Result<Option<Decimal>> {
    let avg_cost: Option<Decimal> = sqlx::query_scalar(
        r#"
        SELECT AVG(unit_cost) FILTER (WHERE unit_cost IS NOT NULL)
        FROM stock_ledger
        WHERE product_id = $1 AND direction IN ('in', 'transfer_in', 'adjust_in')
        "#,
    )
    .bind(product_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(avg_cost)
}

/// 損益表查詢（指定日期範圍內的收入與費用明細）
pub async fn list_profit_loss_rows(
    pool: &PgPool,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
) -> Result<Vec<ProfitLossRow>> {
    let mut qb = sqlx::QueryBuilder::new(
        r#"
        SELECT
            coa.code as account_code,
            coa.name as account_name,
            coa.account_type::text as account_type,
            CASE
                WHEN coa.account_type = 'revenue'
                    THEN COALESCE(SUM(jel.credit_amount), 0) - COALESCE(SUM(jel.debit_amount), 0)
                ELSE
                    COALESCE(SUM(jel.debit_amount), 0) - COALESCE(SUM(jel.credit_amount), 0)
            END as amount
        FROM chart_of_accounts coa
        INNER JOIN journal_entry_lines jel ON jel.account_id = coa.id
        INNER JOIN journal_entries je ON je.id = jel.journal_entry_id
        WHERE coa.is_active = true
          AND coa.account_type IN ('revenue', 'expense')
        "#,
    );

    if let Some(d) = date_from {
        qb.push(" AND je.entry_date >= ");
        qb.push_bind(d);
    }
    if let Some(d) = date_to {
        qb.push(" AND je.entry_date <= ");
        qb.push_bind(d);
    }

    qb.push(" GROUP BY coa.id, coa.code, coa.name, coa.account_type ORDER BY coa.code");

    let rows: Vec<ProfitLossRow> = qb.build_query_as().fetch_all(pool).await?;
    Ok(rows)
}
