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

/// 取得 SKU 類別列表
pub async fn get_sku_categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<CategoriesResponse>> {
    require_permission!(current_user, "product.read");

    let categories = SkuService::get_categories(&state.db).await?;
    Ok(Json(categories))
}

/// 取得 SKU 子類別列表
pub async fn get_sku_subcategories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(code): Path<String>,
) -> Result<Json<SubcategoriesResponse>> {
    require_permission!(current_user, "product.read");

    let subcategories = SkuService::get_subcategories(&state.db, &code).await?;
    Ok(Json(subcategories))
}

/// 生成 SKU
pub async fn generate_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<GenerateSkuRequest>,
) -> Result<Json<GenerateSkuResponse>> {
    require_permission!(current_user, "product.create");

    let result = SkuService::generate(&state.db, &req).await?;
    Ok(Json(result))
}

/// 驗證 SKU
pub async fn validate_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ValidateSkuRequest>,
) -> Result<Json<ValidateSkuResponse>> {
    require_permission!(current_user, "product.read");

    let result = SkuService::validate(&state.db, &req).await?;
    Ok(Json(result))
}

/// 預覽 SKU
pub async fn preview_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SkuPreviewRequest>,
) -> Result<Json<SkuPreviewResponse>> {
    require_permission!(current_user, "product.read");

    let preview = SkuService::preview(&state.db, &req).await?;
    Ok(Json(preview))
}

/// 建立產品（自動生成 SKU）
pub async fn create_product_with_sku(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProductWithSkuRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "product.create");

    let product = SkuService::create_product_with_sku(&state.db, &req).await?;
    Ok(Json(product))
}
