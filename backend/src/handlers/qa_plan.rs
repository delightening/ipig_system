// QA 計畫管理 Handlers（稽查報告、不符合事項、SOP 文件、稽查排程）

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateCapaRequest, CreateInspectionRequest, CreateNcRequest, CreateScheduleRequest,
        CreateSopRequest, InspectionQuery, NcQuery, ScheduleQuery, SopQuery,
        UpdateCapaRequest, UpdateInspectionRequest, UpdateNcRequest,
        UpdateScheduleItemRequest, UpdateScheduleRequest, UpdateSopRequest,
    },
    require_permission,
    services::QaPlanService,
    AppState, Result,
};

// ============================================================
// 稽查報告
// ============================================================

pub async fn list_inspections(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<InspectionQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.inspection.view");

    let items = QaPlanService::list_inspections(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_inspection(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.inspection.view");

    let detail = QaPlanService::get_inspection(&state.db, id).await?;
    Ok(Json(serde_json::json!(detail)))
}

pub async fn create_inspection(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateInspectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "qau.inspection.manage");

    let detail = QaPlanService::create_inspection(
        &state.db,
        &payload,
        current_user.id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!(detail))))
}

pub async fn update_inspection(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInspectionRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.inspection.manage");

    let detail = QaPlanService::update_inspection(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(detail)))
}

// ============================================================
// 不符合事項
// ============================================================

pub async fn list_non_conformances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<NcQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.nc.view");

    let items = QaPlanService::list_non_conformances(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_non_conformance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.nc.view");

    let detail = QaPlanService::get_non_conformance(&state.db, id).await?;
    Ok(Json(serde_json::json!(detail)))
}

pub async fn create_non_conformance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateNcRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "qau.nc.manage");

    let detail = QaPlanService::create_non_conformance(
        &state.db,
        &payload,
        current_user.id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!(detail))))
}

pub async fn update_non_conformance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNcRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.nc.manage");

    let detail = QaPlanService::update_non_conformance(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(detail)))
}

pub async fn create_capa(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(nc_id): Path<Uuid>,
    Json(payload): Json<CreateCapaRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "qau.nc.manage");

    let capa = QaPlanService::create_capa(&state.db, nc_id, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(capa))))
}

pub async fn update_capa(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((nc_id, capa_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCapaRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.nc.manage");

    let capa = QaPlanService::update_capa(&state.db, nc_id, capa_id, &payload).await?;
    Ok(Json(serde_json::json!(capa)))
}

// ============================================================
// SOP 文件
// ============================================================

pub async fn list_sop_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<SopQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.sop.view");

    let items = QaPlanService::list_sop_documents(
        &state.db,
        &params,
        current_user.id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_sop_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.sop.view");

    let doc = QaPlanService::get_sop_document(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn create_sop_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateSopRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "qau.sop.manage");

    let doc = QaPlanService::create_sop(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(doc))))
}

pub async fn update_sop_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSopRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.sop.manage");

    let doc = QaPlanService::update_sop(&state.db, id, &payload, current_user.id).await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn acknowledge_sop(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.sop.view");

    let doc = QaPlanService::acknowledge_sop(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!(doc)))
}

// ============================================================
// 稽查排程
// ============================================================

pub async fn list_schedules(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ScheduleQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.schedule.view");

    let items = QaPlanService::list_schedules(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_schedule(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.schedule.view");

    let detail = QaPlanService::get_schedule(&state.db, id).await?;
    Ok(Json(serde_json::json!(detail)))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateScheduleRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.schedule.manage");

    let detail = QaPlanService::update_schedule(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(detail)))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "qau.schedule.manage");

    let detail =
        QaPlanService::create_schedule(&state.db, &payload, current_user.id).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!(detail))))
}

pub async fn update_schedule_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((schedule_id, item_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateScheduleItemRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.schedule.manage");

    let detail =
        QaPlanService::update_schedule_item(&state.db, schedule_id, item_id, &payload).await?;

    Ok(Json(serde_json::json!(detail)))
}
