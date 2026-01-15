use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// 單據類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "doc_type", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum DocType {
    /// 採購單 Purchase Order
    PO,
    /// 採購入庫 Goods Receipt Note
    GRN,
    /// 採購退貨 Purchase Return
    PR,
    /// 銷售單 Sales Order
    SO,
    /// 銷售出庫 Delivery Order
    DO,
    /// 調撥單 Transfer
    TR,
    /// 盤點單 Stocktake
    STK,
    /// 調整單 Stock Adjustment
    ADJ,
    /// 退料單 Return Material
    RM,
}

impl DocType {
    pub fn prefix(&self) -> &'static str {
        match self {
            DocType::PO => "PO",
            DocType::GRN => "GRN",
            DocType::PR => "PR",
            DocType::SO => "SO",
            DocType::DO => "DO",
            DocType::TR => "TR",
            DocType::STK => "STK",
            DocType::ADJ => "ADJ",
            DocType::RM => "RM",
        }
    }

    /// 是否影響庫存
    pub fn affects_stock(&self) -> bool {
        matches!(self, DocType::GRN | DocType::PR | DocType::DO | DocType::TR | DocType::ADJ)
    }
}

/// 單據狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "doc_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DocStatus {
    Draft,
    Submitted,
    Approved,
    Cancelled,
}

/// 單據頭
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    /// 入庫狀態（僅採購單使用）: pending/partial/complete
    pub receipt_status: Option<String>,
    /// 盤點範圍設定（循環盤點用）
    pub stocktake_scope: Option<serde_json::Value>,
}

/// 單據明細
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
}

/// 建立單據請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    pub doc_type: DocType,
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub doc_date: NaiveDate,
    pub remark: Option<String>,
    /// 盤點範圍設定（僅盤點單使用）
    pub stocktake_scope: Option<serde_json::Value>,
    /// 單據明細（盤點單可選，會根據範圍自動生成）
    #[serde(default)]
    pub lines: Vec<DocumentLineInput>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentLineInput {
    pub product_id: Uuid,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remark: Option<String>,
}

/// 更新單據請求 (僅 Draft 狀態可更新)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub doc_date: Option<NaiveDate>,
    pub remark: Option<String>,
    pub lines: Option<Vec<DocumentLineInput>>,
}

/// 查詢單據
#[derive(Debug, Deserialize)]
pub struct DocumentQuery {
    pub doc_type: Option<DocType>,
    pub status: Option<DocStatus>,
    pub warehouse_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub keyword: Option<String>,
}

/// 單據詳情（含明細）
#[derive(Debug, Serialize)]
pub struct DocumentWithLines {
    #[serde(flatten)]
    pub document: Document,
    pub lines: Vec<DocumentLineWithProduct>,
    pub warehouse_name: Option<String>,
    pub warehouse_from_name: Option<String>,
    pub warehouse_to_name: Option<String>,
    pub partner_name: Option<String>,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentLineWithProduct {
    pub id: Uuid,
    pub document_id: Uuid,
    pub line_no: i32,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub remark: Option<String>,
}

/// 單據列表項
#[derive(Debug, Serialize, FromRow)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub doc_type: DocType,
    pub doc_no: String,
    pub status: DocStatus,
    pub warehouse_name: Option<String>,
    pub partner_name: Option<String>,
    pub doc_date: NaiveDate,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub line_count: i64,
    pub total_amount: Option<Decimal>,
}

/// 採購單入庫狀態
#[derive(Debug, Serialize)]
pub struct PoReceiptStatus {
    pub po_id: Uuid,
    pub po_no: String,
    /// pending: 待入庫, partial: 部分入庫, complete: 完成入庫
    pub status: String,
    pub items: Vec<PoReceiptItem>,
}

/// 採購單入庫項目
#[derive(Debug, Serialize)]
pub struct PoReceiptItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub ordered_qty: Decimal,
    pub received_qty: Decimal,
    pub remaining_qty: Decimal,
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
