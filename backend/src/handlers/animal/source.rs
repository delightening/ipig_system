// 動物來源管理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AnimalSource, CreateAnimalSourceRequest, UpdateAnimalSourceRequest},
    require_permission,
    services::AnimalSourceService,
    AppState, Result,
};

/// 列出所有動物來源
pub async fn list_animal_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalSource>>> {
    let sources = AnimalSourceService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 建立動物來源
pub async fn create_animal_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAnimalSourceRequest>,
) -> Result<Json<AnimalSource>> {
    require_permission!(current_user, "animal.source.manage");
    req.validate()?;

    let actor = ActorContext::User(current_user.clone());
    let source = AnimalSourceService::create_source(&state.db, &actor, &req).await?;
    Ok(Json(source))
}

/// 更新動物來源
pub async fn update_animal_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAnimalSourceRequest>,
) -> Result<Json<AnimalSource>> {
    require_permission!(current_user, "animal.source.manage");

    let actor = ActorContext::User(current_user.clone());
    let source = AnimalSourceService::update_source(&state.db, &actor, id, &req).await?;
    Ok(Json(source))
}

/// 刪除動物來源
pub async fn delete_animal_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.source.manage");

    let actor = ActorContext::User(current_user.clone());
    AnimalSourceService::delete_source(&state.db, &actor, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Animal source deleted successfully" }),
    ))
}
