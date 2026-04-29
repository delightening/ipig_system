use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Animal, AnimalExportRecord, AnimalImportBatch, AnimalSacrifice,
        AnimalStatus, AnimalSuddenDeath, AnimalVaccination, CreateSacrificeRequest,
        CreateSuddenDeathRequest, CreateVaccinationRequest, CreateVetRecommendationRequest,
        CreateVetRecommendationWithAttachmentsRequest, ExportFormat, ExportType, ImportStatus,
        ImportType, UpdateVaccinationRequest, VersionDiff, VersionHistoryResponse,
        VetRecommendation, VetRecordType,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

pub struct AnimalMedicalService;

impl AnimalMedicalService {
    // ============================================
    // 疫苗/驅蟲紀錄
    // ============================================

    /// 取得疫苗紀錄列表（排除已刪除，支援資料隔離）
    pub async fn list_vaccinations(
        pool: &PgPool,
        animal_id: Uuid,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<AnimalVaccination>> {
        let vaccinations = sqlx::query_as::<_, AnimalVaccination>(
            "SELECT * FROM animal_vaccinations WHERE animal_id = $1 AND deleted_at IS NULL AND ($2::timestamptz IS NULL OR created_at > $2) ORDER BY administered_date DESC"
        )
        .bind(animal_id)
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(vaccinations)
    }

    /// 取得單一疫苗紀錄（SDD before snapshot 用）
    pub async fn get_vaccination_by_id(pool: &PgPool, id: Uuid) -> Result<AnimalVaccination> {
        sqlx::query_as::<_, AnimalVaccination>(
            "SELECT * FROM animal_vaccinations WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Vaccination not found".to_string()))
    }

    /// 建立疫苗紀錄 — Service-driven audit
    pub async fn create_vaccination(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateVaccinationRequest,
    ) -> Result<AnimalVaccination> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // 取得動物資訊用於 audit 顯示（Gemini PR #178 pattern）
        let animal = AnimalService::get_by_id(pool, animal_id).await?;

        let mut tx = pool.begin().await?;

