use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use utoipa::ToSchema;

/// SKU 主類別
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SkuCategory {
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// 無敏感欄位
impl crate::models::audit_diff::AuditRedact for SkuCategory {}

/// SKU 子類別
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SkuSubcategory {
    pub id: i32,
    pub category_code: String,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// 無敏感欄位
impl crate::models::audit_diff::AuditRedact for SkuSubcategory {}

/// SKU 流水號
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkuSequence {
    pub category_code: String,
    pub subcategory_code: String,
    pub last_sequence: i32,
}

/// SKU 片段
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SkuSegment {
    pub code: String,
    pub label: String,
    pub value: String,
    pub source: String,
}

/// SKU 預覽請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SkuPreviewRequest {
    pub org: Option<String>,
    pub cat: String,
    pub sub: String,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    pub pack: PackInfo,
    pub source: String,
    pub rule_version_hint: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PackInfo {
    pub uom: String,
    pub qty: i32,
}

/// SKU 預覽回應
#[derive(Debug, Serialize, ToSchema)]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryOption>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryOption {
    pub code: String,
    pub name: String,
}

/// 取得子類別選項回應
#[derive(Debug, Serialize, ToSchema)]
pub struct SubcategoriesResponse {
    pub category: CategoryOption,
    pub subcategories: Vec<CategoryOption>,
}

/// 編輯用：子類別（含 sort_order, is_active）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubcategoryForEdit {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
}

/// 編輯用：品類（含子類與 sort_order, is_active）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryForEdit {
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub subcategories: Vec<SubcategoryForEdit>,
}

/// 編輯分類用：完整品類樹
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoriesTreeResponse {
    pub categories: Vec<CategoryForEdit>,
}

/// 更新品類請求（名稱、排序、啟用狀態）
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSkuCategoryRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 更新子類請求（名稱、排序、啟用狀態）
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSkuSubcategoryRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 新增子類請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSkuSubcategoryRequest {
    /// 子類代碼，3 碼大寫英數字
    pub code: String,
    /// 顯示名稱
    pub name: String,
    /// 排序，預設 0
    pub sort_order: Option<i32>,
    /// 是否啟用，預設 true
    pub is_active: Option<bool>,
}

/// 生成 SKU 請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateSkuRequest {
    pub category: String,
    pub subcategory: String,
}

/// 生成 SKU 回應
#[derive(Debug, Serialize, ToSchema)]
pub struct GenerateSkuResponse {
    pub sku: String,
    pub category: CategoryOption,
    pub subcategory: CategoryOption,
    pub sequence: i32,
}

/// 驗證 SKU 請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidateSkuRequest {
    pub sku: String,
}

/// 驗證 SKU 回應
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateSkuResponse {
    pub valid: bool,
    pub category: Option<CategoryOption>,
    pub subcategory: Option<CategoryOption>,
    pub sequence: Option<i32>,
    pub exists: bool,
    pub error: Option<String>,
}

/// 擴展的產品創建請求（包含 SKU 生成所需資訊）
#[derive(Debug, Deserialize, ToSchema)]
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
