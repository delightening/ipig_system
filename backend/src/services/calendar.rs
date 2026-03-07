// Calendar Service
// Google Calendar 同步服務

use chrono::NaiveDate;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    models::{
        CalendarSyncConflict, CalendarSyncHistory, CalendarSyncStatus, ConflictQuery,
        ConflictWithDetails, ConnectCalendarRequest, EventSyncWithLeave, GoogleCalendarConfig,
        PaginatedResponse, SyncHistoryQuery, UpdateCalendarConfigRequest,
    },
    Result,
};

/// 用於同步查詢的內部結構
#[derive(Debug, FromRow)]
struct PendingSyncItem {
    id: Uuid,
    leave_request_id: Uuid,
    google_event_id: Option<String>,
    user_name: String,
    proxy_user_name: Option<String>,
    leave_type_str: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    reason: Option<String>,
}

pub struct CalendarService;

/// 同步操作的統計結果
struct SyncStats {
    created: i32,
    updated: i32,
    deleted: i32,
    errors: i32,
    error_messages: Vec<String>,
}

impl SyncStats {
    fn new() -> Self {
        Self {
            created: 0,
            updated: 0,
            deleted: 0,
            errors: 0,
            error_messages: Vec::new(),
        }
    }
}

/// 假別代碼轉換為中文顯示名稱
fn get_leave_type_display(leave_type: &str) -> &'static str {
    match leave_type {
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
        _ => "請假",
    }
}

