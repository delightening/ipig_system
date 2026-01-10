use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 產品狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProductStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "discontinued")]
    Discontinued,
}

impl Default for ProductStatus {
    fn default() -> Self {
        ProductStatus::Active
    }
}

/// 保存條件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageCondition {
    #[serde(rename = "RT")]
    RoomTemperature,  // 常溫 15-25°C
    #[serde(rename = "RF")]
    Refrigerated,     // 冷藏 2-8°C
    #[serde(rename = "FZ")]
    Frozen,           // 冷凍 -20°C 以下
    #[serde(rename = "DK")]
    Dark,             // 避光
    #[serde(rename = "DY")]
    Dry,              // 乾燥
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub spec: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub subcategory_code: Option<String>,
    pub base_uom: String,
    pub pack_unit: Option<String>,
    pub pack_qty: Option<i32>,
    pub track_batch: bool,
    pub track_expiry: bool,
    pub default_expiry_days: Option<i32>,
    pub safety_stock: Option<Decimal>,
    pub safety_stock_uom: Option<String>,
    pub reorder_point: Option<Decimal>,
    pub reorder_point_uom: Option<String>,
    pub barcode: Option<String>,
    pub image_url: Option<String>,
    pub license_no: Option<String>,
    pub storage_condition: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub remark: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductCategory {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductUomConversion {
    pub id: Uuid,
    pub product_id: Uuid,
    pub uom: String,
    pub factor_to_base: Decimal,
}

/// 建立產品請求（SKU 由系統自動生成）
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub spec: Option<String>,
    pub category_code: Option<String>,
    pub subcategory_code: Option<String>,
    #[validate(length(min = 1, max = 20, message = "Base UOM must be 1-20 characters"))]
    pub base_uom: String,
    pub pack_unit: Option<String>,
    pub pack_qty: Option<i32>,
    #[serde(default)]
    pub track_batch: bool,
    #[serde(default)]
    pub track_expiry: bool,
    pub default_expiry_days: Option<i32>,
    pub safety_stock: Option<Decimal>,
    pub safety_stock_uom: Option<String>,
    pub reorder_point: Option<Decimal>,
    pub reorder_point_uom: Option<String>,
    pub barcode: Option<String>,
    pub image_url: Option<String>,
    pub license_no: Option<String>,
    pub storage_condition: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<String>,
    #[serde(default)]
    pub uom_conversions: Vec<UomConversionInput>,
}

/// 更新產品請求（SKU 不可修改）
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub spec: Option<String>,
    // 注意：category_code 和 subcategory_code 可更新，但不影響 SKU
    pub category_code: Option<String>,
    pub subcategory_code: Option<String>,
    pub pack_unit: Option<String>,
    pub pack_qty: Option<i32>,
    pub track_batch: Option<bool>,
    pub track_expiry: Option<bool>,
    pub default_expiry_days: Option<i32>,
    pub safety_stock: Option<Decimal>,
    pub safety_stock_uom: Option<String>,
    pub reorder_point: Option<Decimal>,
    pub reorder_point_uom: Option<String>,
    pub barcode: Option<String>,
    pub image_url: Option<String>,
    pub license_no: Option<String>,
    pub storage_condition: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub remark: Option<String>,
    pub is_active: Option<bool>,
    pub uom_conversions: Option<Vec<UomConversionInput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UomConversionInput {
    pub uom: String,
    pub factor_to_base: Decimal,
}

/// 產品查詢參數
#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub keyword: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub subcategory_code: Option<String>,
    pub status: Option<String>,
    pub track_batch: Option<bool>,
    pub track_expiry: Option<bool>,
    pub storage_condition: Option<String>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductWithUom {
    #[serde(flatten)]
    pub product: Product,
    pub uom_conversions: Vec<ProductUomConversion>,
    pub category_name: Option<String>,
    pub subcategory_name: Option<String>,
}

/// 產品列表回應（含類別名稱）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductListItem {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub spec: Option<String>,
    pub category_code: Option<String>,
    pub category_name: Option<String>,
    pub subcategory_code: Option<String>,
    pub subcategory_name: Option<String>,
    pub base_uom: String,
    pub safety_stock: Option<Decimal>,
    pub track_batch: bool,
    pub track_expiry: bool,
    pub status: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 50, message = "Code must be 1-50 characters"))]
    pub code: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// 重複產品檢查請求
#[derive(Debug, Deserialize)]
pub struct CheckDuplicateRequest {
    pub name: String,
    pub spec: String,
    pub category_code: String,
    pub subcategory_code: String,
}

/// 重複產品檢查回應
#[derive(Debug, Serialize)]
pub struct CheckDuplicateResponse {
    pub is_duplicate: bool,
    pub similar_products: Vec<SimilarProduct>,
}

#[derive(Debug, Serialize)]
pub struct SimilarProduct {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub spec: Option<String>,
    pub similarity: f64,
}

/// 產品狀態變更請求
#[derive(Debug, Deserialize)]
pub struct ChangeProductStatusRequest {
    pub status: String,
}

/// 產品圖片上傳回應
#[derive(Debug, Serialize)]
pub struct ProductImageUploadResponse {
    pub image_url: String,
}
