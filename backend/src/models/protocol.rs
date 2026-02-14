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
    PreReviewRevisionRequired,
    VetReview,
    VetRevisionRequired,
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
            ProtocolStatus::PreReviewRevisionRequired => "PRE_REVIEW_REVISION_REQUIRED",
            ProtocolStatus::VetReview => "VET_REVIEW",
            ProtocolStatus::VetRevisionRequired => "VET_REVISION_REQUIRED",
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
            ProtocolStatus::PreReviewRevisionRequired => "行政預審補件",
            ProtocolStatus::VetReview => "獸醫審查",
            ProtocolStatus::VetRevisionRequired => "獸醫要求修訂",
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

/// 計畫活動類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "protocol_activity_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolActivityType {
    // 生命週期
    Created,
    Updated,
    Submitted,
    Resubmitted,
    Approved,
    ApprovedWithConditions,
    Closed,
    Rejected,
    Suspended,
    Deleted,
    // 審查流程
    StatusChanged,
    ReviewerAssigned,
    VetAssigned,
    CoeditorAssigned,
    CoeditorRemoved,
    // 審查意見
    CommentAdded,
    CommentReplied,
    CommentResolved,
    // 附件
    AttachmentUploaded,
    AttachmentDeleted,
    // 版本
    VersionCreated,
    VersionRecovered,
    // 修正案
    AmendmentCreated,
    AmendmentSubmitted,
    // 動物管理
    PigAssigned,
    PigUnassigned,
}

impl ProtocolActivityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolActivityType::Created => "CREATED",
            ProtocolActivityType::Updated => "UPDATED",
            ProtocolActivityType::Submitted => "SUBMITTED",
            ProtocolActivityType::Resubmitted => "RESUBMITTED",
            ProtocolActivityType::Approved => "APPROVED",
            ProtocolActivityType::ApprovedWithConditions => "APPROVED_WITH_CONDITIONS",
            ProtocolActivityType::Closed => "CLOSED",
            ProtocolActivityType::Rejected => "REJECTED",
            ProtocolActivityType::Suspended => "SUSPENDED",
            ProtocolActivityType::Deleted => "DELETED",
            ProtocolActivityType::StatusChanged => "STATUS_CHANGED",
            ProtocolActivityType::ReviewerAssigned => "REVIEWER_ASSIGNED",
            ProtocolActivityType::VetAssigned => "VET_ASSIGNED",
            ProtocolActivityType::CoeditorAssigned => "COEDITOR_ASSIGNED",
            ProtocolActivityType::CoeditorRemoved => "COEDITOR_REMOVED",
            ProtocolActivityType::CommentAdded => "COMMENT_ADDED",
            ProtocolActivityType::CommentReplied => "COMMENT_REPLIED",
            ProtocolActivityType::CommentResolved => "COMMENT_RESOLVED",
            ProtocolActivityType::AttachmentUploaded => "ATTACHMENT_UPLOADED",
            ProtocolActivityType::AttachmentDeleted => "ATTACHMENT_DELETED",
            ProtocolActivityType::VersionCreated => "VERSION_CREATED",
            ProtocolActivityType::VersionRecovered => "VERSION_RECOVERED",
            ProtocolActivityType::AmendmentCreated => "AMENDMENT_CREATED",
            ProtocolActivityType::AmendmentSubmitted => "AMENDMENT_SUBMITTED",
            ProtocolActivityType::PigAssigned => "PIG_ASSIGNED",
            ProtocolActivityType::PigUnassigned => "PIG_UNASSIGNED",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            ProtocolActivityType::Created => "創建草稿",
            ProtocolActivityType::Updated => "編輯計畫",
            ProtocolActivityType::Submitted => "送審",
            ProtocolActivityType::Resubmitted => "重新送審",
            ProtocolActivityType::Approved => "通過",
            ProtocolActivityType::ApprovedWithConditions => "附條件通過",
            ProtocolActivityType::Closed => "結案",
            ProtocolActivityType::Rejected => "否決",
            ProtocolActivityType::Suspended => "暫停",
            ProtocolActivityType::Deleted => "刪除",
            ProtocolActivityType::StatusChanged => "狀態變更",
            ProtocolActivityType::ReviewerAssigned => "指派審查委員",
            ProtocolActivityType::VetAssigned => "指派獸醫師",
            ProtocolActivityType::CoeditorAssigned => "指派共同編輯者",
            ProtocolActivityType::CoeditorRemoved => "移除共同編輯者",
            ProtocolActivityType::CommentAdded => "新增審查意見",
            ProtocolActivityType::CommentReplied => "回覆審查意見",
            ProtocolActivityType::CommentResolved => "解決審查意見",
            ProtocolActivityType::AttachmentUploaded => "上傳附件",
            ProtocolActivityType::AttachmentDeleted => "刪除附件",
            ProtocolActivityType::VersionCreated => "建立版本快照",
            ProtocolActivityType::VersionRecovered => "回復至版本",
            ProtocolActivityType::AmendmentCreated => "建立修正案",
            ProtocolActivityType::AmendmentSubmitted => "送審修正案",
            ProtocolActivityType::PigAssigned => "分配動物",
            ProtocolActivityType::PigUnassigned => "移除動物",
        }
    }
}

