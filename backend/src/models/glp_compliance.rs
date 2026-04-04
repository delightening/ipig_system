// GLP 合規模組 — 文件控制、管理審查、風險管理、變更控制、環境監控、能力評鑑、最終報告、參考標準、配製紀錄

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Reference Standards (參考標準器)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct ReferenceStandard {
    pub id: Uuid,
    pub name: String,
    pub serial_number: Option<String>,
    pub standard_type: String,
    pub traceable_to: Option<String>,
    pub national_standard_number: Option<String>,
    pub calibration_lab: Option<String>,
    pub calibration_lab_accreditation: Option<String>,
    pub last_calibrated_at: Option<NaiveDate>,
    pub next_due_at: Option<NaiveDate>,
    pub certificate_number: Option<String>,
    pub measurement_uncertainty: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReferenceStandardRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub serial_number: Option<String>,
    pub standard_type: Option<String>,
    pub traceable_to: Option<String>,
    pub national_standard_number: Option<String>,
    pub calibration_lab: Option<String>,
    pub calibration_lab_accreditation: Option<String>,
    pub last_calibrated_at: Option<NaiveDate>,
    pub next_due_at: Option<NaiveDate>,
    pub certificate_number: Option<String>,
    pub measurement_uncertainty: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReferenceStandardRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    pub serial_number: Option<String>,
    pub standard_type: Option<String>,
    pub traceable_to: Option<String>,
    pub national_standard_number: Option<String>,
    pub calibration_lab: Option<String>,
    pub calibration_lab_accreditation: Option<String>,
    pub last_calibrated_at: Option<NaiveDate>,
    pub next_due_at: Option<NaiveDate>,
    pub certificate_number: Option<String>,
    pub measurement_uncertainty: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

// ============================================================================
// Controlled Documents (文件控制)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct ControlledDocument {
    pub id: Uuid,
    pub doc_number: String,
    pub title: String,
    pub doc_type: String,
    pub category: Option<String>,
    pub current_version: i32,
    pub status: String,
    pub effective_date: Option<NaiveDate>,
    pub review_due_date: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub obsoleted_at: Option<DateTime<Utc>>,
    pub retention_years: Option<i32>,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ControlledDocumentWithOwner {
    pub id: Uuid,
    pub doc_number: String,
    pub title: String,
    pub doc_type: String,
    pub category: Option<String>,
    pub current_version: i32,
    pub status: String,
    pub effective_date: Option<NaiveDate>,
    pub review_due_date: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub retention_years: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateControlledDocumentRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: String,
    pub doc_type: String,
    pub category: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub review_due_date: Option<NaiveDate>,
    pub retention_years: Option<i32>,
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateControlledDocumentRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub review_due_date: Option<NaiveDate>,
    pub retention_years: Option<i32>,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DocumentRevision {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: i32,
    pub change_summary: String,
    pub revised_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRevisionRequest {
    pub change_summary: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DocumentAcknowledgment {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub acknowledged_at: DateTime<Utc>,
    pub version_acknowledged: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct ControlledDocumentQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub doc_type: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
}

// ============================================================================
// Management Reviews (管理審查)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct ManagementReview {
    pub id: Uuid,
    pub review_number: String,
    pub title: String,
    pub review_date: NaiveDate,
    pub status: String,
    pub agenda: Option<String>,
    pub attendees: Option<serde_json::Value>,
    pub minutes: Option<String>,
    pub decisions: Option<serde_json::Value>,
    pub action_items: Option<serde_json::Value>,
    pub chaired_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateManagementReviewRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: String,
    pub review_date: NaiveDate,
    pub agenda: Option<String>,
    pub attendees: Option<serde_json::Value>,
    pub chaired_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateManagementReviewRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: Option<String>,
    pub review_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub agenda: Option<String>,
    pub attendees: Option<serde_json::Value>,
    pub minutes: Option<String>,
    pub decisions: Option<serde_json::Value>,
    pub action_items: Option<serde_json::Value>,
    pub chaired_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ManagementReviewQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
}

// ============================================================================
// Risk Register (風險登記簿)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct RiskEntry {
    pub id: Uuid,
    pub risk_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub severity: i32,
    pub likelihood: i32,
    pub detectability: Option<i32>,
    pub risk_score: Option<i32>,
    pub status: String,
    pub mitigation_plan: Option<String>,
    pub residual_risk_score: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub review_date: Option<NaiveDate>,
    pub related_nc_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RiskEntryWithOwner {
    pub id: Uuid,
    pub risk_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub severity: i32,
    pub likelihood: i32,
    pub detectability: Option<i32>,
    pub risk_score: Option<i32>,
    pub status: String,
    pub mitigation_plan: Option<String>,
    pub residual_risk_score: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub review_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRiskRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    #[validate(range(min = 1, max = 5))]
    pub severity: i32,
    #[validate(range(min = 1, max = 5))]
    pub likelihood: i32,
    pub detectability: Option<i32>,
    pub mitigation_plan: Option<String>,
    pub owner_id: Option<Uuid>,
    pub review_date: Option<NaiveDate>,
    pub related_nc_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRiskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub severity: Option<i32>,
    pub likelihood: Option<i32>,
    pub detectability: Option<i32>,
    pub status: Option<String>,
    pub mitigation_plan: Option<String>,
    pub residual_risk_score: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub review_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RiskQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub category: Option<String>,
}

// ============================================================================
// Change Requests (變更控制)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct ChangeRequest {
    pub id: Uuid,
    pub change_number: String,
    pub title: String,
    pub change_type: String,
    pub description: String,
    pub justification: Option<String>,
    pub impact_assessment: Option<String>,
    pub status: String,
    pub requested_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub implemented_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ChangeRequestWithNames {
    pub id: Uuid,
    pub change_number: String,
    pub title: String,
    pub change_type: String,
    pub description: String,
    pub justification: Option<String>,
    pub impact_assessment: Option<String>,
    pub status: String,
    pub requested_by: Uuid,
    pub requester_name: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChangeRequestRequest {
    #[validate(length(min = 1, max = 300))]
    pub title: String,
    pub change_type: String,
    #[validate(length(min = 1))]
    pub description: String,
    pub justification: Option<String>,
    pub impact_assessment: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChangeRequestRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub justification: Option<String>,
    pub impact_assessment: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChangeRequestQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub change_type: Option<String>,
}

// ============================================================================
// Environment Monitoring (環境監控)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct EnvironmentMonitoringPoint {
    pub id: Uuid,
    pub name: String,
    pub location_type: String,
    pub building_id: Option<Uuid>,
    pub zone_id: Option<Uuid>,
    pub parameters: serde_json::Value,
    pub monitoring_interval: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMonitoringPointRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub location_type: String,
    pub building_id: Option<Uuid>,
    pub zone_id: Option<Uuid>,
    pub parameters: serde_json::Value,
    pub monitoring_interval: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMonitoringPointRequest {
    pub name: Option<String>,
    pub location_type: Option<String>,
    pub building_id: Option<Uuid>,
    pub zone_id: Option<Uuid>,
    pub parameters: Option<serde_json::Value>,
    pub monitoring_interval: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EnvironmentReading {
    pub id: Uuid,
    pub monitoring_point_id: Uuid,
    pub reading_time: DateTime<Utc>,
    pub readings: serde_json::Value,
    pub is_out_of_range: bool,
    pub out_of_range_params: Option<serde_json::Value>,
    pub recorded_by: Option<Uuid>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReadingRequest {
    pub monitoring_point_id: Uuid,
    pub reading_time: DateTime<Utc>,
    pub readings: serde_json::Value,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ReadingQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub monitoring_point_id: Option<Uuid>,
    pub is_out_of_range: Option<bool>,
}

// ============================================================================
// Competency Assessments (能力評鑑)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct CompetencyAssessment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub assessment_type: String,
    pub skill_area: String,
    pub assessment_date: NaiveDate,
    pub assessor_id: Uuid,
    pub result: String,
    pub score: Option<rust_decimal::Decimal>,
    pub method: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CompetencyAssessmentWithNames {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub assessment_type: String,
    pub skill_area: String,
    pub assessment_date: NaiveDate,
    pub assessor_id: Uuid,
    pub assessor_name: Option<String>,
    pub result: String,
    pub score: Option<rust_decimal::Decimal>,
    pub method: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCompetencyRequest {
    pub user_id: Uuid,
    pub assessment_type: String,
    #[validate(length(min = 1, max = 200))]
    pub skill_area: String,
    pub assessment_date: NaiveDate,
    pub result: String,
    pub score: Option<rust_decimal::Decimal>,
    pub method: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCompetencyRequest {
    pub result: Option<String>,
    pub score: Option<rust_decimal::Decimal>,
    pub method: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CompetencyQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub user_id: Option<Uuid>,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RoleTrainingRequirement {
    pub id: Uuid,
    pub role_code: String,
    pub training_topic: String,
    pub is_mandatory: bool,
    pub recurrence_months: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTrainingRequirementRequest {
    pub role_code: String,
    #[validate(length(min = 1, max = 200))]
    pub training_topic: String,
    pub is_mandatory: Option<bool>,
    pub recurrence_months: Option<i32>,
}

// ============================================================================
// Study Final Reports (最終報告)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct StudyFinalReport {
    pub id: Uuid,
    pub report_number: String,
    pub protocol_id: Uuid,
    pub title: String,
    pub status: String,
    pub summary: Option<String>,
    pub methods: Option<String>,
    pub results: Option<String>,
    pub conclusions: Option<String>,
    pub deviations: Option<String>,
    pub signed_by: Option<Uuid>,
    pub signed_at: Option<DateTime<Utc>>,
    pub signature_id: Option<Uuid>,
    pub qau_statement: Option<String>,
    pub qau_signed_by: Option<Uuid>,
    pub qau_signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStudyReportRequest {
    pub protocol_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub summary: Option<String>,
    pub methods: Option<String>,
    pub results: Option<String>,
    pub conclusions: Option<String>,
    pub deviations: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStudyReportRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
    pub methods: Option<String>,
    pub results: Option<String>,
    pub conclusions: Option<String>,
    pub deviations: Option<String>,
    pub qau_statement: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct StudyReportQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub protocol_id: Option<Uuid>,
}

// ============================================================================
// Formulation Records (配製紀錄)
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
pub struct FormulationRecord {
    pub id: Uuid,
    pub product_id: Uuid,
    pub protocol_id: Option<Uuid>,
    pub formulation_date: NaiveDate,
    pub batch_number: Option<String>,
    pub concentration: Option<String>,
    pub volume: Option<String>,
    pub prepared_by: Uuid,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expiry_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FormulationRecordWithNames {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: Option<String>,
    pub protocol_id: Option<Uuid>,
    pub formulation_date: NaiveDate,
    pub batch_number: Option<String>,
    pub concentration: Option<String>,
    pub volume: Option<String>,
    pub prepared_by: Uuid,
    pub preparer_name: Option<String>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expiry_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateFormulationRequest {
    pub product_id: Uuid,
    pub protocol_id: Option<Uuid>,
    pub formulation_date: NaiveDate,
    pub batch_number: Option<String>,
    pub concentration: Option<String>,
    pub volume: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct FormulationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub product_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
}
