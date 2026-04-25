// 猝死登記 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AnimalSuddenDeath, CreateSuddenDeathRequest},
    require_permission,
    services::{access, AnimalMedicalService},
    AppState, Result,
};

/// 取得動物的猝死記錄
pub async fn get_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSuddenDeath>>> {
    // SEC-IDOR: v2 審計發現原始修復遺漏此端點
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let record = AnimalMedicalService::get_sudden_death(&state.db, animal_id).await?;
    Ok(Json(record))
}

/// 登記猝死
pub async fn create_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSuddenDeathRequest>,
) -> Result<Json<AnimalSuddenDeath>> {
    require_permission!(current_user, "animal.record.create");

    // Audit 已收進 service 層（SUDDEN_DEATH，tx 內 + animal 狀態變更同 tx）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalMedicalService::create_sudden_death(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(record))
}
