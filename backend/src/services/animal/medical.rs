use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        Animal, AnimalExportRecord, AnimalImportBatch, AnimalSacrifice, AnimalStatus,
        AnimalSuddenDeath, AnimalVaccination, CreateSacrificeRequest, CreateSuddenDeathRequest,
        CreateVaccinationRequest, CreateVetRecommendationRequest,
        CreateVetRecommendationWithAttachmentsRequest, ExportFormat, ExportType, ImportStatus,
        ImportType, UpdateVaccinationRequest, VersionDiff, VersionHistoryResponse,
        VetRecommendation, VetRecordType,
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

    pub async fn create_vaccination(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateVaccinationRequest,
        created_by: Uuid,
    ) -> Result<AnimalVaccination> {
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
        .fetch_one(pool)
        .await?;

        Ok(vaccination)
    }

    /// 更新疫苗紀錄
    pub async fn update_vaccination(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateVaccinationRequest,
    ) -> Result<AnimalVaccination> {
        let vaccination = sqlx::query_as::<_, AnimalVaccination>(
            r#"
            UPDATE animal_vaccinations SET
                administered_date = COALESCE($2, administered_date),
                vaccine = COALESCE($3, vaccine),
                deworming_dose = COALESCE($4, deworming_dose)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.administered_date)
        .bind(&req.vaccine)
        .bind(&req.deworming_dose)
        .fetch_one(pool)
        .await?;

        Ok(vaccination)
    }

    /// 刪除疫苗紀錄
    pub async fn soft_delete_vaccination(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM animal_vaccinations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除疫苗紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_vaccination_with_reason(
        pool: &PgPool,
        id: Uuid,
        reason: &str,
        deleted_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('vaccination', $1::text, 'DELETE', $2, $3)
            "#,
        )
        .bind(id.to_string())
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        sqlx::query(
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
        .execute(pool)
        .await?;

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

    pub async fn upsert_sacrifice(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateSacrificeRequest,
        created_by: Uuid,
    ) -> Result<AnimalSacrifice> {
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
        .fetch_one(pool)
        .await?;

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

    /// 登記猝死
    pub async fn create_sudden_death(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateSuddenDeathRequest,
        created_by: Uuid,
    ) -> Result<AnimalSuddenDeath> {
        // 驗證動物狀態可轉換到 SuddenDeath
        let animal = AnimalService::get_by_id(pool, animal_id).await?;
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
        .fetch_one(pool)
        .await?;

        // 自動更新動物狀態為 sudden_death
        sqlx::query("UPDATE animals SET status = 'sudden_death', updated_at = NOW() WHERE id = $1")
            .bind(animal_id)
            .execute(pool)
            .await?;

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

    /// 新增獸醫師建議
    pub async fn add_vet_recommendation(
        pool: &PgPool,
        record_type: VetRecordType,
        record_id: Uuid,
        req: &CreateVetRecommendationRequest,
        created_by: Uuid,
    ) -> Result<VetRecommendation> {
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
        .fetch_one(pool)
        .await?;

        Ok(recommendation)
    }

    /// 新增獸醫師建議（含附件）
    pub async fn add_vet_recommendation_with_attachments(
        pool: &PgPool,
        record_type: VetRecordType,
        record_id: Uuid,
        req: &CreateVetRecommendationWithAttachmentsRequest,
        created_by: Uuid,
    ) -> Result<VetRecommendation> {
        // 驗證 attachments JSONB 結構
        if let Some(ref att) = req.attachments {
            crate::utils::jsonb_validation::validate_attachments(att)?;
        }

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
        .fetch_one(pool)
        .await?;

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

    /// 建立或更新病理報告
    pub async fn upsert_pathology_report(
        pool: &PgPool,
        animal_id: Uuid,
        created_by: Uuid,
    ) -> Result<crate::models::AnimalPathologyReport> {
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
        .fetch_one(pool)
        .await?;

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
