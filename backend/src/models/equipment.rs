// 設備維護管理 Models (實驗室 GLP)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use validator::Validate;

// ========== GLP/GMP 確效階段 ==========

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "validation_phase", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationPhase {
    #[serde(rename = "IQ")]
    Iq,
    #[serde(rename = "OQ")]
    Oq,
    #[serde(rename = "PQ")]
    Pq,
}

// ========== Enums ==========

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "equipment_status", rename_all = "snake_case")]
pub enum EquipmentStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "under_repair")]
    UnderRepair,
    #[serde(rename = "decommissioned")]
    Decommissioned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "calibration_type", rename_all = "snake_case")]
pub enum CalibrationType {
    #[serde(rename = "calibration")]
    Calibration,
    #[serde(rename = "validation")]
    Validation,
    #[serde(rename = "inspection")]
    Inspection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "calibration_cycle", rename_all = "snake_case")]
pub enum CalibrationCycle {
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "quarterly")]
    Quarterly,
    #[serde(rename = "semi_annual")]
    SemiAnnual,
    #[serde(rename = "annual")]
    Annual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "maintenance_type", rename_all = "snake_case")]
pub enum MaintenanceType {
    #[serde(rename = "repair")]
    Repair,
    #[serde(rename = "maintenance")]
    Maintenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "maintenance_status", rename_all = "snake_case")]
pub enum MaintenanceStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "unrepairable")]
    Unrepairable,
    #[serde(rename = "pending_review")]
    PendingReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "disposal_status", rename_all = "snake_case")]
pub enum DisposalStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
}

// ========== Equipment ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Equipment {
    pub id: Uuid,
    pub name: String,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub department: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub status: EquipmentStatus,
    pub calibration_type: Option<CalibrationType>,
    pub calibration_cycle: Option<CalibrationCycle>,
    pub inspection_cycle: Option<CalibrationCycle>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentQuery {
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
    pub status: Option<EquipmentStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEquipmentRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(max = 200))]
    pub model: Option<String>,
    #[validate(length(max = 100))]
    pub serial_number: Option<String>,
    #[validate(length(max = 200))]
    pub location: Option<String>,
    #[validate(length(max = 100))]
    pub department: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub calibration_type: Option<CalibrationType>,
    pub calibration_cycle: Option<CalibrationCycle>,
    pub inspection_cycle: Option<CalibrationCycle>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEquipmentRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    #[validate(length(max = 200))]
    pub model: Option<String>,
    #[validate(length(max = 100))]
    pub serial_number: Option<String>,
    #[validate(length(max = 200))]
    pub location: Option<String>,
    #[validate(length(max = 100))]
    pub department: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub calibration_type: Option<CalibrationType>,
    pub calibration_cycle: Option<CalibrationCycle>,
    pub inspection_cycle: Option<CalibrationCycle>,
}

