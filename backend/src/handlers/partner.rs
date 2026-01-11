use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{CreatePartnerRequest, Partner, PartnerQuery, UpdatePartnerRequest},
    require_permission,
    services::PartnerService,
    AppError, AppState, Result,
};

/// 撱箇?憭乩撈
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

/// ??憭乩撈?”
pub async fn list_partners(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PartnerQuery>,
) -> Result<Json<Vec<Partner>>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partners = PartnerService::list(&state.db, &query).await?;
    Ok(Json(partners))
}

/// ???桐?憭乩撈
pub async fn get_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partner = PartnerService::get_by_id(&state.db, id).await?;
    Ok(Json(partner))
}

/// ?湔憭乩撈
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

/// ?芷憭乩撈
pub async fn delete_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.partner.delete");
    
    PartnerService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Partner deleted successfully" })))
}

