use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    models::{
        CategoriesResponse, CreateProductWithSkuRequest, GenerateSkuRequest, GenerateSkuResponse,
        ProductWithUom, SkuPreviewRequest, SkuPreviewResponse, SubcategoriesResponse,
        ValidateSkuRequest, ValidateSkuResponse,
    },
    require_permission,
    services::SkuService,
    AppState, Result,
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
