// 獸醫師建議 Service
// - AnimalVetAdviceService: 舊版結構化表單（保留，未來巡場報告用）
// - VetAdviceRecordService: 新版多筆紀錄列表

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

/// 獸醫師建議（DB 結構）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnimalVetAdvice {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub sections: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 無敏感欄位，空 impl 即可（見 AuditRedact trait doc 警告）
impl crate::models::audit_diff::AuditRedact for AnimalVetAdvice {}

/// Upsert 請求
#[derive(Debug, Deserialize)]
pub struct UpsertVetAdviceRequest {
    pub sections: serde_json::Value,
}

pub struct AnimalVetAdviceService;

impl AnimalVetAdviceService {
    /// 取得動物的獸醫師建議
    pub async fn get_by_animal(pool: &PgPool, animal_id: Uuid) -> Result<Option<AnimalVetAdvice>> {
        let advice = sqlx::query_as::<_, AnimalVetAdvice>(
            "SELECT * FROM animal_vet_advices WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(advice)
    }

    /// 新增或更新獸醫師建議（upsert）
    pub async fn upsert(
        pool: &PgPool,
        animal_id: Uuid,
        req: &UpsertVetAdviceRequest,
        user_id: Uuid,
    ) -> Result<AnimalVetAdvice> {
        let advice = sqlx::query_as::<_, AnimalVetAdvice>(
            r#"
            INSERT INTO animal_vet_advices (animal_id, sections, created_by, updated_by)
            VALUES ($1, $2, $3, $3)
            ON CONFLICT (animal_id)
            DO UPDATE SET
                sections = $2,
                updated_by = $3,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(animal_id)
        .bind(&req.sections)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(advice)
    }
}

// ── 獸醫師建議紀錄（多筆 CRUD）──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetAdviceRecord {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub advice_date: NaiveDate,
    pub observation: String,
    pub suggested_treatment: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 無敏感欄位，空 impl 即可（見 AuditRedact trait doc 警告）
impl crate::models::audit_diff::AuditRedact for VetAdviceRecord {}

#[derive(Debug, Deserialize)]
pub struct CreateVetAdviceRecordRequest {
    pub advice_date: NaiveDate,
    pub observation: String,
    pub suggested_treatment: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVetAdviceRecordRequest {
    pub advice_date: Option<NaiveDate>,
    pub observation: Option<String>,
    pub suggested_treatment: Option<String>,
}

pub struct VetAdviceRecordService;

impl VetAdviceRecordService {
    pub async fn list(pool: &PgPool, animal_id: Uuid) -> Result<Vec<VetAdviceRecord>> {
        let records = sqlx::query_as::<_, VetAdviceRecord>(
            r#"SELECT id, animal_id, advice_date, observation, suggested_treatment,
                      created_by, updated_by, created_at, updated_at
               FROM animal_vet_advice_records
               WHERE animal_id = $1 AND deleted_at IS NULL
               ORDER BY advice_date DESC, created_at DESC"#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;
        Ok(records)
    }

    pub async fn create(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateVetAdviceRecordRequest,
        user_id: Uuid,
    ) -> Result<VetAdviceRecord> {
        let record = sqlx::query_as::<_, VetAdviceRecord>(
            r#"INSERT INTO animal_vet_advice_records
                   (animal_id, advice_date, observation, suggested_treatment, created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $5)
               RETURNING id, animal_id, advice_date, observation, suggested_treatment,
                         created_by, updated_by, created_at, updated_at"#,
        )
        .bind(animal_id)
        .bind(req.advice_date)
        .bind(&req.observation)
        .bind(&req.suggested_treatment)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(record)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateVetAdviceRecordRequest,
        user_id: Uuid,
    ) -> Result<VetAdviceRecord> {
        let record = sqlx::query_as::<_, VetAdviceRecord>(
            r#"UPDATE animal_vet_advice_records SET
                   advice_date         = COALESCE($2, advice_date),
                   observation         = COALESCE($3, observation),
                   suggested_treatment = COALESCE($4, suggested_treatment),
                   updated_by          = $5,
                   updated_at          = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, animal_id, advice_date, observation, suggested_treatment,
                         created_by, updated_by, created_at, updated_at"#,
        )
        .bind(id)
        .bind(req.advice_date)
        .bind(&req.observation)
        .bind(&req.suggested_treatment)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(record)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE animal_vet_advice_records SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
