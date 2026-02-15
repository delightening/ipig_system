use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        CreateObservationRequest, ObservationListItem, AnimalObservation, UpdateObservationRequest,
    },
    AppError, Result,
};

impl AnimalService {
    // ============================================
    // 觀察試驗紀錄
    // ============================================

    /// 取得觀察紀錄列表（排除已刪除）
    pub async fn list_observations(pool: &PgPool, animal_id: Uuid) -> Result<Vec<AnimalObservation>> {
        let observations = sqlx::query_as::<_, AnimalObservation>(
            "SELECT * FROM animal_observations WHERE animal_id = $1 ORDER BY event_date DESC"
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得觀察紀錄列表（含獸醫師建議數量）
    pub async fn list_observations_with_recommendations(pool: &PgPool, animal_id: Uuid) -> Result<Vec<ObservationListItem>> {
        let observations = sqlx::query_as::<_, ObservationListItem>(
            r#"
            SELECT 
                o.id, o.animal_id, o.event_date, o.record_type, o.content,
                o.no_medication_needed, o.vet_read, o.vet_read_at,
                o.created_by, o.created_at,
                (SELECT COUNT(*) FROM vet_recommendations vr WHERE vr.record_type = 'observation' AND vr.record_id = o.id) as recommendation_count
            FROM animal_observations o
            WHERE o.animal_id = $1
            ORDER BY o.event_date DESC
            "#
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得單一觀察紀錄
    pub async fn get_observation_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalObservation> {
        let observation = sqlx::query_as::<_, AnimalObservation>(
            "SELECT * FROM animal_observations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Observation not found".to_string()))?;

        Ok(observation)
    }

    pub async fn create_observation(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateObservationRequest,
        created_by: Uuid,
    ) -> Result<AnimalObservation> {
        // 如果是緊急給藥，設定狀態為 pending_review
        let emergency_status = if req.is_emergency {
            Some("pending_review".to_string())
        } else {
            None
        };

        let observation = sqlx::query_as::<_, AnimalObservation>(
            r#"
            INSERT INTO animal_observations (
                animal_id, event_date, record_type, equipment_used, anesthesia_start,
                anesthesia_end, content, no_medication_needed, treatments, remark,
                is_emergency, emergency_status, emergency_reason,
                created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(req.event_date)
        .bind(req.record_type)
        .bind(&req.equipment_used)
        .bind(req.anesthesia_start)
        .bind(req.anesthesia_end)
        .bind(&req.content)
        .bind(req.no_medication_needed)
        .bind(&req.treatments)
        .bind(&req.remark)
        .bind(req.is_emergency)
        .bind(&emergency_status)
        .bind(&req.emergency_reason)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(observation)
    }

    /// 更新觀察紀錄
    pub async fn update_observation(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateObservationRequest,
        updated_by: Uuid,
    ) -> Result<AnimalObservation> {
        // 先取得原始紀錄用於版本歷史
        let original = Self::get_observation_by_id(pool, id).await?;
        
        // 保存版本歷史
        Self::save_record_version(pool, "observation", id, &original, updated_by).await?;

        let observation = sqlx::query_as::<_, AnimalObservation>(
            r#"
            UPDATE animal_observations SET
                event_date = COALESCE($2, event_date),
                record_type = COALESCE($3, record_type),
                equipment_used = COALESCE($4, equipment_used),
                anesthesia_start = COALESCE($5, anesthesia_start),
                anesthesia_end = COALESCE($6, anesthesia_end),
                content = COALESCE($7, content),
                no_medication_needed = COALESCE($8, no_medication_needed),
                treatments = COALESCE($9, treatments),
                remark = COALESCE($10, remark),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.event_date)
        .bind(req.record_type)
        .bind(&req.equipment_used)
        .bind(req.anesthesia_start)
        .bind(req.anesthesia_end)
        .bind(&req.content)
        .bind(req.no_medication_needed)
        .bind(&req.treatments)
        .bind(&req.remark)
        .fetch_one(pool)
        .await?;

        Ok(observation)
    }

    /// 刪除觀察紀錄
    pub async fn soft_delete_observation(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM animal_observations WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 軟刪除觀察紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_observation_with_reason(pool: &PgPool, id: Uuid, reason: &str, deleted_by: Uuid) -> Result<()> {
        // 記錄到 change_reasons 表
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('observation', $1::text, 'DELETE', $2, $3)
            "#
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        // 軟刪除（更新 deleted_at 而非硬刪除）
        sqlx::query(
            r#"
            UPDATE animal_observations SET 
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

    /// 複製觀察紀錄
    pub async fn copy_observation(
        pool: &PgPool,
        animal_id: Uuid,
        source_id: Uuid,
        created_by: Uuid,
    ) -> Result<AnimalObservation> {
        let source = Self::get_observation_by_id(pool, source_id).await?;

        let observation = sqlx::query_as::<_, AnimalObservation>(
            r#"
            INSERT INTO animal_observations (
                animal_id, event_date, record_type, equipment_used, anesthesia_start,
                anesthesia_end, content, no_medication_needed, treatments, remark,
                created_by, created_at, updated_at
            )
            VALUES ($1, CURRENT_DATE, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(source.record_type)
        .bind(&source.equipment_used)
        .bind(source.anesthesia_start)
        .bind(source.anesthesia_end)
        .bind(&source.content)
        .bind(source.no_medication_needed)
        .bind(&source.treatments)
        .bind(&source.remark)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(observation)
    }

    /// 標記觀察紀錄獸醫師已讀
    pub async fn mark_observation_vet_read(pool: &PgPool, id: Uuid, vet_user_id: Uuid) -> Result<()> {
        // 更新紀錄本身
        sqlx::query(
            "UPDATE animal_observations SET vet_read = true, vet_read_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        // 記錄已讀歷史
        sqlx::query(
            r#"
            INSERT INTO observation_vet_reads (observation_id, vet_user_id, read_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (observation_id, vet_user_id) DO UPDATE SET read_at = NOW()
            "#
        )
        .bind(id)
        .bind(vet_user_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
