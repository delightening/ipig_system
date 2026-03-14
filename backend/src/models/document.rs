use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 單據類型
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
    /// 銷貨出庫 Delivery Order
    DO,
    /// 調撥單 Transfer
    TR,
    /// 盤點單 Stocktake
    STK,
    /// 調整單 Stock Adjustment
    ADJ,
    /// 退料單 Return Material
    RM,
    /// 銷貨退貨 Sales Return（DB 既有枚舉，與 RM 並存以相容舊資料）
    SR,
    /// 退貨單 Return（DB 既有枚舉）
    RTN,
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
            DocType::SR => "SR",
            DocType::RTN => "RTN",
        }
    }

    /// 是否影響庫存（入庫、出庫、調撥、調整、盤點）
    pub fn affects_stock(&self) -> bool {
        matches!(
            self,
            DocType::GRN
                | DocType::PR
                | DocType::DO
                | DocType::TR
                | DocType::ADJ
                | DocType::SR
                | DocType::RTN
                | DocType::STK
                | DocType::RM
        )
    }

    /// 是否強制要求批號與效期（入庫、銷貨出庫、調整）
    pub fn requires_batch_expiry(&self) -> bool {
        matches!(
            self,
            DocType::GRN | DocType::DO | DocType::SO | DocType::ADJ | DocType::STK
        )
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
    /// 入庫狀態（僅採購單使用）: pending/partial/complete
    pub receipt_status: Option<String>,
    /// 盤點範圍設定（循環盤點用）
    pub stocktake_scope: Option<serde_json::Value>,
    /// IACUC 計畫編號（專案費用歸屬）
    #[sqlx(default)]
    pub iacuc_no: Option<String>,
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
    /// 儲位 ID（GRN 入庫指定儲位）
    pub storage_location_id: Option<Uuid>,
}

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
    /// 儲位 ID（GRN 入庫指定儲位）
    pub storage_location_id: Option<Uuid>,
}

/// 更新單據請求 (僅 Draft 狀態可更新)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateDocumentRequest {
    pub warehouse_id: Option<Uuid>,
    pub warehouse_from_id: Option<Uuid>,
    pub warehouse_to_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
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
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    /// 儲位 ID（GRN 入庫指定儲位）
    pub storage_location_id: Option<Uuid>,
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
