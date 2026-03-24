use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ── DB Entities ──

#[derive(Debug, Clone, FromRow)]
pub struct AiApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub created_by: Uuid,
    pub scopes: serde_json::Value,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rate_limit_per_minute: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AiQueryLog {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub query_summary: Option<serde_json::Value>,
    pub response_status: i16,
    pub duration_ms: i32,
    pub source_ip: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Request DTOs ──

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAiApiKeyRequest {
    /// API key 名稱（用途描述）
    pub name: String,
    /// 允許的權限範圍
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    /// 過期時間（可選）
    pub expires_at: Option<DateTime<Utc>>,
    /// 每分鐘請求上限
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: i32,
}

fn default_scopes() -> Vec<String> {
    vec!["read".to_string()]
}

fn default_rate_limit() -> i32 {
    60
}

/// AI 自然語言查詢請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct AiQueryRequest {
    /// 查詢的資料領域
    pub domain: AiQueryDomain,
    /// 篩選條件
    #[serde(default)]
    pub filters: serde_json::Value,
    /// 分頁：頁碼
    #[serde(default = "default_page")]
    pub page: i64,
    /// 分頁：每頁筆數
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// 排序欄位
    pub sort_by: Option<String>,
    /// 排序方向
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    100
}
fn default_sort_order() -> String {
    "desc".to_string()
}

/// AI 可查詢的資料領域
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AiQueryDomain {
    /// 動物基本資料
    Animals,
    /// 觀察紀錄
    Observations,
    /// 手術紀錄
    Surgeries,
    /// 體重紀錄
    Weights,
    /// 實驗計畫 (AUP)
    Protocols,
    /// 設施 / 欄位
    Facilities,
    /// 庫存
    Stock,
    /// 人資概況
    HrSummary,
}

// ── Response DTOs ──

/// 建立 API key 回應（僅此一次回傳完整金鑰）
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateAiApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    /// 完整 API key（僅此一次顯示，請妥善保存）
    pub api_key: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_per_minute: i32,
    pub created_at: DateTime<Utc>,
}

/// API key 清單項目（不含完整金鑰）
#[derive(Debug, Serialize, ToSchema)]
pub struct AiApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rate_limit_per_minute: i32,
    pub created_at: DateTime<Utc>,
}

/// AI 查詢回應
#[derive(Debug, Serialize, ToSchema)]
pub struct AiQueryResponse {
    pub domain: AiQueryDomain,
    pub data: serde_json::Value,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    /// 系統提供的資料摘要，方便 AI 理解
    pub summary: Option<String>,
}

/// AI 可用端點的 schema 描述
#[derive(Debug, Serialize, ToSchema)]
pub struct AiSchemaResponse {
    pub version: String,
    pub domains: Vec<AiDomainSchema>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AiDomainSchema {
    pub name: String,
    pub description: String,
    pub available_filters: Vec<AiFilterField>,
    pub available_sort_fields: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AiFilterField {
    pub field: String,
    pub field_type: String,
    pub description: String,
    pub example: Option<String>,
}

/// 系統概覽（AI 進入時的第一個端點）
#[derive(Debug, Serialize, ToSchema)]
pub struct AiSystemOverview {
    pub system_name: String,
    pub version: String,
    pub total_animals: i64,
    pub active_animals: i64,
    pub total_protocols: i64,
    pub active_protocols: i64,
    pub total_observations_30d: i64,
    pub total_surgeries_30d: i64,
    pub facilities_count: i64,
    pub generated_at: DateTime<Utc>,
}
