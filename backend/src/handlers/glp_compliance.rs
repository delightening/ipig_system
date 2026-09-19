// GLP 合規模組 Handlers
// 涵蓋：參考標準、文件控制、管理審查、風險管理、變更控制、環境監控、能力評鑑、最終報告、配製紀錄

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::glp_compliance::*,
    require_permission,
    services::GlpComplianceService,
    AppState, Result,
};

use super::SignRecordRequest;

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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_reference_standard(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_reference_standard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateReferenceStandardRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "equipment.manage");
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::update_reference_standard(&state.db, &actor, id, &payload).await?;
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
    Ok(Json(
        serde_json::json!({ "document": doc, "revisions": revisions }),
    ))
}

pub async fn create_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateControlledDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "dms.document.manage");
    let actor = ActorContext::User(current_user);
    let doc = GlpComplianceService::create_controlled_document(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(doc))))
}

pub async fn update_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateControlledDocumentRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.manage");
    let actor = ActorContext::User(current_user);
    let doc =
        GlpComplianceService::update_controlled_document(&state.db, &actor, id, &payload).await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn approve_controlled_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.approve");
    let actor = ActorContext::User(current_user);
    let doc = GlpComplianceService::approve_controlled_document(
        &state.db,
        &actor,
        id,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;
    Ok(Json(serde_json::json!(doc)))
}

pub async fn create_revision(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateRevisionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "dms.document.manage");
    let actor = ActorContext::User(current_user);
    let rev = GlpComplianceService::create_revision(&state.db, &actor, id, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(rev))))
}

pub async fn acknowledge_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dms.document.view");
    let actor = ActorContext::User(current_user);
    let ack = GlpComplianceService::acknowledge_document(&state.db, &actor, id).await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_management_review(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_management_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateManagementReviewRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "glp.management_review.manage");
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::update_management_review(&state.db, &actor, id, &payload).await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_risk(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_risk(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRiskRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "risk.register.manage");
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::update_risk(&state.db, &actor, id, &payload).await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_change_request(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChangeRequestRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.manage");
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::update_change_request(&state.db, &actor, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn approve_change_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "change.request.approve");
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::approve_change_request(
        &state.db,
        &actor,
        id,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;
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
    let items = GlpComplianceService::list_monitoring_points(
        &state.db,
        params.active_only.unwrap_or(false),
    )
    .await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_monitoring_point(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_monitoring_point(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMonitoringPointRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "env.monitoring.manage");
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::update_monitoring_point(&state.db, &actor, id, &payload).await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_reading(&state.db, &actor, &payload).await?;
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
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::create_competency_assessment(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn update_competency_assessment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCompetencyRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "competency.assessment.manage");
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::update_competency_assessment(&state.db, &actor, id, &payload).await?;
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
    let items =
        GlpComplianceService::list_training_requirements(&state.db, params.role_code.as_deref())
            .await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_training_requirement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateTrainingRequirementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    require_permission!(current_user, "competency.assessment.manage");
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::create_training_requirement(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

pub async fn delete_training_requirement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "competency.assessment.manage");
    let actor = ActorContext::User(current_user);
    GlpComplianceService::delete_training_requirement(&state.db, &actor, id).await?;
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
    let items = GlpComplianceService::list_study_reports(&state.db, &current_user, &params).await?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn get_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let item = GlpComplianceService::get_study_report(&state.db, &current_user, id).await?;
    Ok(Json(serde_json::json!(item)))
}

/// 2026-09-05 起不再檢查 `study.report.manage`——建立最終報告改走身分即授權
/// （只有 `req.protocol_id` 對應計畫的 Study Director 或 admin 可建立），
/// 見 `GlpComplianceService::require_study_director`。
pub async fn create_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateStudyReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_study_report(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

/// 同上，改走身分即授權；`qau_statement` 已移出本端點，走 `update_qau_statement`。
pub async fn update_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStudyReportRequest>,
) -> Result<Json<serde_json::Value>> {
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::update_study_report(&state.db, &actor, id, &payload).await?;
    Ok(Json(serde_json::json!(item)))
}

/// SD 簽署最終報告。身分即授權，無 admin 例外——見
/// `GlpComplianceService::sign_study_report`。
pub async fn sign_study_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<serde_json::Value>> {
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::sign_study_report(
        &state.db,
        &actor,
        id,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;
    Ok(Json(serde_json::json!(item)))
}

/// QAU 品保聲明填寫，與報告本文分開授權（P0-1）。
///
/// ⚠️ **這裡的 `validate()` 是顯式呼叫，不是靠 extractor**（CodeRabbit 於 #102 指出）：
/// 本模組沒有任何 validating extractor，`#[derive(Validate)]` 不會自己生效。
/// 少了這一句，空字串的品保聲明會連同 `qau_signed_by` / `qau_signed_at` 一起寫進去
/// ——產生一張「有簽署人、有時間、沒有內容」的品保聲明，在 GLP 稽核上是最糟的形狀。
///
/// ⚠️ 同一個缺口在本模組其他 handler 也在（例如 `create_study_report` 的 `title`
/// 長度限制同樣沒被執行）。那是本 PR 之前就有的既有問題，不在本次範圍內順手擴大；
/// 這則註解留著，讓下一個人知道它是系統性的，而不是這一支漏掉。
pub async fn update_qau_statement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<QauStatementRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "qau.report_statement.write");
    payload.validate()?;
    let actor = ActorContext::User(current_user);
    let item =
        GlpComplianceService::update_qau_statement(&state.db, &actor, id, &payload.qau_statement)
            .await?;
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
    let actor = ActorContext::User(current_user);
    let item = GlpComplianceService::create_formulation_record(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}
