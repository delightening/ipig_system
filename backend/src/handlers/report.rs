use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::access,
    services::report::{
        BloodTestAnalysisQuery, BloodTestAnalysisRow, BloodTestCostReport, CostSummaryReport,
        ProtocolConsumptionReport, PurchaseLinesReport, PurchaseSalesCategorySummary,
        PurchaseSalesMonthlySummary, PurchaseSalesPartnerSummary, ReportQuery, ReportService,
        SalesLinesReport, StockLedgerReport, StockOnHandReport,
    },
    AppState, Result,
};

/// 取得庫存現況報表
#[utoipa::path(get, path = "/api/v1/reports/stock-on-hand", responses((status = 200, description = "庫存現量報表")), tag = "報表", security(("bearer" = [])))]
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
#[utoipa::path(get, path = "/api/v1/reports/stock-ledger", responses((status = 200, description = "庫存異動明細報表")), tag = "報表", security(("bearer" = [])))]
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
#[utoipa::path(get, path = "/api/v1/reports/purchase-lines", responses((status = 200, description = "採購明細報表")), tag = "報表", security(("bearer" = [])))]
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
/// 可參考 `get_blood_test_analysis` 的做法：先向 `services::access` 取呼叫者的
/// 讀取邊界（可存取的計畫 id 集合，`None` 才是不限），再把邊界 AND 進查詢，
/// 呼叫端送來的篩選條件只能在邊界之內縮小範圍。
/// ⚠️ 只排除「還沒掛計畫的資料」不算收斂——那擋不住填別人的計畫編號。
/// # 回傳上限與截斷訊號（🔴 消費端必讀）
///
/// 本端點最多回 **1001** 筆，而第 1001 筆是**哨兵不是資料**：
///
/// - 收到 **1001 筆** → 結果被截斷，來源還有更多。第 1001 筆必須**捨棄**，
///   並向使用者提示「這不是全部」。
/// - 收到 **≤ 1000 筆** → 就是全部，沒有遺漏。
///
/// 用 `LIMIT 1001` 而不是 `LIMIT 1000` 的理由：只取 1000 筆的話，拿到剛好 1000 筆時
/// **分不出**「剛好取完」與「還有更多」。多取一筆就把這兩種狀態分開了。
/// （見 `ReportService::protocol_consumption` 的 `LIMIT 1001`。）
///
/// ⚠️ **這段規則原本只寫在前端的註解裡，不在本契約上**（CodeRabbit 於 MR !15 指出，
/// 2026-09-11，判斷成立）。後果是：本 repo 的前端知道規則所以處理正確，而**任何其他
/// 消費端拿到 1001 筆會當成 1001 筆資料**——多顯示一筆哨兵，且完全不知道資料殘缺。
/// 錯的方向特別壞：使用者看到的是一份**看起來完整、實際少了一大截**的報表。
///
/// 本次選擇把哨兵寫進契約，而不是把回傳改成 `{ rows, truncated }`：訊號本身可靠且
/// 無歧義（1001 就是截斷），真正的缺陷是它沒有被寫下來。改變回傳形狀要同步動
/// handler／service／openapi／前端型別／guest-demo 五處，屬 API contract 改動——
/// **日後若要做，應另開 MR 走完整審查，不要夾帶在別的變更裡。**
#[utoipa::path(
    get,
    path = "/api/v1/reports/protocol-consumption",
    responses((status = 200, description = "\
        最多回 1001 筆。回滿 1001 筆代表結果被截斷，第 1001 筆是哨兵不是資料，\
        消費端須捨棄該筆並向使用者提示資料殘缺；≤ 1000 筆代表結果完整。")),
    tag = "報表",
    security(("bearer" = []))
)]
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
#[utoipa::path(get, path = "/api/v1/reports/sales-lines", responses((status = 200, description = "銷貨明細報表")), tag = "報表", security(("bearer" = [])))]
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
#[utoipa::path(get, path = "/api/v1/reports/cost-summary", responses((status = 200, description = "成本彙總報表")), tag = "報表", security(("bearer" = [])))]
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
#[utoipa::path(get, path = "/api/v1/reports/blood-test-cost", responses((status = 200, description = "血檢費用報表")), tag = "報表", security(("bearer" = [])))]
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
///
/// 需 `animal.record.view`。讀取範圍與單筆動物紀錄的讀取守衛
/// （`access::require_animal_read_access`）用同一個判準：可跨計畫讀取者不限；
/// 其餘只回傳自己可存取之計畫底下的動物，`iacuc_no` / `animal_id` 查詢參數
/// 只能在這個範圍內再縮小。
#[utoipa::path(get, path = "/api/v1/reports/blood-test-analysis", responses((status = 200, description = "血檢分析報表")), tag = "報表", security(("bearer" = [])))]
pub async fn get_blood_test_analysis(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<BloodTestAnalysisQuery>,
) -> Result<Json<Vec<BloodTestAnalysisRow>>> {
    require_permission!(current_user, "animal.record.view");
    let boundary = access::animal_read_boundary(&state.db, &current_user).await?;
    let report = ReportService::blood_test_analysis(&state.db, &query, boundary.as_deref()).await?;
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
