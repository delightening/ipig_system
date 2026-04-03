// 設備維護管理 Handlers (實驗室 GLP)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        ActivityLogQuery, AnnualPlanExecutionSummary, AnnualPlanQuery, AnnualPlanWithEquipment,
        ApproveDisposalRequest, CalibrationQuery, CalibrationWithEquipment,
        CreateAnnualPlanRequest, CreateCalibrationRequest, CreateDisposalRequest,
        CreateEquipmentRequest, CreateEquipmentSupplierRequest, CreateMaintenanceRequest,
        DisposalQuery, DisposalWithDetails, Equipment, EquipmentCalibration,
        EquipmentHistoryQuery, EquipmentMaintenanceRecord, EquipmentQuery, EquipmentStatusLog,
        EquipmentSupplierWithPartner, EquipmentTimelineEntry, ExecutionSummaryQuery,
        GenerateAnnualPlanRequest, MaintenanceQuery, MaintenanceRecordWithDetails,
        PaginatedResponse, ReviewMaintenanceRequest, UpdateAnnualPlanRequest,
        UpdateCalibrationRequest, UpdateEquipmentRequest, UpdateMaintenanceRequest, UserActivityLog,
    },
    repositories,
    services::{AuditService, EquipmentService},
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
    let record =
        EquipmentService::update_equipment(&state.db, id, &payload, &current_user).await?;
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

// ========== Equipment Suppliers ==========

pub async fn list_equipment_suppliers_summary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<crate::models::EquipmentSupplierSummaryRow>>> {
    let result =
        EquipmentService::list_all_equipment_suppliers_summary(&state.db, &current_user).await?;
    Ok(Json(result))
}

pub async fn list_equipment_suppliers(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(equipment_id): Path<Uuid>,
) -> Result<Json<Vec<EquipmentSupplierWithPartner>>> {
    let result =
        EquipmentService::list_equipment_suppliers(&state.db, equipment_id, &current_user).await?;
    Ok(Json(result))
}

