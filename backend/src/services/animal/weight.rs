use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{CreateWeightRequest, AnimalWeight, AnimalWeightResponse, UpdateWeightRequest},
    Result,
};

impl AnimalService {

    // ============================================
    // 體重紀錄
    // ============================================

    /// 取得體重紀錄列表（排除已刪除）
    pub async fn list_weights(pool: &PgPool, animal_id: Uuid) -> Result<Vec<AnimalWeightResponse>> {
        let weights = sqlx::query_as::<_, AnimalWeightResponse>(
            r#"
            SELECT 
                w.id, w.animal_id, w.measure_date, w.weight, 
                w.created_by, u.display_name as created_by_name, w.created_at
            FROM animal_weights w
            LEFT JOIN users u ON w.created_by = u.id
            WHERE w.animal_id = $1 AND w.deleted_at IS NULL
            ORDER BY w.measure_date DESC
            "#
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(weights)
    }

    /// 取得最新體重
    pub async fn get_latest_weight(pool: &PgPool, animal_id: Uuid) -> Result<Option<AnimalWeight>> {
        let weight = sqlx::query_as::<_, AnimalWeight>(
            "SELECT * FROM animal_weights WHERE animal_id = $1 ORDER BY measure_date DESC LIMIT 1"
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(weight)
    }

    pub async fn create_weight(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateWeightRequest,
        created_by: Uuid,
    ) -> Result<AnimalWeight> {
        let weight = sqlx::query_as::<_, AnimalWeight>(
            r#"
            INSERT INTO animal_weights (animal_id, measure_date, weight, created_by, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(req.measure_date)
        .bind(req.weight)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(weight)
    }

    /// 更新體重紀錄
    pub async fn update_weight(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateWeightRequest,
    ) -> Result<AnimalWeight> {
        let weight = sqlx::query_as::<_, AnimalWeight>(
            r#"
            UPDATE animal_weights SET
                measure_date = COALESCE($2, measure_date),
                weight = COALESCE($3, weight)
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.measure_date)
        .bind(req.weight)
        .fetch_one(pool)
        .await?;

        Ok(weight)
    }

    /// 刪除體重紀錄
    pub async fn soft_delete_weight(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM animal_weights WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除體重紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_weight_with_reason(pool: &PgPool, id: Uuid, reason: &str, deleted_by: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('weight', $1::text, 'DELETE', $2, $3)
            "#
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE animal_weights SET 
                deleted_at = NOW(), 
                deletion_reason = $2,
                deleted_by = $3
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        Ok(())
    }
}
