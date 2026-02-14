use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// 豬隻狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "pig_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PigStatus {
    Unassigned,
    InExperiment,
    Completed,
}

impl PigStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            PigStatus::Unassigned => "未分配",
            PigStatus::InExperiment => "實驗中",
            PigStatus::Completed => "實驗完成",
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
    #[serde(rename = "lyd")]
    LYD,
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
            "LYD" => Ok(PigBreed::LYD),
            "other" => Ok(PigBreed::Other),
            _ => Err(format!("Invalid pig_breed value: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PigBreed {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            PigBreed::Minipig => "miniature",
            PigBreed::White => "white",
            PigBreed::LYD => "LYD",
            PigBreed::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }

    fn size_hint(&self) -> usize {
        let s = match self {
            PigBreed::Minipig => "miniature",
            PigBreed::White => "white",
            PigBreed::LYD => "LYD",
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
            PigBreed::LYD => "LYD",
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
