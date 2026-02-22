use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateWarehouseRequest {
    pub code: Option<String>,
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateWarehouseRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WarehouseQuery {
    pub keyword: Option<String>,
    pub is_active: Option<bool>,
}
