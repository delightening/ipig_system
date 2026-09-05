use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::pending_owner::PendingOwner;

/// 單據類型
///
/// ⚠️ DB 的 `doc_type` enum 另含 `'DO'`（銷貨出庫）、`'RM'`（退料單）、`'SR'`（銷貨退貨）、
/// `'RTN'`（退貨單）四個值，**本 Rust enum 刻意不列**：
/// - `DO`（R84-9，2026-07-23）自 2026-07-21（#1005）起封鎖新建，SO 改一段式扣帳後不再需要；
/// - `RM`（R84-12，2026-07-23）前端一直隱藏、後端 `process_single_line` 從未有對應分支，是純死值；
/// - `SR` / `RTN`（R84-13，2026-08-13）使用者裁定業務上不存在銷貨退貨——ERP 出庫全為內部
///   耗材領用、SO 從不認列收入，`post_sr` 沖銷的是一筆從未發生的收入。真有「耗材退回倉庫」
///   需求改用 ADJ 或沖銷單（R84-5），不需要獨立的退貨單據類型。
///
/// 依 `docs/reviews/2026-07-22-r84-9-do-enum-removal-plan.md` §8 使用者裁定的
/// **選項 B**：只清死碼、保留 DB enum 值（無害未用），不對 `documents` /
/// `stock_ledger` 兩張核心表做型別重建。SR/RTN 沿用同一選項（2026-08-13 執行前查證
/// prod：SR/RTN 皆 0 張、科目 1200/4100 分錄 0 筆、有填單價的 SO 明細 0 筆）。
///
/// 因此 sqlx 若真的解碼到這四個值會報錯（報錯優於靜默誤判成別的類型）。
/// 清理當下 prod 四表皆 0 筆、且 Rust 端已無法產生這些值，但「查過一次」不等於
/// 「持續成立」——`startup::seed` 的 schema integrity 步驟每次啟動都會實際 count
/// 這些值並在非 0 時明確告警，不把這件事留在註解裡當假設。
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "doc_type", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum DocType {
    /// 採購單 Purchase Order
    PO,
    /// 採購入庫 Goods Receipt Note
    GRN,
    /// 採購退貨 Purchase Return
    PR,
    /// 銷貨單 Sales Order
    SO,
    /// 調撥單 Transfer
    TR,
    /// 盤點單 Stocktake
    STK,
    /// 調整單 Stock Adjustment
    ADJ,
}

impl DocType {
    pub fn prefix(&self) -> &'static str {
        match self {
            DocType::PO => "PO",
            DocType::GRN => "GRN",
            DocType::PR => "PR",
            DocType::SO => "SO",
            DocType::TR => "TR",
            DocType::STK => "STK",
            DocType::ADJ => "ADJ",
        }
    }

    /// 是否影響庫存（入庫、銷貨、調撥、調整、盤點）
    /// SO 銷貨單自 migration 136 起改為一段式：核准即逐行照「該行儲位所屬倉」扣帳（不再靠 DO）。
    pub fn affects_stock(&self) -> bool {
        matches!(
            self,
            DocType::GRN | DocType::PR | DocType::SO | DocType::TR | DocType::ADJ | DocType::STK
        )
    }

    /// 是否對「有追蹤旗標的品項」強制要求批號與效期。
    /// 實際是否擋下仍逐品項看 `products.track_batch` / `track_expiry`（兩旗標各自獨立判斷，
    /// 見 `crud.rs` 驗證迴圈）——這裡只決定「哪些單據類型會做這個檢查」。
    /// 涵蓋：入庫(GRN)、內部領用(SO)、調整(ADJ)、盤點(STK)、採購退貨(PR)、調撥(TR)。
    pub fn requires_batch_expiry(&self) -> bool {
        matches!(
            self,
            DocType::GRN | DocType::SO | DocType::ADJ | DocType::STK | DocType::PR | DocType::TR
        )
    }

    /// 是否**硬性**強制要求 line.storage_location_id（儲位/貨架）——缺儲位即擋下建單/改單。
    /// 涵蓋出庫/盤點（SO/ADJ/STK）：這些單據若無明確儲位即為無意義操作
    /// （出庫不知從哪個貨架扣、盤點不知盤哪格），故當下必填。
    /// PO（採購尚未入庫）不影響庫存故排除；TR 的來源/目標儲位另由
    /// storage_location_from_id / to_id 處理（已記錄至 ledger），故排除。
    /// GRN（採購入庫）**不在此**——改為軟擋（見 `shelf_soft_expected`）：允許缺儲位核准，
    /// 事後再分配上架，未上架期間列入待辦提醒。
    pub fn requires_shelf(&self) -> bool {
        matches!(self, DocType::SO | DocType::ADJ | DocType::STK)
    }

    /// 是否為「軟擋」儲位單據——**建議**填儲位但缺了不擋，核准後列入未上架待辦。
    /// 目前僅 GRN（採購入庫）：實務上常先收貨、事後再決定放哪格貨架。核准時倉庫總量上升
    /// 但明細 storage_location_id 為 NULL → 產生「未分配」庫存，由 line_shelf_allocations
    /// 追溯來源並在分配時精確扣回。
    pub fn shelf_soft_expected(&self) -> bool {
        matches!(self, DocType::GRN)
    }
}

