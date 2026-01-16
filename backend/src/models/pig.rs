use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// 豬隻狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "pig_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PigStatus {
    Unassigned,
    Assigned,
    InExperiment,
    Completed,
}

impl PigStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            PigStatus::Unassigned => "未分配",
            PigStatus::Assigned => "已分配",
            PigStatus::InExperiment => "實驗中",
            PigStatus::Completed => "實驗完畢",
        }
    }
}

/// 豬隻品種
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PigBreed {
    #[serde(rename = "minipig")]
    Minipig,  // 前端使用 'minipig'，資料庫存儲為 'miniature'
    White,
    Other,
}

// 手動實現 sqlx::Type 以處理資料庫 enum 值 'miniature' 到 Rust enum 'Minipig' 的映射
impl sqlx::Type<sqlx::Postgres> for PigBreed {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("pig_breed")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PigBreed {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = sqlx::Decode::<sqlx::Postgres>::decode(value)?;
        match s {
            "miniature" => Ok(PigBreed::Minipig),
            "white" => Ok(PigBreed::White),
            "other" => Ok(PigBreed::Other),
            _ => Err(format!("Invalid pig_breed value: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PigBreed {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        let s = match self {
            PigBreed::Minipig => "miniature",
            PigBreed::White => "white",
            PigBreed::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }

    fn size_hint(&self) -> usize {
        let s = match self {
            PigBreed::Minipig => "miniature",
            PigBreed::White => "white",
            PigBreed::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::size_hint(&s)
    }
}

impl PigBreed {
    pub fn display_name(&self) -> &'static str {
        match self {
            PigBreed::Minipig => "迷你豬",
            PigBreed::White => "白豬",
            PigBreed::Other => "其他",
        }
    }
}

/// 豬隻性別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "pig_gender", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PigGender {
    Male,
    Female,
}

impl PigGender {
    pub fn display_name(&self) -> &'static str {
        match self {
            PigGender::Male => "公",
            PigGender::Female => "母",
        }
    }
}

/// 紀錄類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "record_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Abnormal,
    Experiment,
    Observation,
}

impl RecordType {
    pub fn display_name(&self) -> &'static str {
        match self {
            RecordType::Abnormal => "異常紀錄",
            RecordType::Experiment => "試驗紀錄",
            RecordType::Observation => "觀察紀錄",
        }
    }
}

