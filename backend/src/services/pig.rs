use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{
    models::{
        BatchAssignRequest, BatchStartExperimentRequest, CreateObservationRequest, CreatePigRequest,
        CreatePigSourceRequest, CreateSacrificeRequest, CreateSurgeryRequest,
        CreateVaccinationRequest, CreateVetRecommendationRequest, CreateWeightRequest, Pig,
        PigListItem, PigObservation, PigQuery, PigSacrifice, PigSource, PigStatus, PigSurgery,
        PigVaccination, PigWeight, PigsByPen, UpdatePigRequest, UpdatePigSourceRequest,
        VetRecommendation, UpdateObservationRequest, UpdateSurgeryRequest, UpdateWeightRequest,
        UpdateVaccinationRequest, RecordVersion, VersionDiff, VersionHistoryResponse,
        PigImportBatch, ImportStatus, ImportType, ImportResult, ImportErrorDetail,
        PigExportRecord, ExportType, ExportFormat, CreateVetRecommendationWithAttachmentsRequest,
        ObservationListItem, SurgeryListItem, PigImportRow, WeightImportRow, PigBreed, PigGender,
    },
    AppError, Result,
};
use calamine::{Reader, Xlsx, Xls, open_workbook_from_rs, Data};
use std::io::Cursor;

pub struct PigService;

impl PigService {
    // ============================================
    // 豬隻來源管理
    // ============================================

    /// 取得豬隻來源列表
    pub async fn list_sources(pool: &PgPool) -> Result<Vec<PigSource>> {
        let sources = sqlx::query_as::<_, PigSource>(
            "SELECT * FROM pig_sources WHERE is_active = true ORDER BY sort_order"
        )
        .fetch_all(pool)
        .await?;

        Ok(sources)
    }

