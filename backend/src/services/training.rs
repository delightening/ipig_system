// 人員訓練紀錄 Service (GLP 合規)

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateTrainingRecordRequest, PaginatedResponse, TrainingQuery,
        TrainingRecord, TrainingRecordWithUser, UpdateTrainingRecordRequest,
    },
    Result,
};

pub struct TrainingService;

impl TrainingService {
    pub async fn list(
        pool: &PgPool,
        query: &TrainingQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<TrainingRecordWithUser>> {
        let has_view = current_user.has_permission("training.view");
        let has_manage = current_user.has_permission("training.manage");
        if !has_view && !has_manage {
            return Err(AppError::Forbidden("無權查看訓練紀錄".into()));
        }

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let user_filter = if has_manage {
            query.user_id
        } else {
            Some(current_user.id)
        };

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM training_records tr
            INNER JOIN users u ON tr.user_id = u.id
            WHERE ($1::uuid IS NULL OR tr.user_id = $1)
              AND ($2::text IS NULL OR tr.course_name ILIKE '%' || $2 || '%')
            "#,
        )
        .bind(user_filter)
        .bind(query.course_name.as_deref())
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, TrainingRecordWithUser>(
            r#"
            SELECT
                tr.id, tr.user_id, u.email as user_email, u.display_name as user_name,
                tr.course_name, tr.completed_at, tr.expires_at, tr.notes, tr.created_at
            FROM training_records tr
            INNER JOIN users u ON tr.user_id = u.id
            WHERE ($1::uuid IS NULL OR tr.user_id = $1)
              AND ($2::text IS NULL OR tr.course_name ILIKE '%' || $2 || '%')
            ORDER BY tr.completed_at DESC, tr.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_filter)
        .bind(query.course_name.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get(pool: &PgPool, id: Uuid, current_user: &CurrentUser) -> Result<TrainingRecord> {
        let record = sqlx::query_as::<_, TrainingRecord>(
            "SELECT * FROM training_records WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("訓練紀錄不存在".into()))?;

        let has_manage = current_user.has_permission("training.manage");
        if !has_manage && record.user_id != current_user.id {
            return Err(AppError::Forbidden("無權查看此訓練紀錄".into()));
        }

        Ok(record)
    }

    pub async fn create(
        pool: &PgPool,
        payload: &CreateTrainingRecordRequest,
        current_user: &CurrentUser,
    ) -> Result<TrainingRecord> {
        if !current_user.has_permission("training.manage") {
            return Err(AppError::Forbidden("無權新增訓練紀錄".into()));
        }
        payload.validate()?;

        let record = sqlx::query_as::<_, TrainingRecord>(
            r#"
            INSERT INTO training_records (user_id, course_name, completed_at, expires_at, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(payload.user_id)
        .bind(&payload.course_name)
        .bind(payload.completed_at)
        .bind(payload.expires_at)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateTrainingRecordRequest,
        current_user: &CurrentUser,
    ) -> Result<TrainingRecord> {
        if !current_user.has_permission("training.manage") {
            return Err(AppError::Forbidden("無權編輯訓練紀錄".into()));
        }
        payload.validate()?;

        let existing = sqlx::query_as::<_, TrainingRecord>(
            "SELECT * FROM training_records WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("訓練紀錄不存在".into()))?;

        let course_name = payload
            .course_name
            .as_deref()
            .unwrap_or(&existing.course_name);
        let completed_at = payload.completed_at.unwrap_or(existing.completed_at);
        let expires_at = payload.expires_at.or(existing.expires_at);
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();

        let record = sqlx::query_as::<_, TrainingRecord>(
            r#"
            UPDATE training_records
            SET course_name = $2, completed_at = $3, expires_at = $4, notes = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(course_name)
        .bind(completed_at)
        .bind(expires_at)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("training.manage") {
            return Err(AppError::Forbidden("無權刪除訓練紀錄".into()));
        }

        let result = sqlx::query("DELETE FROM training_records WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("訓練紀錄不存在".into()));
        }

        Ok(())
    }
}