/// 計畫活動歷程
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolActivity {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub activity_type: ProtocolActivityType,
    pub actor_id: Uuid,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub from_value: Option<String>,
    pub to_value: Option<String>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<Uuid>,
    pub target_entity_name: Option<String>,
    pub remark: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 計畫活動歷程回應（用於 API）
#[derive(Debug, Clone, Serialize)]
pub struct ProtocolActivityResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub activity_type: ProtocolActivityType,
    pub activity_type_display: String,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub actor_email: String,
    pub from_value: Option<String>,
    pub to_value: Option<String>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<Uuid>,
    pub target_entity_name: Option<String>,
    pub remark: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<ProtocolActivity> for ProtocolActivityResponse {
    fn from(activity: ProtocolActivity) -> Self {
        Self {
            id: activity.id,
            protocol_id: activity.protocol_id,
            activity_type: activity.activity_type,
            activity_type_display: activity.activity_type.display_name().to_string(),
            actor_id: activity.actor_id,
            actor_name: activity.actor_name.unwrap_or_default(),
            actor_email: activity.actor_email.unwrap_or_default(),
            from_value: activity.from_value,
            to_value: activity.to_value,
            target_entity_type: activity.target_entity_type,
            target_entity_id: activity.target_entity_id,
            target_entity_name: activity.target_entity_name,
            remark: activity.remark,
            extra_data: activity.extra_data,
            created_at: activity.created_at,
        }
    }
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
    /// 是否為正式審查委員（可撰寫意見，限 2-3 位）
    #[sqlx(default)]
    pub is_primary_reviewer: bool,
    /// 審查階段
    #[sqlx(default)]
    pub review_stage: Option<String>,
}

/// 審查意見
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewComment {
    pub id: Uuid,
    #[sqlx(default)]
    pub protocol_version_id: Option<Uuid>,
    pub reviewer_id: Uuid,
    pub content: String,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    /// 草稿回覆內容（僅 PI/Coeditor 可見）
    pub draft_content: Option<String>,
    /// 草稿撰寫者
    pub drafted_by: Option<Uuid>,
    /// 草稿最後更新時間
    pub draft_updated_at: Option<DateTime<Utc>>,
    /// 直接關聯的計畫 ID（用於預審階段）
    #[sqlx(rename = "protocol_id")]
    pub protocol_id: Option<Uuid>,
    /// 審查階段（PRE_REVIEW, VET_REVIEW, UNDER_REVIEW）
    #[sqlx(default, rename = "review_stage")]
    pub review_stage: Option<String>,
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

/// 獸醫審查指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetReviewAssignment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub vet_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub decision: Option<String>,
    pub decision_remark: Option<String>,
    pub review_form: Option<serde_json::Value>,
}

/// 獸醫審查查檢項
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetReviewItem {
    pub item_name: String,
    pub compliance: String, // V, X, -
    pub comment: Option<String>,
    pub pi_reply: Option<String>,
}

/// 獸醫審查表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetReviewForm {
    pub items: Vec<VetReviewItem>,
    pub vet_signature: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
}

/// 系統設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
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
    /// 審查委員 ID 列表（當目標狀態為 UNDER_REVIEW 時必填 2-3 位）
    pub reviewer_ids: Option<Vec<Uuid>>,
    /// 獸醫師 ID（當目標狀態為 VET_REVIEW 時可選，未設定則使用預設獸醫）
    pub vet_id: Option<Uuid>,
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

