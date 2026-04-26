use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// 變更申請類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "amendment_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmendmentType {
    Major,   // 重大變更
    Minor,   // 小變更
    Pending, // 待分類
}

impl AmendmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmendmentType::Major => "MAJOR",
            AmendmentType::Minor => "MINOR",
            AmendmentType::Pending => "PENDING",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AmendmentType::Major => "重大變更",
            AmendmentType::Minor => "小變更",
            AmendmentType::Pending => "待分類",
        }
    }
}

/// 變更申請狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "amendment_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmendmentStatus {
    Draft,
    Submitted,
    Classified,
    UnderReview,
    RevisionRequired,
    Resubmitted,
    Approved,
    Rejected,
    AdminApproved,
}

impl AmendmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmendmentStatus::Draft => "DRAFT",
            AmendmentStatus::Submitted => "SUBMITTED",
            AmendmentStatus::Classified => "CLASSIFIED",
            AmendmentStatus::UnderReview => "UNDER_REVIEW",
            AmendmentStatus::RevisionRequired => "REVISION_REQUIRED",
            AmendmentStatus::Resubmitted => "RESUBMITTED",
            AmendmentStatus::Approved => "APPROVED",
            AmendmentStatus::Rejected => "REJECTED",
            AmendmentStatus::AdminApproved => "ADMIN_APPROVED",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AmendmentStatus::Draft => "草稿",
            AmendmentStatus::Submitted => "已提交",
            AmendmentStatus::Classified => "已分類",
            AmendmentStatus::UnderReview => "審查中",
            AmendmentStatus::RevisionRequired => "需修訂",
            AmendmentStatus::Resubmitted => "已重送",
            AmendmentStatus::Approved => "已核准",
            AmendmentStatus::Rejected => "已否決",
            AmendmentStatus::AdminApproved => "行政核准",
        }
    }
}

/// 變更申請主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Amendment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub amendment_no: String,
    pub revision_number: i32,
    pub amendment_type: AmendmentType,
    pub status: AmendmentStatus,
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub classified_by: Option<Uuid>,
    pub classified_at: Option<DateTime<Utc>>,
    pub classification_remark: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// C2 (GLP §11.50/§11.70)：核准決定的電子簽章 FK。NOT NULL 後 update 被拒。
    pub approved_signature_id: Option<Uuid>,
    /// C2 (GLP §11.50/§11.70)：否決決定的電子簽章 FK。
    pub rejected_signature_id: Option<Uuid>,
}

/// 變更申請版本
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentVersion {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub version_no: i32,
    pub content_snapshot: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: Uuid,
}

/// 變更申請審查指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentReviewAssignment {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub reviewer_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub decision: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
}

/// 變更申請狀態歷程
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentStatusHistory {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub from_status: Option<AmendmentStatus>,
    pub to_status: AmendmentStatus,
    pub changed_by: Uuid,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Request/Response DTOs
// ============================================

/// 建立變更申請請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAmendmentRequest {
    pub protocol_id: Uuid,
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
}

/// 更新變更申請請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAmendmentRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
}

/// 分類變更申請請求
#[derive(Debug, Deserialize)]
pub struct ClassifyAmendmentRequest {
    pub amendment_type: AmendmentType,
    pub remark: Option<String>,
}

/// 變更狀態請求
#[derive(Debug, Deserialize)]
pub struct ChangeAmendmentStatusRequest {
    pub to_status: AmendmentStatus,
    pub remark: Option<String>,
}

/// 記錄審查決定請求
#[derive(Debug, Deserialize, Validate)]
pub struct RecordAmendmentDecisionRequest {
    pub decision: String, // APPROVE, REJECT, REVISION
    #[validate(length(min = 1, message = "Comment is required for decision"))]
    pub comment: Option<String>,
}

/// 變更申請查詢
#[derive(Debug, Deserialize)]
pub struct AmendmentQuery {
    pub protocol_id: Option<Uuid>,
    pub status: Option<AmendmentStatus>,
    pub amendment_type: Option<AmendmentType>,
}

/// 變更申請回應（含關聯資訊）
#[derive(Debug, Serialize)]
pub struct AmendmentResponse {
    #[serde(flatten)]
    pub amendment: Amendment,
    pub protocol_iacuc_no: Option<String>,
    pub protocol_title: Option<String>,
    pub submitted_by_name: Option<String>,
    pub classified_by_name: Option<String>,
    pub status_display: String,
    pub type_display: String,
}

/// 變更申請列表項目
#[derive(Debug, Serialize, FromRow)]
pub struct AmendmentListItem {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub amendment_no: String,
    pub revision_number: i32,
    pub amendment_type: AmendmentType,
    pub status: AmendmentStatus,
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub classified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub protocol_iacuc_no: Option<String>,
    #[sqlx(default)]
    pub protocol_title: Option<String>,
    #[sqlx(default)]
    pub submitted_by_name: Option<String>,
    #[sqlx(default)]
    pub classified_by_name: Option<String>,
}

/// 審查委員指派回應（含用戶資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct AmendmentReviewAssignmentResponse {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub reviewer_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub decision: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    #[sqlx(default)]
    pub reviewer_name: String,
    #[sqlx(default)]
    pub reviewer_email: String,
}

/// 待處理變更申請數量回應
#[derive(Debug, Serialize)]
pub struct PendingCountResponse {
    pub count: i64,
}
