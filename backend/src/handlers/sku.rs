use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

use axum::extract::Path as PathExtract;

use crate::{
    middleware::CurrentUser,
    models::{
        CategoriesResponse, CategoriesTreeResponse, CreateProductWithSkuRequest,
        CreateSkuSubcategoryRequest, GenerateSkuRequest, GenerateSkuResponse, ProductWithUom,
        SkuPreviewRequest, SkuPreviewResponse, SubcategoriesResponse, UpdateSkuCategoryRequest,
        UpdateSkuSubcategoryRequest, ValidateSkuRequest, ValidateSkuResponse,
    },
    require_permission,
    services::{AuditService, SkuService},
    AppError, AppState, Result,
};

/// 列出 SKU 分類清單
#[utoipa::path(
    get,
    path = "/api/sku/categories",
    responses(
        (status = 200, description = "分類清單", body = CategoriesResponse),
        (status = 401, description = "未認證"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn get_sku_categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<CategoriesResponse>> {
    require_permission!(current_user, "erp.product.view");

    let categories = SkuService::get_categories(&state.db).await?;
    Ok(Json(categories))
}

/// 列出 SKU 子分類清單
#[utoipa::path(
    get,
    path = "/api/sku/categories/{code}/subcategories",
    params(("code" = String, Path, description = "分類代碼")),
    responses(
        (status = 200, description = "子分類清單", body = SubcategoriesResponse),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到分類"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn get_sku_subcategories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(code): Path<String>,
) -> Result<Json<SubcategoriesResponse>> {
    require_permission!(current_user, "erp.product.view");

    let subcategories = SkuService::get_subcategories(&state.db, &code).await?;
    Ok(Json(subcategories))
}

/// 取得完整品類樹（含停用項）供編輯分類使用
#[utoipa::path(
    get,
    path = "/api/sku/categories/tree",
    responses(
        (status = 200, description = "品類樹", body = CategoriesTreeResponse),
        (status = 401, description = "未認證"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn get_sku_categories_tree(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<CategoriesTreeResponse>> {
    require_permission!(current_user, "erp.product.edit");

    let tree = SkuService::get_categories_tree(&state.db).await?;
    Ok(Json(tree))
}

/// 更新品類（名稱、排序、啟用狀態）
#[utoipa::path(
    patch,
    path = "/api/sku/categories/{code}",
    params(("code" = String, Path, description = "品類代碼")),
    request_body = UpdateSkuCategoryRequest,
    responses(
        (status = 200, description = "更新成功"),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到品類"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn update_sku_category(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathExtract(code): PathExtract<String>,
    Json(req): Json<UpdateSkuCategoryRequest>,
) -> Result<Json<crate::models::SkuCategory>> {
    require_permission!(current_user, "erp.product.edit");

    let category = SkuService::update_category(&state.db, &code, &req).await?;
    let after = serde_json::json!({
        "code": category.code,
        "name": category.name,
        "sort_order": category.sort_order,
        "is_active": category.is_active,
    });
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "SKU_CATEGORY_UPDATE",
        Some("sku_category"),
        None,
        Some(&format!("{} {}", category.code, category.name)),
        None,
        Some(after),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (SKU_CATEGORY_UPDATE): {}", e);
    }
    Ok(Json(category))
}

/// 新增子類
#[utoipa::path(
    post,
    path = "/api/sku/categories/{category_code}/subcategories",
    params(("category_code" = String, Path, description = "品類代碼")),
    request_body = CreateSkuSubcategoryRequest,
    responses(
        (status = 201, description = "建立成功"),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到品類"),
        (status = 409, description = "子類代碼已存在"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn create_sku_subcategory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(category_code): Path<String>,
    Json(req): Json<CreateSkuSubcategoryRequest>,
) -> Result<(StatusCode, Json<crate::models::SkuSubcategory>)> {
    require_permission!(current_user, "erp.product.edit");

    let subcategory = SkuService::create_subcategory(&state.db, &category_code, &req).await?;
    let after = serde_json::json!({
        "category_code": subcategory.category_code,
        "code": subcategory.code,
        "name": subcategory.name,
        "sort_order": subcategory.sort_order,
        "is_active": subcategory.is_active,
    });
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "SKU_SUBCATEGORY_CREATE",
        Some("sku_subcategory"),
        None,
        Some(&format!("{}:{} {}", subcategory.category_code, subcategory.code, subcategory.name)),
        None,
        Some(after),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (SKU_SUBCATEGORY_CREATE): {}", e);
    }
    Ok((StatusCode::CREATED, Json(subcategory)))
}

/// 更新子類（名稱、排序、啟用狀態）
#[utoipa::path(
    patch,
    path = "/api/sku/categories/{category_code}/subcategories/{code}",
    params(
        ("category_code" = String, Path, description = "品類代碼"),
        ("code" = String, Path, description = "子類代碼"),
    ),
    request_body = UpdateSkuSubcategoryRequest,
    responses(
        (status = 200, description = "更新成功"),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到子類"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn update_sku_subcategory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathExtract((category_code, code)): PathExtract<(String, String)>,
    Json(req): Json<UpdateSkuSubcategoryRequest>,
) -> Result<Json<crate::models::SkuSubcategory>> {
    require_permission!(current_user, "erp.product.edit");

    let subcategory =
        SkuService::update_subcategory(&state.db, &category_code, &code, &req).await?;
    let after = serde_json::json!({
        "category_code": subcategory.category_code,
        "code": subcategory.code,
        "name": subcategory.name,
        "sort_order": subcategory.sort_order,
        "is_active": subcategory.is_active,
    });
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "SKU_SUBCATEGORY_UPDATE",
        Some("sku_subcategory"),
        None,
        Some(&format!("{}:{} {}", subcategory.category_code, subcategory.code, subcategory.name)),
        None,
        Some(after),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (SKU_SUBCATEGORY_UPDATE): {}", e);
    }
    Ok(Json(subcategory))
}

/// 刪除子類（僅 admin；無產品使用時才可刪除）
#[utoipa::path(
    delete,
    path = "/api/sku/categories/{category_code}/subcategories/{code}",
    params(
        ("category_code" = String, Path, description = "品類代碼"),
        ("code" = String, Path, description = "子類代碼"),
    ),
    responses(
        (status = 204, description = "刪除成功"),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅管理員可刪除分類"),
        (status = 404, description = "找不到子類"),
        (status = 409, description = "尚有產品使用此子類"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn delete_sku_subcategory(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathExtract((category_code, code)): PathExtract<(String, String)>,
) -> Result<StatusCode> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可刪除分類".into()));
    }
    let sub = SkuService::get_subcategory_by_codes(&state.db, &category_code, &code).await?;
    SkuService::delete_subcategory(&state.db, &category_code, &code).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "SKU_SUBCATEGORY_DELETE",
        Some("sku_subcategory"),
        None,
        Some(&format!("{}:{} {}", sub.category_code, sub.code, sub.name)),
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (SKU_SUBCATEGORY_DELETE): {}", e);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// 刪除品類（僅 admin；無產品使用時才可刪除）
#[utoipa::path(
    delete,
    path = "/api/sku/categories/{code}",
    params(("code" = String, Path, description = "品類代碼")),
    responses(
        (status = 204, description = "刪除成功"),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅管理員可刪除分類"),
        (status = 404, description = "找不到品類"),
        (status = 409, description = "尚有產品使用此品類"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn delete_sku_category(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathExtract(code): PathExtract<String>,
) -> Result<StatusCode> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可刪除分類".into()));
    }
    let cat = SkuService::get_category_by_code(&state.db, &code).await?;
    SkuService::delete_category(&state.db, &code).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "SKU_CATEGORY_DELETE",
        Some("sku_category"),
        None,
        Some(&format!("{} {}", cat.code, cat.name)),
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (SKU_CATEGORY_DELETE): {}", e);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// 產生 SKU
#[utoipa::path(
    post,
    path = "/api/sku/generate",
    request_body = GenerateSkuRequest,
    responses(
        (status = 200, description = "產生成功", body = GenerateSkuResponse),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn generate_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<GenerateSkuRequest>,
) -> Result<Json<GenerateSkuResponse>> {
    require_permission!(current_user, "erp.product.create");

    let result = SkuService::generate(&state.db, &req).await?;
    Ok(Json(result))
}

/// 驗證 SKU
#[utoipa::path(
    post,
    path = "/api/sku/validate",
    request_body = ValidateSkuRequest,
    responses(
        (status = 200, description = "驗證結果", body = ValidateSkuResponse),
        (status = 401, description = "未認證"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn validate_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ValidateSkuRequest>,
) -> Result<Json<ValidateSkuResponse>> {
    require_permission!(current_user, "erp.product.view");

    let result = SkuService::validate(&state.db, &req).await?;
    Ok(Json(result))
}

/// 預覽 SKU
#[utoipa::path(
    post,
    path = "/api/skus/preview",
    request_body = SkuPreviewRequest,
    responses(
        (status = 200, description = "預覽結果", body = SkuPreviewResponse),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "SKU",
    security(("bearer" = []))
)]
pub async fn preview_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SkuPreviewRequest>,
) -> Result<Json<SkuPreviewResponse>> {
    require_permission!(current_user, "erp.product.view");

    let preview = SkuService::preview(&state.db, &req).await?;
    Ok(Json(preview))
}

/// 建立產品並自動產生 SKU
#[utoipa::path(
    post,
    path = "/api/products/with-sku",
    request_body = CreateProductWithSkuRequest,
    responses(
        (status = 200, description = "建立成功", body = ProductWithUom),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "產品管理",
    security(("bearer" = []))
)]
pub async fn create_product_with_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProductWithSkuRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.create");

    let product = SkuService::create_product_with_sku(&state.db, &req).await?;
    Ok(Json(product))
}
