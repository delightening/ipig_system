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
        access,
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
    pub async fn get_by_animal(
        pool: &PgPool,
        scope: access::Scoped<access::AnimalRead>,
    ) -> Result<Option<AnimalVetAdvice>> {
        let animal_id = scope.id();
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
        scope: access::Scoped<access::AnimalWrite>,
        req: &UpsertVetAdviceRequest,
    ) -> Result<AnimalVetAdvice> {
        let animal_id = scope.id();
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
            return Err(AppError::NotFound(format!(
                "Animal {} not found",
                animal_id
            )));
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
                entity: Some(AuditEntity::new("animal_vet_advice", advice.id, &display)),
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
    /// 追蹤改善（巡場報告陪同人員填；手動建議為空字串）
    #[serde(default)]
    pub follow_up: String,
    /// 來源巡場條目（NULL = 獸醫在「獸醫師建議」分頁手動寫的 comment）
    pub source_vet_patrol_entry_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// R26-9: `observation` / `suggested_treatment` 為醫療診斷內容，為 GLP 研究資料
// 本身，需完整保留於 audit log。空 `redacted_fields()` 是主動決策。
impl crate::models::audit_diff::AuditRedact for VetAdviceRecord {}

pub struct VetAdviceRecordService;

impl VetAdviceRecordService {
    pub async fn list(
        pool: &PgPool,
        scope: access::Scoped<access::AnimalRead>,
    ) -> Result<Vec<VetAdviceRecord>> {
        let animal_id = scope.id();
        let records = sqlx::query_as::<_, VetAdviceRecord>(
            r#"SELECT id, animal_id, advice_date, observation, suggested_treatment,
                      follow_up, source_vet_patrol_entry_id,
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

    // create/update/delete 已移除：獸醫師建議改為唯讀單一來源（animal_vet_advice_records），
    // 內容由巡場報告 complete_followup 自動歸位（單向同步）。獸醫撰寫入口統一在巡場報告。
}
