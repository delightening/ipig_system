// 擴展的審計 Service

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        ActivityLogQuery, AuditAction, AuditDashboardStats, AuditLog, AuditLogQuery,
        AuditLogWithActor, LoginEventQuery, LoginEventWithUser, PaginatedResponse,
        SecurityAlert, SecurityAlertQuery, SessionQuery, SessionWithUser, UserActivityLog,
    },
    Result,
};

pub struct AuditService;

impl AuditService {
    /// 記錄稽核日誌（原有）
    pub async fn log(
        pool: &PgPool,
        actor_user_id: Uuid,
        action: AuditAction,
        entity_type: &str,
        entity_id: Uuid,
        before: Option<serde_json::Value>,
        after: Option<serde_json::Value>,
    ) -> Result<AuditLog> {
        let log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (id, actor_user_id, action, entity_type, entity_id, before_data, after_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(actor_user_id)
        .bind(action.as_str())
        .bind(entity_type)
        .bind(entity_id)
        .bind(before)
        .bind(after)
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// 查詢稽核日誌（原有）
    pub async fn list(pool: &PgPool, query: &AuditLogQuery) -> Result<Vec<AuditLogWithActor>> {
        let logs = if let Some(ref entity_type) = query.entity_type {
            if let Some(ref action) = query.action {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        eu.email as entity_email, eu.display_name as entity_name,
                        al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                    WHERE al.entity_type = $1 AND al.action = $2
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .bind(action)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        eu.email as entity_email, eu.display_name as entity_name,
                        al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                    WHERE al.entity_type = $1
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(ref action) = query.action {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    eu.email as entity_email, eu.display_name as entity_name,
                    al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                WHERE al.action = $1
                ORDER BY al.created_at DESC
                LIMIT 200
                "#
            )
            .bind(action)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    eu.email as entity_email, eu.display_name as entity_name,
                    al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                ORDER BY al.created_at DESC
                LIMIT 200
                "#
            )
            .fetch_all(pool)
            .await?
        };

        Ok(logs)
    }

    /// 取得特定實體的稽核歷史
    pub async fn get_entity_history(
        pool: &PgPool,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditLogWithActor>> {
        let logs = sqlx::query_as::<_, AuditLogWithActor>(
            r#"
            SELECT 
                al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                al.action, al.entity_type, al.entity_id,
                eu.email as entity_email, eu.display_name as entity_name,
                al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
            FROM audit_logs al
            INNER JOIN users u ON al.actor_user_id = u.id
            LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
            WHERE al.entity_type = $1 AND al.entity_id = $2
            ORDER BY al.created_at DESC
            "#
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// 記錄詳細活動日誌
    pub async fn log_activity(
        pool: &PgPool,
        actor_user_id: Uuid,
        event_category: &str,
        event_type: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        entity_display_name: Option<&str>,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Uuid> {
        let result: (Uuid,) = sqlx::query_as(
            "SELECT log_activity($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10)"
        )
        .bind(actor_user_id)
        .bind(event_category)
        .bind(event_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(entity_display_name)
        .bind(before_data.clone())
        .bind(after_data.clone())
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(pool)
        .await?;

        let log_id = result.0;

        // SEC-34: HMAC 雜湊鏈（有 key 時才啟用）
        if let Ok(hmac_key) = std::env::var("AUDIT_HMAC_KEY") {
            if hmac_key.len() >= 16 {
                let _ = Self::compute_and_store_hmac(
                    pool, log_id, &hmac_key,
                    event_category, event_type,
                    actor_user_id,
                    &before_data, &after_data,
                ).await;
            }
        }

        Ok(log_id)
    }

    /// SEC-34: 計算 HMAC-SHA256 並更新日誌記錄
    async fn compute_and_store_hmac(
        pool: &PgPool,
        log_id: Uuid,
        hmac_key: &str,
        event_category: &str,
        event_type: &str,
        actor_user_id: Uuid,
        before_data: &Option<serde_json::Value>,
        after_data: &Option<serde_json::Value>,
    ) -> Result<()> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // 取得前一筆日誌的 integrity_hash（若有）
        let prev_hash: Option<String> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE created_at < (SELECT created_at FROM user_activity_logs WHERE id = $1)
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(log_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        // 組合雜湊內容: prev_hash + event_category + event_type + actor_id + data
        let mut message = String::new();
        if let Some(ref ph) = prev_hash {
            message.push_str(ph);
        }
        message.push_str(event_category);
        message.push_str(event_type);
        message.push_str(&actor_user_id.to_string());
        if let Some(ref bd) = before_data {
            message.push_str(&bd.to_string());
        }
        if let Some(ref ad) = after_data {
            message.push_str(&ad.to_string());
        }

        // 計算 HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(message.as_bytes());
        let hash_bytes = mac.finalize().into_bytes();
        let hash_result: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // 更新記錄
        sqlx::query(
            "UPDATE user_activity_logs SET integrity_hash = $1, previous_hash = $2 WHERE id = $3"
        )
        .bind(&hash_result)
        .bind(prev_hash.as_deref())
        .bind(log_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // ============================================
    // 新增的 Activity Logs 方法
    // ============================================

    /// 列出使用者活動記錄
    pub async fn list_activities(
        pool: &PgPool,
        query: &ActivityLogQuery,
    ) -> Result<PaginatedResponse<UserActivityLog>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // 計算總數
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, UserActivityLog>(
            r#"
            SELECT * FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            ORDER BY created_at DESC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 匯出使用者活動記錄（不分頁，上限 10000 筆）
    pub async fn export_activities(
        pool: &PgPool,
        query: &ActivityLogQuery,
    ) -> Result<Vec<UserActivityLog>> {
        let data = sqlx::query_as::<_, UserActivityLog>(
            r#"
            SELECT * FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            ORDER BY created_at DESC
            LIMIT 10000
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    // ============================================
    // Login Events
    // ============================================

    /// 列出登入事件
    pub async fn list_login_events(
        pool: &PgPool,
        query: &LoginEventQuery,
    ) -> Result<PaginatedResponse<LoginEventWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR event_type = $2)
              AND ($3::bool IS NULL OR (is_unusual_time OR is_unusual_location OR is_new_device) = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_type)
        .bind(query.is_unusual)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, LoginEventWithUser>(
            r#"
            SELECT 
                le.id, le.user_id, le.email, u.display_name as user_name,
                le.event_type, le.ip_address::TEXT, le.user_agent,
                le.device_type, le.browser, le.os,
                le.is_unusual_time, le.is_unusual_location, le.is_new_device, le.is_mass_login,
                le.failure_reason, le.created_at
            FROM login_events le
            LEFT JOIN users u ON le.user_id = u.id
            WHERE ($1::uuid IS NULL OR le.user_id = $1)
              AND ($2::text IS NULL OR le.event_type = $2)
              AND ($3::bool IS NULL OR (le.is_unusual_time OR le.is_unusual_location OR le.is_new_device OR le.is_mass_login) = $3)
              AND ($4::date IS NULL OR le.created_at::date >= $4)
              AND ($5::date IS NULL OR le.created_at::date <= $5)
            ORDER BY le.created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_type)
        .bind(query.is_unusual)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    // ============================================
    // Sessions
    // ============================================

    /// 列出 Sessions
    pub async fn list_sessions(
        pool: &PgPool,
        query: &SessionQuery,
    ) -> Result<PaginatedResponse<SessionWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_sessions
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::date IS NULL OR started_at::date >= $3)
              AND ($4::date IS NULL OR started_at::date <= $4)
            "#,
        )
        .bind(query.user_id)
        .bind(query.is_active)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, SessionWithUser>(
            r#"
            SELECT 
                s.id, s.user_id, u.email as user_email, u.display_name as user_name,
                s.started_at, s.ended_at, s.last_activity_at,
                s.ip_address::TEXT, s.user_agent,
                s.page_view_count, s.action_count,
                s.is_active, s.ended_reason
            FROM user_sessions s
            INNER JOIN users u ON s.user_id = u.id
            WHERE ($1::uuid IS NULL OR s.user_id = $1)
              AND ($2::bool IS NULL OR s.is_active = $2)
              AND ($3::date IS NULL OR s.started_at::date >= $3)
              AND ($4::date IS NULL OR s.started_at::date <= $4)
            ORDER BY s.started_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(query.is_active)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 強制登出 Session
    pub async fn force_logout_session(
        pool: &PgPool,
        session_id: Uuid,
        admin_id: Uuid,
        reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'forced_logout'
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        // 記錄審計日誌
        Self::log(
            pool,
            admin_id,
            AuditAction::Logout,
            "session",
            session_id,
            None,
            Some(serde_json::json!({ "reason": reason.unwrap_or("admin_forced") })),
        )
        .await?;

        Ok(())
    }

    // ============================================
    // Security Alerts
    // ============================================

    /// 列出安全警報
    pub async fn list_security_alerts(
        pool: &PgPool,
        query: &SecurityAlertQuery,
    ) -> Result<PaginatedResponse<SecurityAlert>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM security_alerts
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::uuid IS NULL OR user_id = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
            "#,
        )
        .bind(&query.status)
        .bind(&query.severity)
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT * FROM security_alerts
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::uuid IS NULL OR user_id = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(&query.status)
        .bind(&query.severity)
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 取得指定安全警報
    pub async fn get_security_alert(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<SecurityAlert> {
        let alert = sqlx::query_as::<_, SecurityAlert>(
            "SELECT * FROM security_alerts WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(alert)
    }

    /// 解決安全警報
    pub async fn resolve_alert(
        pool: &PgPool,
        alert_id: Uuid,
        resolver_id: Uuid,
        notes: Option<&str>,
    ) -> Result<SecurityAlert> {
        let alert = sqlx::query_as::<_, SecurityAlert>(
            r#"
            UPDATE security_alerts
            SET status = 'resolved',
                resolved_by = $2,
                resolved_at = NOW(),
                resolution_notes = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(alert_id)
        .bind(resolver_id)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(alert)
    }

    // ============================================
    // Dashboard
    // ============================================

    /// 取得審計儀表板統計
    pub async fn get_dashboard_stats(pool: &PgPool) -> Result<AuditDashboardStats> {
        let today = chrono::Utc::now().date_naive();
        let week_ago = today - chrono::Duration::days(7);
        let month_ago = today - chrono::Duration::days(30);

        // 活躍用戶數
        let active_today: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date = $1",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let active_week: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date >= $1",
        )
        .bind(week_ago)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let active_month: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date >= $1",
        )
        .bind(month_ago)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        // 登入統計
        let logins_today: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_events WHERE created_at::date = $1 AND event_type = 'login_success'",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let failed_logins: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_events WHERE created_at::date = $1 AND event_type = 'login_failure'",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        // 活躍 Sessions
        let active_sessions: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE is_active = true")
                .fetch_one(pool)
                .await
                .unwrap_or((0,));

        // 警報統計
        let open_alerts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM security_alerts WHERE status IN ('open', 'acknowledged', 'investigating')",
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let critical_alerts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM security_alerts WHERE status IN ('open', 'acknowledged') AND severity = 'critical'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        Ok(AuditDashboardStats {
            active_users_today: active_today.0,
            active_users_week: active_week.0,
            active_users_month: active_month.0,
            total_logins_today: logins_today.0,
            failed_logins_today: failed_logins.0,
            active_sessions: active_sessions.0,
            open_alerts: open_alerts.0,
            critical_alerts: critical_alerts.0,
        })
    }
}
