use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 動物試驗申請須知版本（院區層級，全院共用一份 + 版次制）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationNotice {
    pub id: Uuid,
    pub version_label: String,
    pub title: String,
    /// 須知正文（markdown），顯示給申請人線上閱讀。
    pub content: String,
    /// 選填：原始 PDF/docx 正本（沿用 attachments）。
    pub attachment_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 須知版本（含建立者顯示名稱，列表用）。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ApplicationNoticeResponse {
    pub id: Uuid,
    pub version_label: String,
    pub title: String,
    pub content: String,
    pub attachment_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub is_active: bool,
    pub created_by: Uuid,
    #[sqlx(default)]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 給 `#[derive(Validate)]` 用：要求 trim 後非空白（防純空白字串繞過 `length(min=1)`，
/// 因 service 層會 trim 後存入，否則「  」會落地成空字串）。
fn validate_non_blank(s: &str) -> Result<(), validator::ValidationError> {
    if s.trim().is_empty() {
        return Err(validator::ValidationError::new("不可為空白"));
    }
    Ok(())
}

/// Admin 建立須知版本的 API 請求。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateApplicationNoticeRequest {
    #[validate(
        length(max = 100, message = "版本號至多 100 字"),
        custom(function = "validate_non_blank")
    )]
    pub version_label: String,
    #[validate(
        length(max = 200, message = "標題至多 200 字"),
        custom(function = "validate_non_blank")
    )]
    pub title: String,
    #[validate(custom(function = "validate_non_blank"))]
    pub content: String,
    pub effective_from: NaiveDate,
    pub attachment_id: Option<Uuid>,
}

/// 建立須知版本的輸入（repository insert 用）。
#[derive(Debug, Clone)]
pub struct NewApplicationNotice {
    pub version_label: String,
    pub title: String,
    pub content: String,
    pub effective_from: NaiveDate,
    pub attachment_id: Option<Uuid>,
    pub created_by: Uuid,
}

/// 計畫的須知簽署狀態（前端填表/送審時讀取：是否已簽當前生效須知 + 須知正文）。
#[derive(Debug, Serialize, ToSchema)]
pub struct NoticeAcknowledgementStatus {
    /// 當前生效須知（無生效版本時為 None；無則送審不受須知門檻限制）。
    pub active_notice: Option<ApplicationNotice>,
    /// 此計畫是否已簽「當前生效」須知。
    pub acknowledged: bool,
    /// 簽署時間（acknowledged=true 時有值）。
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// 申請人對某計劃的須知簽署紀錄（一計劃一筆）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProtocolNoticeAcknowledgement {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub notice_id: Uuid,
    pub signer_id: Uuid,
    /// 手寫電子簽章；匯入舊計劃可空（方案 A：改掛紙本掃描）。
    pub signature_id: Option<Uuid>,
    /// 匯入舊計劃的紙本須知簽名掃描。
    pub notice_attachment_id: Option<Uuid>,
    pub acknowledged_at: DateTime<Utc>,
}

/// 申請人簽署須知的 API 請求（手寫電子簽章）。
#[derive(Debug, Deserialize, ToSchema)]
pub struct AcknowledgeNoticeRequest {
    /// 手寫簽名 SVG。
    pub handwriting_svg: String,
    /// 選填：手寫筆畫原始資料。
    pub stroke_data: Option<serde_json::Value>,
}

/// 建立 / 覆寫簽署紀錄的輸入（upsert 用）。
#[derive(Debug, Clone)]
pub struct NewNoticeAcknowledgement {
    pub protocol_id: Uuid,
    pub notice_id: Uuid,
    pub signer_id: Uuid,
    pub signature_id: Option<Uuid>,
    pub notice_attachment_id: Option<Uuid>,
}
