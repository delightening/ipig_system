// Google Calendar 同步 Models
// 包含：Config, EventSync, Conflict, SyncHistory

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Calendar Config (系統設定)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GoogleCalendarConfig {
    pub id: Uuid,
    pub calendar_id: String,
    pub calendar_name: Option<String>,
    pub calendar_description: Option<String>,
    pub auth_method: String,
    pub auth_email: Option<String>,
    pub is_configured: bool,
    pub sync_enabled: bool,
    pub sync_schedule_morning: Option<NaiveTime>,
    pub sync_schedule_evening: Option<NaiveTime>,
    pub sync_timezone: Option<String>,
    pub sync_approved_leaves: bool,
    pub sync_overtime: bool,
    pub event_title_template: Option<String>,
    pub event_color_id: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub last_sync_events_pushed: Option<i32>,
    pub last_sync_events_pulled: Option<i32>,
    pub last_sync_conflicts: Option<i32>,
    pub last_sync_duration_ms: Option<i32>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCalendarConfigRequest {
    pub calendar_id: Option<String>,
    pub calendar_name: Option<String>,
    pub auth_email: Option<String>,
    pub sync_enabled: Option<bool>,
    pub sync_schedule_morning: Option<NaiveTime>,
    pub sync_schedule_evening: Option<NaiveTime>,
    pub sync_approved_leaves: Option<bool>,
    pub sync_overtime: Option<bool>,
    pub event_title_template: Option<String>,
    pub event_color_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectCalendarRequest {
    pub calendar_id: String,
    pub auth_email: String,
    // Password 不透過 API 傳遞，應透過環境變數設定
}

// ============================================
// Event Sync (事件同步狀態)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CalendarEventSync {
    pub id: Uuid,
    pub leave_request_id: Uuid,
    pub google_event_id: Option<String>,
    pub google_event_etag: Option<String>,
    pub google_event_link: Option<String>,
    pub sync_version: i32,
    pub local_updated_at: DateTime<Utc>,
    pub google_updated_at: Option<DateTime<Utc>>,
    pub last_synced_data: Option<serde_json::Value>,
    pub sync_status: String,
    pub last_error: Option<String>,
    pub error_count: i32,
    pub last_error_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EventSyncWithLeave {
    pub id: Uuid,
    pub leave_request_id: Uuid,
    pub user_name: String,
    pub leave_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub google_event_id: Option<String>,
    pub sync_status: String,
    pub last_error: Option<String>,
    pub error_count: i32,
}

// ============================================
// Sync Conflicts (同步衝突)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CalendarSyncConflict {
    pub id: Uuid,
    pub calendar_event_sync_id: Option<Uuid>,
    pub leave_request_id: Option<Uuid>,
    pub conflict_type: String,
    pub ipig_data: serde_json::Value,
    pub google_data: Option<serde_json::Value>,
    pub difference_summary: Option<String>,
    pub status: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    pub requires_new_approval: bool,
    pub new_approval_request_id: Option<Uuid>,
    pub detected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ConflictWithDetails {
    pub id: Uuid,
    pub leave_request_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub leave_type: Option<String>,
    pub conflict_type: String,
    pub difference_summary: Option<String>,
    pub status: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: String, // "keep_ipig", "accept_google", "dismiss"
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConflictQuery {
    pub status: Option<String>,
    pub leave_request_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ============================================
// Sync History (同步歷史)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CalendarSyncHistory {
    pub id: Uuid,
    pub job_type: String,
    pub triggered_by: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub events_created: i32,
    pub events_updated: i32,
    pub events_deleted: i32,
    pub events_checked: i32,
    pub conflicts_detected: i32,
    pub errors_count: i32,
    pub error_messages: Option<serde_json::Value>,
    pub progress_percentage: i32,
    pub current_operation: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SyncHistoryQuery {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ============================================
// Sync Status Response
// ============================================

#[derive(Debug, Serialize)]
pub struct CalendarSyncStatus {
    pub is_configured: bool,
    pub sync_enabled: bool,
    pub calendar_id: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub pending_syncs: i64,
    pub pending_conflicts: i64,
    pub recent_errors: i64,
}
