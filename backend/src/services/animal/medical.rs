use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        CreateSacrificeRequest, CreateVaccinationRequest, CreateVetRecommendationRequest,
        CreateVetRecommendationWithAttachmentsRequest, Pig, PigExportRecord, PigImportBatch,
        PigSacrifice, PigVaccination, RecordVersion, UpdateVaccinationRequest, VersionDiff,
        VersionHistoryResponse, VetRecommendation, VetRecordType,
        ExportType, ExportFormat, ImportType, ImportStatus,
    }, Result,
};

impl AnimalService {

    // ============================================
    // 疫苗/驅蟲紀錄
    // ============================================

    /// 取得疫苗紀錄列表（排除已刪除）
    pub async fn list_vaccinations(pool: &PgPool, pig_id: Uuid) -> Result<Vec<PigVaccination>> {
        let vaccinations = sqlx::query_as::<_, PigVaccination>(
            "SELECT * FROM pig_vaccinations WHERE pig_id = $1 ORDER BY administered_date DESC"
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(vaccinations)
    }

    pub async fn create_vaccination(
        pool: &PgPool,
        pig_id: Uuid,
        req: &CreateVaccinationRequest,
        created_by: Uuid,
    ) -> Result<PigVaccination> {
        let vaccination = sqlx::query_as::<_, PigVaccination>(
            r#"
            INSERT INTO pig_vaccinations (pig_id, administered_date, vaccine, deworming_dose, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
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
    ) -> Result<PigVaccination> {
        let vaccination = sqlx::query_as::<_, PigVaccination>(
            r#"
            UPDATE pig_vaccinations SET
                administered_date = COALESCE($2, administered_date),
                vaccine = COALESCE($3, vaccine),
                deworming_dose = COALESCE($4, deworming_dose)
            WHERE id = $1
            RETURNING *
            "#
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
        sqlx::query("DELETE FROM pig_vaccinations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 軟刪除疫苗紀錄（含刪除原因）- GLP 合規
    pub async fn soft_delete_vaccination_with_reason(pool: &PgPool, id: Uuid, reason: &str, deleted_by: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO change_reasons (entity_type, entity_id, change_type, reason, changed_by)
            VALUES ('vaccination', $1::text, 'DELETE', $2, $3)
            "#
        )
        .bind(id)
        .bind(reason)
        .bind(deleted_by)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE pig_vaccinations SET 
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

    // ============================================
    // 犧牲/採樣紀錄
    // ============================================

    pub async fn get_sacrifice(pool: &PgPool, pig_id: Uuid) -> Result<Option<PigSacrifice>> {
        let sacrifice = sqlx::query_as::<_, PigSacrifice>(
            "SELECT * FROM pig_sacrifices WHERE pig_id = $1"
        )
        .bind(pig_id)
        .fetch_optional(pool)
        .await?;

        Ok(sacrifice)
    }

    pub async fn upsert_sacrifice(
        pool: &PgPool,
        pig_id: Uuid,
        req: &CreateSacrificeRequest,
        created_by: Uuid,
    ) -> Result<PigSacrifice> {
        let sacrifice = sqlx::query_as::<_, PigSacrifice>(
            r#"
            INSERT INTO pig_sacrifices (
                pig_id, sacrifice_date, zoletil_dose, method_electrocution,
                method_bloodletting, method_other, sampling, sampling_other,
                blood_volume_ml, confirmed_sacrifice, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            ON CONFLICT (pig_id) DO UPDATE SET
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
            "#
        )
        .bind(pig_id)
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

        // Note: The database trigger will automatically handle pen_location removal and status update
        // 注意：資料庫觸發器會自動處理欄位編號移除和狀態更新
        // The trigger is set up in migration 023_add_pig_soft_delete_and_sacrifice_pen_removal.sql
        // 觸發器在遷移 023_add_pig_soft_delete_and_sacrifice_pen_removal.sql 中設置

        Ok(sacrifice)
    }

    // ============================================
    // 獸醫師功能
    // ============================================

    /// 標記獸醫師已讀
    pub async fn mark_vet_read(pool: &PgPool, pig_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE pigs SET
                vet_last_viewed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(pig_id)
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
        .bind(&record_type)
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
        let recommendation = sqlx::query_as::<_, VetRecommendation>(
            r#"
            INSERT INTO vet_recommendations (record_type, record_id, content, attachments, is_urgent, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#
        )
        .bind(&record_type)
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
        .bind(&record_type)
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
        // 取得當前最大版本號
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version_no) FROM record_versions WHERE record_type = $1 AND record_id = $2"
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
            VALUES ($1, $2, $3, $4, $5, NOW())
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

    /// 取得紀錄版本歷史
    pub async fn get_record_versions(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
    ) -> Result<VersionHistoryResponse> {
        let versions = sqlx::query_as::<_, RecordVersion>(
            "SELECT * FROM record_versions WHERE record_type = $1 AND record_id = $2 ORDER BY version_no DESC"
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        let version_diffs: Vec<VersionDiff> = versions
            .into_iter()
            .map(|v| VersionDiff {
                version_no: v.version_no,
                changed_at: v.changed_at,
                changed_by: v.changed_by,
                diff_summary: v.diff_summary,
                snapshot: v.snapshot,
            })
            .collect();

        Ok(VersionHistoryResponse {
            record_type: record_type.to_string(),
            record_id,
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
    ) -> Result<PigImportBatch> {
        let batch = sqlx::query_as::<_, PigImportBatch>(
            r#"
            INSERT INTO pig_import_batches (id, import_type, file_name, total_rows, status, created_by, created_at)
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
    ) -> Result<PigImportBatch> {
        let status = if error_count == 0 {
            ImportStatus::Completed
        } else {
            ImportStatus::Completed // 部分成功仍標記為完成
        };

        let batch = sqlx::query_as::<_, PigImportBatch>(
            r#"
            UPDATE pig_import_batches SET
                success_count = $2,
                error_count = $3,
                error_details = $4,
                status = $5,
                completed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
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
    pub async fn list_import_batches(pool: &PgPool, limit: i32) -> Result<Vec<PigImportBatch>> {
        let batches = sqlx::query_as::<_, PigImportBatch>(
            "SELECT * FROM pig_import_batches ORDER BY created_at DESC LIMIT $1"
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
        pig_id: Option<Uuid>,
        iacuc_no: Option<&str>,
        export_type: ExportType,
        export_format: ExportFormat,
        file_path: Option<&str>,
        created_by: Uuid,
    ) -> Result<PigExportRecord> {
        let record = sqlx::query_as::<_, PigExportRecord>(
            r#"
            INSERT INTO pig_export_records (id, pig_id, iacuc_no, export_type, export_format, file_path, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(pig_id)
        .bind(iacuc_no)
        .bind(export_type)
        .bind(export_format)
        .bind(file_path)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 取得豬隻完整病歷資料（用於匯出）
    pub async fn get_pig_medical_data(pool: &PgPool, pig_id: Uuid) -> Result<serde_json::Value> {
        let pig = Self::get_by_id(pool, pig_id).await?;
        let observations = Self::list_observations(pool, pig_id).await?;
        let surgeries = Self::list_surgeries(pool, pig_id).await?;
        let weights = Self::list_weights(pool, pig_id).await?;
        let vaccinations = Self::list_vaccinations(pool, pig_id).await?;
        let sacrifice = Self::get_sacrifice(pool, pig_id).await?;

        let data = serde_json::json!({
            "pig": pig,
            "observations": observations,
            "surgeries": surgeries,
            "weights": weights,
            "vaccinations": vaccinations,
            "sacrifice": sacrifice,
        });

        Ok(data)
    }

    /// 取得計劃下所有豬隻病歷資料（用於匯出）
    pub async fn get_project_medical_data(pool: &PgPool, iacuc_no: &str) -> Result<serde_json::Value> {
        let pigs = sqlx::query_as::<_, Pig>(
            "SELECT * FROM pigs WHERE iacuc_no = $1 AND deleted_at IS NULL ORDER BY id"
        )
        .bind(iacuc_no)
        .fetch_all(pool)
        .await?;

        let mut pig_data = Vec::new();
        for pig in pigs {
            let data = Self::get_pig_medical_data(pool, pig.id).await?;
            pig_data.push(data);
        }

        Ok(serde_json::json!({
            "iacuc_no": iacuc_no,
            "pigs": pig_data,
        }))
    }

    // ============================================
    // 病理報告功能
    // ============================================

    /// 取得病理報告
    pub async fn get_pathology_report(pool: &PgPool, pig_id: Uuid) -> Result<Option<crate::models::PigPathologyReport>> {
        let report = sqlx::query_as::<_, crate::models::PigPathologyReport>(
            "SELECT * FROM pig_pathology_reports WHERE pig_id = $1"
        )
        .bind(pig_id)
        .fetch_optional(pool)
        .await?;

        Ok(report)
    }

    /// 建立或更新病理報告
    pub async fn upsert_pathology_report(
        pool: &PgPool,
        pig_id: Uuid,
        created_by: Uuid,
    ) -> Result<crate::models::PigPathologyReport> {
        let report = sqlx::query_as::<_, crate::models::PigPathologyReport>(
            r#"
            INSERT INTO pig_pathology_reports (pig_id, created_by, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (pig_id) DO UPDATE SET updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(pig_id)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(report)
    }
}
