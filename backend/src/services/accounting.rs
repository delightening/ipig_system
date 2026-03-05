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
            _ => Ok(()),
        }
    }

    async fn get_account_id(tx: &mut Transaction<'_, Postgres>, code: &str) -> Result<Option<Uuid>> {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM chart_of_accounts WHERE code = $1 AND is_active = true")
                .bind(code)
                .fetch_optional(&mut **tx)
                .await?;
        Ok(row.map(|r| r.0))
    }

    async fn next_entry_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
        let n: (i64,) = sqlx::query_as(
            "SELECT nextval('journal_entry_no_seq')",
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(format!("JE{:08}", n.0))
    }

    async fn post_grn(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let inv_id = Self::get_account_id(tx, ACCT_INVENTORY).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1300 存貨 不存在".to_string()))?;
        let ap_id = Self::get_account_id(tx, ACCT_AP).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 2100 應付帳款 不存在".to_string()))?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO journal_entries (id, entry_no, entry_date, description, source_entity_type, source_entity_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, 'document', $5, $6, NOW())
            "#,
        )
        .bind(entry_id)
        .bind(&entry_no)
        .bind(document.doc_date)
        .bind(format!("GRN 採購入庫 {}", document.doc_no))
        .bind(document.id)
        .bind(approved_by)
        .execute(&mut **tx)
        .await?;

        let mut total = Decimal::ZERO;
        for (i, line) in lines.iter().enumerate() {
            let amount = line.qty * line.unit_price.unwrap_or(Decimal::ZERO);
            total += amount;

            sqlx::query(
                r#"
                INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, 0, $5)
                "#,
            )
            .bind(entry_id)
            .bind((i + 1) as i32)
            .bind(inv_id)
            .bind(amount)
            .bind(format!("品項 {} 入庫", line.product_id))
            .execute(&mut **tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
            VALUES (gen_random_uuid(), $1, $2, $3, 0, $4, $5)
            "#,
        )
        .bind(entry_id)
        .bind((lines.len() + 1) as i32)
        .bind(ap_id)
        .bind(total)
        .bind("應付供應商")
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn post_do(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let ar_id = Self::get_account_id(tx, ACCT_AR).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1200 應收帳款 不存在".to_string()))?;
        let rev_id = Self::get_account_id(tx, ACCT_REVENUE).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 4100 銷貨收入 不存在".to_string()))?;
        let cogs_id = Self::get_account_id(tx, ACCT_COGS).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 5200 銷貨成本 不存在".to_string()))?;
        let inv_id = Self::get_account_id(tx, ACCT_INVENTORY).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1300 存貨 不存在".to_string()))?;

        let entry_no = Self::next_entry_no(tx).await?;
        let entry_id = Uuid::new_v4();

        let mut revenue_total = Decimal::ZERO;
        let mut cogs_total = Decimal::ZERO;

        for line in lines {
            let price = line.unit_price.unwrap_or(Decimal::ZERO);
            revenue_total += line.qty * price;
            // 銷貨成本：依移動平均，此處簡化使用相同單價
            cogs_total += line.qty * price;
        }

        sqlx::query(
            r#"
            INSERT INTO journal_entries (id, entry_no, entry_date, description, source_entity_type, source_entity_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, 'document', $5, $6, NOW())
            "#,
        )
        .bind(entry_id)
        .bind(&entry_no)
        .bind(document.doc_date)
        .bind(format!("DO 銷貨出庫 {}", document.doc_no))
        .bind(document.id)
        .bind(approved_by)
        .execute(&mut **tx)
        .await?;

        let mut line_no = 1;
        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, 0, '應收帳款')
            "#,
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(ar_id)
        .bind(revenue_total)
        .execute(&mut **tx)
        .await?;
        line_no += 1;

        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
            VALUES (gen_random_uuid(), $1, $2, $3, 0, $4, '銷貨收入')
            "#,
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(rev_id)
        .bind(revenue_total)
        .execute(&mut **tx)
        .await?;
        line_no += 1;

        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, 0, '銷貨成本')
            "#,
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(cogs_id)
        .bind(cogs_total)
        .execute(&mut **tx)
        .await?;
        line_no += 1;

        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
            VALUES (gen_random_uuid(), $1, $2, $3, 0, $4, '存貨減項')
            "#,
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(inv_id)
        .bind(cogs_total)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

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
                WHERE je.entry_date <= $1 AND doc.doc_type = 'GRN'
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
                WHERE je.entry_date <= $1 AND doc.doc_type = 'DO'
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
        let ap_id = Self::get_account_id(&mut tx, ACCT_AP).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 2100 應付帳款 不存在".to_string()))?;
        let cash_id = Self::get_account_id(&mut tx, "1100").await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1100 現金 不存在".to_string()))?;

        let entry_no = Self::next_entry_no(&mut tx).await?;
        let entry_id = Uuid::new_v4();
        let payment_id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO journal_entries (id, entry_no, entry_date, description, source_entity_type, source_entity_id, created_by, created_at)
               VALUES ($1, $2, $3, $4, 'ap_payment', $5, $6, NOW())"#,
        )
        .bind(entry_id)
        .bind(&entry_no)
        .bind(payment_date)
        .bind(format!("AP 應付帳款付款 {}", payment_no))
        .bind(payment_id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
               VALUES (gen_random_uuid(), $1, 1, $2, $3, 0, '應付帳款')"#,
        )
        .bind(entry_id)
        .bind(ap_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
               VALUES (gen_random_uuid(), $1, 2, $2, 0, $3, '現金')"#,
        )
        .bind(entry_id)
        .bind(cash_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO ap_payments (id, payment_no, partner_id, payment_date, amount, reference, journal_entry_id, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(payment_id)
        .bind(&payment_no)
        .bind(partner_id)
        .bind(payment_date)
        .bind(amount)
        .bind(reference)
        .bind(entry_id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

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
        let ar_id = Self::get_account_id(&mut tx, ACCT_AR).await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1200 應收帳款 不存在".to_string()))?;
        let cash_id = Self::get_account_id(&mut tx, "1100").await?
            .ok_or_else(|| AppError::BusinessRule("會計科目 1100 現金 不存在".to_string()))?;

        let entry_no = Self::next_entry_no(&mut tx).await?;
        let entry_id = Uuid::new_v4();
        let receipt_id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO journal_entries (id, entry_no, entry_date, description, source_entity_type, source_entity_id, created_by, created_at)
               VALUES ($1, $2, $3, $4, 'ar_receipt', $5, $6, NOW())"#,
        )
        .bind(entry_id)
        .bind(&entry_no)
        .bind(receipt_date)
        .bind(format!("AR 應收帳款收款 {}", receipt_no))
        .bind(receipt_id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
               VALUES (gen_random_uuid(), $1, 1, $2, $3, 0, '現金')"#,
        )
        .bind(entry_id)
        .bind(cash_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_entry_lines (id, journal_entry_id, line_no, account_id, debit_amount, credit_amount, description)
               VALUES (gen_random_uuid(), $1, 2, $2, 0, $3, '應收帳款')"#,
        )
        .bind(entry_id)
        .bind(ar_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO ar_receipts (id, receipt_no, partner_id, receipt_date, amount, reference, journal_entry_id, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(receipt_id)
        .bind(&receipt_no)
        .bind(partner_id)
        .bind(receipt_date)
        .bind(amount)
        .bind(reference)
        .bind(entry_id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(receipt_id)
    }
}
