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
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
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
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let surgeries =
        AnimalSurgeryService::list_with_recommendations(&state.db, scope, filter.after).await?;
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
    access::require_animal_read_access(&state.db, &current_user, surgery.animal_id).await?;
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
    // R75-2: 計畫綁定紀錄寫入限自己計畫（對齊 update/delete 的 require_animal_access；
    // 原 create 僅 service require_animal_has_protocol 前置檢查，缺擁有權守衛 → 跨計畫越權寫入）
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;
    req.validate()?;

    // Audit 已收進 service 層（SURGERY_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let surgery = AnimalSurgeryService::create(&state.db, &actor, scope, &req).await?;
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
    // SEC-IDOR: 由紀錄 id 反查 animal_id，驗證計畫歸屬
    let animal_id = access::get_surgery_animal_id(&state.db, id).await?;
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    // Audit 已收進 service 層（SURGERY_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let surgery = AnimalSurgeryService::update(&state.db, &actor, scope, id, &req).await?;
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
    // SEC-IDOR: 由紀錄 id 反查 animal_id，驗證計畫歸屬
    let animal_id = access::get_surgery_animal_id(&state.db, id).await?;
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    // Audit 已收進 service 層（SURGERY_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalSurgeryService::soft_delete_with_reason(&state.db, &actor, scope, id, &req.reason)
        .await?;

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
    // R75 defense-in-depth：來源手術紀錄也須驗證存取權，否則具 copy 權但無 view_all 的
    // （未來自訂）角色可指定他人計畫的 source_id，把跨計畫內容複製進自己可讀的新紀錄
    // （cross-protocol read IDOR）。來源用「讀取權」→ 具 animal.animal.view_all 的全場
    // 人員仍可跨計畫複製（保住全場 copy 訴求）；目標仍須寫入權。
    let source_animal_id = access::get_surgery_animal_id(&state.db, req.source_id).await?;
    access::require_animal_read_access(&state.db, &current_user, source_animal_id).await?;
    // SEC-IDOR: 驗證目標 animal 的計畫歸屬
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    let surgery =
        AnimalSurgeryService::copy(&state.db, scope, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術記錄為獸醫已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    // SEC-IDOR: 由紀錄 id 反查 animal_id，驗證計畫歸屬
    let animal_id = access::get_surgery_animal_id(&state.db, id).await?;
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    AnimalSurgeryService::mark_vet_read(&state.db, scope, id, current_user.id).await?;
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
    access::require_animal_read_access(&state.db, &current_user, surgery.animal_id).await?;
    let versions = AnimalService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}
