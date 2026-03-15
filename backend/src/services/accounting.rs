//! 會計服務
//!
//! 單據核准時產生傳票分錄，串接進銷存與財務
//! GRN → 借：存貨(1300) 貸：應付帳款(2100)
//! DO  → 借：應收帳款(1200) 貸：銷貨收入(4100)；借：銷貨成本(5200) 貸：存貨(1300)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::{DocType, Document, DocumentLine},
    AppError, Result,
};

/// 會計科目
#[derive(Debug, FromRow, Serialize)]
pub struct ChartOfAccount {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
}

/// 試算表列
#[derive(Debug, FromRow, Serialize)]
pub struct TrialBalanceRow {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_balance: Decimal,
    pub credit_balance: Decimal,
}

/// 傳票分錄列（含明細）
#[derive(Debug, FromRow, Serialize)]
pub struct JournalEntryLineRow {
    pub line_id: Uuid,
    pub line_no: i32,
    pub account_code: String,
    pub account_name: String,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub description: Option<String>,
}

/// 傳票摘要
#[derive(Debug, FromRow, Serialize)]
pub struct JournalEntryRow {
    pub id: Uuid,
    pub entry_no: String,
    pub entry_date: NaiveDate,
    pub description: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

/// 應付帳款帳齡列
#[derive(Debug, FromRow, Serialize)]
pub struct ApAgingRow {
    pub partner_id: Uuid,
    pub partner_code: String,
    pub partner_name: String,
    pub total_payable: Decimal,
    pub total_paid: Decimal,
    pub balance: Decimal,
}

/// 應收帳款帳齡列
#[derive(Debug, FromRow, Serialize)]
pub struct ArAgingRow {
    pub partner_id: Uuid,
    pub partner_code: String,
    pub partner_name: String,
    pub total_receivable: Decimal,
    pub total_received: Decimal,
    pub balance: Decimal,
}

/// 會計科目代碼
const ACCT_INVENTORY: &str = "1300";
const ACCT_AR: &str = "1200";
const ACCT_AP: &str = "2100";
const ACCT_REVENUE: &str = "4100";
const ACCT_COGS: &str = "5200";
const ACCT_CASH: &str = "1100";

struct CashJournalParams<'a> {
    entry_date: NaiveDate,
    description: String,
    source_type: &'a str,
    source_id: Uuid,
    debit_account_id: Uuid,
    credit_account_id: Uuid,
    amount: Decimal,
    debit_desc: &'a str,
    credit_desc: &'a str,
    created_by: Uuid,
}

pub struct AccountingService;

