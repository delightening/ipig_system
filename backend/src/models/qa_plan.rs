// QA 計畫管理 Models（稽查報告、不符合事項、SOP 文件、稽查排程）

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

// ========== Enums ==========

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_inspection_type", rename_all = "snake_case")]
pub enum QaInspectionType {
    #[serde(rename = "protocol")]
    Protocol,
    #[serde(rename = "equipment")]
    Equipment,
    #[serde(rename = "facility")]
    Facility,
    #[serde(rename = "training")]
    Training,
    #[serde(rename = "general")]
    General,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_inspection_status", rename_all = "snake_case")]
pub enum QaInspectionStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "submitted")]
    Submitted,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_item_result", rename_all = "snake_case")]
pub enum QaItemResult {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "not_applicable")]
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "nc_severity", rename_all = "snake_case")]
pub enum NcSeverity {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "major")]
    Major,
    #[serde(rename = "minor")]
    Minor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "nc_source", rename_all = "snake_case")]
pub enum NcSource {
    #[serde(rename = "inspection")]
    Inspection,
    #[serde(rename = "observation")]
    Observation,
    #[serde(rename = "external_audit")]
    ExternalAudit,
    #[serde(rename = "self_report")]
    SelfReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "nc_status", rename_all = "snake_case")]
pub enum NcStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "pending_verification")]
    PendingVerification,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "capa_action_type", rename_all = "snake_case")]
pub enum CapaActionType {
    #[serde(rename = "corrective")]
    Corrective,
    #[serde(rename = "preventive")]
    Preventive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "capa_status", rename_all = "snake_case")]
pub enum CapaStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "verified")]
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "sop_status", rename_all = "snake_case")]
pub enum SopStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "obsolete")]
    Obsolete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_schedule_type", rename_all = "snake_case")]
pub enum QaScheduleType {
    #[serde(rename = "annual")]
    Annual,
    #[serde(rename = "periodic")]
    Periodic,
    #[serde(rename = "ad_hoc")]
    AdHoc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_schedule_status", rename_all = "snake_case")]
pub enum QaScheduleStatus {
    #[serde(rename = "planned")]
    Planned,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "qa_schedule_item_status", rename_all = "snake_case")]
pub enum QaScheduleItemStatus {
    #[serde(rename = "planned")]
    Planned,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "overdue")]
    Overdue,
}

// ========== 稽查報告 ==========