/// 單據狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "doc_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DocStatus {
    Draft,
    Submitted,
    Approved,
    Cancelled,
}

/// 單據頭
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Document {
    pub id: Uuid,
    pub doc_type: DocType,
    pub doc_no: String,
    pub status: DocStatus,
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub doc_date: NaiveDate,
    pub remark: Option<String>,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    /// 來源單據 ID（如入庫單關聯採購單）
    pub source_doc_id: Option<Uuid>,
    /// R84-5 紅字沖銷：本單為哪一張原始單據的沖銷單（放在沖銷單身上，指向原單；
    /// 一般單據為 NULL）。migration 139。沖銷 service 邏輯於後續 PR 實作。
    #[sqlx(default)]
    pub reverses_doc_id: Option<Uuid>,
    /// 是否由系統自動產生（非人工開立）。migration 152。
    ///
    /// true 時 `created_by` 只是系統借用的欄位、不代表真正的作者，故 R97-1 的
    /// 職務分離守衛（「建立者不得自核」）對其豁免——目前唯一來源是盤點單(STK)
    /// 核准後自動產生的差異調整單。
    ///
    /// ⚠️ **不得加進任何 request DTO**：`WAREHOUSE_MANAGER` 同時具備
    /// `erp.document.create` 與 `erp.document.approve`，若能自行標記即可繞過守衛。
    #[sqlx(default)]
    pub system_generated: bool,
    /// 入庫狀態（僅採購單使用）: pending/partial/complete
    pub receipt_status: Option<String>,
    /// 盤點範圍設定（循環盤點用）
    pub stocktake_scope: Option<serde_json::Value>,
    /// IACUC 計畫編號（專案費用歸屬）
    #[sqlx(default)]
    pub iacuc_no: Option<String>,
    /// 銷貨計畫 ID（SO/DO 直接關聯計畫，取代手動建立客戶）
    #[sqlx(default)]
    pub protocol_id: Option<Uuid>,
    // 主管簽核相關欄位 (報廢金額超過門檻時使用)
    #[sqlx(default)]
    pub requires_manager_approval: Option<bool>,
    #[sqlx(default)]
    pub scrap_total_amount: Option<Decimal>,
    #[sqlx(default)]
    pub manager_approval_status: Option<String>, // pending, approved, rejected
    #[sqlx(default)]
    pub manager_approved_by: Option<Uuid>,
    #[sqlx(default)]
    pub manager_approved_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub manager_reject_reason: Option<String>,
}

/// 單據明細
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DocumentLine {
    pub id: Uuid,
    pub document_id: Uuid,
    pub line_no: i32,
    pub product_id: Uuid,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remark: Option<String>,
    /// 儲位 ID（GRN 入庫 / PR/DO/SR 等單一儲位流程使用）
    pub storage_location_id: Option<Uuid>,
    /// 該行所屬倉庫（SO 多倉銷貨：從 storage_location_id 反推回填，核准逐行扣帳 / 報表用；
    /// 其他單據沿用表頭 documents.warehouse_id，本欄可為 NULL）。migration 136。
    #[sqlx(default)]
    pub warehouse_id: Option<Uuid>,
    /// 調撥來源儲位 ID（TR 用；migration 069）
    #[sqlx(default)]
    pub storage_location_from_id: Option<Uuid>,
    /// 調撥目標儲位 ID（TR 用；migration 069）
    #[sqlx(default)]
    pub storage_location_to_id: Option<Uuid>,
}

