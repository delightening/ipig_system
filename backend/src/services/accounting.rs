//! 會計服務
//!
//! 單據核准時產生傳票分錄，串接進銷存與財務
//! GRN -> 借：存貨(1300) 貸：應付帳款(2100)
//! DO  -> 借：應收帳款(1200) 貸：銷貨收入(4100)；借：銷貨成本(5200) 貸：存貨(1300)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::middleware::ActorContext;
use crate::models::accounting::{
    ApAgingRow, ArAgingRow, ChartOfAccount, JournalEntryLineRow, JournalEntryRow, ProfitLossRow,
    ProfitLossSummary, TrialBalanceRow,
};
use crate::models::audit_diff::{AuditRedact, DataDiff};
use crate::models::{DocType, Document, DocumentLine};
use crate::repositories::accounting as repo;
use crate::services::audit::{ActivityLogEntry, AuditEntity};
use crate::services::AuditService;
use crate::{AppError, Result};

/// AP 付款 audit 快照（用於 create_only DataDiff）
#[derive(Debug, Serialize)]
struct ApPaymentSnapshot<'a> {
    id: Uuid,
    payment_no: &'a str,
    partner_id: Uuid,
    payment_date: NaiveDate,
    amount: Decimal,
    reference: Option<&'a str>,
    journal_entry_id: Uuid,
}

impl AuditRedact for ApPaymentSnapshot<'_> {}

/// AR 收款 audit 快照
#[derive(Debug, Serialize)]
struct ArReceiptSnapshot<'a> {
    id: Uuid,
    receipt_no: &'a str,
    partner_id: Uuid,
    receipt_date: NaiveDate,
    amount: Decimal,
    reference: Option<&'a str>,
    journal_entry_id: Uuid,
}

impl AuditRedact for ArReceiptSnapshot<'_> {}

/// 會計科目代碼
///
/// R84-13：`ACCT_REVENUE`（4100 銷貨收入）原本只被 `post_sr`（銷貨退貨過帳）使用，
/// SR/RTN 已封鎖新建並清死碼後不再有任何呼叫端，故一併移除；4100 科目本身仍在
/// 會計科目主檔存在，只是暫無程式路徑會過帳到它。
const ACCT_INVENTORY: &str = "1300";
const ACCT_AR: &str = "1200";
const ACCT_AP: &str = "2100";
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

/// 傳票表頭欄位（封裝以符合參數數量上限）。
struct JournalEntryHeader<'a> {
    doc_date: NaiveDate,
    description: String,
    source_type: &'a str,
    source_id: Uuid,
    created_by: Uuid,
}

/// 一筆待寫入的傳票分錄（已決定借貸方向）。
struct PostingLine {
    account_id: Uuid,
    debit: Decimal,
    credit: Decimal,
    description: String,
}

impl PostingLine {
    /// 借方分錄（credit = 0）。
    fn debit(account_id: Uuid, amount: Decimal, description: impl Into<String>) -> Self {
        Self {
            account_id,
            debit: amount,
            credit: Decimal::ZERO,
            description: description.into(),
        }
    }

    /// 貸方分錄（debit = 0）。
    fn credit(account_id: Uuid, amount: Decimal, description: impl Into<String>) -> Self {
        Self {
            account_id,
            debit: Decimal::ZERO,
            credit: amount,
            description: description.into(),
        }
    }
}

/// 剔除零金額分錄。
///
/// `journal_entry_lines` 的 `chk_debit_credit` 約束要求每行借貸恰有一方 > 0；
/// 借貸皆為 0 的分錄（例如未填單價的入庫品項，金額算出來是 0）會違反約束，
/// 且零額分錄本身無會計意義 → 一律剔除。回傳保留的分錄（維持原順序），
/// line_no 由呼叫端重新連續編號。
fn retain_postable_lines(lines: Vec<PostingLine>) -> Vec<PostingLine> {
    lines
        .into_iter()
        .filter(|l| l.debit > Decimal::ZERO || l.credit > Decimal::ZERO)
        .collect()
}

