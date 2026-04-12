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

use crate::error::ErrorResponse;
use crate::{
    middleware::CurrentUser,
    models::{
        ChangeProductStatusRequest, CreateCategoryRequest, CreateProductRequest, Product,
        ProductCategory, ProductImportCheckResult, ProductImportPreviewResult, ProductImportResult,
        ProductQuery, ProductWithUom, UpdateProductRequest,
    },
    require_permission,
    services::{AuditService, ProductService},
    AppError, AppState, Result,
};

/// 建立產品
#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductRequest,
    responses(
        (status = 200, description = "建立成功", body = ProductWithUom),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    get,
    path = "/api/v1/products",
    params(ProductQuery),
    responses(
        (status = 200, description = "產品清單", body = Vec<Product>),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    params(("id" = Uuid, Path, description = "產品 ID")),
    responses(
        (status = 200, description = "產品詳細", body = ProductWithUom),
        (status = 401, description = "未認證", body = ErrorResponse),
        (status = 404, description = "找不到產品", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    put,
    path = "/api/v1/products/{id}",
    params(("id" = Uuid, Path, description = "產品 ID")),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "更新成功", body = ProductWithUom),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
        (status = 404, description = "找不到產品", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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

/// 更新產品狀態（啟用/停用/停產）
pub async fn update_product_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeProductStatusRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.edit");

    let product = ProductService::update_status(&state.db, id, &req.status).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "PRODUCT_STATUS_CHANGE",
        Some("product"),
        Some(id),
        Some(&product.product.name),
        None,
        Some(serde_json::json!({ "status": req.status })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (PRODUCT_STATUS_CHANGE): {}", e);
    }

    Ok(Json(product))
}

/// 刪除產品（軟刪除）
#[utoipa::path(
    delete,
    path = "/api/v1/products/{id}",
    params(("id" = Uuid, Path, description = "產品 ID")),
    responses(
        (status = 200, description = "刪除成功"),
        (status = 401, description = "未認證", body = ErrorResponse),
        (status = 404, description = "找不到產品", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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

/// 硬刪除產品（僅 admin；無單據/庫存/藥物關聯時才可執行）
#[utoipa::path(
    post,
    path = "/api/v1/products/{id}/hard-delete",
    params(("id" = Uuid, Path, description = "產品 ID")),
    responses(
        (status = 200, description = "硬刪除成功"),
        (status = 401, description = "未認證", body = ErrorResponse),
        (status = 403, description = "僅管理員可執行硬刪除", body = ErrorResponse),
        (status = 404, description = "找不到產品", body = ErrorResponse),
        (status = 409, description = "產品有關聯資料無法硬刪除", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn hard_delete_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可執行產品硬刪除".into()));
    }

    let product_name: Option<String> = sqlx::query_scalar("SELECT name FROM products WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "PRODUCT_HARD_DELETE",
        Some("product"),
        Some(id),
        product_name.as_deref(),
        None,
        Some(serde_json::json!({ "action": "hard_delete" })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (PRODUCT_HARD_DELETE): {}", e);
    }

    ProductService::hard_delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Product hard deleted successfully" })))
}

/// 列出所有產品分類
#[utoipa::path(
    get,
    path = "/api/v1/categories",
    responses(
        (status = 200, description = "分類清單", body = Vec<ProductCategory>),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn list_categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProductCategory>>> {
    require_permission!(current_user, "erp.product.view");
    
    let categories = ProductService::list_categories(&state.db).await?;
    Ok(Json(categories))
}

/// 建立產品分類
#[utoipa::path(
    post,
    path = "/api/v1/categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 200, description = "建立成功", body = ProductCategory),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
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

/// 匯入預檢：檢查名稱+規格是否與既有產品重複（規則一）
pub async fn check_product_import_duplicates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ProductImportCheckResult>> {
    require_permission!(current_user, "erp.product.create");

    let (file_data, file_name, _, _) = parse_product_import_file(&mut multipart, false).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = ProductService::check_import_duplicates(&state.db, &file_data, &file_name).await?;
    Ok(Json(result))
}

/// 匯入預覽：回傳解析後的列資料，供前端「依序設定 SKU」使用
#[utoipa::path(
    post,
    path = "/api/v1/products/import/preview",
    responses(
        (status = 200, description = "預覽列資料", body = ProductImportPreviewResult),
        (status = 400, description = "驗證失敗或檔案格式錯誤", body = ErrorResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn preview_product_import(
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ProductImportPreviewResult>> {
    require_permission!(current_user, "erp.product.create");

    let (file_data, file_name, _, _) = parse_product_import_file(&mut multipart, false).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = ProductService::preview_import(&file_data, &file_name)?;
    Ok(Json(result))
}

/// 匯入產品（CSV 或 Excel）
#[utoipa::path(
    post,
    path = "/api/v1/products/import",
    responses(
        (status = 200, description = "匯入結果 (multipart/form-data, field file: CSV/Excel)", body = ProductImportResult),
        (status = 400, description = "驗證失敗或檔案格式錯誤", body = ErrorResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn import_products(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ProductImportResult>> {
    require_permission!(current_user, "erp.product.create");

    let (file_data, file_name, skip_duplicates, regenerate_sku_for_duplicates) =
        parse_product_import_file(&mut multipart, true).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = ProductService::import_products(
        &state.db,
        &file_data,
        &file_name,
        skip_duplicates,
        regenerate_sku_for_duplicates,
    )
    .await?;

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
#[utoipa::path(
    get,
    path = "/api/v1/products/import/template",
    responses(
        (status = 200, description = "Excel 模板檔案 (application/vnd.openxmlformats-officedocument.spreadsheetml.sheet)"),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn download_product_import_template() -> Result<Response> {
    let data = ProductService::generate_import_template()
        .map_err(|e| AppError::Internal(format!("產生模板失敗: {}", e)))?;
    let filename = "product_import_template.xlsx";
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 解析匯入請求。`parse_options` 為 true 時會讀取 form 欄位 skip_duplicates、regenerate_sku_for_duplicates。
async fn parse_product_import_file(
    multipart: &mut Multipart,
    parse_options: bool,
) -> Result<(Vec<u8>, String, bool, bool)> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");
    let mut skip_duplicates = false;
    let mut regenerate_sku_for_duplicates = false;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("解析檔案欄位失敗: {}", e)))?
    {
        match field.name() {
            Some("file") => {
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
            Some("skip_duplicates") if parse_options => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("讀取欄位失敗: {}", e)))?;
                let s = String::from_utf8_lossy(&bytes);
                skip_duplicates = matches!(s.trim().to_lowercase().as_str(), "true" | "1" | "yes");
            }
            Some("regenerate_sku_for_duplicates") if parse_options => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("讀取欄位失敗: {}", e)))?;
                let s = String::from_utf8_lossy(&bytes);
                regenerate_sku_for_duplicates =
                    matches!(s.trim().to_lowercase().as_str(), "true" | "1" | "yes");
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or_else(|| AppError::Validation("未找到檔案".to_string()))?;
    Ok((file_data, file_name, skip_duplicates, regenerate_sku_for_duplicates))
}
