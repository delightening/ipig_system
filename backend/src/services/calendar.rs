// Calendar Service
// Google Calendar 同步服務

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CalendarSyncConflict, CalendarSyncHistory, CalendarSyncStatus, ConflictQuery,
        ConflictWithDetails, ConnectCalendarRequest, EventSyncWithLeave, GoogleCalendarConfig,
        PaginatedResponse, SyncHistoryQuery, UpdateCalendarConfigRequest,
    },
    Result,
};

pub struct CalendarService;

impl CalendarService {
    // ============================================
    // Config
    // ============================================

    pub async fn get_config(pool: &PgPool) -> Result<GoogleCalendarConfig> {
        let config = sqlx::query_as::<_, GoogleCalendarConfig>(
            "SELECT * FROM google_calendar_config LIMIT 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(config)
    }

    pub async fn get_sync_status(pool: &PgPool) -> Result<CalendarSyncStatus> {
        let config = Self::get_config(pool).await.ok();

        let pending_syncs: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM calendar_event_sync WHERE sync_status IN ('pending', 'error')",
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let pending_conflicts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM calendar_sync_conflicts WHERE status = 'pending'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let recent_errors: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM calendar_event_sync WHERE error_count > 0 AND last_error_at > NOW() - INTERVAL '24 hours'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        Ok(CalendarSyncStatus {
            is_configured: config.as_ref().map(|c| c.is_configured).unwrap_or(false),
            sync_enabled: config.as_ref().map(|c| c.sync_enabled).unwrap_or(false),
            calendar_id: config
                .as_ref()
                .map(|c| c.calendar_id.clone())
                .unwrap_or_default(),
            last_sync_at: config.as_ref().and_then(|c| c.last_sync_at),
            last_sync_status: config.as_ref().and_then(|c| c.last_sync_status.clone()),
            next_sync_at: config.as_ref().and_then(|c| c.next_sync_at),
            pending_syncs: pending_syncs.0,
            pending_conflicts: pending_conflicts.0,
            recent_errors: recent_errors.0,
        })
    }

    pub async fn connect(
        pool: &PgPool,
        payload: &ConnectCalendarRequest,
    ) -> Result<GoogleCalendarConfig> {
        // Use INSERT with ON CONFLICT on expression index ((true))
        // This ensures the row exists and properly updates it
        let config = sqlx::query_as::<_, GoogleCalendarConfig>(
            r#"
            INSERT INTO google_calendar_config (id, calendar_id, auth_email, auth_method, is_configured, sync_enabled)
            VALUES (gen_random_uuid(), $1, $2, 'shared_account', true, true)
            ON CONFLICT ((true)) DO UPDATE SET
                calendar_id = EXCLUDED.calendar_id,
                auth_email = EXCLUDED.auth_email,
                auth_method = 'shared_account',
                is_configured = true,
                sync_enabled = true,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&payload.calendar_id)
        .bind(&payload.auth_email)
        .fetch_one(pool)
        .await?;

        Ok(config)
    }

    pub async fn disconnect(pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE google_calendar_config
            SET is_configured = false, sync_enabled = false, updated_at = NOW()
            "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update_config(
        pool: &PgPool,
        payload: &UpdateCalendarConfigRequest,
    ) -> Result<GoogleCalendarConfig> {
        let config = sqlx::query_as::<_, GoogleCalendarConfig>(
            r#"
            UPDATE google_calendar_config
            SET calendar_id = COALESCE($1, calendar_id),
                calendar_name = COALESCE($2, calendar_name),
                auth_email = COALESCE($3, auth_email),
                sync_enabled = COALESCE($4, sync_enabled),
                sync_schedule_morning = COALESCE($5, sync_schedule_morning),
                sync_schedule_evening = COALESCE($6, sync_schedule_evening),
                sync_approved_leaves = COALESCE($7, sync_approved_leaves),
                sync_overtime = COALESCE($8, sync_overtime),
                event_title_template = COALESCE($9, event_title_template),
                event_color_id = COALESCE($10, event_color_id),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&payload.calendar_id)
        .bind(&payload.calendar_name)
        .bind(&payload.auth_email)
        .bind(payload.sync_enabled)
        .bind(payload.sync_schedule_morning)
        .bind(payload.sync_schedule_evening)
        .bind(payload.sync_approved_leaves)
        .bind(payload.sync_overtime)
        .bind(&payload.event_title_template)
        .bind(&payload.event_color_id)
        .fetch_one(pool)
        .await?;

        Ok(config)
    }

    // ============================================
    // Sync
    // ============================================

    pub async fn trigger_sync(
        pool: &PgPool,
        triggered_by: Option<Uuid>,
    ) -> Result<CalendarSyncHistory> {
        // 建立同步歷史記錄
        let history = sqlx::query_as::<_, CalendarSyncHistory>(
            r#"
            INSERT INTO calendar_sync_history (id, job_type, triggered_by, started_at, status)
            VALUES ($1, 'manual', $2, NOW(), 'running')
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(triggered_by)
        .fetch_one(pool)
        .await?;

        // TODO: 實際執行 Google Calendar API 同步
        // 這裡先模擬完成
        let updated = sqlx::query_as::<_, CalendarSyncHistory>(
            r#"
            UPDATE calendar_sync_history
            SET status = 'completed',
                completed_at = NOW(),
                duration_ms = EXTRACT(MILLISECONDS FROM (NOW() - started_at))::int,
                progress_percentage = 100
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(history.id)
        .fetch_one(pool)
        .await?;

        // 更新 config 的 last_sync
        sqlx::query(
            r#"
            UPDATE google_calendar_config
            SET last_sync_at = NOW(),
                last_sync_status = 'success',
                updated_at = NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(updated)
    }

    pub async fn list_sync_history(
        pool: &PgPool,
        query: &SyncHistoryQuery,
    ) -> Result<PaginatedResponse<CalendarSyncHistory>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM calendar_sync_history
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR job_type = $2)
              AND ($3::date IS NULL OR started_at::date >= $3)
              AND ($4::date IS NULL OR started_at::date <= $4)
            "#,
        )
        .bind(&query.status)
        .bind(&query.job_type)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, CalendarSyncHistory>(
            r#"
            SELECT * FROM calendar_sync_history
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR job_type = $2)
              AND ($3::date IS NULL OR started_at::date >= $3)
              AND ($4::date IS NULL OR started_at::date <= $4)
            ORDER BY started_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(&query.status)
        .bind(&query.job_type)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn list_pending_syncs(pool: &PgPool) -> Result<Vec<EventSyncWithLeave>> {
        let events = sqlx::query_as::<_, EventSyncWithLeave>(
            r#"
            SELECT 
                s.id, s.leave_request_id, u.display_name as user_name,
                l.leave_type::text as leave_type, l.start_date, l.end_date,
                s.google_event_id, s.sync_status, s.last_error, s.error_count
            FROM calendar_event_sync s
            INNER JOIN leave_requests l ON s.leave_request_id = l.id
            INNER JOIN users u ON l.user_id = u.id
            WHERE s.sync_status IN ('pending', 'error')
            ORDER BY l.start_date
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    // ============================================
    // Conflicts
    // ============================================

    pub async fn list_conflicts(
        pool: &PgPool,
        query: &ConflictQuery,
    ) -> Result<PaginatedResponse<ConflictWithDetails>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM calendar_sync_conflicts
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::uuid IS NULL OR leave_request_id = $2)
            "#,
        )
        .bind(&query.status)
        .bind(query.leave_request_id)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, ConflictWithDetails>(
            r#"
            SELECT 
                c.id, c.leave_request_id, u.display_name as user_name,
                l.leave_type::text as leave_type, c.conflict_type, c.difference_summary,
                c.status, c.detected_at
            FROM calendar_sync_conflicts c
            LEFT JOIN leave_requests l ON c.leave_request_id = l.id
            LEFT JOIN users u ON l.user_id = u.id
            WHERE ($1::text IS NULL OR c.status = $1)
              AND ($2::uuid IS NULL OR c.leave_request_id = $2)
            ORDER BY c.detected_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&query.status)
        .bind(query.leave_request_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_conflict(pool: &PgPool, id: Uuid) -> Result<CalendarSyncConflict> {
        let conflict = sqlx::query_as::<_, CalendarSyncConflict>(
            "SELECT * FROM calendar_sync_conflicts WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(conflict)
    }

    pub async fn resolve_conflict(
        pool: &PgPool,
        id: Uuid,
        resolver_id: Uuid,
        resolution: &str,
        notes: Option<&str>,
    ) -> Result<CalendarSyncConflict> {
        let status = match resolution {
            "keep_ipig" => "resolved_keep_ipig",
            "accept_google" => "resolved_accept_google",
            "dismiss" => "dismissed",
            _ => "resolved",
        };

        let conflict = sqlx::query_as::<_, CalendarSyncConflict>(
            r#"
            UPDATE calendar_sync_conflicts
            SET status = $2,
                resolved_by = $3,
                resolved_at = NOW(),
                resolution_notes = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(resolver_id)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        // 如果是 keep_ipig，重新排隊同步
        if resolution == "keep_ipig" {
            if let Some(sync_id) = conflict.calendar_event_sync_id {
                sqlx::query(
                    r#"
                    UPDATE calendar_event_sync
                    SET sync_status = 'pending', sync_version = sync_version + 1, updated_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(sync_id)
                .execute(pool)
                .await?;
            }
        }

        Ok(conflict)
    }
}
