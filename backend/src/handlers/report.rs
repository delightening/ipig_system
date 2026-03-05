use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    services::report::{
        BloodTestAnalysisQuery, BloodTestAnalysisRow,
        BloodTestCostReport, CostSummaryReport, PurchaseLinesReport, ReportQuery, ReportService,
        SalesLinesReport, StockLedgerReport, StockOnHandReport,
    },
    AppState, Result,
};

/// 取得庫存現況報表
#[utoipa::path(get, path = "/api/reports/stock-on-hand", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_stock_on_hand_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockOnHandReport>>> {
    let report = ReportService::stock_on_hand(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得庫存流水報表
#[utoipa::path(get, path = "/api/reports/stock-ledger", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_stock_ledger_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockLedgerReport>>> {
    let report = ReportService::stock_ledger(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得採購明細報表
#[utoipa::path(get, path = "/api/reports/purchase-lines", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_lines_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseLinesReport>>> {
    let report = ReportService::purchase_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得銷貨明細報表
#[utoipa::path(get, path = "/api/reports/sales-lines", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_sales_lines_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<SalesLinesReport>>> {
    let report = ReportService::sales_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得成本彙總報表
#[utoipa::path(get, path = "/api/reports/cost-summary", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_cost_summary_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<CostSummaryReport>>> {
    let report = ReportService::cost_summary(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得血液檢查費用報表
#[utoipa::path(get, path = "/api/reports/blood-test-cost", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_blood_test_cost_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<BloodTestCostReport>>> {
    let report = ReportService::blood_test_cost(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得血液檢查結果分析資料
#[utoipa::path(get, path = "/api/reports/blood-test-analysis", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_blood_test_analysis(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<BloodTestAnalysisQuery>,
) -> Result<Json<Vec<BloodTestAnalysisRow>>> {
    let report = ReportService::blood_test_analysis(&state.db, &query).await?;
    Ok(Json(report))
}
