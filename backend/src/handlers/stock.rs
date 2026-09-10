use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AssignUnassignedRequest, InventoryOnHand, InventoryQuery, IssueLocationSuggestion,
        LotMovementsQuery, LotMovementsResponse, LowStockTotal, StockLedgerDetail,
        StockLedgerQuery, UnassignedInventory, UnassignedSourceDoc, UnassignedSourceQuery,
    },
    require_permission,
    services::StockService,
    AppState, Result,
};

/// `suggest_issue_locations` 的查詢參數。`product_id` 必填——沒有品項就無從建議。
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct SuggestIssueLocationQuery {
    pub product_id: uuid::Uuid,
}

/// 建 SO 時的儲位建議：該品項目前有貨、且倉庫被標為領用來源的儲位（FEFO 排序，排除過期）。
///
/// 權限用 `erp.stock.view` 而非開單權限：這支只讀庫存、不建立任何東西，
/// 回傳的也就是使用者本來就查得到的庫存現況，沒有理由要求更高的權限。
///
/// 回空陣列是正常結果，前端據此不預設任何儲位——沒有倉庫被勾為領用來源時
/// （migration 015 的預設狀態）就是這個情況。
pub async fn suggest_issue_locations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<SuggestIssueLocationQuery>,
) -> Result<Json<Vec<IssueLocationSuggestion>>> {
    require_permission!(current_user, "erp.stock.view");

    let suggestions = StockService::suggest_issue_locations(&state.db, query.product_id).await?;
    Ok(Json(suggestions))
}

/// 取得庫存現況
pub async fn get_inventory_on_hand(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<Vec<InventoryOnHand>>> {
    require_permission!(current_user, "erp.stock.view");

    let inventory = StockService::get_on_hand(&state.db, &query).await?;
    Ok(Json(inventory))
}

/// 取得庫存流水
pub async fn get_stock_ledger(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<StockLedgerQuery>,
) -> Result<Json<Vec<StockLedgerDetail>>> {
    require_permission!(current_user, "erp.stock.view");

    let ledger = StockService::get_ledger(&state.db, &query).await?;
    Ok(Json(ledger))
}

/// 批號完整生命週期查詢（R84-6）：時間軸 + 數量對帳，跨倉彙總
pub async fn get_lot_movements(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<LotMovementsQuery>,
) -> Result<Json<LotMovementsResponse>> {
    require_permission!(current_user, "erp.stock.view");

    let result = StockService::get_lot_movements(&state.db, &query).await?;
    Ok(Json(result))
}

/// 取得低庫存彙總清單（全公司總量 < 公司預設安全庫存；一品項一筆 + 各倉分布）
pub async fn get_low_stock_totals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<LowStockTotal>>> {
    require_permission!(current_user, "erp.stock.view");

    let alerts = StockService::get_low_stock_totals(&state.db).await?;
    Ok(Json(alerts))
}

/// 取得未分配庫存（倉庫層級有庫存，但未分配到任何儲位）
pub async fn get_unassigned_inventory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<Vec<UnassignedInventory>>> {
    require_permission!(current_user, "erp.stock.view");

    let rows = StockService::get_unassigned_inventory(&state.db, &query).await?;
    Ok(Json(rows))
}

/// 將未分配庫存分配至儲位
pub async fn assign_unassigned_inventory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignUnassignedRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.stock.adjust");

    StockService::assign_unassigned(&state.db, &req, &ActorContext::User(current_user.clone()))
        .await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// 取得造成未分配的來源 GRN 明細（追溯：這批未分配是哪張採購入庫單造成的）
pub async fn get_unassigned_sources(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UnassignedSourceQuery>,
) -> Result<Json<Vec<UnassignedSourceDoc>>> {
    require_permission!(current_user, "erp.stock.view");

    let rows = StockService::get_unassigned_sources(&state.db, &query).await?;
    Ok(Json(rows))
}
