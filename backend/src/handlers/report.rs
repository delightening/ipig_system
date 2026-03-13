use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    services::report::{
        BloodTestAnalysisQuery, BloodTestAnalysisRow,
        BloodTestCostReport, CostSummaryReport, PurchaseLinesReport,
        PurchaseSalesCategorySummary, PurchaseSalesMonthlySummary,
        PurchaseSalesPartnerSummary, ReportQuery, ReportService,
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
/// 與動物權限綁定：需 animal.record.view；若僅 view_project（無 view_all），僅回傳已指派計畫之動物
#[utoipa::path(get, path = "/api/reports/blood-test-analysis", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_blood_test_analysis(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<BloodTestAnalysisQuery>,
) -> Result<Json<Vec<BloodTestAnalysisRow>>> {
    if !current_user.has_permission("animal.record.view") {
        return Err(crate::AppError::Forbidden(
            "Permission denied: requires animal.record.view".to_string(),
        ));
    }
    let restrict = current_user.has_permission("animal.animal.view_project")
        && !current_user.has_permission("animal.animal.view_all");
    let report = ReportService::blood_test_analysis(&state.db, &query, restrict).await?;
    Ok(Json(report))
}

/// 進銷貨彙總 — 按月份
#[utoipa::path(get, path = "/api/reports/purchase-sales-monthly", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_monthly(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesMonthlySummary>>> {
    let report = ReportService::purchase_sales_monthly(&state.db, &query).await?;
    Ok(Json(report))
}

/// 進銷貨彙總 — 按供應商/客戶
#[utoipa::path(get, path = "/api/reports/purchase-sales-by-partner", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_by_partner(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesPartnerSummary>>> {
    let report = ReportService::purchase_sales_by_partner(&state.db, &query).await?;
    Ok(Json(report))
}

/// 進銷貨彙總 — 按產品類別
#[utoipa::path(get, path = "/api/reports/purchase-sales-by-category", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_by_category(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesCategorySummary>>> {
    let report = ReportService::purchase_sales_by_category(&state.db, &query).await?;
    Ok(Json(report))
}
