// QA 計畫管理 Service（稽查報告、不符合事項、SOP 文件、稽查排程）

use chrono::Datelike;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateCapaRequest, CreateInspectionRequest, CreateNcRequest, CreateScheduleRequest,
        CreateSopRequest, InspectionQuery, NcDetail, NcQuery, QaCapa, QaInspectionDetail,
        QaInspectionWithInspector, QaNonConformanceWithDetails, QaScheduleDetail,
        QaSopDocumentWithAck, ScheduleQuery, SopQuery, UpdateCapaRequest, UpdateInspectionRequest,
        UpdateNcRequest, UpdateScheduleItemRequest, UpdateSopRequest,
    },
    repositories::qa_plan as repo,
    Result,
};

pub struct QaPlanService;

impl QaPlanService {
    // ============================================================
    // 稽查報告
    // ============================================================

    pub async fn list_inspections(
        pool: &PgPool,
        params: &InspectionQuery,
    ) -> Result<Vec<QaInspectionWithInspector>> {
        repo::find_inspections(pool, params).await
    }

    pub async fn get_inspection(pool: &PgPool, id: Uuid) -> Result<QaInspectionDetail> {
        let inspection = repo::find_inspection_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("稽查報告不存在".into()))?;

        let items = repo::find_inspection_items(pool, id).await?;

