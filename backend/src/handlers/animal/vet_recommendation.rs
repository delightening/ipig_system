// 獸醫建議管理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateVetRecommendationRequest, CreateVetRecommendationWithAttachmentsRequest,
        VetRecommendation, VetRecordType,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppError, AppState, Result,
};

use super::{get_pig_info_from_observation, get_pig_info_from_surgery};

/// 為觀察記錄新增獸醫建議
pub async fn add_observation_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation(&state.db, VetRecordType::Observation, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_observation(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "觀察紀錄";
        if let Err(e) = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await {
            tracing::warn!("發送獸醫建議通知失敗: {e}");
        }

    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("觀察紀錄 #{} 獸醫建議", id)),
        None,
        Some(serde_json::json!({ "record_type": "observation", "record_id": id, "is_urgent": req.is_urgent })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}

/// 為手術記錄新增獸醫建議
pub async fn add_surgery_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation(&state.db, VetRecordType::Surgery, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_surgery(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "手術紀錄";
        if let Err(e) = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await {
            tracing::warn!("發送獸醫建議通知失敗: {e}");
        }

    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("手術紀錄 #{} 獸醫建議", id)),
        None,
        Some(serde_json::json!({ "record_type": "surgery", "record_id": id, "is_urgent": req.is_urgent })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}

pub async fn add_observation_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation_with_attachments(&state.db, VetRecordType::Observation, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_observation(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "觀察紀錄";
        if let Err(e) = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await {
            tracing::warn!("發送獸醫建議通知失敗: {e}");
        }

    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("觀察紀錄 #{} 獸醫建議 (含附件)", id)),
        None,
        Some(serde_json::json!({ "record_type": "observation", "record_id": id, "is_urgent": req.is_urgent, "has_attachments": true })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}

pub async fn add_surgery_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation_with_attachments(&state.db, VetRecordType::Surgery, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_surgery(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "手術紀錄";
        if let Err(e) = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await {
            tracing::warn!("發送獸醫建議通知失敗: {e}");
        }

    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("手術紀錄 #{} 獸醫建議 (含附件)", id)),
        None,
        Some(serde_json::json!({ "record_type": "surgery", "record_id": id, "is_urgent": req.is_urgent, "has_attachments": true })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}

pub async fn get_observation_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = AnimalService::get_vet_recommendations(&state.db, VetRecordType::Observation, id).await?;
    Ok(Json(recommendations))
}

/// 取得手術記錄的所有獸醫建議
pub async fn get_surgery_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = AnimalService::get_vet_recommendations(&state.db, VetRecordType::Surgery, id).await?;
    Ok(Json(recommendations))
}
