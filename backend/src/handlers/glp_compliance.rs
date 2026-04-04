// GLP 合規模組 Handlers
// 涵蓋：參考標準、文件控制、管理審查、風險管理、變更控制、環境監控、能力評鑑、最終報告、配製紀錄

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::glp_compliance::*,
    require_permission,
    services::GlpComplianceService,
    AppState, Result,
};

// ============================================================
// Reference Standards (參考標準器)
// ============================================================

pub async fn list_reference_standards(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "equipment.view");
    let items = GlpComplianceService::list_reference_standards(&state.db).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_reference_standard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "equipment.view");
    let item = GlpComplianceService::get_reference_standard(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_reference_standard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateReferenceStandardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "equipment.manage");
    let item = GlpComplianceService::create_reference_standard(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_reference_standard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateReferenceStandardRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "equipment.manage");
    let item = GlpComplianceService::update_reference_standard(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

// ============================================================
// Controlled Documents (文件控制)
// ============================================================

pub async fn list_controlled_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ControlledDocumentQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.view");
    let items = GlpComplianceService::list_controlled_documents(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.view");
    let doc = GlpComplianceService::get_controlled_document(&state.db, id).await?;
    let revisions = GlpComplianceService::get_document_revisions(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "document": doc, "revisions": revisions })))
}

pub async fn create_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateControlledDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "dms.document.manage");
    let doc = GlpComplianceService::create_controlled_document(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(doc))))
}

pub async fn update_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateControlledDocumentRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.manage");
    let doc = GlpComplianceService::update_controlled_document(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn approve_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.approve");
    let doc = GlpComplianceService::approve_controlled_document(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn create_revision(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateRevisionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "dms.document.manage");
    let rev = GlpComplianceService::create_revision(&state.db, id, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(rev))))
}

pub async fn acknowledge_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.view");
    let ack = GlpComplianceService::acknowledge_document(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!(ack)))
}

// ============================================================
// Management Reviews (管理審查)
// ============================================================

pub async fn list_management_reviews(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ManagementReviewQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "glp.management_review.view");
    let items = GlpComplianceService::list_management_reviews(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_management_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "glp.management_review.view");
    let item = GlpComplianceService::get_management_review(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_management_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateManagementReviewRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "glp.management_review.manage");
    let item = GlpComplianceService::create_management_review(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_management_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateManagementReviewRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "glp.management_review.manage");
    let item = GlpComplianceService::update_management_review(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

// ============================================================
// Risk Register (風險管理)
// ============================================================

pub async fn list_risks(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<RiskQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "risk.register.view");
    let items = GlpComplianceService::list_risks(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_risk(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "risk.register.view");
    let item = GlpComplianceService::get_risk(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_risk(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateRiskRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "risk.register.manage");
    let item = GlpComplianceService::create_risk(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_risk(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRiskRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "risk.register.manage");
    let item = GlpComplianceService::update_risk(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

// ============================================================
// Change Requests (變更控制)
// ============================================================

pub async fn list_change_requests(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ChangeRequestQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.view");
    let items = GlpComplianceService::list_change_requests(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.view");
    let item = GlpComplianceService::get_change_request(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateChangeRequestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "change.request.manage");
    let item = GlpComplianceService::create_change_request(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChangeRequestRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.manage");
    let item = GlpComplianceService::update_change_request(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn approve_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.approve");
    let item = GlpComplianceService::approve_change_request(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!(item)))
}

// ============================================================
// Environment Monitoring (環境監控)
// ============================================================

#[derive(serde::Deserialize, Default)]
pub struct MonitoringPointQueryParams {
    pub active_only: Option<bool>,
}

pub async fn list_monitoring_points(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<MonitoringPointQueryParams>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "env.monitoring.view");
    let items = GlpComplianceService::list_monitoring_points(&state.db, params.active_only.unwrap_or(false)).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_monitoring_point(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "env.monitoring.view");
    let item = GlpComplianceService::get_monitoring_point(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_monitoring_point(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateMonitoringPointRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "env.monitoring.manage");
    let item = GlpComplianceService::create_monitoring_point(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_monitoring_point(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMonitoringPointRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "env.monitoring.manage");
    let item = GlpComplianceService::update_monitoring_point(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn list_readings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ReadingQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "env.monitoring.view");
    let items = GlpComplianceService::list_readings(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_reading(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateReadingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "env.monitoring.manage");
    let item = GlpComplianceService::create_reading(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

// ============================================================
// Competency Assessments (能力評鑑)
// ============================================================

pub async fn list_competency_assessments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<CompetencyQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "competency.assessment.view");
    let items = GlpComplianceService::list_competency_assessments(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_competency_assessment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateCompetencyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "competency.assessment.manage");
    let item = GlpComplianceService::create_competency_assessment(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_competency_assessment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCompetencyRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "competency.assessment.manage");
    let item = GlpComplianceService::update_competency_assessment(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

#[derive(serde::Deserialize, Default)]
pub struct TrainingReqQueryParams {
    pub role_code: Option<String>,
}

pub async fn list_training_requirements(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TrainingReqQueryParams>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "competency.assessment.view");
    let items = GlpComplianceService::list_training_requirements(&state.db, params.role_code.as_deref()).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_training_requirement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateTrainingRequirementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "competency.assessment.manage");
    let item = GlpComplianceService::create_training_requirement(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn delete_training_requirement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "competency.assessment.manage");
    GlpComplianceService::delete_training_requirement(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// Study Final Reports (最終報告)
// ============================================================

pub async fn list_study_reports(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StudyReportQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "study.report.view");
    let items = GlpComplianceService::list_study_reports(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "study.report.view");
    let item = GlpComplianceService::get_study_report(&state.db, id).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn create_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateStudyReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "study.report.manage");
    let item = GlpComplianceService::create_study_report(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStudyReportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "study.report.manage");
    let item = GlpComplianceService::update_study_report(&state.db, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

// ============================================================
// Formulation Records (配製紀錄)
// ============================================================

pub async fn list_formulation_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<FormulationQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "formulation.record.view");
    let items = GlpComplianceService::list_formulation_records(&state.db, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_formulation_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateFormulationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "formulation.record.manage");
    let item = GlpComplianceService::create_formulation_record(&state.db, &payload, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}