        let vaccination = sqlx::query_as::<_, AnimalVaccination>(
            r#"
            INSERT INTO animal_vaccinations (animal_id, administered_date, vaccine, deworming_dose, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(req.administered_date)
        .bind(&req.vaccine)
        .bind(&req.deworming_dose)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let vaccine_name = vaccination.vaccine.as_deref().unwrap_or("未指定疫苗");
        let display = format!("[{}] {} - {}", iacuc, animal.ear_tag, vaccine_name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VACCINATION_CREATE",
                entity: Some(AuditEntity::new(
                    "animal_vaccination",
                    vaccination.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&vaccination)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(vaccination)
    }

    /// 更新疫苗紀錄 — Service-driven audit
    pub async fn update_vaccination(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateVaccinationRequest,
    ) -> Result<AnimalVaccination> {
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // Gemini PR #181 HIGH：before snapshot 在 tx 內 + FOR UPDATE 鎖行，避免
        // 讀取到更新之間記錄被其他請求修改、DataDiff 失準。
        let before = sqlx::query_as::<_, AnimalVaccination>(
            "SELECT * FROM animal_vaccinations WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Vaccination not found".to_string()))?;

        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        // Gemini PR #178 pattern：WHERE 加 deleted_at IS NULL 避免更新已軟刪除紀錄
        let vaccination = sqlx::query_as::<_, AnimalVaccination>(
            r#"
            UPDATE animal_vaccinations SET
                administered_date = COALESCE($2, administered_date),
                vaccine = COALESCE($3, vaccine),
                deworming_dose = COALESCE($4, deworming_dose)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.administered_date)
        .bind(&req.vaccine)
        .bind(&req.deworming_dose)
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let vaccine_name = vaccination.vaccine.as_deref().unwrap_or("未指定疫苗");
        let display = format!("[{}] {} - {}", iacuc, animal.ear_tag, vaccine_name);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VACCINATION_UPDATE",
                entity: Some(AuditEntity::new(
                    "animal_vaccination",
                    vaccination.id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&vaccination))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(vaccination)
    }

    /// 軟刪除疫苗紀錄（含刪除原因）- GLP 合規 — Service-driven audit
    pub async fn soft_delete_vaccination_with_reason(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let deleted_by = user.id;

        let mut tx = pool.begin().await?;

        // Gemini PR #181 HIGH：before snapshot 在 tx 內 + FOR UPDATE
        let before = sqlx::query_as::<_, AnimalVaccination>(
            "SELECT * FROM animal_vaccinations WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Vaccination not found".to_string()))?;

        let animal = AnimalService::get_by_id(pool, before.animal_id).await?;

        // Gemini PR #178 pattern：先 UPDATE + 檢查 rows_affected，避免假刪除審計
        let rows = sqlx::query(
            r#"
            UPDATE animal_vaccinations SET
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
                "疫苗紀錄不存在或已被刪除".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('vaccination', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let vaccine_name = before.vaccine.as_deref().unwrap_or("未指定疫苗");
        let display = format!(
            "[{}] {} - {} (原因: {})",
            iacuc, animal.ear_tag, vaccine_name, reason
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VACCINATION_DELETE",
                entity: Some(AuditEntity::new("animal_vaccination", id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // 犧牲/採樣紀錄
    // ============================================

    pub async fn get_sacrifice(pool: &PgPool, animal_id: Uuid) -> Result<Option<AnimalSacrifice>> {
        let sacrifice = sqlx::query_as::<_, AnimalSacrifice>(
            "SELECT * FROM animal_sacrifices WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(sacrifice)
    }

    /// 建立或更新犧牲紀錄 — Service-driven audit
    ///
    /// CRIT-02 修補：當 `confirmed_sacrifice = true` 且動物狀態可轉換時，
    /// 動物狀態 UPDATE 與 sacrifice upsert + audit 收歸同 tx，避免原本
    /// handler 層分兩次 pool 呼叫造成「sacrifice 已寫但 animal 未 euthanized」。
    pub async fn upsert_sacrifice(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateSacrificeRequest,
    ) -> Result<AnimalSacrifice> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let mut tx = pool.begin().await?;

        // Gemini PR #183 HIGH：animal + before 都在 tx 內讀，animal 加 FOR UPDATE
        // 序列化狀態轉換，避免 race（兩個並發 confirmed_sacrifice 都通過檢查）
        let animal: crate::models::Animal = sqlx::query_as(
            "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        // Gemini PR #183 HIGH：confirmed_sacrifice + 無法轉換時明確拒絕，避免
        // 產生「犧牲紀錄已確認但動物狀態未變更」的不一致（比照 sudden_death）。
        // can_transition_to 內部已處理 terminal 狀態，移除多餘 is_terminal 檢查。
        if req.confirmed_sacrifice
            && !animal.status.can_transition_to(AnimalStatus::Euthanized)
        {
            return Err(AppError::BadRequest(format!(
                "無法將「{}」狀態的動物標為安樂死",
                animal.status.display_name()
            )));
        }

        let before = sqlx::query_as::<_, AnimalSacrifice>(
            "SELECT * FROM animal_sacrifices WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?;

        let sacrifice = sqlx::query_as::<_, AnimalSacrifice>(
            r#"
            INSERT INTO animal_sacrifices (
                animal_id, sacrifice_date, zoletil_dose, method_electrocution,
                method_bloodletting, method_other, sampling, sampling_other,
                blood_volume_ml, confirmed_sacrifice, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            ON CONFLICT (animal_id) DO UPDATE SET
                sacrifice_date = EXCLUDED.sacrifice_date,
                zoletil_dose = EXCLUDED.zoletil_dose,
                method_electrocution = EXCLUDED.method_electrocution,
                method_bloodletting = EXCLUDED.method_bloodletting,
                method_other = EXCLUDED.method_other,
                sampling = EXCLUDED.sampling,
                sampling_other = EXCLUDED.sampling_other,
                blood_volume_ml = EXCLUDED.blood_volume_ml,
                confirmed_sacrifice = EXCLUDED.confirmed_sacrifice,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(animal_id)
        .bind(req.sacrifice_date)
        .bind(&req.zoletil_dose)
        .bind(req.method_electrocution)
        .bind(req.method_bloodletting)
        .bind(&req.method_other)
        .bind(&req.sampling)
        .bind(&req.sampling_other)
        .bind(req.blood_volume_ml)
        .bind(req.confirmed_sacrifice)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 犧牲確認時自動將動物狀態設為 euthanized；animal 已在 tx 內 FOR UPDATE 鎖住，
        // 此 UPDATE 安全。上方已驗過 can_transition_to，這裡無需重複檢查。
        if req.confirmed_sacrifice {
            sqlx::query(
                "UPDATE animals SET status = 'euthanized', pen_location = NULL, updated_at = NOW() WHERE id = $1",
            )
            .bind(animal_id)
            .execute(&mut *tx)
            .await?;
        }

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let mut methods: Vec<&str> = Vec::new();
        if req.method_electrocution {
            methods.push("電擊");
        }
        if req.method_bloodletting {
            methods.push("放血");
        }
        if let Some(ref m) = req.method_other {
            methods.push(m);
        }
        let method_str = if methods.is_empty() {
            "未指定方式".to_string()
        } else {
            methods.join("+")
        };
        let display = format!("[{}] {} - {}", iacuc, animal.ear_tag, method_str);
        let data_diff = match &before {
            Some(b) => DataDiff::compute(Some(b), Some(&sacrifice)),
            None => DataDiff::create_only(&sacrifice),
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "SACRIFICE_UPSERT",
                entity: Some(AuditEntity::new("animal_sacrifice", sacrifice.id, &display)),
                data_diff: Some(data_diff),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(sacrifice)
    }

    // ============================================
    // 猝死記錄
    // ============================================

    /// 取得動物的猝死記錄
    pub async fn get_sudden_death(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<Option<AnimalSuddenDeath>> {
        let record = sqlx::query_as::<_, AnimalSuddenDeath>(
            "SELECT * FROM animal_sudden_deaths WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// 登記猝死 — Service-driven audit（CRIT-02 兩表變更合併為同 tx）
    pub async fn create_sudden_death(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateSuddenDeathRequest,
    ) -> Result<AnimalSuddenDeath> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let mut tx = pool.begin().await?;

        // Gemini PR #183 HIGH：animal 在 tx 內 + FOR UPDATE 鎖住，序列化狀態轉換
        let animal: crate::models::Animal = sqlx::query_as(
            "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        // 驗證動物狀態可轉換到 SuddenDeath
        if !animal.status.can_transition_to(AnimalStatus::SuddenDeath) {
            return Err(AppError::BadRequest(format!(
                "無法將「{}」狀態的動物登記為猝死",
                animal.status.display_name()
            )));
        }

        // 建立猝死記錄
        let record = sqlx::query_as::<_, AnimalSuddenDeath>(
            r#"
            INSERT INTO animal_sudden_deaths (
                animal_id, discovered_at, discovered_by, probable_cause,
                iacuc_no, location, remark, requires_pathology, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING *
            "#,
        )
        .bind(animal_id)
        .bind(req.discovered_at)
        .bind(created_by)
        .bind(&req.probable_cause)
        .bind(&animal.iacuc_no)
        .bind(&req.location)
        .bind(&req.remark)
        .bind(req.requires_pathology)
        .fetch_one(&mut *tx)
        .await?;

        // Gemini PR #183 HIGH：猝死比照犧牲紀錄清 pen_location（動物已移出欄位）；
        // animal 已鎖，此處無需額外 status check。
        sqlx::query("UPDATE animals SET status = 'sudden_death', pen_location = NULL, updated_at = NOW() WHERE id = $1")
            .bind(animal_id)
            .execute(&mut *tx)
            .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!("[{}] {} - 猝死", iacuc, animal.ear_tag);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "SUDDEN_DEATH",
                entity: Some(AuditEntity::new("animal_sudden_deaths", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    // ============================================
    // 獸醫師功能
    // ============================================

    /// 標記獸醫師已讀
    pub async fn mark_vet_read(pool: &PgPool, animal_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE animals SET
                vet_last_viewed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(animal_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 新增獸醫師建議 — Service-driven audit
    pub async fn add_vet_recommendation(
        pool: &PgPool,
        actor: &ActorContext,
        record_type: VetRecordType,
        record_id: Uuid,
        req: &CreateVetRecommendationRequest,
    ) -> Result<VetRecommendation> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let mut tx = pool.begin().await?;

        let recommendation = sqlx::query_as::<_, VetRecommendation>(
            r#"
            INSERT INTO vet_recommendations (record_type, record_id, content, is_urgent, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#
        )
        .bind(record_type)
        .bind(record_id)
        .bind(&req.content)
        .bind(req.is_urgent)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let record_type_str = match record_type {
            VetRecordType::Observation => "觀察紀錄",
            VetRecordType::Surgery => "手術紀錄",
        };
        let display = format!(
            "{} #{} 獸醫建議{}",
            record_type_str,
            record_id,
            if req.is_urgent { " (緊急)" } else { "" }
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_RECOMMENDATION_ADD",
                entity: Some(AuditEntity::new(
                    "vet_recommendation",
                    recommendation.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&recommendation)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(recommendation)
    }

    /// 新增獸醫師建議（含附件）— Service-driven audit
    pub async fn add_vet_recommendation_with_attachments(
        pool: &PgPool,
        actor: &ActorContext,
        record_type: VetRecordType,
        record_id: Uuid,
        req: &CreateVetRecommendationWithAttachmentsRequest,
    ) -> Result<VetRecommendation> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // 驗證 attachments JSONB 結構
        if let Some(ref att) = req.attachments {
            crate::utils::jsonb_validation::validate_attachments(att)?;
        }

        let mut tx = pool.begin().await?;

        let recommendation = sqlx::query_as::<_, VetRecommendation>(
            r#"
            INSERT INTO vet_recommendations (record_type, record_id, content, attachments, is_urgent, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#
        )
        .bind(record_type)
        .bind(record_id)
        .bind(&req.content)
        .bind(&req.attachments)
        .bind(req.is_urgent)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let record_type_str = match record_type {
            VetRecordType::Observation => "觀察紀錄",
            VetRecordType::Surgery => "手術紀錄",
        };
        let display = format!(
            "{} #{} 獸醫建議 (含附件){}",
            record_type_str,
            record_id,
            if req.is_urgent { " (緊急)" } else { "" }
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_RECOMMENDATION_ADD",
                entity: Some(AuditEntity::new(
                    "vet_recommendation",
                    recommendation.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&recommendation)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(recommendation)
    }

    /// 取得紀錄的獸醫師建議
    pub async fn get_vet_recommendations(
        pool: &PgPool,
        record_type: VetRecordType,
        record_id: Uuid,
    ) -> Result<Vec<VetRecommendation>> {
        let recommendations = sqlx::query_as::<_, VetRecommendation>(
            "SELECT * FROM vet_recommendations WHERE record_type = $1 AND record_id = $2 ORDER BY created_at DESC"
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        Ok(recommendations)
    }

    /// 取得動物的所有獸醫師建議（彙整觀察紀錄 + 手術紀錄）
    pub async fn get_vet_recommendations_by_animal(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<Vec<VetRecommendation>> {
        let recommendations = sqlx::query_as::<_, VetRecommendation>(
            r#"
            SELECT vr.*
            FROM vet_recommendations vr
            WHERE vr.record_id IN (
                SELECT id FROM animal_observations WHERE animal_id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT id FROM animal_surgeries WHERE animal_id = $1 AND deleted_at IS NULL
            )
            ORDER BY vr.created_at DESC
            "#,
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(recommendations)
    }

    // ============================================
    // 版本歷史功能
    // ============================================

    /// 保存紀錄版本歷史
    pub(super) async fn save_record_version<T: serde::Serialize>(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
        snapshot: &T,
        changed_by: Uuid,
    ) -> Result<()> {
        // 取得當前最大版本號（migration 013 為 ASSIGNMENT cast，避免 text::version_record_type 遞迴，改為 record_type::text 比較）
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version_no) FROM record_versions WHERE record_type::text = $1 AND record_id = $2",
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_one(pool)
        .await?;

        let next_version = max_version.unwrap_or(0) + 1;
        let snapshot_json = serde_json::to_value(snapshot).unwrap_or(serde_json::Value::Null);

        sqlx::query(
            r#"
            INSERT INTO record_versions (record_type, record_id, version_no, snapshot, changed_by, changed_at)
            VALUES ($1::version_record_type, $2, $3, $4, $5, NOW())
            "#
        )
        .bind(record_type)
        .bind(record_id)
        .bind(next_version)
        .bind(snapshot_json)
        .bind(changed_by)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 取得紀錄版本歷史（含 changed_by_name，前端相容格式）
    pub async fn get_record_versions(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
    ) -> Result<VersionHistoryResponse> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            version_no: i32,
            snapshot: serde_json::Value,
            diff_summary: Option<String>,
            changed_by: Option<Uuid>,
            changed_at: DateTime<Utc>,
            changed_by_name: Option<String>,
        }

        let versions = sqlx::query_as::<_, Row>(
            r#"
            SELECT rv.id, rv.version_no, rv.snapshot, rv.diff_summary, rv.changed_by, rv.changed_at,
                   u.display_name AS changed_by_name
            FROM record_versions rv
            LEFT JOIN users u ON rv.changed_by = u.id
            WHERE rv.record_type::text = $1 AND rv.record_id = $2
            ORDER BY rv.version_no DESC
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        let current_version = versions.first().map(|v| v.version_no).unwrap_or(1);
        let version_diffs: Vec<VersionDiff> = versions
            .into_iter()
            .map(|v| VersionDiff {
                id: v.id,
                version_no: v.version_no,
                changed_at: v.changed_at,
                changed_by: v.changed_by,
                snapshot: v.snapshot,
                diff_summary: v.diff_summary,
                changed_by_name: v.changed_by_name,
            })
            .collect();

        Ok(VersionHistoryResponse {
            record_type: record_type.to_string(),
            record_id,
            current_version,
            versions: version_diffs,
        })
    }

    // ============================================
    // 匯入功能
    // ============================================

    /// 建立匯入批次記錄
    pub async fn create_import_batch(
        pool: &PgPool,
        import_type: ImportType,
        file_name: &str,
        total_rows: i32,
        created_by: Uuid,
    ) -> Result<AnimalImportBatch> {
        let batch = sqlx::query_as::<_, AnimalImportBatch>(
            r#"
            INSERT INTO animal_import_batches (id, import_type, file_name, total_rows, status, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(import_type)
        .bind(file_name)
        .bind(total_rows)
        .bind(ImportStatus::Processing)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(batch)
    }

    /// 更新匯入批次結果
    pub async fn update_import_batch_result(
        pool: &PgPool,
        batch_id: Uuid,
        success_count: i32,
        error_count: i32,
        error_details: Option<serde_json::Value>,
    ) -> Result<AnimalImportBatch> {
        let status = ImportStatus::Completed; // 無論部分成功或全部成功都標記為完成

        let batch = sqlx::query_as::<_, AnimalImportBatch>(
            r#"
            UPDATE animal_import_batches SET
                success_count = $2,
                error_count = $3,
                error_details = $4,
                status = $5,
                completed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(batch_id)
        .bind(success_count)
        .bind(error_count)
        .bind(error_details)
        .bind(status)
        .fetch_one(pool)
        .await?;

        Ok(batch)
    }

    /// 取得匯入批次列表
    pub async fn list_import_batches(pool: &PgPool, limit: i32) -> Result<Vec<AnimalImportBatch>> {
        let batches = sqlx::query_as::<_, AnimalImportBatch>(
            "SELECT * FROM animal_import_batches ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(batches)
    }

    // ============================================
    // 匯出功能
    // ============================================

    /// 建立匯出記錄
    pub async fn create_export_record(
        pool: &PgPool,
        animal_id: Option<Uuid>,
        iacuc_no: Option<&str>,
        export_type: ExportType,
        export_format: ExportFormat,
        file_path: Option<&str>,
        created_by: Uuid,
    ) -> Result<AnimalExportRecord> {
        let record = sqlx::query_as::<_, AnimalExportRecord>(
            r#"
            INSERT INTO animal_export_records (id, animal_id, iacuc_no, export_type, export_format, file_path, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(animal_id)
        .bind(iacuc_no)
        .bind(export_type)
        .bind(export_format)
        .bind(file_path)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 取得動物完整病歷資料（用於匯出）
    pub async fn get_animal_medical_data(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<serde_json::Value> {
        let animal = AnimalService::get_by_id(pool, animal_id).await?;
        let observations = super::AnimalObservationService::list(pool, animal_id, None).await?;
        let surgeries = super::AnimalSurgeryService::list(pool, animal_id, None).await?;
        let weights = super::AnimalWeightService::list(pool, animal_id, None).await?;
        let vaccinations = Self::list_vaccinations(pool, animal_id, None).await?;
        let sacrifice = Self::get_sacrifice(pool, animal_id).await?;

        let data = serde_json::json!({
            "animal": animal,
            "observations": observations,
            "surgeries": surgeries,
            "weights": weights,
            "vaccinations": vaccinations,
            "sacrifice": sacrifice,
        });

        Ok(data)
    }

    /// 取得計劃下所有動物病歷資料（用於匯出）
    pub async fn get_project_medical_data(
        pool: &PgPool,
        iacuc_no: &str,
    ) -> Result<serde_json::Value> {
        let animals = sqlx::query_as::<_, Animal>(
            "SELECT * FROM animals WHERE iacuc_no = $1 AND deleted_at IS NULL ORDER BY id",
        )
        .bind(iacuc_no)
        .fetch_all(pool)
        .await?;

        let mut animal_data = Vec::new();
        for animal in animals {
            let data = Self::get_animal_medical_data(pool, animal.id).await?;
            animal_data.push(data);
        }

        Ok(serde_json::json!({
            "iacuc_no": iacuc_no,
            "animals": animal_data,
        }))
    }

    // ============================================
    // 病理報告功能
    // ============================================

    /// 取得病理報告
    pub async fn get_pathology_report(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<Option<crate::models::AnimalPathologyReport>> {
        let report = sqlx::query_as::<_, crate::models::AnimalPathologyReport>(
            "SELECT * FROM animal_pathology_reports WHERE animal_id = $1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(report)
    }

    /// 建立或更新病理報告 — Service-driven audit
    pub async fn upsert_pathology_report(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
    ) -> Result<crate::models::AnimalPathologyReport> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let animal = AnimalService::get_by_id(pool, animal_id).await?;
        let before = Self::get_pathology_report(pool, animal_id).await?;

        let mut tx = pool.begin().await?;

        let report = sqlx::query_as::<_, crate::models::AnimalPathologyReport>(
            r#"
            INSERT INTO animal_pathology_reports (animal_id, created_by, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (animal_id) DO UPDATE SET updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(animal_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
        let display = format!("[{}] {}", iacuc, animal.ear_tag);
        let data_diff = match &before {
            Some(b) => DataDiff::compute(Some(b), Some(&report)),
            None => DataDiff::create_only(&report),
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "PATHOLOGY_UPSERT",
                entity: Some(AuditEntity::new("animal_pathology", report.id, &display)),
                data_diff: Some(data_diff),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(report)
    }
    // ============================================
    // 匯入匯出批次管理
    // ============================================

    /// 更新匯入批次狀態
    pub async fn update_import_batch_status(
        pool: &PgPool,
        id: Uuid,
        status: crate::models::ImportStatus,
        success_count: i32,
        error_count: i32,
        error_log: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE animal_import_batches SET
                status = $2,
                success_count = $3,
                error_count = $4,
                error_log = $5,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(success_count)
        .bind(error_count)
        .bind(error_log)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 列出匯出記錄
    pub async fn list_export_records(
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<crate::models::AnimalExportRecord>> {
        let records = sqlx::query_as::<_, crate::models::AnimalExportRecord>(
            "SELECT * FROM animal_export_records ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}
