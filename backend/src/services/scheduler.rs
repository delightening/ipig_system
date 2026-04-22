use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_util::sync::CancellationToken;
use sqlx::PgPool;
use tracing::{info, error};
use chrono::{Timelike, Datelike, Weekday};

use crate::{
    config::Config,
    services::{EmailService, InvitationService, NotificationService, BalanceExpirationJob, CalendarService, PartitionMaintenanceJob, EuthanasiaService, SecurityNotifier, SecurityNotification},
};

type SchedulerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub struct SchedulerService;

impl SchedulerService {
    /// 啟動排程服務
    ///
    /// `shutdown_token`：graceful shutdown 訊號；cancel 時所有 job 跳過下一次觸發。
    /// 正在執行中的 job 會跑完目前批次才退出（commit 5 會在每個 job 內部加 select!）。
    pub async fn start(
        db: PgPool,
        config: Arc<Config>,
        shutdown_token: CancellationToken,
    ) -> Result<JobScheduler, Box<dyn std::error::Error + Send + Sync>> {
        let sched = JobScheduler::new().await?;
        let mut job_count = 0;

        // 所有 job 接 CancellationToken：shutdown 時跳過下一輪觸發。
        // 執行中的 job 跑完當輪才退出（避免中斷 DB 操作造成不一致狀態）。
        let t = &shutdown_token;

        Self::register_low_stock_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_expiry_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_notification_cleanup_job(&sched, &db, t, &mut job_count).await?;
        Self::register_balance_expiration_job(&sched, &db, t, &mut job_count).await?;
        Self::register_calendar_sync_jobs(&sched, &db, t, &mut job_count).await?;
        Self::register_partition_maintenance_job(&sched, &db, t, &mut job_count).await?;
        Self::register_euthanasia_timeout_job(&sched, &db, t, &mut job_count).await?;
        Self::register_po_pending_receipt_job(&sched, &db, t, &mut job_count).await?;
        Self::register_equipment_overdue_job(&sched, &db, t, &mut job_count).await?;
        Self::register_monthly_report_job(&sched, &db, t, &mut job_count).await?;
        Self::register_invitation_expiry_job(&sched, &db, t, &mut job_count).await?;
        Self::register_db_analyze_job(&sched, &db, t, &mut job_count).await?;
        Self::register_iacuc_submission_notify_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_unresolved_alert_sweep_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_audit_chain_verify_job(&sched, &db, &config, t, &mut job_count).await?;

        sched.start().await?;
        info!("[Scheduler] ✓ All {} jobs registered and scheduler started successfully", job_count);

        Ok(sched)
    }

    // ── Job 註冊 helpers ──

    /// 每小時整點觸發低庫存檢查（由 DB routing 設定決定實際執行時機）
    async fn register_low_stock_job(sched: &JobScheduler, db: &PgPool, config: &Arc<Config>, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] low_stock_check skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::maybe_run_low_stock_check(&db, &config).await {
                    error!("Low stock check runner failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'low_stock_check' registered (dynamic schedule)");
        *count += 1;
        Ok(())
    }

    /// 每小時整點觸發效期檢查（由 DB routing 設定決定實際執行時機）
    async fn register_expiry_job(sched: &JobScheduler, db: &PgPool, config: &Arc<Config>, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] expiry_check skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::maybe_run_expiry_check(&db, &config).await {
                    error!("Expiry check runner failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'expiry_check' registered (dynamic schedule)");
        *count += 1;
        Ok(())
    }