/// 驗證一組分錄合法且平衡：每行借貸恰有一方 > 0，且借方總額 = 貸方總額。
///
/// 由 `post_balanced_entry` 在寫表頭前呼叫。零金額過濾後若仍出現方向不合法
/// （例如負金額品項：借方被濾掉但貸方總額仍含該負值 → 不平衡）或借貸不對等，
/// 寧可 fail fast（在 best-effort savepoint 內被攔下、不阻擋核准），也不要靜默
/// 寫出不平衡傳票。
fn validate_balanced(lines: &[PostingLine]) -> Result<()> {
    let mut total_debit = Decimal::ZERO;
    let mut total_credit = Decimal::ZERO;
    for line in lines {
        match (line.debit > Decimal::ZERO, line.credit > Decimal::ZERO) {
            (true, false) => total_debit += line.debit,
            (false, true) => total_credit += line.credit,
            _ => return Err(AppError::BusinessRule("會計分錄借貸方向不合法".into())),
        }
    }
    if total_debit != total_credit {
        return Err(AppError::BusinessRule("會計分錄借貸不平衡".into()));
    }
    Ok(())
}

pub struct AccountingService;

impl AccountingService {
    /// R84-5 沖銷：把原單的傳票借貸相反地鏡射一張新傳票（紅字沖銷）。
    ///
    /// 刻意**不**重跑 `post_document`——沖銷鏡射的是原單當初真的過了什麼帳，
    /// 而非用現在的平均成本重算（後者會因期間進貨而算出不同金額，兩張傳票就沖不平）。
    ///
    /// ⚠️ 與一般核准的 `post_accounting_best_effort` 不同，本函式**沒有** SAVEPOINT 隔離：
    /// 沖銷失敗必須讓整筆 tx rollback。庫存已經退回去、傳票卻沒鏡射成功的話，
    /// 帳會永久歪掉且無從察覺——這條是合規路徑，不接受 best-effort。
    /// （2026-07-23 使用者裁定，刻意偏離既有模式。）
    pub async fn reverse_document_posting(
        tx: &mut Transaction<'_, Postgres>,
        original: &Document,
        reversal: &Document,
        approved_by: Uuid,
    ) -> Result<()> {
        let entries: Vec<(Uuid, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, description FROM journal_entries
            WHERE source_entity_type = 'document' AND source_entity_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(original.id)
        .fetch_all(&mut **tx)
        .await?;

        for (entry_id, description) in entries {
            let lines: Vec<(Uuid, Decimal, Decimal, Option<String>)> = sqlx::query_as(
                r#"
                SELECT account_id, debit_amount, credit_amount, description
                FROM journal_entry_lines
                WHERE journal_entry_id = $1
                ORDER BY line_no
                "#,
            )
            .bind(entry_id)
            .fetch_all(&mut **tx)
            .await?;

            // 借貸互換即為紅字沖銷；金額原樣沿用，保證與原傳票精確互抵。
            let mirrored: Vec<PostingLine> = lines
                .into_iter()
                .map(|(account_id, debit, credit, desc)| PostingLine {
                    account_id,
                    debit: credit,
                    credit: debit,
                    description: desc.unwrap_or_else(|| "沖銷".to_string()),
                })
                .collect();

            Self::post_balanced_entry(
                tx,
                JournalEntryHeader {
                    doc_date: reversal.doc_date,
                    description: format!(
                        "沖銷 {}（原傳票：{}）",
                        original.doc_no,
                        description.unwrap_or_else(|| "-".to_string())
                    ),
                    source_type: "document",
                    source_id: reversal.id,
                    created_by: approved_by,
                },
                mirrored,
            )
            .await?;
        }

        Ok(())
    }

    /// 單據核准時過帳（產生傳票分錄）
    /// 僅處理 GRN、SO、PR；其他單據類型暫不產生分錄
    ///
    /// R84-13（2026-08-13）：原本此處還有 `SR | RTN => post_sr` 分支。使用者裁定業務上
    /// 不存在銷貨退貨（SO 從不認列收入，`post_sr` 沖銷的是一筆從未發生的收入），`DocType`
    /// 已移除 `SR` / `RTN`，反序列化階段就無法產生該值，此分支成為不可能路徑，一併清除。
    pub async fn post_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        match document.doc_type {
            DocType::GRN => Self::post_grn(tx, document, lines, approved_by).await,
            // SO 一段式銷貨（migration 136）：核准即過帳，但**只記成本轉列**
            // （借：銷貨成本／貸：存貨），不記應收與收入——ERP 銷貨全為內部耗材領用，
            // 沒有真實對外收入可認列（見 crud.rs::validate_line_qty_price 與 ERP_SYSTEM.md §5.1）。
            DocType::SO => Self::post_sales(tx, document, lines, approved_by).await,
            DocType::PR => Self::post_pr(tx, document, lines, approved_by).await,
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

    /// 寫入一張傳票：先剔除零金額分錄；若無有效分錄則不建立傳票（避免空傳票），
    /// 否則建立表頭並依序寫入分錄行。各 `post_*` 過帳函式統一走此路徑，
    /// 確保零單價品項不會產生違反 `chk_debit_credit` 的無效分錄。
    async fn post_balanced_entry(
        tx: &mut Transaction<'_, Postgres>,
        header: JournalEntryHeader<'_>,
        candidate_lines: Vec<PostingLine>,
    ) -> Result<()> {
        let lines = retain_postable_lines(candidate_lines);
        if lines.is_empty() {
            return Ok(());
        }
        validate_balanced(&lines)?;

        let (entry_id, _) = Self::create_journal_entry(
            tx,
            header.doc_date,
            header.description,
            header.source_type,
            header.source_id,
            header.created_by,
        )
        .await?;

        for (i, line) in lines.iter().enumerate() {
            repo::insert_entry_line(
                tx,
                entry_id,
                (i + 1) as i32,
                line.account_id,
                line.debit,
                line.credit,
                &line.description,
            )
            .await?;
        }
        Ok(())
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

        let mut candidates = Vec::with_capacity(lines.len() + 1);
        let mut total = Decimal::ZERO;
        for line in lines {
            let amount = line.qty * line.unit_price.unwrap_or(Decimal::ZERO);
            total += amount;
            candidates.push(PostingLine::debit(
                inv_id,
                amount,
                format!("品項 {} 入庫", line.product_id),
            ));
        }
        candidates.push(PostingLine::credit(ap_id, total, "應付供應商"));

        Self::post_balanced_entry(
            tx,
            JournalEntryHeader {
                doc_date: document.doc_date,
                description: format!("GRN 採購入庫 {}", document.doc_no),
                source_type: "document",
                source_id: document.id,
                created_by: approved_by,
            },
            candidates,
        )
        .await
    }

    /// SO 領用過帳：**只記成本轉列**——借銷貨成本 / 貸存貨（進貨平均成本）。
    ///
    /// SO（2026-07-21 裁定）：ERP 出庫全為內部耗材領用，**永不認列收入**。
    /// `validate_line_qty_price` 已在建單階段拒收單價，此處不再讀取 revenue 金額，
    /// 因此也不需要 1200 應收帳款 / 4100 銷貨收入 兩個科目。
    ///
    /// R84-9（2026-07-23）：本函式原為 SO 與 DO 共用、依 doc_type 決定要不要記
    /// 應收+收入。DO 移除後 `post_document` 只會把 SO 派進來，該分支已不可達並一併清除，
    /// doc comment 同步改為 SO-only，避免後續維護者依舊契約把分支加回來。
    async fn post_sales(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let cogs_id = Self::require_account_id(tx, ACCT_COGS, "銷貨成本").await?;
        let inv_id = Self::require_account_id(tx, ACCT_INVENTORY, "存貨").await?;

        let (_revenue_total, cogs_total) = Self::calc_revenue_and_cogs(tx, lines).await?;

        let candidates = vec![
            PostingLine::debit(cogs_id, cogs_total, "銷貨成本"),
            PostingLine::credit(inv_id, cogs_total, "存貨減項"),
        ];

        Self::post_balanced_entry(
            tx,
            JournalEntryHeader {
                doc_date: document.doc_date,
                description: format!("SO 銷貨 {}", document.doc_no),
                source_type: "document",
                source_id: document.id,
                created_by: approved_by,
            },
            candidates,
        )
        .await
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

        let candidates = vec![
            PostingLine::debit(ap_id, total, "沖銷應付帳款"),
            PostingLine::credit(inv_id, total, "退回存貨"),
        ];

        Self::post_balanced_entry(
            tx,
            JournalEntryHeader {
                doc_date: document.doc_date,
                description: format!("PR 採購退貨 {}", document.doc_no),
                source_type: "document",
                source_id: document.id,
                created_by: approved_by,
            },
            candidates,
        )
        .await
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

        // N+1 修復：原本每張傳票各查一次分錄行，最壞 1 + MAX_PAGE_SIZE 次 round-trip。
        let entry_ids: Vec<Uuid> = entries.iter().map(|e| e.id).collect();
        let mut lines_by_entry: std::collections::HashMap<Uuid, Vec<JournalEntryLineRow>> =
            std::collections::HashMap::new();
        for (entry_id, line) in
            repo::find_journal_entry_lines_by_entry_ids(pool, &entry_ids).await?
        {
            lines_by_entry.entry(entry_id).or_default().push(line);
        }

        // 無分錄行的傳票不會出現在 join 結果中，對應原本查回空 Vec 的行為。
        Ok(entries
            .into_iter()
            .map(|e| {
                let lines = lines_by_entry.remove(&e.id).unwrap_or_default();
                (e, lines)
            })
            .collect())
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
        repo::insert_entry_line(
            tx,
            entry_id,
            1,
            p.debit_account_id,
            p.amount,
            Decimal::ZERO,
            p.debit_desc,
        )
        .await?;
        repo::insert_entry_line(
            tx,
            entry_id,
            2,
            p.credit_account_id,
            Decimal::ZERO,
            p.amount,
            p.credit_desc,
        )
        .await?;
        Ok(entry_id)
    }

    /// 建立 AP 付款並過帳（借：應付 2100，貸：現金 1100）
    pub async fn create_ap_payment(
        pool: &PgPool,
        actor: &ActorContext,
        partner_id: Uuid,
        payment_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
    ) -> Result<Uuid> {
        if amount <= Decimal::ZERO {
            return Err(AppError::BadRequest("付款金額必須為正數".into()));
        }
        let created_by = actor.require_user()?.id;
        let mut tx = pool.begin().await?;
        let payment_no = repo::next_ap_payment_no(&mut tx).await?;
        let ap_id = Self::require_account_id(&mut tx, ACCT_AP, "應付帳款").await?;
        let cash_id = Self::require_account_id(&mut tx, ACCT_CASH, "現金").await?;

        let payment_id = Uuid::new_v4();
        let desc = format!("AP 應付帳款付款 {}", payment_no);
        let entry_id = Self::insert_cash_journal(
            &mut tx,
            CashJournalParams {
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
            },
        )
        .await?;

        repo::insert_ap_payment(
            &mut tx,
            payment_id,
            &payment_no,
            partner_id,
            payment_date,
            amount,
            reference.as_deref(),
            entry_id,
            created_by,
        )
        .await?;

        let snapshot = ApPaymentSnapshot {
            id: payment_id,
            payment_no: &payment_no,
            partner_id,
            payment_date,
            amount,
            reference: reference.as_deref(),
            journal_entry_id: entry_id,
        };
        let display = format!("AP {} ({})", payment_no, amount);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "AP_PAYMENT_CREATED",
                entity: Some(AuditEntity::new("ap_payment", payment_id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(payment_id)
    }

    /// 建立 AR 收款並過帳（借：現金 1100，貸：應收 1200）
    pub async fn create_ar_receipt(
        pool: &PgPool,
        actor: &ActorContext,
        partner_id: Uuid,
        receipt_date: NaiveDate,
        amount: Decimal,
        reference: Option<String>,
    ) -> Result<Uuid> {
        if amount <= Decimal::ZERO {
            return Err(AppError::BadRequest("收款金額必須為正數".into()));
        }
        let created_by = actor.require_user()?.id;
        let mut tx = pool.begin().await?;
        let receipt_no = repo::next_ar_receipt_no(&mut tx).await?;
        let ar_id = Self::require_account_id(&mut tx, ACCT_AR, "應收帳款").await?;
        let cash_id = Self::require_account_id(&mut tx, ACCT_CASH, "現金").await?;

        let receipt_id = Uuid::new_v4();
        let desc = format!("AR 應收帳款收款 {}", receipt_no);
        let entry_id = Self::insert_cash_journal(
            &mut tx,
            CashJournalParams {
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
            },
        )
        .await?;

        repo::insert_ar_receipt(
            &mut tx,
            receipt_id,
            &receipt_no,
            partner_id,
            receipt_date,
            amount,
            reference.as_deref(),
            entry_id,
            created_by,
        )
        .await?;

        let snapshot = ArReceiptSnapshot {
            id: receipt_id,
            receipt_no: &receipt_no,
            partner_id,
            receipt_date,
            amount,
            reference: reference.as_deref(),
            journal_entry_id: entry_id,
        };
        let display = format!("AR {} ({})", receipt_no, amount);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "AR_RECEIPT_CREATED",
                entity: Some(AuditEntity::new("ar_receipt", receipt_id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

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

#[cfg(test)]
mod tests {
    use super::{retain_postable_lines, validate_balanced, PostingLine};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn line(debit: i64, credit: i64) -> PostingLine {
        PostingLine {
            account_id: Uuid::nil(),
            debit: Decimal::new(debit, 0),
            credit: Decimal::new(credit, 0),
            description: String::new(),
        }
    }

    #[test]
    fn drops_zero_zero_lines() {
        // 借貸皆 0 的分錄（未填單價品項）會違反 chk_debit_credit → 必須剔除
        let kept = retain_postable_lines(vec![line(0, 0), line(100, 0), line(0, 0), line(0, 50)]);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].debit, Decimal::new(100, 0));
        assert_eq!(kept[1].credit, Decimal::new(50, 0));
    }

    #[test]
    fn all_zero_yields_empty() {
        // 全品項零金額（整張單無單價）→ 不應建立任何分錄（呼叫端據此略過空傳票）
        let kept = retain_postable_lines(vec![line(0, 0), line(0, 0)]);
        assert!(kept.is_empty());
    }

    #[test]
    fn keeps_all_nonzero_lines() {
        let kept = retain_postable_lines(vec![line(100, 0), line(0, 100)]);
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn validate_balanced_accepts_balanced_entry() {
        // 借存貨 100 = 貸應付 100
        assert!(validate_balanced(&[line(100, 0), line(0, 100)]).is_ok());
    }

    #[test]
    fn validate_balanced_rejects_unbalanced_entry() {
        // 負金額品項過濾後可能殘留：借 100、貸 70 → 不平衡
        assert!(validate_balanced(&[line(100, 0), line(0, 70)]).is_err());
    }

    #[test]
    fn validate_balanced_rejects_dual_sided_line() {
        // 單行同時有借與貸 → 方向不合法（違反 chk_debit_credit「恰一方 > 0」）
        assert!(validate_balanced(&[line(100, 50)]).is_err());
    }
}
