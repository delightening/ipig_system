// 設施管理 Models
// 包含：Species, Facility, Building, Zone, Pen, Department

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Species (物種)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Species {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpeciesRequest {
    pub code: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSpeciesRequest {
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

// ============================================
// Facility (設施)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Facility {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacilityRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacilityRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
}

// ============================================
// Building (棟舍)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Building {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BuildingWithFacility {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub facility_code: String,
    pub facility_name: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateBuildingRequest {
    pub facility_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBuildingRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

// ============================================
// Zone (區域)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Zone {
    pub id: Uuid,
    pub building_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub is_active: bool,
    pub layout_config: Option<serde_json::Value>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ZoneWithBuilding {
    pub id: Uuid,
    pub building_id: Uuid,
    pub building_code: String,
    pub building_name: String,
    pub facility_id: Uuid,
    pub facility_name: String,
    pub code: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub is_active: bool,
    pub layout_config: Option<serde_json::Value>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateZoneRequest {
    pub building_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub layout_config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateZoneRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
    pub layout_config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

// ============================================
// Pen (欄位)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pen {
    pub id: Uuid,
    pub zone_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub capacity: i32,
    pub current_count: i32,
    pub status: String,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PenDetails {
    pub id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub capacity: i32,
    pub current_count: i32,
    pub status: String,
    pub zone_id: Uuid,
    pub zone_code: String,
    pub zone_name: Option<String>,
    pub zone_color: Option<String>,
    pub building_id: Uuid,
    pub building_code: String,
    pub building_name: String,
    pub facility_id: Uuid,
    pub facility_code: String,
    pub facility_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePenRequest {
    pub zone_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePenRequest {
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub status: Option<String>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PenQuery {
    pub zone_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub facility_id: Option<Uuid>,
    pub status: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================
// Department (部門)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DepartmentWithManager {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub parent_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}
