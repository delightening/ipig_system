use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::*;

/// 動物來源
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalSource {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 動物主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Animal {
    pub id: Uuid,
    pub animal_no: Option<String>,  // 動物編號（由使用者命名）
    pub ear_tag: String,
    pub status: AnimalStatus,
    pub breed: AnimalBreed,
    pub source_id: Option<Uuid>,
    pub gender: AnimalGender,
    pub birth_date: Option<NaiveDate>,
    pub entry_date: NaiveDate,
    pub entry_weight: Option<rust_decimal::Decimal>,
    pub pen_location: Option<String>,
    pub pre_experiment_code: Option<String>,
    pub iacuc_no: Option<String>,
    pub experiment_date: Option<NaiveDate>,
    pub remark: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub deletion_reason: Option<String>,  // GLP: 刪除原因
    pub vet_weight_viewed_at: Option<DateTime<Utc>>,
    pub vet_vaccine_viewed_at: Option<DateTime<Utc>>,
    pub vet_sacrifice_viewed_at: Option<DateTime<Utc>>,
    pub vet_last_viewed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub animal_id: Option<Uuid>,  // 動物 ID
    pub breed_other: Option<String>,  // 其他品種說明
    pub experiment_assigned_by: Option<Uuid>,  // 分配至實驗的操作者
    #[sqlx(default)]
    pub experiment_assigned_by_name: Option<String>,  // 分配者名稱（JOIN 查詢時填入）
}

/// 觀察試驗紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalObservation {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub event_date: NaiveDate,
    pub record_type: RecordType,
    pub equipment_used: Option<serde_json::Value>,
    pub anesthesia_start: Option<DateTime<Utc>>,
    pub anesthesia_end: Option<DateTime<Utc>>,
    pub content: String,
    pub no_medication_needed: bool,
    pub stop_medication: bool,
    pub treatments: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // 緊急給藥相關欄位
    #[sqlx(default)]
    pub is_emergency: Option<bool>,
    #[sqlx(default)]
    pub emergency_status: Option<String>,  // pending_review, approved, rejected
    #[sqlx(default)]
    pub emergency_reason: Option<String>,
    #[sqlx(default)]
    pub reviewed_by: Option<Uuid>,
    #[sqlx(default)]
    pub reviewed_at: Option<DateTime<Utc>>,
}

/// 手術紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalSurgery {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub is_first_experiment: bool,
    pub surgery_date: NaiveDate,
    pub surgery_site: String,
    pub induction_anesthesia: Option<serde_json::Value>,
    pub pre_surgery_medication: Option<serde_json::Value>,
    pub positioning: Option<String>,
    pub anesthesia_maintenance: Option<serde_json::Value>,
    pub anesthesia_observation: Option<String>,
    pub vital_signs: Option<serde_json::Value>,
    pub reflex_recovery: Option<String>,
    pub respiration_rate: Option<i32>,
    pub post_surgery_medication: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub no_medication_needed: bool,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 體重紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalWeight {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 體重紀錄回應（含建立者名稱）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalWeightResponse {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
    pub created_by: Option<Uuid>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}


/// 疫苗/驅蟲紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalVaccination {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub administered_date: NaiveDate,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 犧牲/採樣紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalSacrifice {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub sacrifice_date: Option<NaiveDate>,
    pub zoletil_dose: Option<String>,
    pub method_electrocution: bool,
    pub method_bloodletting: bool,
    pub method_other: Option<String>,
    pub sampling: Option<String>,
    pub sampling_other: Option<String>,
    pub blood_volume_ml: Option<rust_decimal::Decimal>,
    pub confirmed_sacrifice: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 病理組織報告
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalPathologyReport {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 血液檢查項目模板
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BloodTestTemplate {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub default_unit: Option<String>,
    pub reference_range: Option<String>,
    pub default_price: Option<rust_decimal::Decimal>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 血液檢查主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalBloodTest {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub status: String,
    pub remark: Option<String>,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub delete_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 血液檢查項目明細
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalBloodTestItem {
    pub id: Uuid,
    pub blood_test_id: Uuid,
    pub template_id: Option<Uuid>,
    pub item_name: String,
    pub result_value: Option<String>,
    pub result_unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_abnormal: bool,
    pub remark: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// 血液檢查詳情（含明細項目）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimalBloodTestWithItems {
    pub blood_test: AnimalBloodTest,
    pub items: Vec<AnimalBloodTestItem>,
    pub created_by_name: Option<String>,
}

/// 血液檢查列表項目
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BloodTestListItem {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub status: String,
    pub remark: Option<String>,
    pub vet_read: bool,
    pub item_count: Option<i64>,
    pub abnormal_count: Option<i64>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
