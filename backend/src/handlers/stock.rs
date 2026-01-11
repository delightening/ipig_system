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

/// ?亥岷摨怠??暹?
pub async fn get_inventory_on_hand(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<Vec<InventoryOnHand>>> {
    require_permission!(current_user, "erp.stock.view");
    
    let inventory = StockService::get_on_hand(&state.db, &query).await?;
    Ok(Json(inventory))
}

/// ?亥岷摨怠?瘚偌
pub async fn get_stock_ledger(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<StockLedgerQuery>,
) -> Result<Json<Vec<StockLedgerDetail>>> {
    require_permission!(current_user, "erp.stock.view");
    
    let ledger = StockService::get_ledger(&state.db, &query).await?;
    Ok(Json(ledger))
}

/// ?亥岷雿澈摮郎蝷?
pub async fn get_low_stock_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<LowStockAlert>>> {
    require_permission!(current_user, "erp.stock.view");
    
    let alerts = StockService::get_low_stock_alerts(&state.db).await?;
    Ok(Json(alerts))
}

