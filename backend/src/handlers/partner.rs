use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{CreatePartnerRequest, GenerateCodeResponse, Partner, PartnerQuery, SupplierCategory, UpdatePartnerRequest},
    require_permission,
    services::PartnerService,
    AppError, AppState, Result,
};

/// 建立合作夥伴
pub async fn create_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePartnerRequest>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let partner = PartnerService::create(&state.db, &req).await?;
    Ok(Json(partner))
}

/// 列出所有合作夥伴
pub async fn list_partners(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PartnerQuery>,
) -> Result<Json<Vec<Partner>>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partners = PartnerService::list(&state.db, &query).await?;
    Ok(Json(partners))
}

/// 取得單個合作夥伴
pub async fn get_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partner = PartnerService::get_by_id(&state.db, id).await?;
    Ok(Json(partner))
}

/// 更新合作夥伴
pub async fn update_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePartnerRequest>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let partner = PartnerService::update(&state.db, id, &req).await?;
    Ok(Json(partner))
}

/// 刪除合作夥伴
pub async fn delete_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.partner.delete");
    
    PartnerService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Partner deleted successfully" })))
}

#[derive(Debug, Deserialize)]
pub struct GenerateCodeQuery {
    pub partner_type: Option<String>,
    pub category: Option<String>,
}

/// 生成供應商代碼
pub async fn generate_partner_code(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<GenerateCodeQuery>,
) -> Result<Json<GenerateCodeResponse>> {
    require_permission!(current_user, "erp.partner.create");
    
    let partner_type = match params.partner_type.as_deref() {
        Some("supplier") => crate::models::PartnerType::Supplier,
        Some("customer") => crate::models::PartnerType::Customer,
        None => crate::models::PartnerType::Supplier,
        _ => return Err(AppError::Validation("Invalid partner_type. Must be supplier or customer".to_string())),
    };
    
    let category = if partner_type == crate::models::PartnerType::Supplier {
        match params.category.as_deref() {
            Some("drug") => Some(SupplierCategory::Drug),
            Some("consumable") => Some(SupplierCategory::Consumable),
            Some("feed") => Some(SupplierCategory::Feed),
            Some("equipment") => Some(SupplierCategory::Equipment),
            _ => return Err(AppError::Validation("Invalid category for supplier. Must be one of: drug, consumable, feed, equipment".to_string())),
        }
    } else {
        None
    };
    
    let code = PartnerService::generate_code(&state.db, partner_type, category).await?;
    Ok(Json(GenerateCodeResponse { code }))
}
