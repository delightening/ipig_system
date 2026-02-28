use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateStorageLocationRequest, StorageLocation, StorageLocationQuery,
        StorageLocationWithWarehouse, UpdateStorageLayoutRequest, UpdateStorageLocationRequest,
    },
    require_permission,
    services::StorageLocationService,
    AppState, Result,
};

/// 建立儲位
pub async fn create_storage_location(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateStorageLocationRequest>,
) -> Result<Json<StorageLocation>> {
    require_permission!(current_user, "erp.storage.create");
    req.validate()?;

    let location = StorageLocationService::create(&state.db, &req).await?;
    Ok(Json(location))
}

/// 列出儲位
pub async fn list_storage_locations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<StorageLocationQuery>,
) -> Result<Json<Vec<StorageLocationWithWarehouse>>> {
    require_permission!(current_user, "erp.storage.view");

    let locations = StorageLocationService::list(&state.db, &query).await?;
    Ok(Json(locations))
}

/// 取得單一儲位
pub async fn get_storage_location(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<StorageLocationWithWarehouse>> {
    require_permission!(current_user, "erp.storage.view");

    let location = StorageLocationService::get_by_id(&state.db, id).await?;
    Ok(Json(location))
}

/// 更新儲位
pub async fn update_storage_location(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStorageLocationRequest>,
) -> Result<Json<StorageLocation>> {
    require_permission!(current_user, "erp.storage.edit");
    req.validate()?;

    let location = StorageLocationService::update(&state.db, id, &req).await?;
    Ok(Json(location))
}

/// 批次更新倉庫佈局
pub async fn update_warehouse_layout(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(warehouse_id): Path<Uuid>,
    Json(req): Json<UpdateStorageLayoutRequest>,
) -> Result<Json<Vec<StorageLocation>>> {
    require_permission!(current_user, "erp.storage.edit");

    let locations = StorageLocationService::update_layout(&state.db, warehouse_id, &req).await?;
    Ok(Json(locations))
}

/// 刪除儲位
pub async fn delete_storage_location(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.storage.delete");

    StorageLocationService::delete(&state.db, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Storage location deleted successfully" }),
    ))
}

/// 產生儲位代碼
pub async fn generate_storage_location_code(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.storage.create");

    let code = StorageLocationService::generate_code(&state.db, warehouse_id).await?;
    Ok(Json(serde_json::json!({ "code": code })))
}

/// 取得儲位庫存明細
pub async fn get_storage_location_inventory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::StorageLocationInventoryItem>>> {
    require_permission!(current_user, "erp.storage.view");

    let items = StorageLocationService::get_inventory(&state.db, id).await?;
    Ok(Json(items))
}

/// 更新儲位庫存項目數量
pub async fn update_storage_location_inventory_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(item_id): Path<Uuid>,
    Json(req): Json<crate::models::UpdateStorageLocationInventoryItemRequest>,
) -> Result<Json<crate::models::StorageLocationInventoryItem>> {
    // 特定權限：僅管理員可直接修改庫存
    require_permission!(current_user, "erp.storage.inventory.edit");
    // Note: validation for rust_decimal::Decimal is done in the service layer

    let item = StorageLocationService::update_inventory_item(&state.db, item_id, &req).await?;
    Ok(Json(item))
}

/// 新增儲位庫存項目
pub async fn create_storage_location_inventory_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(storage_location_id): Path<Uuid>,
    Json(req): Json<crate::models::CreateStorageLocationInventoryItemRequest>,
) -> Result<Json<crate::models::StorageLocationInventoryItem>> {
    require_permission!(current_user, "erp.storage.inventory.edit");
    req.validate()?;

    let item = StorageLocationService::create_inventory_item(&state.db, storage_location_id, &req).await?;
    Ok(Json(item))
}

/// 調撥儲位庫存 (同倉庫內)
pub async fn transfer_storage_location_inventory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(item_id): Path<Uuid>,
    Json(req): Json<crate::models::TransferStorageLocationInventoryRequest>,
) -> Result<Json<crate::models::StorageLocationInventoryItem>> {
    require_permission!(current_user, "erp.storage.inventory.edit");
    req.validate()?;

    let item = StorageLocationService::transfer_inventory(&state.db, item_id, &req).await?;
    Ok(Json(item))
}
