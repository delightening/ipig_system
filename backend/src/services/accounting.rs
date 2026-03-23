//! 會計服務
//!
//! 單據核准時產生傳票分錄，串接進銷存與財務
//! GRN -> 借：存貨(1300) 貸：應付帳款(2100)
//! DO  -> 借：應收帳款(1200) 貸：銷貨收入(4100)；借：銷貨成本(5200) 貸：存貨(1300)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::accounting::{
    ApAgingRow, ArAgingRow, ChartOfAccount, JournalEntryLineRow, JournalEntryRow, ProfitLossRow,
    ProfitLossSummary, TrialBalanceRow,
};
use crate::models::{DocType, Document, DocumentLine};
use crate::repositories::accounting as repo;
use crate::{AppError, Result};

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
    /// 僅處理 GRN、DO、PR、SR/RTN；其他單據類型暫不產生分錄
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

    // --- 共用子函式 ---

    /// 取得科目 ID，不存在時回傳 BusinessRule 錯誤
    async fn require_account_id(
        tx: &mut Transaction<'_, Postgres>,
        code: &str,
        label: &str,
    ) -> Result<Uuid> {
        repo::find_account_id_by_code(tx, code)
            .await?
            .ok_or_else(|| AppError::BusinessRule(format!("會計科目 {} {} 不存在", code, label)))
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

            let avg_cost = repo::find_avg_cost_by_product(tx, line.product_id).await?;
            let cost = avg_cost.unwrap_or(price);
            cogs_total += line.qty * cost;
        }

        Ok((revenue_total, cogs_total))
    }

    /// 建立傳票表頭 + 回傳 entry_id
    async fn create_journal_entry(
        tx: &mut Transaction<'_, Postgres>,
        doc_date: NaiveDate,
        description: String,
        source_type: &str,
        source_id: Uuid,
        created_by: Uuid,
    ) -> Result<(Uuid, String)> {
        let entry_no = repo::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();
        repo::insert_journal_entry(
            tx,
            entry_id,
            &entry_no,
            doc_date,
            &description,
            source_type,
            source_id,
            created_by,
        )
        .await?;
        Ok((entry_id, entry_no))
    }

    // --- 過帳函式 ---

    async fn post_grn(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;
        let ap_id = Self::require_account_id(tx, ACCT_AP, "應付帳款").await?;

        let desc = format!("GRN 採購入庫 {}", document.doc_no);
        let (entry_id, _) =
            Self::create_journal_entry(tx, document.doc_date, desc, "document", document.id, approved_by).await?;

        let mut total = Decimal::ZERO;
        for (i, line) in lines.iter().enumerate() {
            let amount = line.qty * line.unit_price.unwrap_or(Decimal::ZERO);
            total += amount;
            let line_desc = format!("品項 {} 入庫", line.product_id);
            repo::insert_entry_line(tx, entry_id, (i + 1) as i32, inv_id, amount, Decimal::ZERO, &line_desc).await?;
        }

        repo::insert_entry_line(tx, entry_id, (lines.len() + 1) as i32, ap_id, Decimal::ZERO, total, "應付供應商").await?;
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

        let desc = format!("DO 銷貨出庫 {}", document.doc_no);
        let (entry_id, _) =
            Self::create_journal_entry(tx, document.doc_date, desc, "document", document.id, approved_by).await?;

        repo::insert_entry_line(tx, entry_id, 1, ar_id, revenue_total, Decimal::ZERO, "應收帳款").await?;
        repo::insert_entry_line(tx, entry_id, 2, rev_id, Decimal::ZERO, revenue_total, "銷貨收入").await?;
        repo::insert_entry_line(tx, entry_id, 3, cogs_id, cogs_total, Decimal::ZERO, "銷貨成本").await?;
        repo::insert_entry_line(tx, entry_id, 4, inv_id, Decimal::ZERO, cogs_total, "存貨減項").await?;
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
        let total = Self::calc_lines_total(lines);

        let desc = format!("PR 採購退貨 {}", document.doc_no);
        let (entry_id, _) =
            Self::create_journal_entry(tx, document.doc_date, desc, "document", document.id, approved_by).await?;

        repo::insert_entry_line(tx, entry_id, 1, ap_id, total, Decimal::ZERO, "沖銷應付帳款").await?;
        repo::insert_entry_line(tx, entry_id, 2, inv_id, Decimal::ZERO, total, "退回存貨").await?;
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

        let desc = format!("SR 銷貨退貨 {}", document.doc_no);
        let (entry_id, _) =
            Self::create_journal_entry(tx, document.doc_date, desc, "document", document.id, approved_by).await?;

        repo::insert_entry_line(tx, entry_id, 1, rev_id, revenue_total, Decimal::ZERO, "沖銷銷貨收入").await?;
        repo::insert_entry_line(tx, entry_id, 2, ar_id, Decimal::ZERO, revenue_total, "沖銷應收帳款").await?;
        repo::insert_entry_line(tx, entry_id, 3, inv_id, cogs_total, Decimal::ZERO, "退回存貨").await?;
        repo::insert_entry_line(tx, entry_id, 4, cogs_id, Decimal::ZERO, cogs_total, "沖銷銷貨成本").await?;
        Ok(())
    }

    // --- 查詢函式 ---

    /// 列出會計科目
    pub async fn list_chart_of_accounts(pool: &PgPool) -> Result<Vec<ChartOfAccount>> {
        repo::list_active_accounts(pool).await
    }

    /// 試算表（截至指定日期之各科目餘額）
    pub async fn get_trial_balance(
        pool: &PgPool,
        as_of_date: NaiveDate,
    ) -> Result<Vec<TrialBalanceRow>> {
        repo::list_trial_balance(pool, as_of_date).await
    }

    /// 傳票清單（含分錄行）
    pub async fn list_journal_entries(
        pool: &PgPool,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        limit: i64,
    ) -> Result<Vec<(JournalEntryRow, Vec<JournalEntryLineRow>)>> {
        let entries = repo::list_journal_entries(pool, date_from, date_to, limit).await?;
        let mut result = Vec::with_capacity(entries.len());
        for e in entries {
            let lines = repo::find_journal_entry_lines(pool, e.id).await?;
            result.push((e, lines));
        }
        Ok(result)
    }

    /// 應付帳款帳齡
    pub async fn get_ap_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ApAgingRow>> {
        repo::list_ap_aging(pool, as_of_date).await
    }

    /// 應收帳款帳齡
    pub async fn get_ar_aging(pool: &PgPool, as_of_date: NaiveDate) -> Result<Vec<ArAgingRow>> {
        repo::list_ar_aging(pool, as_of_date).await
    }

    // --- AP/AR 付款收款 ---

    /// 建立 AP/AR 的傳票分錄（借方與貸方），回傳 entry_id
    async fn insert_cash_journal(
        tx: &mut Transaction<'_, Postgres>,
        p: CashJournalParams<'_>,
    ) -> Result<Uuid> {
        let (entry_id, _) = Self::create_journal_entry(
            tx,
            p.entry_date,
            p.description,
            p.source_type,
            p.source_id,
            p.created_by,
        )
        .await?;
        repo::insert_entry_line(tx, entry_id, 1, p.debit_account_id, p.amount, Decimal::ZERO, p.debit_desc).await?;
        repo::insert_entry_line(tx, entry_id, 2, p.credit_account_id, Decimal::ZERO, p.amount, p.credit_desc).await?;
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
        let payment_no = repo::next_ap_payment_no(&mut tx).await?;
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

        repo::insert_ap_payment(&mut tx, payment_id, &payment_no, partner_id, payment_date, amount, reference.as_deref(), entry_id, created_by).await?;
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
        let receipt_no = repo::next_ar_receipt_no(&mut tx).await?;
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

        repo::insert_ar_receipt(&mut tx, receipt_id, &receipt_no, partner_id, receipt_date, amount, reference.as_deref(), entry_id, created_by).await?;
        tx.commit().await?;
        Ok(receipt_id)
    }

    // --- 損益表 ---

    /// 損益表（指定日期範圍內的收入與費用摘要）
    pub async fn get_profit_loss(
        pool: &PgPool,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<ProfitLossSummary> {
        let rows = repo::list_profit_loss_rows(pool, date_from, date_to).await?;
        Ok(Self::summarize_profit_loss(rows))
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