// R26-12：單據頭 + 明細作為同一 audit 事件的 snapshot。headline 無敏感欄位
// （remark 為研究/會計備註，需完整留）；lines.unit_price 為會計資料，GLP
// 審計需要完整保留。空 allowlist。
impl crate::models::audit_diff::AuditRedact for Document {}
impl crate::models::audit_diff::AuditRedact for DocumentLine {}

/// Audit snapshot：單據頭 + 明細組合，用於 DataDiff::compute。
/// 內部用；不作為 API response（DocumentWithLines 才是 API 格式）。
#[derive(Debug, Serialize)]
pub struct DocumentAuditSnapshot<'a> {
    pub document: &'a Document,
    pub lines: &'a [DocumentLine],
}

impl crate::models::audit_diff::AuditRedact for DocumentAuditSnapshot<'_> {}

/// 建立單據請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateDocumentRequest {
    pub doc_type: DocType,
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub source_doc_id: Option<Uuid>,
    pub doc_date: NaiveDate,
    pub remark: Option<String>,
    /// 盤點範圍設定（僅盤點單使用）
    pub stocktake_scope: Option<serde_json::Value>,
    /// IACUC 計畫編號（專案費用歸屬）
    pub iacuc_no: Option<String>,
    /// 銷貨計畫 ID（SO/DO 使用）
    pub protocol_id: Option<Uuid>,
    /// 單據明細（盤點單可選，會根據範圍自動生成）
    #[serde(default)]
    pub lines: Vec<DocumentLineInput>,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct DocumentLineInput {
    pub product_id: Uuid,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remark: Option<String>,
    /// 儲位 ID（GRN 入庫 / PR/DO/SR 等單一儲位流程使用）
    pub storage_location_id: Option<Uuid>,
    /// 調撥來源儲位 ID（TR 用；migration 069）
    #[serde(default)]
    pub storage_location_from_id: Option<Uuid>,
    /// 調撥目標儲位 ID（TR 用；migration 069）
    #[serde(default)]
    pub storage_location_to_id: Option<Uuid>,
}

/// 更新單據請求 (僅 Draft 狀態可更新)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateDocumentRequest {
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
    pub source_doc_id: Option<Uuid>,
    pub doc_date: Option<NaiveDate>,
    pub remark: Option<String>,
    pub lines: Option<Vec<DocumentLineInput>>,
}

/// 查詢單據
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct DocumentQuery {
    pub doc_type: Option<DocType>,
    /// 多類型篩選，逗號分隔，例如 "PO,GRN,PR"；與 doc_type 同時存在時 doc_type 優先
    pub doc_types: Option<String>,
    pub status: Option<DocStatus>,
    pub warehouse_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub keyword: Option<String>,
    pub iacuc_no: Option<String>,
    /// 只列出含此產品明細的單據（產品詳情頁「相關單據」用）
    pub product_id: Option<Uuid>,
}

