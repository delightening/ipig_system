use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{CreateWarehouseRequest, UpdateWarehouseRequest, Warehouse, WarehouseQuery},
    require_permission,
    services::WarehouseService,
    AppError, AppState, Result,
};

/// 撱箇??澈
pub async fn create_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let warehouse = WarehouseService::create(&state.db, &req).await?;
    Ok(Json(warehouse))
}

/// ???澈?”
pub async fn list_warehouses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WarehouseQuery>,
) -> Result<Json<Vec<Warehouse>>> {
    require_permission!(current_user, "erp.warehouse.view");
    
    let warehouses = WarehouseService::list(&state.db, &query).await?;
    Ok(Json(warehouses))
}

/// ???桐??澈
pub async fn get_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.view");
    
    let warehouse = WarehouseService::get_by_id(&state.db, id).await?;
    Ok(Json(warehouse))
}

/// ?湔?澈
pub async fn update_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let warehouse = WarehouseService::update(&state.db, id, &req).await?;
    Ok(Json(warehouse))
}

/// ?芷?澈
pub async fn delete_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.warehouse.delete");
    
    WarehouseService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Warehouse deleted successfully" })))
}

