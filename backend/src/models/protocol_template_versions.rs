use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 計畫書範本版本（院區層級，admin 管理）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProtocolTemplateVersion {
    pub id: Uuid,
    pub version_label: String,
    pub effective_date: Option<NaiveDate>,
    pub is_current: bool,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 計畫書範本版本（含建立者顯示名稱，列表用）。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ProtocolTemplateVersionResponse {
    pub id: Uuid,
    pub version_label: String,
    pub effective_date: Option<NaiveDate>,
    pub is_current: bool,
    pub notes: Option<String>,
    pub created_by: Uuid,
    #[sqlx(default)]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 範本版本文件（沿用 attachments，entity_type=protocol_template_version）。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct TemplateVersionDocument {
    pub id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProtocolTemplateVersionRequest {
    #[validate(length(min = 1, max = 100, message = "版本號為必填，至多 100 字"))]
    pub version_label: String,
    pub effective_date: Option<NaiveDate>,
    #[validate(length(max = 2000, message = "備註至多 2000 字"))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProtocolTemplateVersionRequest {
    #[validate(length(min = 1, max = 100, message = "版本號至多 100 字"))]
    pub version_label: Option<String>,
    pub effective_date: Option<NaiveDate>,
    #[validate(length(max = 2000, message = "備註至多 2000 字"))]
    pub notes: Option<String>,
}
