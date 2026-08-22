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
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalSource>>> {
    // F1（pentest 2026-07-05）：來源清單含供應商 address/contact/phone（聯絡 PII），
    // 修補前無 gate → 外部 CLIENT 等任何已認證者可讀供應商 PII。
    // gate 用 `animal.animal.view_all`（非 `animal.source.manage`）：來源清單是
    // 動物建檔/編輯表單（AnimalEditPage / AnimalsPage）的下拉參考資料，主要建檔角色
    // EXPERIMENT_STAFF 有 view_all 但無 source.manage（管理權僅 admin），用 manage 會
    // 誤殺其下拉；view_all 涵蓋所有內部動物營運角色、擋外部 CLIENT（僅 view_project）。
    require_permission!(current_user, "animal.animal.view_all");
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
