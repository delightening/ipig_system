use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{AnimalMedicalService, AnimalService};
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalObservation, CreateObservationRequest, ObservationListItem,
        UpdateObservationRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, SignatureService,
    },
    utils::jsonb_validation::{validate_equipment_used, validate_treatments},
    AppError, Result,
};

pub struct AnimalObservationService;

impl AnimalObservationService {
    // ============================================
    // 觀察試驗紀錄
    // ============================================

    /// 取得觀察紀錄列表（排除已刪除，支援資料隔離）
    pub async fn list(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<AnimalObservation>> {
        let observations = sqlx::query_as::<_, AnimalObservation>(
            r#"SELECT o.*, u.display_name as created_by_name
               FROM animal_observations o
               LEFT JOIN users u ON o.created_by = u.id
               WHERE o.animal_id = $1 AND o.deleted_at IS NULL AND ($2::timestamptz IS NULL OR o.created_at > $2)
               ORDER BY o.event_date DESC"#
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得觀察紀錄列表（含獸醫師建議數量，支援資料隔離）
    pub async fn list_with_recommendations(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<ObservationListItem>> {
        let observations = sqlx::query_as::<_, ObservationListItem>(
            r#"
            SELECT 
                o.id, o.animal_id, o.event_date, o.record_type, o.content,
                o.no_medication_needed, o.vet_read, o.vet_read_at,
                o.created_by, o.created_at,
                (SELECT COUNT(*) FROM vet_recommendations vr WHERE vr.record_type = 'observation'::vet_record_type AND vr.record_id = o.id) as recommendation_count
            FROM animal_observations o
            WHERE o.animal_id = $1 AND o.deleted_at IS NULL AND ($2::timestamptz IS NULL OR o.created_at > $2)
            ORDER BY o.event_date DESC
            "#
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得單一觀察紀錄
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalObservation> {
        let observation = sqlx::query_as::<_, AnimalObservation>(
            "SELECT * FROM animal_observations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Observation not found".to_string()))?;

        Ok(observation)
    }

    /// 建立觀察紀錄 — Service-driven audit
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateObservationRequest,
    ) -> Result<AnimalObservation> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // 驗證 JSONB 欄位結構
        if let Some(ref eq) = req.equipment_used {
            validate_equipment_used(eq)?;
        }
        if let Some(ref tr) = req.treatments {
            validate_treatments(tr)?;
        }

        // 如果是緊急給藥，設定狀態為 pending_review
        let emergency_status = if req.is_emergency {
            Some("pending_review".to_string())
        } else {
            None
        };

        // H5：audit display 帶 IACUC + 耳號（與 surgery / blood_test 一致）
        let animal = AnimalService::get_by_id(pool, animal_id).await?;

        let mut tx = pool.begin().await?;

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
            "#,
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
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!(
            "[{}] {} @ {}: {:?}",
            iacuc, animal.ear_tag, observation.event_date, observation.record_type
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "OBSERVATION_CREATE",
                entity: Some(AuditEntity::new(
                    "animal_observation",
                    observation.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&observation)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(observation)
    }

    /// 更新觀察紀錄 — Service-driven audit
    ///
    /// 注意：`save_record_version` 目前接 `&PgPool`（GLP 強制版本歷史）。若改 tx 版
    /// 需調整 `AnimalMedicalService`；本 PR 暫以 pool 版本使用，接受版本歷史寫入
    /// 不在 tx 內的輕微風險（失敗僅丟失該版本歷史，主流程繼續 — 與 R26-8 同類
    /// 待進一步 tx 化）。audit log 仍在 tx 內，符合 R26 DoD-1。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateObservationRequest,
    ) -> Result<AnimalObservation> {
        let user = actor.require_user()?;
        let updated_by = user.id;

        // 驗證 JSONB 欄位結構
        if let Some(ref eq) = req.equipment_used {
            validate_equipment_used(eq)?;
        }
        if let Some(ref tr) = req.treatments {
            validate_treatments(tr)?;
        }

        // C1 (GLP) fail-fast：簽章後鎖定的記錄拒絕修改
        SignatureService::ensure_not_locked_uuid(pool, "observation", id).await?;

        // 先取得原始紀錄用於版本歷史（在 tx 外查詢，pool read OK）
        let before = Self::get_by_id(pool, id).await?;

        // H5：audit display 帶 IACUC + 耳號
        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        // 保存版本歷史（目前 pool-based；tx 化歸 R26-8）
        AnimalMedicalService::save_record_version(pool, "observation", id, &before, updated_by)
            .await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：tx 內以 FOR UPDATE 再次驗證，避免 fail-fast 與 UPDATE 之間的 race
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "observation", id).await?;

        let after = sqlx::query_as::<_, AnimalObservation>(
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
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
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
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!(
            "[{}] {} @ {}: {:?}",
            iacuc, animal.ear_tag, after.event_date, after.record_type
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "OBSERVATION_UPDATE",
                entity: Some(AuditEntity::new("animal_observation", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 刪除觀察紀錄
    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM animal_observations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除觀察紀錄（含刪除原因）— Service-driven audit (GLP 合規)
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;

        // C1 (GLP) fail-fast：簽章後鎖定的觀察紀錄拒絕刪除（與 update / 其他 service
        // soft_delete 對齊雙層守衛 pattern：避免空開 tx + 取 row lock 才被擋下）
        SignatureService::ensure_not_locked_uuid(pool, "observation", id).await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：拒絕刪除已鎖定（已簽章）記錄
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "observation", id).await?;

        // SELECT FOR UPDATE 取 before；同時守門防止重複刪除
        let before = sqlx::query_as::<_, AnimalObservation>(
            "SELECT * FROM animal_observations WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("觀察紀錄不存在或已刪除".into()))?;

        // H5：audit display 帶 IACUC + 耳號（取 before.animal_id 對應的動物）
        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        // 記錄到 change_reasons 表
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('observation', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?;

        // 軟刪除（更新 deleted_at 而非硬刪除）
        let after = sqlx::query_as::<_, AnimalObservation>(
            r#"
            UPDATE animal_observations SET
                deleted_at = NOW(),
                deletion_reason = $2,
                deleted_by = $3,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!(
            "[{}] {} @ {}: {:?} — {}",
            iacuc, animal.ear_tag, before.event_date, before.record_type, reason
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "OBSERVATION_DELETE",
                entity: Some(AuditEntity::new("animal_observation", before.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 複製觀察紀錄
    pub async fn copy(
        pool: &PgPool,
        animal_id: Uuid,
        source_id: Uuid,
        created_by: Uuid,
    ) -> Result<AnimalObservation> {
        let source = Self::get_by_id(pool, source_id).await?;

        let observation = sqlx::query_as::<_, AnimalObservation>(
            r#"
            INSERT INTO animal_observations (
                animal_id, event_date, record_type, equipment_used, anesthesia_start,
                anesthesia_end, content, no_medication_needed, treatments, remark,
                created_by, created_at, updated_at
            )
            VALUES ($1, CURRENT_DATE, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING *
            "#,
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
    pub async fn mark_vet_read(pool: &PgPool, id: Uuid, vet_user_id: Uuid) -> Result<()> {
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
            "#,
        )
        .bind(id)
        .bind(vet_user_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