pub async fn add_equipment_supplier(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(equipment_id): Path<Uuid>,
    Json(payload): Json<CreateEquipmentSupplierRequest>,
) -> Result<(StatusCode, Json<EquipmentSupplierWithPartner>)> {
    let record = EquipmentService::add_equipment_supplier(
        &state.db,
        equipment_id,
        &payload,
        &current_user,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn remove_equipment_supplier(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    EquipmentService::remove_equipment_supplier(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ========== Status Logs ==========

pub async fn list_status_logs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(equipment_id): Path<Uuid>,
) -> Result<Json<Vec<EquipmentStatusLog>>> {
    let result =
        EquipmentService::list_status_logs(&state.db, equipment_id, &current_user).await?;
    Ok(Json(result))
}

// ========== Equipment Timeline (設備履歷) ==========

pub async fn get_equipment_timeline(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(equipment_id): Path<Uuid>,
    Query(params): Query<EquipmentHistoryQuery>,
) -> Result<Json<PaginatedResponse<EquipmentTimelineEntry>>> {
    let result = EquipmentService::get_equipment_history(
        &state.db, equipment_id, &params, &current_user,
    ).await?;
    Ok(Json(result))
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
    let record =
        EquipmentService::create_calibration(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_calibration(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCalibrationRequest>,
) -> Result<Json<EquipmentCalibration>> {
    let record =
        EquipmentService::update_calibration(&state.db, id, &payload, &current_user).await?;
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

// ========== Maintenance Records ==========

pub async fn list_maintenance_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<MaintenanceQuery>,
) -> Result<Json<PaginatedResponse<MaintenanceRecordWithDetails>>> {
    let result =
        EquipmentService::list_maintenance_records(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

pub async fn create_maintenance_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateMaintenanceRequest>,
) -> Result<(StatusCode, Json<EquipmentMaintenanceRecord>)> {
    let record =
        EquipmentService::create_maintenance_record(&state.db, &payload, &current_user).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "EQUIPMENT", "MAINTENANCE_CREATE",
        Some("maintenance_record"), Some(record.id), None,
        None,
        Some(serde_json::to_value(&record).unwrap_or_default()),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (MAINTENANCE_CREATE): {e}");
    }

    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_maintenance_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaintenanceRequest>,
) -> Result<Json<EquipmentMaintenanceRecord>> {
    let before = repositories::equipment::find_maintenance_record_by_id(&state.db, id).await?;
    let before_json = before.map(|b| serde_json::to_value(&b).unwrap_or_default());

    let record =
        EquipmentService::update_maintenance_record(&state.db, id, &payload, &current_user)
            .await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "EQUIPMENT", "MAINTENANCE_UPDATE",
        Some("maintenance_record"), Some(id), None,
        before_json,
        Some(serde_json::to_value(&record).unwrap_or_default()),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (MAINTENANCE_UPDATE): {e}");
    }

    Ok(Json(record))
}

pub async fn delete_maintenance_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let before = repositories::equipment::find_maintenance_record_by_id(&state.db, id).await?;
    let before_json = before.map(|b| serde_json::to_value(&b).unwrap_or_default());

    EquipmentService::delete_maintenance_record(&state.db, id, &current_user).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "EQUIPMENT", "MAINTENANCE_DELETE",
        Some("maintenance_record"), Some(id), None,
        before_json,
        None,
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (MAINTENANCE_DELETE): {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn review_maintenance_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewMaintenanceRequest>,
) -> Result<Json<EquipmentMaintenanceRecord>> {
    let before = repositories::equipment::find_maintenance_record_by_id(&state.db, id).await?;
    let before_json = before.map(|b| serde_json::to_value(&b).unwrap_or_default());

    let record =
        EquipmentService::review_maintenance_record(&state.db, id, &payload, &current_user)
            .await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "EQUIPMENT", "MAINTENANCE_REVIEW",
        Some("maintenance_record"), Some(id), None,
        before_json,
        Some(serde_json::to_value(&record).unwrap_or_default()),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (MAINTENANCE_REVIEW): {e}");
    }

    Ok(Json(record))
}

pub async fn get_maintenance_history(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    let query = ActivityLogQuery {
        user_id: None,
        event_category: Some("EQUIPMENT".to_string()),
        event_type: None,
        entity_type: Some("maintenance_record".to_string()),
        entity_id: Some(id),
        is_suspicious: None,
        from: None,
        to: None,
        page: Some(1),
        per_page: Some(100),
    };
    let result = AuditService::list_activities(&state.db, &query).await?;
    Ok(Json(result))
}

// ========== Disposal Records ==========

pub async fn list_disposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<DisposalQuery>,
) -> Result<Json<PaginatedResponse<DisposalWithDetails>>> {
    let result = EquipmentService::list_disposals(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

pub async fn create_disposal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateDisposalRequest>,
) -> Result<(StatusCode, Json<DisposalWithDetails>)> {
    let record = EquipmentService::create_disposal(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn approve_disposal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveDisposalRequest>,
) -> Result<Json<DisposalWithDetails>> {
    let record =
        EquipmentService::approve_disposal(&state.db, id, &payload, &current_user).await?;
    Ok(Json(record))
}

pub async fn restore_equipment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DisposalWithDetails>> {
    let record = EquipmentService::restore_equipment(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

// ========== Annual Plan ==========

pub async fn list_annual_plans(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AnnualPlanQuery>,
) -> Result<Json<Vec<AnnualPlanWithEquipment>>> {
    let result = EquipmentService::list_annual_plans(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}

pub async fn generate_annual_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<GenerateAnnualPlanRequest>,
) -> Result<Json<Vec<AnnualPlanWithEquipment>>> {
    let result =
        EquipmentService::generate_annual_plan(&state.db, &payload, &current_user).await?;
    Ok(Json(result))
}

pub async fn create_annual_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateAnnualPlanRequest>,
) -> Result<(StatusCode, Json<AnnualPlanWithEquipment>)> {
    let plan =
        EquipmentService::create_annual_plan(&state.db, &payload, &current_user).await?;
    Ok((StatusCode::CREATED, Json(plan)))
}

pub async fn update_annual_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAnnualPlanRequest>,
) -> Result<Json<AnnualPlanWithEquipment>> {
    let plan =
        EquipmentService::update_annual_plan(&state.db, id, &payload, &current_user).await?;
    Ok(Json(plan))
}

pub async fn delete_annual_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    EquipmentService::delete_annual_plan(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_annual_plan_execution_summary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ExecutionSummaryQuery>,
) -> Result<Json<AnnualPlanExecutionSummary>> {
    let result =
        EquipmentService::get_execution_summary(&state.db, &params, &current_user).await?;
    Ok(Json(result))
}
