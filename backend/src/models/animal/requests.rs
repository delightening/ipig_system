use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::entities::*;
use super::enums::*;

// ============================================
// Request/Response DTOs
// ============================================

/// 驗證耳號必須為三位數
pub(crate) fn validate_ear_tag(ear_tag: &str) -> Result<(), validator::ValidationError> {
    // 如果是數字，格式化為三位數後檢查
    let formatted = if let Ok(num) = ear_tag.parse::<u32>() {
        format!("{:03}", num)
    } else {
        ear_tag.to_string()
    };

    // 檢查是否為三位數字
    if formatted.len() == 3 && formatted.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("ear_tag_three_digits"))
    }
}

/// 驗證欄位必須填寫（當值存在時，不能為空字串）
pub(crate) fn validate_pen_location(pen_location: &str) -> Result<(), validator::ValidationError> {
    if pen_location.trim().is_empty() {
        let mut error = validator::ValidationError::new("pen_location_required");
        error.message = Some(std::borrow::Cow::Borrowed("欄位為必填"));
        Err(error)
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAnimalRequest {
    #[validate(length(min = 1, max = 10, message = "Ear tag must be 1-10 characters"))]
    #[validate(custom(function = "validate_ear_tag", message = "耳號必須為三位數"))]
    pub ear_tag: String,
    pub breed: AnimalBreed,
    pub breed_other: Option<String>,
    pub source_id: Option<Uuid>,
    pub gender: AnimalGender,
    pub birth_date: Option<NaiveDate>,
    pub entry_date: NaiveDate,
    pub entry_weight: rust_decimal::Decimal,
    #[validate(required(message = "欄位為必填"))]
    #[validate(custom(function = "validate_pen_location", message = "欄位不能為空"))]
    pub pen_location: Option<String>,
    pub pen_id: Option<Uuid>,
    pub species_id: Option<Uuid>,
    pub pre_experiment_code: Option<String>,
    pub remark: Option<String>,
    /// 強制建立（跳過耳號重複警告，但同耳號同出生日期仍會阻擋）
    #[serde(default)]
    pub force_create: bool,
}

#[derive(Debug, Deserialize, Validate, Default, ToSchema)]
pub struct UpdateAnimalRequest {
    // 以下欄位於建立後不可更改，已從更新請求中移除：
    // - ear_tag (耳號)
    // - breed (品種)
    // - gender (性別)
    // - source_id (來源)
    // - birth_date (出生日期)
    // - entry_date (進場日期)
    // - entry_weight (進場體重)
    // - pre_experiment_code (實驗前代號)
    pub status: Option<AnimalStatus>,
    pub pen_location: Option<String>,
    pub pen_id: Option<Uuid>,
    pub species_id: Option<Uuid>,
    pub iacuc_no: Option<String>,
    pub experiment_date: Option<NaiveDate>,
    pub remark: Option<String>,
    /// Optimistic locking: if provided, update only succeeds when DB version matches
    pub version: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnimalQuery {
    pub status: Option<AnimalStatus>,
    pub breed: Option<AnimalBreed>,
    pub gender: Option<AnimalGender>,
    pub iacuc_no: Option<String>,
    pub pen_location: Option<String>,
    pub keyword: Option<String>,
    pub is_on_medication: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 資料隔離：用於記錄列表查詢的可選時間過濾
#[derive(Debug, Deserialize, Default, ToSchema, utoipa::IntoParams)]
pub struct RecordFilterQuery {
    /// 僅回傳 created_at > after 的記錄（轉讓資料隔離）
    pub after: Option<DateTime<Utc>>,
}

/// 資料隔離：回傳當前使用者的可見時間界線
#[derive(Debug, Serialize, ToSchema)]
pub struct DataBoundaryResponse {
    /// 若為 Some，僅應顯示 created_at > boundary 的記錄；null 代表可見所有
    pub boundary: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchAssignRequest {
    pub animal_ids: Vec<Uuid>,
    pub iacuc_no: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateObservationRequest {
    pub event_date: NaiveDate,
    pub record_type: RecordType,
    pub equipment_used: Option<serde_json::Value>,
    pub anesthesia_start: Option<DateTime<Utc>>,
    pub anesthesia_end: Option<DateTime<Utc>>,
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
    #[serde(default)]
    pub no_medication_needed: bool,
    pub treatments: Option<serde_json::Value>,
    pub remark: Option<String>,
    // 緊急給藥 (當獸醫不在時，可先執行並待補簽)
    #[serde(default)]
    pub is_emergency: bool,
    pub emergency_reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateSurgeryRequest {
    #[serde(default = "default_true")]
    pub is_first_experiment: bool,
    pub surgery_date: NaiveDate,
    #[validate(length(min = 1, max = 200, message = "Surgery site is required"))]
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
    #[serde(default)]
    pub no_medication_needed: bool,
}

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateWeightRequest {
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateVaccinationRequest {
    pub administered_date: NaiveDate,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSacrificeRequest {
    pub sacrifice_date: Option<NaiveDate>,
    pub zoletil_dose: Option<String>,
    #[serde(default)]
    pub method_electrocution: bool,
    #[serde(default)]
    pub method_bloodletting: bool,
    pub method_other: Option<String>,
    pub sampling: Option<String>,
    pub sampling_other: Option<String>,
    pub blood_volume_ml: Option<rust_decimal::Decimal>,
    #[serde(default)]
    pub confirmed_sacrifice: bool,
}

/// 猝死登記請求
#[derive(Debug, Deserialize)]
pub struct CreateSuddenDeathRequest {
    pub discovered_at: DateTime<Utc>,
    pub probable_cause: Option<String>,
    pub location: Option<String>,
    pub remark: Option<String>,
    #[serde(default)]
    pub requires_pathology: bool,
}

/// 發起轉讓請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransferRequest {
    pub reason: String,
    pub remark: Option<String>,
    /// 轉讓類型：external = 轉給其他機構（完成時清空欄位），internal = 仍在機構內（保留欄位）。預設 internal。
    #[serde(default = "default_transfer_type")]
    pub transfer_type: String,
}

fn default_transfer_type() -> String {
    "internal".to_string()
}

/// 獸醫評估轉讓請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct VetEvaluateTransferRequest {
    pub health_status: String,
    pub is_fit_for_transfer: bool,
    pub conditions: Option<String>,
}

/// 指定新計劃請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignTransferPlanRequest {
    pub to_iacuc_no: String,
}

/// 拒絕轉讓請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectTransferRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateVetRecommendationRequest {
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
    #[serde(default)]
    pub is_urgent: bool, // 是否為緊急建議（影響通知管道）
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnimalSourceRequest {
    #[validate(length(min = 1, max = 20, message = "Code must be 1-20 characters"))]
    pub code: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnimalSourceRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// 動物列表項目（含來源名稱）
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct AnimalListItem {
    pub id: Uuid,
    pub animal_no: Option<String>, // 動物編號（由使用者命名）
    pub ear_tag: String,
    pub status: AnimalStatus,
    pub breed: AnimalBreed,
    pub breed_other: Option<String>,
    pub gender: AnimalGender,
    pub pen_location: Option<String>,
    pub pen_id: Option<Uuid>,
    pub species_id: Option<Uuid>,
    #[sqlx(default)]
    pub species_name: Option<String>,
    pub iacuc_no: Option<String>,
    pub entry_date: NaiveDate,
    pub source_name: Option<String>,
    pub vet_last_viewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub has_abnormal_record: Option<bool>,
    #[sqlx(default)]
    pub is_on_medication: Option<bool>,
    #[sqlx(default)]
    pub vet_recommendation_date: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub latest_weight: Option<rust_decimal::Decimal>,
    #[sqlx(default)]
    pub latest_weight_date: Option<NaiveDate>,
}

/// 依欄位分組的動物
#[derive(Debug, Serialize, ToSchema)]
pub struct AnimalsByPen {
    pub pen_location: String,
    pub animals: Vec<AnimalListItem>,
}

// ============================================
// 匯入匯出相關類型
// ============================================

/// 匯入狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "import_status", rename_all = "snake_case")]
pub enum ImportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// 匯入類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "import_type", rename_all = "snake_case")]
pub enum ImportType {
    AnimalBasic,
    AnimalWeight,
}

/// 匯出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "export_format", rename_all = "snake_case")]
pub enum ExportFormat {
    Pdf,
    Excel,
}

/// 匯出類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "export_type", rename_all = "snake_case")]
pub enum ExportType {
    MedicalSummary,
    ObservationRecords,
    SurgeryRecords,
    ExperimentRecords,
}

/// 匯入批次記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalImportBatch {
    pub id: Uuid,
    pub import_type: ImportType,
    pub file_name: String,
    pub total_rows: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub status: ImportStatus,
    pub error_details: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 匯出記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalExportRecord {
    pub id: Uuid,
    pub animal_id: Option<Uuid>,
    pub iacuc_no: Option<String>,
    pub export_type: ExportType,
    pub export_format: ExportFormat,
    pub file_path: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 紀錄版本歷史
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecordVersion {
    pub id: Uuid,
    pub record_type: String,
    pub record_id: Uuid,
    pub version_no: i32,
    pub snapshot: serde_json::Value,
    pub diff_summary: Option<String>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
}

/// 匯入錯誤詳情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportErrorDetail {
    pub row: i32,
    pub ear_tag: Option<String>,
    pub error: String,
}

/// 匯入結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub batch_id: Uuid,
    pub total_rows: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub errors: Vec<ImportErrorDetail>,
}

// ============================================
// 新增 Request/Response DTOs
// ============================================

/// 更新觀察紀錄請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateObservationRequest {
    pub event_date: Option<NaiveDate>,
    pub record_type: Option<RecordType>,
    pub equipment_used: Option<serde_json::Value>,
    pub anesthesia_start: Option<DateTime<Utc>>,
    pub anesthesia_end: Option<DateTime<Utc>>,
    pub content: Option<String>,
    pub no_medication_needed: Option<bool>,
    pub treatments: Option<serde_json::Value>,
    pub remark: Option<String>,
    // 緊急給藥審核 (VET/PI 可審核)
    pub emergency_status: Option<String>, // approved, rejected
}

/// 更新手術紀錄請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateSurgeryRequest {
    pub is_first_experiment: Option<bool>,
    pub surgery_date: Option<NaiveDate>,
    pub surgery_site: Option<String>,
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
    pub no_medication_needed: Option<bool>,
}

/// 更新體重紀錄請求
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateWeightRequest {
    pub measure_date: Option<NaiveDate>,
    pub weight: Option<rust_decimal::Decimal>,
}

/// 更新疫苗紀錄請求
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateVaccinationRequest {
    pub administered_date: Option<NaiveDate>,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
}

/// 建立血液檢查請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBloodTestRequest {
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub remark: Option<String>,
    #[serde(default)]
    pub items: Vec<CreateBloodTestItemInput>,
}

/// 血液檢查項目輸入
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateBloodTestItemInput {
    pub template_id: Option<Uuid>,
    pub item_name: String,
    pub result_value: Option<String>,
    pub result_unit: Option<String>,
    pub reference_range: Option<String>,
    #[serde(default)]
    pub is_abnormal: bool,
    pub remark: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
}

/// 更新血液檢查請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBloodTestRequest {
    pub test_date: Option<NaiveDate>,
    pub lab_name: Option<String>,
    pub remark: Option<String>,
    pub items: Option<Vec<CreateBloodTestItemInput>>,
}

/// 建立血液檢查項目模板請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBloodTestTemplateRequest {
    #[validate(length(min = 1, max = 20, message = "代碼為必填，最多 20 字"))]
    pub code: String,
    #[validate(length(min = 1, max = 200, message = "名稱為必填，最多 200 字"))]
    pub name: String,
    pub default_unit: Option<String>,
    pub reference_range: Option<String>,
    pub default_price: Option<rust_decimal::Decimal>,
    #[serde(default)]
    pub sort_order: i32,
    /// 所屬分類（panel）ID
    pub panel_id: Option<Uuid>,
}

/// 更新血液檢查項目模板請求
#[derive(Debug, Deserialize)]
pub struct UpdateBloodTestTemplateRequest {
    pub name: Option<String>,
    pub default_unit: Option<String>,
    pub reference_range: Option<String>,
    pub default_price: Option<rust_decimal::Decimal>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    /// 所屬分類（panel）ID
    pub panel_id: Option<Uuid>,
}

// ============================================
// 血液檢查組合 (Panel)
// ============================================

/// 血液檢查組合主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BloodTestPanel {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl crate::models::audit_diff::AuditRedact for BloodTestPanel {}

/// 組合含模板項目（API 回應用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodTestPanelWithItems {
    #[serde(flatten)]
    pub panel: BloodTestPanel,
    pub items: Vec<BloodTestTemplate>,
}

