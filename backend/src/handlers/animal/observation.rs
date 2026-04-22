// 觀察記錄管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalObservation, CopyRecordRequest, CreateObservationRequest, DeleteRequest,
        ObservationListItem, RecordFilterQuery, UpdateObservationRequest, VersionHistoryResponse,
    },
    require_permission,
    services::{access, AnimalObservationService, AnimalService, AuditService},
    AppState, Result,
};

/// 列出動物的所有觀察記錄
#[utoipa::path(
    get,
    path = "/api/v1/animals/{animal_id}/observations",
    params(("animal_id" = Uuid, Path, description = "動物 ID"), RecordFilterQuery),
    responses((status = 200, description = "觀察記錄清單", body = Vec<AnimalObservation>), (status = 401, description = "未認證")),
    tag = "動物子模組",
    security(("bearer" = []))
)]
pub async fn list_animal_observations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<AnimalObservation>>> {
    // C2: 驗證使用者對此動物所屬計畫的存取權限，防止 IDOR
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let observations = AnimalObservationService::list(&state.db, animal_id, filter.after).await?;
    Ok(Json(observations))
}

/// 列出動物的觀察記錄（包含獸醫建議）
pub async fn list_animal_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<ObservationListItem>>> {
    // C2: 驗證使用者對此動物所屬計畫的存取權限，防止 IDOR
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let observations =
        AnimalObservationService::list_with_recommendations(&state.db, animal_id, filter.after)
            .await?;
    Ok(Json(observations))
}

/// 取得單個觀察記錄
#[utoipa::path(
    get,
    path = "/api/v1/observations/{id}",
    params(("id" = Uuid, Path, description = "觀察記錄 ID")),
    responses((status = 200, description = "觀察記錄", body = AnimalObservation), (status = 401, description = "未認證"), (status = 404, description = "找不到")),
    tag = "動物子模組",
    security(("bearer" = []))
)]
pub async fn get_animal_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalObservation>> {
    // C2: 透過觀察記錄找到 animal，再驗證計畫存取權限，防止 IDOR
    let animal_id = access::get_observation_animal_id(&state.db, id).await?;
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let observation = AnimalObservationService::get_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 建立觀察記錄
#[utoipa::path(
    post,
    path = "/api/v1/animals/{animal_id}/observations",
    params(("animal_id" = Uuid, Path, description = "動物 ID")),
    request_body = CreateObservationRequest,
    responses((status = 200, description = "建立成功", body = AnimalObservation), (status = 400, description = "驗證失敗"), (status = 401, description = "未認證")),
    tag = "動物子模組",
    security(("bearer" = []))
)]
pub async fn create_animal_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateObservationRequest>,
) -> Result<Json<AnimalObservation>> {
    require_permission!(current_user, "animal.record.create");
    req.validate()?;

    // 檢查是否為緊急給藥，需驗證是否有緊急給藥權限
    if req.is_emergency {
        require_permission!(current_user, "animal.record.emergency");
    }

    // Audit 已收進 service 層（OBSERVATION_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let observation =
        AnimalObservationService::create(&state.db, &actor, animal_id, &req).await?;

    if req.is_emergency {
        // 取得動物資訊
        if let Ok(animal) = AnimalService::get_by_id(&state.db, animal_id).await {
            let notification_service = crate::services::NotificationService::new(state.db.clone());
            let emergency_reason = req.emergency_reason.as_deref().unwrap_or("未提供原因");

            // 異步發送通知，不阻塞主流程
            if let Err(e) = notification_service
                .notify_emergency_medication(
                    animal_id,
                    observation.id,
                    &animal.ear_tag,
                    animal.iacuc_no.as_deref(),
                    &current_user.email,
                    emergency_reason,
                )
                .await
            {
                tracing::warn!("發送緊急給藥通知失敗: {e}");
            }

            tracing::warn!(
                "[Emergency Medication] User {} recorded emergency medication for animal {} (observation {})",
                current_user.email,
                animal.ear_tag,
                observation.id
            );
        }
    }

    // 異常紀錄通知 → 通知 VET（依路由表 animal_abnormal_record）
    if matches!(req.record_type, crate::models::RecordType::Abnormal) {
        if let Ok(animal) = AnimalService::get_by_id(&state.db, animal_id).await {
            let summary: String = if req.content.is_empty() {
                "（未提供摘要）".to_string()
            } else {
                req.content.chars().take(100).collect()
            };
            let operator = current_user.email.clone();
            let ear_tag = animal.ear_tag.clone();
            let iacuc_no = animal.iacuc_no.clone();
            let a_id = animal_id;
            let db = state.db.clone();
            tokio::spawn(async move {
                let svc = crate::services::NotificationService::new(db);
                if let Err(e) = svc
                    .notify_abnormal_record(
                        a_id,
                        &ear_tag,
                        iacuc_no.as_deref(),
                        &summary,
                        &operator,
                    )
                    .await
                {
                    tracing::warn!("發送異常紀錄通知失敗: {e}");
                }
            });
        }
    }

    Ok(Json(observation))
}

/// 更新觀察記錄
#[utoipa::path(
    put,
    path = "/api/v1/observations/{id}",
    params(("id" = Uuid, Path, description = "觀察記錄 ID")),
    request_body = UpdateObservationRequest,
    responses((status = 200, description = "更新成功", body = AnimalObservation), (status = 400, description = "驗證失敗"), (status = 401, description = "未認證"), (status = 404, description = "找不到")),
    tag = "動物子模組",
    security(("bearer" = []))
)]
pub async fn update_animal_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<AnimalObservation>> {
    require_permission!(current_user, "animal.record.edit");

    // Audit 已收進 service 層（OBSERVATION_UPDATE with before/after diff，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let observation =
        AnimalObservationService::update(&state.db, &actor, id, &req).await?;

    Ok(Json(observation))
}

/// 刪除觀察記錄（軟刪除 + 刪除原因）- GLP 合規
#[utoipa::path(
    delete,
    path = "/api/v1/observations/{id}",
    params(("id" = Uuid, Path, description = "觀察記錄 ID")),
    responses((status = 200, description = "刪除成功"), (status = 401, description = "未認證"), (status = 404, description = "找不到")),
    tag = "動物子模組",
    security(("bearer" = []))
)]
pub async fn delete_animal_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    // Audit 已收進 service 層（OBSERVATION_SOFT_DELETE with change_reasons，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalObservationService::soft_delete_with_reason(&state.db, &actor, id, &req.reason)
        .await?;

    if let Err(e) =
        crate::services::FileService::delete_by_entity(&state.db, "observation", &id).await
    {
        tracing::warn!("清理觀察紀錄附件失敗 (non-fatal): {}", e);
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "OBSERVATION_DELETE",
        Some("animal_observation"),
        None,
        Some(&format!("觀察紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_DELETE): {}", e);
    }

    Ok(Json(
        serde_json::json!({ "message": "Observation deleted successfully" }),
    ))
}

/// 複製觀察記錄
pub async fn copy_animal_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<AnimalObservation>> {
    require_permission!(current_user, "animal.record.copy");

    let observation =
        AnimalObservationService::copy(&state.db, animal_id, req.source_id, current_user.id)
            .await?;
    Ok(Json(observation))
}

/// 標記觀察記錄為獸醫已讀
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");

    AnimalObservationService::mark_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得觀察記錄的版本歷史
pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    // SEC-IDOR: v2 審計發現 — 版本歷程需驗證動物計畫歸屬
    let animal_id = access::get_observation_animal_id(&state.db, id).await?;
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let versions = AnimalService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}
