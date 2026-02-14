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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_serde() {
        let json = r#"{
            "code": "PIG",
            "name": "豬",
            "name_en": "Pig",
            "icon": null,
            "config": null,
            "sort_order": 1
        }"#;
        let req: CreateSpeciesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.code, "PIG");
        assert_eq!(req.name, "豬");
        assert_eq!(req.name_en.as_deref(), Some("Pig"));
        assert_eq!(req.sort_order, Some(1));
    }

    #[test]
    fn test_facility_serde() {
        let json = r#"{
            "code": "FAC-001",
            "name": "動物試驗中心",
            "address": "台南市中西區",
            "phone": "06-1234567",
            "contact_person": "王小明",
            "config": null
        }"#;
        let req: CreateFacilityRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.code, "FAC-001");
        assert_eq!(req.name, "動物試驗中心");
        assert_eq!(req.phone.as_deref(), Some("06-1234567"));
    }

    #[test]
    fn test_pen_query_defaults() {
        let json = r#"{}"#;
        let query: PenQuery = serde_json::from_str(json).unwrap();
        assert!(query.zone_id.is_none());
        assert!(query.building_id.is_none());
        assert!(query.facility_id.is_none());
        assert!(query.status.is_none());
        assert!(query.is_active.is_none());
    }

    #[test]
    fn test_create_pen_request() {
        let zone_id = Uuid::new_v4();
        let json = format!(r#"{{
            "zone_id": "{}",
            "code": "P-001",
            "name": "1號欄",
            "capacity": 10
        }}"#, zone_id);
        let req: CreatePenRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.zone_id, zone_id);
        assert_eq!(req.code, "P-001");
        assert_eq!(req.capacity, Some(10));
    }

    #[test]
    fn test_department_serde() {
        let json = r#"{
            "code": "DEPT-001",
            "name": "研發部",
            "parent_id": null,
            "manager_id": null,
            "config": null,
            "sort_order": 1
        }"#;
        let req: CreateDepartmentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.code, "DEPT-001");
        assert_eq!(req.name, "研發部");
        assert!(req.parent_id.is_none());
    }

    #[test]
    fn test_update_pen_optional_fields() {
        let json = r#"{"capacity": 20}"#;
        let req: UpdatePenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.capacity, Some(20));
        assert!(req.name.is_none());
        assert!(req.status.is_none());
        assert!(req.is_active.is_none());
    }
}