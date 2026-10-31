use chrono::NaiveDate;
use chrono::Offset;
use chrono::TimeZone;
use rust_decimal::prelude::FromPrimitive;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Animal, AnimalObservation, AnimalSacrifice, AnimalSurgery,
        AnimalVaccination, AnimalWeight, CreateImportBatchRequest, ImportBatchReport,
        IngestPigRecordsRequest, IngestPigRecordsResponse, IngestPigRecordsResultItem,
        LegacyImportBatch, LegacyImportItem, LegacyImportRecord,
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

        let animal_opt = Self::match_animal(
            pool,
            &req.pig.ear_tag,
            req.pig.birth_date,
            req.pig.iacuc_no.as_deref(),
        )
        .await?;
        let _animal_id = animal_opt.as_ref().map(|a| a.id);
        // Unmatched → whole request 422 per spec
        let animal = match animal_opt {
            Some(a) => a,
            None => {
                return Err(AppError::BusinessRule(
                    "PIG_NOT_FOUND：耳號/生日/IACUC 無法唯一配對到新系統動物".into(),
                ))
            }
        };

        let mut results = Vec::with_capacity(req.records.len());
        // Run whole ingest in one transaction when not dry_run
        let mut maybe_tx: Option<Transaction<'_, Postgres>> = if req.dry_run {
            None
        } else {
            Some(pool.begin().await?)
        };
        for item in &req.records {
            let res = if req.dry_run {
                // Simulate staging without writes
                let duplicate = Self::detect_duplicate(pool, animal.id, item).await?;
                let status = if duplicate {
                    "skipped_duplicate".to_string()
                } else {
                    "pending".to_string()
                };
                // recorder mapping preview: only honor explicit mapped_user_id; no display-name fallback
                let (resolved_user_id, resolved_status) = if let Some(uid) = item.mapped_user_id {
                    // validate existence
                    let exists: Option<bool> =
                        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                            .bind(uid)
                            .fetch_one(pool)
                            .await
                            .ok();
                    if exists == Some(true) {
                        (Some(uid), Some("explicit"))
                    } else {
                        (None, Some("unmatched"))
                    }
                } else {
                    (None, Some("unmatched"))
                };
                // Validate anesthesia time formats for observations
                let mut warnings: Vec<String> = Vec::new();
                if item.kind == "observation" {
                    if let Some(t) = item
                        .payload
                        .get("anesthesia_start_time")
                        .and_then(|v| v.as_str())
                    {
                        let ok = chrono::NaiveTime::parse_from_str(t, "%H:%M:%S")
                            .or_else(|_| chrono::NaiveTime::parse_from_str(t, "%H:%M"))
                            .is_ok();
                        if !ok {
                            warnings
                                .push(format!("WARN_INVALID_TIME anesthesia_start_time='{}'", t));
                        }
                    }
                    if let Some(t) = item
                        .payload
                        .get("anesthesia_end_time")
                        .and_then(|v| v.as_str())
                    {
                        let ok = chrono::NaiveTime::parse_from_str(t, "%H:%M:%S")
                            .or_else(|_| chrono::NaiveTime::parse_from_str(t, "%H:%M"))
                            .is_ok();
                        if !ok {
                            warnings.push(format!("WARN_INVALID_TIME anesthesia_end_time='{}'", t));
                        }
                    }
                }
                let mut msg = format!(
                    "recorder_map={}; mapped_user_id={}",
                    resolved_status.unwrap_or("unknown"),
                    resolved_user_id
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "import_user".to_string())
                );
                if !warnings.is_empty() {
                    msg.push_str(&format!("; {}", warnings.join("; ")));
                }
                (status, None, Some(msg))
            } else {
                if let Some(ref mut tx) = maybe_tx {
                    Self::stage_one_record_tx(tx, pool, &animal, batch_id, user.id, item).await?
                } else {
                    unreachable!()
                }
            };
            results.push(IngestPigRecordsResultItem {
                legacy_record_id: item.legacy_record_id.clone(),
                kind: item.kind.clone(),
                status: res.0,
                error_code: res.1,
                message: res.2,
            });
        }
        if let Some(tx) = maybe_tx {
            tx.commit().await?;
        }
        Ok(IngestPigRecordsResponse {
            animal_id: Some(animal.id),
            results,
        })
    }

    async fn match_animal(
        pool: &PgPool,
        ear_tag: &Option<String>,
        birth_date: Option<NaiveDate>,
        iacuc_no: Option<&str>,
    ) -> Result<Option<Animal>> {
        // Require ear_tag and birth_date per spec; if absent return None
        let Some(et) = ear_tag
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
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
            1 => rows.into_iter().next(),
            _ => None, // ambiguous: caller will 422 the whole request
        })
    }

    async fn stage_one_record_tx(
        tx: &mut Transaction<'_, Postgres>,
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
            match item.record_type.as_deref() {
                Some(rt @ ("abnormal" | "experiment" | "observation")) => Some(rt.to_string()),
                _ => Some("observation".to_string()), // default to observation; '-' already tracked in legacy_record_type_text
            }
        } else {
            None
        };

        // Duplicate detection against formal tables (simple heuristics per type)
        let is_duplicate = Self::detect_duplicate(pool, animal.id, item).await?;
        let initial_status = if is_duplicate {
            "skipped_duplicate"
        } else {
            "pending"
        };

        // Resolve recorder mapping: only honor explicit mapped_user_id; no display-name fallback
        let (resolved_user_id, resolved_status) = if let Some(uid) = item.mapped_user_id {
            (Some(uid), Some("explicit"))
        } else {
            (None, Some("unmatched"))
        };

        // Existing row check for idempotency and conflict (no overwrite of approved rows)
        let existing: Option<(String, serde_json::Value, String)> = sqlx::query_as(
            r#"SELECT status, payload, record_kind FROM legacy_import_records
               WHERE legacy_source = $1 AND record_kind = $2 AND legacy_record_id = $3"#,
        )
        .bind(legacy_source)
        .bind(&item.kind)
        .bind(&item.legacy_record_id)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(existing) = existing {
            let (exist_status, exist_payload, _kind) = existing;
            if exist_status == "approved" && exist_payload != item.payload {
                // Do not overwrite approved rows; report conflict, write nothing
                let msg = format!(
                    "recorder_map={}; mapped_user_id={}",
                    resolved_status.unwrap_or("unknown"),
                    resolved_user_id
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "import_user".to_string())
                );
                return Ok(("conflict".to_string(), None, Some(msg)));
            }
        }

        // Upsert into staging (no overwrite of payload; mark conflict when changed)
        // Validate explicit mapped_user_id exists when provided; else drop
        let resolved_user_id = if let Some(uid) = resolved_user_id {
            let ok: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(uid)
                .fetch_one(&mut **tx)
                .await?;
            if ok {
                Some(uid)
            } else {
                None
            }
        } else {
            None
        };

        let (row_id, status): (Uuid, String) = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            INSERT INTO legacy_import_records (
                legacy_source, record_kind, legacy_record_id, legacy_pig_system_id,
                matched_animal_id, legacy_record_type_text, original_recorder_name, mapped_user_id,
                legacy_created_at, record_type, status, payload, import_batch_id, created_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,NOW())
            ON CONFLICT (legacy_source, record_kind, legacy_record_id)
            DO UPDATE SET
                payload = CASE
                    WHEN legacy_import_records.status = 'pending' THEN EXCLUDED.payload
                    ELSE legacy_import_records.payload
                END,
                import_batch_id = CASE
                    WHEN legacy_import_records.status = 'pending' THEN EXCLUDED.import_batch_id
                    ELSE legacy_import_records.import_batch_id
                END,
                status = CASE
                    WHEN legacy_import_records.status = 'approved' AND legacy_import_records.payload <> EXCLUDED.payload THEN 'conflict'
                    ELSE legacy_import_records.status
                END
            RETURNING id, status
        "#,
        )
        .bind(legacy_source)
        .bind(&item.kind)
        .bind(&item.legacy_record_id)
        .bind(None::<i32>)
        .bind(Some(animal.id))
        .bind(&item.legacy_record_type_text)
        .bind(&item.original_recorder_name)
        .bind(resolved_user_id)
        .bind(item.legacy_created_at)
        .bind(&mapped_type)
        .bind(initial_status)
        .bind(&item.payload)
        .bind(batch_id)
        .fetch_one(&mut **tx)
        .await?;

        // Audit enqueue (staging write)
        // Audit (oneshot)
        AuditService::log_activity_tx(
            tx,
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
                entity: Some(AuditEntity::new(
                    "legacy_import_record",
                    row_id,
                    &item.legacy_record_id,
                )),
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

    async fn detect_duplicate(
        pool: &PgPool,
        animal_id: Uuid,
        item: &LegacyImportItem,
    ) -> Result<bool> {
        match item.kind.as_str() {
            "observation" => {
                if let (Some(date), Some(content), Some(rt)) = (
                    item.event_date,
                    item.content.as_deref(),
                    item.record_type.as_deref().or(Some("observation")),
                ) {
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT id FROM animal_observations
                        WHERE animal_id = $1 AND event_date = $2 AND record_type = $3::record_type AND content = $4
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
                if let (Some(date), Some(weight_val)) = (
                    item.event_date,
                    item.payload.get("weight").and_then(|v| v.as_str()),
                ) {
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
                    let vaccine = item
                        .payload
                        .get("vaccine")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let deworm = item
                        .payload
                        .get("deworming_dose")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
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
                    let site = item
                        .payload
                        .get("surgery_site")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
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
    /// Returns (approved_count, skipped_duplicate_count, error_count).
    pub async fn approve_batch(
        pool: &PgPool,
        actor: &ActorContext,
        batch_id: Uuid,
        import_user_id: Option<Uuid>,
        dry_run: bool,
        recorder_map: Option<&HashMap<String, Uuid>>,
    ) -> Result<(i64, i64, i64)> {
        // Permission gate: require animal.legacy_import.approve or admin
        let actor_user = actor.require_user()?;
        // Consider admin via CurrentUser roles OR database roles membership
        let is_admin_token = actor_user.is_admin();
        let is_admin_db: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
              SELECT 1 FROM user_roles ur
              JOIN roles r ON r.id = ur.role_id
              WHERE ur.user_id = $1 AND r.is_active = true
                AND r.code IN ($2, $3)
            )
            "#,
        )
        .bind(actor_user.id)
        .bind(crate::constants::ROLE_SYSTEM_ADMIN)
        .bind(crate::constants::ROLE_ADMIN_LEGACY)
        .fetch_one(pool)
        .await
        .unwrap_or(false);
        let allowed = if is_admin_token || is_admin_db {
            true
        } else {
            let has_perm: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(
                    SELECT 1 FROM role_permissions rp
                    JOIN user_roles ur ON ur.role_id = rp.role_id
                    JOIN permissions p ON p.id = rp.permission_id
                    WHERE ur.user_id = $1 AND p.code = 'animal.legacy_import.approve'
                )"#,
            )
            .bind(actor_user.id)
            .fetch_one(pool)
            .await?;
            has_perm
        };
        if !allowed {
            return Err(AppError::Forbidden(
                "無權核准匯入（需 animal.legacy_import.approve）".into(),
            ));
        }
        // Batch size limit (safety)
        let (pending_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'pending'",
        )
        .bind(batch_id)
        .fetch_one(pool)
        .await?;
        if pending_count > 5000 {
            return Err(AppError::BusinessRule(
                "批次過大，請分批核准（> 5000）".into(),
            ));
        }
        let mut tx = pool.begin().await?;

        // Load all pending rows
        let rows: Vec<LegacyImportRecord> = sqlx::query_as(
            r#"SELECT * FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'pending' ORDER BY created_at ASC"#,
        )
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await?;

        let mut approved = 0_i64;
        let mut skipped = 0_i64;
        let mut errors = 0_i64;

        // Group rows by animal_id (must be matched by staging)
        let mut by_animal: HashMap<Uuid, Vec<LegacyImportRecord>> = HashMap::new();
        for r in rows {
            if let Some(aid) = r.matched_animal_id {
                by_animal.entry(aid).or_default().push(r);
            } else {
                // unmatched should not happen (ingest enforces), but keep a count as error
                errors += 1;
            }
        }

        // Helper: resolve created_by from recorder_map (by legacy original name) or fallback to import_user_id
        let resolve_created_by = |legacy_name: &Option<String>| -> Option<Uuid> {
            if let Some(name) = legacy_name.as_deref() {
                if let Some(map) = recorder_map {
                    if let Some(uid) = map.get(name) {
                        return Some(*uid);
                    }
                }
            }
            import_user_id
        };

        // Pre-compile pain assessment numeric extractor once per batch
        let pain_re = regex::Regex::new(r"^\s*(\d+)")
            .map_err(|e| AppError::Internal(format!("compile pain regex: {}", e)))?;

        // Process each animal within a SAVEPOINT; on any DB error, roll back this animal only and
        // mark its rows as status='error' (except known skipped_duplicate sacrifice rows).
        for (animal_id, recs) in by_animal {
            // Open savepoint (reused name per-animal is acceptable; redefines previous savepoint)
            sqlx::query("SAVEPOINT li_sp").execute(&mut *tx).await?;

            // Track duplicates we intentionally skip (e.g., second sacrifice)
            let mut duplicate_ids: Vec<Uuid> = Vec::new();
            // Pre-scan sacrifice duplicates so they are counted even if another record for this animal fails later
            for r in &recs {
                if r.record_kind == "sacrifice" {
                    let exists: Option<Uuid> = sqlx::query_scalar(
                        "SELECT id FROM animal_sacrifices WHERE animal_id = $1 LIMIT 1",
                    )
                    .bind(animal_id)
                    .fetch_optional(&mut *tx)
                    .await?;
                    if exists.is_some() && !duplicate_ids.contains(&r.id) {
                        duplicate_ids.push(r.id);
                        skipped += 1;
                    }
                }
            }
            // Track created target ids to set on staging after insert
            // Per record branch handles updates as before; they are all within savepoint.
            let mut animal_failed: Option<(String, String)> = None; // (code, message)

            for r in &recs {
                if animal_failed.is_some() {
                    break;
                }
                // Skip sacrifice rows already marked as duplicates; they will be updated after rollback/release
                if r.record_kind == "sacrifice" && duplicate_ids.contains(&r.id) {
                    continue;
                }
                let created_by = resolve_created_by(&r.original_recorder_name);
                let legacy_name = r.original_recorder_name.clone();
                match r.record_kind.as_str() {
                    "observation" => {
                        // required: event_date and content
                        let event_date = r
                            .payload
                            .get("event_date")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
                        let content = r
                            .payload
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let record_type = r.record_type.as_deref().unwrap_or("observation");
                        // Compose remark: legacy remark + optional extras without any prefix tags
                        let mut remark_parts: Vec<String> = Vec::new();
                        if let Some(s) = r
                            .payload
                            .get("remark")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                        {
                            remark_parts.push(s.to_string());
                        }
                        for key in [
                            "vet_advice_text",
                            "care_medication_text",
                            "pain_assessment_text",
                        ] {
                            if let Some(s) = r
                                .payload
                                .get(key)
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                            {
                                remark_parts.push(s.to_string());
                            }
                        }
                        let remark_combined = if remark_parts.is_empty() {
                            None
                        } else {
                            Some(remark_parts.join("\n"))
                        };
                        if let Some(ed) = event_date {
                            // equipment_used mapping: accept string or array-of-strings
                            let equipment_used_json: Option<serde_json::Value> =
                                if let Some(eq) = r.payload.get("equipment_used") {
                                    if let Some(s) = eq.as_str() {
                                        Some(serde_json::json!([s]))
                                    } else if eq.is_array() {
                                        Some(eq.clone())
                                    } else {
                                        None
                                    }
                                } else if let Some(eq_text) =
                                    r.payload.get("equipment_text").and_then(|v| v.as_str())
                                {
                                    if eq_text.trim() == "-" || eq_text.trim().is_empty() {
                                        None
                                    } else {
                                        Some(serde_json::json!([eq_text.trim()]))
                                    }
                                } else {
                                    None
                                };
                            // anesthesia start/end from HH:MM or HH:MM:SS with event_date in +08:00
                            let tz = chrono::FixedOffset::east_opt(8 * 3600)
                                .unwrap_or_else(|| chrono::Utc.fix());
                            // helper for parsing both formats
                            fn parse_hhmm_or_hhmmss(s: &str) -> Option<chrono::NaiveTime> {
                                chrono::NaiveTime::parse_from_str(s, "%H:%M:%S")
                                    .ok()
                                    .or_else(|| chrono::NaiveTime::parse_from_str(s, "%H:%M").ok())
                            }
                            let (anesthesia_start, start_err) = r
                                .payload
                                .get("anesthesia_start_time")
                                .and_then(|v| v.as_str())
                                .map(|t| match parse_hhmm_or_hhmmss(t) {
                                    Some(nt) => {
                                        let ndt = ed.and_time(nt);
                                        let zdt = tz
                                            .from_local_datetime(&ndt)
                                            .single()
                                            .map(|dt| dt.with_timezone(&chrono::Utc));
                                        (zdt, false)
                                    }
                                    None => (None, true),
                                })
                                .unwrap_or((None, false));
                            let (anesthesia_end, end_err) = r
                                .payload
                                .get("anesthesia_end_time")
                                .and_then(|v| v.as_str())
                                .map(|t| match parse_hhmm_or_hhmmss(t) {
                                    Some(nt) => {
                                        let ndt = ed.and_time(nt);
                                        let zdt = tz
                                            .from_local_datetime(&ndt)
                                            .single()
                                            .map(|dt| dt.with_timezone(&chrono::Utc));
                                        (zdt, false)
                                    }
                                    None => (None, true),
                                })
                                .unwrap_or((None, false));
                            // If invalid time strings present, mark conflict (do not silently drop)
                            if (start_err && r.payload.get("anesthesia_start_time").is_some())
                                || (end_err && r.payload.get("anesthesia_end_time").is_some())
                            {
                                // soft failure (business rule), not a DB error — mark conflict and continue
                                if !dry_run {
                                    if let Err(e) = sqlx::query("UPDATE legacy_import_records SET status = 'conflict' WHERE id = $1")
                                    .bind(r.id)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("mark conflict failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                }
                                }
                                continue;
                            }
                            if !dry_run {
                                // Map old checkbox to no_medication_needed only; stop_medication keeps default
                                let no_medication_needed = r
                                    .payload
                                    .get("no_medication_needed")
                                    .and_then(|v| v.as_bool())
                                    .or_else(|| {
                                        r.payload.get("stop_medication").and_then(|v| v.as_bool())
                                    })
                                    .unwrap_or(false);
                                let inserted_res = sqlx::query_as::<_, AnimalObservation>(
                                r#"
                                INSERT INTO animal_observations (
                                    animal_id, event_date, record_type, equipment_used, anesthesia_start,
                                    anesthesia_end, content, no_medication_needed, stop_medication, treatments, remark,
                                    created_by, created_at, updated_at, legacy_recorder_name
                                )
                                VALUES ($1,$2,$3::record_type, $4, $5, $6, $7, $8, false, NULL, $9,
                                        $10, COALESCE($11, NOW()), NOW(), $12)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(record_type)
                            .bind(&equipment_used_json)
                            .bind(anesthesia_start)
                            .bind(anesthesia_end)
                            .bind(content)
                            .bind(no_medication_needed)
                            .bind(remark_combined)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await;
                                match inserted_res {
                                    Ok(inserted) => {
                                        // Optional: create a care_medication_records pain assessment row if a numeric leading score exists
                                        if let Some(text) = r
                                            .payload
                                            .get("pain_assessment_text")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.trim())
                                            .filter(|s| !s.is_empty())
                                        {
                                            if let Some(cap) = pain_re.captures(text) {
                                                if let Some(m) = cap.get(1) {
                                                    if let Ok(score) = m.as_str().parse::<i16>() {
                                                        let _ = sqlx::query(
                                                            r#"
                                                            INSERT INTO care_medication_records (record_type, record_id, record_mode, pain_score)
                                                            VALUES ($1::vet_record_type, $2, $3::care_record_mode, $4)
                                                            "#,
                                                        )
                                                        .bind("observation")
                                                        .bind(inserted.id)
                                                        .bind("pain_assessment")
                                                        .bind(score)
                                                        .execute(&mut *tx)
                                                        .await;
                                                    }
                                                }
                                            }
                                        }
                                        // set target_record_id and status
                                        if let Err(e) = sqlx::query(
                                        "UPDATE legacy_import_records SET status = 'approved', target_record_id = $2, mapped_user_id = $3 WHERE id = $1",
                                    )
                                    .bind(r.id)
                                    .bind(inserted.id)
                                    .bind(created_by)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("update staging after observation insert failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                        approved += 1;
                                    }
                                    Err(e) => {
                                        let code = e
                                            .as_database_error()
                                            .and_then(|d| d.code())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "DB_ERROR".to_string());
                                        let msg = e.to_string();
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    "weight" => {
                        if let (Some(ed), Some(w)) = (
                            r.payload
                                .get("event_date")
                                .and_then(|v| v.as_str())
                                .and_then(|s| {
                                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                                }),
                            r.payload.get("weight").and_then(|v| v.as_str()),
                        ) {
                            if !dry_run {
                                let inserted_res = sqlx::query_as::<_, AnimalWeight>(
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
                            .await;
                                match inserted_res {
                                    Ok(inserted) => {
                                        if let Err(e) = sqlx::query(
                                        "UPDATE legacy_import_records SET status='approved', target_record_id = $2, mapped_user_id = $3 WHERE id = $1",
                                    )
                                    .bind(r.id)
                                    .bind(inserted.id)
                                    .bind(created_by)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("update staging after weight insert failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                        approved += 1;
                                    }
                                    Err(e) => {
                                        let code = e
                                            .as_database_error()
                                            .and_then(|d| d.code())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "DB_ERROR".to_string());
                                        let msg = e.to_string();
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                }
                            }
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
                                let inserted_res = sqlx::query_as::<_, AnimalVaccination>(
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
                            .await;
                                match inserted_res {
                                    Ok(inserted) => {
                                        if let Err(e) = sqlx::query(
                                        "UPDATE legacy_import_records SET status='approved', target_record_id = $2, mapped_user_id = $3 WHERE id = $1",
                                    )
                                    .bind(r.id)
                                    .bind(inserted.id)
                                    .bind(created_by)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("update staging after vaccine insert failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                        approved += 1;
                                    }
                                    Err(e) => {
                                        let code = e
                                            .as_database_error()
                                            .and_then(|d| d.code())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "DB_ERROR".to_string());
                                        let msg = e.to_string();
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    "surgery" => {
                        if let Some(ed) = r
                            .payload
                            .get("event_date")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                        {
                            let site = r
                                .payload
                                .get("surgery_site")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .unwrap_or("未記載");
                            // Respiration rate: per owner update 2026-10-31 — NEVER import into numeric column; keep original text in remark
                            // Combine any legacy remark + respiration text
                            let mut remark_parts: Vec<String> = Vec::new();
                            if let Some(s) = r
                                .payload
                                .get("remark")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                            {
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
                                // Read JSON blobs directly if provided
                                let induction = r.payload.get("induction_anesthesia").cloned();
                                let premed = r.payload.get("pre_surgery_medication").cloned();
                                let maint = r.payload.get("anesthesia_maintenance").cloned();
                                let vitals = r.payload.get("vital_signs").cloned();
                                // Map additional text fields per design
                                let anesthesia_observation = r
                                    .payload
                                    .get("anesthesia_observation")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty());
                                let reflex_recovery = r
                                    .payload
                                    .get("reflex_recovery")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty());
                                let postmed = r.payload.get("post_surgery_medication").cloned();
                                // positioning: combine list and other text
                                let positioning = r
                                    .payload
                                    .get("positioning_list")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|x| x.as_str())
                                            .map(|s| s.trim())
                                            .filter(|s| !s.is_empty())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .or_else(|| {
                                        r.payload
                                            .get("positioning")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.trim().to_string())
                                    });
                                let positioning = {
                                    let mut base = positioning.unwrap_or_default();
                                    if let Some(other) = r
                                        .payload
                                        .get("positioning_other")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.trim())
                                        .filter(|s| !s.is_empty())
                                    {
                                        if base.is_empty() {
                                            base = other.to_string();
                                        } else {
                                            base = format!("{}, {}", base, other);
                                        }
                                    }
                                    if base.is_empty() {
                                        None
                                    } else {
                                        Some(base)
                                    }
                                };
                                let inserted_res = sqlx::query_as::<_, AnimalSurgery>(
                                r#"
                                INSERT INTO animal_surgeries (
                                    animal_id, is_first_experiment, surgery_date, surgery_site,
                                    induction_anesthesia, pre_surgery_medication, positioning,
                                    anesthesia_maintenance, anesthesia_observation, vital_signs, reflex_recovery,
                                    respiration_rate, post_surgery_medication, remark,
                                    no_medication_needed, vet_read, vet_read_at,
                                    created_by, created_at, updated_at, legacy_recorder_name
                                )
                                VALUES (
                                    $1, false, $2, $3,
                                    $4, $5, $6,
                                    $7, $8, $9, $10,
                                    NULL, $11, $12,
                                    false, false, NULL,
                                    $13, COALESCE($14, NOW()), NOW(), $15
                                )
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(ed)
                            .bind(site)
                            .bind(induction)
                            .bind(premed)
                            .bind(positioning)
                            .bind(maint)
                            .bind(anesthesia_observation)
                            .bind(vitals)
                            .bind(reflex_recovery)
                            .bind(postmed)
                            .bind(remark_combined)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await;
                                match inserted_res {
                                    Ok(inserted) => {
                                        if let Err(e) = sqlx::query(
                                        "UPDATE legacy_import_records SET status='approved', target_record_id = $2, mapped_user_id = $3 WHERE id = $1",
                                    )
                                    .bind(r.id)
                                    .bind(inserted.id)
                                    .bind(created_by)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("update staging after surgery insert failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                        approved += 1;
                                    }
                                    Err(e) => {
                                        let code = e
                                            .as_database_error()
                                            .and_then(|d| d.code())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "DB_ERROR".to_string());
                                        let msg = e.to_string();
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    "sacrifice" => {
                        // animal_sacrifices unique per animal; map fields and skip duplicates at approval time
                        // Skip if sacrifice already exists for this animal (report as skipped_duplicate)
                        let exists: Option<Uuid> = sqlx::query_scalar(
                            "SELECT id FROM animal_sacrifices WHERE animal_id = $1 LIMIT 1",
                        )
                        .bind(animal_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                        if exists.is_some() {
                            duplicate_ids.push(r.id);
                            skipped += 1;
                        } else {
                            // Map fields
                            let sacrifice_date = r
                                .payload
                                .get("event_date")
                                .and_then(|v| v.as_str())
                                .and_then(|s| {
                                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                                });
                            let zoletil_dose = r
                                .payload
                                .get("zoletil_dose")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                            let method_electrocution = r
                                .payload
                                .get("method_electrocution")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let method_bloodletting = r
                                .payload
                                .get("method_bloodletting")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let method_other = r
                                .payload
                                .get("method_other")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                            let sampling = r
                                .payload
                                .get("sampling")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                            let sampling_other = r
                                .payload
                                .get("sampling_other")
                                .and_then(|v| v.as_str())
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                            let blood_volume_ml = r.payload.get("blood_volume_ml").and_then(|v| {
                                if let Some(s) = v.as_str() {
                                    rust_decimal::Decimal::from_str_exact(s).ok()
                                } else if let Some(n) = v.as_f64() {
                                    rust_decimal::Decimal::from_f64(n)
                                } else {
                                    None
                                }
                            });
                            let confirmed_sacrifice = r
                                .payload
                                .get("confirmed_sacrifice")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true);
                            if !dry_run {
                                let inserted_res = sqlx::query_as::<_, AnimalSacrifice>(
                                r#"
                                INSERT INTO animal_sacrifices (
                                    animal_id, sacrifice_date, zoletil_dose,
                                    method_electrocution, method_bloodletting, method_other,
                                    sampling, sampling_other, blood_volume_ml, confirmed_sacrifice,
                                    created_by, created_at, updated_at, legacy_recorder_name
                                )
                                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,COALESCE($12,NOW()),NOW(),$13)
                                RETURNING *
                                "#,
                            )
                            .bind(animal_id)
                            .bind(sacrifice_date)
                            .bind(zoletil_dose)
                            .bind(method_electrocution)
                            .bind(method_bloodletting)
                            .bind(method_other)
                            .bind(sampling)
                            .bind(sampling_other)
                            .bind(blood_volume_ml)
                            .bind(confirmed_sacrifice)
                            .bind(created_by)
                            .bind(r.legacy_created_at)
                            .bind(legacy_name)
                            .fetch_one(&mut *tx)
                            .await;
                                match inserted_res {
                                    Ok(inserted) => {
                                        if let Err(e) = sqlx::query(
                                        "UPDATE legacy_import_records SET status='approved', target_record_id = $2, mapped_user_id = $3 WHERE id = $1",
                                    )
                                    .bind(r.id)
                                    .bind(inserted.id)
                                    .bind(created_by)
                                    .execute(&mut *tx)
                                    .await {
                                        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string()).unwrap_or_else(|| "APP_ERROR".to_string());
                                        let msg = format!("update staging after sacrifice insert failed: {}", e);
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                        approved += 1;
                                    }
                                    Err(e) => {
                                        let code = e
                                            .as_database_error()
                                            .and_then(|d| d.code())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "DB_ERROR".to_string());
                                        let msg = e.to_string();
                                        animal_failed = Some((code, msg));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // unknown kind: treat as application error for this animal
                        animal_failed = Some((
                            "APP_ERROR".to_string(),
                            format!("unknown record_kind: {}", r.record_kind),
                        ));
                    }
                }
            }

            // Check if animal failed; if so rollback to savepoint and mark error statuses for all this animal's rows
            if let Some((code, message)) = animal_failed {
                // Roll back this animal's work
                sqlx::query("ROLLBACK TO SAVEPOINT li_sp")
                    .execute(&mut *tx)
                    .await?;
                // Mark duplicates back to skipped_duplicate (not error)
                if !dry_run && !duplicate_ids.is_empty() {
                    sqlx::query(
                    "UPDATE legacy_import_records SET status = 'skipped_duplicate' WHERE id = ANY($1)",
                )
                .bind(&duplicate_ids)
                .execute(&mut *tx)
                .await?;
                }
                // Mark remaining pending rows for this animal as error with code/message
                if !dry_run {
                    sqlx::query(
                        r#"
                    UPDATE legacy_import_records
                    SET status = 'error', error_code = $3, error_message = $4
                    WHERE import_batch_id = $1
                      AND matched_animal_id = $2
                      AND status = 'pending'
                    "#,
                    )
                    .bind(batch_id)
                    .bind(animal_id)
                    .bind(&code)
                    .bind(&message)
                    .execute(&mut *tx)
                    .await?;
                }
                errors += 1;
                // Continue with next animal
                continue;
            } else {
                // No hard error: apply skipped_duplicate marks (if any) outside of savepoint
                if !dry_run && !duplicate_ids.is_empty() {
                    sqlx::query(
                    "UPDATE legacy_import_records SET status = 'skipped_duplicate' WHERE id = ANY($1)",
                )
                .bind(&duplicate_ids)
                .execute(&mut *tx)
                .await?;
                }
                // RELEASE is optional; we just move on
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
                            "error": errors,
                        }));
                        dd
                    }),
                    request_context: None,
                },
            )
            .await?;
            tx.commit().await?;
        }

        Ok((approved, skipped, errors))
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
        let (conflict,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'conflict'",
        )
        .bind(batch_id)
        .fetch_one(pool)
        .await?;
        let (error_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1 AND status = 'error'",
        )
        .bind(batch_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let dash_type_records: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM legacy_import_records WHERE import_batch_id = $1 AND legacy_record_type_text = '-'",
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await?;
        // Per-row lists
        let skipped_duplicate_rows: Vec<crate::models::LegacyReportRow> = sqlx::query_as(
            r#"SELECT id, legacy_record_id, record_kind, status
               FROM legacy_import_records
               WHERE import_batch_id = $1 AND status = 'skipped_duplicate'
               ORDER BY created_at"#,
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await?;
        let conflict_rows: Vec<crate::models::LegacyReportRow> = sqlx::query_as(
            r#"SELECT id, legacy_record_id, record_kind, status
               FROM legacy_import_records
               WHERE import_batch_id = $1 AND status = 'conflict'
               ORDER BY created_at"#,
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await?;
        let error_rows: Vec<crate::models::LegacyReportRow> = sqlx::query_as(
            r#"SELECT id, legacy_record_id, record_kind, status
               FROM legacy_import_records
               WHERE import_batch_id = $1 AND status = 'error'
               ORDER BY created_at"#,
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|_| Vec::new());
        let unmatched_recorder_rows: Vec<crate::models::LegacyReportRow> = sqlx::query_as(
            r#"SELECT id, legacy_record_id, record_kind, status
               FROM legacy_import_records
               WHERE import_batch_id = $1 AND original_recorder_name IS NOT NULL AND mapped_user_id IS NULL
               ORDER BY created_at"#,
        )
        .bind(batch_id)
        .fetch_all(pool)
        .await?;
        let disabled_account_rows: Vec<crate::models::LegacyReportRow> = sqlx::query_as(
            r#"SELECT r.id, r.legacy_record_id, r.record_kind, r.status
               FROM legacy_import_records r
               JOIN users u ON r.mapped_user_id = u.id
               WHERE r.import_batch_id = $1 AND u.is_active = false
               ORDER BY r.created_at"#,
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
            error: error_count,
            dash_type_records,
            skipped_duplicate_rows,
            conflict_rows,
            unmatched_recorder_rows,
            disabled_account_rows,
            error_rows,
        })
    }
}