/// 豬隻來源
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigSource {
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

/// 豬隻主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pig {
    pub id: i32,
    pub ear_tag: String,
    pub status: PigStatus,
    pub breed: PigBreed,
    pub breed_other: Option<String>,
    pub source_id: Option<Uuid>,
    pub gender: PigGender,
    pub birth_date: Option<NaiveDate>,
    pub entry_date: NaiveDate,
    pub entry_weight: Option<rust_decimal::Decimal>,
    pub pen_location: Option<String>,
    pub pre_experiment_code: Option<String>,
    pub iacuc_no: Option<String>,
    pub experiment_date: Option<NaiveDate>,
    pub remark: Option<String>,
    pub vet_weight_viewed_at: Option<DateTime<Utc>>,
    pub vet_vaccine_viewed_at: Option<DateTime<Utc>>,
    pub vet_sacrifice_viewed_at: Option<DateTime<Utc>>,
    pub vet_last_viewed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>, // 軟刪除時間
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 觀察試驗紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigObservation {
    pub id: i32,
    pub pig_id: i32,
    pub event_date: NaiveDate,
    pub record_type: RecordType,
    pub equipment_used: Option<serde_json::Value>,
    pub anesthesia_start: Option<DateTime<Utc>>,
    pub anesthesia_end: Option<DateTime<Utc>>,
    pub content: String,
    pub no_medication_needed: bool,
    pub treatments: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub vet_read: bool,
    pub vet_read_at: Option<DateTime<Utc>>,
    pub copied_from_id: Option<i32>,       // 複製來源紀錄 ID
    pub deleted_at: Option<DateTime<Utc>>, // 軟刪除時間
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 手術紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigSurgery {
    pub id: i32,
    pub pig_id: i32,
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
    pub copied_from_id: Option<i32>,       // 複製來源紀錄 ID
    pub deleted_at: Option<DateTime<Utc>>, // 軟刪除時間
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 體重紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigWeight {
    pub id: i32,
    pub pig_id: i32,
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
    pub deleted_at: Option<DateTime<Utc>>, // 軟刪除時間
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 疫苗/驅蟲紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigVaccination {
    pub id: i32,
    pub pig_id: i32,
    pub administered_date: NaiveDate,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>, // 軟刪除時間
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 犧牲/採樣紀錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PigSacrifice {
    pub id: i32,
    pub pig_id: i32,
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
pub struct PigPathologyReport {
    pub id: i32,
    pub pig_id: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 獸醫師建議
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetRecommendation {
    pub id: i32,
    pub record_type: String,
    pub record_id: i32,
    pub content: String,
    pub attachments: Option<serde_json::Value>, // 附件（含圖片）
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Request/Response DTOs
// ============================================

/// 驗證耳號必須為三位數
fn validate_ear_tag(ear_tag: &str) -> Result<(), validator::ValidationError> {
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
fn validate_pen_location(pen_location: &String) -> Result<(), validator::ValidationError> {
    if pen_location.trim().is_empty() {
        let mut error = validator::ValidationError::new("pen_location_required");
        error.message = Some(std::borrow::Cow::Borrowed("欄位為必填"));
        Err(error)
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePigRequest {
    #[validate(length(min = 1, max = 10, message = "Ear tag must be 1-10 characters"))]
    #[validate(custom(function = "validate_ear_tag", message = "耳號必須為三位數"))]
    pub ear_tag: String,
    pub breed: PigBreed,
    pub breed_other: Option<String>,
    pub source_id: Option<Uuid>,
    pub gender: PigGender,
    pub birth_date: Option<NaiveDate>,
    pub entry_date: NaiveDate,
    pub entry_weight: Option<rust_decimal::Decimal>,
    #[validate(required(message = "欄位為必填"))]
    #[validate(custom(function = "validate_pen_location", message = "欄位不能為空"))]
    pub pen_location: Option<String>,
    pub pre_experiment_code: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdatePigRequest {
    // 以下欄位於建立後不可更改，已從更新請求中移除：
    // - ear_tag (耳號)
    // - breed (品種)
    // - gender (性別)
    // - source_id (來源)
    // - birth_date (出生日期)
    // - entry_date (進場日期)
    // - entry_weight (進場體重)
    // - pre_experiment_code (實驗前代號)
    
    pub status: Option<PigStatus>,
    pub pen_location: Option<String>,
    pub iacuc_no: Option<String>,
    pub experiment_date: Option<NaiveDate>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PigQuery {
    pub status: Option<PigStatus>,
    pub breed: Option<PigBreed>,
    pub gender: Option<PigGender>,
    pub iacuc_no: Option<String>,
    pub pen_location: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchAssignRequest {
    pub pig_ids: Vec<i32>,
    pub iacuc_no: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchStartExperimentRequest {
    pub pig_ids: Vec<i32>,
}

#[derive(Debug, Deserialize, Validate)]
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
}

#[derive(Debug, Deserialize, Validate)]
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

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct CreateWeightRequest {
    pub measure_date: NaiveDate,
    pub weight: rust_decimal::Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateVaccinationRequest {
    pub administered_date: NaiveDate,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateVetRecommendationRequest {
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePigSourceRequest {
    #[validate(length(min = 1, max = 20, message = "Code must be 1-20 characters"))]
    pub code: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePigSourceRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// 豬隻列表項目（含來源名稱）
#[derive(Debug, Serialize, FromRow)]
pub struct PigListItem {
    pub id: i32,
    pub ear_tag: String,
    pub status: PigStatus,
    pub breed: PigBreed,
    pub breed_other: Option<String>,
    pub gender: PigGender,
    pub pen_location: Option<String>,
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
}

/// 依欄位分組的豬隻
#[derive(Debug, Serialize)]
pub struct PigsByPen {
    pub pen_location: String,
    pub pigs: Vec<PigListItem>,
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
    PigBasic,
    PigWeight,
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
pub struct PigImportBatch {
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
pub struct PigExportRecord {
    pub id: Uuid,
    pub pig_id: Option<i32>,
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
    pub id: i32,
    pub record_type: String,
    pub record_id: i32,
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
#[derive(Debug, Deserialize, Validate)]
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
}

/// 更新手術紀錄請求
#[derive(Debug, Deserialize, Validate)]
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
#[derive(Debug, Deserialize)]
pub struct UpdateWeightRequest {
    pub measure_date: Option<NaiveDate>,
    pub weight: Option<rust_decimal::Decimal>,
}

/// 更新疫苗紀錄請求
#[derive(Debug, Deserialize)]
pub struct UpdateVaccinationRequest {
    pub administered_date: Option<NaiveDate>,
    pub vaccine: Option<String>,
    pub deworming_dose: Option<String>,
}

/// 複製紀錄請求
#[derive(Debug, Deserialize)]
pub struct CopyRecordRequest {
    pub source_id: i32,
}

/// 獸醫師建議請求（含附件）
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVetRecommendationWithAttachmentsRequest {
    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,
    pub attachments: Option<serde_json::Value>, // [{file_name, file_path, file_type}]
}

/// 匯出請求
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub pig_id: Option<i32>,
    pub iacuc_no: Option<String>,
    pub export_type: ExportType,
    pub format: ExportFormat,
}

/// 豬隻匯入行資料
#[derive(Debug, Deserialize)]
pub struct PigImportRow {
    #[serde(alias = "\u{feff}Number", alias = "Number", alias = "耳號*", alias = "耳號")]
    pub ear_tag: String,
    #[serde(alias = "Species", alias = "品種*", alias = "品種")]
    pub breed: String,
    #[serde(alias = "Breed Other", alias = "品種其他", alias = "其他品種", default)]
    pub breed_other: Option<String>,
    #[serde(alias = "Sex", alias = "性別*", alias = "性別")]
    pub gender: String,
    #[serde(alias = "Source", alias = "來源代碼", default)]
    pub source_code: Option<String>,
    #[serde(alias = "Birthday", alias = "出生日期", default)]
    pub birth_date: Option<String>,
    #[serde(alias = "Import Date", alias = "進場日期*", alias = "進場日期")]
    pub entry_date: String,
    #[serde(alias = "Weight", alias = "Weight ", alias = "進場體重", alias = "進場體重(kg)", default)]
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
    #[serde(alias = "Weight", alias = "體重(kg)*", alias = "體重(kg)", alias = "體重")]
    pub weight: String,
}

/// 觀察紀錄列表項目（含獸醫師建議數量）
#[derive(Debug, Serialize, FromRow)]
pub struct ObservationListItem {
    pub id: i32,
    pub pig_id: i32,
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
    pub id: i32,
    pub pig_id: i32,
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

/// 版本歷史比對結果
#[derive(Debug, Serialize)]
pub struct VersionDiff {
    pub version_no: i32,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub diff_summary: Option<String>,
    pub snapshot: serde_json::Value,
}

/// 版本歷史查詢回應
#[derive(Debug, Serialize)]
pub struct VersionHistoryResponse {
    pub record_type: String,
    pub record_id: i32,
    pub versions: Vec<VersionDiff>,
}
