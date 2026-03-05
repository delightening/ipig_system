use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateWarehouseRequest {
    pub code: Option<String>,
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateWarehouseRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WarehouseQuery {
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 倉庫樹節點（含貨架）
#[derive(Debug, Clone, Serialize)]
pub struct WarehouseTreeNode {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub shelves: Vec<ShelfNode>,
}

/// 貨架節點
#[derive(Debug, Clone, Serialize)]
pub struct ShelfNode {
    pub id: Uuid,
    pub code: String,
    pub name: Option<String>,
}

/// 倉庫匯入 CSV 列
#[derive(Debug, Clone, Default)]
pub struct WarehouseImportRow {
    pub name: String,
    pub code: Option<String>,
    pub address: Option<String>,
}

/// 倉庫匯入錯誤明細
#[derive(Debug, Serialize)]
pub struct WarehouseImportErrorDetail {
    pub row: i32,
    pub code: Option<String>,
    pub error: String,
}

/// 倉庫匯入結果
#[derive(Debug, Serialize)]
pub struct WarehouseImportResult {
    pub success_count: i32,
    pub error_count: i32,
    pub errors: Vec<WarehouseImportErrorDetail>,
}
