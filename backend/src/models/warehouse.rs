use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::storage_location::StorageLocationInventoryItem;

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

/// 儲位含庫存（報表用）
#[derive(Debug, Serialize, ToSchema)]
pub struct StorageLocationWithInventory {
    pub id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub location_type: String,
    pub row_index: i32,
    pub col_index: i32,
    pub width: i32,
    pub height: i32,
    pub capacity: Option<i32>,
    pub current_count: i32,
    pub color: Option<String>,
    pub is_active: bool,
    pub inventory: Vec<StorageLocationInventoryItem>,
}

/// 倉庫報表彙總統計
#[derive(Debug, Serialize, ToSchema)]
pub struct WarehouseReportSummary {
    pub total_locations: i32,
    pub active_locations: i32,
    pub total_capacity: i32,
    pub total_current_count: i32,
    pub total_inventory_items: i32,
}

/// 倉庫現況報表回應
#[derive(Debug, Serialize, ToSchema)]
pub struct WarehouseReportData {
    pub warehouse: Warehouse,
    pub summary: WarehouseReportSummary,
    pub locations: Vec<StorageLocationWithInventory>,
    pub generated_at: DateTime<Utc>,
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
