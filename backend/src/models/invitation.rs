use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use super::UserResponse;

/// 邀請 DB entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub organization: Option<String>,
    pub invitation_token: String,
    pub invited_by: Uuid,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 建立邀請請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvitationRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 254, message = "Email must be at most 254 characters"))]
    pub email: String,
    pub organization: Option<String>,
}

/// 邀請回應（列表用）
#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub organization: Option<String>,
    pub invited_by: Uuid,
    pub invited_by_name: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_user_id: Option<Uuid>,
    pub invite_link: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 建立邀請回應
#[derive(Debug, Serialize)]
pub struct CreateInvitationResponse {
    pub invitation: InvitationResponse,
    pub invite_link: String,
}

/// 驗證邀請回應
#[derive(Debug, Serialize)]
pub struct VerifyInvitationResponse {
    pub valid: bool,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub reason: Option<String>,
}

/// 接受邀請請求
#[derive(Debug, Deserialize, Validate)]
pub struct AcceptInvitationRequest {
    pub invitation_token: String,
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,
    pub phone: String,
    pub organization: String,
    #[validate(length(min = 8, max = 128, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub position: Option<String>,
    pub agree_terms: bool,
}

/// 接受邀請回應
#[derive(Debug, Serialize)]
pub struct AcceptInvitationResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

/// 邀請列表查詢參數
#[derive(Debug, Deserialize)]
pub struct InvitationListQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 邀請狀態常數
pub const INVITATION_STATUS_PENDING: &str = "pending";
pub const INVITATION_STATUS_ACCEPTED: &str = "accepted";
pub const INVITATION_STATUS_EXPIRED: &str = "expired";
pub const INVITATION_STATUS_REVOKED: &str = "revoked";

/// 邀請有效天數
pub const INVITATION_EXPIRY_DAYS: i64 = 7;
