// HR Service
// 包含：Attendance, Overtime, Leave, Balances

use chrono::{NaiveDate, TimeZone, Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        AdjustBalanceRequest, AnnualLeaveBalanceView, AnnualLeaveEntitlement,
        AttendanceCorrectionRequest, AttendanceQuery, AttendanceRecord, AttendanceWithUser,
        BalanceSummary, CompTimeBalanceView, CreateAnnualLeaveRequest,
        CreateLeaveRequest, CreateOvertimeRequest, DashboardCalendarData, LeaveQuery, LeaveRequest,
        LeaveRequestWithUser, OvertimeQuery, OvertimeRecord, OvertimeWithUser, PaginatedResponse,
        TodayLeaveInfo, UpdateLeaveRequest, UpdateOvertimeRequest,
    },
    Result,
};

pub struct HrService;

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

    // ============================================
    // Overtime
    // ============================================

    pub async fn list_overtime(
        pool: &PgPool,
        query: &OvertimeQuery,
    ) -> Result<PaginatedResponse<OvertimeWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM overtime_records
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR status = $2)
              AND ($3::date IS NULL OR overtime_date >= $3)
              AND ($4::date IS NULL OR overtime_date <= $4)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE ($1::uuid IS NULL OR o.user_id = $1)
              AND ($2::text IS NULL OR o.status = $2)
              AND ($3::date IS NULL OR o.overtime_date >= $3)
              AND ($4::date IS NULL OR o.overtime_date <= $4)
            ORDER BY o.overtime_date DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_overtime(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
    ) -> Result<OvertimeWithUser> {
        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn create_overtime(
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        // 將 NaiveTime 結合 overtime_date 轉換為 DateTime<Utc>
        let start_datetime = Utc.from_utc_datetime(
            &payload.overtime_date.and_time(payload.start_time)
        );
        let end_datetime = Utc.from_utc_datetime(
            &payload.overtime_date.and_time(payload.end_time)
        );
        
        // 計算時數 (從 NaiveTime)
        let start_minutes = payload.start_time.hour() as i64 * 60 + payload.start_time.minute() as i64;
        let end_minutes = payload.end_time.hour() as i64 * 60 + payload.end_time.minute() as i64;
        let hours = (end_minutes - start_minutes) as f64 / 60.0;

        let multiplier = match payload.overtime_type.as_str() {
            "weekend" => 1.33,
            "holiday" => 1.66,
            _ => 1.0,
        };

        let comp_time_hours = hours * multiplier;
        let expires_at = payload.overtime_date + chrono::Duration::days(365);

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO overtime_records (
                id, user_id, overtime_date, start_time, end_time, hours,
                overtime_type, multiplier, comp_time_hours, comp_time_expires_at,
                status, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'draft', $11)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(payload.overtime_date)
        .bind(start_datetime)
        .bind(end_datetime)
        .bind(hours)
        .bind(&payload.overtime_type)
        .bind(multiplier)
        .bind(comp_time_hours)
        .bind(expires_at)
        .bind(&payload.reason)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_overtime(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
        payload: &UpdateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET start_time = COALESCE($2, start_time),
                end_time = COALESCE($3, end_time),
                overtime_type = COALESCE($4, overtime_type),
                reason = COALESCE($5, reason),
                updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            "#,
        )
        .bind(id)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(&payload.overtime_type)
        .bind(&payload.reason)
        .execute(pool)
        .await?;

        Self::get_overtime(pool, id, _current_user).await
    }

    pub async fn delete_overtime(pool: &PgPool, id: Uuid, _current_user: &CurrentUser) -> Result<()> {
        sqlx::query("DELETE FROM overtime_records WHERE id = $1 AND status = 'draft'")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn submit_overtime(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET status = 'pending', submitted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_overtime(pool, id, _current_user).await
    }

    pub async fn approve_overtime(
        pool: &PgPool,
        id: Uuid,
        approver_id: Uuid,
    ) -> Result<OvertimeWithUser> {
        let record: OvertimeRecord = sqlx::query_as(
            r#"
            UPDATE overtime_records
            SET status = 'approved', approved_by = $2, approved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(approver_id)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO comp_time_balances (
                id, user_id, overtime_record_id, original_hours, earned_date, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(record.user_id)
        .bind(record.id)
        .bind(record.comp_time_hours)
        .bind(record.overtime_date)
        .bind(record.comp_time_expires_at)
        .execute(pool)
        .await?;

        let result = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn reject_overtime(
        pool: &PgPool,
        id: Uuid,
        rejecter_id: Uuid,
        reason: &str,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET status = 'rejected', rejected_by = $2, rejected_at = NOW(), rejection_reason = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(rejecter_id)
        .bind(reason)
        .execute(pool)
        .await?;

        let result = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    // ============================================
    // Leave
    // ============================================

    pub async fn list_leaves(
        pool: &PgPool,
        query: &LeaveQuery,
        _current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<LeaveRequestWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // 如果是待審核查詢，篩選所有 PENDING 狀態的請假
        let is_pending_approval = query.pending_approval.unwrap_or(false);

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM leave_requests
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR status::text = $2)
              AND ($3::text IS NULL OR leave_type::text = $3)
              AND ($4::date IS NULL OR start_date >= $4)
              AND ($5::date IS NULL OR end_date <= $5)
              AND ($6::bool = false OR status::text LIKE 'PENDING%')
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(&query.leave_type)
        .bind(query.from)
        .bind(query.to)
        .bind(is_pending_approval)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, LeaveRequestWithUser>(
            r#"
            SELECT 
                l.id, l.user_id, u.email as user_email, u.display_name as user_name,
                l.proxy_user_id, proxy.display_name as proxy_user_name,
                l.leave_type::text as leave_type, l.start_date, l.end_date, l.total_days, l.reason,
                l.is_urgent, l.is_retroactive, l.status::text as status,
                l.current_approver_id, approver.display_name as current_approver_name,
                l.submitted_at, l.created_at
            FROM leave_requests l
            INNER JOIN users u ON l.user_id = u.id
            LEFT JOIN users proxy ON l.proxy_user_id = proxy.id
            LEFT JOIN users approver ON l.current_approver_id = approver.id
            WHERE ($1::uuid IS NULL OR l.user_id = $1)
              AND ($2::text IS NULL OR l.status::text = $2)
              AND ($3::text IS NULL OR l.leave_type::text = $3)
              AND ($4::date IS NULL OR l.start_date >= $4)
              AND ($5::date IS NULL OR l.end_date <= $5)
              AND ($6::bool = false OR l.status::text LIKE 'PENDING%')
            ORDER BY l.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(&query.leave_type)
        .bind(query.from)
        .bind(query.to)
        .bind(is_pending_approval)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_leave(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
    ) -> Result<LeaveRequest> {
        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn create_leave(
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateLeaveRequest,
    ) -> Result<LeaveRequest> {
        let id = Uuid::new_v4();
        
        // 處理 supporting_documents 轉為 JSON
        let supporting_docs = payload.supporting_documents.as_ref()
            .map(|docs| serde_json::json!(docs))
            .unwrap_or_else(|| serde_json::json!([]));
        
        // 理由處理：特休假可以為空，其他假別需要檢查
        let reason = payload.reason.clone().unwrap_or_default();
        
        sqlx::query(
            r#"
            INSERT INTO leave_requests (
                id, user_id, proxy_user_id, leave_type, start_date, end_date, start_time, end_time,
                total_days, total_hours, reason, supporting_documents, is_urgent, is_retroactive, status
            ) VALUES ($1, $2, $3, $4::leave_type, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'DRAFT'::leave_status)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(payload.proxy_user_id)
        .bind(&payload.leave_type)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(payload.total_days)
        .bind(payload.total_hours)
        .bind(&reason)
        .bind(&supporting_docs)
        .bind(payload.is_urgent.unwrap_or(false))
        .bind(payload.is_retroactive.unwrap_or(false))
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_leave(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
        payload: &UpdateLeaveRequest,
    ) -> Result<LeaveRequest> {
        sqlx::query(
            r#"
            UPDATE leave_requests
            SET start_date = COALESCE($2, start_date),
                end_date = COALESCE($3, end_date),
                start_time = COALESCE($4, start_time),
                end_time = COALESCE($5, end_time),
                total_days = COALESCE($6, total_days),
                total_hours = COALESCE($7, total_hours),
                reason = COALESCE($8, reason),
                proxy_user_id = COALESCE($9, proxy_user_id),
                updated_at = NOW()
            WHERE id = $1 AND status = 'DRAFT'::leave_status
            "#,
        )
        .bind(id)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(payload.total_days)
        .bind(payload.total_hours)
        .bind(&payload.reason)
        .bind(payload.proxy_user_id)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_leave(pool: &PgPool, id: Uuid, _current_user: &CurrentUser) -> Result<()> {
        sqlx::query("DELETE FROM leave_requests WHERE id = $1 AND status = 'DRAFT'::leave_status")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn submit_leave(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
    ) -> Result<LeaveRequest> {
        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = 'PENDING_L1'::leave_status, submitted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'DRAFT'::leave_status
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn approve_leave(
        pool: &PgPool,
        id: Uuid,
        approver_id: Uuid,
        comments: Option<&str>,
    ) -> Result<LeaveRequest> {
        let current: LeaveRequest = sqlx::query_as(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await?;

        let next_status = match current.status.as_str() {
            "PENDING_L1" => "APPROVED",
            "PENDING_L2" => "APPROVED",
            "PENDING_HR" => "APPROVED",
            "PENDING_GM" => "APPROVED",
            _ => return Err(AppError::Validation("無法核准此狀態的請假".to_string())),
        };

        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, 'APPROVE', $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(approver_id)
        .bind(&current.status)
        .bind(comments)
        .execute(pool)
        .await?;

        let approved_at = if next_status == "APPROVED" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = $2::leave_status, approved_at = $3, current_approver_id = NULL, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(next_status)
        .bind(approved_at)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn reject_leave(
        pool: &PgPool,
        id: Uuid,
        rejecter_id: Uuid,
        reason: &str,
    ) -> Result<LeaveRequest> {
        let current: LeaveRequest = sqlx::query_as(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, 'REJECT', $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(rejecter_id)
        .bind(&current.status)
        .bind(reason)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = 'REJECTED'::leave_status, rejected_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn cancel_leave(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
        reason: Option<&str>,
    ) -> Result<LeaveRequest> {
        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = 'CANCELLED'::leave_status, cancelled_at = NOW(), cancellation_reason = $2, updated_at = NOW()
            WHERE id = $1 AND status IN ('DRAFT'::leave_status, 'PENDING_L1'::leave_status, 'PENDING_L2'::leave_status, 'PENDING_HR'::leave_status, 'PENDING_GM'::leave_status, 'APPROVED'::leave_status)
            "#,
        )
        .bind(id)
        .bind(reason)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                comp_time_source_ids, annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    // ============================================
    // Balances
    // ============================================

    pub async fn get_annual_leave_balances(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AnnualLeaveBalanceView>> {
        let rows: Vec<(i32, f64, f64, f64, NaiveDate, i32)> = sqlx::query_as(
            r#"
            SELECT 
                entitlement_year,
                entitled_days::float8,
                used_days::float8,
                (entitled_days - used_days)::float8 as remaining_days,
                expires_at,
                (expires_at - CURRENT_DATE)::integer as days_until_expiry
            FROM annual_leave_entitlements
            WHERE user_id = $1 AND is_expired = false
            ORDER BY entitlement_year DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let balances = rows
            .into_iter()
            .map(|r| AnnualLeaveBalanceView {
                entitlement_year: r.0,
                entitled_days: r.1,
                used_days: r.2,
                remaining_days: r.3,
                expires_at: r.4,
                days_until_expiry: r.5,
            })
            .collect();

        Ok(balances)
    }

    pub async fn get_comp_time_balances(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<CompTimeBalanceView>> {
        let rows: Vec<(Uuid, NaiveDate, f64, f64, f64, NaiveDate, i32)> = sqlx::query_as(
            r#"
            SELECT 
                id,
                earned_date,
                original_hours::float8,
                used_hours::float8,
                (original_hours - used_hours)::float8 as remaining_hours,
                expires_at,
                (expires_at - CURRENT_DATE)::integer as days_until_expiry
            FROM comp_time_balances
            WHERE user_id = $1 AND is_expired = false
            ORDER BY expires_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let balances = rows
            .into_iter()
            .map(|r| CompTimeBalanceView {
                id: r.0,
                earned_date: r.1,
                original_hours: r.2,
                used_hours: r.3,
                remaining_hours: r.4,
                expires_at: r.5,
                days_until_expiry: r.6,
            })
            .collect();

        Ok(balances)
    }

    pub async fn get_balance_summary(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<BalanceSummary> {
        // 取得使用者名稱
        let user_name: (String,) = sqlx::query_as("SELECT display_name FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let annual: (f64, f64) = sqlx::query_as(
            r#"
            SELECT 
                COALESCE(SUM(entitled_days), 0)::float8,
                COALESCE(SUM(used_days), 0)::float8
            FROM annual_leave_entitlements
            WHERE user_id = $1 AND is_expired = false
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let comp: (f64, f64) = sqlx::query_as(
            r#"
            SELECT 
                COALESCE(SUM(original_hours), 0)::float8,
                COALESCE(SUM(used_hours), 0)::float8
            FROM comp_time_balances
            WHERE user_id = $1 AND is_expired = false
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        // 計算即將到期的餘額 (14天內)
        let expiring_annual: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(entitled_days - used_days), 0)::float8
            FROM annual_leave_entitlements
            WHERE user_id = $1 
              AND is_expired = false
              AND expires_at <= CURRENT_DATE + INTERVAL '14 days'
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let expiring_comp: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(original_hours - used_hours), 0)::float8
            FROM comp_time_balances
            WHERE user_id = $1 
              AND is_expired = false
              AND expires_at <= CURRENT_DATE + INTERVAL '14 days'
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(BalanceSummary {
            user_id,
            user_name: user_name.0,
            annual_leave_total: annual.0,
            annual_leave_used: annual.1,
            annual_leave_remaining: annual.0 - annual.1,
            comp_time_total: comp.0,
            comp_time_used: comp.1,
            comp_time_remaining: comp.0 - comp.1,
            expiring_soon_days: expiring_annual.0,
            expiring_soon_hours: expiring_comp.0,
        })
    }

    pub async fn create_annual_leave_entitlement(
        pool: &PgPool,
        creator_id: Uuid,
        payload: &CreateAnnualLeaveRequest,
    ) -> Result<AnnualLeaveEntitlement> {
        let id = Uuid::new_v4();
        // 計算到期日：發放年度 + 2 年
        let expires_at = NaiveDate::from_ymd_opt(payload.entitlement_year + 2, 12, 31)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(payload.entitlement_year + 2, 12, 30).unwrap());

        let record = sqlx::query_as::<_, AnnualLeaveEntitlement>(
            r#"
            INSERT INTO annual_leave_entitlements (
                id, user_id, entitlement_year, entitled_days, expires_at, 
                calculation_basis, notes, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(payload.user_id)
        .bind(payload.entitlement_year)
        .bind(payload.entitled_days)
        .bind(expires_at)
        .bind(&payload.calculation_basis)
        .bind(&payload.notes)
        .bind(creator_id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn adjust_annual_leave(
        pool: &PgPool,
        id: Uuid,
        _adjuster_id: Uuid,
        payload: &AdjustBalanceRequest,
    ) -> Result<AnnualLeaveEntitlement> {
        let record = sqlx::query_as::<_, AnnualLeaveEntitlement>(
            r#"
            UPDATE annual_leave_entitlements
            SET entitled_days = entitled_days + $2,
                notes = COALESCE(notes || E'\n', '') || $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(payload.adjustment_days)
        .bind(&payload.reason)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    // ============================================
    // Dashboard Calendar
    // ============================================

    pub async fn get_dashboard_calendar(pool: &PgPool) -> Result<DashboardCalendarData> {
        // 使用台灣時區取得今日日期
        let taiwan_tz = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let today = Utc::now().with_timezone(&taiwan_tz).date_naive();
        let upcoming_end = today + chrono::Duration::days(7);

        // 取得今日請假中的人 (已核准且日期涵蓋今天)
        let today_leaves_rows: Vec<(Uuid, String, String, NaiveDate, NaiveDate, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>)> = sqlx::query_as(
            r#"
            SELECT 
                l.user_id,
                u.display_name as user_name,
                l.leave_type::text as leave_type,
                l.start_date,
                l.end_date,
                l.start_time,
                l.end_time
            FROM leave_requests l
            INNER JOIN users u ON l.user_id = u.id
            WHERE l.status::text = 'APPROVED'
              AND l.start_date <= $1
              AND l.end_date >= $1
            ORDER BY u.display_name
            "#,
        )
        .bind(today)
        .fetch_all(pool)
        .await?;

        let today_leaves: Vec<TodayLeaveInfo> = today_leaves_rows
            .into_iter()
            .map(|(user_id, user_name, leave_type, start_date, end_date, start_time, end_time)| {
                let leave_type_display = match leave_type.as_str() {
                    "ANNUAL" => "特休假",
                    "PERSONAL" => "事假",
                    "SICK" => "病假",
                    "COMPENSATORY" => "補休假",
                    "MARRIAGE" => "婚假",
                    "BEREAVEMENT" => "喪假",
                    "MATERNITY" => "產假",
                    "PATERNITY" => "陪產假",
                    "MENSTRUAL" => "生理假",
                    "OFFICIAL" => "公假",
                    "UNPAID" => "無薪假",
                    _ => "請假",
                };
                let is_all_day = start_time.is_none() && end_time.is_none();
                TodayLeaveInfo {
                    user_id,
                    user_name,
                    leave_type,
                    leave_type_display: leave_type_display.to_string(),
                    is_all_day,
                    start_date,
                    end_date,
                }
            })
            .collect();

        // 取得近期請假 (未來7天內開始，已核准)
        let upcoming_leaves_rows: Vec<(Uuid, String, String, NaiveDate, NaiveDate, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>)> = sqlx::query_as(
            r#"
            SELECT 
                l.user_id,
                u.display_name as user_name,
                l.leave_type::text as leave_type,
                l.start_date,
                l.end_date,
                l.start_time,
                l.end_time
            FROM leave_requests l
            INNER JOIN users u ON l.user_id = u.id
            WHERE l.status::text = 'APPROVED'
              AND l.start_date > $1
              AND l.start_date <= $2
            ORDER BY l.start_date, u.display_name
            LIMIT 10
            "#,
        )
        .bind(today)
        .bind(upcoming_end)
        .fetch_all(pool)
        .await?;

        let upcoming_leaves: Vec<TodayLeaveInfo> = upcoming_leaves_rows
            .into_iter()
            .map(|(user_id, user_name, leave_type, start_date, end_date, start_time, end_time)| {
                let leave_type_display = match leave_type.as_str() {
                    "ANNUAL" => "特休假",
                    "PERSONAL" => "事假",
                    "SICK" => "病假",
                    "COMPENSATORY" => "補休假",
                    "MARRIAGE" => "婚假",
                    "BEREAVEMENT" => "喪假",
                    "MATERNITY" => "產假",
                    "PATERNITY" => "陪產假",
                    "MENSTRUAL" => "生理假",
                    "OFFICIAL" => "公假",
                    "UNPAID" => "無薪假",
                    _ => "請假",
                };
                let is_all_day = start_time.is_none() && end_time.is_none();
                TodayLeaveInfo {
                    user_id,
                    user_name,
                    leave_type,
                    leave_type_display: leave_type_display.to_string(),
                    is_all_day,
                    start_date,
                    end_date,
                }
            })
            .collect();

        // 嘗試取得 Google Calendar 事件 (如果已設定)
        let today_events = match crate::services::CalendarService::get_config(pool).await {
            Ok(config) if config.is_configured => {
                let client = crate::services::google_calendar::GoogleCalendarClient::new(&config.calendar_id);
                client.fetch_events(today, today).await.unwrap_or_default()
            }
            _ => vec![],
        };

        Ok(DashboardCalendarData {
            today,
            today_leaves,
            today_events,
            upcoming_leaves,
        })
    }
}

