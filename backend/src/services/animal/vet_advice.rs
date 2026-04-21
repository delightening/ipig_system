// 獸醫師建議 Service
// - AnimalVetAdviceService: 舊版結構化表單（保留，未來巡場報告用）
// - VetAdviceRecordService: 新版多筆紀錄列表

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::audit_diff::DataDiff,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

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

    /// 新增或更新獸醫師建議（create_or_update — 取代原 upsert 以獲完整 audit trail）
    ///
    /// 原 `upsert` 用 `INSERT ... ON CONFLICT DO UPDATE` 無法取得 before state。
    /// 此版先 SELECT 判斷存在與否：
    /// - 不存在 → INSERT → log_activity_tx::create
    /// - 存在   → UPDATE → log_activity_tx::update（完整 before/after diff）
    pub async fn create_or_update(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &UpsertVetAdviceRequest,
    ) -> Result<AnimalVetAdvice> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, AnimalVetAdvice>(
            "SELECT * FROM animal_vet_advices WHERE animal_id = $1 FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        match existing {
            Some(before) => {
                let after = sqlx::query_as::<_, AnimalVetAdvice>(
                    r#"
                    UPDATE animal_vet_advices SET
                        sections = $2,
                        updated_by = $3,
                        updated_at = NOW()
                    WHERE animal_id = $1
                    RETURNING *
                    "#,
                )
                .bind(animal_id)
                .bind(&req.sections)
                .bind(user.id)
                .fetch_one(&mut *tx)
                .await?;

                let display = format!("獸醫建議 (animal: {})", animal_id);
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry::update(
                        "ANIMAL",
                        "VET_ADVICE_UPDATE",
                        AuditEntity::new("animal_vet_advice", after.id, &display),
                        DataDiff::compute(Some(&before), Some(&after)),
                    ),
                )
                .await?;

                tx.commit().await?;
                Ok(after)
            }
            None => {
                let created = sqlx::query_as::<_, AnimalVetAdvice>(
                    r#"
                    INSERT INTO animal_vet_advices (animal_id, sections, created_by, updated_by)
                    VALUES ($1, $2, $3, $3)
                    RETURNING *
                    "#,
                )
                .bind(animal_id)
                .bind(&req.sections)
                .bind(user.id)
                .fetch_one(&mut *tx)
                .await?;

                let display = format!("獸醫建議 (animal: {})", animal_id);
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry::create(
                        "ANIMAL",
                        "VET_ADVICE_CREATE",
                        AuditEntity::new("animal_vet_advice", created.id, &display),
                        &created,
                    ),
                )
                .await?;

                tx.commit().await?;
                Ok(created)
            }
        }
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
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateVetAdviceRecordRequest,
    ) -> Result<VetAdviceRecord> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

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
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("獸醫建議紀錄 {}", record.advice_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::create(
                "ANIMAL",
                "VET_ADVICE_RECORD_CREATE",
                AuditEntity::new("vet_advice_record", record.id, &display),
                &record,
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateVetAdviceRecordRequest,
    ) -> Result<VetAdviceRecord> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetAdviceRecord>(
            r#"SELECT id, animal_id, advice_date, observation, suggested_treatment,
                      created_by, updated_by, created_at, updated_at
               FROM animal_vet_advice_records
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Vet advice record not found".to_string()))?;

        let after = sqlx::query_as::<_, VetAdviceRecord>(
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
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("獸醫建議紀錄 {}", after.advice_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::update(
                "ANIMAL",
                "VET_ADVICE_RECORD_UPDATE",
                AuditEntity::new("vet_advice_record", after.id, &display),
                DataDiff::compute(Some(&before), Some(&after)),
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetAdviceRecord>(
            r#"SELECT id, animal_id, advice_date, observation, suggested_treatment,
                      created_by, updated_by, created_at, updated_at
               FROM animal_vet_advice_records
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Vet advice record not found or already deleted".to_string()))?;

        sqlx::query(
            "UPDATE animal_vet_advice_records SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        let display = format!("獸醫建議紀錄 {}", before.advice_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::delete(
                "ANIMAL",
                "VET_ADVICE_RECORD_DELETE",
                AuditEntity::new("vet_advice_record", before.id, &display),
                &before,
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
