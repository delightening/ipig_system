// 豬源管理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{CreatePigSourceRequest, PigSource, UpdatePigSourceRequest},
    require_permission,
    services::AnimalService,
    AppError, AppState, Result,
};

/// 列出所有豬源
pub async fn list_pig_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigSource>>> {
    let sources = AnimalService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 建立豬源
pub async fn create_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "animal.animal.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let source = AnimalService::create_source(&state.db, &req).await?;
    Ok(Json(source))
}

/// 更新豬源
pub async fn update_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "animal.animal.edit");
    
    let source = AnimalService::update_source(&state.db, id, &req).await?;
    Ok(Json(source))
}

/// 刪除豬源
pub async fn delete_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.animal.delete");
    
    AnimalService::delete_source(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig source deleted successfully" })))
}
