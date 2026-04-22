use sqlx::PgPool;
use uuid::Uuid;

use super::super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{audit_diff::DataDiff, Animal},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

impl AnimalService {
    /// 軟刪除動物
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE animals SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 軟刪除動物（含刪除原因）- GLP 合規 — Service-driven audit
    pub async fn delete_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;

        // before snapshot
        let before: Animal = sqlx::query_as(
            "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在或已刪除".into()))?;

        let mut tx = pool.begin().await?;

        // Gemini PR #178 pattern：先 UPDATE 且檢查 rows_affected，防假刪除審計
        let rows = sqlx::query(
            r#"
            UPDATE animals SET
                deleted_at = NOW(),
                deletion_reason = $2,
                deleted_by = $3,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFound("動物不存在或已刪除".into()));
        }

        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('animal', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?;

        let display = format!("{} (原因: {})", before.ear_tag, reason);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "ANIMAL_DELETE",
                entity: Some(AuditEntity::new("animal", id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