// ========== Calibrations (校正/確效/查核) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentCalibration {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub calibration_type: CalibrationType,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    pub result: Option<String>,
    pub notes: Option<String>,
    pub partner_id: Option<Uuid>,
    pub report_number: Option<String>,
    pub inspector: Option<String>,
    pub equipment_serial_number: Option<String>,
    // ISO 17025 合規欄位
    pub certificate_number: Option<String>,
    pub performed_by: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub measurement_uncertainty: Option<String>,
    // GMP/GLP 確效欄位（calibration_type = validation 時使用）
    pub validation_phase: Option<ValidationPhase>,
    pub protocol_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CalibrationWithEquipment {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_serial_number: Option<String>,
    pub calibration_type: CalibrationType,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    pub result: Option<String>,
    pub notes: Option<String>,
    pub partner_id: Option<Uuid>,
    pub partner_name: Option<String>,
    pub report_number: Option<String>,
    pub inspector: Option<String>,
    // ISO 17025 合規欄位
    pub certificate_number: Option<String>,
    pub performed_by: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub measurement_uncertainty: Option<String>,
    // GMP/GLP 確效欄位
    pub validation_phase: Option<ValidationPhase>,
    pub protocol_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CalibrationQuery {
    pub equipment_id: Option<Uuid>,
    pub calibration_type: Option<CalibrationType>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCalibrationRequest {
    pub equipment_id: Uuid,
    pub calibration_type: CalibrationType,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    #[validate(length(max = 50))]
    pub result: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub partner_id: Option<Uuid>,
    #[validate(length(max = 100))]
    pub report_number: Option<String>,
    #[validate(length(max = 100))]
    pub inspector: Option<String>,
    // ISO 17025 合規欄位
    #[validate(length(max = 100))]
    pub certificate_number: Option<String>,
    #[validate(length(max = 100))]
    pub performed_by: Option<String>,
    #[validate(length(max = 200))]
    pub acceptance_criteria: Option<String>,
    #[validate(length(max = 100))]
    pub measurement_uncertainty: Option<String>,
    // GMP/GLP 確效欄位
    pub validation_phase: Option<ValidationPhase>,
    #[validate(length(max = 100))]
    pub protocol_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCalibrationRequest {
    pub calibration_type: Option<CalibrationType>,
    pub calibrated_at: Option<NaiveDate>,
    pub next_due_at: Option<NaiveDate>,
    #[validate(length(max = 50))]
    pub result: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub partner_id: Option<Uuid>,
    #[validate(length(max = 100))]
    pub report_number: Option<String>,
    #[validate(length(max = 100))]
    pub inspector: Option<String>,
    // ISO 17025 合規欄位
    #[validate(length(max = 100))]
    pub certificate_number: Option<String>,
    #[validate(length(max = 100))]
    pub performed_by: Option<String>,
    #[validate(length(max = 200))]
    pub acceptance_criteria: Option<String>,
    #[validate(length(max = 100))]
    pub measurement_uncertainty: Option<String>,
    // GMP/GLP 確效欄位
    pub validation_phase: Option<ValidationPhase>,
    #[validate(length(max = 100))]
    pub protocol_number: Option<String>,
}

// ========== Equipment Suppliers (設備-廠商關聯) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentSupplier {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub partner_id: Uuid,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentSupplierWithPartner {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: String,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
    pub partner_phone: Option<String>,
    pub partner_phone_ext: Option<String>,
    pub partner_email: Option<String>,
    pub partner_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEquipmentSupplierRequest {
    pub partner_id: Uuid,
    #[validate(length(max = 100))]
    pub contact_person: Option<String>,
    #[validate(length(max = 50))]
    pub contact_phone: Option<String>,
    #[validate(length(max = 255))]
    pub contact_email: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

// ========== Supplier Summary (廠商摘要) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentSupplierSummaryRow {
    pub equipment_id: Uuid,
    pub partner_name: String,
}

// ========== Status Log (狀態變更紀錄) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentStatusLog {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub old_status: EquipmentStatus,
    pub new_status: EquipmentStatus,
    pub changed_by: Uuid,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ========== Maintenance Records (維修/保養紀錄) ==========

// 維修保養紀錄，無敏感欄位（all fields 皆稽核項目）
impl crate::models::audit_diff::AuditRedact for EquipmentMaintenanceRecord {}

// 設備本體無敏感欄位（location / status / model 等均為可稽核項目）
impl crate::models::audit_diff::AuditRedact for Equipment {}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentMaintenanceRecord {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub status: MaintenanceStatus,
    pub reported_at: NaiveDate,
    pub completed_at: Option<NaiveDate>,
    pub problem_description: Option<String>,
    pub repair_content: Option<String>,
    pub repair_partner_id: Option<Uuid>,
    pub maintenance_items: Option<String>,
    pub performed_by: Option<String>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer_signature_id: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MaintenanceRecordWithDetails {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub maintenance_type: MaintenanceType,
    pub status: MaintenanceStatus,
    pub reported_at: NaiveDate,
    pub completed_at: Option<NaiveDate>,
    pub problem_description: Option<String>,
    pub repair_content: Option<String>,
    pub repair_partner_id: Option<Uuid>,
    pub repair_partner_name: Option<String>,
    pub maintenance_items: Option<String>,
    pub performed_by: Option<String>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceQuery {
    pub equipment_id: Option<Uuid>,
    pub maintenance_type: Option<MaintenanceType>,
    pub status: Option<MaintenanceStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMaintenanceRequest {
    pub equipment_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub reported_at: NaiveDate,
    pub completed_at: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub problem_description: Option<String>,
    #[validate(length(max = 2000))]
    pub repair_content: Option<String>,
    pub repair_partner_id: Option<Uuid>,
    #[validate(length(max = 2000))]
    pub maintenance_items: Option<String>,
    #[validate(length(max = 100))]
    pub performed_by: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMaintenanceRequest {
    pub status: Option<MaintenanceStatus>,
    pub completed_at: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub problem_description: Option<String>,
    #[validate(length(max = 2000))]
    pub repair_content: Option<String>,
    pub repair_partner_id: Option<Uuid>,
    #[validate(length(max = 2000))]
    pub maintenance_items: Option<String>,
    #[validate(length(max = 100))]
    pub performed_by: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReviewMaintenanceRequest {
    pub approved: bool,
    #[validate(length(max = 2000))]
    pub review_notes: Option<String>,
}

// ========== Disposal Records (報廢紀錄) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentDisposal {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub status: DisposalStatus,
    pub disposal_date: Option<NaiveDate>,
    pub reason: String,
    pub disposal_method: Option<String>,
    pub applied_by: Uuid,
    pub applied_at: DateTime<Utc>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub applicant_signature_id: Option<Uuid>,
    pub approver_signature_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DisposalWithDetails {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub status: DisposalStatus,
    pub disposal_date: Option<NaiveDate>,
    pub reason: String,
    pub disposal_method: Option<String>,
    pub applied_by: Uuid,
    pub applicant_name: String,
    pub applied_at: DateTime<Utc>,
    pub approved_by: Option<Uuid>,
    pub approver_name: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DisposalQuery {
    pub equipment_id: Option<Uuid>,
    pub status: Option<DisposalStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDisposalRequest {
    pub equipment_id: Uuid,
    pub disposal_date: Option<NaiveDate>,
    #[validate(length(min = 1, max = 2000))]
    pub reason: String,
    #[validate(length(max = 2000))]
    pub disposal_method: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveDisposalRequest {
    pub approved: bool,
    #[validate(length(max = 2000))]
    pub rejection_reason: Option<String>,
}

// ========== Idle Requests (閒置審批) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct IdleRequestWithDetails {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub request_type: String,
    pub reason: String,
    pub status: DisposalStatus,
    pub applied_by: Uuid,
    pub applicant_name: String,
    pub applied_at: DateTime<Utc>,
    pub approved_by: Option<Uuid>,
    pub approver_name: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct IdleRequestQuery {
    pub equipment_id: Option<Uuid>,
    pub status: Option<DisposalStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateIdleRequestRequest {
    pub equipment_id: Uuid,
    /// "idle" 或 "restore"
    pub request_type: String,
    #[validate(length(min = 1, max = 2000))]
    pub reason: String,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveIdleRequestRequest {
    pub approved: bool,
    #[validate(length(max = 2000))]
    pub rejection_reason: Option<String>,
}

// ========== Annual Plan (年度維護校正計畫表) ==========

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentAnnualPlan {
    pub id: Uuid,
    pub year: i32,
    pub equipment_id: Uuid,
    pub calibration_type: CalibrationType,
    pub cycle: CalibrationCycle,
    pub month_1: bool,
    pub month_2: bool,
    pub month_3: bool,
    pub month_4: bool,
    pub month_5: bool,
    pub month_6: bool,
    pub month_7: bool,
    pub month_8: bool,
    pub month_9: bool,
    pub month_10: bool,
    pub month_11: bool,
    pub month_12: bool,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AnnualPlanWithEquipment {
    pub id: Uuid,
    pub year: i32,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_serial_number: Option<String>,
    pub calibration_type: CalibrationType,
    pub cycle: CalibrationCycle,
    pub month_1: bool,
    pub month_2: bool,
    pub month_3: bool,
    pub month_4: bool,
    pub month_5: bool,
    pub month_6: bool,
    pub month_7: bool,
    pub month_8: bool,
    pub month_9: bool,
    pub month_10: bool,
    pub month_11: bool,
    pub month_12: bool,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AnnualPlanQuery {
    pub year: i32,
    pub equipment_id: Option<Uuid>,
    pub calibration_type: Option<CalibrationType>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateAnnualPlanRequest {
    pub year: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnualPlanRequest {
    pub year: i32,
    pub equipment_id: Uuid,
    pub calibration_type: CalibrationType,
    pub cycle: CalibrationCycle,
    pub month_1: bool,
    pub month_2: bool,
    pub month_3: bool,
    pub month_4: bool,
    pub month_5: bool,
    pub month_6: bool,
    pub month_7: bool,
    pub month_8: bool,
    pub month_9: bool,
    pub month_10: bool,
    pub month_11: bool,
    pub month_12: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnnualPlanRequest {
    pub calibration_type: Option<CalibrationType>,
    pub cycle: Option<CalibrationCycle>,
    pub month_1: bool,
    pub month_2: bool,
    pub month_3: bool,
    pub month_4: bool,
    pub month_5: bool,
    pub month_6: bool,
    pub month_7: bool,
    pub month_8: bool,
    pub month_9: bool,
    pub month_10: bool,
    pub month_11: bool,
    pub month_12: bool,
}

// ========== Annual Plan Execution Summary (計畫 vs 實際執行) ==========

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonthExecutionStatus {
    Unplanned,
    PlannedPending,
    Completed,
    Overdue,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthExecutionDetail {
    pub month: i32,
    pub planned: bool,
    pub status: MonthExecutionStatus,
    pub calibration_id: Option<Uuid>,
    pub calibrated_at: Option<chrono::NaiveDate>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnualPlanExecutionRow {
    pub plan_id: Uuid,
    pub year: i32,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_serial_number: Option<String>,
    pub calibration_type: CalibrationType,
    pub cycle: CalibrationCycle,
    pub months: Vec<MonthExecutionDetail>,
    pub planned_count: i32,
    pub completed_count: i32,
    pub overdue_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnualPlanExecutionSummary {
    pub year: i32,
    pub total_planned: i32,
    pub total_completed: i32,
    pub total_overdue: i32,
    pub completion_rate: f64,
    pub rows: Vec<AnnualPlanExecutionRow>,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionSummaryQuery {
    pub year: i32,
    pub equipment_id: Option<Uuid>,
    pub calibration_type: Option<CalibrationType>,
}

// ========== Equipment Timeline (設備履歷) ==========

/// SQL UNION ALL 查詢的中間 struct
#[derive(Debug, FromRow)]
pub struct TimelineRow {
    pub id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub sub_type: Option<String>,
    pub sub_status: Option<String>,
    pub summary: Option<String>,
    pub notes: Option<String>,
    pub actor_name: Option<String>,
}

/// API 回傳的 timeline entry
#[derive(Debug, Clone, Serialize)]
pub struct EquipmentTimelineEntry {
    pub id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub subtitle: Option<String>,
    pub detail: JsonValue,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentHistoryQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
