use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    services::report::{
        CostSummaryReport, PurchaseLinesReport, ReportQuery, ReportService, SalesLinesReport,
        StockLedgerReport, StockOnHandReport,
    },
    AppState, Result,
};

/// 取得庫存現況報表
pub async fn get_stock_on_hand_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockOnHandReport>>> {
    let report = ReportService::stock_on_hand(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得庫存流水報表
pub async fn get_stock_ledger_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockLedgerReport>>> {
    let report = ReportService::stock_ledger(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得採購明細報表
pub async fn get_purchase_lines_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseLinesReport>>> {
    let report = ReportService::purchase_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得銷售明細報表
pub async fn get_sales_lines_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<SalesLinesReport>>> {
    let report = ReportService::sales_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得成本彙總報表
pub async fn get_cost_summary_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<CostSummaryReport>>> {
    let report = ReportService::cost_summary(&state.db, &query).await?;
    Ok(Json(report))
}
