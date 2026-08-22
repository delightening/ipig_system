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
    /// 異動發生的儲位 ID（GLP §11 audit trail；migration 069 加入）。
    /// TR 兩端各自寫一筆 ledger，from 端記 from_id、to 端記 to_id；
    /// 其他 doc type 直接記 document_line.storage_location_id（若有）。
    #[sqlx(default)]
    pub storage_location_id: Option<Uuid>,
}

/// 庫存現況
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryOnHand {
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location_name: Option<String>,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub category_code: Option<String>,
    pub qty_on_hand: Decimal,
    pub avg_cost: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub safety_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    /// 最後異動時間（stock_ledger 最新一筆的 trx_date）
    pub last_updated_at: Option<DateTime<Utc>>,
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
    pub limit: Option<i64>,  // 新增: 分頁限制 (預設 100)
    pub offset: Option<i64>, // 新增: 分頁偏移 (預設 0)
}

/// 批號追溯查詢（R84-6）：批號身分＝(product_id, batch_no, expiry_date) 三欄一組
#[derive(Debug, Deserialize)]
pub struct LotMovementsQuery {
    pub product_id: Uuid,
    pub batch_no: String,
    pub expiry_date: Option<NaiveDate>,
}

/// 批號時間軸單筆紀錄（跨倉彙總，見 ERP流程.md §6.2.2）
#[derive(Debug, Serialize, FromRow)]
pub struct LotMovement {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub trx_date: DateTime<Utc>,
    pub doc_type: DocType,
    pub doc_id: Uuid,
    pub doc_no: String,
    pub direction: StockDirection,
    pub qty_base: Decimal,
}

/// 批號對帳結果分級（R84-6 補強）
///
/// 批號層級不平不一定代表數量出錯——2026-05-20 (migration 069) 之前的異動只寫
/// stock_ledger 不寫 storage_location_inventory，之後的 R62-2 補帳與 PHANTOMFIX
/// 幻影庫存清理雖然把品項總量補平了，沖銷列卻未帶批號，導致差額掛在批號之外。
/// 因此先看品項總量：總量相符代表只是批號歸屬對不上，數量本身無虞。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LotReconciliationStatus {
    /// 批號層級帳實相符
    Balanced,
    /// 批號層級不符，但該品項總量相符 → 純屬批號歸屬問題，總量無虞
    AttributionOnly,
    /// 品項總量也對不上 → 真的帳實不符，需人工查證
    Unbalanced,
}

/// 批號數量對帳摘要：收到＋客戶退貨收到＋調整增減(淨額) = 內部領用＋退回供應商＋目前剩餘
#[derive(Debug, Serialize)]
pub struct LotReconciliation {
    pub received: Decimal,
    pub customer_returned: Decimal,
    pub internal_consumed: Decimal,
    pub returned_to_supplier: Decimal,
    pub adjusted_net: Decimal,
    /// 目前剩餘：獨立來源（storage_location_inventory 各儲位加總），非公式推算，用於跟下方公式互相校驗
    pub remaining: Decimal,
    /// 由分類加總反推的剩餘量：received + customer_returned + adjusted_net - internal_consumed - returned_to_supplier
    pub derived_remaining: Decimal,
    /// remaining 與 derived_remaining 是否一致（僅看本批號）
    pub balanced: bool,
    /// 對帳分級：先看品項總量再判定本批號，避免歷史補帳造成的批號歸屬差異被誤報成帳實不符
    pub status: LotReconciliationStatus,
    /// 該品項（跨全部批號）由分類加總反推的總量
    pub product_derived_total: Decimal,
    /// 該品項（跨全部批號）各儲位實際庫存加總
    pub product_remaining_total: Decimal,
    /// 該品項未帶批號的 ADJ 淨額（adjust_in - adjust_out），供人工追查歸屬用
    pub unattributed_adjust_net: Decimal,
}

#[derive(Debug, Serialize)]
pub struct LotMovementsResponse {
    pub movements: Vec<LotMovement>,
    pub reconciliation: LotReconciliation,
}

/// 庫存現況查詢
#[derive(Debug, Default, Deserialize)]
pub struct InventoryQuery {
    pub warehouse_id: Option<Uuid>,
    /// 指定儲位/貨架時，查詢該儲位庫存（來自 storage_location_inventory）
    pub storage_location_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub keyword: Option<String>,
    pub batch_no: Option<String>,
    /// 效期預警篩選：僅顯示 N 天內到期的品項
    pub expiry_within_days: Option<i32>,
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
    #[sqlx(default)]
    pub iacuc_no: Option<String>,
    /// 異動儲位 ID + 名稱（migration 069）— ledger detail 報表顯示用
    #[sqlx(default)]
    pub storage_location_id: Option<Uuid>,
    #[sqlx(default)]
    pub storage_location_name: Option<String>,
}

/// 低庫存警示（逐倉庫；`v_low_stock_alerts` view + 通知/email/排程專用）
///
/// 注意：此 DTO 對應資料庫 view `v_low_stock_alerts`（每倉一列），由
/// `services/notification/alert.rs` 以 `SELECT * → Vec<LowStockAlert>` 反序列化。
/// 庫存查詢頁與 dashboard 低庫存計數已改用 [`LowStockTotal`]（全公司總量），
/// 兩者刻意分流：通知保留逐倉庫早期預警，庫存查詢改全公司總量避免重複虛報。
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

/// 低庫存品項在單一倉庫的分布（[`LowStockTotal`] 展開明細用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LowStockWarehouseQty {
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub qty_on_hand: Decimal,
}

/// 低庫存彙總（全公司總量 vs 公司預設安全庫存；一品項一筆）
///
/// `safety_stock` 目前為 `products` 表的「公司預設安全庫存」（全倉庫共用一個值），
/// 故警示以全公司總量 `total_on_hand` 判斷，避免同品項在多倉各自低於門檻而重複虛報。
/// `warehouse_breakdown` 提供各倉分布供展開查看；未來若導入 per 倉庫安全庫存，
/// 為加法擴充（覆寫表 + COALESCE），不需改動此結構。
#[derive(Debug, Clone, Serialize)]
pub struct LowStockTotal {
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub total_on_hand: Decimal,
    pub safety_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub stock_status: String,
    pub warehouse_breakdown: Vec<LowStockWarehouseQty>,
}

/// 未分配庫存（倉庫層級有庫存，但不在任何儲位）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UnassignedInventory {
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub qty_on_warehouse: Decimal,
    pub qty_on_shelves: Decimal,
    pub qty_unassigned: Decimal,
}

/// 未分配庫存的來源單據（造成此未分配的 GRN 明細，含剩餘未上架量）。
/// 供「未分配庫存」列展開追溯：這批未分配是哪張採購入庫單造成的。
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UnassignedSourceDoc {
    pub document_id: Uuid,
    pub doc_no: String,
    pub doc_date: NaiveDate,
    pub line_no: i32,
    pub partner_name: Option<String>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remaining_unshelved: Decimal,
}

/// 未分配來源查詢參數（必須指定倉庫 + 產品）
#[derive(Debug, Clone, Deserialize)]
pub struct UnassignedSourceQuery {
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
}

/// 將未分配庫存分配至儲位
#[derive(Debug, Clone, Deserialize)]
pub struct AssignUnassignedRequest {
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
    pub storage_location_id: Uuid,
    pub qty: Decimal,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}
