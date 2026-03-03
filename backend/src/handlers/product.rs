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
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory, ProductImportResult,
        ProductQuery, ProductWithUom, UpdateProductRequest,
    },
    require_permission,
    services::{AuditService, ProductService},
    AppError, AppState, Result,
};

/// 建立產品
pub async fn create_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.create");
    req.validate()?;
    
    let product = ProductService::create(&state.db, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "PRODUCT_CREATE",
        Some("product"), Some(product.product.id), Some(&product.product.name),
        None,
        Some(serde_json::json!({
            "name": product.product.name,
            "sku": product.product.sku,
        })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PRODUCT_CREATE): {}", e);
    }

    Ok(Json(product))
}

/// 列出所有產品
pub async fn list_products(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>> {
    require_permission!(current_user, "erp.product.view");
    
    let products = ProductService::list(&state.db, &query).await?;
    Ok(Json(products))
}

/// 取得單個產品
pub async fn get_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.view");
    
    let product = ProductService::get_by_id(&state.db, id).await?;
    Ok(Json(product))
}

/// 更新產品
pub async fn update_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.edit");
    req.validate()?;
    
    let product = ProductService::update(&state.db, id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "PRODUCT_UPDATE",
        Some("product"), Some(id), Some(&product.product.name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PRODUCT_UPDATE): {}", e);
    }

    Ok(Json(product))
}

/// 刪除產品
pub async fn delete_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.product.delete");

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "PRODUCT_DELETE",
        Some("product"), Some(id), None,
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PRODUCT_DELETE): {}", e);
    }
    
    ProductService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Product deleted successfully" })))
}

/// 列出所有產品分類
pub async fn list_categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProductCategory>>> {
    require_permission!(current_user, "erp.product.view");
    
    let categories = ProductService::list_categories(&state.db).await?;
    Ok(Json(categories))
}

/// 建立產品分類
pub async fn create_category(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<ProductCategory>> {
    require_permission!(current_user, "erp.product.create");
    req.validate()?;
    
    let category = ProductService::create_category(&state.db, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "CATEGORY_CREATE",
        Some("product_category"), Some(category.id), Some(&category.name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (CATEGORY_CREATE): {}", e);
    }

    Ok(Json(category))
}

/// 匯入產品（CSV 或 Excel）
pub async fn import_products(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ProductImportResult>> {
    require_permission!(current_user, "erp.product.create");

    let (file_data, file_name) = parse_product_import_file(&mut multipart).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = ProductService::import_products(&state.db, &file_data, &file_name).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "PRODUCT_IMPORT",
        Some("product"),
        None,
        Some(&format!(
            "匯入產品: {} (成功: {}, 失敗: {})",
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
        tracing::error!("寫入審計日誌失敗 (PRODUCT_IMPORT): {}", e);
    }

    Ok(Json(result))
}

/// 下載產品匯入模板
pub async fn download_product_import_template() -> Result<Response> {
    let data = ProductService::generate_import_template()
        .map_err(|e| AppError::Internal(format!("產生模板失敗: {}", e)))?;
    let filename = "product_import_template.xlsx";
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

async fn parse_product_import_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String)> {
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
