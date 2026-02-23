use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// 動物狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "animal_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AnimalStatus {
    Unassigned,
    InExperiment,
    Completed,
    Euthanized,
    SuddenDeath,
    Transferred,
}

impl AnimalStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimalStatus::Unassigned => "未分配",
            AnimalStatus::InExperiment => "實驗中",
            AnimalStatus::Completed => "實驗完成",
            AnimalStatus::Euthanized => "安樂死",
            AnimalStatus::SuddenDeath => "猝死",
            AnimalStatus::Transferred => "已轉讓",
        }
    }

    /// 檢查狀態轉換是否合法
    pub fn can_transition_to(&self, target: AnimalStatus) -> bool {
        matches!(
            (self, target),
            // 未分配 → 實驗中 / 安樂死（犧牲紀錄）/ 猝死
            (Self::Unassigned, Self::InExperiment)
            | (Self::Unassigned, Self::Euthanized)
            | (Self::Unassigned, Self::SuddenDeath)
            // 實驗中 → 存活完成 / 安樂死（犧牲或安樂死申請）/ 猝死
            | (Self::InExperiment, Self::Completed)
            | (Self::InExperiment, Self::Euthanized)
            | (Self::InExperiment, Self::SuddenDeath)
            // 存活完成 → 已轉讓（透過 transfer API）/ 安樂死（犧牲紀錄）
            | (Self::Completed, Self::Transferred)
            | (Self::Completed, Self::Euthanized)
            // 已轉讓 → 實驗中（轉讓完成後重新入組）
            | (Self::Transferred, Self::InExperiment)
        )
    }

    /// 是否為終態
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Euthanized | Self::SuddenDeath)
    }
}

/// 動物轉讓狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "animal_transfer_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AnimalTransferStatus {
    Pending,
    VetEvaluated,
    PlanAssigned,
    PiApproved,
    Completed,
    Rejected,
}

impl AnimalTransferStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "待審",
            Self::VetEvaluated => "獸醫已評估",
            Self::PlanAssigned => "已指定新計劃",
            Self::PiApproved => "PI 已同意",
            Self::Completed => "轉讓完成",
            Self::Rejected => "已拒絕",
        }
    }
}

/// 動物品種
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimalBreed {
    #[serde(rename = "minipig")]
    Minipig, // 前端使用 'minipig'，資料庫存儲為 'miniature'
    White,
    #[serde(rename = "lyd")]
    LYD,
    Other,
}

// 手動實現 sqlx::Type 以處理資料庫 enum 值 'miniature' 到 Rust enum 'Minipig' 的映射
impl sqlx::Type<sqlx::Postgres> for AnimalBreed {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("animal_breed")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AnimalBreed {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = sqlx::Decode::<sqlx::Postgres>::decode(value)?;
        match s {
            "miniature" => Ok(AnimalBreed::Minipig),
            "white" => Ok(AnimalBreed::White),
            "LYD" => Ok(AnimalBreed::LYD),
            "other" => Ok(AnimalBreed::Other),
            _ => Err(format!("Invalid animal_breed value: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AnimalBreed {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            AnimalBreed::Minipig => "miniature",
            AnimalBreed::White => "white",
            AnimalBreed::LYD => "LYD",
            AnimalBreed::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }

    fn size_hint(&self) -> usize {
        let s = match self {
            AnimalBreed::Minipig => "miniature",
            AnimalBreed::White => "white",
            AnimalBreed::LYD => "LYD",
            AnimalBreed::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::size_hint(&s)
    }
}

impl AnimalBreed {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimalBreed::Minipig => "迷你豬",
            AnimalBreed::White => "白豬",
            AnimalBreed::LYD => "LYD",
            AnimalBreed::Other => "其他",
        }
    }
}

/// 動物性別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "animal_gender", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AnimalGender {
    Male,
    Female,
}

impl AnimalGender {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimalGender::Male => "公",
            AnimalGender::Female => "母",
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

/// 獸醫師紀錄類型（對應資料庫 vet_record_type enum）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "vet_record_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VetRecordType {
    Observation,
    Surgery,
}

/// 獸醫師建議
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetRecommendation {
    pub id: Uuid,
    pub record_type: VetRecordType,
    pub record_id: Uuid,
    pub content: String,
    pub attachments: Option<serde_json::Value>, // 附件（含圖片）
    #[sqlx(default)]
    pub is_urgent: bool,    // 是否為緊急建議
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
