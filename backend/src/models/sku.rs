use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

/// SKU 主類別
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkuCategory {
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// SKU 子類別
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkuSubcategory {
    pub id: i32,
    pub category_code: String,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// SKU 流水號
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkuSequence {
    pub category_code: String,
    pub subcategory_code: String,
    pub last_sequence: i32,
}

/// SKU 片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkuSegment {
    pub code: String,
    pub label: String,
    pub value: String,
    pub source: String,
}

/// SKU 預覽請求
#[derive(Debug, Deserialize)]
pub struct SkuPreviewRequest {
    pub org: Option<String>,
    pub cat: String,
    pub sub: String,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    pub pack: PackInfo,
    pub source: String,
    pub rule_version_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackInfo {
    pub uom: String,
    pub qty: i32,
}

/// SKU 預覽回應
#[derive(Debug, Serialize)]
pub struct SkuPreviewResponse {
    pub preview_sku: String,
    pub segments: Vec<SkuSegment>,
    pub rule_version: String,
    pub rule_updated_at: Option<String>,
}

/// SKU 預覽錯誤
#[derive(Debug, Serialize)]
pub struct SkuPreviewError {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub field: Option<String>,
}

/// 取得類別選項回應
#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryOption>,
}

#[derive(Debug, Serialize)]
pub struct CategoryOption {
    pub code: String,
    pub name: String,
}

/// 取得子類別選項回應
#[derive(Debug, Serialize)]
pub struct SubcategoriesResponse {
    pub category: CategoryOption,
    pub subcategories: Vec<CategoryOption>,
}

/// 生成 SKU 請求
#[derive(Debug, Deserialize)]
pub struct GenerateSkuRequest {
    pub category: String,
    pub subcategory: String,
}

/// 生成 SKU 回應
#[derive(Debug, Serialize)]
pub struct GenerateSkuResponse {
    pub sku: String,
    pub category: CategoryOption,
    pub subcategory: CategoryOption,
    pub sequence: i32,
}

/// 驗證 SKU 請求
#[derive(Debug, Deserialize)]
pub struct ValidateSkuRequest {
    pub sku: String,
}

/// 驗證 SKU 回應
#[derive(Debug, Serialize)]
pub struct ValidateSkuResponse {
    pub valid: bool,
    pub category: Option<CategoryOption>,
    pub subcategory: Option<CategoryOption>,
    pub sequence: Option<i32>,
    pub exists: bool,
    pub error: Option<String>,
}

/// 擴展的產品創建請求（包含 SKU 生成所需資訊）
#[derive(Debug, Deserialize)]
pub struct CreateProductWithSkuRequest {
    pub name: Option<String>,
    pub spec: Option<String>,
    pub base_uom: String,
    pub pack_unit: Option<String>,
    pub pack_qty: Option<i32>,
    #[serde(default)]
    pub track_batch: bool,
    #[serde(default)]
    pub track_expiry: bool,
    pub safety_stock: Option<rust_decimal::Decimal>,
    pub reorder_point: Option<rust_decimal::Decimal>,
    pub category_code: String,
    pub subcategory_code: String,
    pub source_code: String,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}
