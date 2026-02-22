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
    services::{AuditService, WarehouseService},
    AppError, AppState, Result,
};

/// 建立倉庫
#[utoipa::path(post, path = "/api/warehouses", request_body = CreateWarehouseRequest, responses((status = 200, description = "建立成功", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn create_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let warehouse = WarehouseService::create(&state.db, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "WAREHOUSE_CREATE",
        Some("warehouse"), Some(warehouse.id), Some(&warehouse.name),
        None,
        Some(serde_json::json!({
            "name": warehouse.name,
            "code": warehouse.code,
        })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (WAREHOUSE_CREATE): {}", e);
    }

    Ok(Json(warehouse))
}

/// 列出所有倉庫
#[utoipa::path(get, path = "/api/warehouses", responses((status = 200, description = "倉庫清單", body = Vec<Warehouse>)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn list_warehouses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WarehouseQuery>,
) -> Result<Json<Vec<Warehouse>>> {
    require_permission!(current_user, "erp.warehouse.view");
    
    let warehouses = WarehouseService::list(&state.db, &query).await?;
    Ok(Json(warehouses))
}

/// 取得單個倉庫
#[utoipa::path(get, path = "/api/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "倉庫資訊", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn get_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.view");
    
    let warehouse = WarehouseService::get_by_id(&state.db, id).await?;
    Ok(Json(warehouse))
}

/// 更新倉庫
#[utoipa::path(put, path = "/api/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), request_body = UpdateWarehouseRequest, responses((status = 200, description = "更新成功", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn update_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let warehouse = WarehouseService::update(&state.db, id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "WAREHOUSE_UPDATE",
        Some("warehouse"), Some(id), Some(&warehouse.name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (WAREHOUSE_UPDATE): {}", e);
    }

    Ok(Json(warehouse))
}

/// 刪除倉庫
#[utoipa::path(delete, path = "/api/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "刪除成功")), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn delete_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.warehouse.delete");

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "WAREHOUSE_DELETE",
        Some("warehouse"), Some(id), None,
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (WAREHOUSE_DELETE): {}", e);
    }
    
    WarehouseService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Warehouse deleted successfully" })))
}
