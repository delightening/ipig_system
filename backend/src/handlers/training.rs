// 人員訓練紀錄 Handlers (GLP 合規)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateTrainingRecordRequest, PaginatedResponse, TrainingQuery,
        TrainingRecord, TrainingRecordWithUser, UpdateTrainingRecordRequest,
    },
    services::TrainingService,
    AppState, Result,
};

/// 列出訓練紀錄
pub async fn list_training_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TrainingQuery>,
) -> Result<Json<PaginatedResponse<TrainingRecordWithUser>>> {
    let result = TrainingService::list(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

/// 取得單筆訓練紀錄
pub async fn get_training_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<TrainingRecord>> {
    let record = TrainingService::get(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

/// 新增訓練紀錄
pub async fn create_training_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateTrainingRecordRequest>,
) -> Result<(StatusCode, Json<TrainingRecord>)> {
    let record = TrainingService::create(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// 更新訓練紀錄
pub async fn update_training_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTrainingRecordRequest>,
) -> Result<Json<TrainingRecord>> {
    let record = TrainingService::update(&state.db, id, &payload, &current_user).await?;
    Ok(Json(record))
}

/// 刪除訓練紀錄
pub async fn delete_training_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    TrainingService::delete(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}
