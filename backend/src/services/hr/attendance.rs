// HR 出勤管理

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        AttendanceCorrectionRequest, AttendanceQuery, AttendanceRecord,
        AttendanceWithUser, PaginatedResponse,
    },
    Result,
};

use super::HrService;

impl HrService {
    // ============================================
    // Attendance
    // ============================================

    pub async fn list_attendance(
        pool: &PgPool,
        query: &AttendanceQuery,
    ) -> Result<PaginatedResponse<AttendanceWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM attendance_records
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::date IS NULL OR work_date >= $2)
              AND ($3::date IS NULL OR work_date <= $3)
              AND ($4::text IS NULL OR status = $4)
            "#,
        )
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, AttendanceWithUser>(
            r#"
            SELECT 
                a.id, a.user_id, u.email as user_email, u.display_name as user_name,
                a.work_date, a.clock_in_time, a.clock_out_time,
                a.regular_hours, a.overtime_hours, a.status, a.remark, a.is_corrected
            FROM attendance_records a
            INNER JOIN users u ON a.user_id = u.id
            WHERE ($1::uuid IS NULL OR a.user_id = $1)
              AND ($2::date IS NULL OR a.work_date >= $2)
              AND ($3::date IS NULL OR a.work_date <= $3)
              AND ($4::text IS NULL OR a.status = $4)
            ORDER BY a.work_date DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn clock_in(
        pool: &PgPool,
        user_id: Uuid,
        source: Option<&str>,
        ip: Option<&str>,
    ) -> Result<AttendanceRecord> {
        // 使用台灣時區 (UTC+8) 的日期，而不是 UTC 日期
        // 這樣當使用者在凌晨打卡時，work_date 會是正確的本地日期
        let taipei_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let today = Utc::now().with_timezone(&taipei_offset).date_naive();

        let existing: Option<AttendanceRecord> = sqlx::query_as(
            "SELECT * FROM attendance_records WHERE user_id = $1 AND work_date = $2",
        )
        .bind(user_id)
        .bind(today)
        .fetch_optional(pool)
        .await?;

        if let Some(record) = existing {
            if record.clock_in_time.is_some() {
                return Err(AppError::Validation("今天已經打卡上班".to_string()));
            }
        }

        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            INSERT INTO attendance_records (id, user_id, work_date, clock_in_time, clock_in_source, clock_in_ip, status)
            VALUES ($1, $2, $3, NOW(), $4, $5::inet, 'normal')
            ON CONFLICT (user_id, work_date) DO UPDATE SET
                clock_in_time = NOW(),
                clock_in_source = $4,
                clock_in_ip = $5::inet,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(today)
        .bind(source.unwrap_or("web"))
        .bind(ip)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn clock_out(
        pool: &PgPool,
        user_id: Uuid,
        source: Option<&str>,
        ip: Option<&str>,
    ) -> Result<AttendanceRecord> {
        // 使用台灣時區 (UTC+8) 的日期，與 clock_in 保持一致
        let taipei_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let today = Utc::now().with_timezone(&taipei_offset).date_naive();

        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET clock_out_time = NOW(),
                clock_out_source = $3,
                clock_out_ip = $4::inet,
                regular_hours = EXTRACT(EPOCH FROM (NOW() - clock_in_time)) / 3600,
                updated_at = NOW()
            WHERE user_id = $1 AND work_date = $2
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(today)
        .bind(source.unwrap_or("web"))
        .bind(ip)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Validation("請先打卡上班".to_string()))?;

        Ok(record)
    }

    pub async fn correct_attendance(
        pool: &PgPool,
        id: Uuid,
        corrector_id: Uuid,
        payload: &AttendanceCorrectionRequest,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE attendance_records
            SET original_clock_in = clock_in_time,
                original_clock_out = clock_out_time,
                clock_in_time = COALESCE($2, clock_in_time),
                clock_out_time = COALESCE($3, clock_out_time),
                is_corrected = true,
                corrected_by = $4,
                corrected_at = NOW(),
                correction_reason = $5,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(payload.clock_in_time)
        .bind(payload.clock_out_time)
        .bind(corrector_id)
        .bind(&payload.reason)
        .execute(pool)
        .await?;

        Ok(())
    }
}
