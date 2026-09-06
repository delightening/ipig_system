// Service 層：GLP 合規模組業務邏輯
// R63-A: 所有 mutation 走 TX + audit logging；approve 加電子簽章

use crate::error::AppError;
use crate::middleware::{ActorContext, CurrentUser};
use crate::models::audit_diff::DataDiff;
use crate::models::glp_compliance::*;
use crate::models::Protocol;
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
    //
    // 2026-09-05 授權重構（docs/reviews/2026-09-03-code-side-issues.md P0-1）：
    // `study.report.manage` 曾是「授給零角色，只有 admin 能用」的角色權限碼；
    // SD 不是全域角色（2026-08-17 已廢除），只存在於「該計畫的 SD」
    // （protocols.study_director_user_id），角色授予表達不出「該報告的 SD 才能寫」。
    // 改走身分即授權，比照 `services/protocol/closure.rs` 的結案雙簽模式。
    // ========================================================================

    /// 是否有權檢視某最終報告：具 `study.report.view` / admin（`has_permission`
    /// 對 admin 恆真）／`qau.report_statement.write`（QAU 需要先看到報告才能寫聲明）／
    /// 本人是該計畫的 SD——SD 至少要能看到自己要簽的那份報告。
    async fn can_view_study_report(
        pool: &PgPool,
        user: &CurrentUser,
        protocol_id: Uuid,
    ) -> Result<bool> {
        if user.has_permission("study.report.view")
            || user.has_permission("qau.report_statement.write")
        {
            return Ok(true);
        }
        let sd: Option<Uuid> =
            sqlx::query_scalar("SELECT study_director_user_id FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(pool)
                .await?
                .flatten();
        Ok(sd == Some(user.id))
    }

    /// 讀出計畫並確認 `user` 是不是該計畫的 Study Director。
    ///
    /// `allow_admin_escape`：
    /// - 撰寫／編輯報告內容（create/update） → `true`：admin 可代為修正，這不是具結行為。
    /// - 簽署（`sign_study_report`，寫入 signed_by/signed_at/signature_id） → `false`：
    ///   比照 `protocol_closure` 的結案簽章——**代簽的簽章在稽核上沒有價值**。
    ///   SD 異動時應改派新 SD（既有的 protocol 編輯流程），不透過本函式繞過簽署。
    async fn require_study_director(
        pool: &PgPool,
        user: &CurrentUser,
        protocol_id: Uuid,
        allow_admin_escape: bool,
    ) -> Result<Protocol> {
        let protocol = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(protocol_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

        if allow_admin_escape && user.is_admin() {
            return Ok(protocol);
        }

        match protocol.study_director_user_id {
            Some(uid) if uid == user.id => Ok(protocol),
            Some(_) => Err(AppError::Forbidden(
                "只有本計畫的計劃負責人（Study Director）可以撰寫或簽署最終報告。".into(),
            )),
            None => Err(AppError::BusinessRule(
                "本計畫尚未指派計劃負責人（Study Director），無法建立或簽署最終報告。\
                 請先請執行秘書指派。"
                    .into(),
            )),
        }
    }

    pub async fn list_study_reports(
        pool: &PgPool,
        user: &CurrentUser,
        params: &StudyReportQuery,
    ) -> Result<Vec<StudyFinalReport>> {
        let restrict_to_sd = if user.has_permission("study.report.view")
            || user.has_permission("qau.report_statement.write")
        {
            None
        } else {
            Some(user.id)
        };
        repo::find_study_reports(pool, params, restrict_to_sd).await
    }

    pub async fn get_study_report(
        pool: &PgPool,
        user: &CurrentUser,
        id: Uuid,
    ) -> Result<StudyFinalReport> {
        let item = repo::find_study_report_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("最終報告不存在".into()))?;
        if !Self::can_view_study_report(pool, user, item.protocol_id).await? {
            return Err(AppError::Forbidden("沒有權限檢視此最終報告".into()));
        }
        Ok(item)
    }

    pub async fn create_study_report(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateStudyReportRequest,
    ) -> Result<StudyFinalReport> {
        let user = actor.require_user()?;
        Self::require_study_director(pool, user, req.protocol_id, true).await?;

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
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, StudyFinalReport>(
            "SELECT * FROM study_final_reports WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("最終報告不存在".into()))?;

        Self::require_study_director(pool, user, before.protocol_id, true).await?;

        // CSO-r2 #4: 「approved/signed」為簽署發布類狀態，不得由僅持編輯授權的泛型
        // update 直接設定（會跳過電子簽章、signature 欄位留 NULL），維持最終報告簽署權責分離（SoD）。
        // 「signed」現已有正式流程（`sign_study_report`）；「approved」（審查核准後、SD 簽署前）
        // 仍無對應端點，此守衛繼續擋住兩者的直接設定——之後補 approved 流程時這條不必動。
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

    /// SD 簽署最終報告（比照 `services/protocol/closure.rs::sign_closure` 的身分即授權，
    /// 但為單簽而非雙簽——最終報告只有一個 SD 要簽，QAU 走獨立的 `update_qau_statement`）。
    ///
    /// 🔴 無 admin 例外：代簽的簽章在稽核上沒有價值（比照 protocol_closure 的理由）。
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_study_report(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&JsonValue>,
    ) -> Result<StudyFinalReport> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, StudyFinalReport>(
            "SELECT * FROM study_final_reports WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("最終報告不存在".into()))?;

        if before.status == "signed" {
            return Err(AppError::BusinessRule(
                "此報告已簽署，不可重複簽署。".into(),
            ));
        }

        // 身分即授權，無 admin 例外——鎖 protocols 該列，避免簽署期間 SD 被改派（TOCTOU）。
        let protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(before.protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;
        match protocol.study_director_user_id {
            Some(uid) if uid == user.id => {}
            Some(_) => {
                return Err(AppError::Forbidden(
                    "只有本計畫的計劃負責人（Study Director）可以簽署最終報告。".into(),
                ))
            }
            None => {
                return Err(AppError::BusinessRule(
                    "本計畫尚未指派計劃負責人（Study Director），無法簽署最終報告。".into(),
                ))
            }
        }

        let content = format!(
            "study_final_report:{},report_number:{},title:{}",
            before.id, before.report_number, before.title
        );

        let signature = SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "study_final_report",
            &before.id.to_string(),
            user.id,
            // §11.50 "responsibility"：SD 對報告內容具結，不是審查方核准
            // （同 protocol_closure 對 Confirm/Approve 的區分理由）。
            SignatureType::Confirm,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let after = sqlx::query_as::<_, StudyFinalReport>(
            r#"
            UPDATE study_final_reports SET
                status = 'signed',
                signed_by = $2,
                signed_at = NOW(),
                signature_id = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(user.id)
        .bind(signature.id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "GLP",
                event_type: "SIGN",
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

    /// QAU 品保聲明填寫。呼叫端（handler）已透過 `qau.report_statement.write`
    /// 權限碼把關；本函式另外擋「QAU 簽署人不得與該計畫 SD 為同一人」——
    /// 這是結構性 SoD，不只依賴「記得別把兩個角色給同一人」（admin 除外）。
    pub async fn update_qau_statement(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        qau_statement: &str,
    ) -> Result<StudyFinalReport> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, StudyFinalReport>(
            "SELECT * FROM study_final_reports WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("最終報告不存在".into()))?;

        let sd: Option<Uuid> =
            sqlx::query_scalar("SELECT study_director_user_id FROM protocols WHERE id = $1")
                .bind(before.protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .flatten();
        if !user.is_admin() && sd == Some(user.id) {
            return Err(AppError::Forbidden(
                "本計畫的 Study Director 不可同時填寫 QAU 品保聲明（職責分離）。".into(),
            ));
        }

        let after = sqlx::query_as::<_, StudyFinalReport>(
            r#"
            UPDATE study_final_reports SET
                qau_statement = $2,
                qau_signed_by = $3,
                qau_signed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#,
        )
        .bind(id)
        .bind(qau_statement)
        .bind(user.id)
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
