use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::report::{
        BloodTestAnalysisQuery, BloodTestAnalysisRow, BloodTestCostReport, CostSummaryReport,
        ProtocolConsumptionReport, PurchaseLinesReport, PurchaseSalesCategorySummary,
        PurchaseSalesMonthlySummary, PurchaseSalesPartnerSummary, ReportQuery, ReportService,
        SalesLinesReport, StockLedgerReport, StockOnHandReport,
    },
    AppState, Result,
};

/// 取得庫存現況報表
#[utoipa::path(get, path = "/api/v1/reports/stock-on-hand", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_stock_on_hand_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockOnHandReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::stock_on_hand(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得庫存流水報表
#[utoipa::path(get, path = "/api/v1/reports/stock-ledger", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_stock_ledger_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<StockLedgerReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::stock_ledger(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得採購明細報表
#[utoipa::path(get, path = "/api/v1/reports/purchase-lines", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_lines_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseLinesReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::purchase_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得案件消耗報表
///
/// 權限沿用 `erp.report.view`，與其餘 ERP 報表一致。
///
/// # 🔴 本端點沒有物件層授權
///
/// `query.protocol_id` 由呼叫端任意指定，`ReportService::protocol_consumption`
/// 直接把它推進 WHERE（`services/report.rs`），**不檢查這個計畫與呼叫者有無關係**；
/// 不帶 `protocol_id` 就是全部案件。
///
/// 現在沒有外洩，是因為 `erp.report.view` 目前只授予 WAREHOUSE_MANAGER／
/// PURCHASING／ADMIN_STAFF——這三個角色本來就該看全廠。PI 的權限清單裡
/// 零個 `erp.*`（`startup/permissions.rs` 的 PI 區塊），所以 PI 進不了這道閘。
///
/// ⚠️ **這是角色表的現況，不是程式碼給的保證。** 只要有人把 `erp.report.view`
/// 加進任何「只該看自己案子」的角色，當天就會變成「甲 PI 看得到乙 PI」，
/// 而這裡不會有任何東西擋下來，也不會有測試轉紅。
///
/// # 日後要開放給「只該看自己計畫」的身分時
///
/// 必須把查詢**綁回 `current_user`**：依實際的成員關係（計畫主持人／協同人員）
/// 收斂 protocol 範圍，而不是相信呼叫端送來的 `protocol_id`。
///
/// 🔴 **不要照抄 `get_blood_test_analysis`。** 它看起來像樣板，其實不是：
/// 它的 `restrict` 旗標（`animal.animal.view_project` 且非 `view_all`）傳進
/// `ReportService::blood_test_analysis` 之後，實際只加一條
/// `AND a.iacuc_no IS NOT NULL`——那排除的是**還沒掛計畫的動物**，
/// 不是**不屬於我的計畫**。該函式的參數名 `restrict_to_project_animals`
/// 與它的 doc 都是這樣寫的，它從未宣稱做後者。而且它的 `query.iacuc_no`
/// 同樣由呼叫端指定，restrict 為真時也不阻止你填別人的編號。
/// 照抄它不會解決「甲看到乙」，只會讓人以為解決了。
#[utoipa::path(get, path = "/api/v1/reports/protocol-consumption", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_protocol_consumption_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<ProtocolConsumptionReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::protocol_consumption(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得銷貨明細報表
#[utoipa::path(get, path = "/api/v1/reports/sales-lines", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_sales_lines_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<SalesLinesReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::sales_lines(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得成本彙總報表
#[utoipa::path(get, path = "/api/v1/reports/cost-summary", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_cost_summary_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<CostSummaryReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::cost_summary(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得血液檢查費用報表
#[utoipa::path(get, path = "/api/v1/reports/blood-test-cost", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_blood_test_cost_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<BloodTestCostReport>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::blood_test_cost(&state.db, &query).await?;
    Ok(Json(report))
}

/// 取得血液檢查結果分析資料
/// 與動物權限綁定：需 animal.record.view；若僅 view_project（無 view_all），僅回傳已指派計畫之動物
#[utoipa::path(get, path = "/api/v1/reports/blood-test-analysis", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
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
#[utoipa::path(get, path = "/api/v1/reports/purchase-sales-monthly", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_monthly(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesMonthlySummary>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::purchase_sales_monthly(&state.db, &query).await?;
    Ok(Json(report))
}

/// 進銷貨彙總 — 按供應商/客戶
#[utoipa::path(get, path = "/api/v1/reports/purchase-sales-by-partner", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_by_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesPartnerSummary>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::purchase_sales_by_partner(&state.db, &query).await?;
    Ok(Json(report))
}

/// 進銷貨彙總 — 按產品類別
#[utoipa::path(get, path = "/api/v1/reports/purchase-sales-by-category", responses((status = 200)), tag = "報表", security(("bearer" = [])))]
pub async fn get_purchase_sales_by_category(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Vec<PurchaseSalesCategorySummary>>> {
    require_permission!(current_user, "erp.report.view");
    let report = ReportService::purchase_sales_by_category(&state.db, &query).await?;
    Ok(Json(report))
}
