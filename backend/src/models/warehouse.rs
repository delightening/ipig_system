use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::storage_location::StorageLocationInventoryItem;

// 倉庫主檔，無敏感欄位
impl crate::models::audit_diff::AuditRedact for Warehouse {}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub is_active: bool,
    /// 排除於低庫存警報之外（migration 014）。
    ///
    /// 用於帳面準確度不受維護的倉庫（每天領用的儲藏室），或本來就不是庫存資產的
    /// 地點（廢棄物處理區）。`v_low_stock_alerts` 直接讀這個欄位過濾。
    pub exclude_from_alerts: bool,
    /// 不納入例行盤點（migration 014）。
    ///
    /// 命名是 routine 而非 scheduled：本系統沒有盤點排程器，月盤是人工開單。
    /// ⚠️ 目前**沒有任何後端邏輯讀它**——作用點在前端（開盤點單時提示）與人的流程。
    /// 日後若做排程器，直接讀這個欄位即可。
    pub skip_routine_stocktake: bool,
    /// 此倉庫是否為領用來源（migration 015）。
    ///
    /// 建 SO 時，若所選品項在這個倉庫有貨，儲位欄位會自動帶該倉的儲位。
    /// **可同時有多個倉庫為 true**——藥品在準備室藥品櫃、耗材在儲藏室鐵櫃，
    /// 兩者都是正當的領用點，系統再依品項實際庫存挑選。故刻意沒有唯一性約束。
    ///
    /// 排除的是「有庫存數字但不該從那裡領」的地點，廢棄物處理區必然為 false。
    pub is_default_issue_source: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 建立倉庫。
///
/// 🔴 三個政策旗標是 2026-09-09 補的（CodeRabbit 於 MR !3 指出）。在那之前它們只存在於
/// `UpdateWarehouseRequest`，而**前端的建立表單一直顯示著那三個開關**：送出的 JSON 帶了
/// 旗標，serde 靜默丟棄未宣告的欄位，請求回 200，值卻沒進 DB。使用者建了廢棄物處理區、
/// 關掉警報、看到「建立成功」，而那個旗標是 false——**沒有任何錯誤訊息**。
///
/// 這是「成功回應 ≠ 工作完成」的實例。新增可設定的欄位時，**建立與更新兩條路徑要一起改**，
/// 否則就會留下這種只在事後對帳才發現的洞。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateWarehouseRequest {
    pub code: Option<String>,
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub address: Option<String>,
    /// 見 `Warehouse::exclude_from_alerts`。None = 用欄位預設（false）。
    pub exclude_from_alerts: Option<bool>,
    /// 見 `Warehouse::skip_routine_stocktake`。None = 用欄位預設（false）。
    pub skip_routine_stocktake: Option<bool>,
    /// 見 `Warehouse::is_default_issue_source`。None = 用欄位預設（false）。
    pub is_default_issue_source: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateWarehouseRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
    /// 見 `Warehouse::exclude_from_alerts`。None = 不改動。
    pub exclude_from_alerts: Option<bool>,
    /// 見 `Warehouse::skip_routine_stocktake`。None = 不改動。
    pub skip_routine_stocktake: Option<bool>,
    /// 見 `Warehouse::is_default_issue_source`。None = 不改動。
    pub is_default_issue_source: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WarehouseQuery {
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 倉庫樹節點（含貨架）
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WarehouseTreeNode {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub shelves: Vec<ShelfNode>,
}

/// 貨架節點
#[derive(Debug, Clone, Serialize, ToSchema)]
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
    /// R35-3 (redo on R35-16): 庫存價值總額 = SUM(on_hand_qty × products.selling_price)。
    /// 缺價產品 (`selling_price IS NULL`) 不計入；以字串序列化避免 JS 浮點誤差。
    #[schema(value_type = String)]
    pub total_inventory_value: rust_decimal::Decimal,
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
