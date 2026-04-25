use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{AnimalMedicalService, AnimalService};
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalSurgery, CreateSurgeryRequest, SurgeryListItem,
        UpdateSurgeryRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, SignatureService,
    },
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

    /// 建立手術紀錄 — Service-driven audit
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateSurgeryRequest,
    ) -> Result<AnimalSurgery> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // 驗證 JSONB 欄位結構
        validate_surgery_jsonb_fields(
            &req.induction_anesthesia,
            &req.pre_surgery_medication,
            &req.anesthesia_maintenance,
            &req.vital_signs,
            &req.post_surgery_medication,
        )?;

        // 取得動物資訊用於 audit 顯示（Gemini PR #178：顯示 IACUC + 耳號 而非 UUID）
        let animal = AnimalService::get_by_id(pool, animal_id).await?;

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

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!("[{}] {} - {}", iacuc, animal.ear_tag, surgery.surgery_site);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "SURGERY_CREATE",
                entity: Some(AuditEntity::new(
                    "animal_surgery",
                    surgery.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&surgery)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(surgery)
    }

    /// 更新手術紀錄 — Service-driven audit
    ///
    /// 注意：`save_record_version` 目前接 `&PgPool`（GLP 強制版本歷史）。與
    /// `AnimalObservation::update` 同屬 R26-8 待 tx 化範圍 — 失敗僅丟失該版本
    /// 歷史，主流程繼續。audit log 仍在 tx 內，符合 R26 DoD-1。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateSurgeryRequest,
    ) -> Result<AnimalSurgery> {
        let user = actor.require_user()?;
        let updated_by = user.id;

        // 驗證 JSONB 欄位結構
        validate_surgery_jsonb_fields(
            &req.induction_anesthesia,
            &req.pre_surgery_medication,
            &req.anesthesia_maintenance,
            &req.vital_signs,
            &req.post_surgery_medication,
        )?;

        // C1 (GLP) fail-fast：簽章後鎖定的記錄拒絕修改
        SignatureService::ensure_not_locked_uuid(pool, "surgery", id).await?;

        // 先取得原始紀錄用於版本歷史 + audit before snapshot（tx 外 pool read OK）
        let before = Self::get_by_id(pool, id).await?;

        // 取得動物資訊用於 audit 顯示（Gemini PR #178：顯示 IACUC + 耳號 而非 UUID）
        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        // 保存版本歷史（目前 pool-based；tx 化歸 R26-8）
        AnimalMedicalService::save_record_version(pool, "surgery", id, &before, updated_by)
            .await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：tx 內以 FOR UPDATE 再次驗證，避免 fail-fast 與 UPDATE 之間的 race
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "surgery", id).await?;

        // Gemini PR #178：加 `AND deleted_at IS NULL` 避免更新已軟刪除紀錄；
        // 若不存在或已刪除，`fetch_one` 會回 `RowNotFound` 自動映射為 NotFound。
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
            WHERE id = $1 AND deleted_at IS NULL
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
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!("[{}] {} - {}", iacuc, animal.ear_tag, surgery.surgery_site);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "SURGERY_UPDATE",
                entity: Some(AuditEntity::new(
                    "animal_surgery",
                    surgery.id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&surgery))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
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

    /// 軟刪除手術紀錄（含刪除原因）- GLP 合規 — Service-driven audit
    pub async fn soft_delete_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;

        // C1 (GLP) fail-fast：簽章後鎖定的記錄拒絕刪除
        SignatureService::ensure_not_locked_uuid(pool, "surgery", id).await?;

        // before snapshot for audit（tx 外 pool read OK）
        let before = Self::get_by_id(pool, id).await?;

        // 取得動物資訊用於 audit 顯示（Gemini PR #178：顯示 IACUC + 耳號 而非 UUID）
        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        let mut tx = pool.begin().await?;

        // C1 atomic：tx 內以 FOR UPDATE 再次驗證鎖定狀態
        SignatureService::ensure_not_locked_uuid_tx(&mut tx, "surgery", id).await?;

        // Gemini PR #178：先 UPDATE 並檢查 rows_affected；若 0 回 NotFound 並
        // rollback（不會寫入 change_reasons 或 audit log）— 防止 race 下產生
        // 假刪除的審計紀錄。
        let rows = sqlx::query(
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
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFound(
                "手術紀錄不存在或已被刪除".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('surgery', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!(
            "[{}] {} - {} (原因: {})",
            iacuc, animal.ear_tag, before.surgery_site, reason
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "SURGERY_DELETE",
                entity: Some(AuditEntity::new("animal_surgery", id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
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
