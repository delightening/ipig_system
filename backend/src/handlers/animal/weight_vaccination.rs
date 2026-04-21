// 體重 + 疫苗記錄管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalVaccination, AnimalWeight, AnimalWeightResponse, CreateVaccinationRequest,
        CreateWeightRequest, DeleteRequest, RecordFilterQuery, UpdateVaccinationRequest,
        UpdateWeightRequest,
    },
    require_permission,
    services::{access, AnimalMedicalService, AnimalService, AnimalWeightService, AuditService},
    AppState, Result,
};

// ============================================
// 體重記錄管理
// ============================================

/// 列出動物的所有體重記錄
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/weights", params(("animal_id" = Uuid, Path, description = "動物 ID"), RecordFilterQuery), responses((status = 200, body = Vec<AnimalWeightResponse>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn list_animal_weights(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<AnimalWeightResponse>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let weights = AnimalWeightService::list(&state.db, animal_id, filter.after).await?;
    Ok(Json(weights))
}

/// 建立體重記錄
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/weights", params(("animal_id" = Uuid, Path, description = "動物 ID")), request_body = CreateWeightRequest, responses((status = 200, body = AnimalWeight), (status = 400), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn create_animal_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateWeightRequest>,
) -> Result<Json<AnimalWeight>> {
    require_permission!(current_user, "animal.record.create");

    let actor = ActorContext::User(current_user.clone());
    let weight = AnimalWeightService::create(&state.db, &actor, animal_id, &req).await?;

    Ok(Json(weight))
}

/// 更新體重記錄
#[utoipa::path(put, path = "/api/v1/weights/{id}", params(("id" = Uuid, Path, description = "記錄 ID")), request_body = UpdateWeightRequest, responses((status = 200, body = AnimalWeight), (status = 400), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn update_animal_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWeightRequest>,
) -> Result<Json<AnimalWeight>> {
    require_permission!(current_user, "animal.record.edit");

    let actor = ActorContext::User(current_user.clone());
    let weight = AnimalWeightService::update(&state.db, &actor, id, &req).await?;

    Ok(Json(weight))
}

/// 刪除體重記錄（軟刪除 + 刪除原因）- GLP 合規
#[utoipa::path(delete, path = "/api/v1/weights/{id}", params(("id" = Uuid, Path, description = "記錄 ID")), request_body = DeleteRequest, responses((status = 200), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn delete_animal_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    let actor = ActorContext::User(current_user.clone());
    AnimalWeightService::soft_delete_with_reason(&state.db, &actor, id, &req.reason).await?;

    Ok(Json(
        serde_json::json!({ "message": "Weight record deleted successfully" }),
    ))
}

// ============================================
// 疫苗接種記錄管理
// ============================================

/// 列出動物的所有疫苗接種記錄
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/vaccinations", params(("animal_id" = Uuid, Path, description = "動物 ID"), RecordFilterQuery), responses((status = 200, body = Vec<AnimalVaccination>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn list_animal_vaccinations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<AnimalVaccination>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let vaccinations =
        AnimalMedicalService::list_vaccinations(&state.db, animal_id, filter.after).await?;
    Ok(Json(vaccinations))
}

/// 建立疫苗接種記錄
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/vaccinations", params(("animal_id" = Uuid, Path, description = "動物 ID")), request_body = CreateVaccinationRequest, responses((status = 200, body = AnimalVaccination), (status = 400), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn create_animal_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateVaccinationRequest>,
) -> Result<Json<AnimalVaccination>> {
    require_permission!(current_user, "animal.record.create");
    req.validate()?;

    let actor = ActorContext::User(current_user.clone());
    let vaccination =
        AnimalMedicalService::create_vaccination(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(vaccination))
}

/// 更新疫苗接種記錄
#[utoipa::path(put, path = "/api/v1/vaccinations/{id}", params(("id" = Uuid, Path, description = "記錄 ID")), request_body = UpdateVaccinationRequest, responses((status = 200, body = AnimalVaccination), (status = 400), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn update_animal_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVaccinationRequest>,
) -> Result<Json<AnimalVaccination>> {
    require_permission!(current_user, "animal.record.edit");

    let actor = ActorContext::User(current_user.clone());
    let vaccination = AnimalMedicalService::update_vaccination(&state.db, &actor, id, &req).await?;
    Ok(Json(vaccination))
}

/// 刪除疫苗接種記錄（軟刪除 + 刪除原因）- GLP 合規
#[utoipa::path(delete, path = "/api/v1/vaccinations/{id}", params(("id" = Uuid, Path, description = "記錄 ID")), request_body = DeleteRequest, responses((status = 200), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn delete_animal_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    let actor = ActorContext::User(current_user.clone());
    AnimalMedicalService::soft_delete_vaccination_with_reason(&state.db, &actor, id, &req.reason)
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "Vaccination record deleted successfully" }),
    ))
}
