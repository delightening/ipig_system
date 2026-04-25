// 手術記錄管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalSurgery, CopyRecordRequest, CreateSurgeryRequest, DeleteRequest, RecordFilterQuery,
        SurgeryListItem, UpdateSurgeryRequest, VersionHistoryResponse,
    },
    require_permission,
    services::{access, AnimalService, AnimalSurgeryService},
    AppState, Result,
};

/// 列出動物的所有手術記錄
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/surgeries", params(("animal_id" = Uuid, Path, description = "動物 ID"), RecordFilterQuery), responses((status = 200, body = Vec<AnimalSurgery>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn list_animal_surgeries(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<AnimalSurgery>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let surgeries = AnimalSurgeryService::list(&state.db, animal_id, filter.after).await?;
    Ok(Json(surgeries))
}
/// 列出動物的手術記錄（包含獸醫建議）
pub async fn list_animal_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let surgeries =
        AnimalSurgeryService::list_with_recommendations(&state.db, animal_id, filter.after).await?;
    Ok(Json(surgeries))
}

/// 取得單個手術記錄
#[utoipa::path(get, path = "/api/v1/surgeries/{id}", params(("id" = Uuid, Path, description = "手術記錄 ID")), responses((status = 200, body = AnimalSurgery), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn get_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalSurgery>> {
    let surgery = AnimalSurgeryService::get_by_id(&state.db, id).await?;
    // SEC-IDOR: 透過手術記錄所屬動物驗證計畫存取權限
    access::require_animal_access(&state.db, &current_user, surgery.animal_id).await?;
    Ok(Json(surgery))
}

/// 建立手術記錄
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/surgeries", params(("animal_id" = Uuid, Path, description = "動物 ID")), request_body = CreateSurgeryRequest, responses((status = 200, body = AnimalSurgery), (status = 400), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn create_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSurgeryRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.create");
    req.validate()?;

    // Audit 已收進 service 層（SURGERY_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let surgery = AnimalSurgeryService::create(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(surgery))
}

/// 更新手術記錄
#[utoipa::path(put, path = "/api/v1/surgeries/{id}", params(("id" = Uuid, Path, description = "手術記錄 ID")), request_body = UpdateSurgeryRequest, responses((status = 200, body = AnimalSurgery), (status = 400), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn update_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.edit");

    // Audit 已收進 service 層（SURGERY_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let surgery = AnimalSurgeryService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(surgery))
}

/// 刪除手術記錄（軟刪除 + 刪除原因）- GLP 合規
#[utoipa::path(delete, path = "/api/v1/surgeries/{id}", params(("id" = Uuid, Path, description = "手術記錄 ID")), responses((status = 200), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn delete_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    // Audit 已收進 service 層（SURGERY_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalSurgeryService::soft_delete_with_reason(&state.db, &actor, id, &req.reason).await?;

    Ok(Json(
        serde_json::json!({ "message": "Surgery deleted successfully" }),
    ))
}

/// 複製手術記錄
pub async fn copy_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.copy");

    let surgery =
        AnimalSurgeryService::copy(&state.db, animal_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術記錄為獸醫已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");

    AnimalSurgeryService::mark_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得手術記錄的版本歷史
pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    // SEC-IDOR: v2 審計發現 — 版本歷程需驗證動物計畫歸屬
    let surgery = AnimalSurgeryService::get_by_id(&state.db, id).await?;
    access::require_animal_access(&state.db, &current_user, surgery.animal_id).await?;
    let versions = AnimalService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}
