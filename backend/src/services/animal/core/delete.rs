use sqlx::PgPool;
use uuid::Uuid;

use super::super::AnimalService;
use crate::Result;

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

    /// 軟刪除動物（含刪除原因）- GLP 合規
    pub async fn delete_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('animal', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        sqlx::query(
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
        .execute(pool)
        .await?;

        Ok(())
    }
}