impl AccountingService {
    /// 單據核准時過帳（產生傳票分錄）
    /// 僅處理 GRN、DO；其他單據類型暫不產生分錄
    pub async fn post_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        match document.doc_type {
            DocType::GRN => Self::post_grn(tx, document, lines, approved_by).await,
            DocType::DO => Self::post_do(tx, document, lines, approved_by).await,
            DocType::PR => Self::post_pr(tx, document, lines, approved_by).await,
            DocType::SR | DocType::RTN => Self::post_sr(tx, document, lines, approved_by).await,
            _ => Ok(()),
        }
    }

    // ─── 共用子函式 ───────────────────────────────────────

    async fn get_account_id(tx: &mut Transaction<'_, Postgres>, code: &str) -> Result<Option<Uuid>> {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM chart_of_accounts WHERE code = $1 AND is_active = true")
                .bind(code)
                .fetch_optional(&mut **tx)
                .await?;
        Ok(row.map(|r| r.0))
    }

    /// 取得科目 ID，不存在時回傳 BusinessRule 錯誤
    async fn require_account_id(
        tx: &mut Transaction<'_, Postgres>,
        code: &str,
        label: &str,
    ) -> Result<Uuid> {
        Self::get_account_id(tx, code).await?.ok_or_else(|| {
            AppError::BusinessRule(format!("會計科目 {} {} 不存在", code, label))
        })
    }

    async fn next_entry_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
        let n: (i64,) = sqlx::query_as(
            "SELECT nextval('journal_entry_no_seq')",
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(format!("JE{:08}", n.0))
    }

    /// 新增傳票表頭
    async fn insert_journal_entry(
        tx: &mut Transaction<'_, Postgres>,
        entry_id: Uuid,
        entry_no: &str,
        doc_date: NaiveDate,
        description: String,
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
    async fn insert_entry_line(
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

    /// 計算明細行總金額（qty * unit_price）
    fn calc_lines_total(lines: &[DocumentLine]) -> Decimal {
        lines
            .iter()
            .map(|l| l.qty * l.unit_price.unwrap_or(Decimal::ZERO))
            .sum()
    }

    /// 計算銷貨收入與銷貨成本（加權平均）
    async fn calc_revenue_and_cogs(
        tx: &mut Transaction<'_, Postgres>,
        lines: &[DocumentLine],
    ) -> Result<(Decimal, Decimal)> {
        let mut revenue_total = Decimal::ZERO;
        let mut cogs_total = Decimal::ZERO;

        for line in lines {
            let price = line.unit_price.unwrap_or(Decimal::ZERO);
            revenue_total += line.qty * price;

            let avg_cost: Option<Decimal> = sqlx::query_scalar(
                r#"
                SELECT AVG(unit_cost) FILTER (WHERE unit_cost IS NOT NULL)
                FROM stock_ledger
                WHERE product_id = $1 AND direction IN ('in', 'transfer_in', 'adjust_in')
                "#,
            )
            .bind(line.product_id)
            .fetch_one(&mut **tx)
            .await?;

            let cost = avg_cost.unwrap_or(price);
            cogs_total += line.qty * cost;
        }

        Ok((revenue_total, cogs_total))
    }

    // ─── 過帳函式 ─────────────────────────────────────────

    async fn post_grn(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;
        let ap_id = Self::require_account_id(tx, ACCT_AP, "應付帳款").await?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();
        let desc = format!("GRN 採購入庫 {}", document.doc_no);
        Self::insert_journal_entry(tx, entry_id, &entry_no, document.doc_date, desc, "document", document.id, approved_by).await?;

        let mut total = Decimal::ZERO;
        for (i, line) in lines.iter().enumerate() {
            let amount = line.qty * line.unit_price.unwrap_or(Decimal::ZERO);
            total += amount;
            let line_desc = format!("品項 {} 入庫", line.product_id);
            Self::insert_entry_line(tx, entry_id, (i + 1) as i32, inv_id, amount, Decimal::ZERO, &line_desc).await?;
        }

        Self::insert_entry_line(tx, entry_id, (lines.len() + 1) as i32, ap_id, Decimal::ZERO, total, "應付供應商").await?;
        Ok(())
    }

    async fn post_do(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let ar_id = Self::require_account_id(tx, ACCT_AR, "應收帳款").await?;
        let rev_id = Self::require_account_id(tx, ACCT_REVENUE, "銷貨收入").await?;
        let cogs_id = Self::require_account_id(tx, ACCT_COGS, "銷貨成本").await?;
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;

        let (revenue_total, cogs_total) = Self::calc_revenue_and_cogs(tx, lines).await?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();
        let desc = format!("DO 銷貨出庫 {}", document.doc_no);
        Self::insert_journal_entry(tx, entry_id, &entry_no, document.doc_date, desc, "document", document.id, approved_by).await?;

        Self::insert_entry_line(tx, entry_id, 1, ar_id, revenue_total, Decimal::ZERO, "應收帳款").await?;
        Self::insert_entry_line(tx, entry_id, 2, rev_id, Decimal::ZERO, revenue_total, "銷貨收入").await?;
        Self::insert_entry_line(tx, entry_id, 3, cogs_id, cogs_total, Decimal::ZERO, "銷貨成本").await?;
        Self::insert_entry_line(tx, entry_id, 4, inv_id, Decimal::ZERO, cogs_total, "存貨減項").await?;
        Ok(())
    }

    /// 採購退貨過帳（GRN 的反向）
    /// 借：應付帳款 2100，貸：存貨 1300
    async fn post_pr(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let ap_id = Self::require_account_id(tx, ACCT_AP, "應付帳款").await?;
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();
        let total = Self::calc_lines_total(lines);

        let desc = format!("PR 採購退貨 {}", document.doc_no);
        Self::insert_journal_entry(tx, entry_id, &entry_no, document.doc_date, desc, "document", document.id, approved_by).await?;

        // 借：應付帳款（減少負債）
        Self::insert_entry_line(tx, entry_id, 1, ap_id, total, Decimal::ZERO, "沖銷應付帳款").await?;
        // 貸：存貨（退回品項）
        Self::insert_entry_line(tx, entry_id, 2, inv_id, Decimal::ZERO, total, "退回存貨").await?;
        Ok(())
    }

    /// 銷貨退貨過帳（DO 的反向）
    /// 借：銷貨收入 4100 / 存貨 1300，貸：應收帳款 1200 / 銷貨成本 5200
    async fn post_sr(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let ar_id = Self::require_account_id(tx, ACCT_AR, "應收帳款").await?;
        let rev_id = Self::require_account_id(tx, ACCT_REVENUE, "銷貨收入").await?;
        let cogs_id = Self::require_account_id(tx, ACCT_COGS, "銷貨成本").await?;
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;

        let (revenue_total, cogs_total) = Self::calc_revenue_and_cogs(tx, lines).await?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();
        let desc = format!("SR 銷貨退貨 {}", document.doc_no);
        Self::insert_journal_entry(tx, entry_id, &entry_no, document.doc_date, desc, "document", document.id, approved_by).await?;

        // 借：銷貨收入（沖銷收入）
        Self::insert_entry_line(tx, entry_id, 1, rev_id, revenue_total, Decimal::ZERO, "沖銷銷貨收入").await?;
        // 貸：應收帳款（減少應收）
        Self::insert_entry_line(tx, entry_id, 2, ar_id, Decimal::ZERO, revenue_total, "沖銷應收帳款").await?;
        // 借：存貨（退回品項）
        Self::insert_entry_line(tx, entry_id, 3, inv_id, cogs_total, Decimal::ZERO, "退回存貨").await?;
        // 貸：銷貨成本（沖銷成本）
        Self::insert_entry_line(tx, entry_id, 4, cogs_id, Decimal::ZERO, cogs_total, "沖銷銷貨成本").await?;
        Ok(())
    }

    // ─── 查詢函式 ─────────────────────────────────────────

    /// 列出會計科目
    pub async fn list_chart_of_accounts(pool: &PgPool) -> Result<Vec<ChartOfAccount>> {
        let rows = sqlx::query_as::<_, ChartOfAccount>(
            "SELECT id, code, name, account_type::text as account_type FROM chart_of_accounts WHERE is_active = true ORDER BY code",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 試算表（截至指定日期之各科目餘額）
    pub async fn get_trial_balance(
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

    /// 傳票清單（含分錄行）
    pub async fn list_journal_entries(
        pool: &PgPool,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        limit: i64,
    ) -> Result<Vec<(JournalEntryRow, Vec<JournalEntryLineRow>)>> {
        let mut qb = sqlx::QueryBuilder::new(
            "SELECT id, entry_no, entry_date, description, source_entity_type, source_entity_id FROM journal_entries WHERE 1=1",
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
        let mut result = Vec::new();
        for e in entries {
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
            .bind(e.id)
            .fetch_all(pool)
            .await?;
            result.push((e, lines));
        }
        Ok(result)
    }

    /// 應付帳款帳齡（供應商餘額）
    pub async fn get_ap_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ApAgingRow>> {
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
    pub async fn get_ar_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ArAgingRow>> {
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

    // ─── AP/AR 付款收款 ───────────────────────────────────

    /// 建立 AP/AR 的傳票分錄（借方與貸方），回傳 entry_id
    async fn insert_cash_journal(
        tx: &mut Transaction<'_, Postgres>,
        p: CashJournalParams<'_>,
    ) -> Result<Uuid> {
        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();

        Self::insert_journal_entry(tx, entry_id, &entry_no, p.entry_date, p.description, p.source_type, p.source_id, p.created_by).await?;
        Self::insert_entry_line(tx, entry_id, 1, p.debit_account_id, p.amount, Decimal::ZERO, p.debit_desc).await?;
        Self::insert_entry_line(tx, entry_id, 2, p.credit_account_id, Decimal::ZERO, p.amount, p.credit_desc).await?;

        Ok(entry_id)
    }

    /// 建立 AP 付款並過帳（借：應付 2100，貸：現金 1100）
    pub async fn create_ap_payment(
        pool: &PgPool,
        partner_id: Uuid,
        payment_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
        created_by: Uuid,
    ) -> Result<Uuid> {
        let mut tx = pool.begin().await?;
        let payment_no = format!(
            "AP{:08}",
            sqlx::query_scalar::<_, i64>("SELECT nextval('ap_payment_no_seq')")
                .fetch_one(&mut *tx)
                .await?
        );
        let ap_id = Self::require_account_id(&mut tx, ACCT_AP, "應付帳款").await?;
        let cash_id = Self::require_account_id(&mut tx, ACCT_CASH, "現金").await?;

        let payment_id = Uuid::new_v4();
        let desc = format!("AP 應付帳款付款 {}", payment_no);
        let entry_id = Self::insert_cash_journal(&mut tx, CashJournalParams {
            entry_date: payment_date,
            description: desc,
            source_type: "ap_payment",
            source_id: payment_id,
            debit_account_id: ap_id,
            credit_account_id: cash_id,
            amount,
            debit_desc: "應付帳款",
            credit_desc: "現金",
            created_by,
        }).await?;

        Self::insert_ap_payment_record(&mut tx, payment_id, &payment_no, partner_id, payment_date, amount, reference, entry_id, created_by).await?;
        tx.commit().await?;
        Ok(payment_id)
    }

    /// 建立 AR 收款並過帳（借：現金 1100，貸：應收 1200）
    pub async fn create_ar_receipt(
        pool: &PgPool,
        partner_id: Uuid,
        receipt_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
        created_by: Uuid,
    ) -> Result<Uuid> {
        let mut tx = pool.begin().await?;
        let receipt_no = format!(
            "AR{:08}",
            sqlx::query_scalar::<_, i64>("SELECT nextval('ar_receipt_no_seq')")
                .fetch_one(&mut *tx)
                .await?
        );
        let ar_id = Self::require_account_id(&mut tx, ACCT_AR, "應收帳款").await?;
        let cash_id = Self::require_account_id(&mut tx, ACCT_CASH, "現金").await?;

        let receipt_id = Uuid::new_v4();
        let desc = format!("AR 應收帳款收款 {}", receipt_no);
        let entry_id = Self::insert_cash_journal(&mut tx, CashJournalParams {
            entry_date: receipt_date,
            description: desc,
            source_type: "ar_receipt",
            source_id: receipt_id,
            debit_account_id: cash_id,
            credit_account_id: ar_id,
            amount,
            debit_desc: "現金",
            credit_desc: "應收帳款",
            created_by,
        }).await?;

        Self::insert_ar_receipt_record(&mut tx, receipt_id, &receipt_no, partner_id, receipt_date, amount, reference, entry_id, created_by).await?;
        tx.commit().await?;
        Ok(receipt_id)
    }

    async fn insert_ap_payment_record(
        tx: &mut Transaction<'_, Postgres>,
        payment_id: Uuid,
        payment_no: &str,
        partner_id: Uuid,
        payment_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
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

    async fn insert_ar_receipt_record(
        tx: &mut Transaction<'_, Postgres>,
        receipt_id: Uuid,
        receipt_no: &str,
        partner_id: Uuid,
        receipt_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
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

    // ─── 損益表 ───────────────────────────────────────────

    /// 損益表（指定日期範圍內的收入與費用摘要）
    pub async fn get_profit_loss(
        pool: &PgPool,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<ProfitLossSummary> {
        let rows = Self::query_profit_loss_rows(pool, date_from, date_to).await?;
        Ok(Self::summarize_profit_loss(rows))
    }

    async fn query_profit_loss_rows(
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

    fn summarize_profit_loss(rows: Vec<ProfitLossRow>) -> ProfitLossSummary {
        let total_revenue: Decimal = rows
            .iter()
            .filter(|r| r.account_type == "revenue")
            .map(|r| r.amount)
            .sum();
        let total_expense: Decimal = rows
            .iter()
            .filter(|r| r.account_type == "expense")
            .map(|r| r.amount)
            .sum();

        ProfitLossSummary {
            rows,
            total_revenue,
            total_expense,
            net_income: total_revenue - total_expense,
        }
    }
}

/// 損益表科目列
#[derive(Debug, FromRow, Serialize)]
pub struct ProfitLossRow {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub amount: Decimal,
}

/// 損益表摘要
#[derive(Debug, Serialize)]
pub struct ProfitLossSummary {
    pub rows: Vec<ProfitLossRow>,
    pub total_revenue: Decimal,
    pub total_expense: Decimal,
    pub net_income: Decimal,
}
