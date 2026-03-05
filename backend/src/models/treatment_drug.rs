// 治療方式藥物選項 Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 治療方式藥物選項
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TreatmentDrugOption {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub default_dosage_unit: Option<String>,
    pub available_units: Option<Vec<String>>,
    pub default_dosage_value: Option<String>,
    pub erp_product_id: Option<Uuid>,
    pub category: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 建立藥物選項請求
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateTreatmentDrugRequest {
    #[validate(length(min = 1, max = 200, message = "藥品名稱須為 1-200 字元"))]
    pub name: String,
    pub display_name: Option<String>,
    #[validate(length(max = 20, message = "劑量單位最多 20 字元"))]
    pub default_dosage_unit: Option<String>,
    pub available_units: Option<Vec<String>>,
    pub default_dosage_value: Option<String>,
    pub erp_product_id: Option<Uuid>,
    #[validate(length(max = 50, message = "分類最多 50 字元"))]
    pub category: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
}

/// 更新藥物選項請求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTreatmentDrugRequest {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub default_dosage_unit: Option<String>,
    pub available_units: Option<Vec<String>>,
    pub default_dosage_value: Option<String>,
    pub erp_product_id: Option<Uuid>,
    pub category: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 藥物選項查詢參數
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TreatmentDrugQuery {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

/// 從 ERP 匯入請求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ImportFromErpRequest {
    pub product_ids: Vec<Uuid>,
    pub category: Option<String>,
}
