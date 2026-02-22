use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// 安樂死單據狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "euthanasia_order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EuthanasiaOrderStatus {
    PendingPi,       // 等待 PI 回應
    Approved,        // PI 同意執行
    Appealed,        // PI 申請暫緩
    ChairArbitration, // CHAIR 仲裁中
    Executed,        // 已執行
    Cancelled,       // 已取消
}

impl EuthanasiaOrderStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            EuthanasiaOrderStatus::PendingPi => "等待 PI 回應",
            EuthanasiaOrderStatus::Approved => "PI 同意執行",
            EuthanasiaOrderStatus::Appealed => "申請暫緩中",
            EuthanasiaOrderStatus::ChairArbitration => "CHAIR 仲裁中",
            EuthanasiaOrderStatus::Executed => "已執行",
            EuthanasiaOrderStatus::Cancelled => "已取消",
        }
    }
}

/// 安樂死單據
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EuthanasiaOrder {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub vet_user_id: Uuid,
    pub pi_user_id: Uuid,
    pub reason: String,
    pub status: EuthanasiaOrderStatus,
    pub deadline_at: DateTime<Utc>,
    pub pi_responded_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 安樂死暫緩申請
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EuthanasiaAppeal {
    pub id: Uuid,
    pub order_id: Uuid,
    pub pi_user_id: Uuid,
    pub reason: String,
    pub attachment_path: Option<String>,
    pub chair_user_id: Option<Uuid>,
    pub chair_decision: Option<String>,
    pub chair_decided_at: Option<DateTime<Utc>>,
    pub chair_deadline_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 審查委員決議
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewerDecision {
    pub id: Uuid,
    pub protocol_version_id: Uuid,
    pub reviewer_id: Uuid,
    pub decision: String,
    pub comment: Option<String>,
    pub decided_at: DateTime<Utc>,
}

/// 全員會議請求
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeetingRequest {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub requested_by: Uuid,
    pub reason: String,
    pub status: String,
    pub meeting_date: Option<DateTime<Utc>>,
    pub chair_decision: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Request/Response DTOs
// ============================================

/// 建立安樂死單據請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEuthanasiaOrderRequest {
    pub animal_id: Uuid,
    #[validate(length(min = 1, max = 2_000, message = "Reason must be 1-2000 characters"))]
    pub reason: String,
}

/// 安樂死暫緩申請請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEuthanasiaAppealRequest {
    #[validate(length(min = 1, max = 2_000, message = "Reason must be 1-2000 characters"))]
    pub reason: String,
    pub attachment_path: Option<String>,
}

/// CHAIR 裁決請求
#[derive(Debug, Deserialize)]
pub struct ChairDecisionRequest {
    pub decision: String,  // 'approve_appeal' or 'reject_appeal'
    pub comment: Option<String>,
}

/// 審查委員決議請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateReviewerDecisionRequest {
    pub protocol_version_id: Uuid,
    #[validate(length(min = 1, max = 100, message = "Decision must be 1-100 characters"))]
    pub decision: String,  // 'approve', 'revision_required', 'reject'
    pub comment: Option<String>,
}

/// 全員會議請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMeetingRequest {
    pub protocol_id: Uuid,
    #[validate(length(min = 1, max = 2_000, message = "Reason must be 1-2000 characters"))]
    pub reason: String,
}

/// 安樂死單據回應（含關聯資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct EuthanasiaOrderResponse {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub vet_user_id: Uuid,
    pub pi_user_id: Uuid,
    pub reason: String,
    pub status: EuthanasiaOrderStatus,
    pub deadline_at: DateTime<Utc>,
    pub pi_responded_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // 關聯資訊
    #[sqlx(default)]
    pub animal_ear_tag: Option<String>,
    #[sqlx(default)]
    pub animal_iacuc_no: Option<String>,
    #[sqlx(default)]
    pub vet_name: Option<String>,
    #[sqlx(default)]
    pub pi_name: Option<String>,
}

/// 安樂死暫緩回應（含關聯資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct EuthanasiaAppealResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub pi_user_id: Uuid,
    pub reason: String,
    pub attachment_path: Option<String>,
    pub chair_user_id: Option<Uuid>,
    pub chair_decision: Option<String>,
    pub chair_decided_at: Option<DateTime<Utc>>,
    pub chair_deadline_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub pi_name: Option<String>,
    #[sqlx(default)]
    pub chair_name: Option<String>,
}

/// 審查委員決議回應
#[derive(Debug, Serialize, FromRow)]
pub struct ReviewerDecisionResponse {
    pub id: Uuid,
    pub protocol_version_id: Uuid,
    pub reviewer_id: Uuid,
    pub decision: String,
    pub comment: Option<String>,
    pub decided_at: DateTime<Utc>,
    #[sqlx(default)]
    pub reviewer_name: Option<String>,
    #[sqlx(default)]
    pub reviewer_email: Option<String>,
}

/// 全員會議請求回應
#[derive(Debug, Serialize, FromRow)]
pub struct MeetingRequestResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub requested_by: Uuid,
    pub reason: String,
    pub status: String,
    pub meeting_date: Option<DateTime<Utc>>,
    pub chair_decision: Option<String>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub requester_name: Option<String>,
    #[sqlx(default)]
    pub protocol_no: Option<String>,
    #[sqlx(default)]
    pub protocol_title: Option<String>,
}
