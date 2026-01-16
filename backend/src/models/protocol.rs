use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// 計畫狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "protocol_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolStatus {
    Draft,
    Submitted,
    PreReview,
    UnderReview,
    RevisionRequired,
    Resubmitted,
    Approved,
    ApprovedWithConditions,
    Deferred,
    Rejected,
    Suspended,
    Closed,
    Deleted,
}

impl ProtocolStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolStatus::Draft => "DRAFT",
            ProtocolStatus::Submitted => "SUBMITTED",
            ProtocolStatus::PreReview => "PRE_REVIEW",
            ProtocolStatus::UnderReview => "UNDER_REVIEW",
            ProtocolStatus::RevisionRequired => "REVISION_REQUIRED",
            ProtocolStatus::Resubmitted => "RESUBMITTED",
            ProtocolStatus::Approved => "APPROVED",
            ProtocolStatus::ApprovedWithConditions => "APPROVED_WITH_CONDITIONS",
            ProtocolStatus::Deferred => "DEFERRED",
            ProtocolStatus::Rejected => "REJECTED",
            ProtocolStatus::Suspended => "SUSPENDED",
            ProtocolStatus::Closed => "CLOSED",
            ProtocolStatus::Deleted => "DELETED",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            ProtocolStatus::Draft => "草稿",
            ProtocolStatus::Submitted => "已提交",
            ProtocolStatus::PreReview => "行政預審",
            ProtocolStatus::UnderReview => "審查中",
            ProtocolStatus::RevisionRequired => "需修訂",
            ProtocolStatus::Resubmitted => "已重送",
            ProtocolStatus::Approved => "已核准",
            ProtocolStatus::ApprovedWithConditions => "附條件核准",
            ProtocolStatus::Deferred => "延後審議",
            ProtocolStatus::Rejected => "已否決",
            ProtocolStatus::Suspended => "已暫停",
            ProtocolStatus::Closed => "已結案",
            ProtocolStatus::Deleted => "已刪除",
        }
    }
}

/// 計畫書主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Protocol {
    pub id: Uuid,
    pub protocol_no: String,
    pub iacuc_no: Option<String>,
    pub title: String,
    pub status: ProtocolStatus,
    pub pi_user_id: Uuid,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 計畫版本快照
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolVersion {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub version_no: i32,
    pub content_snapshot: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: Uuid,
}

/// 計畫狀態歷程
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolStatusHistory {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub from_status: Option<ProtocolStatus>,
    pub to_status: ProtocolStatus,
    pub changed_by: Uuid,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 審查人員指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewAssignment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 審查意見
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewComment {
    pub id: Uuid,
    pub protocol_version_id: Uuid,
    pub reviewer_id: Uuid,
    pub content: String,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 計畫附件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolAttachment {
    pub id: Uuid,
    pub protocol_version_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 計畫中的角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "protocol_role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolRole {
    Pi,
    Client,
    CoEditor,
}

/// 使用者計畫關聯
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProtocol {
    pub user_id: Uuid,
    pub protocol_id: Uuid,
    pub role_in_protocol: ProtocolRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProtocolRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: String,
    pub pi_user_id: Option<Uuid>,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProtocolRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: Option<String>,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusRequest {
    pub to_status: ProtocolStatus,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignReviewerRequest {
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AssignCoEditorRequest {
    pub protocol_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    pub protocol_version_id: Uuid,
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReplyCommentRequest {
    pub parent_comment_id: Uuid,
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ProtocolQuery {
    pub status: Option<ProtocolStatus>,
    pub pi_user_id: Option<Uuid>,
    pub keyword: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// 計畫書回應（含關聯資訊）
#[derive(Debug, Serialize)]
pub struct ProtocolResponse {
    #[serde(flatten)]
    pub protocol: Protocol,
    pub pi_name: Option<String>,
    pub pi_email: Option<String>,
    pub pi_organization: Option<String>,
    pub status_display: String,
}

/// 計畫列表項目
#[derive(Debug, Serialize, FromRow)]
pub struct ProtocolListItem {
    pub id: Uuid,
    pub protocol_no: String,
    pub iacuc_no: Option<String>,
    pub title: String,
    pub status: ProtocolStatus,
    pub pi_user_id: Uuid,
    pub pi_name: String,
    pub pi_organization: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub apply_study_number: Option<String>,
}

/// 審查意見回應（含審查者資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct ReviewCommentResponse {
    pub id: Uuid,
    pub protocol_version_id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewer_email: String,
    pub content: String,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    #[sqlx(default)]
    pub replied_by_name: Option<String>,
    #[sqlx(default)]
    pub replied_by_email: Option<String>,
    pub created_at: DateTime<Utc>,
}
