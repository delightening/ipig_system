use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "partner_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PartnerType {
    Supplier,
    Customer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "supplier_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SupplierCategory {
    Drug,        // 藥物
    Consumable,  // 耗材
    Feed,        // 飼料
    Equipment,   // 儀器
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Partner {
    pub id: Uuid,
    pub partner_type: PartnerType,
    pub code: String,
    pub name: String,
    pub supplier_category: Option<SupplierCategory>,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePartnerRequest {
    pub partner_type: PartnerType,
    pub code: Option<String>,  // 改為可選，如果為空則自動生成
    pub supplier_category: Option<SupplierCategory>,
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePartnerRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub tax_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PartnerQuery {
    pub partner_type: Option<PartnerType>,
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GenerateCodeResponse {
    pub code: String,
}
