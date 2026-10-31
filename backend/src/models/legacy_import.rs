use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct LegacyImportBatch {
    pub id: Uuid,
    pub source: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
impl crate::models::audit_diff::AuditRedact for LegacyImportBatch {}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct LegacyImportRecord {
    pub id: Uuid,
    pub legacy_source: String,
    pub record_kind: String, // observation/surgery/weight/vaccine/sacrifice
    pub legacy_record_id: String,
    pub legacy_pig_system_id: Option<i32>,
    pub matched_animal_id: Option<Uuid>,
    pub legacy_record_type_text: Option<String>,
    pub original_recorder_name: Option<String>,
    pub mapped_user_id: Option<Uuid>,
    pub legacy_created_at: Option<DateTime<Utc>>,
    pub record_type: Option<String>, // abnormal/experiment/observation for observations
    pub status: String,              // pending/approved/rejected/skipped_duplicate/conflict
    pub payload: serde_json::Value,
    pub import_batch_id: Uuid,
    pub target_record_id: Option<Uuid>,
    #[sqlx(default)]
    pub error_code: Option<String>,
    #[sqlx(default)]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl crate::models::audit_diff::AuditRedact for LegacyImportRecord {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateImportBatchRequest {
    pub source: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LegacyPigDescriptor {
    pub legacy_pig_system_id: Option<i32>,
    pub ear_tag: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub iacuc_no: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LegacyImportItem {
    pub kind: String,             // observation/surgery/weight/vaccine/sacrifice
    pub legacy_record_id: String, // stable key from legacy
    pub legacy_record_type_text: Option<String>, // textual label (e.g. "-")
    pub record_type: Option<String>, // mapped for observations: abnormal/experiment/observation
    pub legacy_created_at: Option<DateTime<Utc>>,
    pub event_date: Option<chrono::NaiveDate>, // observation/surgery/weight/vaccine dates
    pub content: Option<String>,               // observation content
    pub payload: serde_json::Value,            // raw legacy payload (for reports and mapping)
    pub original_recorder_name: Option<String>,
    pub mapped_user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestPigRecordsRequest {
    pub pig: LegacyPigDescriptor,
    /// Up to 500 records in one request
    pub records: Vec<LegacyImportItem>,
    /// If true, validate and stage only; approval always writes to formal tables, not here.
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestPigRecordsResultItem {
    pub legacy_record_id: String,
    pub kind: String,
    pub status: String, // created | skipped | skipped_duplicate | conflict | error
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestPigRecordsResponse {
    pub animal_id: Option<Uuid>,
    pub results: Vec<IngestPigRecordsResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ImportBatchReport {
    pub batch_id: Uuid,
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
    pub skipped_duplicate: i64,
    pub conflict: i64,
    #[serde(default)]
    pub error: i64,
    pub dash_type_records: Vec<Uuid>, // legacy_import_records.id having legacy_record_type_text == "-"
    pub skipped_duplicate_rows: Vec<LegacyReportRow>,
    pub conflict_rows: Vec<LegacyReportRow>,
    pub unmatched_recorder_rows: Vec<LegacyReportRow>,
    pub disabled_account_rows: Vec<LegacyReportRow>,
    #[serde(default)]
    pub error_rows: Vec<LegacyReportRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct LegacyReportRow {
    pub id: Uuid,
    pub legacy_record_id: String,
    pub record_kind: String,
    pub status: String,
}
