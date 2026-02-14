// 觀察記錄管理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CopyRecordRequest, CreateObservationRequest, DeleteRequest, ObservationListItem,
        PigObservation, UpdateObservationRequest, VersionHistoryResponse,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppError, AppState, Result,
};

/// 列出豬的所有觀察記錄
pub async fn list_pig_observations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<PigObservation>>> {
    let observations = AnimalService::list_observations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 列出豬的觀察記錄（包含獸醫建議）
pub async fn list_pig_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<ObservationListItem>>> {
    let observations = AnimalService::list_observations_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 取得單個觀察記錄
pub async fn get_pig_observation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PigObservation>> {
    let observation = AnimalService::get_observation_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 建立觀察記錄
pub async fn create_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 檢查是否為緊急給藥，需驗證是否有緊急給藥權限
    if req.is_emergency {
        require_permission!(current_user, "animal.record.emergency");
    }
    
    let observation = AnimalService::create_observation(&state.db, pig_id, &req, current_user.id).await?;
    
    // 如果是緊急給藥，發送通知給 VET 和 PI
    if req.is_emergency {
        // 取得豬隻資訊
        if let Ok(pig) = AnimalService::get_by_id(&state.db, pig_id).await {
            let notification_service = crate::services::NotificationService::new(state.db.clone());
            let emergency_reason = req.emergency_reason.as_deref().unwrap_or("未提供原因");
            
            // 異步發送通知，不阻塞主流程
            if let Err(e) = notification_service.notify_emergency_medication(
                pig_id,
                observation.id,
                &pig.ear_tag,
                pig.iacuc_no.as_deref(),
                &current_user.email,
                emergency_reason,
            ).await {
                tracing::warn!("發送緊急給藥通知失敗: {e}");
            }

            
            tracing::warn!(
                "[Emergency Medication] User {} recorded emergency medication for pig {} (observation {})",
                current_user.email,
                pig.ear_tag,
                observation.id
            );
        }
    }

    // 取得豬隻資訊用於日誌顯示
    let obs_display = match AnimalService::get_by_id(&state.db, pig_id).await {
        Ok(pig) => {
            let iacuc = pig.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {}", iacuc, pig.ear_tag)
        }
        _ => format!("觀察紀錄 #{} (pig: {})", observation.id, pig_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_CREATE",
        Some("pig_observation"), Some(pig_id),
        Some(&obs_display),
        None,
        Some(serde_json::json!({
            "observation_id": observation.id,
            "is_emergency": req.is_emergency,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_CREATE): {}", e);
    }

    Ok(Json(observation))
}

/// 更新觀察記錄
pub async fn update_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.edit");
    
    let observation = AnimalService::update_observation(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_UPDATE",
        Some("pig_observation"), None,
        Some(&format!("觀察紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_UPDATE): {}", e);
    }

    Ok(Json(observation))
}

/// 刪除觀察記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::soft_delete_observation_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_DELETE",
        Some("pig_observation"), None,
        Some(&format!("觀察紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Observation deleted successfully" })))
}

/// 複製觀察記錄
pub async fn copy_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.copy");
    
    let observation = AnimalService::copy_observation(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(observation))
}

/// 標記觀察記錄為獸醫已讀
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_observation_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得觀察記錄的版本歷史
pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = AnimalService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}
