use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::AnimalGender;

/// 預定試驗（規劃中、未核准的試驗需求；執行秘書手填。核准後 protocol_id 連結真 protocol）。
/// 動物預約與試驗規劃子系統（見 docs/design/animal-reservation/）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PlannedExperiment {
    pub id: Uuid,
    /// 委託單位 / 客戶名（如「昱展新藥」）
    pub unit: String,
    /// 試驗內容概述
    pub description: Option<String>,
    /// 需求動物數
    pub demand_count: i32,
    /// 核准後連結真 protocol（非空 = 已核准）
    pub protocol_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 預定試驗（含建立者顯示名 + 連結計畫案號，列表用）。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct PlannedExperimentResponse {
    pub id: Uuid,
    pub unit: String,
    pub description: Option<String>,
    pub demand_count: i32,
    pub protocol_id: Option<Uuid>,
    #[sqlx(default)]
    pub protocol_iacuc_no: Option<String>,
    pub created_by: Uuid,
    #[sqlx(default)]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 非空白字串驗證（trim 後不可為空）。
fn validate_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("blank"));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePlannedExperimentRequest {
    #[validate(
        length(min = 1, max = 200, message = "委託單位為必填，至多 200 字"),
        custom(function = "validate_non_blank", message = "委託單位不可為空白")
    )]
    pub unit: String,
    #[validate(length(max = 2000, message = "試驗內容概述至多 2000 字"))]
    pub description: Option<String>,
    #[validate(range(min = 0, max = 100_000, message = "需求動物數須為 0 以上"))]
    pub demand_count: i32,
    pub protocol_id: Option<Uuid>,
}

/// 手動編輯欄位。`protocol_id`（核准連結）刻意不在此 DTO — 由核准流程管理，
/// 避免 partial update 省略欄位時靜默清空已核准連結（coderabbit #837）。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePlannedExperimentRequest {
    #[validate(length(min = 1, max = 200, message = "委託單位至多 200 字"))]
    pub unit: Option<String>,
    #[validate(length(max = 2000, message = "試驗內容概述至多 2000 字"))]
    pub description: Option<String>,
    #[validate(range(min = 0, max = 100_000, message = "需求動物數須為 0 以上"))]
    pub demand_count: Option<i32>,
}

/// 批次預約動物到試驗（Phase 2）。目標為 planned_experiment 或已核准 protocol 二擇一
/// （service 層強制恰好一個）；僅未分配、未預約的動物可被預約。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReserveAnimalsRequest {
    #[validate(length(min = 1, message = "至少選擇一隻動物"))]
    pub animal_ids: Vec<Uuid>,
    pub planned_experiment_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
}

/// 批次解除預約（Phase 2）。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UnreserveAnimalsRequest {
    #[validate(length(min = 1, message = "至少選擇一隻動物"))]
    pub animal_ids: Vec<Uuid>,
}

/// 備註 inline 編輯（Phase 5，規劃頁整格點擊編輯）。last-write-wins（備註低競爭、
/// 不做樂觀鎖）；service 層仍寫 before/after DataDiff 進 audit chain。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRemarkRequest {
    #[validate(length(max = 2000, message = "備註至多 2000 字"))]
    pub remark: Option<String>,
}

/// 搜尋可預約動物（未分配未預約）的篩選條件（Phase 2）。None 者不過濾。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReservableQuery {
    pub gender: Option<AnimalGender>,
    pub age_months_min: Option<i32>,
    pub age_months_max: Option<i32>,
    pub weight_min: Option<Decimal>,
    pub weight_max: Option<Decimal>,
}

/// 規劃分組中的一隻動物（Phase 3）。`group_key` 供後端分組、不對外輸出。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct PlanningAnimalRow {
    #[serde(skip)]
    #[sqlx(default)]
    pub group_key: String,
    pub id: Uuid,
    pub ear_tag: String,
    pub animal_no: Option<String>,
    pub gender: AnimalGender,
    pub birth_date: Option<NaiveDate>,
    #[sqlx(default)]
    pub latest_weight_kg: Option<Decimal>,
    #[sqlx(default)]
    pub weight_measured_at: Option<NaiveDate>,
    pub remark: Option<String>,
    pub status: String,
    /// 該動物與此試驗的關係（Phase 5 起三態）：
    /// "reserved"（已預約，未分配）/ "in_experiment"（已分配、實驗中）/
    /// "completed"（實驗完成、待淘汰，仍計入缺口）
    pub relation: String,
}

/// 試驗規劃分組（Phase 3；Phase 5 擴充）：一個已核准 protocol、一個規劃中
/// planned_experiment，或置底的 orphan catch-all 組。
#[derive(Debug, Serialize, ToSchema)]
pub struct ReservationPlanningGroup {
    /// "approved"（已核准 protocol）/ "planned"（規劃中預定試驗）/
    /// "orphan"（掛在非顯示計畫〔已結案/暫停〕下的存活動物 catch-all，置底）
    pub group_type: String,
    pub id: Uuid,
    pub iacuc_no: Option<String>,
    /// 委託單位（approved 取 sponsor 機構、planned 取 unit）
    pub unit: Option<String>,
    pub pi_name: Option<String>,
    pub description: Option<String>,
    pub demand: i64,
    pub reserved_count: i64,
    /// 已分配、實驗中（Phase 5 起與 completed 拆開）
    pub in_experiment_count: i64,
    /// 實驗完成（待淘汰）— 仍計入缺口
    pub completed_count: i64,
    pub animals: Vec<PlanningAnimalRow>,
}

/// 可預約動物一筆（含最新體重 + 月齡）。
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ReservableAnimalRow {
    pub id: Uuid,
    pub ear_tag: String,
    pub animal_no: Option<String>,
    pub gender: AnimalGender,
    pub birth_date: Option<NaiveDate>,
    #[sqlx(default)]
    pub age_months: Option<i32>,
    pub pen_location: Option<String>,
    #[sqlx(default)]
    pub latest_weight_kg: Option<Decimal>,
    #[sqlx(default)]
    pub weight_measured_at: Option<NaiveDate>,
    /// 備用池亦可 inline 編輯備註（Phase 5，「全部可編」）
    pub remark: Option<String>,
}