        Ok(QaInspectionDetail { inspection, items })
    }

    pub async fn create_inspection(
        pool: &PgPool,
        payload: &CreateInspectionRequest,
        inspector_id: Uuid,
    ) -> Result<QaInspectionDetail> {
        let number = Self::generate_inspection_number(pool).await?;
        let inspection_type = format!("{:?}", payload.inspection_type).to_lowercase();

        let inspection = repo::insert_inspection(
            pool,
            &number,
            &payload.title,
            &inspection_type,
            payload.inspection_date,
            inspector_id,
            payload.related_entity_type.as_deref(),
            payload.related_entity_id,
            payload.findings.as_deref(),
            payload.conclusion.as_deref(),
        )
        .await?;

        repo::insert_inspection_items(pool, inspection.id, &payload.items).await?;

        let detail = Self::get_inspection(pool, inspection.id).await?;
        Ok(detail)
    }

    pub async fn update_inspection(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateInspectionRequest,
    ) -> Result<QaInspectionDetail> {
        repo::find_inspection_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("稽查報告不存在".into()))?;

        repo::update_inspection(pool, id, payload).await?;

        if let Some(items) = &payload.items {
            repo::delete_inspection_items(pool, id).await?;
            repo::insert_inspection_items(pool, id, items).await?;
        }

        Self::get_inspection(pool, id).await
    }

    async fn generate_inspection_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().year();
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM qa_inspections WHERE EXTRACT(YEAR FROM created_at) = $1",
        )
        .bind(year)
        .fetch_one(pool)
        .await?;

        Ok(format!("QAI-{year}-{:04}", count.0 + 1))
    }

    // ============================================================
    // 不符合事項
    // ============================================================

    pub async fn list_non_conformances(
        pool: &PgPool,
        params: &NcQuery,
    ) -> Result<Vec<QaNonConformanceWithDetails>> {
        repo::find_non_conformances(pool, params).await
    }

    pub async fn get_non_conformance(pool: &PgPool, id: Uuid) -> Result<NcDetail> {
        let nc = repo::find_nc_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("不符合事項不存在".into()))?;

        let capa = repo::find_capa_by_nc(pool, id).await?;

        Ok(NcDetail { nc, capa })
    }

    pub async fn create_non_conformance(
        pool: &PgPool,
        payload: &CreateNcRequest,
        created_by: Uuid,
    ) -> Result<NcDetail> {
        let number = Self::generate_nc_number(pool).await?;

        let nc = repo::insert_non_conformance(pool, &number, payload, created_by).await?;
        Self::get_non_conformance(pool, nc.id).await
    }

    pub async fn update_non_conformance(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateNcRequest,
    ) -> Result<NcDetail> {
        repo::find_nc_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("不符合事項不存在".into()))?;

        repo::update_non_conformance(pool, id, payload).await?;
        Self::get_non_conformance(pool, id).await
    }

    pub async fn create_capa(
        pool: &PgPool,
        nc_id: Uuid,
        payload: &CreateCapaRequest,
    ) -> Result<QaCapa> {
        repo::find_nc_by_id(pool, nc_id)
            .await?
            .ok_or(AppError::NotFound("不符合事項不存在".into()))?;

        repo::insert_capa(pool, nc_id, payload).await
    }

    pub async fn update_capa(
        pool: &PgPool,
        nc_id: Uuid,
        capa_id: Uuid,
        payload: &UpdateCapaRequest,
    ) -> Result<QaCapa> {
        let capas = repo::find_capa_by_nc(pool, nc_id).await?;
        if !capas.iter().any(|c| c.id == capa_id) {
            return Err(AppError::NotFound("CAPA 不存在".into()));
        }

        repo::update_capa(pool, capa_id, payload).await
    }

    async fn generate_nc_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().year();
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM qa_non_conformances WHERE EXTRACT(YEAR FROM created_at) = $1",
        )
        .bind(year)
        .fetch_one(pool)
        .await?;

        Ok(format!("NC-{year}-{:04}", count.0 + 1))
    }

    // ============================================================
    // SOP 文件
    // ============================================================

    pub async fn list_sop_documents(
        pool: &PgPool,
        params: &SopQuery,
        current_user_id: Uuid,
    ) -> Result<Vec<QaSopDocumentWithAck>> {
        repo::find_sop_documents(pool, params, current_user_id).await
    }

    pub async fn get_sop_document(
        pool: &PgPool,
        id: Uuid,
        current_user_id: Uuid,
    ) -> Result<QaSopDocumentWithAck> {
        repo::find_sop_by_id(pool, id, current_user_id)
            .await?
            .ok_or(AppError::NotFound("SOP 文件不存在".into()))
    }

    pub async fn create_sop(
        pool: &PgPool,
        payload: &CreateSopRequest,
        created_by: Uuid,
    ) -> Result<QaSopDocumentWithAck> {
        let number = Self::generate_sop_number(pool).await?;

        let sop = repo::insert_sop(
            pool,
            &number,
            &payload.title,
            &payload.version,
            payload.category.as_deref(),
            payload.file_path.as_deref(),
            payload.effective_date,
            payload.review_date,
            payload.description.as_deref(),
            created_by,
        )
        .await?;

        Self::get_sop_document(pool, sop.id, created_by).await
    }

    pub async fn update_sop(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateSopRequest,
        current_user_id: Uuid,
    ) -> Result<QaSopDocumentWithAck> {
        repo::find_sop_by_id(pool, id, current_user_id)
            .await?
            .ok_or(AppError::NotFound("SOP 文件不存在".into()))?;

        repo::update_sop(pool, id, payload).await?;
        Self::get_sop_document(pool, id, current_user_id).await
    }

    pub async fn acknowledge_sop(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<QaSopDocumentWithAck> {
        repo::find_sop_by_id(pool, id, user_id)
            .await?
            .ok_or(AppError::NotFound("SOP 文件不存在".into()))?;

        repo::upsert_sop_acknowledgment(pool, id, user_id).await?;
        Self::get_sop_document(pool, id, user_id).await
    }

    async fn generate_sop_number(pool: &PgPool) -> Result<String> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM qa_sop_documents")
            .fetch_one(pool)
            .await?;

        Ok(format!("SOP-{:04}", count.0 + 1))
    }

    // ============================================================
    // 稽查排程
    // ============================================================

    pub async fn list_schedules(
        pool: &PgPool,
        params: &ScheduleQuery,
    ) -> Result<Vec<QaAuditSchedule>> {
        repo::find_schedules(pool, params).await
    }

    pub async fn get_schedule(pool: &PgPool, id: Uuid) -> Result<QaScheduleDetail> {
        let schedule = repo::find_schedule_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("稽查排程不存在".into()))?;

        let items = repo::find_schedule_items(pool, id).await?;

        Ok(QaScheduleDetail { schedule, items })
    }

    pub async fn create_schedule(
        pool: &PgPool,
        payload: &CreateScheduleRequest,
        created_by: Uuid,
    ) -> Result<QaScheduleDetail> {
        let schedule_type = format!("{:?}", payload.schedule_type)
            .to_lowercase()
            .replace("adhoc", "ad_hoc");

        let schedule = repo::insert_schedule(
            pool,
            payload.year,
            &payload.title,
            &schedule_type,
            payload.description.as_deref(),
            created_by,
        )
        .await?;

        repo::insert_schedule_items(pool, schedule.id, &payload.items).await?;

        Self::get_schedule(pool, schedule.id).await
    }

    pub async fn update_schedule_item(
        pool: &PgPool,
        schedule_id: Uuid,
        item_id: Uuid,
        payload: &UpdateScheduleItemRequest,
    ) -> Result<QaScheduleDetail> {
        repo::find_schedule_by_id(pool, schedule_id)
            .await?
            .ok_or(AppError::NotFound("稽查排程不存在".into()))?;

        let items = repo::find_schedule_items(pool, schedule_id).await?;
        if !items.iter().any(|i| i.id == item_id) {
            return Err(AppError::NotFound("排程項目不存在".into()));
        }

        repo::update_schedule_item(pool, item_id, payload).await?;
        Self::get_schedule(pool, schedule_id).await
    }
}

use crate::models::QaAuditSchedule;
