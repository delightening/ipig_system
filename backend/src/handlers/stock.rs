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

/// 取得低庫存警示清單
pub async fn get_low_stock_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<LowStockAlert>>> {
    require_permission!(current_user, "erp.stock.view");
    
    let alerts = StockService::get_low_stock_alerts(&state.db).await?;
    Ok(Json(alerts))
}
