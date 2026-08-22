// Service 層：GLP 合規模組業務邏輯
// R63-A: 所有 mutation 走 TX + audit logging；approve 加電子簽章

use crate::error::AppError;
use crate::middleware::ActorContext;
use crate::models::audit_diff::DataDiff;
use crate::models::glp_compliance::*;
use crate::repositories::glp_compliance as repo;
use crate::services::audit::{ActivityLogEntry, AuditEntity};
use crate::services::{AuditService, SignatureService, SignatureType};
use crate::Result;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

pub struct GlpComplianceService;

impl GlpComplianceService {
    // ========================================================================
    // Reference Standards
    // ========================================================================

    pub async fn list_reference_standards(pool: &PgPool) -> Result<Vec<ReferenceStandard>> {
        repo::find_reference_standards(pool).await
    }

    pub async fn get_reference_standard(pool: &PgPool, id: Uuid) -> Result<ReferenceStandard> {
        repo::find_reference_standard_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("參考標準器不存在".into()))
    }

    pub async fn create_reference_standard(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateReferenceStandardRequest,
    ) -> Result<ReferenceStandard> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, ReferenceStandard>(
            r#"
            INSERT INTO reference_standards
                (name, serial_number, standard_type, traceable_to, national_standard_number,
                 calibration_lab, calibration_lab_accreditation, last_calibrated_at, next_due_at,
                 certificate_number, measurement_uncertainty, notes, created_by)
            VALUES ($1,$2,COALESCE($3,'working'),$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *
        "#,
        )
        .bind(&req.name)
        .bind(&req.serial_number)
        .bind(&req.standard_type)
        .bind(&req.traceable_to)
        .bind(&req.national_standard_number)
        .bind(&req.calibration_lab)
        .bind(&req.calibration_lab_accreditation)
        .bind(req.last_calibrated_at)
        .bind(req.next_due_at)
        .bind(&req.certificate_number)
        .bind(&req.measurement_uncertainty)
        .bind(&req.notes)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "reference_standard",
                    after.id,
                    &after.name,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_reference_standard(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateReferenceStandardRequest,
    ) -> Result<ReferenceStandard> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ReferenceStandard>(
            "SELECT * FROM reference_standards WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("參考標準器不存在".into()))?;

        let after = sqlx::query_as::<_, ReferenceStandard>(
            r#"
            UPDATE reference_standards SET
                name = COALESCE($2, name),
                serial_number = COALESCE($3, serial_number),
                standard_type = COALESCE($4, standard_type),
                traceable_to = COALESCE($5, traceable_to),
                national_standard_number = COALESCE($6, national_standard_number),
                calibration_lab = COALESCE($7, calibration_lab),
                calibration_lab_accreditation = COALESCE($8, calibration_lab_accreditation),
                last_calibrated_at = COALESCE($9, last_calibrated_at),
                next_due_at = COALESCE($10, next_due_at),
                certificate_number = COALESCE($11, certificate_number),
                measurement_uncertainty = COALESCE($12, measurement_uncertainty),
                status = COALESCE($13, status),
                notes = COALESCE($14, notes),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.serial_number)
        .bind(&req.standard_type)
        .bind(&req.traceable_to)
        .bind(&req.national_standard_number)
        .bind(&req.calibration_lab)
        .bind(&req.calibration_lab_accreditation)
        .bind(req.last_calibrated_at)
        .bind(req.next_due_at)
        .bind(&req.certificate_number)
        .bind(&req.measurement_uncertainty)
        .bind(&req.status)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new("reference_standard", id, &after.name)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    // ========================================================================
    // Controlled Documents
    // ========================================================================

    pub async fn list_controlled_documents(
        pool: &PgPool,
        params: &ControlledDocumentQuery,
    ) -> Result<Vec<ControlledDocumentWithOwner>> {
        repo::find_controlled_documents(pool, params).await
    }

    pub async fn get_controlled_document(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<ControlledDocumentWithOwner> {
        repo::find_controlled_document_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))
    }

    pub async fn create_controlled_document(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateControlledDocumentRequest,
    ) -> Result<ControlledDocument> {
        let user = actor.require_user()?;
        let number = Self::generate_doc_number(pool, &req.doc_type).await?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, ControlledDocument>(r#"
            INSERT INTO controlled_documents
                (doc_number, title, doc_type, category, effective_date, review_due_date, retention_years, file_path, owner_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *
        "#)
        .bind(&number)
        .bind(&req.title)
        .bind(&req.doc_type)
        .bind(&req.category)
        .bind(req.effective_date)
        .bind(req.review_due_date)
        .bind(req.retention_years)
        .bind(&req.file_path)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "controlled_document",
                    after.id,
                    &after.doc_number,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_controlled_document(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateControlledDocumentRequest,
    ) -> Result<ControlledDocument> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ControlledDocument>(
            "SELECT * FROM controlled_documents WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("受控文件不存在".into()))?;

        // CSO-r2 #4: 受控文件「發布類」狀態必須走 approve_controlled_document（需 dms.document.approve），
        // 不得由僅持 dms.document.manage 的泛型 update 直接設定，維持 21 CFR 11 簽核權責分離（SoD）。
        if let Some(ref new_status) = req.status {
            const RELEASE_STATUSES: [&str; 2] = ["approved", "effective"];
            if RELEASE_STATUSES.contains(&new_status.as_str()) && *new_status != before.status {
                return Err(AppError::BusinessRule(
                    "文件核准（approved/effective）須透過核准流程，不可由編輯更新直接設定".into(),
                ));
            }
        }

        let after = sqlx::query_as::<_, ControlledDocument>(
            r#"
            UPDATE controlled_documents SET
                title = COALESCE($2, title),
                category = COALESCE($3, category),
                status = COALESCE($4, status),
                effective_date = COALESCE($5, effective_date),
                review_due_date = COALESCE($6, review_due_date),
                retention_years = COALESCE($7, retention_years),
                file_path = COALESCE($8, file_path),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.category)
        .bind(&req.status)
        .bind(req.effective_date)
        .bind(req.review_due_date)
        .bind(req.retention_years)
        .bind(&req.file_path)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new(
                    "controlled_document",
                    id,
                    &after.doc_number,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn approve_controlled_document(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&JsonValue>,
    ) -> Result<ControlledDocument> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ControlledDocument>(
            "SELECT * FROM controlled_documents WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("受控文件不存在".into()))?;

        if before.status != "under_review" {
            return Err(AppError::BusinessRule(
                "只有審查中（under_review）的文件可以核准；草稿須先送審".into(),
            ));
        }
        // A3 職權分離：受控文件全類型，核准人不得為文件擁有者（撰寫者）本人。
        if before.owner_id == Some(user.id) {
            return Err(AppError::Forbidden(
                "職權分離：受控文件的核准人不得為文件擁有者（撰寫者）本人".into(),
            ));
        }

        let after = sqlx::query_as::<_, ControlledDocument>(
            r#"
            UPDATE controlled_documents SET
                status = 'approved', approved_by = $2, approved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let content = format!(
            "APPROVE:controlled_document:{}:v{}",
            after.doc_number, after.current_version
        );
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "controlled_document",
            &id.to_string(),
            user.id,
            SignatureType::Approve,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "APPROVE",
                entity: Some(AuditEntity::new(
                    "controlled_document",
                    id,
                    &after.doc_number,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn get_document_revisions(
        pool: &PgPool,
        document_id: Uuid,
    ) -> Result<Vec<DocumentRevision>> {
        repo::find_document_revisions(pool, document_id).await
    }

    pub async fn create_revision(
        pool: &PgPool,
        actor: &ActorContext,
        document_id: Uuid,
        req: &CreateRevisionRequest,
    ) -> Result<DocumentRevision> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let doc = sqlx::query_as::<_, ControlledDocument>(
            "SELECT * FROM controlled_documents WHERE id = $1 FOR UPDATE",
        )
        .bind(document_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("受控文件不存在".into()))?;

        let new_version = doc.current_version + 1;

        sqlx::query(
            r#"
            UPDATE controlled_documents SET
                status = 'draft', current_version = $2, updated_at = NOW()
            WHERE id = $1
        "#,
        )
        .bind(document_id)
        .bind(new_version)
        .execute(&mut *tx)
        .await?;

        let after = sqlx::query_as::<_, DocumentRevision>(r#"
            INSERT INTO document_revisions (document_id, version, change_summary, revised_by, file_path)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING *
        "#)
        .bind(document_id)
        .bind(new_version)
        .bind(&req.change_summary)
        .bind(user.id)
        .bind(&req.file_path)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} v{}", doc.doc_number, new_version);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("document_revision", after.id, &display)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn acknowledge_document(
        pool: &PgPool,
        actor: &ActorContext,
        document_id: Uuid,
    ) -> Result<DocumentAcknowledgment> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let doc = sqlx::query_as::<_, ControlledDocument>(
            "SELECT * FROM controlled_documents WHERE id = $1",
        )
        .bind(document_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("受控文件不存在".into()))?;

        let after = sqlx::query_as::<_, DocumentAcknowledgment>(r#"
            INSERT INTO document_acknowledgments (document_id, user_id, version_acknowledged)
            VALUES ($1,$2,$3)
            ON CONFLICT (document_id, user_id, version_acknowledged) DO UPDATE SET acknowledged_at = NOW()
            RETURNING *
        "#)
        .bind(document_id)
        .bind(user.id)
        .bind(doc.current_version)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} v{}", doc.doc_number, doc.current_version);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "ACKNOWLEDGE",
                entity: Some(AuditEntity::new(
                    "document_acknowledgment",
                    after.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    async fn generate_doc_number(pool: &PgPool, doc_type: &str) -> Result<String> {
        let prefix = match doc_type {
            "quality_manual" => "QM",
            "sop" => "SOP",
            "form" => "FM",
            "external" => "EXT",
            "policy" => "POL",
            "report" => "RPT",
            _ => "DOC",
        };
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM controlled_documents WHERE doc_type = $1 AND EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .bind(doc_type)
        .fetch_one(pool)
        .await?;
        Ok(format!("{prefix}-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Management Reviews
    // ========================================================================

    pub async fn list_management_reviews(
        pool: &PgPool,
        params: &ManagementReviewQuery,
    ) -> Result<Vec<ManagementReview>> {
        repo::find_management_reviews(pool, params).await
    }

    pub async fn get_management_review(pool: &PgPool, id: Uuid) -> Result<ManagementReview> {
        repo::find_management_review_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("管理審查不存在".into()))
    }

    pub async fn create_management_review(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateManagementReviewRequest,
    ) -> Result<ManagementReview> {
        let number = Self::generate_review_number(pool).await?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, ManagementReview>(r#"
            INSERT INTO management_reviews (review_number, title, review_date, agenda, attendees, chaired_by)
            VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING *
        "#)
        .bind(&number)
        .bind(&req.title)
        .bind(req.review_date)
        .bind(&req.agenda)
        .bind(&req.attendees)
        .bind(req.chaired_by)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "management_review",
                    after.id,
                    &after.review_number,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_management_review(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateManagementReviewRequest,
    ) -> Result<ManagementReview> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ManagementReview>(
            "SELECT * FROM management_reviews WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("管理審查不存在".into()))?;

        // R71-6：管理審查結案保護（軟性 SoD）。須具 glp.management_review.approve 的情境：
        //   (a) 目前已是 completed/closed —— 防 manage-only 篡改或降級已結案審查（內容不可竄改）
        //   (b) 正轉入 completed/closed —— 防 manage-only 自行結案、繞過簽核
        // before 由上方 SELECT ... FOR UPDATE 於同 tx 鎖列取得，故無 TOCTOU 競態。
        {
            const RELEASE_STATUSES: [&str; 2] = ["completed", "closed"];
            let currently_released = RELEASE_STATUSES.contains(&before.status.as_str());
            let transitioning_to_released = req
                .status
                .as_deref()
                .map(|s| RELEASE_STATUSES.contains(&s) && s != before.status.as_str())
                .unwrap_or(false);
            if currently_released || transitioning_to_released {
                let current_user = actor.require_user()?;
                if !current_user.has_permission("glp.management_review.approve") {
                    return Err(AppError::Forbidden(
                        "修改已結案之管理審查或將其結案（completed/closed）須具備 glp.management_review.approve 權限".into(),
                    ));
                }
            }
        }

        let after = sqlx::query_as::<_, ManagementReview>(
            r#"
            UPDATE management_reviews SET
                title = COALESCE($2, title),
                review_date = COALESCE($3, review_date),
                status = COALESCE($4, status),
                agenda = COALESCE($5, agenda),
                attendees = COALESCE($6, attendees),
                minutes = COALESCE($7, minutes),
                decisions = COALESCE($8, decisions),
                action_items = COALESCE($9, action_items),
                chaired_by = COALESCE($10, chaired_by),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(req.review_date)
        .bind(&req.status)
        .bind(&req.agenda)
        .bind(&req.attendees)
        .bind(&req.minutes)
        .bind(&req.decisions)
        .bind(&req.action_items)
        .bind(req.chaired_by)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new(
                    "management_review",
                    id,
                    &after.review_number,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    async fn generate_review_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM management_reviews WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("MR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Risk Register
    // ========================================================================

    pub async fn list_risks(pool: &PgPool, params: &RiskQuery) -> Result<Vec<RiskEntryWithOwner>> {
        repo::find_risks(pool, params).await
    }

    pub async fn get_risk(pool: &PgPool, id: Uuid) -> Result<RiskEntryWithOwner> {
        repo::find_risk_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("風險項目不存在".into()))
    }

    pub async fn create_risk(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateRiskRequest,
    ) -> Result<RiskEntry> {
        let number = Self::generate_risk_number(pool).await?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, RiskEntry>(
            r#"
            INSERT INTO risk_register
                (risk_number, title, description, category, source, severity, likelihood,
                 detectability, mitigation_plan, owner_id, review_date, related_nc_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING *
        "#,
        )
        .bind(&number)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.category)
        .bind(&req.source)
        .bind(req.severity)
        .bind(req.likelihood)
        .bind(req.detectability)
        .bind(&req.mitigation_plan)
        .bind(req.owner_id)
        .bind(req.review_date)
        .bind(req.related_nc_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("risk_entry", after.id, &after.risk_number)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_risk(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateRiskRequest,
    ) -> Result<RiskEntry> {
        let mut tx = pool.begin().await?;

        let before =
            sqlx::query_as::<_, RiskEntry>("SELECT * FROM risk_register WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(AppError::NotFound("風險項目不存在".into()))?;

        let after = sqlx::query_as::<_, RiskEntry>(
            r#"
            UPDATE risk_register SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                category = COALESCE($4, category),
                source = COALESCE($5, source),
                severity = COALESCE($6, severity),
                likelihood = COALESCE($7, likelihood),
                detectability = COALESCE($8, detectability),
                status = COALESCE($9, status),
                mitigation_plan = COALESCE($10, mitigation_plan),
                residual_risk_score = COALESCE($11, residual_risk_score),
                owner_id = COALESCE($12, owner_id),
                review_date = COALESCE($13, review_date),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.category)
        .bind(&req.source)
        .bind(req.severity)
        .bind(req.likelihood)
        .bind(req.detectability)
        .bind(&req.status)
        .bind(&req.mitigation_plan)
        .bind(req.residual_risk_score)
        .bind(req.owner_id)
        .bind(req.review_date)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new("risk_entry", id, &after.risk_number)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    async fn generate_risk_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM risk_register WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("RISK-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Change Requests
    // ========================================================================

    pub async fn list_change_requests(
        pool: &PgPool,
        params: &ChangeRequestQuery,
    ) -> Result<Vec<ChangeRequestWithNames>> {
        repo::find_change_requests(pool, params).await
    }

    pub async fn get_change_request(pool: &PgPool, id: Uuid) -> Result<ChangeRequest> {
        repo::find_change_request_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("變更申請不存在".into()))
    }

    pub async fn create_change_request(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateChangeRequestRequest,
    ) -> Result<ChangeRequest> {
        let user = actor.require_user()?;
        let number = Self::generate_change_number(pool).await?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, ChangeRequest>(r#"
            INSERT INTO change_requests
                (change_number, title, change_type, description, justification, impact_assessment, requested_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
        "#)
        .bind(&number)
        .bind(&req.title)
        .bind(&req.change_type)
        .bind(&req.description)
        .bind(&req.justification)
        .bind(&req.impact_assessment)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "change_request",
                    after.id,
                    &after.change_number,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_change_request(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateChangeRequestRequest,
    ) -> Result<ChangeRequest> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ChangeRequest>(
            "SELECT * FROM change_requests WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("變更申請不存在".into()))?;

        // CSO-r2 #4: 「approved」為發布類狀態，須走 approve_change_request（需 change.request.approve
        // + 電子簽章），不得由僅持 change.request.manage 的泛型 update 直接設定，維持簽核權責分離（SoD）。
        if let Some(ref new_status) = req.status {
            const RELEASE_STATUSES: [&str; 1] = ["approved"];
            if RELEASE_STATUSES.contains(&new_status.as_str()) && *new_status != before.status {
                return Err(AppError::BusinessRule(
                    "變更申請核准（approved）須透過核准流程，不可由編輯更新直接設定".into(),
                ));
            }
        }

        let after = sqlx::query_as::<_, ChangeRequest>(
            r#"
            UPDATE change_requests SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                justification = COALESCE($4, justification),
                impact_assessment = COALESCE($5, impact_assessment),
                status = COALESCE($6, status),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.justification)
        .bind(&req.impact_assessment)
        .bind(&req.status)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new("change_request", id, &after.change_number)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn approve_change_request(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&JsonValue>,
    ) -> Result<ChangeRequest> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, ChangeRequest>(
            "SELECT * FROM change_requests WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("變更申請不存在".into()))?;

        // A3 職權分離（Part 11 變更管制）：核准人不得為申請人本人。
        if before.requested_by == user.id {
            return Err(AppError::Forbidden(
                "職權分離：變更申請的核准人不得為申請人本人".into(),
            ));
        }

        if before.status != "submitted" && before.status != "under_review" {
            return Err(AppError::BusinessRule(
                "只有已提交或審查中的變更申請可以核准".into(),
            ));
        }

        let after = sqlx::query_as::<_, ChangeRequest>(
            r#"
            UPDATE change_requests SET
                status = 'approved', approved_by = $2, approved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let content = format!("APPROVE:change_request:{}", after.change_number);
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "change_request",
            &id.to_string(),
            user.id,
            SignatureType::Approve,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "APPROVE",
                entity: Some(AuditEntity::new("change_request", id, &after.change_number)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    async fn generate_change_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM change_requests WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("CR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Environment Monitoring
    // ========================================================================

    pub async fn list_monitoring_points(
        pool: &PgPool,
        active_only: bool,
    ) -> Result<Vec<EnvironmentMonitoringPoint>> {
        repo::find_monitoring_points(pool, active_only).await
    }

    pub async fn get_monitoring_point(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<EnvironmentMonitoringPoint> {
        repo::find_monitoring_point_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("監控點不存在".into()))
    }

    pub async fn create_monitoring_point(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateMonitoringPointRequest,
    ) -> Result<EnvironmentMonitoringPoint> {
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, EnvironmentMonitoringPoint>(
            r#"
            INSERT INTO environment_monitoring_points
                (name, location_type, building_id, zone_id, parameters, monitoring_interval)
            VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING *
        "#,
        )
        .bind(&req.name)
        .bind(&req.location_type)
        .bind(req.building_id)
        .bind(req.zone_id)
        .bind(&req.parameters)
        .bind(&req.monitoring_interval)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("monitoring_point", after.id, &after.name)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_monitoring_point(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateMonitoringPointRequest,
    ) -> Result<EnvironmentMonitoringPoint> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, EnvironmentMonitoringPoint>(
            "SELECT * FROM environment_monitoring_points WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("監控點不存在".into()))?;

        let after = sqlx::query_as::<_, EnvironmentMonitoringPoint>(
            r#"
            UPDATE environment_monitoring_points SET
                name = COALESCE($2, name),
                location_type = COALESCE($3, location_type),
                building_id = COALESCE($4, building_id),
                zone_id = COALESCE($5, zone_id),
                parameters = COALESCE($6, parameters),
                monitoring_interval = COALESCE($7, monitoring_interval),
                is_active = COALESCE($8, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.location_type)
        .bind(req.building_id)
        .bind(req.zone_id)
        .bind(&req.parameters)
        .bind(&req.monitoring_interval)
        .bind(req.is_active)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new("monitoring_point", id, &after.name)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn list_readings(
        pool: &PgPool,
        params: &ReadingQuery,
    ) -> Result<Vec<EnvironmentReading>> {
        repo::find_readings(pool, params).await
    }

    pub async fn create_reading(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateReadingRequest,
    ) -> Result<EnvironmentReading> {
        let user = actor.require_user()?;
        let point = repo::find_monitoring_point_by_id(pool, req.monitoring_point_id)
            .await?
            .ok_or(AppError::NotFound("監控點不存在".into()))?;

        let (is_oor, oor_params) = Self::check_out_of_range(&point.parameters, &req.readings);
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, EnvironmentReading>(r#"
            INSERT INTO environment_readings
                (monitoring_point_id, reading_time, readings, is_out_of_range, out_of_range_params, recorded_by, source, notes)
            VALUES ($1,$2,$3,$4,$5,$6,'manual',$7)
            RETURNING *
        "#)
        .bind(req.monitoring_point_id)
        .bind(req.reading_time)
        .bind(&req.readings)
        .bind(is_oor)
        .bind(&oor_params)
        .bind(user.id)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} @ {}",
            point.name,
            req.reading_time.format("%Y-%m-%d %H:%M")
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("environment_reading", after.id, &display)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    fn check_out_of_range(
        parameters: &serde_json::Value,
        readings: &serde_json::Value,
    ) -> (bool, Option<serde_json::Value>) {
        let mut oor_list: Vec<String> = Vec::new();

        if let (Some(params_arr), Some(readings_obj)) =
            (parameters.as_array(), readings.as_object())
        {
            for param in params_arr {
                let name = param.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let min = param.get("min").and_then(|v| v.as_f64());
                let max = param.get("max").and_then(|v| v.as_f64());
                let value = readings_obj.get(name).and_then(|v| v.as_f64());

                if let Some(val) = value {
                    if let Some(mn) = min {
                        if val < mn {
                            oor_list.push(name.to_string());
                            continue;
                        }
                    }
                    if let Some(mx) = max {
                        if val > mx {
                            oor_list.push(name.to_string());
                        }
                    }
                }
            }
        }

        if oor_list.is_empty() {
            (false, None)
        } else {
            (true, Some(serde_json::json!(oor_list)))
        }
    }

    // ========================================================================
    // Competency Assessments
    // ========================================================================

    pub async fn list_competency_assessments(
        pool: &PgPool,
        params: &CompetencyQuery,
    ) -> Result<Vec<CompetencyAssessmentWithNames>> {
        repo::find_competency_assessments(pool, params).await
    }

    pub async fn create_competency_assessment(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateCompetencyRequest,
    ) -> Result<CompetencyAssessment> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, CompetencyAssessment>(r#"
            INSERT INTO competency_assessments
                (user_id, assessment_type, skill_area, assessment_date, assessor_id, result, score, method, valid_until, notes)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *
        "#)
        .bind(req.user_id)
        .bind(&req.assessment_type)
        .bind(&req.skill_area)
        .bind(req.assessment_date)
        .bind(user.id)
        .bind(&req.result)
        .bind(req.score)
        .bind(&req.method)
        .bind(req.valid_until)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} - {}", req.skill_area, req.assessment_type);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "competency_assessment",
                    after.id,
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_competency_assessment(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateCompetencyRequest,
    ) -> Result<CompetencyAssessment> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, CompetencyAssessment>(
            "SELECT * FROM competency_assessments WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("能力評鑑不存在".into()))?;

        let after = sqlx::query_as::<_, CompetencyAssessment>(
            r#"
            UPDATE competency_assessments SET
                result = COALESCE($2, result),
                score = COALESCE($3, score),
                method = COALESCE($4, method),
                valid_until = COALESCE($5, valid_until),
                notes = COALESCE($6, notes),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.result)
        .bind(req.score)
        .bind(&req.method)
        .bind(req.valid_until)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} - {}", after.skill_area, after.assessment_type);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new("competency_assessment", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn list_training_requirements(
        pool: &PgPool,
        role_code: Option<&str>,
    ) -> Result<Vec<RoleTrainingRequirement>> {
        repo::find_training_requirements(pool, role_code).await
    }

    pub async fn create_training_requirement(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateTrainingRequirementRequest,
    ) -> Result<RoleTrainingRequirement> {
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, RoleTrainingRequirement>(r#"
            INSERT INTO role_training_requirements (role_code, training_topic, is_mandatory, recurrence_months)
            VALUES ($1,$2,COALESCE($3, true),$4)
            RETURNING *
        "#)
        .bind(&req.role_code)
        .bind(&req.training_topic)
        .bind(req.is_mandatory)
        .bind(req.recurrence_months)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} / {}", req.role_code, req.training_topic);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("training_requirement", after.id, &display)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn delete_training_requirement(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, RoleTrainingRequirement>(
            "SELECT * FROM role_training_requirements WHERE id = $1 AND deleted_at IS NULL FOR UPDATE"
        ).bind(id).fetch_optional(&mut *tx).await?
         .ok_or(AppError::NotFound("訓練需求不存在".into()))?;

        sqlx::query("UPDATE role_training_requirements SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let display = format!("{} / {}", before.role_code, before.training_topic);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "DELETE",
                entity: Some(AuditEntity::new("training_requirement", id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // ========================================================================
    // Study Final Reports
    // ========================================================================

    pub async fn list_study_reports(
        pool: &PgPool,
        params: &StudyReportQuery,
    ) -> Result<Vec<StudyFinalReport>> {
        repo::find_study_reports(pool, params).await
    }

    pub async fn get_study_report(pool: &PgPool, id: Uuid) -> Result<StudyFinalReport> {
        repo::find_study_report_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("最終報告不存在".into()))
    }

    pub async fn create_study_report(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateStudyReportRequest,
    ) -> Result<StudyFinalReport> {
        let number = Self::generate_report_number(pool).await?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, StudyFinalReport>(r#"
            INSERT INTO study_final_reports
                (report_number, protocol_id, title, summary, methods, results, conclusions, deviations)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
        "#)
        .bind(&number)
        .bind(req.protocol_id)
        .bind(&req.title)
        .bind(&req.summary)
        .bind(&req.methods)
        .bind(&req.results)
        .bind(&req.conclusions)
        .bind(&req.deviations)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new(
                    "study_final_report",
                    after.id,
                    &after.report_number,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    pub async fn update_study_report(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateStudyReportRequest,
    ) -> Result<StudyFinalReport> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, StudyFinalReport>(
            "SELECT * FROM study_final_reports WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("最終報告不存在".into()))?;

        // CSO-r2 #4: 「approved/signed」為簽署發布類狀態，不得由僅持 study.report.manage 的泛型
        // update 直接設定（會跳過電子簽章、signature 欄位留 NULL），維持最終報告簽署權責分離（SoD）。
        // 注意：目前尚無正式 sign_study_report 流程，此守衛先止血；完整簽署流程為 follow-up。
        if let Some(ref new_status) = req.status {
            const RELEASE_STATUSES: [&str; 2] = ["approved", "signed"];
            if RELEASE_STATUSES.contains(&new_status.as_str()) && *new_status != before.status {
                return Err(AppError::BusinessRule(
                    "最終報告核准／簽署（approved/signed）須透過簽署流程，不可由編輯更新直接設定"
                        .into(),
                ));
            }
        }

        let after = sqlx::query_as::<_, StudyFinalReport>(
            r#"
            UPDATE study_final_reports SET
                title = COALESCE($2, title),
                status = COALESCE($3, status),
                summary = COALESCE($4, summary),
                methods = COALESCE($5, methods),
                results = COALESCE($6, results),
                conclusions = COALESCE($7, conclusions),
                deviations = COALESCE($8, deviations),
                qau_statement = COALESCE($9, qau_statement),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.status)
        .bind(&req.summary)
        .bind(&req.methods)
        .bind(&req.results)
        .bind(&req.conclusions)
        .bind(&req.deviations)
        .bind(&req.qau_statement)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "UPDATE",
                entity: Some(AuditEntity::new(
                    "study_final_report",
                    id,
                    &after.report_number,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    async fn generate_report_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM study_final_reports WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("SFR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Formulation Records
    // ========================================================================

    pub async fn list_formulation_records(
        pool: &PgPool,
        params: &FormulationQuery,
    ) -> Result<Vec<FormulationRecordWithNames>> {
        repo::find_formulation_records(pool, params).await
    }

    pub async fn create_formulation_record(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateFormulationRequest,
    ) -> Result<FormulationRecord> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let after = sqlx::query_as::<_, FormulationRecord>(r#"
            INSERT INTO formulation_records
                (product_id, protocol_id, formulation_date, batch_number, concentration, volume, prepared_by, expiry_date, notes)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *
        "#)
        .bind(req.product_id)
        .bind(req.protocol_id)
        .bind(req.formulation_date)
        .bind(&req.batch_number)
        .bind(&req.concentration)
        .bind(&req.volume)
        .bind(user.id)
        .bind(req.expiry_date)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("配製 {}", req.formulation_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "CREATE",
                entity: Some(AuditEntity::new("formulation_record", after.id, &display)),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }
}
