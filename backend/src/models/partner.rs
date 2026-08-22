use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "partner_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PartnerType {
    Supplier,
    Customer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "supplier_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SupplierCategory {
    Drug,       // 藥物
    Consumable, // 耗材
    Feed,       // 飼料
    Equipment,  // 儀器
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "customer_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CustomerCategory {
    Internal, // 內部單位
    External, // 外部客戶
    Research, // 研究計畫
    Other,    // 其他
}

// 合作夥伴主檔（供應商/客戶），無敏感欄位（tax_id / phone / email 皆稽核必要）
impl crate::models::audit_diff::AuditRedact for Partner {}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Partner {
    pub id: Uuid,
    pub partner_type: PartnerType,
    pub code: String,
    pub name: String,
    pub supplier_category: Option<SupplierCategory>,
    #[sqlx(default)]
    pub customer_category: Option<CustomerCategory>,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePartnerRequest {
    pub partner_type: PartnerType,
    pub code: Option<String>, // 改為可選，如果為空則自動生成
    pub supplier_category: Option<SupplierCategory>,
    pub customer_category: Option<CustomerCategory>,
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePartnerRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct PartnerQuery {
    pub partner_type: Option<PartnerType>,
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenerateCodeResponse {
    pub code: String,
}

/// 夥伴匯入 CSV 列
#[derive(Debug, Clone, Default)]
pub struct PartnerImportRow {
    pub partner_type: String,
    pub name: String,
    pub supplier_category: Option<String>,
    pub customer_category: Option<String>,
    pub code: Option<String>,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
}

/// 夥伴匯入錯誤明細
#[derive(Debug, Serialize, ToSchema)]
pub struct PartnerImportErrorDetail {
    pub row: i32,
    pub code: Option<String>,
    pub error: String,
}

/// 夥伴匯入結果
#[derive(Debug, Serialize, ToSchema)]
pub struct PartnerImportResult {
    pub success_count: i32,
    pub error_count: i32,
    pub errors: Vec<PartnerImportErrorDetail>,
}
