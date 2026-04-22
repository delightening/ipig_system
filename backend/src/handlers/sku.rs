use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

use axum::extract::Path as PathExtract;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        CategoriesResponse, CategoriesTreeResponse, CreateProductWithSkuRequest,
        CreateSkuSubcategoryRequest, GenerateSkuRequest, GenerateSkuResponse, ProductWithUom,
        SkuPreviewRequest, SkuPreviewResponse, SubcategoriesResponse, UpdateSkuCategoryRequest,
        UpdateSkuSubcategoryRequest, ValidateSkuRequest, ValidateSkuResponse,
    },
    require_permission,
    services::SkuService,
    AppError, AppState, Result,
};

/// 列出 SKU 分類清單
#[utoipa::path(
    get,
    path = "/api/v1/sku/categories",
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
    path = "/api/v1/sku/categories/{code}/subcategories",
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
    path = "/api/v1/sku/categories/tree",
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
    path = "/api/v1/sku/categories/{code}",
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

    let actor = ActorContext::User(current_user.clone());
    let category = SkuService::update_category(&state.db, &actor, &code, &req).await?;
    Ok(Json(category))
}

/// 新增子類
#[utoipa::path(
    post,
    path = "/api/v1/sku/categories/{category_code}/subcategories",
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

    let actor = ActorContext::User(current_user.clone());
    let subcategory = SkuService::create_subcategory(&state.db, &actor, &category_code, &req).await?;
    Ok((StatusCode::CREATED, Json(subcategory)))
}

/// 更新子類（名稱、排序、啟用狀態）
#[utoipa::path(
    patch,
    path = "/api/v1/sku/categories/{category_code}/subcategories/{code}",
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

    let actor = ActorContext::User(current_user.clone());
    let subcategory =
        SkuService::update_subcategory(&state.db, &actor, &category_code, &code, &req).await?;
    Ok(Json(subcategory))
}

/// 刪除子類（僅 admin；無產品使用時才可刪除）
#[utoipa::path(
    delete,
    path = "/api/v1/sku/categories/{category_code}/subcategories/{code}",
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
    let actor = ActorContext::User(current_user.clone());
    SkuService::delete_subcategory(&state.db, &actor, &category_code, &code).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 刪除品類（僅 admin；無產品使用時才可刪除）
#[utoipa::path(
    delete,
    path = "/api/v1/sku/categories/{code}",
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
    let actor = ActorContext::User(current_user.clone());
    SkuService::delete_category(&state.db, &actor, &code).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 產生 SKU
#[utoipa::path(
    post,
    path = "/api/v1/sku/generate",
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
    path = "/api/v1/sku/validate",
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
    path = "/api/v1/skus/preview",
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
    path = "/api/v1/products/with-sku",
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

    let actor = ActorContext::User(current_user.clone());
    let product = SkuService::create_product_with_sku(&state.db, &actor, &req).await?;
    Ok(Json(product))
}
