//! 會計相關 API：試算表、傳票、會計科目、AP/AR 帳齡、付款/收款

use axum::{
    extract::{Query, State},
    Extension, Json,
};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::accounting::{
        ApAgingRow, ArAgingRow, ChartOfAccount, JournalEntryLineRow, JournalEntryRow,
        ProfitLossSummary, TrialBalanceRow,
    },
    require_permission,
    services::AccountingService,
    time,
    AppState, Result,
};

/// 會計報表查詢參數
#[derive(Debug, Deserialize)]
pub struct AccountingQuery {
    pub as_of_date: Option<NaiveDate>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

/// 建立 AP 付款請求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateApPaymentRequest {
    pub partner_id: Uuid,
    pub payment_date: NaiveDate,
    pub amount: Decimal,
    pub reference: Option<String>,
}

/// 建立 AR 收款請求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateArReceiptRequest {
    pub partner_id: Uuid,
    pub receipt_date: NaiveDate,
    pub amount: Decimal,
    pub reference: Option<String>,
}

/// 會計傳票回應（含分錄行）
#[derive(Debug, serde::Serialize)]
pub struct JournalEntryResponse {
    pub entry: JournalEntryRow,
    pub lines: Vec<JournalEntryLineRow>,
}

/// 取得會計科目表
#[utoipa::path(get, path = "/api/accounting/chart-of-accounts", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_chart_of_accounts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ChartOfAccount>>> {
    require_permission!(current_user, "erp.report.view");
    let accounts = AccountingService::list_chart_of_accounts(&state.db).await?;
    Ok(Json(accounts))
}

/// 取得試算表
#[utoipa::path(get, path = "/api/accounting/trial-balance", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_trial_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AccountingQuery>,
) -> Result<Json<Vec<TrialBalanceRow>>> {
    require_permission!(current_user, "erp.report.view");
    let as_of = query.as_of_date.unwrap_or_else(time::today_taiwan_naive);
    let rows = AccountingService::get_trial_balance(&state.db, as_of).await?;
    Ok(Json(rows))
}

/// 取得傳票清單
#[utoipa::path(get, path = "/api/accounting/journal-entries", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_journal_entries(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AccountingQuery>,
) -> Result<Json<Vec<JournalEntryResponse>>> {
    require_permission!(current_user, "erp.report.view");
    let limit = query.limit.unwrap_or(100);
    let entries =
        AccountingService::list_journal_entries(
            &state.db,
            query.date_from,
            query.date_to,
            limit,
        )
        .await?;
    let response: Vec<JournalEntryResponse> = entries
        .into_iter()
        .map(|(entry, lines)| JournalEntryResponse { entry, lines })
        .collect();
    Ok(Json(response))
}

/// 取得應付帳款帳齡
#[utoipa::path(get, path = "/api/accounting/ap-aging", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_ap_aging(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AccountingQuery>,
) -> Result<Json<Vec<ApAgingRow>>> {
    require_permission!(current_user, "erp.report.view");
    let as_of = query.as_of_date.unwrap_or_else(time::today_taiwan_naive);
    let rows = AccountingService::get_ap_aging(&state.db, as_of).await?;
    Ok(Json(rows))
}

/// 取得應收帳款帳齡
#[utoipa::path(get, path = "/api/accounting/ar-aging", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_ar_aging(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AccountingQuery>,
) -> Result<Json<Vec<ArAgingRow>>> {
    require_permission!(current_user, "erp.report.view");
    let as_of = query.as_of_date.unwrap_or_else(time::today_taiwan_naive);
    let rows = AccountingService::get_ar_aging(&state.db, as_of).await?;
    Ok(Json(rows))
}

/// 建立 AP 付款
#[utoipa::path(post, path = "/api/accounting/ap-payments", request_body = CreateApPaymentRequest, responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn create_ap_payment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateApPaymentRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.document.create");
    let id = AccountingService::create_ap_payment(
        &state.db,
        req.partner_id,
        req.payment_date,
        req.amount,
        req.reference,
        current_user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

/// 建立 AR 收款
#[utoipa::path(post, path = "/api/accounting/ar-receipts", request_body = CreateArReceiptRequest, responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn create_ar_receipt(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateArReceiptRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.document.create");
    let id = AccountingService::create_ar_receipt(
        &state.db,
        req.partner_id,
        req.receipt_date,
        req.amount,
        req.reference,
        current_user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

/// 取得損益表
#[utoipa::path(get, path = "/api/accounting/profit-loss", responses((status = 200)), tag = "會計", security(("bearer" = [])))]
pub async fn get_profit_loss(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AccountingQuery>,
) -> Result<Json<ProfitLossSummary>> {
    require_permission!(current_user, "erp.report.view");
    let summary = AccountingService::get_profit_loss(
        &state.db,
        query.date_from,
        query.date_to,
    )
    .await?;
    Ok(Json(summary))
}
