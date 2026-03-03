use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use axum::extract::Multipart;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateWarehouseRequest, UpdateWarehouseRequest, Warehouse, WarehouseImportResult,
        WarehouseQuery,
    },
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
    req.validate()?;
    
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
    req.validate()?;
    
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

/// 匯入倉庫（CSV 或 Excel）
pub async fn import_warehouses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<WarehouseImportResult>> {
    require_permission!(current_user, "erp.warehouse.create");

    let (file_data, file_name) = parse_warehouse_import_file(&mut multipart).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = WarehouseService::import_warehouses(&state.db, &file_data, &file_name).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "WAREHOUSE_IMPORT",
        Some("warehouse"),
        None,
        Some(&format!(
            "匯入倉庫: {} (成功: {}, 失敗: {})",
            file_name, result.success_count, result.error_count
        )),
        None,
        Some(serde_json::json!({
            "file_name": file_name,
            "success_count": result.success_count,
            "error_count": result.error_count
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (WAREHOUSE_IMPORT): {}", e);
    }

    Ok(Json(result))
}

/// 下載倉庫匯入模板
pub async fn download_warehouse_import_template() -> Result<Response> {
    let data = WarehouseService::generate_import_template()
        .map_err(|e| AppError::Internal(format!("產生模板失敗: {}", e)))?;
    let filename = "warehouse_import_template.xlsx";
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

async fn parse_warehouse_import_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String)> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("解析檔案欄位失敗: {}", e)))?
    {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("讀取檔案資料失敗: {}", e)))?;
            file_data = Some(data.to_vec());
        }
    }
    let file_data = file_data.ok_or_else(|| AppError::Validation("未找到檔案".to_string()))?;
    Ok((file_data, file_name))
}
