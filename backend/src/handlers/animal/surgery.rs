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
        CopyRecordRequest, CreateSurgeryRequest, DeleteRequest, AnimalSurgery, SurgeryListItem,
        UpdateSurgeryRequest, VersionHistoryResponse, RecordFilterQuery,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppError, AppState, Result,
};

/// 列出動物的所有手術記錄
pub async fn list_animal_surgeries(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<AnimalSurgery>>> {
    let surgeries = AnimalService::list_surgeries(&state.db, animal_id, filter.after).await?;
    Ok(Json(surgeries))
}
/// 列出動物的手術記錄（包含獸醫建議）
pub async fn list_animal_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    let surgeries = AnimalService::list_surgeries_with_recommendations(&state.db, animal_id, filter.after).await?;
    Ok(Json(surgeries))
}

/// 取得單個手術記錄
pub async fn get_animal_surgery(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalSurgery>> {
    let surgery = AnimalService::get_surgery_by_id(&state.db, id).await?;
    Ok(Json(surgery))
}

/// 建立手術記錄
pub async fn create_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSurgeryRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.create");
    req.validate()?;
    
    let surgery = AnimalService::create_surgery(&state.db, animal_id, &req, current_user.id).await?;

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
        &state.db, current_user.id, "ANIMAL", "SURGERY_CREATE",
        Some("animal_surgery"), Some(animal_id),
        Some(&surg_display),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_CREATE): {}", e);
    }

    Ok(Json(surgery))
}

/// 更新手術記錄
pub async fn update_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.edit");
    
    let surgery = AnimalService::update_surgery(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SURGERY_UPDATE",
        Some("animal_surgery"), None,
        Some(&format!("手術紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_UPDATE): {}", e);
    }

    Ok(Json(surgery))
}

/// 刪除手術記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;
    
    AnimalService::soft_delete_surgery_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SURGERY_DELETE",
        Some("animal_surgery"), None,
        Some(&format!("手術紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Surgery deleted successfully" })))
}

/// 複製手術記錄
pub async fn copy_animal_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<AnimalSurgery>> {
    require_permission!(current_user, "animal.record.copy");
    
    let surgery = AnimalService::copy_surgery(&state.db, animal_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術記錄為獸醫已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_surgery_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得手術記錄的版本歷史
pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = AnimalService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}
