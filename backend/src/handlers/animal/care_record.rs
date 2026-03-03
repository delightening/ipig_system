// 照護紀錄（疼痛評估）Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::DeleteRequest,
    require_permission,
    services::{
        AuditService, CareRecord, CareRecordService, CreateCareRecordRequest, UpdateCareRecordRequest,
    },
    AppState, Result,
};

/// GET /animals/:id/care-records — 列出動物的照護紀錄
pub async fn list_care_records(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<CareRecord>>> {
    let records = CareRecordService::list_by_animal(&state.db, animal_id).await?;
    Ok(Json(records))
}

/// POST /animals/:id/care-records — 建立照護紀錄
pub async fn create_care_record(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(_animal_id): Path<Uuid>,
    Json(req): Json<CreateCareRecordRequest>,
) -> Result<Json<CareRecord>> {
    let record = CareRecordService::create(&state.db, &req).await?;
    Ok(Json(record))
}

/// PUT /care-records/:id — 更新照護紀錄
pub async fn update_care_record(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCareRecordRequest>,
) -> Result<Json<CareRecord>> {
    let record = CareRecordService::update(&state.db, id, &req).await?;
    Ok(Json(record))
}

/// DELETE /care-records/:id — 刪除照護紀錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_care_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    CareRecordService::soft_delete_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "CARE_RECORD_DELETE",
        Some("care_medication_record"),
        None,
        Some(&format!("照護紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (CARE_RECORD_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Care record deleted successfully" })))
}
