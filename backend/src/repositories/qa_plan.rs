// QA 計畫管理 Repository（稽查報告、不符合事項、SOP 文件、稽查排程）

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateCapaRequest, CreateInspectionItemRequest, CreateNcRequest, CreateScheduleItemRequest,
        InspectionQuery, NcQuery, QaAuditSchedule, QaCapa, QaInspection, QaInspectionItem,
        QaInspectionWithInspector, QaNonConformance,
        QaNonConformanceWithDetails, QaScheduleItem, QaSopDocument, QaSopDocumentWithAck,
        ScheduleQuery, SopQuery, UpdateCapaRequest, UpdateInspectionRequest, UpdateNcRequest,
        UpdateScheduleItemRequest, UpdateSopRequest,
    },
    Result,
};

// ============================================================
// 稽查報告
// ============================================================

pub async fn find_inspections(
    pool: &PgPool,
    params: &InspectionQuery,
) -> Result<Vec<QaInspectionWithInspector>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, QaInspectionWithInspector>(
        r#"
        SELECT i.*,
               u.name AS inspector_name
        FROM qa_inspections i
        JOIN users u ON u.id = i.inspector_id
        WHERE ($1::text IS NULL OR i.inspection_type::text = $1)
          AND ($2::text IS NULL OR i.status::text = $2)
        ORDER BY i.inspection_date DESC, i.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&params.inspection_type)
    .bind(&params.status)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_inspection_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<QaInspectionWithInspector>> {
    let row = sqlx::query_as::<_, QaInspectionWithInspector>(
        r#"
        SELECT i.*,
               u.name AS inspector_name
        FROM qa_inspections i
        JOIN users u ON u.id = i.inspector_id
        WHERE i.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_inspection_items(
    pool: &PgPool,
    inspection_id: Uuid,
) -> Result<Vec<QaInspectionItem>> {
    let rows = sqlx::query_as::<_, QaInspectionItem>(
        "SELECT * FROM qa_inspection_items WHERE inspection_id = $1 ORDER BY item_order",
    )
    .bind(inspection_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn insert_inspection(
    pool: &PgPool,
    number: &str,
    title: &str,
    inspection_type: &str,
    inspection_date: chrono::NaiveDate,
    inspector_id: Uuid,
    related_entity_type: Option<&str>,
    related_entity_id: Option<Uuid>,
    findings: Option<&str>,
    conclusion: Option<&str>,
) -> Result<QaInspection> {
    let row = sqlx::query_as::<_, QaInspection>(
        r#"
        INSERT INTO qa_inspections
            (inspection_number, title, inspection_type, inspection_date, inspector_id,
             related_entity_type, related_entity_id, findings, conclusion)
        VALUES ($1, $2, $3::qa_inspection_type, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(number)
    .bind(title)
    .bind(inspection_type)
    .bind(inspection_date)
    .bind(inspector_id)
    .bind(related_entity_type)
    .bind(related_entity_id)
    .bind(findings)
    .bind(conclusion)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn insert_inspection_items(
    pool: &PgPool,
    inspection_id: Uuid,
    items: &[CreateInspectionItemRequest],
) -> Result<()> {
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO qa_inspection_items
                (inspection_id, item_order, description, result, remarks)
            VALUES ($1, $2, $3, $4::qa_item_result, $5)
            "#,
        )
        .bind(inspection_id)
        .bind(item.item_order)
        .bind(&item.description)
        .bind(&item.result)
        .bind(&item.remarks)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn update_inspection(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateInspectionRequest,
) -> Result<QaInspection> {
    let row = sqlx::query_as::<_, QaInspection>(
        r#"
        UPDATE qa_inspections SET
            title           = COALESCE($2, title),
            inspection_date = COALESCE($3, inspection_date),
            findings        = COALESCE($4, findings),
            conclusion      = COALESCE($5, conclusion),
            status          = COALESCE($6::qa_inspection_status, status),
            updated_at      = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.title)
    .bind(payload.inspection_date)
    .bind(&payload.findings)
    .bind(&payload.conclusion)
    .bind(payload.status.as_ref().map(|s| format!("{s:?}").to_lowercase()))
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_inspection_items(pool: &PgPool, inspection_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM qa_inspection_items WHERE inspection_id = $1")
        .bind(inspection_id)
        .execute(pool)
        .await?;

    Ok(())
}

// ============================================================
// 不符合事項
// ============================================================

pub async fn find_non_conformances(
    pool: &PgPool,
    params: &NcQuery,
) -> Result<Vec<QaNonConformanceWithDetails>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, QaNonConformanceWithDetails>(
        r#"
        SELECT nc.*,
               a.name AS assignee_name,
               c.name AS creator_name
        FROM qa_non_conformances nc
        LEFT JOIN users a ON a.id = nc.assignee_id
        JOIN      users c ON c.id = nc.created_by
        WHERE ($1::text IS NULL OR nc.severity::text = $1)
          AND ($2::text IS NULL OR nc.status::text = $2)
        ORDER BY nc.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&params.severity)
    .bind(&params.status)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_nc_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<QaNonConformanceWithDetails>> {
    let row = sqlx::query_as::<_, QaNonConformanceWithDetails>(
        r#"
        SELECT nc.*,
               a.name AS assignee_name,
               c.name AS creator_name
        FROM qa_non_conformances nc
        LEFT JOIN users a ON a.id = nc.assignee_id
        JOIN      users c ON c.id = nc.created_by
        WHERE nc.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_capa_by_nc(pool: &PgPool, nc_id: Uuid) -> Result<Vec<QaCapa>> {
    let rows = sqlx::query_as::<_, QaCapa>(
        "SELECT * FROM qa_capa WHERE nc_id = $1 ORDER BY created_at",
    )
    .bind(nc_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn insert_non_conformance(
    pool: &PgPool,
    number: &str,
    payload: &CreateNcRequest,
    created_by: Uuid,
) -> Result<QaNonConformance> {
    let row = sqlx::query_as::<_, QaNonConformance>(
        r#"
        INSERT INTO qa_non_conformances
            (nc_number, title, description, severity, source,
             related_inspection_id, assignee_id, due_date, created_by)
        VALUES ($1, $2, $3, $4::nc_severity, $5::nc_source, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(number)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(format!("{:?}", payload.severity).to_lowercase())
    .bind(format!("{:?}", payload.source).to_lowercase().replace("externalaudit", "external_audit").replace("selfreport", "self_report"))
    .bind(payload.related_inspection_id)
    .bind(payload.assignee_id)
    .bind(payload.due_date)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_non_conformance(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateNcRequest,
) -> Result<QaNonConformance> {
    let closed_at = payload.status.as_ref().and_then(|s| {
        matches!(s, crate::models::NcStatus::Closed).then_some(chrono::Utc::now())
    });

    let row = sqlx::query_as::<_, QaNonConformance>(
        r#"
        UPDATE qa_non_conformances SET
            title         = COALESCE($2, title),
            description   = COALESCE($3, description),
            assignee_id   = COALESCE($4, assignee_id),
            due_date      = COALESCE($5, due_date),
            status        = COALESCE($6::nc_status, status),
            root_cause    = COALESCE($7, root_cause),
            closure_notes = COALESCE($8, closure_notes),
            closed_at     = COALESCE($9, closed_at),
            updated_at    = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.assignee_id)
    .bind(payload.due_date)
    .bind(payload.status.as_ref().map(|s| format!("{s:?}").to_lowercase().replace("inprogress", "in_progress").replace("pendingverification", "pending_verification")))
    .bind(&payload.root_cause)
    .bind(&payload.closure_notes)
    .bind(closed_at)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn insert_capa(
    pool: &PgPool,
    nc_id: Uuid,
    payload: &CreateCapaRequest,
) -> Result<QaCapa> {
    let row = sqlx::query_as::<_, QaCapa>(
        r#"
        INSERT INTO qa_capa (nc_id, action_type, description, assignee_id, due_date)
        VALUES ($1, $2::capa_action_type, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(nc_id)
    .bind(format!("{:?}", payload.action_type).to_lowercase())
    .bind(&payload.description)
    .bind(payload.assignee_id)
    .bind(payload.due_date)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_capa(pool: &PgPool, id: Uuid, payload: &UpdateCapaRequest) -> Result<QaCapa> {
    let completed_at = payload.status.as_ref().and_then(|s| {
        matches!(s, crate::models::CapaStatus::Completed | crate::models::CapaStatus::Verified)
            .then_some(chrono::Utc::now())
    });

    let row = sqlx::query_as::<_, QaCapa>(
        r#"
        UPDATE qa_capa SET
            description  = COALESCE($2, description),
            assignee_id  = COALESCE($3, assignee_id),
            due_date     = COALESCE($4, due_date),
            status       = COALESCE($5::capa_status, status),
            completed_at = COALESCE($6, completed_at),
            updated_at   = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.description)
    .bind(payload.assignee_id)
    .bind(payload.due_date)
    .bind(payload.status.as_ref().map(|s| format!("{s:?}").to_lowercase().replace("inprogress", "in_progress")))
    .bind(completed_at)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

// ============================================================
// SOP 文件
// ============================================================

pub async fn find_sop_documents(
    pool: &PgPool,
    params: &SopQuery,
    current_user_id: Uuid,
) -> Result<Vec<QaSopDocumentWithAck>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, QaSopDocumentWithAck>(
        r#"
        SELECT s.*,
               u.name AS creator_name,
               EXISTS(
                   SELECT 1 FROM qa_sop_acknowledgments
                   WHERE sop_id = s.id AND user_id = $3
               ) AS acknowledged_by_me,
               (SELECT COUNT(*) FROM qa_sop_acknowledgments WHERE sop_id = s.id) AS ack_count
        FROM qa_sop_documents s
        JOIN users u ON u.id = s.created_by
        WHERE ($1::text IS NULL OR s.status::text = $1)
          AND ($2::text IS NULL OR s.category = $2)
        ORDER BY s.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(&params.status)
    .bind(&params.category)
    .bind(current_user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_sop_by_id(
    pool: &PgPool,
    id: Uuid,
    current_user_id: Uuid,
) -> Result<Option<QaSopDocumentWithAck>> {
    let row = sqlx::query_as::<_, QaSopDocumentWithAck>(
        r#"
        SELECT s.*,
               u.name AS creator_name,
               EXISTS(
                   SELECT 1 FROM qa_sop_acknowledgments
                   WHERE sop_id = s.id AND user_id = $2
               ) AS acknowledged_by_me,
               (SELECT COUNT(*) FROM qa_sop_acknowledgments WHERE sop_id = s.id) AS ack_count
        FROM qa_sop_documents s
        JOIN users u ON u.id = s.created_by
        WHERE s.id = $1
        "#,
    )
    .bind(id)
    .bind(current_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_sop(
    pool: &PgPool,
    number: &str,
    title: &str,
    version: &str,
    category: Option<&str>,
    file_path: Option<&str>,
    effective_date: Option<chrono::NaiveDate>,
    review_date: Option<chrono::NaiveDate>,
    description: Option<&str>,
    created_by: Uuid,
) -> Result<QaSopDocument> {
    let row = sqlx::query_as::<_, QaSopDocument>(
        r#"
        INSERT INTO qa_sop_documents
            (document_number, title, version, category, file_path,
             effective_date, review_date, description, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(number)
    .bind(title)
    .bind(version)
    .bind(category)
    .bind(file_path)
    .bind(effective_date)
    .bind(review_date)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_sop(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateSopRequest,
) -> Result<QaSopDocument> {
    let row = sqlx::query_as::<_, QaSopDocument>(
        r#"
        UPDATE qa_sop_documents SET
            title          = COALESCE($2, title),
            version        = COALESCE($3, version),
            category       = COALESCE($4, category),
            file_path      = COALESCE($5, file_path),
            effective_date = COALESCE($6, effective_date),
            review_date    = COALESCE($7, review_date),
            status         = COALESCE($8::sop_status, status),
            description    = COALESCE($9, description),
            updated_at     = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.title)
    .bind(&payload.version)
    .bind(&payload.category)
    .bind(&payload.file_path)
    .bind(payload.effective_date)
    .bind(payload.review_date)
    .bind(payload.status.as_ref().map(|s| format!("{s:?}").to_lowercase()))
    .bind(&payload.description)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_sop_acknowledgment(
    pool: &PgPool,
    sop_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO qa_sop_acknowledgments (sop_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (sop_id, user_id) DO UPDATE SET acknowledged_at = NOW()
        "#,
    )
    .bind(sop_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================
// 稽查排程
// ============================================================

pub async fn find_schedules(
    pool: &PgPool,
    params: &ScheduleQuery,
) -> Result<Vec<QaAuditSchedule>> {
    let rows = sqlx::query_as::<_, QaAuditSchedule>(
        r#"
        SELECT * FROM qa_audit_schedules
        WHERE ($1::int IS NULL OR year = $1)
          AND ($2::text IS NULL OR status::text = $2)
        ORDER BY year DESC, created_at DESC
        "#,
    )
    .bind(params.year)
    .bind(&params.status)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_schedule_by_id(pool: &PgPool, id: Uuid) -> Result<Option<QaAuditSchedule>> {
    let row = sqlx::query_as::<_, QaAuditSchedule>(
        "SELECT * FROM qa_audit_schedules WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_schedule_items(
    pool: &PgPool,
    schedule_id: Uuid,
) -> Result<Vec<QaScheduleItem>> {
    let rows = sqlx::query_as::<_, QaScheduleItem>(
        r#"
        SELECT si.*,
               u.name AS responsible_name
        FROM qa_schedule_items si
        LEFT JOIN users u ON u.id = si.responsible_person_id
        WHERE si.schedule_id = $1
        ORDER BY si.planned_date
        "#,
    )
    .bind(schedule_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn insert_schedule(
    pool: &PgPool,
    year: i32,
    title: &str,
    schedule_type: &str,
    description: Option<&str>,
    created_by: Uuid,
) -> Result<QaAuditSchedule> {
    let row = sqlx::query_as::<_, QaAuditSchedule>(
        r#"
        INSERT INTO qa_audit_schedules (year, title, schedule_type, description, created_by)
        VALUES ($1, $2, $3::qa_schedule_type, $4, $5)
        RETURNING *
        "#,
    )
    .bind(year)
    .bind(title)
    .bind(schedule_type)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn insert_schedule_items(
    pool: &PgPool,
    schedule_id: Uuid,
    items: &[CreateScheduleItemRequest],
) -> Result<()> {
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO qa_schedule_items
                (schedule_id, inspection_type, title, planned_date, responsible_person_id, notes)
            VALUES ($1, $2::qa_inspection_type, $3, $4, $5, $6)
            "#,
        )
        .bind(schedule_id)
        .bind(format!("{:?}", item.inspection_type).to_lowercase())
        .bind(&item.title)
        .bind(item.planned_date)
        .bind(item.responsible_person_id)
        .bind(&item.notes)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn update_schedule_item(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateScheduleItemRequest,
) -> Result<QaScheduleItem> {
    let row = sqlx::query_as::<_, QaScheduleItem>(
        r#"
        UPDATE qa_schedule_items SET
            actual_date           = COALESCE($2, actual_date),
            responsible_person_id = COALESCE($3, responsible_person_id),
            related_inspection_id = COALESCE($4, related_inspection_id),
            status                = COALESCE($5::qa_schedule_item_status, status),
            notes                 = COALESCE($6, notes),
            updated_at            = NOW()
        WHERE id = $1
        RETURNING *,
                  (SELECT name FROM users WHERE id = responsible_person_id) AS responsible_name
        "#,
    )
    .bind(id)
    .bind(payload.actual_date)
    .bind(payload.responsible_person_id)
    .bind(payload.related_inspection_id)
    .bind(payload.status.as_ref())
    .bind(&payload.notes)
    .fetch_one(pool)
    .await?;

    Ok(row)
}
