// 設備與校準紀錄 Models (實驗室 GLP)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Equipment {
    pub id: Uuid,
    pub name: String,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EquipmentCalibration {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    pub result: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CalibrationWithEquipment {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    pub result: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentQuery {
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CalibrationQuery {
    pub equipment_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEquipmentRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(max = 200))]
    pub model: Option<String>,
    #[validate(length(max = 100))]
    pub serial_number: Option<String>,
    #[validate(length(max = 200))]
    pub location: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEquipmentRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    #[validate(length(max = 200))]
    pub model: Option<String>,
    #[validate(length(max = 100))]
    pub serial_number: Option<String>,
    #[validate(length(max = 200))]
    pub location: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCalibrationRequest {
    pub equipment_id: Uuid,
    pub calibrated_at: NaiveDate,
    pub next_due_at: Option<NaiveDate>,
    #[validate(length(max = 50))]
    pub result: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCalibrationRequest {
    pub calibrated_at: Option<NaiveDate>,
    pub next_due_at: Option<NaiveDate>,
    #[validate(length(max = 50))]
    pub result: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}