/// 建立日曆事件標題：[假別] 人員（代理人）
fn build_event_summary(user_name: &str, proxy_name: Option<&str>, leave_type: &str) -> String {
    let display = get_leave_type_display(leave_type);
    match proxy_name {
        Some(proxy) => format!("[{}] {}（{}）", display, user_name, proxy),
        None => format!("[{}] {}", display, user_name),
    }
}

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

        let pending_conflicts: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM calendar_sync_conflicts WHERE status = 'pending'")
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
        use crate::services::google_calendar::GoogleCalendarClient;

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

        // 取得 Calendar 設定
        let config = Self::get_config(pool).await?;
        if !config.is_configured {
            return Err(crate::error::AppError::Validation(
                "Google Calendar 尚未設定".to_string(),
            ));
        }

        let client = GoogleCalendarClient::new(&config.calendar_id);
        let mut stats = SyncStats::new();

        // 依序處理 create / update / delete
        Self::process_pending_creates(pool, &client, &config, &mut stats).await;
        Self::process_pending_updates(pool, &client, &config, &mut stats).await;
        Self::process_pending_deletes(pool, &client, &mut stats).await;

        // 更新歷史記錄為完成
        let status = if stats.errors > 0 {
            "completed_with_errors"
        } else {
            "completed"
        };
        let error_json = serde_json::to_value(&stats.error_messages).unwrap_or_default();

        let updated = sqlx::query_as::<_, CalendarSyncHistory>(
            r#"
            UPDATE calendar_sync_history
            SET status = $2,
                completed_at = NOW(),
                duration_ms = EXTRACT(MILLISECONDS FROM (NOW() - started_at))::int,
                progress_percentage = 100,
                events_created = $3,
                events_updated = $4,
                events_deleted = $5,
                errors_count = $6,
                error_messages = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(history.id)
        .bind(status)
        .bind(stats.created)
        .bind(stats.updated)
        .bind(stats.deleted)
        .bind(stats.errors)
        .bind(error_json)
        .fetch_one(pool)
        .await?;

        // 更新 config 的 last_sync
        let last_status = if stats.errors > 0 {
            "partial"
        } else {
            "success"
        };
        sqlx::query(
            r#"
            UPDATE google_calendar_config
            SET last_sync_at = NOW(),
                last_sync_status = $1,
                last_sync_events_pushed = $2,
                last_sync_events_pulled = 0,
                updated_at = NOW()
            "#,
        )
        .bind(last_status)
        .bind(stats.created + stats.updated)
        .execute(pool)
        .await?;

        Ok(updated)
    }

    /// 處理待建立的日曆事件
    async fn process_pending_creates(
        pool: &PgPool,
        client: &impl crate::services::google_calendar::CalendarApi,
        config: &GoogleCalendarConfig,
        stats: &mut SyncStats,
    ) {
        use crate::services::google_calendar::NewCalendarEvent;

        let pending: Vec<PendingSyncItem> = sqlx::query_as(
            r#"
            SELECT 
                s.id, s.leave_request_id, s.google_event_id,
                u.display_name as user_name,
                proxy.display_name as proxy_user_name,
                l.leave_type::text as leave_type_str, 
                l.start_date, l.end_date,
                l.reason
            FROM calendar_event_sync s
            INNER JOIN leave_requests l ON s.leave_request_id = l.id
            INNER JOIN users u ON l.user_id = u.id
            LEFT JOIN users proxy ON l.proxy_user_id = proxy.id
            WHERE s.sync_status = 'pending_create'
            "#,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for item in pending {
            let summary = build_event_summary(
                &item.user_name,
                item.proxy_user_name.as_deref(),
                &item.leave_type_str,
            );
            let new_event = NewCalendarEvent {
                summary,
                description: item.reason.clone(),
                start_date: item.start_date,
                end_date: item.end_date,
                all_day: true,
                color_id: config.event_color_id.clone(),
            };

            match client.create_event(new_event).await {
                Ok(created) => {
                    if let Err(e) = sqlx::query(
                        r#"
                        UPDATE calendar_event_sync
                        SET google_event_id = $1,
                            google_event_link = $2,
                            google_event_etag = $3,
                            sync_status = 'synced',
                            sync_version = sync_version + 1,
                            google_updated_at = NOW(),
                            updated_at = NOW()
                        WHERE id = $4
                        "#,
                    )
                    .bind(&created.id)
                    .bind(&created.html_link)
                    .bind(&created.etag)
                    .bind(item.id)
                    .execute(pool)
                    .await
                    {
                        tracing::warn!("DB 操作失敗: {e}");
                    }
                    stats.created += 1;
                }
                Err(e) => {
                    let err_msg =
                        format!("建立事件失敗 (leave_id={}): {}", item.leave_request_id, e);
                    stats.error_messages.push(err_msg);
                    Self::mark_sync_error(pool, item.id, &e.to_string()).await;
                    stats.errors += 1;
                }
            }
        }
    }

    /// 處理待更新的日曆事件
    async fn process_pending_updates(
        pool: &PgPool,
        client: &impl crate::services::google_calendar::CalendarApi,
        config: &GoogleCalendarConfig,
        stats: &mut SyncStats,
    ) {
        use crate::services::google_calendar::NewCalendarEvent;

        let pending: Vec<PendingSyncItem> = sqlx::query_as(
            r#"
            SELECT 
                s.id, s.leave_request_id, s.google_event_id,
                u.display_name as user_name,
                proxy.display_name as proxy_user_name,
                l.leave_type::text as leave_type_str, 
                l.start_date, l.end_date,
                l.reason
            FROM calendar_event_sync s
            INNER JOIN leave_requests l ON s.leave_request_id = l.id
            INNER JOIN users u ON l.user_id = u.id
            LEFT JOIN users proxy ON l.proxy_user_id = proxy.id
            WHERE s.sync_status = 'pending_update' AND s.google_event_id IS NOT NULL
            "#,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for item in pending {
            if let Some(ref google_event_id) = item.google_event_id {
                let summary = build_event_summary(
                    &item.user_name,
                    item.proxy_user_name.as_deref(),
                    &item.leave_type_str,
                );
                let update_event = NewCalendarEvent {
                    summary,
                    description: item.reason.clone(),
                    start_date: item.start_date,
                    end_date: item.end_date,
                    all_day: true,
                    color_id: config.event_color_id.clone(),
                };

                match client.update_event(google_event_id, update_event).await {
                    Ok(updated) => {
                        if let Err(e) = sqlx::query(
                            r#"
                            UPDATE calendar_event_sync
                            SET google_event_etag = $1,
                                sync_status = 'synced',
                                sync_version = sync_version + 1,
                                google_updated_at = NOW(),
                                updated_at = NOW()
                            WHERE id = $2
                            "#,
                        )
                        .bind(&updated.etag)
                        .bind(item.id)
                        .execute(pool)
                        .await
                        {
                            tracing::warn!("DB 操作失敗: {e}");
                        }
                        stats.updated += 1;
                    }
                    Err(e) => {
                        let err_msg =
                            format!("更新事件失敗 (leave_id={}): {}", item.leave_request_id, e);
                        stats.error_messages.push(err_msg);
                        Self::mark_sync_error(pool, item.id, &e.to_string()).await;
                        stats.errors += 1;
                    }
                }
            }
        }
    }

    /// 處理待刪除的日曆事件
    async fn process_pending_deletes(
        pool: &PgPool,
        client: &impl crate::services::google_calendar::CalendarApi,
        stats: &mut SyncStats,
    ) {
        let pending: Vec<(Uuid, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, google_event_id 
            FROM calendar_event_sync 
            WHERE sync_status = 'pending_delete' AND google_event_id IS NOT NULL
            "#,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (sync_id, google_event_id) in pending {
            if let Some(ref event_id) = google_event_id {
                match client.delete_event(event_id).await {
                    Ok(_) => {
                        if let Err(e) = sqlx::query(
                            r#"
                            UPDATE calendar_event_sync
                            SET sync_status = 'deleted',
                                google_event_id = NULL,
                                updated_at = NOW()
                            WHERE id = $1
                            "#,
                        )
                        .bind(sync_id)
                        .execute(pool)
                        .await
                        {
                            tracing::warn!("DB 操作失敗: {e}");
                        }
                        stats.deleted += 1;
                    }
                    Err(e) => {
                        let err_msg = format!("刪除事件失敗 (sync_id={}): {}", sync_id, e);
                        stats.error_messages.push(err_msg);
                        Self::mark_sync_error(pool, sync_id, &e.to_string()).await;
                        stats.errors += 1;
                    }
                }
            }
        }
    }

    /// 將同步記錄標記為錯誤（共用 helper，避免重複的 SQL）
    async fn mark_sync_error(pool: &PgPool, sync_id: Uuid, error_msg: &str) {
        if let Err(e) = sqlx::query(
            r#"
            UPDATE calendar_event_sync
            SET sync_status = 'error',
                last_error = $1,
                error_count = error_count + 1,
                last_error_at = NOW(),
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(error_msg)
        .bind(sync_id)
        .execute(pool)
        .await
        {
            tracing::warn!("DB 操作失敗: {e}");
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_leave_type_display_known_types() {
        assert_eq!(get_leave_type_display("ANNUAL"), "特休假");
        assert_eq!(get_leave_type_display("PERSONAL"), "事假");
        assert_eq!(get_leave_type_display("SICK"), "病假");
        assert_eq!(get_leave_type_display("COMPENSATORY"), "補休假");
        assert_eq!(get_leave_type_display("MARRIAGE"), "婚假");
        assert_eq!(get_leave_type_display("BEREAVEMENT"), "喪假");
        assert_eq!(get_leave_type_display("MATERNITY"), "產假");
        assert_eq!(get_leave_type_display("PATERNITY"), "陪產假");
        assert_eq!(get_leave_type_display("MENSTRUAL"), "生理假");
        assert_eq!(get_leave_type_display("OFFICIAL"), "公假");
    }

    #[test]
    fn test_get_leave_type_display_unknown() {
        assert_eq!(get_leave_type_display("UNKNOWN"), "請假");
        assert_eq!(get_leave_type_display(""), "請假");
    }

    #[test]
    fn test_build_event_summary_without_proxy() {
        let result = build_event_summary("張三", None, "ANNUAL");
        assert_eq!(result, "[特休假] 張三");
    }

    #[test]
    fn test_build_event_summary_with_proxy() {
        let result = build_event_summary("張三", Some("李四"), "SICK");
        assert_eq!(result, "[病假] 張三（李四）");
    }

    #[test]
    fn test_build_event_summary_unknown_leave_type() {
        let result = build_event_summary("王五", None, "CUSTOM");
        assert_eq!(result, "[請假] 王五");
    }
}
