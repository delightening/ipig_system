use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalWeight, AnimalWeightResponse, CreateWeightRequest,
        UpdateWeightRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

pub struct AnimalWeightService;

impl AnimalWeightService {
    // ============================================
    // 體重紀錄
    // ============================================

    /// 取得體重紀錄列表（排除已刪除，支援資料隔離）
    pub async fn list(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<AnimalWeightResponse>> {
        let weights = sqlx::query_as::<_, AnimalWeightResponse>(
            r#"
            SELECT
                w.id, w.animal_id, w.measure_date, w.weight,
                w.created_by, u.display_name as created_by_name, w.created_at
            FROM animal_weights w
            LEFT JOIN users u ON w.created_by = u.id
            WHERE w.animal_id = $1 AND w.deleted_at IS NULL
              AND ($2::timestamptz IS NULL OR w.created_at > $2)
            ORDER BY w.measure_date DESC
            "#,
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(weights)
    }

    /// 取得最新體重
    pub async fn get_latest(pool: &PgPool, animal_id: Uuid) -> Result<Option<AnimalWeight>> {
        let weight = sqlx::query_as::<_, AnimalWeight>(
            "SELECT * FROM animal_weights WHERE animal_id = $1 ORDER BY measure_date DESC LIMIT 1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(weight)
    }

    /// 建立體重紀錄 — Service-driven audit
    ///
    /// Actor 政策：接受 User（一般建立）與 System（animal create 初始體重 /
    /// batch import）。Anonymous 拒絕。`created_by` 依 actor 取值：User → user.id；
    /// System → SYSTEM_USER_ID 常數。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateWeightRequest,
    ) -> Result<AnimalWeight> {
        // Gemini PR #169 建議：用 ActorContext::actor_user_id() 簡化，
        // None 即 Anonymous（無權建立體重紀錄）。
        let created_by = actor.actor_user_id().ok_or_else(|| {
            AppError::Forbidden("建立體重紀錄須由已登入使用者或系統觸發".into())
        })?;

        let mut tx = pool.begin().await?;

        let weight = sqlx::query_as::<_, AnimalWeight>(
            r#"
            INSERT INTO animal_weights (animal_id, measure_date, weight, created_by, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
        )
        .bind(animal_id)
        .bind(req.measure_date)
        .bind(req.weight)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "animal {} @ {}: {}kg",
            weight.animal_id, weight.measure_date, weight.weight
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "WEIGHT_CREATE",
                entity: Some(AuditEntity::new("animal_weight", weight.id, &display)),
                data_diff: Some(DataDiff::create_only(&weight)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(weight)
    }

    /// 更新體重紀錄 — Service-driven audit
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateWeightRequest,
    ) -> Result<AnimalWeight> {
        actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AnimalWeight>(
            "SELECT * FROM animal_weights WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("體重紀錄不存在".into()))?;

        let after = sqlx::query_as::<_, AnimalWeight>(
            r#"
            UPDATE animal_weights SET
                measure_date = COALESCE($2, measure_date),
                weight = COALESCE($3, weight)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.measure_date)
        .bind(req.weight)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "animal {} @ {}: {}kg",
            after.animal_id, after.measure_date, after.weight
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "WEIGHT_UPDATE",
                entity: Some(AuditEntity::new("animal_weight", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 軟刪除體重紀錄（含刪除原因）— Service-driven audit
    ///
    /// GLP 合規：刪除原因寫入 change_reasons 與 animal_weights.deletion_reason，
    /// 並同 tx 寫 audit（event_type 含 RETROACTIVE 尾綴表示含原因）。
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AnimalWeight>(
            "SELECT * FROM animal_weights WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("體重紀錄不存在".into()))?;

        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('weight', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?;

        let after = sqlx::query_as::<_, AnimalWeight>(
            r#"
            UPDATE animal_weights SET
                deleted_at = NOW(),
                deletion_reason = $2,
                deleted_by = $3
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "animal {} @ {}: {}kg — {}",
            before.animal_id, before.measure_date, before.weight, reason
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "WEIGHT_SOFT_DELETE",
                entity: Some(AuditEntity::new("animal_weight", before.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
