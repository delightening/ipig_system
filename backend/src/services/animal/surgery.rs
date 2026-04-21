use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalMedicalService;
use crate::{
    models::{AnimalSurgery, CreateSurgeryRequest, SurgeryListItem, UpdateSurgeryRequest},
    utils::jsonb_validation::{validate_medication_jsonb, validate_vital_signs},
    AppError, Result,
};

pub struct AnimalSurgeryService;

/// 驗證手術紀錄中的所有 JSONB 藥物/生命徵象欄位
fn validate_surgery_jsonb_fields(
    induction: &Option<serde_json::Value>,
    pre_med: &Option<serde_json::Value>,
    maintenance: &Option<serde_json::Value>,
    vitals: &Option<serde_json::Value>,
    post_med: &Option<serde_json::Value>,
) -> crate::Result<()> {
    if let Some(ref v) = induction {
        validate_medication_jsonb("induction_anesthesia", v)?;
    }
    if let Some(ref v) = pre_med {
        validate_medication_jsonb("pre_surgery_medication", v)?;
    }
    if let Some(ref v) = maintenance {
        validate_medication_jsonb("anesthesia_maintenance", v)?;
    }
    if let Some(ref v) = vitals {
        validate_vital_signs(v)?;
    }
    if let Some(ref v) = post_med {
        validate_medication_jsonb("post_surgery_medication", v)?;
    }
    Ok(())
}

impl AnimalSurgeryService {
    // ============================================
    // 手術紀錄
    // ============================================

