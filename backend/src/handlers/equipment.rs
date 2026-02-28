// 設備與校準紀錄 Handlers (實驗室 GLP)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CalibrationQuery, CalibrationWithEquipment, CreateCalibrationRequest,
        CreateEquipmentRequest, Equipment, EquipmentCalibration, EquipmentQuery,
        PaginatedResponse, UpdateCalibrationRequest, UpdateEquipmentRequest,
    },
    services::EquipmentService,
    AppState, Result,
};

// ========== Equipment ==========

pub async fn list_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<EquipmentQuery>,
) -> Result<Json<PaginatedResponse<Equipment>>> {
    let result = EquipmentService::list_equipment(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

pub async fn get_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Equipment>> {
    let record = EquipmentService::get_equipment(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

pub async fn create_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateEquipmentRequest>,
) -> Result<(StatusCode, Json<Equipment>)> {
    let record = EquipmentService::create_equipment(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEquipmentRequest>,
) -> Result<Json<Equipment>> {
    let record = EquipmentService::update_equipment(&state.db, id, &payload, &current_user).await?;
    Ok(Json(record))
}

pub async fn delete_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    EquipmentService::delete_equipment(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ========== Calibrations ==========

pub async fn list_calibrations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<CalibrationQuery>,
) -> Result<Json<PaginatedResponse<CalibrationWithEquipment>>> {
    let result = EquipmentService::list_calibrations(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

pub async fn get_calibration(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<EquipmentCalibration>> {
    let record = EquipmentService::get_calibration(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

pub async fn create_calibration(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateCalibrationRequest>,
) -> Result<(StatusCode, Json<EquipmentCalibration>)> {
    let record = EquipmentService::create_calibration(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_calibration(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCalibrationRequest>,
) -> Result<Json<EquipmentCalibration>> {
    let record = EquipmentService::update_calibration(&state.db, id, &payload, &current_user).await?;
    Ok(Json(record))
}

pub async fn delete_calibration(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    EquipmentService::delete_calibration(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}