/// 儲存草稿回覆請求
#[derive(Debug, Deserialize, Validate)]
pub struct SaveDraftRequest {
    pub comment_id: Uuid,
    #[validate(length(min = 1, message = "Draft content is required"))]
    pub draft_content: String,
}

/// 送出回覆請求（將草稿正式送出）
#[derive(Debug, Deserialize)]
pub struct SubmitReplyRequest {
    pub comment_id: Uuid,
}

/// 儲存獸醫審查表請求
#[derive(Debug, Deserialize)]
pub struct SaveVetReviewFormRequest {
    pub protocol_id: Uuid,
    pub review_form: serde_json::Value,
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
    pub protocol: Protocol,
    pub pi_name: Option<String>,
    pub pi_email: Option<String>,
    pub pi_organization: Option<String>,
    pub status_display: String,
    pub vet_review: Option<VetReviewAssignment>,
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
    #[sqlx(rename = "protocol_id")]
    pub protocol_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    #[sqlx(default)]
    pub replied_by_name: Option<String>,
    #[sqlx(default)]
    pub replied_by_email: Option<String>,
    /// 草稿回覆內容（僅 PI/Coeditor 可見，審查委員不可見）
    #[sqlx(default)]
    pub draft_content: Option<String>,
    /// 草稿撰寫者
    #[sqlx(default)]
    pub drafted_by: Option<Uuid>,
    /// 草稿撰寫者姓名
    #[sqlx(default)]
    pub drafted_by_name: Option<String>,
    /// 草稿最後更新時間
    #[sqlx(default)]
    pub draft_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 審查指派回應（含審查者與指派者資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct ReviewAssignmentResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewer_email: String,
    pub assigned_by: Uuid,
    pub assigned_by_name: String,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub is_primary_reviewer: bool,
    #[sqlx(default)]
    pub review_stage: Option<String>,
}

