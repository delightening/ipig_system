use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 稽核日誌
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
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
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

/// 請求上下文資訊（用於稽核）
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