    /// 建立豬隻來源
    pub async fn create_source(pool: &PgPool, req: &CreatePigSourceRequest) -> Result<PigSource> {
        let source = sqlx::query_as::<_, PigSource>(
            r#"
            INSERT INTO pig_sources (id, code, name, address, contact, phone, is_active, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, 0, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.contact)
        .bind(&req.phone)
        .fetch_one(pool)
        .await?;

        Ok(source)
    }

    /// 更新豬隻來源
    pub async fn update_source(pool: &PgPool, id: Uuid, req: &UpdatePigSourceRequest) -> Result<PigSource> {
        let source = sqlx::query_as::<_, PigSource>(
            r#"
            UPDATE pig_sources SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                contact = COALESCE($4, contact),
                phone = COALESCE($5, phone),
                is_active = COALESCE($6, is_active),
                sort_order = COALESCE($7, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.contact)
        .bind(&req.phone)
        .bind(req.is_active)
        .bind(req.sort_order)
        .fetch_one(pool)
        .await?;

        Ok(source)
    }

    /// 刪除（停用）豬隻來源
    pub async fn delete_source(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE pig_sources SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ============================================
    // 豬隻管理
    // ============================================

    /// 取得豬隻列表
    pub async fn list(pool: &PgPool, query: &PigQuery) -> Result<Vec<PigListItem>> {
        // Build query with proper parameterized queries
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                p.id, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at,
                -- Computed fields for frontend
                EXISTS(
                    SELECT 1 FROM pig_observations po 
                    WHERE po.pig_id = p.id 
                    AND po.record_type = 'abnormal'
                    AND po.deleted_at IS NULL
                ) as has_abnormal_record,
                EXISTS(
                    SELECT 1 FROM pig_observations po 
                    WHERE po.pig_id = p.id 
                    AND po.no_medication_needed = false
                    AND po.event_date >= CURRENT_DATE - INTERVAL '30 days'
                    AND po.deleted_at IS NULL
                ) as is_on_medication,
                (
                    SELECT MAX(vr.created_at) 
                    FROM vet_recommendations vr
                    WHERE (vr.record_type = 'observation' AND vr.record_id IN (
                        SELECT id FROM pig_observations WHERE pig_id = p.id AND deleted_at IS NULL
                    ))
                    OR (vr.record_type = 'surgery' AND vr.record_id IN (
                        SELECT id FROM pig_surgeries WHERE pig_id = p.id AND deleted_at IS NULL
                    ))
                ) as vet_recommendation_date
            FROM pigs p
            LEFT JOIN pig_sources s ON p.source_id = s.id
            WHERE 1=1
            "#
        );

        // Add filters with proper parameterization
        if let Some(status) = &query.status {
            query_builder.push(" AND p.status = ");
            query_builder.push_bind(status);
        }
        if let Some(breed) = &query.breed {
            // 轉換 breed enum 為資料庫期望的字串值
            let breed_str = match breed {
                crate::models::PigBreed::Minipig => "miniature",
                crate::models::PigBreed::White => "white",
                crate::models::PigBreed::Other => "other",
            };
            query_builder.push(" AND p.breed = ");
            query_builder.push_bind(breed_str);
        }
        if let Some(iacuc_no) = &query.iacuc_no {
            query_builder.push(" AND p.iacuc_no = ");
            query_builder.push_bind(iacuc_no);
        }
        if let Some(keyword) = &query.keyword {
            let keyword_pattern = format!("%{}%", keyword);
            query_builder.push(" AND (p.ear_tag ILIKE ");
            query_builder.push_bind(keyword_pattern.clone());
            query_builder.push(" OR p.pen_location ILIKE ");
            query_builder.push_bind(keyword_pattern);
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY p.id DESC");

        let pigs = query_builder
            .build_query_as::<PigListItem>()
            .fetch_all(pool)
            .await?;

        Ok(pigs)
    }

    /// 依欄位分組取得豬隻
    pub async fn list_by_pen(pool: &PgPool) -> Result<Vec<PigsByPen>> {
        let pigs = sqlx::query_as::<_, PigListItem>(
            r#"
            SELECT 
                p.id, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at
            FROM pigs p
            LEFT JOIN pig_sources s ON p.source_id = s.id
            WHERE p.pen_location IS NOT NULL
            ORDER BY p.pen_location, p.id
            "#
        )
        .fetch_all(pool)
        .await?;

        // 依欄位分組
        let mut grouped: std::collections::HashMap<String, Vec<PigListItem>> = std::collections::HashMap::new();
        for pig in pigs {
            if let Some(pen) = &pig.pen_location {
                grouped.entry(pen.clone()).or_default().push(pig);
            }
        }

        let result: Vec<PigsByPen> = grouped
            .into_iter()
            .map(|(pen, pigs)| PigsByPen {
                pen_location: pen,
                pigs,
            })
            .collect();

        Ok(result)
    }

    /// 取得單一豬隻
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Pig> {
        let pig = sqlx::query_as::<_, Pig>("SELECT * FROM pigs WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Pig not found".to_string()))?;

        Ok(pig)
    }

    /// 建立豬隻
    pub async fn create(pool: &PgPool, req: &CreatePigRequest, created_by: Uuid) -> Result<Pig> {
        // 格式化耳號：如果是數字則補零至三位數
        let formatted_ear_tag = if let Ok(num) = req.ear_tag.parse::<u32>() {
            format!("{:03}", num)
        } else {
            req.ear_tag.clone()
        };

        // 將 breed enum 轉換為資料庫期望的字串值
        let breed_str = match req.breed {
            crate::models::PigBreed::Minipig => "miniature",
            crate::models::PigBreed::White => "white",
            crate::models::PigBreed::Other => "other",
        };
        
        let pig = sqlx::query_as::<_, Pig>(
            r#"
            INSERT INTO pigs (
                ear_tag, status, breed, breed_other, source_id, gender, birth_date,
                entry_date, entry_weight, pen_location, pre_experiment_code,
                remark, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3::pig_breed, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(&formatted_ear_tag)
        .bind(PigStatus::Unassigned)
        .bind(breed_str)
        .bind(&req.breed_other)
        .bind(req.source_id)
        .bind(req.gender)
        .bind(req.birth_date)
        .bind(req.entry_date)
        .bind(req.entry_weight)
        .bind(&req.pen_location)
        .bind(&req.pre_experiment_code)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(pig)
    }

    /// 更新豬隻
    pub async fn update(pool: &PgPool, id: i32, req: &UpdatePigRequest) -> Result<Pig> {
        // 以下欄位於建立後不可更改，不會在更新時修改：
        // - ear_tag (耳號)
        // - breed (品種)
        // - gender (性別)
        // - source_id (來源)
        // - birth_date (出生日期)
        // - entry_date (進場日期)
        // - entry_weight (進場體重)
        // - pre_experiment_code (實驗前代號)
        
        let pig = sqlx::query_as::<_, Pig>(
            r#"
            UPDATE pigs SET
                status = COALESCE($2, status),
                pen_location = COALESCE($3, pen_location),
                iacuc_no = COALESCE($4, iacuc_no),
                experiment_date = COALESCE($5, experiment_date),
                remark = COALESCE($6, remark),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.status)
        .bind(&req.pen_location)
        .bind(&req.iacuc_no)
        .bind(req.experiment_date)
        .bind(&req.remark)
        .fetch_one(pool)
        .await?;

        Ok(pig)
    }

    /// 刪除豬隻
    pub async fn delete(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM pigs WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 批次分配豬隻至計劃
    pub async fn batch_assign(pool: &PgPool, req: &BatchAssignRequest) -> Result<Vec<Pig>> {
        let mut updated_pigs = Vec::new();

        for pig_id in &req.pig_ids {
            let pig = sqlx::query_as::<_, Pig>(
                r#"
                UPDATE pigs SET
                    iacuc_no = $2,
                    status = $3,
                    updated_at = NOW()
                WHERE id = $1 AND status = $4
                RETURNING *
                "#
            )
            .bind(pig_id)
            .bind(&req.iacuc_no)
            .bind(PigStatus::Assigned)
            .bind(PigStatus::Unassigned)
            .fetch_optional(pool)
            .await?;

            if let Some(p) = pig {
                updated_pigs.push(p);
            }
        }

        Ok(updated_pigs)
    }

    /// 批次進入實驗
    pub async fn batch_start_experiment(pool: &PgPool, req: &BatchStartExperimentRequest) -> Result<Vec<Pig>> {
        let mut updated_pigs = Vec::new();

        for pig_id in &req.pig_ids {
            let pig = sqlx::query_as::<_, Pig>(
                r#"
                UPDATE pigs SET
                    status = $2,
                    experiment_date = CURRENT_DATE,
                    updated_at = NOW()
                WHERE id = $1 AND status = $3
                RETURNING *
                "#
            )
            .bind(pig_id)
            .bind(PigStatus::InExperiment)
            .bind(PigStatus::Assigned)
            .fetch_optional(pool)
            .await?;

            if let Some(p) = pig {
                updated_pigs.push(p);
            }
        }

        Ok(updated_pigs)
    }

    // ============================================
    // 觀察試驗紀錄
    // ============================================

    /// 取得觀察紀錄列表（排除已刪除）
    pub async fn list_observations(pool: &PgPool, pig_id: i32) -> Result<Vec<PigObservation>> {
        let observations = sqlx::query_as::<_, PigObservation>(
            "SELECT * FROM pig_observations WHERE pig_id = $1 AND deleted_at IS NULL ORDER BY event_date DESC"
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得觀察紀錄列表（含獸醫師建議數量）
    pub async fn list_observations_with_recommendations(pool: &PgPool, pig_id: i32) -> Result<Vec<ObservationListItem>> {
        let observations = sqlx::query_as::<_, ObservationListItem>(
            r#"
            SELECT 
                o.id, o.pig_id, o.event_date, o.record_type, o.content,
                o.no_medication_needed, o.vet_read, o.vet_read_at,
                o.created_by, o.created_at,
                (SELECT COUNT(*) FROM vet_recommendations vr WHERE vr.record_type = 'observation' AND vr.record_id = o.id) as recommendation_count
            FROM pig_observations o
            WHERE o.pig_id = $1 AND o.deleted_at IS NULL
            ORDER BY o.event_date DESC
            "#
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(observations)
    }

    /// 取得單一觀察紀錄
    pub async fn get_observation_by_id(pool: &PgPool, id: i32) -> Result<PigObservation> {
        let observation = sqlx::query_as::<_, PigObservation>(
            "SELECT * FROM pig_observations WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Observation not found".to_string()))?;

        Ok(observation)
    }

    pub async fn create_observation(
        pool: &PgPool,
        pig_id: i32,
        req: &CreateObservationRequest,
        created_by: Uuid,
    ) -> Result<PigObservation> {
        let observation = sqlx::query_as::<_, PigObservation>(
            r#"
            INSERT INTO pig_observations (
                pig_id, event_date, record_type, equipment_used, anesthesia_start,
                anesthesia_end, content, no_medication_needed, treatments, remark,
                created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
        .bind(req.event_date)
        .bind(req.record_type)
        .bind(&req.equipment_used)
        .bind(req.anesthesia_start)
        .bind(req.anesthesia_end)
        .bind(&req.content)
        .bind(req.no_medication_needed)
        .bind(&req.treatments)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(observation)
    }

    /// 更新觀察紀錄
    pub async fn update_observation(
        pool: &PgPool,
        id: i32,
        req: &UpdateObservationRequest,
        updated_by: Uuid,
    ) -> Result<PigObservation> {
        // 先取得原始紀錄用於版本歷史
        let original = Self::get_observation_by_id(pool, id).await?;
        
        // 保存版本歷史
        Self::save_record_version(pool, "observation", id, &original, updated_by).await?;

        let observation = sqlx::query_as::<_, PigObservation>(
            r#"
            UPDATE pig_observations SET
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

    /// 軟刪除觀察紀錄
    pub async fn soft_delete_observation(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE pig_observations SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 複製觀察紀錄
    pub async fn copy_observation(
        pool: &PgPool,
        pig_id: i32,
        source_id: i32,
        created_by: Uuid,
    ) -> Result<PigObservation> {
        let source = Self::get_observation_by_id(pool, source_id).await?;

        let observation = sqlx::query_as::<_, PigObservation>(
            r#"
            INSERT INTO pig_observations (
                pig_id, event_date, record_type, equipment_used, anesthesia_start,
                anesthesia_end, content, no_medication_needed, treatments, remark,
                copied_from_id, created_by, created_at, updated_at
            )
            VALUES ($1, CURRENT_DATE, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
        .bind(source.record_type)
        .bind(&source.equipment_used)
        .bind(source.anesthesia_start)
        .bind(source.anesthesia_end)
        .bind(&source.content)
        .bind(source.no_medication_needed)
        .bind(&source.treatments)
        .bind(&source.remark)
        .bind(source_id)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(observation)
    }

    /// 標記觀察紀錄獸醫師已讀
    pub async fn mark_observation_vet_read(pool: &PgPool, id: i32, vet_user_id: Uuid) -> Result<()> {
        // 更新紀錄本身
        sqlx::query(
            "UPDATE pig_observations SET vet_read = true, vet_read_at = NOW(), updated_at = NOW() WHERE id = $1"
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

    // ============================================
    // 手術紀錄
    // ============================================

    /// 取得手術紀錄列表（排除已刪除）
    pub async fn list_surgeries(pool: &PgPool, pig_id: i32) -> Result<Vec<PigSurgery>> {
        let surgeries = sqlx::query_as::<_, PigSurgery>(
            "SELECT * FROM pig_surgeries WHERE pig_id = $1 AND deleted_at IS NULL ORDER BY surgery_date DESC"
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(surgeries)
    }

    /// 取得手術紀錄列表（含獸醫師建議數量）
    pub async fn list_surgeries_with_recommendations(pool: &PgPool, pig_id: i32) -> Result<Vec<SurgeryListItem>> {
        let surgeries = sqlx::query_as::<_, SurgeryListItem>(
            r#"
            SELECT 
                s.id, s.pig_id, s.is_first_experiment, s.surgery_date, s.surgery_site,
                s.no_medication_needed, s.vet_read, s.vet_read_at,
                s.created_by, s.created_at,
                (SELECT COUNT(*) FROM vet_recommendations vr WHERE vr.record_type = 'surgery' AND vr.record_id = s.id) as recommendation_count
            FROM pig_surgeries s
            WHERE s.pig_id = $1 AND s.deleted_at IS NULL
            ORDER BY s.surgery_date DESC
            "#
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(surgeries)
    }

    /// 取得單一手術紀錄
    pub async fn get_surgery_by_id(pool: &PgPool, id: i32) -> Result<PigSurgery> {
        let surgery = sqlx::query_as::<_, PigSurgery>(
            "SELECT * FROM pig_surgeries WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Surgery not found".to_string()))?;

        Ok(surgery)
    }

    pub async fn create_surgery(
        pool: &PgPool,
        pig_id: i32,
        req: &CreateSurgeryRequest,
        created_by: Uuid,
    ) -> Result<PigSurgery> {
        let surgery = sqlx::query_as::<_, PigSurgery>(
            r#"
            INSERT INTO pig_surgeries (
                pig_id, is_first_experiment, surgery_date, surgery_site,
                induction_anesthesia, pre_surgery_medication, positioning,
                anesthesia_maintenance, anesthesia_observation, vital_signs,
                reflex_recovery, respiration_rate, post_surgery_medication,
                remark, no_medication_needed, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
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
        .fetch_one(pool)
        .await?;

        Ok(surgery)
    }

    /// 更新手術紀錄
    pub async fn update_surgery(
        pool: &PgPool,
        id: i32,
        req: &UpdateSurgeryRequest,
        updated_by: Uuid,
    ) -> Result<PigSurgery> {
        // 先取得原始紀錄用於版本歷史
        let original = Self::get_surgery_by_id(pool, id).await?;
        
        // 保存版本歷史
        Self::save_record_version(pool, "surgery", id, &original, updated_by).await?;

        let surgery = sqlx::query_as::<_, PigSurgery>(
            r#"
            UPDATE pig_surgeries SET
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
            "#
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

    /// 軟刪除手術紀錄
    pub async fn soft_delete_surgery(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE pig_surgeries SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 複製手術紀錄
    pub async fn copy_surgery(
        pool: &PgPool,
        pig_id: i32,
        source_id: i32,
        created_by: Uuid,
    ) -> Result<PigSurgery> {
        let source = Self::get_surgery_by_id(pool, source_id).await?;

        let surgery = sqlx::query_as::<_, PigSurgery>(
            r#"
            INSERT INTO pig_surgeries (
                pig_id, is_first_experiment, surgery_date, surgery_site,
                induction_anesthesia, pre_surgery_medication, positioning,
                anesthesia_maintenance, anesthesia_observation, vital_signs,
                reflex_recovery, respiration_rate, post_surgery_medication,
                remark, no_medication_needed, copied_from_id, created_by, created_at, updated_at
            )
            VALUES ($1, false, CURRENT_DATE, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
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
        .bind(source_id)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(surgery)
    }

    /// 標記手術紀錄獸醫師已讀
    pub async fn mark_surgery_vet_read(pool: &PgPool, id: i32, vet_user_id: Uuid) -> Result<()> {
        // 更新紀錄本身
        sqlx::query(
            "UPDATE pig_surgeries SET vet_read = true, vet_read_at = NOW(), updated_at = NOW() WHERE id = $1"
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
            "#
        )
        .bind(id)
        .bind(vet_user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // ============================================
    // 體重紀錄
    // ============================================

    /// 取得體重紀錄列表（排除已刪除）
    pub async fn list_weights(pool: &PgPool, pig_id: i32) -> Result<Vec<PigWeight>> {
        let weights = sqlx::query_as::<_, PigWeight>(
            "SELECT * FROM pig_weights WHERE pig_id = $1 AND deleted_at IS NULL ORDER BY measure_date DESC"
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(weights)
    }

    /// 取得最新體重
    pub async fn get_latest_weight(pool: &PgPool, pig_id: i32) -> Result<Option<PigWeight>> {
        let weight = sqlx::query_as::<_, PigWeight>(
            "SELECT * FROM pig_weights WHERE pig_id = $1 AND deleted_at IS NULL ORDER BY measure_date DESC LIMIT 1"
        )
        .bind(pig_id)
        .fetch_optional(pool)
        .await?;

        Ok(weight)
    }

    pub async fn create_weight(
        pool: &PgPool,
        pig_id: i32,
        req: &CreateWeightRequest,
        created_by: Uuid,
    ) -> Result<PigWeight> {
        let weight = sqlx::query_as::<_, PigWeight>(
            r#"
            INSERT INTO pig_weights (pig_id, measure_date, weight, created_by, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#
        )
        .bind(pig_id)
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
        id: i32,
        req: &UpdateWeightRequest,
    ) -> Result<PigWeight> {
        let weight = sqlx::query_as::<_, PigWeight>(
            r#"
            UPDATE pig_weights SET
                measure_date = COALESCE($2, measure_date),
                weight = COALESCE($3, weight)
            WHERE id = $1 AND deleted_at IS NULL
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

    /// 軟刪除體重紀錄
    pub async fn soft_delete_weight(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("UPDATE pig_weights SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ============================================
    // 疫苗/驅蟲紀錄
    // ============================================

    /// 取得疫苗紀錄列表（排除已刪除）
    pub async fn list_vaccinations(pool: &PgPool, pig_id: i32) -> Result<Vec<PigVaccination>> {
        let vaccinations = sqlx::query_as::<_, PigVaccination>(
            "SELECT * FROM pig_vaccinations WHERE pig_id = $1 AND deleted_at IS NULL ORDER BY administered_date DESC"
        )
        .bind(pig_id)
        .fetch_all(pool)
        .await?;

        Ok(vaccinations)
    }

    pub async fn create_vaccination(
        pool: &PgPool,
        pig_id: i32,
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
        id: i32,
        req: &UpdateVaccinationRequest,
    ) -> Result<PigVaccination> {
        let vaccination = sqlx::query_as::<_, PigVaccination>(
            r#"
            UPDATE pig_vaccinations SET
                administered_date = COALESCE($2, administered_date),
                vaccine = COALESCE($3, vaccine),
                deworming_dose = COALESCE($4, deworming_dose)
            WHERE id = $1 AND deleted_at IS NULL
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

    /// 軟刪除疫苗紀錄
    pub async fn soft_delete_vaccination(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("UPDATE pig_vaccinations SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ============================================
    // 犧牲/採樣紀錄
    // ============================================

    pub async fn get_sacrifice(pool: &PgPool, pig_id: i32) -> Result<Option<PigSacrifice>> {
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
        pig_id: i32,
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

        // 如果確定犧牲，更新豬隻狀態
        if req.confirmed_sacrifice {
            sqlx::query("UPDATE pigs SET status = $2, updated_at = NOW() WHERE id = $1")
                .bind(pig_id)
                .bind(PigStatus::Completed)
                .execute(pool)
                .await?;
        }

        Ok(sacrifice)
    }

    // ============================================
    // 獸醫師功能
    // ============================================

    /// 標記獸醫師已讀
    pub async fn mark_vet_read(pool: &PgPool, pig_id: i32) -> Result<()> {
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
        record_type: &str,
        record_id: i32,
        req: &CreateVetRecommendationRequest,
        created_by: Uuid,
    ) -> Result<VetRecommendation> {
        let recommendation = sqlx::query_as::<_, VetRecommendation>(
            r#"
            INSERT INTO vet_recommendations (record_type, record_id, content, created_by, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#
        )
        .bind(record_type)
        .bind(record_id)
        .bind(&req.content)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(recommendation)
    }

    /// 新增獸醫師建議（含附件）
    pub async fn add_vet_recommendation_with_attachments(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
        req: &CreateVetRecommendationWithAttachmentsRequest,
        created_by: Uuid,
    ) -> Result<VetRecommendation> {
        let recommendation = sqlx::query_as::<_, VetRecommendation>(
            r#"
            INSERT INTO vet_recommendations (record_type, record_id, content, attachments, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#
        )
        .bind(record_type)
        .bind(record_id)
        .bind(&req.content)
        .bind(&req.attachments)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(recommendation)
    }

    /// 取得紀錄的獸醫師建議
    pub async fn get_vet_recommendations(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
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
    async fn save_record_version<T: serde::Serialize>(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
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
        record_id: i32,
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
        pig_id: Option<i32>,
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
    pub async fn get_pig_medical_data(pool: &PgPool, pig_id: i32) -> Result<serde_json::Value> {
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
            "SELECT * FROM pigs WHERE iacuc_no = $1 ORDER BY id"
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
    pub async fn get_pathology_report(pool: &PgPool, pig_id: i32) -> Result<Option<crate::models::PigPathologyReport>> {
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
        pig_id: i32,
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

    // ============================================
    // 模板生成功能
    // ============================================

    /// 生成豬隻基本資料匯入模板
    pub fn generate_basic_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        
        // 創建格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new()
            .set_font_color("#808080")
            .set_italic();

        // 創建工作表
        let worksheet = workbook.add_worksheet();
        
        // 設置欄位寬度
        worksheet.set_column_width(0, 15.0)?; // 耳號
        worksheet.set_column_width(1, 12.0)?; // 品種
        worksheet.set_column_width(2, 10.0)?; // 性別
        worksheet.set_column_width(3, 15.0)?; // 出生日期
        worksheet.set_column_width(4, 15.0)?; // 進場日期
        worksheet.set_column_width(5, 12.0)?; // 進場體重
        worksheet.set_column_width(6, 12.0)?; // 欄位編號
        worksheet.set_column_width(7, 15.0)?; // 實驗前代號
        worksheet.set_column_width(8, 12.0)?; // 備註

        // 寫入標題行
        worksheet.write_string_with_format(0, 0, "耳號*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "品種*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "性別*", &header_format)?;
        worksheet.write_string_with_format(0, 3, "出生日期", &header_format)?;
        worksheet.write_string_with_format(0, 4, "進場日期*", &header_format)?;
        worksheet.write_string_with_format(0, 5, "進場體重", &header_format)?;
        worksheet.write_string_with_format(0, 6, "欄位編號", &header_format)?;
        worksheet.write_string_with_format(0, 7, "實驗前代號", &header_format)?;
        worksheet.write_string_with_format(0, 8, "備註", &header_format)?;

        // 寫入說明行
        worksheet.write_string_with_format(1, 0, "001", &example_format)?;
        worksheet.write_string(1, 1, "miniature")?;
        worksheet.write_string(1, 2, "male")?;
        worksheet.write_string_with_format(1, 3, "2024-01-15", &example_format)?;
        worksheet.write_string_with_format(1, 4, "2024-02-01", &example_format)?;
        worksheet.write_string(1, 5, "25.5")?;
        worksheet.write_string(1, 6, "A-01")?;
        worksheet.write_string(1, 7, "PRE-001")?;
        worksheet.write_string(1, 8, "")?;

        // 凍結第一行
        worksheet.set_freeze_panes(1, 0)?;

        // 將工作簿轉換為字節數組
        let buffer = workbook.save_to_buffer()?;

        Ok(buffer)
    }

    /// 生成豬隻體重匯入模板
    pub fn generate_weight_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        
        // 創建格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new()
            .set_font_color("#808080")
            .set_italic();

        // 創建工作表
        let worksheet = workbook.add_worksheet();
        
        // 設置欄位寬度
        worksheet.set_column_width(0, 15.0)?; // 耳號
        worksheet.set_column_width(1, 15.0)?; // 測量日期
        worksheet.set_column_width(2, 12.0)?; // 體重

        // 寫入標題行
        worksheet.write_string_with_format(0, 0, "耳號*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "測量日期*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "體重(kg)*", &header_format)?;

        // 寫入說明行
        worksheet.write_string_with_format(1, 0, "001", &example_format)?;
        worksheet.write_string_with_format(1, 1, "2024-02-01", &example_format)?;
        worksheet.write_string(1, 2, "25.5")?;

        // 凍結第一行
        worksheet.set_freeze_panes(1, 0)?;

        // 將工作簿轉換為字節數組
        let buffer = workbook.save_to_buffer()?;

        Ok(buffer)
    }

    /// 生成豬隻基本資料匯入 CSV 模板
    pub fn generate_basic_import_template_csv() -> Result<Vec<u8>> {
        // 優先讀取專案根目錄下的 file imput.csv
        // 在 Docker 或 生產環境中可能需要調整路徑，此處先嘗試相對路徑
        let paths = ["file imput.csv", "../file imput.csv", "./backend/file imput.csv"];
        for path in paths {
            if let Ok(data) = std::fs::read(path) {
                return Ok(data);
            }
        }

        // 如果找不到檔案，則生成預設模板
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        // 寫入標題行
        wtr.write_record(&[
            "耳號*",
            "品種*",
            "性別*",
            "出生日期",
            "進場日期*",
            "進場體重",
            "欄位編號",
            "實驗前代號",
            "備註",
        ]).map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;
        
        // 寫入範例行
        wtr.write_record(&[
            "001",
            "miniature",
            "male",
            "2024-01-15",
            "2024-02-01",
            "25.5",
            "A-01",
            "PRE-001",
            "",
        ]).map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;
        
        wtr.flush().map_err(|e| AppError::Internal(format!("CSV flush 失敗: {}", e)))?;
        wtr.into_inner().map_err(|e| {
            AppError::Internal(format!("CSV 生成失敗: {}", e))
        })
    }

    /// 生成豬隻體重匯入 CSV 模板
    pub fn generate_weight_import_template_csv() -> Result<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        // 寫入標題行
        wtr.write_record(&[
            "耳號*",
            "測量日期*",
            "體重(kg)*",
        ]).map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;
        
        // 寫入範例行
        wtr.write_record(&[
            "001",
            "2024-02-01",
            "25.5",
        ]).map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;
        
        wtr.flush().map_err(|e| AppError::Internal(format!("CSV flush 失敗: {}", e)))?;
        wtr.into_inner().map_err(|e| {
            AppError::Internal(format!("CSV 生成失敗: {}", e))
        })
    }

    // ============================================
    // 檔案匯入處理
    // ============================================

    /// 匯入豬隻基本資料
    pub async fn import_basic_data(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        created_by: Uuid,
    ) -> Result<ImportResult> {
        // 根據檔案副檔名判斷格式
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        // 解析檔案
        let rows = if is_excel {
            Self::parse_excel_file(file_data)?
        } else {
            Self::parse_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        // 建立匯入批次記錄
        let batch = Self::create_import_batch(
            pool,
            ImportType::PigBasic,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        // 處理每一行資料
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();
        let mut seen_ear_tags = std::collections::HashSet::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32; // +2 因為第一行是標題，索引從 0 開始

            // 驗證必填欄位
            if row.ear_tag.is_empty() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: None,
                    error: "耳號為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 檢查檔案內重複
            if !seen_ear_tags.insert(row.ear_tag.clone()) {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(row.ear_tag.clone()),
                    error: "耳號在檔案中重複".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 驗證品種
            let breed = match row.breed.to_lowercase().as_str() {
                "minipig" | "miniature" | "mini" | "m" | "迷你豬" | "迷你" => PigBreed::Minipig,
                "white" | "w" | "白豬" | "大白豬" => PigBreed::White,
                _ => PigBreed::Other,
            };

            // 驗證性別
            let gender = match row.gender.to_lowercase().as_str() {
                "male" | "m" | "公" | "公豬" => PigGender::Male,
                "female" | "f" | "母" | "母豬" => PigGender::Female,
                _ => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("無效的性別值: {}，必須是 male/female 或 m/f (或公/母)", row.gender),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 驗證進場日期
            let entry_date_str = row.entry_date.replace("/", "-");
            let entry_date = match chrono::NaiveDate::parse_from_str(&entry_date_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("無效的進場日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", row.entry_date),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 解析出生日期（選填）
            let birth_date = if let Some(ref bd) = row.birth_date {
                if bd.is_empty() {
                    None
                } else {
                    let bd_str = bd.replace("/", "-");
                    match chrono::NaiveDate::parse_from_str(&bd_str, "%Y-%m-%d") {
                        Ok(date) => Some(date),
                        Err(_) => {
                            errors.push(ImportErrorDetail {
                                row: row_number,
                                ear_tag: Some(row.ear_tag.clone()),
                                error: format!("無效的出生日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", bd),
                            });
                            error_count += 1;
                            continue;
                        }
                    }
                }
            } else {
                None
            };

            // 解析進場體重（選填）
            let entry_weight = row.entry_weight.as_ref().and_then(|w| {
                w.replace("kg", "").replace("公斤", "").trim().parse::<f64>().ok()
            }).map(|w| {
                rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default()
            });

            // 查找來源 ID（如果有提供 source_code）
            let source_id = if let Some(ref code) = row.source_code {
                if !code.is_empty() {
                    let source = sqlx::query_as::<_, PigSource>(
                        "SELECT * FROM pig_sources WHERE code = $1 AND is_active = true"
                    )
                    .bind(code)
                    .fetch_optional(pool)
                    .await?;
                    source.map(|s| s.id)
                } else {
                    None
                }
            } else {
                None
            };

            // 格式化耳號
            let formatted_ear_tag = if let Ok(num) = row.ear_tag.parse::<u32>() {
                format!("{:03}", num)
            } else {
                row.ear_tag.clone()
            };

            // 檢查耳號是否已存在
            let existing = sqlx::query_scalar::<_, i32>(
                "SELECT id FROM pigs WHERE ear_tag = $1"
            )
            .bind(&formatted_ear_tag)
            .fetch_optional(pool)
            .await?;

            if existing.is_some() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(formatted_ear_tag),
                    error: "耳號已存在於系統中".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 處理欄位位置
            let pen_location = row.pen_location.clone().or_else(|| {
                match (&row.field_region, &row.field_number) {
                    (Some(r), Some(n)) => Some(format!("{}{}", r, n)),
                    (Some(r), None) => Some(r.clone()),
                    (None, Some(n)) => Some(n.clone()),
                    (None, None) => None,
                }
            }).map(|s| s.trim().to_string());

            // 建立豬隻資料
            let create_req = CreatePigRequest {
                ear_tag: formatted_ear_tag.clone(),
                breed,
                source_id,
                gender,
                birth_date,
                entry_date,
                entry_weight,
                pen_location,
                pre_experiment_code: row.pre_experiment_code.clone(),
                remark: row.remark.clone(),
            };

            match Self::create(pool, &create_req, created_by).await {
                Ok(pig) => {
                    // 如果有計畫編號，更新它
                    if let Some(ref iacuc) = row.iacuc_no {
                        if !iacuc.is_empty() {
                            let _ = Self::update(pool, pig.id, &UpdatePigRequest {
                                iacuc_no: Some(iacuc.clone()),
                                ..Default::default()
                            }).await;
                        }
                    }
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        // 更新批次結果
        let error_details = if errors.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null))
        };

        let _batch = Self::update_import_batch_result(
            pool,
            batch.id,
            success_count,
            error_count,
            error_details,
        )
        .await?;

        Ok(ImportResult {
            batch_id: batch.id,
            total_rows: rows.len() as i32,
            success_count,
            error_count,
            errors,
        })
    }

    /// 匯入體重資料
    pub async fn import_weight_data(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        created_by: Uuid,
    ) -> Result<ImportResult> {
        // 根據檔案副檔名判斷格式
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        // 解析檔案
        let rows = if is_excel {
            Self::parse_weight_excel_file(file_data)?
        } else {
            Self::parse_weight_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        // 建立匯入批次記錄
        let batch = Self::create_import_batch(
            pool,
            ImportType::PigWeight,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        // 處理每一行資料
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            // 驗證必填欄位
            if row.ear_tag.is_empty() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: None,
                    error: "耳號為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 驗證測量日期
            let measure_date_str = row.measure_date.replace("/", "-");
            let measure_date = match chrono::NaiveDate::parse_from_str(&measure_date_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("無效的測量日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", row.measure_date),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 驗證體重
            let weight_val = row.weight.replace("kg", "").replace("公斤", "").trim().parse::<f64>().unwrap_or(0.0);
            if weight_val <= 0.0 {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(row.ear_tag.clone()),
                    error: format!("無效的體重值: {}，必須是大於 0 的數字", row.weight),
                });
                error_count += 1;
                continue;
            }

            // 格式化耳號
            let formatted_ear_tag = if let Ok(num) = row.ear_tag.parse::<u32>() {
                format!("{:03}", num)
            } else {
                row.ear_tag.clone()
            };

            // 查找豬隻
            let pig = sqlx::query_scalar::<_, i32>(
                "SELECT id FROM pigs WHERE ear_tag = $1"
            )
            .bind(&formatted_ear_tag)
            .fetch_optional(pool)
            .await?;

            let pig_id = match pig {
                Some(id) => id,
                None => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: "找不到對應的豬隻".to_string(),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 建立體重紀錄
            let weight_decimal = rust_decimal::Decimal::from_f64_retain(weight_val)
                .unwrap_or_default();

            let create_req = CreateWeightRequest {
                measure_date,
                weight: weight_decimal,
            };

            match Self::create_weight(pool, pig_id, &create_req, created_by).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        // 更新批次結果
        let error_details = if errors.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null))
        };

        let _batch = Self::update_import_batch_result(
            pool,
            batch.id,
            success_count,
            error_count,
            error_details,
        )
        .await?;

        Ok(ImportResult {
            batch_id: batch.id,
            total_rows: rows.len() as i32,
            success_count,
            error_count,
            errors,
        })
    }

    /// 解析 Excel 檔案（基本資料）
    fn parse_excel_file(file_data: &[u8]) -> Result<Vec<PigImportRow>> {
        let range = {
            let cursor = Cursor::new(file_data);
            if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                let sheet_name = wb.sheet_names().first().cloned()
                    .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            } else {
                let cursor = Cursor::new(file_data);
                let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor)
                    .map_err(|e| AppError::Validation(format!("無法讀取 Excel 檔案，請確認檔案格式為 .xlsx 或 .xls")))?;
                let sheet_name = wb.sheet_names().first().cloned()
                    .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            }
        };

        let mut rows = Vec::new();
        let mut iter = range.rows();

        // 跳過標題行
        iter.next();

        for (row_idx, row) in iter.enumerate() {
            if row.len() < 4 {
                continue; // 跳過資料不足的行
            }

            let ear_tag = Self::get_cell_string(row.get(0));
            let breed = Self::get_cell_string(row.get(1));
            let gender = Self::get_cell_string(row.get(2));
            let birth_date = {
                let s = Self::get_cell_string(row.get(3));
                if s.is_empty() { None } else { Some(s) }
            };
            let entry_date = Self::get_cell_string(row.get(4));
            let entry_weight = row.get(5)
                .and_then(|c| match c {
                    Data::Float(f) => Some(*f),
                    Data::Int(i) => Some(*i as f64),
                    _ => None,
                });
            let pen_location = {
                let s = Self::get_cell_string(row.get(6));
                if s.is_empty() { None } else { Some(s) }
            };
            let pre_experiment_code = {
                let s = Self::get_cell_string(row.get(7));
                if s.is_empty() { None } else { Some(s) }
            };
            let remark = {
                let s = Self::get_cell_string(row.get(8));
                if s.is_empty() { None } else { Some(s) }
            };

            // 如果必填欄位為空，跳過這行
            if ear_tag.is_empty() || breed.is_empty() || gender.is_empty() || entry_date.is_empty() {
                continue;
            }

            rows.push(PigImportRow {
                ear_tag,
                breed,
                gender,
                source_code: None, // Excel 範本中沒有這個欄位
                birth_date,
                entry_date,
                entry_weight: entry_weight.map(|w| w.to_string()),
                pen_location,
                pre_experiment_code,
                remark,
                iacuc_no: None,
                field_region: None,
                field_number: None,
            });
        }

        Ok(rows)
    }

    /// 解析 CSV 檔案（基本資料）
    fn parse_csv_file(file_data: &[u8]) -> Result<Vec<PigImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let mut rows = Vec::new();

        let mut row_idx = 1;
        for result in reader.deserialize::<PigImportRow>() {
            row_idx += 1;
            match result {
                Ok(row) => {
                    if !row.ear_tag.is_empty() {
                        rows.push(row);
                    }
                }
                Err(e) => {
                    return Err(AppError::Validation(format!("基本資料 CSV 第 {} 行解析錯誤: {}", row_idx, e)));
                }
            }
        }

        Ok(rows)
    }

    /// 解析 Excel 檔案（體重資料）
    fn parse_weight_excel_file(file_data: &[u8]) -> Result<Vec<WeightImportRow>> {
        let range = {
            let cursor = Cursor::new(file_data);
            if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                let sheet_name = wb.sheet_names().first().cloned()
                    .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            } else {
                let cursor = Cursor::new(file_data);
                let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor)
                    .map_err(|e| AppError::Validation(format!("無法讀取 Excel 檔案，請確認檔案格式為 .xlsx 或 .xls")))?;
                let sheet_name = wb.sheet_names().first().cloned()
                    .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            }
        };

        let mut rows = Vec::new();
        let mut iter = range.rows();

        // 跳過標題行
        iter.next();

        for row in iter {
            if row.len() < 3 {
                continue;
            }

            let ear_tag = Self::get_cell_string(row.get(0)).trim().to_string();
            let measure_date = Self::get_cell_string(row.get(1)).trim().to_string();
            let weight = row.get(2)
                .and_then(|c| match c {
                    Data::Float(f) => Some(*f),
                    Data::Int(i) => Some(*i as f64),
                    _ => None,
                })
                .unwrap_or(0.0);

            if ear_tag.is_empty() || measure_date.is_empty() || weight <= 0.0 {
                continue;
            }

            rows.push(WeightImportRow {
                ear_tag,
                measure_date,
                weight: weight.to_string(),
            });
        }

        Ok(rows)
    }

    /// 解析 CSV 檔案（體重資料）
    fn parse_weight_csv_file(file_data: &[u8]) -> Result<Vec<WeightImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let mut rows = Vec::new();

        let mut row_idx = 1;
        for result in reader.deserialize::<WeightImportRow>() {
            row_idx += 1;
            match result {
                Ok(row) => {
                    if !row.ear_tag.is_empty() {
                        rows.push(row);
                    }
                }
                Err(e) => {
                    return Err(AppError::Validation(format!("體重資料 CSV 第 {} 行解析錯誤: {}", row_idx, e)));
                }
            }
        }

        Ok(rows)
    }

    /// 從 Excel 儲存格取得字串值
    fn get_cell_string(cell: Option<&Data>) -> String {
        if let Some(c) = cell {
            match c {
                Data::String(s) => s.trim().to_string(),
                Data::Int(i) => i.to_string(),
                Data::Float(f) => f.to_string(),
                Data::Bool(b) => b.to_string(),
                Data::DateTime(dt) => {
                    let f = dt.as_f64();
                    let days = f as i64;
                    // Excel 基準日期 1899-12-30
                    if let Some(base_date) = chrono::NaiveDate::from_ymd_opt(1899, 12, 30) {
                        if let Some(date) = base_date.checked_add_signed(chrono::Duration::days(days)) {
                            return date.format("%Y-%m-%d").to_string();
                        }
                    }
                    f.to_string()
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }
}