    /// 取得手術紀錄列表（排除已刪除，支援資料隔離）
    pub async fn list(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<AnimalSurgery>> {
        let surgeries = sqlx::query_as::<_, AnimalSurgery>(
            r#"SELECT s.*, u.display_name as created_by_name
               FROM animal_surgeries s
               LEFT JOIN users u ON s.created_by = u.id
               WHERE s.animal_id = $1 AND s.deleted_at IS NULL AND ($2::timestamptz IS NULL OR s.created_at > $2)
               ORDER BY s.surgery_date DESC"#
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(surgeries)
    }

    /// 取得手術紀錄列表（含獸醫師建議數量，支援資料隔離）
    pub async fn list_with_recommendations(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<SurgeryListItem>> {
        let surgeries = sqlx::query_as::<_, SurgeryListItem>(
            r#"
            SELECT 
                s.id, s.animal_id, s.is_first_experiment, s.surgery_date, s.surgery_site,
                s.no_medication_needed, s.vet_read, s.vet_read_at,
                s.created_by, s.created_at,
                (SELECT COUNT(*) FROM vet_recommendations vr WHERE vr.record_type = 'surgery'::vet_record_type AND vr.record_id = s.id) as recommendation_count
            FROM animal_surgeries s
            WHERE s.animal_id = $1 AND s.deleted_at IS NULL AND ($2::timestamptz IS NULL OR s.created_at > $2)
            ORDER BY s.surgery_date DESC
            "#
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(surgeries)
    }

    /// 取得單一手術紀錄
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalSurgery> {
        let surgery =
            sqlx::query_as::<_, AnimalSurgery>("SELECT * FROM animal_surgeries WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("Surgery not found".to_string()))?;

        Ok(surgery)
    }

    /// 新增手術紀錄（Service-driven audit）
    pub async fn create(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        animal_id: Uuid,
        req: &CreateSurgeryRequest,
    ) -> Result<AnimalSurgery> {
        // 驗證 JSONB 欄位結構
        validate_surgery_jsonb_fields(
            &req.induction_anesthesia,
            &req.pre_surgery_medication,
            &req.anesthesia_maintenance,
            &req.vital_signs,
            &req.post_surgery_medication,
        )?;

        let created_by = actor.actor_user_id().unwrap_or(crate::middleware::SYSTEM_USER_ID);
        let mut tx = pool.begin().await?;

        let surgery = sqlx::query_as::<_, AnimalSurgery>(
            r#"
            INSERT INTO animal_surgeries (
                animal_id, is_first_experiment, surgery_date, surgery_site,
                induction_anesthesia, pre_surgery_medication, positioning,
                anesthesia_maintenance, anesthesia_observation, vital_signs,
                reflex_recovery, respiration_rate, post_surgery_medication,
                remark, no_medication_needed, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(req.is_first_experiment)
        .bind(req.surgery_date)
        .bind(&req.surgery_site)
        .bind(&req.induction_anesthesia)
        .bind(&req.pre_surgery_medication)
        .bind(&req.positioning)
        .bind(&req.anesthesia_maintenance)
        .bind(&req.anesthesia_observation)
        .bind(&req.vital_signs)
        .bind(&req.reflex_recovery)
        .bind(req.respiration_rate)
        .bind(&req.post_surgery_medication)
        .bind(&req.remark)
        .bind(req.no_medication_needed)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("手術 {}", surgery.surgery_date);
        crate::services::AuditService::log_activity_tx(
            &mut tx,
            actor,
            crate::services::audit::ActivityLogEntry::create(
                "ANIMAL",
                "SURGERY_CREATE",
                crate::services::audit::AuditEntity::new("animal_surgery", surgery.id, &display),
                &surgery,
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(surgery)
    }

    /// 更新手術紀錄
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateSurgeryRequest,
        updated_by: Uuid,
    ) -> Result<AnimalSurgery> {
        // 驗證 JSONB 欄位結構
        validate_surgery_jsonb_fields(
            &req.induction_anesthesia,
            &req.pre_surgery_medication,
            &req.anesthesia_maintenance,
            &req.vital_signs,
            &req.post_surgery_medication,
        )?;

        // 先取得原始紀錄用於版本歷史
        let original = Self::get_by_id(pool, id).await?;

        // 保存版本歷史
        AnimalMedicalService::save_record_version(pool, "surgery", id, &original, updated_by)
            .await?;

        let surgery = sqlx::query_as::<_, AnimalSurgery>(
            r#"
            UPDATE animal_surgeries SET
                is_first_experiment = COALESCE($2, is_first_experiment),
                surgery_date = COALESCE($3, surgery_date),
                surgery_site = COALESCE($4, surgery_site),
                induction_anesthesia = COALESCE($5, induction_anesthesia),
                pre_surgery_medication = COALESCE($6, pre_surgery_medication),
                positioning = COALESCE($7, positioning),
                anesthesia_maintenance = COALESCE($8, anesthesia_maintenance),
                anesthesia_observation = COALESCE($9, anesthesia_observation),
                vital_signs = COALESCE($10, vital_signs),
                reflex_recovery = COALESCE($11, reflex_recovery),
                respiration_rate = COALESCE($12, respiration_rate),
                post_surgery_medication = COALESCE($13, post_surgery_medication),
                remark = COALESCE($14, remark),
                no_medication_needed = COALESCE($15, no_medication_needed),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.is_first_experiment)
        .bind(req.surgery_date)
        .bind(&req.surgery_site)
        .bind(&req.induction_anesthesia)
        .bind(&req.pre_surgery_medication)
        .bind(&req.positioning)
        .bind(&req.anesthesia_maintenance)
        .bind(&req.anesthesia_observation)
        .bind(&req.vital_signs)
        .bind(&req.reflex_recovery)
        .bind(req.respiration_rate)
        .bind(&req.post_surgery_medication)
        .bind(&req.remark)
        .bind(req.no_medication_needed)
        .fetch_one(pool)
        .await?;

        Ok(surgery)
    }

    /// 刪除手術紀錄
    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM animal_surgeries WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除手術紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('surgery', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE animal_surgeries SET 
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

    /// 複製手術紀錄
    pub async fn copy(
        pool: &PgPool,
        animal_id: Uuid,
        source_id: Uuid,
        created_by: Uuid,
    ) -> Result<AnimalSurgery> {
        let source = Self::get_by_id(pool, source_id).await?;

        let surgery = sqlx::query_as::<_, AnimalSurgery>(
            r#"
            INSERT INTO animal_surgeries (
                animal_id, is_first_experiment, surgery_date, surgery_site,
                induction_anesthesia, pre_surgery_medication, positioning,
                anesthesia_maintenance, anesthesia_observation, vital_signs,
                reflex_recovery, respiration_rate, post_surgery_medication,
                remark, no_medication_needed, created_by, created_at, updated_at
            )
            VALUES ($1, false, CURRENT_DATE, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(&source.surgery_site)
        .bind(&source.induction_anesthesia)
        .bind(&source.pre_surgery_medication)
        .bind(&source.positioning)
        .bind(&source.anesthesia_maintenance)
        .bind(&source.anesthesia_observation)
        .bind(&source.vital_signs)
        .bind(&source.reflex_recovery)
        .bind(source.respiration_rate)
        .bind(&source.post_surgery_medication)
        .bind(&source.remark)
        .bind(source.no_medication_needed)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(surgery)
    }

    /// 標記手術紀錄獸醫師已讀
    pub async fn mark_vet_read(pool: &PgPool, id: Uuid, vet_user_id: Uuid) -> Result<()> {
        // 更新紀錄本身
        sqlx::query(
            "UPDATE animal_surgeries SET vet_read = true, vet_read_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        // 記錄已讀歷史
        sqlx::query(
            r#"
            INSERT INTO surgery_vet_reads (surgery_id, vet_user_id, read_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (surgery_id, vet_user_id) DO UPDATE SET read_at = NOW()
            "#,
        )
        .bind(id)
        .bind(vet_user_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
