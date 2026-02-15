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
}

impl AnimalStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimalStatus::Unassigned => "未分配",
            AnimalStatus::InExperiment => "實驗中",
            AnimalStatus::Completed => "實驗完成",
        }
    }
}

/// 動物品種
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimalBreed {
    #[serde(rename = "minipig")]
    Minipig,  // 前端使用 'minipig'，資料庫存儲為 'miniature'
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
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
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
    pub is_urgent: bool,  // 是否為緊急建議
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
