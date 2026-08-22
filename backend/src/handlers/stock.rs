use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AssignUnassignedRequest, InventoryOnHand, InventoryQuery, LotMovementsQuery,
        LotMovementsResponse, LowStockTotal, StockLedgerDetail, StockLedgerQuery,
        UnassignedInventory, UnassignedSourceDoc, UnassignedSourceQuery,
    },
    require_permission,
    services::StockService,
    AppState, Result,
};

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