/// 建立組合請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBloodTestPanelRequest {
    #[validate(length(min = 1, max = 30, message = "Key 為必填，最多 30 字"))]
    pub key: String,
    #[validate(length(min = 1, max = 100, message = "名稱為必填，最多 100 字"))]
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub template_ids: Vec<Uuid>,
}

/// 更新組合請求
#[derive(Debug, Deserialize)]
pub struct UpdateBloodTestPanelRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 更新組合項目請求
#[derive(Debug, Deserialize)]
pub struct UpdateBloodTestPanelItemsRequest {
    pub template_ids: Vec<Uuid>,
}

// ============================================
// 血液檢查常用組合 (Preset) - 分析頁一鍵選取
// ============================================

/// 常用組合主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BloodTestPreset {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub panel_keys: Vec<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl crate::models::audit_diff::AuditRedact for BloodTestPreset {}

/// 建立常用組合請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBloodTestPresetRequest {
    #[validate(length(min = 1, max = 100, message = "名稱為必填，最多 100 字"))]
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub panel_keys: Vec<String>,
    #[serde(default)]
    pub sort_order: i32,
}

/// 更新常用組合請求
#[derive(Debug, Deserialize)]
pub struct UpdateBloodTestPresetRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub panel_keys: Option<Vec<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 複製紀錄請求
#[derive(Debug, Deserialize)]
pub struct CopyRecordRequest {
    pub source_id: Uuid,
}

