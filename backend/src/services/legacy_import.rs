use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        Animal, AnimalObservation, AnimalSacrifice, AnimalSurgery, AnimalVaccination, AnimalWeight,
        CreateImportBatchRequest, ImportBatchReport, IngestPigRecordsRequest,
        IngestPigRecordsResponse, IngestPigRecordsResultItem, LegacyImportBatch, LegacyImportItem,
        LegacyImportRecord,
        audit_diff::DataDiff,
    },
    services::audit::{ActivityLogEntry, AuditEntity, AuditService},
    AppError, Result,
};

pub struct LegacyImportService;

impl LegacyImportService {
    pub async fn create_batch(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateImportBatchRequest,
    ) -> Result<LegacyImportBatch> {
        let user = actor.require_user()?;
        let batch = sqlx::query_as::<_, LegacyImportBatch>(
            r#"
            INSERT INTO legacy_import_batches (source, description, created_by, created_at)
            VALUES (COALESCE($1, 'ipig.pigmodel.asia'), $2, $3, NOW())
            RETURNING *
            "#,
        )
        .bind(&req.source)
        .bind(&req.description)
        .bind(user.id)
        .fetch_one(pool)
        .await?;

        // Audit (category: legacy_import)
        AuditService::log_activity_oneshot(
            pool,
            actor,
            ActivityLogEntry {
                event_category: "legacy_import",
                event_type: "BATCH_CREATE",
                entity: Some(AuditEntity::new(
                    "legacy_import_batch",
                    batch.id,
                    req.description.as_deref().unwrap_or(""),
                )),
                data_diff: Some(DataDiff::create_only(&batch)),
                request_context: None,
            },
        )
        .await?;

        Ok(batch)
    }

    /// Ingest records for one pig; returns per-record results.
    ///
    /// - Validates pig matching (ear_tag + birth_date + optional iacuc_no).
    /// - Upserts into legacy_import_records by (legacy_source, record_kind, legacy_record_id).
    /// - Marks duplicates against formal tables as skipped_duplicate.
    pub async fn ingest_pig_records(
        pool: &PgPool,
        actor: &ActorContext,
        batch_id: Uuid,
        req: &IngestPigRecordsRequest,
    ) -> Result<IngestPigRecordsResponse> {
        if req.records.len() > 500 {
            return Err(AppError::Validation("每次請求最多 500 筆紀錄".into()));
        }

        let user = actor.require_user()?;

        let animal = Self::match_animal(
            pool,
            &req.pig.ear_tag,
            req.pig.birth_date,
            req.pig.iacuc_no.as_deref(),
        )
        .await?;
        let _animal_id = animal.as_ref().map(|a| a.id);

        // Unmatched → whole request 422 per spec
        if animal.is_none() {
            return Err(AppError::BusinessRule(
                "PIG_NOT_FOUND：耳號/生日/IACUC 無法唯一配對到新系統動物".into(),
            ));
        }
        let animal = animal.unwrap();

        let mut results = Vec::with_capacity(req.records.len());
        for item in &req.records {
            let (status, error_code, message) =
                Self::stage_one_record(pool, &animal, batch_id, user.id, item).await?;
            results.push(IngestPigRecordsResultItem {
                legacy_record_id: item.legacy_record_id.clone(),
                kind: item.kind.clone(),
                status,
                error_code,
                message,
            });
        }
        Ok(IngestPigRecordsResponse { animal_id: Some(animal.id), results })
    }

    async fn match_animal(
        pool: &PgPool,
        ear_tag: &Option<String>,
        birth_date: Option<NaiveDate>,
        iacuc_no: Option<&str>,
    ) -> Result<Option<Animal>> {
        // Require ear_tag and birth_date per spec; if absent return None
        let Some(et) = ear_tag.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) else {
            return Ok(None);
        };
        let Some(bd) = birth_date else {
            return Ok(None);
        };

