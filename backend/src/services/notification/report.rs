// 定期報表相關

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateScheduledReportRequest, PaginatedResponse, ReportHistory,
        ScheduledReport, UpdateScheduledReportRequest,
    },
};

use super::NotificationService;

impl NotificationService {
    /// 取得定期報表列表
    pub async fn list_scheduled_reports(&self) -> Result<Vec<ScheduledReport>, AppError> {
        let reports: Vec<ScheduledReport> = sqlx::query_as(
            r#"
            SELECT * FROM scheduled_reports
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// 取得單一定期報表
    pub async fn get_scheduled_report(&self, id: Uuid) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"SELECT * FROM scheduled_reports WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Scheduled report not found".to_string()))?;

        Ok(report)
    }

    /// 建立定期報表
    pub async fn create_scheduled_report(
        &self,
        request: CreateScheduledReportRequest,
        created_by: Uuid,
    ) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"
            INSERT INTO scheduled_reports 
                (id, report_type, schedule_type, day_of_week, day_of_month, 
                 hour_of_day, parameters, recipients, created_by)
            VALUES 
                (gen_random_uuid(), $1::report_type, $2::schedule_type, $3, $4, 
                 $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&request.report_type)
        .bind(&request.schedule_type)
        .bind(request.day_of_week)
        .bind(request.day_of_month)
        .bind(request.hour_of_day)
        .bind(&request.parameters)
        .bind(&request.recipients)
        .bind(created_by)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// 更新定期報表
    pub async fn update_scheduled_report(
        &self,
        id: Uuid,
        request: UpdateScheduledReportRequest,
    ) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"
            UPDATE scheduled_reports
            SET 
                day_of_week = COALESCE($2, day_of_week),
                day_of_month = COALESCE($3, day_of_month),
                hour_of_day = COALESCE($4, hour_of_day),
                parameters = COALESCE($5, parameters),
                recipients = COALESCE($6, recipients),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(request.day_of_week)
        .bind(request.day_of_month)
        .bind(request.hour_of_day)
        .bind(&request.parameters)
        .bind(&request.recipients)
        .bind(request.is_active)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// 刪除定期報表
    pub async fn delete_scheduled_report(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"DELETE FROM scheduled_reports WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Scheduled report not found".to_string()));
        }

        Ok(())
    }

    /// 取得報表歷史記錄
    pub async fn list_report_history(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<ReportHistory>, AppError> {
        let offset = (page - 1) * per_page;

        let reports: Vec<ReportHistory> = sqlx::query_as(
            r#"
            SELECT * FROM report_history
            ORDER BY generated_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM report_history"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(reports, total.0, page, per_page))
    }

    /// 取得單一報表歷史
    pub async fn get_report_history(&self, id: Uuid) -> Result<ReportHistory, AppError> {
        let report: ReportHistory = sqlx::query_as(
            r#"SELECT * FROM report_history WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Report history not found".to_string()))?;

        Ok(report)
    }
}