/// 獸醫師建議請求（含附件）
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVetRecommendationWithAttachmentsRequest {
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
    pub attachments: Option<serde_json::Value>, // [{file_name, file_path, file_type}]
    #[serde(default)]
    pub is_urgent: bool,   // 是否為緊急建議（影響通知管道）
}

/// 匯出請求
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub animal_id: Option<Uuid>,
    pub iacuc_no: Option<String>,
    pub export_type: ExportType,
    pub format: ExportFormat,
}

/// 動物匯入行資料
#[derive(Debug, Deserialize)]
pub struct AnimalImportRow {
    #[serde(
        alias = "\u{feff}Number",
        alias = "Number",
        alias = "耳號*",
        alias = "耳號"
    )]
    pub ear_tag: String,
    #[serde(
        alias = "Species",
        alias = "Species (minipig/white)",
        alias = "品種*",
        alias = "品種"
    )]
    pub breed: String,
    #[serde(alias = "Breed Other", alias = "品種其他", alias = "其他品種", default)]
    pub breed_other: Option<String>,
    #[serde(
        alias = "Sex",
        alias = "Sex (male/female)",
        alias = "性別*",
        alias = "性別"
    )]
    pub gender: String,
    #[serde(alias = "Source", alias = "來源代碼", default)]
    pub source_code: Option<String>,
    #[serde(alias = "Birthday", alias = "出生日期", default)]
    pub birth_date: Option<String>,
    #[serde(alias = "Import Date", alias = "進場日期*", alias = "進場日期")]
    pub entry_date: String,
    #[serde(
        alias = "Weight",
        alias = "Weight ",
        alias = "進場體重",
        alias = "進場體重(kg)",
        default
    )]
    pub entry_weight: Option<String>,
    #[serde(alias = "欄位編號", alias = "欄位", default)]
    pub pen_location: Option<String>,
    #[serde(alias = "IACUC No. Before Experiment", alias = "實驗前代號", default)]
    pub pre_experiment_code: Option<String>,
    #[serde(alias = "IACUC No.", alias = "計畫編號", default)]
    pub iacuc_no: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    // 額外欄位用於支援 file imput.csv
    #[serde(alias = "Field Region", alias = "區域", default)]
    pub field_region: Option<String>,
    #[serde(alias = "Field Number", alias = "區域編號", default)]
    pub field_number: Option<String>,
}

