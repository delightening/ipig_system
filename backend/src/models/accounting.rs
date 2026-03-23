//! 會計模組資料模型
//!
//! 包含會計科目、傳票、試算表、應付/應收帳齡、損益表等 DTO

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

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