/// Co-Editor 指派回應（含用戶資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct CoEditorAssignmentResponse {
    pub user_id: Uuid,
    pub protocol_id: Uuid,
    pub role_in_protocol: ProtocolRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
    #[sqlx(default)]
    pub user_name: String,
    #[sqlx(default)]
    pub user_email: String,
    #[sqlx(default)]
    pub granted_by_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_status_as_str() {
        assert_eq!(ProtocolStatus::Draft.as_str(), "DRAFT");
        assert_eq!(ProtocolStatus::Submitted.as_str(), "SUBMITTED");
        assert_eq!(ProtocolStatus::PreReview.as_str(), "PRE_REVIEW");
        assert_eq!(ProtocolStatus::VetReview.as_str(), "VET_REVIEW");
        assert_eq!(ProtocolStatus::UnderReview.as_str(), "UNDER_REVIEW");
        assert_eq!(ProtocolStatus::Approved.as_str(), "APPROVED");
        assert_eq!(ProtocolStatus::ApprovedWithConditions.as_str(), "APPROVED_WITH_CONDITIONS");
        assert_eq!(ProtocolStatus::Rejected.as_str(), "REJECTED");
        assert_eq!(ProtocolStatus::Closed.as_str(), "CLOSED");
        assert_eq!(ProtocolStatus::Deleted.as_str(), "DELETED");
    }

    #[test]
    fn test_protocol_status_display_name() {
        assert_eq!(ProtocolStatus::Draft.display_name(), "草稿");
        assert_eq!(ProtocolStatus::Approved.display_name(), "已核准");
        assert_eq!(ProtocolStatus::ApprovedWithConditions.display_name(), "附條件核准");
        assert_eq!(ProtocolStatus::UnderReview.display_name(), "審查中");
        assert_eq!(ProtocolStatus::Suspended.display_name(), "已暫停");
    }

    #[test]
    fn test_protocol_status_all_variants_have_as_str() {
        // 確認所有 16 個變體都有對應字串
        let variants = vec![
            ProtocolStatus::Draft, ProtocolStatus::Submitted,
            ProtocolStatus::PreReview, ProtocolStatus::PreReviewRevisionRequired,
            ProtocolStatus::VetReview, ProtocolStatus::VetRevisionRequired,
            ProtocolStatus::UnderReview, ProtocolStatus::RevisionRequired,
            ProtocolStatus::Resubmitted, ProtocolStatus::Approved,
            ProtocolStatus::ApprovedWithConditions, ProtocolStatus::Deferred,
            ProtocolStatus::Rejected, ProtocolStatus::Suspended,
            ProtocolStatus::Closed, ProtocolStatus::Deleted,
        ];
        for v in &variants {
            assert!(!v.as_str().is_empty());
            assert!(!v.display_name().is_empty());
        }
        assert_eq!(variants.len(), 16);
    }

    #[test]
    fn test_activity_type_as_str() {
        assert_eq!(ProtocolActivityType::Created.as_str(), "CREATED");
        assert_eq!(ProtocolActivityType::Submitted.as_str(), "SUBMITTED");
        assert_eq!(ProtocolActivityType::ReviewerAssigned.as_str(), "REVIEWER_ASSIGNED");
        assert_eq!(ProtocolActivityType::PigAssigned.as_str(), "PIG_ASSIGNED");
        assert_eq!(ProtocolActivityType::AmendmentCreated.as_str(), "AMENDMENT_CREATED");
    }

    #[test]
    fn test_activity_type_display_name() {
        assert_eq!(ProtocolActivityType::Created.display_name(), "創建草稿");
        assert_eq!(ProtocolActivityType::Submitted.display_name(), "送審");
        assert_eq!(ProtocolActivityType::ReviewerAssigned.display_name(), "指派審查委員");
        assert_eq!(ProtocolActivityType::PigAssigned.display_name(), "分配動物");
    }

    #[test]
    fn test_activity_type_all_variants() {
        // 確認所有 20 個變體都有對應字串
        let variants = vec![
            ProtocolActivityType::Created, ProtocolActivityType::Updated,
            ProtocolActivityType::Submitted, ProtocolActivityType::Resubmitted,
            ProtocolActivityType::Approved, ProtocolActivityType::ApprovedWithConditions,
            ProtocolActivityType::Closed, ProtocolActivityType::Rejected,
            ProtocolActivityType::Suspended, ProtocolActivityType::Deleted,
            ProtocolActivityType::StatusChanged, ProtocolActivityType::ReviewerAssigned,
            ProtocolActivityType::VetAssigned, ProtocolActivityType::CoeditorAssigned,
            ProtocolActivityType::CoeditorRemoved, ProtocolActivityType::CommentAdded,
            ProtocolActivityType::CommentReplied, ProtocolActivityType::CommentResolved,
            ProtocolActivityType::AttachmentUploaded, ProtocolActivityType::AttachmentDeleted,
            ProtocolActivityType::VersionCreated, ProtocolActivityType::VersionRecovered,
            ProtocolActivityType::AmendmentCreated, ProtocolActivityType::AmendmentSubmitted,
            ProtocolActivityType::PigAssigned, ProtocolActivityType::PigUnassigned,
        ];
        for v in &variants {
            assert!(!v.as_str().is_empty());
            assert!(!v.display_name().is_empty());
        }
        assert_eq!(variants.len(), 26);
    }

    #[test]
    fn test_protocol_activity_response_from() {
        use chrono::Utc;
        let activity = ProtocolActivity {
            id: Uuid::new_v4(),
            protocol_id: Uuid::new_v4(),
            activity_type: ProtocolActivityType::Created,
            actor_id: Uuid::new_v4(),
            actor_name: Some("測試用戶".to_string()),
            actor_email: Some("test@example.com".to_string()),
            from_value: None,
            to_value: Some("DRAFT".to_string()),
            target_entity_type: None,
            target_entity_id: None,
            target_entity_name: None,
            remark: Some("建立計畫".to_string()),
            extra_data: None,
            created_at: Utc::now(),
        };
        let resp = ProtocolActivityResponse::from(activity);
        assert_eq!(resp.activity_type_display, "創建草稿");
        assert_eq!(resp.actor_name, "測試用戶");
    }

    #[test]
    fn test_protocol_status_serde_roundtrip() {
        let status = ProtocolStatus::ApprovedWithConditions;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"APPROVED_WITH_CONDITIONS\"");
        let parsed: ProtocolStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}