/// 體重匯入行資料
#[derive(Debug, Deserialize)]
pub struct WeightImportRow {
    #[serde(alias = "No.", alias = "耳號*", alias = "耳號")]
    pub ear_tag: String,
    #[serde(alias = "Measure Date", alias = "測量日期*", alias = "測量日期")]
    pub measure_date: String,
    #[serde(
        alias = "Weight",
        alias = "體重(kg)*",
        alias = "體重(kg)",
        alias = "體重"
    )]
    pub weight: String,
}

/// 觀察紀錄列表項目（含獸醫師建議數量）
#[derive(Debug, Serialize, FromRow)]
pub struct ObservationListItem {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub event_date: NaiveDate,
    pub record_type: RecordType,
    pub content: String,
    pub no_medication_needed: bool,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub recommendation_count: Option<i64>,
}

/// 手術紀錄列表項目
#[derive(Debug, Serialize, FromRow)]
pub struct SurgeryListItem {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub is_first_experiment: bool,
    pub surgery_date: NaiveDate,
    pub surgery_site: String,
    pub no_medication_needed: bool,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub recommendation_count: Option<i64>,
}

/// 版本歷史比對結果（前端相容：record_snapshot、created_at、changed_by_name）
#[derive(Debug, Serialize, FromRow)]
pub struct VersionDiff {
    pub id: Uuid,
    pub version_no: i32,
    #[serde(rename = "created_at")]
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    #[serde(rename = "record_snapshot")]
    pub snapshot: serde_json::Value,
    pub diff_summary: Option<String>,
    pub changed_by_name: Option<String>,
}