    /// 每週日 03:00 清理過期通知
    async fn register_notification_cleanup_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 3 * * Sun", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] notification_cleanup skipped during shutdown");
                    return;
                }
                info!("Running weekly notification cleanup...");
                if let Err(e) = Self::cleanup_notifications(&db).await {
                    error!("Notification cleanup failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'notification_cleanup' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 00:30 執行餘額到期檢查
    async fn register_balance_expiration_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 30 0 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] balance_expiration skipped during shutdown");
                    return;
                }
                info!("Running daily balance expiration check...");
                match BalanceExpirationJob::run(&db).await {
                    Ok(summary) => {
                        info!("Balance expiration check completed: {} annual, {} comp_time expired",
                              summary.annual_leave_expired, summary.comp_time_expired);
                    }
                    Err(e) => error!("Balance expiration check failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'balance_expiration' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 08:00 與 18:00 執行 Google Calendar 同步
    async fn register_calendar_sync_jobs(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (cron, label) in [("0 0 8 * * *", "morning"), ("0 0 18 * * *", "evening")] {
            let db_clone = db.clone();
            let token_outer = token.clone();
            let job = Job::new_async(cron, move |_uuid, _l| {
                let db = db_clone.clone();
                let token = token_outer.clone();
                Box::pin(async move {
                    if token.is_cancelled() {
                        info!("[Scheduler] calendar_sync_{} skipped during shutdown", label);
                        return;
                    }
                    info!("Running scheduled calendar sync ({})...", label);
                    match CalendarService::trigger_sync(&db, None).await {
                        Ok(history) => info!("Calendar sync completed: {:?}", history.status),
                        Err(e) => error!("Calendar sync failed: {}", e),
                    }
                })
            })?;
            sched.add(job).await?;
            info!("[Scheduler] ✓ Job 'calendar_sync_{}' registered", label);
            *count += 1;
        }
        Ok(())
    }

    /// 每年 12 月 1 日 03:00 執行分區表維護
    async fn register_partition_maintenance_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 3 1 12 *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] partition_maintenance skipped during shutdown");
                    return;
                }
                info!("Running annual partition maintenance...");
                match PartitionMaintenanceJob::ensure_partitions(&db).await {
                    Ok(result) => {
                        info!("Partition maintenance completed: {} checked, {} created, {} existing",
                              result.checked, result.created, result.existing);
                    }
                    Err(e) => error!("Partition maintenance failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'partition_maintenance' registered");
        *count += 1;
        Ok(())
    }

    /// 每 5 分鐘檢查安樂死單據超時
    async fn register_euthanasia_timeout_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0/5 * * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    return;
                }
                match EuthanasiaService::check_expired_orders(&db).await {
                    Ok(c) if c > 0 => info!("Euthanasia timeout check: {} orders auto-approved", c),
                    Ok(_) => {}
                    Err(e) => error!("Euthanasia timeout check failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'euthanasia_timeout' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 09:00 檢查已核准但未入庫的採購單
    async fn register_po_pending_receipt_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 9 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] po_pending_receipt_check skipped during shutdown");
                    return;
                }
                info!("Running daily PO pending receipt check...");
                if let Err(e) = Self::check_po_pending_receipt(&db).await {
                    error!("PO pending receipt check failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'po_pending_receipt_check' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 08:30 檢查設備校正/確效逾期
    async fn register_equipment_overdue_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 30 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] equipment_overdue_check skipped during shutdown");
                    return;
                }
                info!("Running daily equipment overdue check...");
                let service = NotificationService::new(db);
                match service.send_equipment_overdue_notifications().await {
                    Ok(c) if c > 0 => info!("Equipment overdue check: notified {} recipients", c),
                    Ok(_) => {}
                    Err(e) => error!("Equipment overdue check failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'equipment_overdue_check' registered");
        *count += 1;
        Ok(())
    }

    /// 每月 1 號 06:00 產出上月進銷貨+血液檢查報表
    async fn register_monthly_report_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 6 1 * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] monthly_report skipped during shutdown");
                    return;
                }
                info!("Running monthly report generation...");
                if let Err(e) = Self::generate_monthly_report(&db).await {
                    error!("Monthly report generation failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'monthly_report' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 04:00 清理過期邀請
    async fn register_invitation_expiry_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 4 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] invitation_expiry skipped during shutdown");
                    return;
                }
                info!("Running daily invitation expiry check...");
                match InvitationService::expire_stale(&db).await {
                    Ok(c) if c > 0 => info!("Invitation expiry check: {} invitations expired", c),
                    Ok(_) => {}
                    Err(e) => error!("Invitation expiry check failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'invitation_expiry' registered");
        *count += 1;
        Ok(())
    }

    /// 每兩小時（平日 07:00–15:00 台灣時間）檢查新送審 IACUC 計畫書並通知執行秘書
    async fn register_iacuc_submission_notify_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] iacuc_submission_notify skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::check_iacuc_new_submissions(&db, &config).await {
                    error!("[IACUC] Submission notify job failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'iacuc_submission_notify' registered (every 2h, Mon-Fri 07:00-15:00 CST)");
        *count += 1;
        Ok(())
    }

    /// 每天 03:30 執行 ANALYZE
    async fn register_db_analyze_job(sched: &JobScheduler, db: &PgPool, token: &CancellationToken, count: &mut u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 30 3 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] db_maintenance_analyze skipped during shutdown");
                    return;
                }
                info!("Running scheduled ANALYZE on high-write tables...");
                if let Err(e) = sqlx::query("SELECT maintenance_vacuum_analyze()")
                    .execute(&db)
                    .await
                {
                    error!("Scheduled ANALYZE failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'db_maintenance_analyze' registered");
        *count += 1;
        Ok(())
    }

    /// R22-13: 每 6 小時掃描未處理的 security_alerts，重送通知
    async fn register_unresolved_alert_sweep_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 */6 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] unresolved_alert_sweep skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::sweep_unresolved_alerts(&db, &config).await {
                    error!("[R22-13] Unresolved alert sweep failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'unresolved_alert_sweep' registered");
        *count += 1;
        Ok(())
    }

    /// R26-2：每日 02:00 UTC 驗證昨日 audit HMAC chain 完整性。
    ///
    /// 斷鏈時寫入 `security_alerts` 並觸發 `SecurityNotifier::dispatch`；
    /// 完整時僅 log info。正在執行時若收到 shutdown，當輪跑完才退出。
    async fn register_audit_chain_verify_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        // 每日 02:00:00 UTC 觸發
        let job = Job::new_async("0 0 2 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] audit_chain_verify skipped during shutdown");
                    return;
                }
                if let Err(e) =
                    crate::services::audit_chain_verify::verify_yesterday_chain(&db, &config).await
                {
                    error!("[R26-2] Audit chain verify failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'audit_chain_verify' registered (daily 02:00 UTC)");
        *count += 1;
        Ok(())
    }

    async fn sweep_unresolved_alerts(db: &PgPool, config: &Config) -> SchedulerResult {
        #[derive(sqlx::FromRow)]
        struct AlertRow {
            id: uuid::Uuid,
            alert_type: String,
            severity: String,
            title: String,
            description: Option<String>,
            context_data: Option<serde_json::Value>,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let alerts: Vec<AlertRow> = sqlx::query_as(
            r#"
            SELECT id, alert_type, severity, title, description, context_data, created_at
            FROM security_alerts
            WHERE status = 'open'
              AND severity IN ('critical', 'warning')
              AND created_at < NOW() - INTERVAL '24 hours'
              AND (last_notified_at IS NULL OR last_notified_at < NOW() - INTERVAL '6 hours')
            ORDER BY created_at ASC
            LIMIT 20
            "#,
        )
        .fetch_all(db)
        .await?;

        if alerts.is_empty() {
            return Ok(());
        }

        info!(
            "[R22-13] Found {} unresolved alerts older than 24h, re-sending notifications",
            alerts.len()
        );

        for row in alerts {
            // Update last_notified_at before dispatching to prevent duplicate sends on crash/retry
            sqlx::query(
                "UPDATE security_alerts SET last_notified_at = NOW(), updated_at = NOW() WHERE id = $1",
            )
            .bind(row.id)
            .execute(db)
            .await?;

            let notification = SecurityNotification {
                alert_id: row.id,
                alert_type: row.alert_type,
                severity: row.severity,
                title: format!("[Reminder] {}", row.title),
                description: row.description,
                context_data: row.context_data,
                created_at: row.created_at,
            };
            SecurityNotifier::dispatch(db, config, &notification).await;
        }

        Ok(())
    }

    // ── 業務邏輯 helpers ──

    /// 動態排程：依 DB routing 設定判斷是否執行低庫存檢查
    async fn maybe_run_low_stock_check(db: &PgPool, config: &Config) -> SchedulerResult {
        if Self::should_run_now(db, "low_stock_alert").await? {
            Self::check_low_stock(db, config).await?;
        }
        Ok(())
    }

    /// 動態排程：依 DB routing 設定判斷是否執行效期檢查
    async fn maybe_run_expiry_check(db: &PgPool, config: &Config) -> SchedulerResult {
        if Self::should_run_now(db, "expiry_alert").await? {
            Self::check_expiry(db, config).await?;
        }
        Ok(())
    }

    /// 判斷當前時間是否符合指定事件的任一 routing 規則排程
    async fn should_run_now(db: &PgPool, event_type: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let rows: Vec<(String, i16, Option<i16>)> = sqlx::query_as(
            r#"
            SELECT frequency, hour_of_day, day_of_week
            FROM notification_routing
            WHERE event_type = $1 AND is_active = true
              AND frequency != 'immediate'
            "#,
        )
        .bind(event_type)
        .fetch_all(db)
        .await?;

        if rows.is_empty() {
            return Ok(false);
        }

        let now = chrono::Local::now();
        let current_hour = now.hour() as i16;
        let current_dow = now.weekday().num_days_from_sunday() as i16;
        let current_day = now.day();

        for (frequency, hour_of_day, day_of_week) in rows {
            let matches = match frequency.as_str() {
                "daily"   => hour_of_day == current_hour,
                "weekly"  => hour_of_day == current_hour && day_of_week == Some(current_dow),
                "monthly" => hour_of_day == current_hour && current_day == 1,
                _         => false,
            };
            if matches {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 檢查低庫存並發送通知
    async fn check_low_stock(db: &PgPool, config: &Config) -> SchedulerResult {
        let service = NotificationService::new(db.clone());
        let alerts = service.list_low_stock_alerts(1, 100).await?;

        if alerts.data.is_empty() {
            info!("No low stock alerts found");
            return Ok(());
        }

        match service.send_low_stock_notifications().await {
            Ok(count) => info!("Low stock in-app notifications: {} sent", count),
            Err(e) => tracing::warn!("發送庫存不足站內通知失敗: {e}"),
        }

        let users = Self::fetch_stock_email_recipients(db, "email_low_stock").await?;
        let alerts_html = Self::build_low_stock_html(&alerts.data);
        let email_count = Self::send_low_stock_emails(config, &users, &alerts_html, alerts.data.len()).await;

        info!("Low stock check completed: {} alerts, {} emails sent", alerts.data.len(), email_count);
        Ok(())
    }

    /// 檢查效期並發送通知（含月度閾值邏輯）
    async fn check_expiry(db: &PgPool, config: &Config) -> SchedulerResult {
        use crate::services::notification::expiry_monthly::{current_ym, previous_ym};

        let service = NotificationService::new(db.clone());
        let cfg = service.get_expiry_notification_config().await?;

        let alerts: Vec<crate::models::ExpiryAlert> = sqlx::query_as(
            "SELECT * FROM fn_expiry_alerts($1, $2)",
        )
        .bind(cfg.warn_days as i32)
        .bind(cfg.cutoff_days as i32)
        .fetch_all(db)
        .await?;

        if alerts.is_empty() {
            info!("No expiry alerts found");
            return Ok(());
        }

        // 月度閾值邏輯：超過閾值天數的品項走月度通知路徑
        if let Some(threshold) = cfg.monthly_threshold_days {
            let ym = current_ym();
            if let Err(e) = service.take_expiry_monthly_snapshot(&ym, threshold).await {
                tracing::warn!("月度快照寫入失敗: {e}");
            }
            if let Some(prev_ym) = previous_ym(&ym) {
                match service.compare_expiry_snapshots(&ym, &prev_ym).await {
                    Ok(diff) => match service.send_monthly_expiry_comparison(&diff, &ym, threshold).await {
                        Ok(c) => info!("Monthly expiry comparison notifications: {} sent", c),
                        Err(e) => tracing::warn!("月度效期比較通知發送失敗: {e}"),
                    },
                    Err(e) => tracing::warn!("月度快照比較失敗: {e}"),
                }
            }
        }

        // 一般效期通知（排除月度閾值範圍的品項）
        let threshold_days = cfg.monthly_threshold_days.unwrap_or(i16::MAX);
        let regular_alerts: Vec<_> = alerts.into_iter()
            .filter(|a| a.days_until_expiry >= -(threshold_days as i32))
            .collect();

        if regular_alerts.is_empty() {
            return Ok(());
        }

        let expired_count = regular_alerts.iter().filter(|a| a.expiry_status == "expired").count();
        let expiring_count = regular_alerts.iter().filter(|a| a.expiry_status == "expiring_soon").count();

        match service.send_expiry_notifications().await {
            Ok(count) => info!("Expiry in-app notifications: {} sent", count),
            Err(e) => tracing::warn!("發送效期預警站內通知失敗: {e}"),
        }

        let users = Self::fetch_stock_email_recipients(db, "email_expiry_warning").await?;
        let alerts_html = Self::build_expiry_html(&regular_alerts);
        let email_count = Self::send_expiry_emails(config, &users, &alerts_html, expired_count, expiring_count).await;

        info!("Expiry check completed: {} alerts ({} expired, {} expiring), {} emails sent",
              regular_alerts.len(), expired_count, expiring_count, email_count);
        Ok(())
    }

    /// 取得庫存相關 email 通知的收件者
    async fn fetch_stock_email_recipients(
        db: &PgPool,
        setting_column: &str,
    ) -> Result<Vec<(uuid::Uuid, String, String)>, Box<dyn std::error::Error + Send + Sync>> {
        // setting_column 為白名單欄位名（email_low_stock / email_expiry_warning）
        let sql = format!(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.{} = true
            "#,
            match setting_column {
                "email_low_stock" => "email_low_stock",
                "email_expiry_warning" => "email_expiry_warning",
                _ => return Ok(vec![]),
            }
        );
        Ok(sqlx::query_as(&sql).fetch_all(db).await?)
    }

    /// 發送低庫存 email 並回傳成功數
    async fn send_low_stock_emails(
        config: &Config,
        users: &[(uuid::Uuid, String, String)],
        alerts_html: &str,
        alert_count: usize,
    ) -> usize {
        let mut email_count = 0;
        for (_user_id, email, name) in users {
            if let Err(e) = EmailService::send_low_stock_alert_email(
                config, email, name, alerts_html, alert_count,
            ).await {
                error!("Failed to send low stock email to {}: {}", email, e);
            } else {
                email_count += 1;
            }
        }
        email_count
    }

    /// 發送效期預警 email 並回傳成功數
    async fn send_expiry_emails(
        config: &Config,
        users: &[(uuid::Uuid, String, String)],
        alerts_html: &str,
        expired_count: usize,
        expiring_count: usize,
    ) -> usize {
        let mut email_count = 0;
        for (_user_id, email, name) in users {
            if let Err(e) = EmailService::send_expiry_alert_email(
                config, email, name, alerts_html, expired_count, expiring_count,
            ).await {
                error!("Failed to send expiry email to {}: {}", email, e);
            } else {
                email_count += 1;
            }
        }
        email_count
    }

    /// 清理過期通知
    async fn cleanup_notifications(db: &PgPool) -> SchedulerResult {
        let service = NotificationService::new(db.clone());
        let deleted = service.cleanup_old_notifications().await?;
        info!("Notification cleanup completed: {} old notifications deleted", deleted);
        Ok(())
    }

    /// 建構低庫存 HTML 表格
    fn build_low_stock_html(alerts: &[crate::models::LowStockAlert]) -> String {
        let mut html = String::from(
            r#"<table class="alert-table">
            <thead>
                <tr>
                    <th>SKU</th><th>品名</th><th>倉庫</th><th>現有量</th><th>安全庫存</th>
                </tr>
            </thead>
            <tbody>"#
        );

        for alert in alerts.iter().take(20) {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{} {}</td><td>{}</td></tr>",
                alert.product_sku, alert.product_name, alert.warehouse_name,
                alert.qty_on_hand, alert.base_uom,
                alert.safety_stock.map(|s| s.to_string()).unwrap_or("-".to_string()),
            ));
        }

        html.push_str("</tbody></table>");
        if alerts.len() > 20 {
            html.push_str(&format!("<p>...另外還有 {} 項，請登入系統查看完整列表</p>", alerts.len() - 20));
        }
        html
    }

    /// 建構效期預警 HTML 表格
    fn build_expiry_html(alerts: &[crate::models::ExpiryAlert]) -> String {
        let mut html = String::from(
            r#"<table class="alert-table">
            <thead>
                <tr>
                    <th>SKU</th><th>品名</th><th>批號</th><th>效期</th>
                    <th>剩餘天數</th><th>近效期量</th><th>總量</th>
                </tr>
            </thead>
            <tbody>"#
        );

        for alert in alerts.iter().take(20) {
            let status_class = if alert.expiry_status == "expired" { "expired" } else { "expiring" };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td>\
                 <td class=\"{}\">{}</td><td>{} {}</td><td>{} {}</td></tr>",
                alert.sku, alert.product_name,
                alert.batch_no.as_deref().unwrap_or("-"), alert.expiry_date,
                status_class, alert.days_until_expiry,
                alert.on_hand_qty, alert.base_uom, alert.total_qty, alert.base_uom,
            ));
        }

        html.push_str("</tbody></table>");
        if alerts.len() > 20 {
            html.push_str(&format!("<p>...另外還有 {} 項，請登入系統查看完整列表</p>", alerts.len() - 20));
        }
        html
    }

    /// 手動觸發低庫存檢查（供 API 使用）
    pub async fn trigger_low_stock_check(db: &PgPool, config: &Config) -> SchedulerResult {
        Self::check_low_stock(db, config).await
    }

    /// 手動觸發效期檢查（供 API 使用）
    pub async fn trigger_expiry_check(db: &PgPool, config: &Config) -> SchedulerResult {
        Self::check_expiry(db, config).await
    }

    /// 檢查已核准但未入庫的採購單並發送通知
    async fn check_po_pending_receipt(db: &PgPool) -> SchedulerResult {
        let service = NotificationService::new(db.clone());
        let count = service.notify_po_pending_receipt().await?;
        info!("PO pending receipt check completed: {} notifications sent", count);
        Ok(())
    }

    /// 手動觸發採購單未入庫檢查（供 API 使用）
    pub async fn trigger_po_pending_receipt_check(db: &PgPool) -> SchedulerResult {
        Self::check_po_pending_receipt(db).await
    }

    // ── 月報表 ──

    /// 產出每月進銷貨+血液檢查報表
    async fn generate_monthly_report(db: &PgPool) -> SchedulerResult {
        let (first_day, last_day, month_str) = Self::compute_previous_month_range()?;
        info!("[Monthly Report] 統計期間：{} ~ {}", first_day, last_day);

        let (po_count, po_amount) = Self::query_purchase_summary(db, first_day, last_day).await?;
        let (so_count, so_amount) = Self::query_sales_summary(db, first_day, last_day).await?;
        let blood_test_stats = Self::query_blood_test_stats(db, first_day, last_day).await?;

        let content = Self::build_report_content(
            &month_str, po_count, &po_amount, so_count, &so_amount, &blood_test_stats,
        );

        let count = Self::send_report_notifications(db, &month_str, &content).await?;

        info!(
            "[Monthly Report] {}報表已產出並發送給 {} 位使用者（PO: {}, SO: {}, 血檢項: {}）",
            month_str, count, po_count, so_count, blood_test_stats.len()
        );
        Ok(())
    }

    /// 計算上月的起迄日期
    fn compute_previous_month_range() -> Result<(chrono::NaiveDate, chrono::NaiveDate, String), Box<dyn std::error::Error + Send + Sync>> {
        use chrono::{Datelike, NaiveDate};

        let now = crate::time::today_taiwan_naive();
        let year = if now.month() == 1 { now.year() - 1 } else { now.year() };
        let month = if now.month() == 1 { 12 } else { now.month() - 1 };
        let first_day = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| format!("invalid date: {year}-{month}-01"))?;
        let last_day = if now.month() == 1 {
            NaiveDate::from_ymd_opt(now.year(), 1, 1)
                .ok_or_else(|| format!("invalid date: {}-01-01", now.year()))?
                .pred_opt()
                .ok_or_else(|| "failed to get last day of previous year".to_string())?
        } else {
            NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .ok_or_else(|| format!("invalid date: {}-{}-01", now.year(), now.month()))?
                .pred_opt()
                .ok_or_else(|| format!("failed to get last day of {}-{}", now.year(), now.month() - 1))?
        };
        let month_str = format!("{}年{}月", year, month);
        Ok((first_day, last_day, month_str))
    }

    /// 查詢採購彙總
    async fn query_purchase_summary(
        db: &PgPool,
        first_day: chrono::NaiveDate,
        last_day: chrono::NaiveDate,
    ) -> Result<(i64, Option<rust_decimal::Decimal>), Box<dyn std::error::Error + Send + Sync>> {
        let row: Option<(i64, Option<rust_decimal::Decimal>)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as cnt,
                   SUM(dl.qty * COALESCE(dl.unit_price, 0)) as total_amount
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.doc_type = 'PO' AND d.status = 'approved'
              AND d.doc_date BETWEEN $1 AND $2
            "#,
        )
        .bind(first_day)
        .bind(last_day)
        .fetch_optional(db)
        .await?;
        Ok(row.unwrap_or((0, None)))
    }

    /// 查詢銷貨彙總
    async fn query_sales_summary(
        db: &PgPool,
        first_day: chrono::NaiveDate,
        last_day: chrono::NaiveDate,
    ) -> Result<(i64, Option<rust_decimal::Decimal>), Box<dyn std::error::Error + Send + Sync>> {
        let row: Option<(i64, Option<rust_decimal::Decimal>)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as cnt,
                   SUM(dl.qty * COALESCE(dl.unit_price,
                       (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                        WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL),
                       0)) as total_amount
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.doc_type = 'SO' AND d.status = 'approved'
              AND d.doc_date BETWEEN $1 AND $2
            "#,
        )
        .bind(first_day)
        .bind(last_day)
        .fetch_optional(db)
        .await?;
        Ok(row.unwrap_or((0, None)))
    }

    /// 查詢血液檢查統計
    async fn query_blood_test_stats(
        db: &PgPool,
        first_day: chrono::NaiveDate,
        last_day: chrono::NaiveDate,
    ) -> Result<Vec<(Option<String>, String, i64)>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(sqlx::query_as(
            r#"
            SELECT p.iacuc_no, bti.item_name, COUNT(*) as cnt
            FROM animal_blood_test_items bti
            JOIN animal_blood_tests bt ON bti.blood_test_id = bt.id
            JOIN animals pg ON bt.animal_id = pg.id
            LEFT JOIN protocols p ON pg.protocol_id = p.id
            WHERE bt.test_date BETWEEN $1 AND $2
            GROUP BY p.iacuc_no, bti.item_name
            ORDER BY p.iacuc_no, cnt DESC
            "#,
        )
        .bind(first_day)
        .bind(last_day)
        .fetch_all(db)
        .await
        .unwrap_or_default())
    }

    /// 構建報表內容文字
    fn build_report_content(
        month_str: &str,
        po_count: i64,
        po_amount: &Option<rust_decimal::Decimal>,
        so_count: i64,
        so_amount: &Option<rust_decimal::Decimal>,
        blood_test_stats: &[(Option<String>, String, i64)],
    ) -> String {
        let mut content = format!(
            "{}月度報表\n\n=== 進銷貨彙總 ===\n\
             採購單（已核准）：{} 筆，金額 ${}\n\
             銷貨單（已核准）：{} 筆，金額 ${}\n",
            month_str, po_count,
            po_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
            so_count,
            so_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
        );

        if !blood_test_stats.is_empty() {
            content.push_str("\n=== 血液檢查統計 ===\n");
            for (iacuc_no, item_name, cnt) in blood_test_stats {
                content.push_str(&format!(
                    "計畫 {}：{} × {} 次\n",
                    iacuc_no.as_deref().unwrap_or("-"), item_name, cnt,
                ));
            }
        }
        content
    }

    /// 發送報表通知給相關使用者
    async fn send_report_notifications(
        db: &PgPool,
        month_str: &str,
        content: &str,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let service = NotificationService::new(db.clone());
        let mut recipients = service.get_users_by_role(crate::constants::ROLE_WAREHOUSE_MANAGER).await?;
        let admins = service.get_users_by_role(crate::constants::ROLE_SYSTEM_ADMIN).await?;
        recipients.extend(admins);
        recipients.sort_by_key(|(id, _, _)| *id);
        recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] {}月度進銷貨+血液檢查報表", month_str);
        let mut count = 0;
        for (user_id, _email, _name) in &recipients {
            if let Err(e) = service
                .create_notification(crate::models::CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: crate::models::NotificationType::MonthlyReport,
                    title: title.clone(),
                    content: Some(content.to_string()),
                    related_entity_type: Some("report".to_string()),
                    related_entity_id: None,
                })
                .await
            {
                tracing::warn!("create_notification 失敗: {e}");
            }
            count += 1;
        }
        Ok(count)
    }

    /// 檢查最近 150 分鐘內新送審的 IACUC 計畫書，發送 Email 通知
    async fn check_iacuc_new_submissions(db: &PgPool, config: &Arc<Config>) -> SchedulerResult {
        // 僅在台灣時間（UTC+8）平日 07:00–15:00 執行
        let taipei = chrono::FixedOffset::east_opt(8 * 3600).expect("valid offset");
        let now_taipei = chrono::Utc::now().with_timezone(&taipei);
        let hour = now_taipei.hour();
        let is_workday = matches!(now_taipei.weekday(), Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri);
        if !is_workday || !(7..=15).contains(&hour) {
            return Ok(());
        }

        // 從 system_settings 讀取通知信箱
        let notify_raw: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT value FROM system_settings WHERE key = 'iacuc_notify_emails'",
        )
        .fetch_optional(db)
        .await?;

        let notify_emails = notify_raw
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();

        if notify_emails.is_empty() {
            info!("[IACUC] iacuc_notify_emails 未設定，跳過通知");
            return Ok(());
        }

        // 查詢最近 150 分鐘內的新送審案
        let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT p.protocol_no, p.title, u.name
            FROM protocols p
            LEFT JOIN users u ON u.id = p.pi_user_id
            WHERE p.status = 'SUBMITTED'
              AND p.updated_at >= NOW() - INTERVAL '150 minutes'
            ORDER BY p.updated_at DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        if rows.is_empty() {
            info!("[IACUC] 過去 150 分鐘無新送審案");
            return Ok(());
        }

        let count = rows.len();
        let case_list_html: String = rows.iter()
            .map(|(no, title, pi)| {
                format!(
                    "<li><strong>{no}</strong> — {title}（申請人：{}）</li>",
                    pi.as_deref().unwrap_or("—")
                )
            })
            .collect();
        let case_list_plain: String = rows.iter()
            .map(|(no, title, pi)| format!("{no} — {title}（{}）", pi.as_deref().unwrap_or("—")))
            .collect::<Vec<_>>()
            .join("\n");

        let subject = format!("【iPig IACUC】新送審案件通知 - 共 {count} 件");
        let body_html = format!(
            r#"<html><body style="font-family:Microsoft JhengHei,sans-serif;max-width:600px;margin:0 auto">
<h2 style="color:#1e40af">IACUC 新送審案件通知</h2>
<p>以下計畫書已於過去 2 小時內完成送審，請至 iPig 系統進行行政預審：</p>
<ul style="line-height:2">{case_list_html}</ul>
<p style="margin-top:24px">
  <a href="https://ipigsystem.asia" style="background:#2563eb;color:#fff;padding:8px 16px;border-radius:6px;text-decoration:none">
    前往 iPig 系統
  </a>
</p>
<hr style="margin-top:32px"/>
<p style="color:#94a3b8;font-size:12px">此信由 iPig 系統自動發送，請勿直接回覆</p>
</body></html>"#
        );
        let body_plain = format!(
            "IACUC 新送審案件通知\n\n{case_list_plain}\n\n請至 https://ipigsystem.asia 進行行政預審。"
        );

        let smtp = EmailService::resolve_smtp(db, config).await;
        let recipients: Vec<&str> = notify_emails
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        for addr in &recipients {
            if let Err(e) = EmailService::send_email_smtp(
                &smtp, addr, "IACUC 執行秘書", &subject, &body_plain, &body_html,
            )
            .await
            {
                error!("[IACUC] 發送通知至 {} 失敗: {}", addr, e);
            }
        }

        info!(
            "[IACUC] 送審通知已發送：{} 件新案 → {} 位收件人",
            count,
            recipients.len()
        );
        Ok(())
    }
}