#[derive(Debug, Serialize, FromRow)]
pub struct QaInspection {
    pub id:                  Uuid,
    pub inspection_number:   String,
    pub title:               String,
    pub inspection_type:     QaInspectionType,
    pub inspection_date:     NaiveDate,
    pub inspector_id:        Uuid,
    pub related_entity_type: Option<String>,
    pub related_entity_id:   Option<Uuid>,
    pub status:              QaInspectionStatus,
    pub findings:            Option<String>,
    pub conclusion:          Option<String>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaInspectionWithInspector {
    pub id:                  Uuid,
    pub inspection_number:   String,
    pub title:               String,
    pub inspection_type:     QaInspectionType,
    pub inspection_date:     NaiveDate,
    pub inspector_id:        Uuid,
    pub inspector_name:      String,
    pub related_entity_type: Option<String>,
    pub related_entity_id:   Option<Uuid>,
    pub status:              QaInspectionStatus,
    pub findings:            Option<String>,
    pub conclusion:          Option<String>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaInspectionItem {
    pub id:            Uuid,
    pub inspection_id: Uuid,
    pub item_order:    i32,
    pub description:   String,
    pub result:        QaItemResult,
    pub remarks:       Option<String>,
    pub created_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct QaInspectionDetail {
    #[serde(flatten)]
    pub inspection: QaInspectionWithInspector,
    pub items:      Vec<QaInspectionItem>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInspectionRequest {
    #[validate(length(min = 1, max = 255))]
    pub title:               String,
    pub inspection_type:     QaInspectionType,
    pub inspection_date:     NaiveDate,
    pub related_entity_type: Option<String>,
    pub related_entity_id:   Option<Uuid>,
    pub findings:            Option<String>,
    pub conclusion:          Option<String>,
    pub items:               Vec<CreateInspectionItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInspectionItemRequest {
    pub item_order:  i32,
    pub description: String,
    pub result:      QaItemResult,
    pub remarks:     Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInspectionRequest {
    #[validate(length(min = 1, max = 255))]
    pub title:       Option<String>,
    pub inspection_date: Option<NaiveDate>,
    pub findings:    Option<String>,
    pub conclusion:  Option<String>,
    pub status:      Option<QaInspectionStatus>,
    pub items:       Option<Vec<CreateInspectionItemRequest>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct InspectionQuery {
    pub page:            Option<i64>,
    pub page_size:       Option<i64>,
    pub inspection_type: Option<String>,
    pub status:          Option<String>,
}

// ========== 不符合事項 ==========

#[derive(Debug, Serialize, FromRow)]
pub struct QaNonConformance {
    pub id:                    Uuid,
    pub nc_number:             String,
    pub title:                 String,
    pub description:           String,
    pub severity:              NcSeverity,
    pub source:                NcSource,
    pub related_inspection_id: Option<Uuid>,
    pub assignee_id:           Option<Uuid>,
    pub due_date:              Option<NaiveDate>,
    pub status:                NcStatus,
    pub root_cause:            Option<String>,
    pub closure_notes:         Option<String>,
    pub closed_at:             Option<DateTime<Utc>>,
    pub created_by:            Uuid,
    pub created_at:            DateTime<Utc>,
    pub updated_at:            DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaNonConformanceWithDetails {
    pub id:                    Uuid,
    pub nc_number:             String,
    pub title:                 String,
    pub description:           String,
    pub severity:              NcSeverity,
    pub source:                NcSource,
    pub related_inspection_id: Option<Uuid>,
    pub assignee_id:           Option<Uuid>,
    pub assignee_name:         Option<String>,
    pub due_date:              Option<NaiveDate>,
    pub status:                NcStatus,
    pub root_cause:            Option<String>,
    pub closure_notes:         Option<String>,
    pub closed_at:             Option<DateTime<Utc>>,
    pub created_by:            Uuid,
    pub creator_name:          String,
    pub created_at:            DateTime<Utc>,
    pub updated_at:            DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaCapa {
    pub id:           Uuid,
    pub nc_id:        Uuid,
    pub action_type:  CapaActionType,
    pub description:  String,
    pub assignee_id:  Option<Uuid>,
    pub due_date:     Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status:       CapaStatus,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NcDetail {
    #[serde(flatten)]
    pub nc:   QaNonConformanceWithDetails,
    pub capa: Vec<QaCapa>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNcRequest {
    #[validate(length(min = 1, max = 255))]
    pub title:                 String,
    pub description:           String,
    pub severity:              NcSeverity,
    pub source:                NcSource,
    pub related_inspection_id: Option<Uuid>,
    pub assignee_id:           Option<Uuid>,
    pub due_date:              Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNcRequest {
    pub title:         Option<String>,
    pub description:   Option<String>,
    pub assignee_id:   Option<Uuid>,
    pub due_date:      Option<NaiveDate>,
    pub status:        Option<NcStatus>,
    pub root_cause:    Option<String>,
    pub closure_notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCapaRequest {
    pub action_type: CapaActionType,
    #[validate(length(min = 1))]
    pub description: String,
    pub assignee_id: Option<Uuid>,
    pub due_date:    Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCapaRequest {
    pub description:  Option<String>,
    pub assignee_id:  Option<Uuid>,
    pub due_date:     Option<NaiveDate>,
    pub status:       Option<CapaStatus>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NcQuery {
    pub page:      Option<i64>,
    pub page_size: Option<i64>,
    pub severity:  Option<String>,
    pub status:    Option<String>,
}

// ========== SOP 文件 ==========

#[derive(Debug, Serialize, FromRow)]
pub struct QaSopDocument {
    pub id:              Uuid,
    pub document_number: String,
    pub title:           String,
    pub version:         String,
    pub category:        Option<String>,
    pub file_path:       Option<String>,
    pub effective_date:  Option<NaiveDate>,
    pub review_date:     Option<NaiveDate>,
    pub status:          SopStatus,
    pub description:     Option<String>,
    pub created_by:      Uuid,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaSopDocumentWithAck {
    pub id:                Uuid,
    pub document_number:   String,
    pub title:             String,
    pub version:           String,
    pub category:          Option<String>,
    pub file_path:         Option<String>,
    pub effective_date:    Option<NaiveDate>,
    pub review_date:       Option<NaiveDate>,
    pub status:            SopStatus,
    pub description:       Option<String>,
    pub created_by:        Uuid,
    pub creator_name:      String,
    pub acknowledged_by_me: bool,
    pub ack_count:         i64,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSopRequest {
    #[validate(length(min = 1, max = 255))]
    pub title:          String,
    #[validate(length(min = 1, max = 20))]
    pub version:        String,
    pub category:       Option<String>,
    pub file_path:      Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub review_date:    Option<NaiveDate>,
    pub description:    Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSopRequest {
    pub title:          Option<String>,
    pub version:        Option<String>,
    pub category:       Option<String>,
    pub file_path:      Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub review_date:    Option<NaiveDate>,
    pub status:         Option<SopStatus>,
    pub description:    Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SopQuery {
    pub page:      Option<i64>,
    pub page_size: Option<i64>,
    pub status:    Option<String>,
    pub category:  Option<String>,
}

// ========== 稽查排程 ==========

#[derive(Debug, Serialize, FromRow)]
pub struct QaAuditSchedule {
    pub id:            Uuid,
    pub year:          i32,
    pub title:         String,
    pub schedule_type: QaScheduleType,
    pub description:   Option<String>,
    pub status:        QaScheduleStatus,
    pub created_by:    Uuid,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QaScheduleItem {
    pub id:                    Uuid,
    pub schedule_id:           Uuid,
    pub inspection_type:       QaInspectionType,
    pub title:                 String,
    pub planned_date:          NaiveDate,
    pub actual_date:           Option<NaiveDate>,
    pub responsible_person_id: Option<Uuid>,
    pub responsible_name:      Option<String>,
    pub related_inspection_id: Option<Uuid>,
    pub status:                QaScheduleItemStatus,
    pub notes:                 Option<String>,
    pub created_at:            DateTime<Utc>,
    pub updated_at:            DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct QaScheduleDetail {
    #[serde(flatten)]
    pub schedule: QaAuditSchedule,
    pub items:    Vec<QaScheduleItem>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateScheduleRequest {
    pub year:          i32,
    #[validate(length(min = 1, max = 255))]
    pub title:         String,
    pub schedule_type: QaScheduleType,
    pub description:   Option<String>,
    pub items:         Vec<CreateScheduleItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleItemRequest {
    pub inspection_type:       QaInspectionType,
    pub title:                 String,
    pub planned_date:          NaiveDate,
    pub responsible_person_id: Option<Uuid>,
    pub notes:                 Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleItemRequest {
    pub actual_date:           Option<NaiveDate>,
    pub responsible_person_id: Option<Uuid>,
    pub related_inspection_id: Option<Uuid>,
    pub status:                Option<QaScheduleItemStatus>,
    pub notes:                 Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ScheduleQuery {
    pub year:   Option<i32>,
    pub status: Option<String>,
}
