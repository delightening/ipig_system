// 照護紀錄（疼痛評估）Service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

/// 照護給藥紀錄模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "care_record_mode", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CareRecordMode {
    Legacy,
    PainAssessment,
}

/// 獸醫紀錄類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "vet_record_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CareVetRecordType {
    Observation,
    Surgery,
}

/// 照護紀錄（DB 結構）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CareRecord {
    pub id: Uuid,
    pub record_type: CareVetRecordType,
    pub record_id: Uuid,
    pub record_mode: CareRecordMode,
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    pub spirit: Option<String>,
    pub appetite: Option<String>,
    pub mobility_standing: Option<String>,
    pub mobility_walking: Option<String>,
    pub attitude_behavior: Option<String>,
    pub vet_read: bool,
    pub created_at: DateTime<Utc>,
}

/// 建立照護紀錄請求
#[derive(Debug, Deserialize)]
pub struct CreateCareRecordRequest {
    pub record_type: CareVetRecordType,
    pub record_id: Uuid,
    #[serde(default = "default_pain_assessment")]
    pub record_mode: CareRecordMode,
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    pub spirit: Option<String>,
    pub appetite: Option<String>,
    pub mobility_standing: Option<String>,
    pub mobility_walking: Option<String>,
    pub attitude_behavior: Option<String>,
}

fn default_pain_assessment() -> CareRecordMode {
    CareRecordMode::PainAssessment
}

/// 更新照護紀錄請求
#[derive(Debug, Deserialize)]
pub struct UpdateCareRecordRequest {
    pub post_op_days: Option<i32>,
    pub time_period: Option<String>,
    pub spirit: Option<String>,
    pub appetite: Option<String>,
    pub mobility_standing: Option<String>,
    pub mobility_walking: Option<String>,
    pub attitude_behavior: Option<String>,
}

pub struct CareRecordService;

impl CareRecordService {
    /// 列出某動物的照護紀錄（排除已軟刪除）
    /// 透過 record_id (observation/surgery) 關聯到 animal
    pub async fn list_by_animal(pool: &PgPool, animal_id: Uuid) -> Result<Vec<CareRecord>> {
        let records = sqlx::query_as::<_, CareRecord>(
            r#"
            SELECT c.id, c.record_type, c.record_id, c.record_mode, c.post_op_days,
                   c.time_period, c.spirit, c.appetite, c.mobility_standing,
                   c.mobility_walking, c.attitude_behavior, c.vet_read, c.created_at
            FROM care_medication_records c
            WHERE c.deleted_at IS NULL
              AND (
                (c.record_type = 'observation' AND c.record_id IN (
                    SELECT id FROM animal_observations WHERE animal_id = $1 AND deleted_at IS NULL
                ))
                OR
                (c.record_type = 'surgery' AND c.record_id IN (
                    SELECT id FROM animal_surgeries WHERE animal_id = $1 AND deleted_at IS NULL
                ))
            )
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 建立照護紀錄
    pub async fn create(pool: &PgPool, req: &CreateCareRecordRequest) -> Result<CareRecord> {
        let record = sqlx::query_as::<_, CareRecord>(
            r#"
            INSERT INTO care_medication_records
                (record_type, record_id, record_mode, post_op_days, time_period,
                 spirit, appetite, mobility_standing, mobility_walking, attitude_behavior)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(req.record_type)
        .bind(req.record_id)
        .bind(req.record_mode)
        .bind(req.post_op_days)
        .bind(&req.time_period)
        .bind(&req.spirit)
        .bind(&req.appetite)
        .bind(&req.mobility_standing)
        .bind(&req.mobility_walking)
        .bind(&req.attitude_behavior)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 更新照護紀錄
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateCareRecordRequest,
    ) -> Result<CareRecord> {
        let record = sqlx::query_as::<_, CareRecord>(
            r#"
            UPDATE care_medication_records
            SET post_op_days = $2,
                time_period = $3,
                spirit = $4,
                appetite = $5,
                mobility_standing = $6,
                mobility_walking = $7,
                attitude_behavior = $8
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.post_op_days)
        .bind(&req.time_period)
        .bind(&req.spirit)
        .bind(&req.appetite)
        .bind(&req.mobility_standing)
        .bind(&req.mobility_walking)
        .bind(&req.attitude_behavior)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 軟刪除照護紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE care_medication_records SET
                deleted_at = NOW(),
                deletion_reason = $2,
                deleted_by = $3
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        Ok(())
    }
}
