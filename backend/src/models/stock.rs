use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

use super::DocType;

/// 庫存流水方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "stock_direction", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum StockDirection {
    In,
    Out,
    TransferIn,
    TransferOut,
    AdjustIn,
    AdjustOut,
}

/// 庫存流水紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockLedger {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
    pub trx_date: DateTime<Utc>,
    pub doc_type: DocType,
    pub doc_id: Uuid,
    pub doc_no: String,
    pub line_id: Option<Uuid>,
    pub direction: StockDirection,
    pub qty_base: Decimal,
    pub unit_cost: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

/// 庫存現況
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryOnHand {
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub qty_on_hand: Decimal,
    pub avg_cost: Option<Decimal>,
    pub safety_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
}

/// 庫存快照（可選快取表）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventorySnapshot {
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
    pub on_hand_qty_base: Decimal,
    pub avg_cost: Option<Decimal>,
    pub updated_at: DateTime<Utc>,
}

/// 庫存流水查詢
#[derive(Debug, Deserialize)]
pub struct StockLedgerQuery {
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub doc_type: Option<DocType>,
    pub batch_no: Option<String>,
}

/// 庫存現況查詢
#[derive(Debug, Deserialize)]
pub struct InventoryQuery {
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub keyword: Option<String>,
    pub batch_no: Option<String>,
    pub low_stock_only: Option<bool>,
}

/// 庫存流水詳情（含關聯名稱）
#[derive(Debug, Serialize, FromRow)]
pub struct StockLedgerDetail {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub trx_date: DateTime<Utc>,
    pub doc_type: DocType,
    pub doc_id: Uuid,
    pub doc_no: String,
    pub direction: StockDirection,
    pub qty_base: Decimal,
    pub unit_cost: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub running_balance: Option<Decimal>,
}

/// 低庫存警示
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LowStockAlert {
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub qty_on_hand: Decimal,
    pub safety_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub stock_status: String,
}
