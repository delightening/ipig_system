use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use super::enums::*;

/// 動物來源
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AnimalSource {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 無敏感欄位，空 impl 即可（see AuditRedact trait doc）
impl crate::models::audit_diff::AuditRedact for AnimalSource {}

/// 動物主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Animal {
    pub id: Uuid,
    pub animal_no: Option<String>, // 動物編號（由使用者命名）
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
    pub deletion_reason: Option<String>, // GLP: 刪除原因
    pub vet_weight_viewed_at: Option<DateTime<Utc>>,
    pub vet_vaccine_viewed_at: Option<DateTime<Utc>>,
    pub vet_sacrifice_viewed_at: Option<DateTime<Utc>>,
    pub vet_last_viewed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub animal_id: Option<Uuid>,              // 動物 ID
    pub breed_other: Option<String>,          // 其他品種說明
    #[sqlx(default)]
    pub pen_id: Option<Uuid>,                 // 欄位 FK → pens(id)
    #[sqlx(default)]
    pub species_id: Option<Uuid>,             // 物種 FK → species(id)
    pub experiment_assigned_by: Option<Uuid>, // 分配至實驗的操作者
    #[sqlx(default)]
    pub experiment_assigned_by_name: Option<String>, // 分配者名稱（JOIN 查詢時填入）
}

/// 觀察試驗紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    // 軟刪除（SELECT/RETURNING 需對應，FromRow 不允許多餘欄位）
    #[sqlx(default)]
    pub deleted_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub deletion_reason: Option<String>,
    #[sqlx(default)]
    pub deleted_by: Option<Uuid>,
    // 緊急給藥相關欄位
    #[sqlx(default)]
    pub is_emergency: Option<bool>,
    #[sqlx(default)]
    pub emergency_status: Option<String>, // pending_review, approved, rejected
    #[sqlx(default)]
    pub emergency_reason: Option<String>,
    #[sqlx(default)]
    pub reviewed_by: Option<Uuid>,
    #[sqlx(default)]
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    #[sqlx(default)]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Optimistic locking (migration 014)
    #[sqlx(default)]
    pub version: Option<i32>,
}

/// ⚠️ **R26-9 警告**：`AnimalObservation.content` / `equipment_used` / `treatments` /
/// `remark` / `emergency_reason` 為自由文字 / JSON，含醫療細節。**目前**採空 impl
/// 允許 audit diff 記錄完整內容（對稽核員為必要），但 R26-9（CodeRabbit PR #156 Major）
/// 建議改為 allowlist 或 summary log，避免醫療資料過度暴露在 audit UI。
/// 在 R26-9 完成前，此實作與其他 animal entity（source/weight）一致採最簡模式。
impl crate::models::audit_diff::AuditRedact for AnimalObservation {}

/// 手術紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    #[sqlx(default)]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// R26-9 warning 同 AnimalObservation：手術紀錄含 JSONB 麻醉/藥物/vital signs
// 等醫療內容；採空 impl 允許 audit diff 記錄完整內容。R26-9 若決定降敏再覆寫。
impl crate::models::audit_diff::AuditRedact for AnimalSurgery {}

/// 體重紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AnimalWeight {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// 無敏感欄位（體重數值、測量日期）；空 impl 即可。
impl crate::models::audit_diff::AuditRedact for AnimalWeight {}

/// 體重紀錄回應（含建立者名稱）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AnimalVaccination {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub administered_date: NaiveDate,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// 無敏感欄位（疫苗名、驅蟲劑量、給藥日期為研究紀錄本身）；空 impl 即可。
impl crate::models::audit_diff::AuditRedact for AnimalVaccination {}

/// 犧牲/採樣紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

/// 猝死記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalSuddenDeath {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub discovered_at: DateTime<Utc>,
    pub discovered_by: Uuid,
    pub probable_cause: Option<String>,
    pub iacuc_no: Option<String>,
    pub location: Option<String>,
    pub remark: Option<String>,
    pub requires_pathology: bool,
    pub created_at: DateTime<Utc>,
}

/// 動物轉讓記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AnimalTransfer {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub from_iacuc_no: String,
    pub to_iacuc_no: Option<String>,
    pub status: super::AnimalTransferStatus,
    /// 轉讓類型：external = 轉給其他機構（完成時清空欄位），internal = 仍在機構內（保留欄位）
    #[serde(default = "default_transfer_type_entity")]
    pub transfer_type: String,
    pub initiated_by: Uuid,
    pub reason: String,
    pub remark: Option<String>,
    pub rejected_by: Option<Uuid>,
    pub rejected_reason: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_transfer_type_entity() -> String {
    "internal".to_string()
}

// 轉讓流程的 reason / remark / rejected_reason 為自由文字；依 R26-9 方針暫時
// 全保留於 audit log（GLP 需完整變更軌跡）。若後續決定降敏再覆寫。
impl crate::models::audit_diff::AuditRedact for AnimalTransfer {}

/// 轉讓獸醫評估記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TransferVetEvaluation {
    pub id: Uuid,
    pub transfer_id: Uuid,
    pub vet_id: Uuid,
    pub health_status: String,
    pub is_fit_for_transfer: bool,
    pub conditions: Option<String>,
    pub evaluated_at: DateTime<Utc>,
}

// 獸醫評估含 health_status / conditions 自由文字醫療判斷；同 AnimalObservation
// R26-9 方針，空 allowlist。
impl crate::models::audit_diff::AuditRedact for TransferVetEvaluation {}

/// 病理組織報告
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

/// 動物狀態統計（輕量級，僅 COUNT）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnimalStatsResponse {
    /// 各狀態的動物數量
    pub status_counts: std::collections::HashMap<String, i64>,
    /// 有欄位的動物數量
    pub pen_animals_count: i64,
    /// 全部動物總數
    pub total: i64,
}
