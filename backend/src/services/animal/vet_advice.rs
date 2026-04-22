// 獸醫師建議 Service
// - AnimalVetAdviceService: 舊版結構化表單（保留，未來巡場報告用）
// - VetAdviceRecordService: 新版多筆紀錄列表

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::audit_diff::{AuditRedact, DataDiff},
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

// R26-9: `sections` 為 JSONB，含獸醫診斷內容屬醫療資訊。目前採空 allowlist
// 先保留全值於 audit（GLP 需要完整變更軌跡），後續若決定降敏再於此覆寫
// `redacted_fields()`。
impl AuditRedact for AnimalVetAdvice {}

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

    /// 新增或更新獸醫師建議（R26-10 併發安全化 + Service-driven audit）
    ///
    /// 併發策略：先對父層 `animals` 行 `SELECT FOR UPDATE`，使同一動物的 upsert
    /// 序列化，避免兩個並發請求都走到 None 分支同時 INSERT。鎖於 tx commit 時
    /// 釋放。鎖住父層而非 `animal_vet_advices` 是因 row 可能不存在。
    ///
    /// Audit 策略：依 before snapshot 明確區分 CREATE / UPDATE 事件類型，利於
    /// GLP 稽核查詢。
    pub async fn upsert(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &UpsertVetAdviceRequest,
    ) -> Result<AnimalVetAdvice> {
        let user = actor.require_user()?;
        let user_id = user.id;

        let mut tx = pool.begin().await?;

        // 鎖父層 animals 行：確保動物存在（未 soft-delete）並序列化同一動物
        // 的 upsert 操作，避免 `animal_vet_advices` 缺 row 時的 INSERT race。
        let animal_locked: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        if animal_locked.is_none() {
            return Err(AppError::NotFound(format!("Animal {} not found", animal_id)));
        }

        // 取 before snapshot（父層已鎖，此讀取為一致性觀測）
        let before = sqlx::query_as::<_, AnimalVetAdvice>(
            "SELECT * FROM animal_vet_advices WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (advice, event_type, data_diff) = match &before {
            None => {
                let advice = sqlx::query_as::<_, AnimalVetAdvice>(
                    r#"
                    INSERT INTO animal_vet_advices (animal_id, sections, created_by, updated_by)
                    VALUES ($1, $2, $3, $3)
                    RETURNING *
                    "#,
                )
                .bind(animal_id)
                .bind(&req.sections)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
                let diff = DataDiff::create_only(&advice);
                (advice, "VET_ADVICE_CREATE", diff)
            }
            Some(b) => {
                let advice = sqlx::query_as::<_, AnimalVetAdvice>(
                    r#"
                    UPDATE animal_vet_advices
                    SET sections = $2, updated_by = $3, updated_at = NOW()
                    WHERE animal_id = $1
                    RETURNING *
                    "#,
                )
                .bind(animal_id)
                .bind(&req.sections)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
                let diff = DataDiff::compute(Some(b), Some(&advice));
                (advice, "VET_ADVICE_UPDATE", diff)
            }
        };

        let display = format!("animal {} vet advice", animal_id);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type,
                entity: Some(AuditEntity::new(
                    "animal_vet_advice",
                    advice.id,
                    &display,
                )),
                data_diff: Some(data_diff),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
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

// R26-9: `observation` / `suggested_treatment` 為醫療診斷內容，為 GLP 研究資料
// 本身，需完整保留於 audit log。空 `redacted_fields()` 是主動決策。
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

    /// 刪除獸醫建議紀錄（R26-11：要求呼叫端傳入已驗權的 animal_id，
    /// service 層 UPDATE 以 (id, animal_id) 雙重作用域防 IDOR）。
    pub async fn delete(pool: &PgPool, id: Uuid, animal_id: Uuid) -> Result<()> {
        let rows = sqlx::query(
            "UPDATE animal_vet_advice_records
             SET deleted_at = NOW()
             WHERE id = $1 AND animal_id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(animal_id)
        .execute(pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(crate::AppError::NotFound(
                "Vet advice record not found".into(),
            ));
        }
        Ok(())
    }
}