/// 單據詳情（含明細）
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentWithLines {
    #[serde(flatten)]
    pub document: Document,
    pub lines: Vec<DocumentLineWithProduct>,
    pub warehouse_name: Option<String>,
    pub warehouse_from_name: Option<String>,
    pub warehouse_to_name: Option<String>,
    pub partner_name: Option<String>,
    /// 銷貨計畫編號（protocol_id 對應）
    pub protocol_no: Option<String>,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
    /// R84-5 沖銷關聯（本單是沖銷單時）：被本單沖銷的原單單號。
    /// `document.reverses_doc_id` 只有 UUID，前端要顯示得有單號。
    pub reverses_doc_no: Option<String>,
    /// R84-5 沖銷關聯（本單被沖銷時）：沖銷本單的那張沖銷單。
    /// 反向查詢（`WHERE reverses_doc_id = 本單`），原單身上沒有這個欄位。
    pub reversed_by_doc_id: Option<Uuid>,
    pub reversed_by_doc_no: Option<String>,
    /// 沖銷生效時間（沖銷單的 approved_at）；沖銷單尚未核准時為 None。
    pub reversed_at: Option<DateTime<Utc>>,
    /// 這張單現在卡在誰手上（僅 `submitted` 有值）。前端「待核准」徽章 hover 用。
    /// 計算來源見 `services/pending_owner.rs`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_owner: Option<PendingOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DocumentLineWithProduct {
    pub id: Uuid,
    pub document_id: Uuid,
    pub line_no: i32,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    /// 品項基本單位。編輯既有單據時，單位下拉要能重建 `base_uom` + `alt_uoms` 這組選項，
    /// 光有 `uom`（本行當初選的那一個）不夠。
    #[sqlx(default)]
    pub product_base_uom: String,
    /// 同 `Product::alt_uoms`：本品項在 `product_uom_conversions` 有換算率的其他單位。
    #[serde(default)]
    #[sqlx(default)]
    pub product_alt_uoms: Vec<String>,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remark: Option<String>,
    /// 儲位 ID（GRN 入庫 / PR/DO/SR 等單一儲位流程使用）
    pub storage_location_id: Option<Uuid>,
    /// 該行所屬倉庫（SO 多倉銷貨：從 storage_location_id 反推回填，核准逐行扣帳 / 報表用；
    /// 其他單據沿用表頭 documents.warehouse_id，本欄可為 NULL）。migration 136。
    #[sqlx(default)]
    pub warehouse_id: Option<Uuid>,
    /// 調撥來源儲位 ID（TR 用；migration 069）
    #[sqlx(default)]
    pub storage_location_from_id: Option<Uuid>,
    /// 調撥目標儲位 ID（TR 用；migration 069）
    #[sqlx(default)]
    pub storage_location_to_id: Option<Uuid>,
}

/// 單據列表項
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub doc_type: DocType,
    pub doc_no: String,
    pub status: DocStatus,
    pub warehouse_name: Option<String>,
    pub partner_id: Option<Uuid>,
    pub partner_name: Option<String>,
    pub protocol_id: Option<Uuid>,
    pub protocol_no: Option<String>,
    pub doc_date: NaiveDate,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub line_count: i64,
    pub total_amount: Option<Decimal>,
    #[sqlx(default)]
    pub iacuc_no: Option<String>,
    #[sqlx(default)]
    pub receipt_status: Option<String>,
    /// 是否已產生會計傳票（核准後觸發過帳的類型：GRN, DO, PR）
    #[sqlx(default)]
    pub has_journal_entry: bool,
    /// 這張單現在卡在誰手上（僅 `submitted` 有值）。前端「待核准」徽章 hover 用。
    ///
    /// 不由 SQL 直接產生（授權判準是程式邏輯不是欄位），由
    /// `services/pending_owner.rs::resolve_for_documents` 在列表查詢後整頁批次補上。
    ///
    /// ⚠️ 必須是 `skip` 不是 `default`：`default` 仍會先嘗試 `try_get` 再 fallback，
    /// 於是要求型別實作 `Decode`（`PendingOwner` 不是 DB 型別，沒有也不該有）。
    #[sqlx(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_owner: Option<PendingOwner>,
}

/// 採購單入庫狀態
#[derive(Debug, Serialize, ToSchema)]
pub struct PoReceiptStatus {
    pub po_id: Uuid,
    pub po_no: String,
    /// pending: 待入庫, partial: 部分入庫, complete: 完成入庫
    pub status: String,
    pub items: Vec<PoReceiptItem>,
}

/// 採購單入庫項目
#[derive(Debug, Serialize, ToSchema)]
pub struct PoReceiptItem {
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub base_uom: String,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub ordered_qty: Decimal,
    pub received_qty: Decimal,
    pub remaining_qty: Decimal,
}

/// ADMIN 駁回請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminRejectRequest {
    pub reason: String,
}

/// 盤點範圍設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StocktakeScope {
    /// 盤點類型: full (全盤) / partial (循環盤點)
    pub scope_type: String,
    /// 依類別篩選
    pub category_codes: Option<Vec<String>>,
    /// 依倉庫篩選
    pub warehouse_ids: Option<Vec<Uuid>>,
    /// 依品項篩選
    pub product_ids: Option<Vec<Uuid>>,
}

