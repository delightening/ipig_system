use axum::{
    extract::{Query, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    models::{InventoryOnHand, InventoryQuery, LowStockAlert, StockLedgerDetail, StockLedgerQuery},
    require_permission,
    services::StockService,
    AppState, Result,
};

/// 查詢庫存現況
pub async fn get_inventory_on_hand(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<Vec<InventoryOnHand>>> {
    require_permission!(current_user, "stock.read");
    
    let inventory = StockService::get_on_hand(&state.db, &query).await?;
    Ok(Json(inventory))
}

/// 查詢庫存流水
pub async fn get_stock_ledger(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<StockLedgerQuery>,
) -> Result<Json<Vec<StockLedgerDetail>>> {
    require_permission!(current_user, "stock.read");
    
    let ledger = StockService::get_ledger(&state.db, &query).await?;
    Ok(Json(ledger))
}

/// 查詢低庫存警示
pub async fn get_low_stock_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<LowStockAlert>>> {
    require_permission!(current_user, "stock.read");
    
    let alerts = StockService::get_low_stock_alerts(&state.db).await?;
    Ok(Json(alerts))
}
