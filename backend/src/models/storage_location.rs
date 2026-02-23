// 儲位/貨架 Models
// 用於倉庫內部視覺化佈局管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 儲位類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, Default)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    #[default]
    Shelf, // 貨架
    Rack, // 儲物架
    Zone, // 區域
    Bin,  // 儲物格
}

/// 儲位/貨架資料結構
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocation {
    pub id: Uuid,
    pub warehouse_id: Uuid,
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
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 儲位詳細資料（包含倉庫資訊）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocationWithWarehouse {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
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
    pub config: Option<serde_json::Value>,
}

/// 建立儲位請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateStorageLocationRequest {
    pub warehouse_id: Uuid,
    /// 代碼（選填，系統會自動生成）
    #[validate(length(max = 50, message = "Code must be at most 50 characters"))]
    pub code: Option<String>,
    /// 名稱（必填）
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub location_type: Option<String>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub capacity: Option<i32>,
    pub color: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// 更新儲位請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStorageLocationRequest {
    #[validate(length(min = 1, max = 50, message = "Code must be 1-50 characters"))]
    pub code: Option<String>,
    #[validate(length(max = 200, message = "Name must be at most 200 characters"))]
    pub name: Option<String>,
    pub location_type: Option<String>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub capacity: Option<i32>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
}

/// 單一儲位佈局項目（用於批次更新）
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageLayoutItem {
    pub id: Uuid,
    pub row_index: i32,
    pub col_index: i32,
    pub width: i32,
    pub height: i32,
}

/// 批次更新儲位佈局請求
#[derive(Debug, Deserialize)]
pub struct UpdateStorageLayoutRequest {
    pub items: Vec<StorageLayoutItem>,
}

/// 儲位查詢參數
#[derive(Debug, Deserialize)]
pub struct StorageLocationQuery {
    pub warehouse_id: Option<Uuid>,
    pub location_type: Option<String>,
    pub is_active: Option<bool>,
    pub keyword: Option<String>,
}

/// 儲位庫存項目（用於顯示儲位內的庫存）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocationInventoryItem {
    pub id: Uuid,
    pub storage_location_id: Uuid,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub on_hand_qty: rust_decimal::Decimal,
    pub base_uom: String,
    pub batch_no: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub updated_at: DateTime<Utc>,
}

/// 更新儲位庫存項目請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStorageLocationInventoryItemRequest {
    // Note: range validator doesn't work with rust_decimal::Decimal
    // Validation for non-negative values is handled in the service layer
    pub on_hand_qty: rust_decimal::Decimal,
}

/// 新增儲位庫存項目請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateStorageLocationInventoryItemRequest {
    pub product_id: Uuid,
    pub on_hand_qty: rust_decimal::Decimal,
    #[validate(length(max = 50, message = "Batch number must be at most 50 characters"))]
    pub batch_no: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
}

/// 調撥儲位庫存請求 (同倉庫內不需單據)
#[derive(Debug, Deserialize, Validate)]
pub struct TransferStorageLocationInventoryRequest {
    pub to_storage_location_id: Uuid,
    pub qty: rust_decimal::Decimal,
}