        let rows = if let Some(iacuc) = iacuc_no {
            sqlx::query_as::<_, Animal>(
                r#"
                SELECT * FROM animals
                WHERE ear_tag = $1 AND birth_date = $2 AND iacuc_no IS NOT DISTINCT FROM $3
                  AND deleted_at IS NULL
                "#,
            )
            .bind(&et)
            .bind(bd)
            .bind(iacuc)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Animal>(
                r#"
                SELECT * FROM animals
                WHERE ear_tag = $1 AND birth_date = $2
                  AND deleted_at IS NULL
                "#,
            )
            .bind(&et)
            .bind(bd)
            .fetch_all(pool)
            .await?
        };
        Ok(match rows.len() {
            0 => None,
            1 => Some(rows.into_iter().next().unwrap()),
            _ => None, // ambiguous: caller will 422 the whole request
        })
    }

    async fn stage_one_record(
        pool: &PgPool,
        animal: &Animal,
        batch_id: Uuid,
        operator_id: Uuid,
        item: &LegacyImportItem,
    ) -> Result<(String, Option<String>, Option<String>)> {
        // Idempotency: (legacy_source, record_kind, legacy_record_id)
        // For now legacy_source fixed
        let legacy_source = "ipig.pigmodel.asia";

        // Determine mapped record_type for observations; '-' becomes observation
        let mapped_type = if item.kind == "observation" {
            match item
                .record_type
                .as_deref()
                .or(item.legacy_record_type_text.as_deref().map(|s| if s.trim() == "-" { "observation" } else { "observation" }))
            {
                Some(rt @ ("abnormal" | "experiment" | "observation")) => Some(rt.to_string()),
                _ => Some("observation".to_string()),
            }
        } else {
            None
        };

        // Duplicate detection against formal tables (simple heuristics per type)
        let is_duplicate = Self::detect_duplicate(pool, animal.id, item).await?;
        let initial_status = if is_duplicate { "skipped_duplicate" } else { "pending" };

        // Resolve recorder mapping: prefer explicit mapped_user_id; else match by display_name
        let (resolved_user_id, resolved_status) = if let Some(uid) = item.mapped_user_id {
            (Some(uid), Some("explicit"))
        } else if let Some(name) = item.original_recorder_name.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let row: Option<(Uuid, bool)> = sqlx::query_as(
                r#"SELECT id, is_active FROM users WHERE display_name = $1 LIMIT 1"#,
            )
            .bind(name)
            .fetch_optional(pool)
            .await?;
            match row {
                Some((id, true)) => (Some(id), Some("matched_active")),
                Some((id, false)) => (Some(id), Some("matched_disabled")),
                None => (None, Some("unmatched")),
            }
        } else {
            (None, Some("unmatched"))
        };

        // Upsert into staging
        let (row_id, status): (Uuid, String) = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            INSERT INTO legacy_import_records (
                legacy_source, record_kind, legacy_record_id, legacy_pig_system_id,
                legacy_record_type_text, original_recorder_name, mapped_user_id,
                legacy_created_at, record_type, status, payload, import_batch_id, created_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NOW())
            ON CONFLICT (legacy_source, record_kind, legacy_record_id)
            DO UPDATE SET
                legacy_pig_system_id = EXCLUDED.legacy_pig_system_id,
                legacy_record_type_text = EXCLUDED.legacy_record_type_text,
                original_recorder_name = EXCLUDED.original_recorder_name,
                mapped_user_id = EXCLUDED.mapped_user_id,
                legacy_created_at = EXCLUDED.legacy_created_at,
                record_type = EXCLUDED.record_type,
                payload = EXCLUDED.payload,
                status = CASE 
                    WHEN legacy_import_records.payload = EXCLUDED.payload THEN legacy_import_records.status
                    ELSE 'conflict'
                END
            RETURNING id, status
        "#,
        )
        .bind(legacy_source)
        .bind(&item.kind)
        .bind(&item.legacy_record_id)
        .bind(None::<i32>)
        .bind(&item.legacy_record_type_text)
        .bind(&item.original_recorder_name)
        .bind(resolved_user_id)
        .bind(item.legacy_created_at)
        .bind(&mapped_type)
        .bind(initial_status)
        .bind(&item.payload)
        .bind(batch_id)
        .fetch_one(pool)
        .await?;

        // Audit enqueue (staging write)
        // Audit (oneshot)
        AuditService::log_activity_oneshot(
            pool,
            &ActorContext::User(crate::middleware::CurrentUser {
                id: operator_id,
                email: String::new(),
                roles: vec![],
                permissions: vec![],
                jti: String::new(),
                exp: 0,
                impersonated_by: None,
            }),
            ActivityLogEntry {
                event_category: "legacy_import",
                event_type: "STAGE_RECORD",
                entity: Some(AuditEntity::new("legacy_import_record", row_id, &item.legacy_record_id)),
                data_diff: Some({
                    let mut dd = DataDiff::empty();
                    dd.after = Some(serde_json::json!({
                        "animal_id": animal.id,
                        "record_kind": item.kind,
                        "legacy_record_id": item.legacy_record_id,
                        "status": status,
                    }));
                    dd
                }),
                request_context: None,
            },
        )
        .await?;

        let result_status = if status == "pending" {
            "created"
        } else if status == "skipped_duplicate" {
            "skipped_duplicate"
        } else if status == "conflict" {
            "conflict"
        } else {
            &status
        };
        let msg = format!(
            "recorder_map={}; mapped_user_id={}",
            resolved_status.unwrap_or("unknown"),
            resolved_user_id
                .map(|u| u.to_string())
                .unwrap_or_else(|| "import_user".to_string())
        );
        Ok((result_status.to_string(), None, Some(msg)))
    }

    async fn detect_duplicate(pool: &PgPool, animal_id: Uuid, item: &LegacyImportItem) -> Result<bool> {
        match item.kind.as_str() {
            "observation" => {
                if let (Some(date), Some(content), Some(rt)) =
                    (item.event_date, item.content.as_deref(), item.record_type.as_deref().or(Some("observation")))
                {
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT id FROM animal_observations
                        WHERE animal_id = $1 AND event_date = $2 AND record_type::text = $3 AND content = $4
                          AND deleted_at IS NULL
                        LIMIT 1
                        "#,
                    )
                    .bind(animal_id)
                    .bind(date)
                    .bind(rt)
                    .bind(content)
                    .fetch_optional(pool)
                    .await?;
                    Ok(exists.is_some())
                } else {
                    Ok(false)
                }
            }
            "weight" => {
                if let (Some(date), Some(weight_val)) =
                    (item.event_date, item.payload.get("weight").and_then(|v| v.as_str()))
                {
                    // weight provided as string in payload; compare text equality after cast
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT id FROM animal_weights
                        WHERE animal_id = $1 AND measure_date = $2 AND weight::text = $3
                        LIMIT 1
                        "#,
                    )
                    .bind(animal_id)
                    .bind(date)
                    .bind(weight_val)
                    .fetch_optional(pool)
                    .await?;
                    Ok(exists.is_some())
                } else {
                    Ok(false)
                }
            }
            "vaccine" => {
                if let Some(date) = item.event_date {
                    let vaccine = item.payload.get("vaccine").and_then(|v| v.as_str()).unwrap_or_default();
                    let deworm = item.payload.get("deworming_dose").and_then(|v| v.as_str()).unwrap_or_default();
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT id FROM animal_vaccinations
                        WHERE animal_id = $1 AND administered_date = $2
                          AND COALESCE(vaccine,'') = $3 AND COALESCE(deworming_dose,'') = $4
                        LIMIT 1
                        "#,
                    )
                    .bind(animal_id)
                    .bind(date)
                    .bind(vaccine)
                    .bind(deworm)
                    .fetch_optional(pool)
                    .await?;
                    Ok(exists.is_some())
                } else {
                    Ok(false)
                }
            }
            "surgery" => {
                if let Some(date) = item.event_date {
                    let site = item.payload.get("surgery_site").and_then(|v| v.as_str()).unwrap_or_default();
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT id FROM animal_surgeries
                        WHERE animal_id = $1 AND surgery_date = $2 AND COALESCE(surgery_site,'') = $3
                          AND deleted_at IS NULL
                        LIMIT 1
                        "#,
                    )
                    .bind(animal_id)
                    .bind(date)
                    .bind(site)
                    .fetch_optional(pool)
                    .await?;
                    Ok(exists.is_some())
                } else {
                    Ok(false)
                }
            }
            "sacrifice" => {
                // unique per animal by schema; detect existence
                let exists: Option<Uuid> = sqlx::query_scalar(
                    "SELECT id FROM animal_sacrifices WHERE animal_id = $1 LIMIT 1",
                )
                .bind(animal_id)
                .fetch_optional(pool)
                .await?;
                Ok(exists.is_some())
            }
            _ => Ok(false),
        }
    }

    /// Approve a batch: write formal records with created_at from legacy time, bypassing handlers.
    /// Returns (approved_count, skipped_count, conflict_count).
    pub async fn approve_batch(
        pool: &PgPool,
        actor: &ActorContext,
        batch_id: Uuid,
        import_user_id: Option<Uuid>,
        dry_run: bool,
    ) -> Result<(i64, i64, i64)> {
        // Permission gate is checked at the caller/CLI; here we just execute.
        let mut tx = pool.begin().await?;

        // Load all pending rows
        let rows: Vec<LegacyImportRecord> = sqlx::query_as(
            r#"SELECT * FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'pending' ORDER BY created_at ASC"#,
        )
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await?;

        let mut approved = 0_i64;
        let skipped = 0_i64;
        let mut conflicts = 0_i64;

        for r in &rows {
            // Extract minimal fields
            let animal_id: Uuid = match r
                .payload
                .get("animal_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            {
                Some(id) => id,
                None => {
                    // Fallback: caller may not include animal_id in payload; attempt to join via animals table by ear_tag/birth_date if provided
                    // For simplicity, require payload.animal_id; otherwise skip with conflict
                    conflicts += 1;
                    continue;
                }
            };
            let created_by = r.mapped_user_id.or(import_user_id);
            let legacy_name = r.original_recorder_name.clone();
            match r.record_kind.as_str() {
                "observation" => {
                    // required: event_date and content
                    let event_date = r
                        .payload
                        .get("event_date")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
                    let content = r.payload.get("content").and_then(|v| v.as_str()).unwrap_or_default();
                    let record_type = r
                        .record_type
                        .as_deref()
                        .unwrap_or("observation");
                    // Compose remark: legacy remark + optional extras without any prefix tags
                    let mut remark_parts: Vec<String> = Vec::new();
                    if let Some(s) = r.payload.get("remark").and_then(|v| v.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        remark_parts.push(s.to_string());
                    }
                    for key in ["vet_advice_text", "care_medication_text", "pain_assessment_text"] {
                        if let Some(s) = r.payload.get(key).and_then(|v| v.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                            remark_parts.push(s.to_string());
                        }
                    }
                    let remark_combined = if remark_parts.is_empty() {
                        None
                    } else {
                        Some(remark_parts.join("\n"))
                    };
                    if let Some(ed) = event_date {
                        if !dry_run {
                            let inserted: AnimalObservation = sqlx::query_as(
                                r#"
                                INSERT INTO animal_observations (
                                    animal_id, event_date, record_type, equipment_used, anesthesia_start,
                                    anesthesia_end, content, no_medication_needed, stop_medication, treatments, remark,
                                    created_by, created_at, updated_at, legacy_recorder_name
                                )
                                VALUES ($1,$2,$3, NULL, NULL, NULL, $4, false, false, NULL, $5,
                                        $6, COALESCE($7, NOW()), NOW(), $8)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(record_type)
                            .bind(content)
                            .bind(remark_combined)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await?;
                            // set target_record_id and status
                            sqlx::query(
                                "UPDATE legacy_import_records SET status = 'approved', target_record_id = $2 WHERE id = $1",
                            )
                            .bind(r.id)
                            .bind(inserted.id)
                            .execute(&mut *tx)
                            .await?;
                        }
                        approved += 1;
                    } else {
                        conflicts += 1;
                    }
                }
                "weight" => {
                    if let (Some(ed), Some(w)) = (
                        r.payload
                            .get("event_date")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
                        r.payload.get("weight").and_then(|v| v.as_str()),
                    ) {
                        if !dry_run {
                            let inserted: AnimalWeight = sqlx::query_as(
                                r#"
                                INSERT INTO animal_weights (animal_id, measure_date, weight, created_by, created_at, legacy_recorder_name)
                                VALUES ($1,$2,$3::decimal, $4, COALESCE($5,NOW()), $6)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(w)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await?;
                            sqlx::query(
                                "UPDATE legacy_import_records SET status='approved', target_record_id = $2 WHERE id = $1",
                            )
                            .bind(r.id)
                            .bind(inserted.id)
                            .execute(&mut *tx)
                            .await?;
                        }
                        approved += 1;
                    } else {
                        conflicts += 1;
                    }
                }
                "vaccine" => {
                    if let Some(ed) = r
                        .payload
                        .get("event_date")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                    {
                        if !dry_run {
                            let inserted: AnimalVaccination = sqlx::query_as(
                                r#"
                                INSERT INTO animal_vaccinations (animal_id, administered_date, vaccine, deworming_dose, created_by, created_at, legacy_recorder_name)
                                VALUES ($1,$2,$3,$4,$5,COALESCE($6,NOW()), $7)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(r.payload.get("vaccine").and_then(|v| v.as_str()))
                            .bind(r.payload.get("deworming_dose").and_then(|v| v.as_str()))
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await?;
                            sqlx::query(
                                "UPDATE legacy_import_records SET status='approved', target_record_id = $2 WHERE id = $1",
                            )
                            .bind(r.id)
                            .bind(inserted.id)
                            .execute(&mut *tx)
                            .await?;
                        }
                        approved += 1;
                    } else {
                        conflicts += 1;
                    }
                }
                "surgery" => {
                    if let Some(ed) = r
                        .payload
                        .get("event_date")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                    {
                        let site = r.payload.get("surgery_site").and_then(|v| v.as_str()).unwrap_or("未記載");
                        // Respiration rate: per owner update 2026-10-31 — NEVER import into numeric column; keep original text in remark
                        // Combine any legacy remark + respiration text
                        let mut remark_parts: Vec<String> = Vec::new();
                        if let Some(s) = r.payload.get("remark").and_then(|v| v.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                            remark_parts.push(s.to_string());
                        }
                        if let Some(rt) = r
                            .payload
                            .get("respiration_text")
                            .or_else(|| r.payload.get("respiration_rate_text"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                        {
                            remark_parts.push(rt.to_string());
                        }
                        let remark_combined = if remark_parts.is_empty() {
                            None
                        } else {
                            Some(remark_parts.join("\n"))
                        };
                        if !dry_run {
                            let inserted: AnimalSurgery = sqlx::query_as(
                                r#"
                                INSERT INTO animal_surgeries (
                                    animal_id, is_first_experiment, surgery_date, surgery_site,
                                    induction_anesthesia, pre_surgery_medication, positioning,
                                    anesthesia_maintenance, anesthesia_observation, vital_signs, reflex_recovery,
                                    respiration_rate, post_surgery_medication, remark,
                                    no_medication_needed, vet_read, vet_read_at,
                                    created_by, created_at, updated_at, legacy_recorder_name
                                )
                                VALUES ($1,false,$2,$3,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,$4,false,false,NULL,
                                        $5, COALESCE($6,NOW()), NOW(), $7)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(site)
                            .bind(remark_combined)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await?;
                            sqlx::query(
                                "UPDATE legacy_import_records SET status='approved', target_record_id = $2 WHERE id = $1",
                            )
                            .bind(r.id)
                            .bind(inserted.id)
                            .execute(&mut *tx)
                            .await?;
                        }
                        approved += 1;
                    } else {
                        conflicts += 1;
                    }
                }
                "sacrifice" => {
                    // animal_sacrifices unique per animal; map minimal fields
                    if !dry_run {
                        let inserted: AnimalSacrifice = sqlx::query_as(
                            r#"
                            INSERT INTO animal_sacrifices (
                                animal_id, sacrifice_date, zoletil_dose,
                                method_electrocution, method_bloodletting, method_other,
                                sampling, sampling_other, blood_volume_ml, confirmed_sacrifice,
                                created_by, created_at, updated_at, legacy_recorder_name
                            )
                            VALUES ($1, NULL, NULL, false, false, NULL, NULL, NULL, NULL, true, $2, COALESCE($3,NOW()), NOW(), $4)
                            RETURNING *
                            "#,
                        )
                        .bind(animal_id)
                        .bind(created_by)
                        .bind(r.legacy_created_at)
                        .bind(legacy_name)
                        .fetch_one(&mut *tx)
                        .await?;
                        sqlx::query(
                            "UPDATE legacy_import_records SET status='approved', target_record_id = $2 WHERE id = $1",
                        )
                        .bind(r.id)
                        .bind(inserted.id)
                        .execute(&mut *tx)
                        .await?;
                    }
                    approved += 1;
                }
                _ => {
                    conflicts += 1;
                }
            }
        }

        if dry_run {
            tx.rollback().await?;
        } else {
            // Audit the approval on commit
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "legacy_import",
                    event_type: "BATCH_APPROVE",
                    entity: Some(AuditEntity::new("legacy_import_batch", batch_id, "")),
                    data_diff: Some({
                        let mut dd = DataDiff::empty();
                        dd.after = Some(serde_json::json!({
                            "approved": approved,
                            "skipped": skipped,
                            "conflict": conflicts,
                        }));
                        dd
                    }),
                    request_context: None,
                },
            )
            .await?;
            tx.commit().await?;
        }

        Ok((approved, skipped, conflicts))
    }

    pub async fn get_batch_report(pool: &PgPool, batch_id: Uuid) -> Result<ImportBatchReport> {
        let (pending,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'pending'")
                .bind(batch_id)
                .fetch_one(pool)
                .await?;
        let (approved,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'approved'")
                .bind(batch_id)
                .fetch_one(pool)
                .await?;
        let (rejected,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'rejected'")
                .bind(batch_id)
                .fetch_one(pool)
                .await?;
        let (skipped_duplicate,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'skipped_duplicate'",
        )
        .bind(batch_id)
        .fetch_one(pool)
        .await?;
        let (conflict,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'conflict'")
                .bind(batch_id)
                .fetch_one(pool)
                .await?;

        let dash_type_records: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM legacy_import_records WHERE import_batch_id = $1 AND legacy_record_type_text = '-'",
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await?;

        Ok(ImportBatchReport {
            batch_id,
            pending,
            approved,
            rejected,
            skipped_duplicate,
            conflict,
            dash_type_records,
        })
    }
}

