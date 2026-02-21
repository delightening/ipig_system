// 照護紀錄（疼痛評估）Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    services::{
        CareRecord, CareRecordService, CreateCareRecordRequest, UpdateCareRecordRequest,
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

/// DELETE /care-records/:id — 刪除照護紀錄
pub async fn delete_care_record(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    CareRecordService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Care record deleted" })))
}
