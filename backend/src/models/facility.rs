// 設施管理 Models
// 包含：Species, Facility, Building, Zone, Pen, Department

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================
// Species (物種)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Species {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 物種清單項目 — Species 外加「被幾隻動物引用」。
///
/// species_id 自 P1 起是動物種類的真相源，停用/刪除使用中的物種會讓那些動物的
/// 品種顯示變成孤兒，故清單一併帶出引用數供前端標示「使用中」並停用刪除鈕。
/// 計數**含軟刪除的動物**：牠們的明細與稽核紀錄仍要顯示物種名稱。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SpeciesListItem {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub animal_count: i64,
}

/// 物種清單查詢參數。
///
/// 預設只回啟用中的物種——動物表單的品種下拉吃同一支 API，停用物種不該混進去。
/// 物種管理頁需要 `include_inactive=true` 才能看到停用物種並重新啟用它。
#[derive(Debug, Default, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SpeciesListQuery {
    #[serde(default)]
    pub include_inactive: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSpeciesRequest {
    pub code: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSpeciesRequest {
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
    pub parent_id: Option<Uuid>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

// ============================================
// Facility (設施)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFacilityRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

#[derive(Debug, Serialize, FromRow, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBuildingRequest {
    pub facility_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

#[derive(Debug, Serialize, FromRow, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateZoneRequest {
    pub building_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub layout_config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    /// 欄位長度（公尺）；與 `width_m` 同時有值或同時為空
    pub length_m: Option<Decimal>,
    /// 欄位寬度（公尺）
    pub width_m: Option<Decimal>,
}

/// 欄位地板面積判定結果（只警示，不擋操作）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FloorAreaStatus {
    /// 所需面積不超過欄位面積
    Ok,
    /// 所需面積超過欄位面積
    Exceeded,
    /// 欄位沒有填長寬，無法判定
    NoArea,
}

/// 欄位地板面積檢視：依欄內動物的最新體重與隻數，對照最小地板面積表算出所需面積。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct PenFloorArea {
    pub status: FloorAreaStatus,
    /// 欄位面積（m²）；沒填長寬為空
    pub area_m2: Option<Decimal>,
    /// 欄內動物所需的最小地板面積合計（m²）；缺體重的動物不計入
    pub required_area_m2: Decimal,
    /// 欄內動物數（含缺體重者，決定適用單獨／2–5 隻／6 隻以上哪一段）
    pub animal_count: i64,
    /// 缺體重而無法計入所需面積的動物數
    pub animals_missing_weight: i64,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct PenDetails {
    pub id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub capacity: i32,
    pub current_count: i32,
    pub status: String,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub length_m: Option<Decimal>,
    pub width_m: Option<Decimal>,
    /// 由後端依體重與隻數算出，不在資料表內
    #[sqlx(skip)]
    pub floor_area: Option<PenFloorArea>,
    pub zone_id: Uuid,
    pub zone_code: String,
    pub zone_name: Option<String>,
    pub zone_color: Option<String>,
    pub zone_layout_config: Option<serde_json::Value>,
    pub building_id: Uuid,
    pub building_code: String,
    pub building_name: String,
    pub facility_id: Uuid,
    pub facility_code: String,
    pub facility_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePenRequest {
    pub zone_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    /// 欄位長度（公尺，最多兩位小數）；須與 `width_m` 一起給
    pub length_m: Option<Decimal>,
    /// 欄位寬度（公尺，最多兩位小數）
    pub width_m: Option<Decimal>,
}

/// 批次建立欄位請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchCreatePensRequest {
    pub zone_id: Uuid,
    /// 代碼前綴（如 "Q"）
    pub prefix: String,
    /// 建立數量
    pub count: i32,
    /// 排列模式："single"=單欄, "double"=兩欄並排
    #[serde(default = "default_pen_layout")]
    pub layout: String,
    /// 每欄容量
    #[serde(default = "default_pen_capacity")]
    pub capacity: i32,
    /// 每欄長度（公尺）；須與 `width_m` 一起給
    pub length_m: Option<Decimal>,
    /// 每欄寬度（公尺）
    pub width_m: Option<Decimal>,
}

fn default_pen_layout() -> String {
    "double".to_string()
}
fn default_pen_capacity() -> i32 {
    1
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePenRequest {
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub status: Option<String>,
    pub row_index: Option<i32>,
    pub col_index: Option<i32>,
    pub is_active: Option<bool>,
    /// 欄位長度（公尺）；須與 `width_m` 一起給，兩者都不給表示不改尺寸
    pub length_m: Option<Decimal>,
    /// 欄位寬度（公尺）
    pub width_m: Option<Decimal>,
}

/// 把同一個尺寸套用到整個區內所有啟用中的欄位。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyPenSizeRequest {
    pub length_m: Decimal,
    pub width_m: Decimal,
    /// 使用者確認預覽時看到的啟用中欄位 ID。有給時，伺服器發現區內實際的啟用欄位集合與此不同
    /// 就拒絕（409），讓使用者重新確認；不給則維持舊行為（套用到區內所有啟用中的欄位）
    #[serde(default)]
    pub expected_pen_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplyPenSizeResult {
    /// 實際更新的欄位數
    pub updated_count: i64,
}

/// 放入預覽的查詢參數：問「再放一隻這個體重的動物，欄位會不會超標」。
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PenAreaPreviewQuery {
    /// 預計放入動物的體重（公斤）；不給就回目前狀態
    pub add_weight_kg: Option<Decimal>,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

#[derive(Debug, Serialize, FromRow, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub config: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
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
        let req: CreateSpeciesRequest =
            serde_json::from_str(json).expect("反序列化 CreateSpeciesRequest 失敗");
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
        let req: CreateFacilityRequest =
            serde_json::from_str(json).expect("反序列化 CreateFacilityRequest 失敗");
        assert_eq!(req.code, "FAC-001");
        assert_eq!(req.name, "動物試驗中心");
        assert_eq!(req.phone.as_deref(), Some("06-1234567"));
    }

    #[test]
    fn test_pen_query_defaults() {
        let json = r#"{}"#;
        let query: PenQuery = serde_json::from_str(json).expect("反序列化 PenQuery 失敗");
        assert!(query.zone_id.is_none());
        assert!(query.building_id.is_none());
        assert!(query.facility_id.is_none());
        assert!(query.status.is_none());
        assert!(query.is_active.is_none());
    }

    #[test]
    fn test_create_pen_request() {
        let zone_id = Uuid::new_v4();
        let json = format!(
            r#"{{
            "zone_id": "{}",
            "code": "P-001",
            "name": "1號欄",
            "capacity": 10
        }}"#,
            zone_id
        );
        let req: CreatePenRequest =
            serde_json::from_str(&json).expect("反序列化 CreatePenRequest 失敗");
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
        let req: CreateDepartmentRequest =
            serde_json::from_str(json).expect("反序列化 CreateDepartmentRequest 失敗");
        assert_eq!(req.code, "DEPT-001");
        assert_eq!(req.name, "研發部");
        assert!(req.parent_id.is_none());
    }

    #[test]
    fn test_update_pen_optional_fields() {
        let json = r#"{"capacity": 20}"#;
        let req: UpdatePenRequest =
            serde_json::from_str(json).expect("反序列化 UpdatePenRequest 失敗");
        assert_eq!(req.capacity, Some(20));
        assert!(req.name.is_none());
        assert!(req.status.is_none());
        assert!(req.is_active.is_none());
    }
}
