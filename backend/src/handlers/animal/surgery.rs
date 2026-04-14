// 手術記錄管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AnimalSurgery, CopyRecordRequest, CreateSurgeryRequest, DeleteRequest, RecordFilterQuery,
        SurgeryListItem, UpdateSurgeryRequest, VersionHistoryResponse,
    },
    require_permission,
    services::{access, AnimalService, AnimalSurgeryService, AuditService},
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

    let surgery = AnimalSurgeryService::create(&state.db, animal_id, &req, current_user.id).await?;

    // 取得動物資訊用於日誌顯示
    let surg_display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - {}", iacuc, animal.ear_tag, req.surgery_site)
        }
        _ => format!("手術紀錄 #{} (animal: {})", surgery.id, animal_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SURGERY_CREATE",
        Some("animal_surgery"),
        Some(animal_id),
        Some(&surg_display),
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_CREATE): {}", e);
    }

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

    let surgery = AnimalSurgeryService::update(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SURGERY_UPDATE",
        Some("animal_surgery"),
        None,
        Some(&format!("手術紀錄 #{}", id)),
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_UPDATE): {}", e);
    }

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

    AnimalSurgeryService::soft_delete_with_reason(&state.db, id, &req.reason, current_user.id)
        .await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SURGERY_DELETE",
        Some("animal_surgery"),
        None,
        Some(&format!("手術紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_DELETE): {}", e);
    }

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
