// 擴展的審計相關 Models
// 包含：UserActivityLog, LoginEvent, UserSession, SecurityAlert

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 稽核日誌（原有，保留相容）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_user_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditLogWithActor {
    pub id: Uuid,
    pub actor_user_id: Uuid,
    pub actor_email: String,
    pub actor_name: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_email: Option<String>,
    pub entity_name: Option<String>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 使用者活動日誌（新增的詳細版本）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserActivityLog {
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_display_name: Option<String>,
    pub actor_roles: Option<serde_json::Value>,
    pub session_id: Option<Uuid>,
    pub event_category: String,
    pub event_type: String,
    pub event_severity: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub entity_display_name: Option<String>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_fields: Option<Vec<String>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub response_status: Option<i32>,
    pub is_suspicious: bool,
    pub suspicious_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub partition_date: NaiveDate,
}

/// 登入事件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginEvent {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub event_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub geo_country: Option<String>,
    pub geo_city: Option<String>,
    pub geo_timezone: Option<String>,
    pub is_unusual_time: bool,
    pub is_unusual_location: bool,
    pub is_new_device: bool,
    pub is_mass_login: bool,
    pub device_fingerprint: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 使用者 Session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub refresh_token_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub page_view_count: i32,
    pub action_count: i32,
    pub is_active: bool,
    pub ended_reason: Option<String>,
}

/// 安全警報
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub user_id: Option<Uuid>,
    pub activity_log_id: Option<Uuid>,
    pub login_event_id: Option<Uuid>,
    pub context_data: Option<serde_json::Value>,
    pub status: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 使用者活動聚合（每日統計）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserActivityAggregate {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub aggregate_date: NaiveDate,
    pub login_count: i32,
    pub failed_login_count: i32,
    pub session_count: i32,
    pub total_session_minutes: i32,
    pub page_view_count: i32,
    pub action_count: i32,
    pub actions_by_category: Option<serde_json::Value>,
    pub pages_visited: Option<serde_json::Value>,
    pub entities_modified: Option<serde_json::Value>,
    pub unique_ip_count: i32,
    pub unusual_activity_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 稽核動作類型
#[derive(Debug, Clone, Copy)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Submit,
    Approve,
    Cancel,
    Login,
    Logout,
    PasswordReset,
    PasswordChange,
    StatusChange,
    Assign,
    Unassign,
    Reject,
    Impersonate,
    StopImpersonate,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "CREATE",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
            AuditAction::Submit => "SUBMIT",
            AuditAction::Approve => "APPROVE",
            AuditAction::Cancel => "CANCEL",
            AuditAction::Login => "LOGIN",
            AuditAction::Logout => "LOGOUT",
            AuditAction::PasswordReset => "PASSWORD_RESET",
            AuditAction::PasswordChange => "PASSWORD_CHANGE",
            AuditAction::StatusChange => "STATUS_CHANGE",
            AuditAction::Assign => "ASSIGN",
            AuditAction::Unassign => "UNASSIGN",
            AuditAction::Reject => "REJECT",
            AuditAction::Impersonate => "IMPERSONATE",
            AuditAction::StopImpersonate => "STOP_IMPERSONATE",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub actor_user_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

/// 活動日誌查詢參數
#[derive(Debug, Deserialize)]
pub struct ActivityLogQuery {
    pub user_id: Option<Uuid>,
    pub event_category: Option<String>,
    pub event_type: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub is_suspicious: Option<bool>,
    /// R30-14: 自由文字搜尋（ILIKE 比對 entity_display_name / actor_display_name /
    /// actor_email / event_type / ip_address），長度上限 100 字
    pub query: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 登入事件查詢參數
#[derive(Debug, Deserialize)]
pub struct LoginEventQuery {
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub is_unusual: Option<bool>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Session 查詢參數
#[derive(Debug, Deserialize)]
pub struct SessionQuery {
    pub user_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 安全警報查詢參數
#[derive(Debug, Deserialize)]
pub struct SecurityAlertQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub user_id: Option<Uuid>,
    pub query: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 請求上下文資訊（用於稽核）
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// 解析安全警報的請求
#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    pub resolution: String,
    pub resolution_notes: Option<String>,
}

/// 警報所指對象目前的封鎖/鎖定狀態（供 admin 在警報詳情直接解除）
#[derive(Debug, Serialize)]
pub struct AlertLockStatus {
    /// 警報所指的帳號；email 解析不出來（如 CSP 報告）時為 None
    pub account: Option<AccountLockoutStatus>,
    /// 與此警報相關且仍生效的 IP 封鎖；沒有則為 None
    pub ip: Option<IpBlockStatus>,
}

/// 帳號鎖定狀態。
///
/// 鎖定沒有持久旗標——是「視窗內失敗次數 >= 門檻」的現算結果，
/// 故 `unlock_at` 為推算的自動解鎖時刻（最舊那筆計入的失敗滑出視窗的時間）。
#[derive(Debug, Serialize)]
pub struct AccountLockoutStatus {
    pub email: String,
    /// email 是否對應到實際帳號。枚舉攻擊產生的警報會是 false（無帳號可解鎖）
    pub user_exists: bool,
    pub locked: bool,
    pub fail_count: i64,
    pub max_attempts: i64,
    pub unlock_at: Option<DateTime<Utc>>,
    /// 上次解鎖標記的時間（密碼變更或管理員手動解鎖，取較晚者）
    pub cleared_at: Option<DateTime<Utc>>,
}

/// 仍生效的 IP 封鎖項目（`ip_address` 已轉為純文字，不含 CIDR 後綴）
#[derive(Debug, Serialize, FromRow)]
pub struct IpBlockStatus {
    pub id: Uuid,
    pub ip_address: String,
    pub reason: String,
    pub source: String,
    pub blocked_until: Option<DateTime<Utc>>,
}

/// 解除帳號鎖定的請求
#[derive(Debug, Deserialize)]
pub struct ClearAccountLockoutRequest {
    pub email: String,
    pub reason: String,
}

/// 批次解析安全警報的請求
#[derive(Debug, Deserialize)]
pub struct BulkResolveAlertsRequest {
    pub ids: Vec<uuid::Uuid>,
    pub resolution_notes: Option<String>,
}

/// 強制登出請求
#[derive(Debug, Deserialize)]
pub struct ForceLogoutRequest {
    pub reason: Option<String>,
}

/// Session 附帶使用者資訊（用於顯示）
#[derive(Debug, Serialize, FromRow)]
pub struct SessionWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub page_view_count: i32,
    pub action_count: i32,
    pub is_active: bool,
    pub ended_reason: Option<String>,
}

/// 登入事件附帶使用者資訊
#[derive(Debug, Serialize, FromRow)]
pub struct LoginEventWithUser {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub user_name: Option<String>,
    pub event_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub is_unusual_time: bool,
    pub is_unusual_location: bool,
    pub is_new_device: bool,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 審計儀表板統計
#[derive(Debug, Serialize)]
pub struct AuditDashboardStats {
    pub active_users_today: i64,
    pub active_users_week: i64,
    pub active_users_month: i64,
    pub total_logins_today: i64,
    pub failed_logins_today: i64,
    pub active_sessions: i64,
    pub open_alerts: i64,
    pub critical_alerts: i64,
}