/// 建立盤點單請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateStocktakeRequest {
    pub warehouse_id: Uuid,
    pub doc_date: NaiveDate,
    pub remark: Option<String>,
    /// 盤點範圍設定
    pub scope: Option<StocktakeScope>,
}

/// 盤點結果輸入（匯入用）
#[derive(Debug, Deserialize)]
pub struct StocktakeResultInput {
    pub product_id: Uuid,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    /// 實際盤點數量
    pub actual_qty: Decimal,
}

/// 盤點差異項目
#[derive(Debug, Serialize)]
pub struct StocktakeDifferenceItem {
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    /// 系統庫存
    pub system_qty: Decimal,
    /// 實際盤點
    pub actual_qty: Decimal,
    /// 差異 (actual - system)
    pub difference: Decimal,
    pub uom: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_shelf_covers_inventory_affecting_single_shelf_types() {
        // SO/ADJ/STK：出庫/盤點、需單一儲位 → 硬性必填（杜絕未分配 drift）
        assert!(DocType::SO.requires_shelf());
        assert!(DocType::ADJ.requires_shelf());
        assert!(DocType::STK.requires_shelf());
    }

    #[test]
    fn grn_is_soft_shelf_not_hard_required() {
        // 2026-07-16：GRN（採購入庫）改軟擋——不硬性必填，改列入 shelf_soft_expected，
        // 允許缺儲位核准、事後分配上架。
        assert!(!DocType::GRN.requires_shelf());
        assert!(DocType::GRN.shelf_soft_expected());
        // 其餘硬擋型別不屬於軟擋
        assert!(!DocType::SO.shelf_soft_expected());
        assert!(!DocType::ADJ.shelf_soft_expected());
    }

    /// R84-9 / R84-12 / R84-13：`DO`（銷貨出庫）、`RM`（退料單）、`SR`（銷貨退貨）、
    /// `RTN`（退貨單）已從 Rust enum 移除，DB enum 值則刻意保留（選項 B）。本測試鎖定
    /// 「移除後這些值無法從 API 進入系統」這個不變式——它取代了原本
    /// `erp_so_multi_warehouse.rs::deprecated_do_creation_is_blocked` 在 service 層做的
    /// 封鎖檢查，保障層級從 service 前移到反序列化。
    ///
    /// 為何這件事重要：若 `DO` 能與一段式 `SO` 並用，同一筆銷貨會被雙扣庫存、雙認營收；
    /// 若 `SR`/`RTN` 能建立，`post_document` 會沖銷一筆從未發生的收入（R84-13）。
    #[test]
    fn deprecated_doc_types_are_rejected_at_deserialization() {
        assert!(
            serde_json::from_str::<DocType>("\"DO\"").is_err(),
            "DO 已棄用，不得能從 API 反序列化進入系統"
        );
        assert!(
            serde_json::from_str::<DocType>("\"RM\"").is_err(),
            "RM 為死值，不得能從 API 反序列化進入系統"
        );
        assert!(
            serde_json::from_str::<DocType>("\"SR\"").is_err(),
            "SR 業務上不存在銷貨退貨，不得能從 API 反序列化進入系統（R84-13）"
        );
        assert!(
            serde_json::from_str::<DocType>("\"RTN\"").is_err(),
            "RTN 業務上不存在銷貨退貨，不得能從 API 反序列化進入系統（R84-13）"
        );
        // 對照組：現役類型仍須正常解析，確認不是整個反序列化都壞掉
        assert_eq!(
            serde_json::from_str::<DocType>("\"SO\"").expect("SO 應可解析"),
            DocType::SO
        );
    }

    #[test]
    fn requires_shelf_excludes_purchase_orders_and_transfer() {
        // PO/PR：採購尚未入庫 → 不需儲位
        assert!(!DocType::PO.requires_shelf());
        assert!(!DocType::PR.requires_shelf());
        // TR：from/to 兩個儲位另由 storage_location_from_id / to_id 處理（目前 model 未接，scope 外）
        assert!(!DocType::TR.requires_shelf());
    }
}
