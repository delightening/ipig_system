use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory, ProductQuery,
        ProductWithUom, UpdateProductRequest,
    },
    require_permission,
    services::ProductService,
    AppError, AppState, Result,
};

/// 撱箇??Ｗ?
pub async fn create_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let product = ProductService::create(&state.db, &req).await?;
    Ok(Json(product))
}

/// ???Ｗ??”
pub async fn list_products(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>> {
    require_permission!(current_user, "erp.product.view");
    
    let products = ProductService::list(&state.db, &query).await?;
    Ok(Json(products))
}

/// ???桐??Ｗ?
pub async fn get_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.view");
    
    let product = ProductService::get_by_id(&state.db, id).await?;
    Ok(Json(product))
}

/// ?湔?Ｗ?
pub async fn update_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductWithUom>> {
    require_permission!(current_user, "erp.product.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let product = ProductService::update(&state.db, id, &req).await?;
    Ok(Json(product))
}

/// ?芷?Ｗ?
pub async fn delete_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.product.delete");
    
    ProductService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Product deleted successfully" })))
}

/// ???Ｗ?憿?”
pub async fn list_categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProductCategory>>> {
    require_permission!(current_user, "erp.product.view");
    
    let categories = ProductService::list_categories(&state.db).await?;
    Ok(Json(categories))
}

/// 撱箇??Ｗ?憿
pub async fn create_category(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<ProductCategory>> {
    require_permission!(current_user, "erp.product.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let category = ProductService::create_category(&state.db, &req).await?;
    Ok(Json(category))
}

