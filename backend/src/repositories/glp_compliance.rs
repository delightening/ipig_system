// Repository 層：GLP 合規模組所有 SQL 查詢

use crate::models::glp_compliance::*;
use crate::Result;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Reference Standards
// ============================================================================

pub async fn find_reference_standards(pool: &PgPool) -> Result<Vec<ReferenceStandard>> {
    let rows = sqlx::query_as::<_, ReferenceStandard>(
        "SELECT * FROM reference_standards ORDER BY name LIMIT 200",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_reference_standard_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ReferenceStandard>> {
    let row =
        sqlx::query_as::<_, ReferenceStandard>("SELECT * FROM reference_standards WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn update_reference_standard(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateReferenceStandardRequest,
) -> Result<ReferenceStandard> {
    let row = sqlx::query_as::<_, ReferenceStandard>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ============================================================================
// Controlled Documents
// ============================================================================

pub async fn find_controlled_documents(
    pool: &PgPool,
    params: &ControlledDocumentQuery,
) -> Result<Vec<ControlledDocumentWithOwner>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, ControlledDocumentWithOwner>(
        r#"
        SELECT d.id, d.doc_number, d.title, d.doc_type, d.category,
               d.current_version, d.status, d.effective_date, d.review_due_date,
               d.owner_id, u.display_name AS owner_name,
               d.approved_by, d.approved_at, d.retention_years,
               d.created_at, d.updated_at
        FROM controlled_documents d
        LEFT JOIN users u ON u.id = d.owner_id
        WHERE ($1::text IS NULL OR d.doc_type = $1)
          AND ($2::text IS NULL OR d.status = $2)
          AND ($3::text IS NULL OR d.category = $3)
        ORDER BY d.updated_at DESC
        LIMIT $4 OFFSET $5
    "#,
    )
    .bind(&params.doc_type)
    .bind(&params.status)
    .bind(&params.category)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_controlled_document_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ControlledDocumentWithOwner>> {
    let row = sqlx::query_as::<_, ControlledDocumentWithOwner>(
        r#"
        SELECT d.id, d.doc_number, d.title, d.doc_type, d.category,
               d.current_version, d.status, d.effective_date, d.review_due_date,
               d.owner_id, u.display_name AS owner_name,
               d.approved_by, d.approved_at, d.retention_years,
               d.created_at, d.updated_at
        FROM controlled_documents d
        LEFT JOIN users u ON u.id = d.owner_id
        WHERE d.id = $1
    "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_controlled_document(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateControlledDocumentRequest,
) -> Result<ControlledDocument> {
    let row = sqlx::query_as::<_, ControlledDocument>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn approve_controlled_document(
    pool: &PgPool,
    id: Uuid,
    approver_id: Uuid,
) -> Result<ControlledDocument> {
    let row = sqlx::query_as::<_, ControlledDocument>(
        r#"
        UPDATE controlled_documents SET
            status = 'approved', approved_by = $2, approved_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING *
    "#,
    )
    .bind(id)
    .bind(approver_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_document_revisions(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Vec<DocumentRevision>> {
    let rows = sqlx::query_as::<_, DocumentRevision>(
        "SELECT * FROM document_revisions WHERE document_id = $1 ORDER BY version DESC",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ============================================================================
// Management Reviews
// ============================================================================

pub async fn find_management_reviews(
    pool: &PgPool,
    params: &ManagementReviewQuery,
) -> Result<Vec<ManagementReview>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, ManagementReview>(
        r#"
        SELECT * FROM management_reviews
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY review_date DESC
        LIMIT $2 OFFSET $3
    "#,
    )
    .bind(&params.status)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_management_review_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ManagementReview>> {
    let row =
        sqlx::query_as::<_, ManagementReview>("SELECT * FROM management_reviews WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn update_management_review(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateManagementReviewRequest,
) -> Result<ManagementReview> {
    let row = sqlx::query_as::<_, ManagementReview>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ============================================================================
// Risk Register
// ============================================================================

pub async fn find_risks(pool: &PgPool, params: &RiskQuery) -> Result<Vec<RiskEntryWithOwner>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, RiskEntryWithOwner>(
        r#"
        SELECT r.id, r.risk_number, r.title, r.description, r.category, r.source,
               r.severity, r.likelihood, r.detectability, r.risk_score,
               r.status, r.mitigation_plan, r.residual_risk_score,
               r.owner_id, u.display_name AS owner_name,
               r.review_date, r.created_at, r.updated_at
        FROM risk_register r
        LEFT JOIN users u ON u.id = r.owner_id
        WHERE ($1::text IS NULL OR r.status = $1)
          AND ($2::text IS NULL OR r.category = $2)
        ORDER BY r.risk_score DESC NULLS LAST, r.created_at DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(&params.status)
    .bind(&params.category)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_risk_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RiskEntryWithOwner>> {
    let row = sqlx::query_as::<_, RiskEntryWithOwner>(
        r#"
        SELECT r.id, r.risk_number, r.title, r.description, r.category, r.source,
               r.severity, r.likelihood, r.detectability, r.risk_score,
               r.status, r.mitigation_plan, r.residual_risk_score,
               r.owner_id, u.display_name AS owner_name,
               r.review_date, r.created_at, r.updated_at
        FROM risk_register r
        LEFT JOIN users u ON u.id = r.owner_id
        WHERE r.id = $1
    "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_risk(pool: &PgPool, id: Uuid, req: &UpdateRiskRequest) -> Result<RiskEntry> {
    let row = sqlx::query_as::<_, RiskEntry>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ============================================================================
// Change Requests
// ============================================================================

pub async fn find_change_requests(
    pool: &PgPool,
    params: &ChangeRequestQuery,
) -> Result<Vec<ChangeRequestWithNames>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, ChangeRequestWithNames>(
        r#"
        SELECT c.id, c.change_number, c.title, c.change_type, c.description,
               c.justification, c.impact_assessment, c.status,
               c.requested_by, u.display_name AS requester_name,
               c.approved_by, c.approved_at,
               c.created_at, c.updated_at
        FROM change_requests c
        LEFT JOIN users u ON u.id = c.requested_by
        WHERE ($1::text IS NULL OR c.status = $1)
          AND ($2::text IS NULL OR c.change_type = $2)
        ORDER BY c.created_at DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(&params.status)
    .bind(&params.change_type)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_change_request_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ChangeRequest>> {
    let row = sqlx::query_as::<_, ChangeRequest>("SELECT * FROM change_requests WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn update_change_request(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateChangeRequestRequest,
) -> Result<ChangeRequest> {
    let row = sqlx::query_as::<_, ChangeRequest>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn approve_change_request(
    pool: &PgPool,
    id: Uuid,
    approver_id: Uuid,
) -> Result<ChangeRequest> {
    let row = sqlx::query_as::<_, ChangeRequest>(
        r#"
        UPDATE change_requests SET
            status = 'approved', approved_by = $2, approved_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING *
    "#,
    )
    .bind(id)
    .bind(approver_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ============================================================================
// Environment Monitoring
// ============================================================================

pub async fn find_monitoring_points(
    pool: &PgPool,
    active_only: bool,
) -> Result<Vec<EnvironmentMonitoringPoint>> {
    let rows = if active_only {
        sqlx::query_as::<_, EnvironmentMonitoringPoint>(
            "SELECT * FROM environment_monitoring_points WHERE is_active = true ORDER BY name LIMIT 200"
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, EnvironmentMonitoringPoint>(
            "SELECT * FROM environment_monitoring_points ORDER BY name LIMIT 200",
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn find_monitoring_point_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<EnvironmentMonitoringPoint>> {
    let row = sqlx::query_as::<_, EnvironmentMonitoringPoint>(
        "SELECT * FROM environment_monitoring_points WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_monitoring_point(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateMonitoringPointRequest,
) -> Result<EnvironmentMonitoringPoint> {
    let row = sqlx::query_as::<_, EnvironmentMonitoringPoint>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_readings(
    pool: &PgPool,
    params: &ReadingQuery,
) -> Result<Vec<EnvironmentReading>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, EnvironmentReading>(
        r#"
        SELECT * FROM environment_readings
        WHERE ($1::uuid IS NULL OR monitoring_point_id = $1)
          AND ($2::bool IS NULL OR is_out_of_range = $2)
        ORDER BY reading_time DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(params.monitoring_point_id)
    .bind(params.is_out_of_range)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ============================================================================
// Competency Assessments
// ============================================================================

pub async fn find_competency_assessments(
    pool: &PgPool,
    params: &CompetencyQuery,
) -> Result<Vec<CompetencyAssessmentWithNames>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, CompetencyAssessmentWithNames>(
        r#"
        SELECT ca.id, ca.user_id, u.display_name AS user_name,
               ca.assessment_type, ca.skill_area, ca.assessment_date,
               ca.assessor_id, a.display_name AS assessor_name,
               ca.result, ca.score, ca.method, ca.valid_until, ca.notes,
               ca.created_at
        FROM competency_assessments ca
        JOIN users u ON u.id = ca.user_id
        JOIN users a ON a.id = ca.assessor_id
        WHERE ($1::uuid IS NULL OR ca.user_id = $1)
          AND ($2::text IS NULL OR ca.result = $2)
        ORDER BY ca.assessment_date DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(params.user_id)
    .bind(&params.result)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_competency_assessment(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateCompetencyRequest,
) -> Result<CompetencyAssessment> {
    let row = sqlx::query_as::<_, CompetencyAssessment>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_training_requirements(
    pool: &PgPool,
    role_code: Option<&str>,
) -> Result<Vec<RoleTrainingRequirement>> {
    let rows = sqlx::query_as::<_, RoleTrainingRequirement>(
        r#"
        SELECT * FROM role_training_requirements
        WHERE deleted_at IS NULL
          AND ($1::text IS NULL OR role_code = $1)
        ORDER BY role_code, training_topic
        LIMIT 200
    "#,
    )
    .bind(role_code)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete_training_requirement(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM role_training_requirements WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// Study Final Reports
// ============================================================================

pub async fn find_study_reports(
    pool: &PgPool,
    params: &StudyReportQuery,
) -> Result<Vec<StudyFinalReport>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, StudyFinalReport>(
        r#"
        SELECT * FROM study_final_reports
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::uuid IS NULL OR protocol_id = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(&params.status)
    .bind(params.protocol_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_study_report_by_id(pool: &PgPool, id: Uuid) -> Result<Option<StudyFinalReport>> {
    let row =
        sqlx::query_as::<_, StudyFinalReport>("SELECT * FROM study_final_reports WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn update_study_report(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateStudyReportRequest,
) -> Result<StudyFinalReport> {
    let row = sqlx::query_as::<_, StudyFinalReport>(
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
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ============================================================================
// Formulation Records
// ============================================================================

pub async fn find_formulation_records(
    pool: &PgPool,
    params: &FormulationQuery,
) -> Result<Vec<FormulationRecordWithNames>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, FormulationRecordWithNames>(
        r#"
        SELECT f.id, f.product_id, p.name AS product_name,
               f.protocol_id, f.formulation_date, f.batch_number,
               f.concentration, f.volume,
               f.prepared_by, u.display_name AS preparer_name,
               f.verified_by, f.verified_at, f.expiry_date, f.notes,
               f.created_at
        FROM formulation_records f
        JOIN products p ON p.id = f.product_id
        JOIN users u ON u.id = f.prepared_by
        WHERE ($1::uuid IS NULL OR f.product_id = $1)
          AND ($2::uuid IS NULL OR f.protocol_id = $2)
        ORDER BY f.formulation_date DESC
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(params.product_id)
    .bind(params.protocol_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
