// 人員訓練紀錄 Models (GLP 合規)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrainingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_name: String,
    pub completed_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TrainingRecordWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: String,
    pub course_name: String,
    pub completed_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TrainingQuery {
    pub user_id: Option<Uuid>,
    pub course_name: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTrainingRecordRequest {
    pub user_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub course_name: String,
    pub completed_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTrainingRecordRequest {
    #[validate(length(min = 1, max = 200))]
    pub course_name: Option<String>,
    pub completed_at: Option<NaiveDate>,
    pub expires_at: Option<NaiveDate>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}
