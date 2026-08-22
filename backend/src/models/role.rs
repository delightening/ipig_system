use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// R30-27：role / permission 變更時附帶的電子簽章（密碼 + 手寫雙因子）。
///
/// `config.role_signature_required = true` 時 backend 強制要求；false 時欄位忽略。
/// 對應 21 CFR §11.10(d) 存取控制簽章不可否認性。
///
/// `stroke_data` 為手寫筆畫向量（含時序、壓力等），用於日後鑑定簽章樣式（客戶 / 員工 /
/// 外部操作人員 / 老闆 — 各角色簽名特徵差異）；可選但建議帶。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MutationSignaturePayload {
    /// 簽章人密碼（驗證身分）
    pub password: String,
    /// 手寫簽名 SVG（必填，桌機可由 R30-27c 手機 bridge 帶回）
    pub handwriting_svg: String,
    /// 手寫筆畫資料（時序 / 壓力 / 座標序列）— 鑑定簽名真偽用
    pub stroke_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_internal: bool,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 無敏感欄位，空 impl 即可（見 AuditRedact trait doc 警告）
impl crate::models::audit_diff::AuditRedact for Role {}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub module: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 50, message = "Code must be 1-50 characters"))]
    pub code: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_is_internal")]
    pub is_internal: bool,
    #[serde(default)]
    pub permission_ids: Vec<Uuid>,
    /// R30-27：簽章 payload；config.role_signature_required=true 時必填，否則忽略
    #[serde(default)]
    pub mutation_signature: Option<MutationSignaturePayload>,
}

fn default_is_internal() -> bool {
    true
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_internal: Option<bool>,
    pub permission_ids: Option<Vec<Uuid>>,
    /// R30-27：簽章 payload；config.role_signature_required=true 時必填，否則忽略
    #[serde(default)]
    pub mutation_signature: Option<MutationSignaturePayload>,
}

/// R30-27：role 刪除請求的簽章 payload。delete handler 可選地接收 body
/// （DELETE 帶 body 非標準但 axum 支援）。flag=true 時必填。
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteRoleRequest {
    #[serde(default)]
    pub mutation_signature: Option<MutationSignaturePayload>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_internal: bool,
    pub is_system: bool,
    pub is_active: bool,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignUserRoleRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PermissionQuery {
    pub module: Option<String>,
}
