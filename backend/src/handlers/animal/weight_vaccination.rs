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
    services::{access, AnimalMedicalService, AnimalWeightService},
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
    // R70-3: 體重為免計畫紀錄，允許未指派計畫的動物存取
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
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
    // R75-2: 對齊 update/delete 的免計畫紀錄 IDOR 防護（原 create 缺擁有權守衛）
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（WEIGHT_CREATE，tx 內）
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
    // R70-3: 體重為免計畫紀錄（record_id → animal_id 仍保留 IDOR 防護）
    let animal_id = access::get_weight_animal_id(&state.db, id).await?;
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（WEIGHT_UPDATE with before/after diff，tx 內）
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
    // R70-3: 體重為免計畫紀錄（record_id → animal_id 仍保留 IDOR 防護）
    let animal_id = access::get_weight_animal_id(&state.db, id).await?;
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（WEIGHT_SOFT_DELETE with change_reasons，tx 內）
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
    // R70-3: 疫苗/驅蟲為免計畫紀錄，允許未指派計畫的動物存取
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
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
    Json(mut req): Json<CreateVaccinationRequest>,
) -> Result<Json<AnimalVaccination>> {
    require_permission!(current_user, "animal.record.create");
    // R75-2: 對齊 update/delete 的免計畫紀錄 IDOR 防護（原 create 缺擁有權守衛）
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
    req.validate()?;
    // 驗證只檢查 trim 後長度，實際寫入前把前後空白清掉，避免用空白填充繞過長度上限存進 DB
    req.vaccine = req.vaccine.map(|s| s.trim().to_string());
    req.deworming_dose = req.deworming_dose.map(|s| s.trim().to_string());

    // Audit 已收進 service 層（VACCINATION_CREATE，tx 內）
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
    Json(mut req): Json<UpdateVaccinationRequest>,
) -> Result<Json<AnimalVaccination>> {
    require_permission!(current_user, "animal.record.edit");
    req.validate()?;
    // 驗證只檢查 trim 後長度，實際寫入前把前後空白清掉，避免用空白填充繞過長度上限存進 DB
    req.vaccine = req.vaccine.map(|s| s.trim().to_string());
    req.deworming_dose = req.deworming_dose.map(|s| s.trim().to_string());
    // R70-3: 疫苗/驅蟲為免計畫紀錄（record_id → animal_id 仍保留 IDOR 防護）
    let animal_id = access::get_vaccination_animal_id(&state.db, id).await?;
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（VACCINATION_UPDATE with before/after diff，tx 內）
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
    // R70-3: 疫苗/驅蟲為免計畫紀錄（record_id → animal_id 仍保留 IDOR 防護）
    let animal_id = access::get_vaccination_animal_id(&state.db, id).await?;
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（VACCINATION_DELETE with change_reasons，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalMedicalService::soft_delete_vaccination_with_reason(&state.db, &actor, id, &req.reason)
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "Vaccination record deleted successfully" }),
    ))
}