/// 版本歷史查詢回應
#[derive(Debug, Serialize)]
pub struct VersionHistoryResponse {
    pub record_type: String,
    pub record_id: Uuid,
    /// 目前版本號（最新 version_no，若無則 1）
    pub current_version: i32,
    pub versions: Vec<VersionDiff>,
}

// ============================================
// GLP 合規相關類型
// ============================================

/// 刪除請求（含刪除原因）- GLP 合規要求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct DeleteRequest {
    #[validate(length(min = 1, message = "刪除原因為必填"))]
    pub reason: String,
}

/// 變更原因記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChangeReason {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub change_type: String,
    pub reason: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changed_fields: Option<Vec<String>>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// 電子簽章（Phase 2）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ElectronicSignature {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub signer_id: Uuid,
    pub signature_type: String,
    pub content_hash: String,
    pub signature_data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub signed_at: DateTime<Utc>,
    pub is_valid: bool,
    pub invalidated_reason: Option<String>,
    pub invalidated_at: Option<DateTime<Utc>>,
    pub invalidated_by: Option<Uuid>,
}

/// 記錄附註（Phase 2）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecordAnnotation {
    pub id: Uuid,
    pub record_type: String,
    pub record_id: Uuid,
    pub annotation_type: String,
    pub content: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub signature_id: Option<Uuid>,
}

/// 帶變更原因的更新動物請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAnimalWithReasonRequest {
    #[serde(flatten)]
    pub data: UpdateAnimalRequest,
    #[validate(length(min = 1, message = "變更原因為必填"))]
    pub change_reason: String,
}

/// 帶變更原因的更新觀察紀錄請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateObservationWithReasonRequest {
    #[serde(flatten)]
    pub data: UpdateObservationRequest,
    #[validate(length(min = 1, message = "變更原因為必填"))]
    pub change_reason: String,
}

/// 帶變更原因的更新手術紀錄請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSurgeryWithReasonRequest {
    #[serde(flatten)]
    pub data: UpdateSurgeryRequest,
    #[validate(length(min = 1, message = "變更原因為必填"))]
    pub change_reason: String,
}

/// 帶變更原因的更新體重紀錄請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWeightWithReasonRequest {
    #[serde(flatten)]
    pub data: UpdateWeightRequest,
    #[validate(length(min = 1, message = "變更原因為必填"))]
    pub change_reason: String,
}

/// 帶變更原因的更新疫苗紀錄請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVaccinationWithReasonRequest {
    #[serde(flatten)]
    pub data: UpdateVaccinationRequest,
    #[validate(length(min = 1, message = "變更原因為必填"))]
    pub change_reason: String,
}

// ============================================
// 動物欄位修正申請（需 admin 批准）
// ============================================

/// 可申請修正的欄位
pub const CORRECTABLE_FIELDS: &[&str] = &["ear_tag", "birth_date", "gender", "breed"];

/// 建立動物欄位修正申請
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAnimalFieldCorrectionRequest {
    pub field_name: String,
    pub new_value: String,
    #[validate(length(min = 1, message = "修正原因為必填"))]
    pub reason: String,
}

/// 審核動物欄位修正申請
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewAnimalFieldCorrectionRequest {
    pub approved: bool,
    pub reject_reason: Option<String>,
}

/// 動物欄位修正申請列表項目
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct AnimalFieldCorrectionRequestListItem {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub reason: String,
    pub status: String,
    pub requested_by: Uuid,
    pub requested_by_name: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub animal_ear_tag: Option<String>,
}
