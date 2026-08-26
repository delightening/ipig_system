use chrono::{Datelike, Timelike, Weekday};
use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{
    config::Config,
    constants::SCHEDULER_LEADER_LOCK_ID,
    middleware::ActorContext,
    services::notification::{DispatchOutcome, StaffEmail},
    services::{
        BalanceExpirationJob, CalendarService, EmailService, EuthanasiaService, InvitationService,
        NotificationService, PartitionMaintenanceJob, RetentionEnforcer, SecurityNotification,
        SecurityNotifier, SessionManager,
    },
};

type SchedulerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// R26-2 audit HMAC chain 每日驗證 cron schedule（每日 02:00:00 UTC）。
const AUDIT_CHAIN_VERIFY_CRON: &str = "0 0 2 * * *";

/// R30-17 retention enforcement cron schedule（每日 03:00:00 UTC，避開 audit_chain_verify 02:00）。
const RETENTION_ENFORCER_CRON: &str = "0 0 3 * * *";

/// R39 vet patrol draft GC cron schedule（每日 03:35:00 UTC）。
/// 03:00 retention enforcer / 03:30 db_maintenance_analyze 之後 5 分鐘空檔，
/// 避免 GC 多表 DELETE 與 ANALYZE 同時跑造成 lock contention。
/// 清掉 7 天未動的 draft 報告（auto-save 留下的孤兒）。
const VET_PATROL_DRAFT_GC_CRON: &str = "0 35 3 * * *";

/// R40-A 站內信 GC：每日 03:40 UTC，hard delete 軟刪 ≥30 天的訊息 + unlink 附件
const MESSAGING_GC_CRON: &str = "0 40 3 * * *";

/// signature bridge GC：每日 03:45 UTC，刪掉 CONSUMED/EXPIRED、過 TTL 的 PENDING、
/// 以及 COMPLETED 但 1 小時未被桌機取走的棄單 session，縮短明文 payload at-rest 殘留。
const SIGNATURE_BRIDGE_GC_CRON: &str = "0 45 3 * * *";

/// 置頂待辦對帳：每日 03:50 UTC（＝台灣 11:50），排在其他 GC 之後。
/// 降級「置頂中、但關聯業務實體已不存在 / 已刪 / 已在終態」的通知。
/// 存在理由見 `services/notification/reconcile.rs`：待辦的解除掛在各流程手寫的終態
/// 路徑上，漏接一條使用者就永久卡住且無法自救（2026-08-07 事故）。本作業是安全網。
const PINNED_TODO_RECONCILE_CRON: &str = "0 50 3 * * *";

/// R28-5 audit HMAC legacy backfill 監控 cron — 每 10 分鐘更新 gauge
/// `ipig_audit_hmac_legacy_rows{version="null"}`，目標 → 0。
const HMAC_LEGACY_GAUGE_CRON: &str = "0 */10 * * * *";

/// 2026-05-18: session idle cleanup — 每 5 分鐘檢查 idle 過久的 session 並關閉。
/// 過去 cleanup_expired 從未被呼叫 → server-side idle timeout 形同虛設，搭配
/// 前端 6h 倒數造成「<6h 被登出」的問題。
const SESSION_CLEANUP_CRON: &str = "0 */5 * * * *";
/// Fallback inactivity threshold（分鐘）— DB `system_settings.session_timeout_minutes`
/// 缺值或讀取失敗時使用。對齊 migration 068。
const SESSION_CLEANUP_FALLBACK_MINUTES: i32 = 480;

/// R28-5 監控 metric 名稱與 label（避免散落字串）。
const METRIC_AUDIT_HMAC_LEGACY_ROWS: &str = "ipig_audit_hmac_legacy_rows";
const METRIC_LABEL_VERSION: &str = "version";
const METRIC_LABEL_VERSION_NULL: &str = "null";

pub struct SchedulerService;

impl SchedulerService {
    /// R63-C3: 嘗試取得 scheduler leader advisory lock。
    ///
    /// 回傳 `Ok(true)` = 本 instance 為 leader（已 spawn 背景任務在 shutdown 時釋放鎖）；
    /// `Ok(false)` = 其他 instance 持有鎖；`Err` = DB/查詢錯誤（**不可**靜默降級為非
    /// leader，否則 DB 暫時不可用時所有 instance 都跳過排程且無錯誤日誌）。
    ///
    /// 用 session-level advisory lock：持有 connection 存活 = lock 存活。pooled
    /// connection 的 drop 只歸還連線、不關閉實體連線，session lock 會殘留直到連線
    /// 真正關閉，故 shutdown 時顯式 `pg_advisory_unlock` 再歸還。
    async fn acquire_leader_lock(
        db: &PgPool,
        shutdown_token: &CancellationToken,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut leader_conn = db.acquire().await?;
        let is_leader: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
            .bind(SCHEDULER_LEADER_LOCK_ID)
            .fetch_one(&mut *leader_conn)
            .await
            .map_err(|e| {
                error!("[Scheduler] 取得 leader advisory lock 失敗: {e}");
                e
            })?;

        if is_leader {
            let token_hold = shutdown_token.clone();
            tokio::spawn(async move {
                token_hold.cancelled().await;
                if let Err(e) = sqlx::query("SELECT pg_advisory_unlock($1)")
                    .bind(SCHEDULER_LEADER_LOCK_ID)
                    .execute(&mut *leader_conn)
                    .await
                {
                    error!("[Scheduler] 釋放 leader advisory lock 失敗: {e}");
                }
                drop(leader_conn);
                info!("[Scheduler] Leader advisory lock released");
            });
        }

        Ok(is_leader)
    }

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

        // R63-C3: Leader election — 多 instance 部署時只有一個跑排程。
        if !Self::acquire_leader_lock(&db, &shutdown_token).await? {
            info!("[Scheduler] Another instance holds the scheduler leader lock — skipping job registration");
            sched.start().await?;
            return Ok(sched);
        }

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
        Self::register_surgery_sales_audit_job(&sched, &db, t, &mut job_count).await?;
        Self::register_equipment_overdue_job(&sched, &db, t, &mut job_count).await?;
        Self::register_monthly_report_job(&sched, &db, t, &mut job_count).await?;
        Self::register_invitation_expiry_job(&sched, &db, t, &mut job_count).await?;
        Self::register_db_analyze_job(&sched, &db, t, &mut job_count).await?;
        Self::register_iacuc_submission_notify_job(&sched, &db, t, &mut job_count).await?;
        Self::register_unresolved_alert_sweep_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_audit_chain_verify_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_retention_enforcer_job(&sched, &db, &config, t, &mut job_count).await?;
        Self::register_vet_patrol_draft_gc_job(&sched, &db, t, &mut job_count).await?;
        Self::register_messaging_gc_job(&sched, &db, t, &mut job_count).await?;
        Self::register_signature_bridge_gc_job(&sched, &db, t, &mut job_count).await?;
        Self::register_pinned_todo_reconcile_job(&sched, &db, t, &mut job_count).await?;
        Self::register_hmac_legacy_gauge_job(&sched, &db, t, &mut job_count).await?;
        Self::register_backup_admin_reminder_job(&sched, &db, t, &mut job_count).await?;
        Self::register_session_cleanup_job(&sched, &db, t, &mut job_count).await?;

        sched.start().await?;
        info!(
            "[Scheduler] ✓ All {} jobs registered and scheduler started successfully",
            job_count
        );

        Ok(sched)
    }

    // ── Job 註冊 helpers ──

    /// 每小時整點觸發低庫存檢查（由 DB routing 設定決定實際執行時機）
    async fn register_low_stock_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn register_expiry_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn register_notification_cleanup_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn register_balance_expiration_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                        info!(
                            "Balance expiration check completed: {} annual, {} comp_time expired",
                            summary.annual_leave_expired, summary.comp_time_expired
                        );
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
    ///
    /// R26-1：長工作改以 `tokio::select!` 與 shutdown token 並跑；shutdown 時
    /// 立即中止目前正在進行的 HTTP 呼叫（Google API 可能分頁，單輪可能數十
    /// 秒），不再等整輪結束。
    async fn register_calendar_sync_jobs(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (cron, label) in [("0 0 8 * * *", "morning"), ("0 0 18 * * *", "evening")] {
            let db_clone = db.clone();
            let token_outer = token.clone();
            let job = Job::new_async(cron, move |_uuid, _l| {
                let db = db_clone.clone();
                let token = token_outer.clone();
                Box::pin(async move {
                    if token.is_cancelled() {
                        info!(
                            "[Scheduler] calendar_sync_{} skipped during shutdown",
                            label
                        );
                        return;
                    }
                    info!("Running scheduled calendar sync ({})...", label);
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => {
                            info!("[Scheduler] calendar_sync_{} interrupted by shutdown", label);
                        }
                        result = CalendarService::trigger_sync(&db, None) => {
                            match result {
                                Ok(history) => info!("Calendar sync completed: {:?}", history.status),
                                Err(e) => error!("Calendar sync failed: {}", e),
                            }
                        }
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
    async fn register_partition_maintenance_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                        info!(
                            "Partition maintenance completed: {} checked, {} created, {} existing",
                            result.checked, result.created, result.existing
                        );
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
    async fn register_euthanasia_timeout_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn register_po_pending_receipt_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    /// 每日 09:30 稽核：有手術但時間窗內缺對應銷貨單據(DO/SO)的計畫 → 通知 SD + 倉管
    async fn register_surgery_sales_audit_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 30 9 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] surgery_sales_audit skipped during shutdown");
                    return;
                }
                info!("Running daily surgery-sales compliance audit...");
                if let Err(e) = Self::check_surgery_sales_compliance(&db).await {
                    error!("Surgery-sales compliance audit failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'surgery_sales_audit' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 08:30 檢查設備校正/確效逾期
    async fn register_equipment_overdue_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    /// 每月 1 日 06:00 生成月報（R26-1：長工作 + tokio::select! cancellation）
    async fn register_monthly_report_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                tokio::select! {
                    biased;
                    _ = token.cancelled() => {
                        info!("[Scheduler] monthly_report interrupted by shutdown");
                    }
                    result = Self::generate_monthly_report(&db) => {
                        if let Err(e) = result {
                            error!("Monthly report generation failed: {}", e);
                        }
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'monthly_report' registered");
        *count += 1;
        Ok(())
    }

    /// 每日 04:00 清理過期邀請
    async fn register_invitation_expiry_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    /// 每兩小時檢查新送審 IACUC 計畫書並通知執行秘書
    /// （cron `0 0 */2 * * *` = UTC 偶數整點，即台灣時間每日 08/10/12…/06 時，全日全週；非僅平日、非 07:00 起）
    async fn register_iacuc_submission_notify_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] iacuc_submission_notify skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::check_iacuc_new_submissions(&db).await {
                    error!("[IACUC] Submission notify job failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'iacuc_submission_notify' registered (every 2h, 24/7, even UTC hours)");
        *count += 1;
        Ok(())
    }

    /// 每天 03:30 執行 ANALYZE
    /// 每日 03:30 ANALYZE 高寫入表（R26-1：長工作 + tokio::select! cancellation）
    ///
    /// 注意：drop sqlx 的 in-flight future 只會關閉客戶端連線；PostgreSQL 端
    /// 的 VACUUM ANALYZE 仍會繼續完成或由 `statement_timeout` 中止。對 shutdown
    /// 而言足夠：服務立刻下線，DB 自行收尾。
    async fn register_db_analyze_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                tokio::select! {
                    biased;
                    _ = token.cancelled() => {
                        info!("[Scheduler] db_maintenance_analyze interrupted by shutdown");
                    }
                    result = sqlx::query("SELECT maintenance_vacuum_analyze()").execute(&db) => {
                        if let Err(e) = result {
                            error!("Scheduled ANALYZE failed: {}", e);
                        }
                    }
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
        let job = Job::new_async(AUDIT_CHAIN_VERIFY_CRON, move |_uuid, _l| {
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

    /// R30-17：每日 03:00 UTC 執行 retention enforcement。
    ///
    /// 找出 `data_retention_policies` 中標 `hard_delete` 且已過 retention_years 的
    /// soft-deleted row 真實刪除；對 `partition_drop` 表 (e.g. user_activity_logs)
    /// 走 DETACH PARTITION + DROP，避開 R30-F BEFORE DELETE trigger。
    ///
    /// 執行結果寫入 `RETENTION_ENFORCEMENT_RUN` audit event（actor = SystemActor）。
    async fn register_retention_enforcer_job(
        sched: &JobScheduler,
        db: &PgPool,
        config: &Arc<Config>,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let config_clone = config.clone();
        let token_outer = token.clone();
        let job = Job::new_async(RETENTION_ENFORCER_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] retention_enforcer skipped during shutdown");
                    return;
                }
                if !config.retention_enforcer_enabled {
                    info!(
                        "[Scheduler] retention_enforcer disabled (RETENTION_ENFORCER_ENABLED=false); \
                         skipping. Enable after staging dry-run review."
                    );
                    return;
                }
                info!("[Scheduler] Running daily retention enforcer...");
                if let Err(e) = RetentionEnforcer::run(&db).await {
                    error!("[R30-17] retention_enforcer failed: {e}");
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'retention_enforcer' registered (daily 03:00 UTC)");
        *count += 1;
        Ok(())
    }

    /// R39：每日 03:30 UTC 清理超過 7 天未動的草稿巡場報告。
    ///
    /// 動機：dialog 改 auto-save pattern 後，使用者開了 dialog 但取消／關閉而沒有
    /// 送出的報告會以 `status='draft'` 留在 DB，需週期性清理避免堆積。
    /// CASCADE 連帶清掉 entries / entry_photos / report photos。
    async fn register_vet_patrol_draft_gc_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(VET_PATROL_DRAFT_GC_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] vet_patrol_draft_gc skipped during shutdown");
                    return;
                }
                info!("[Scheduler] Running vet_patrol draft GC...");
                match crate::services::VetPatrolReportService::cleanup_stale_drafts(&db).await {
                    Ok((0, _)) => info!("[Scheduler] vet_patrol_draft_gc: no stale drafts"),
                    Ok((n, 0)) => info!(
                        "[Scheduler] vet_patrol_draft_gc: deleted {n} stale drafts (all files unlinked)"
                    ),
                    Ok((n, fails)) => tracing::warn!(
                        "[Scheduler] vet_patrol_draft_gc: deleted {n} stale drafts; {fails} files failed to unlink"
                    ),
                    Err(e) => error!("[Scheduler] vet_patrol_draft_gc failed: {e}"),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'vet_patrol_draft_gc' registered (daily 03:35 UTC)");
        *count += 1;
        Ok(())
    }

    /// 置頂待辦對帳（安全網）：每日 03:50 UTC 降級孤兒待辦。
    ///
    /// 「待處理」清單的正確性取決於各業務流程有沒有在終態呼叫
    /// `resolve_pinned_notifications`。那些呼叫是手寫的，每新增一條終態路徑
    /// （撤回 / 作廢 / 刪除 / 轉單…）就多一次漏接機會；漏接的後果是使用者的待辦
    /// 永久卡死，且依設計無法手動已讀清除。本作業定期補救。
    ///
    /// 不違反「待辦不可手動略過」——降級依據是業務實體的真實狀態，不是使用者意願。
    async fn register_pinned_todo_reconcile_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(PINNED_TODO_RECONCILE_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] pinned_todo_reconcile skipped during shutdown");
                    return;
                }
                info!("[Scheduler] Running pinned todo reconcile...");
                let svc = crate::services::NotificationService::new(db);
                match svc.reconcile_pinned_notifications(false).await {
                    Ok(r) => {
                        if r.resolved.is_empty() {
                            info!(
                                "[Scheduler] pinned_todo_reconcile: 無孤兒待辦（在途 {} 筆）",
                                r.still_pending
                            );
                        } else {
                            // 有命中代表某條終態路徑漏接了解除 hook —— 用 warn 讓它在日誌中顯眼。
                            // 對帳只是止血，真正該修的是漏掉的那條路徑，所以逐筆印出 reason。
                            tracing::warn!(
                                "[Scheduler] pinned_todo_reconcile: 降級 {} 筆孤兒待辦（在途 {} 筆）— 表示有終態路徑漏接解除 hook",
                                r.resolved.len(),
                                r.still_pending
                            );
                            for row in &r.resolved {
                                // 刻意不記 email：本 warn 會進營運日誌（留存久、存取範圍比 DB 廣），
                                // email 屬直接識別個人的資料。需要對應到人時用 user_id 查 users 表 ——
                                // 那是有存取控管的路徑。
                                tracing::warn!(
                                    "[Scheduler] pinned_todo_reconcile 降級: notification={} entity={} entity_id={:?} user={} reason={}",
                                    row.id,
                                    row.related_entity_type,
                                    row.related_entity_id,
                                    row.user_id,
                                    row.reason
                                );
                            }
                        }
                        // 認不得的 entity_type 保守未處理 —— 必須讓維運者看見。否則新增待辦
                        // 類型後對帳會靜靜跳過它，而「沒有 warn」看起來跟「一切正常」一模一樣。
                        for (ty, n) in &r.unknown_entity_types {
                            tracing::warn!(
                                "[Scheduler] pinned_todo_reconcile 未涵蓋的 entity_type: {ty} — {n} 筆置頂待辦未做判斷，請補進 services/notification/reconcile.rs"
                            );
                        }
                        // NULL entity_id：無業務實體可追溯，本作業無從判斷，會永遠留在
                        // 待處理清單。不印出來就沒人知道那幾筆為什麼一直在。
                        if r.null_entity_id > 0 {
                            tracing::warn!(
                                "[Scheduler] pinned_todo_reconcile: {} 筆置頂待辦的 related_entity_id 為 NULL —— 無業務實體可追溯，無從判斷，將永遠留在待處理清單，需人工決定處置",
                                r.null_entity_id
                            );
                        }
                    }
                    // 這支作業是「待辦卡死」這類 bug 唯一的自動偵測手段；它自己死掉而沒人
                    // 發現的話，等於偵測層靜默失效。error! 讓它在 Loki 顯眼。
                    // TODO(ops): 目前 alert_rules.yml 沒有任何 scheduler job 的告警規則，
                    // 本作業連續失敗不會觸發通知。補 gauge + alert 屬 follow-up。
                    Err(e) => error!("[Scheduler] pinned_todo_reconcile failed: {e}"),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'pinned_todo_reconcile' registered (daily 03:50 UTC)");
        *count += 1;
        Ok(())
    }

    /// R40-A：每日 03:40 UTC hard delete 軟刪 ≥30 天的訊息 + unlink 附件實體檔案
    async fn register_messaging_gc_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(MESSAGING_GC_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] messaging_gc skipped during shutdown");
                    return;
                }
                info!("[Scheduler] Running messaging GC...");
                match crate::services::MessagingService::cleanup_soft_deleted_messages(&db).await {
                    Ok((0, _)) => info!("[Scheduler] messaging_gc: no stale messages"),
                    Ok((n, 0)) => info!(
                        "[Scheduler] messaging_gc: hard-deleted {n} messages (all attachments unlinked)"
                    ),
                    Ok((n, fails)) => tracing::warn!(
                        "[Scheduler] messaging_gc: hard-deleted {n} messages; {fails} attachment files failed to unlink"
                    ),
                    Err(e) => error!("[Scheduler] messaging_gc failed: {e}"),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'messaging_gc' registered (daily 03:40 UTC)");
        *count += 1;
        Ok(())
    }

    /// signature bridge GC：每日清掉已無用 / 棄置的簽名 bridge session，
    /// 縮短 payload 明文（密碼 + 手寫筆跡）at-rest 殘留。見
    /// `SignatureBridgeService::cleanup_stale_sessions`。
    async fn register_signature_bridge_gc_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(SIGNATURE_BRIDGE_GC_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] signature_bridge_gc skipped during shutdown");
                    return;
                }
                info!("[Scheduler] Running signature bridge GC...");
                match crate::services::SignatureBridgeService::cleanup_stale_sessions(&db).await {
                    Ok(0) => info!("[Scheduler] signature_bridge_gc: no stale sessions"),
                    Ok(n) => info!("[Scheduler] signature_bridge_gc: deleted {n} stale sessions"),
                    Err(e) => error!("[Scheduler] signature_bridge_gc failed: {e}"),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'signature_bridge_gc' registered (daily 03:45 UTC)");
        *count += 1;
        Ok(())
    }

    /// R28-5：每 10 分鐘更新 `ipig_audit_hmac_legacy_rows{version="null"}` gauge。
    /// 用途：監控 backfill 進度（目標 → 0），到 0 持續 30 天後即可移除 verifier
    /// 的 try-both fallback（見 docs/security/HMAC_VERSIONING.md §3 階段 C）。
    async fn register_hmac_legacy_gauge_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(HMAC_LEGACY_GAUGE_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    return;
                }
                match crate::repositories::audit_log::AuditLogRepository::count_legacy_hmac_rows(
                    &db,
                )
                .await
                {
                    Ok(n) => {
                        metrics::gauge!(
                            METRIC_AUDIT_HMAC_LEGACY_ROWS,
                            METRIC_LABEL_VERSION => METRIC_LABEL_VERSION_NULL,
                        )
                        .set(n as f64);
                    }
                    Err(e) => {
                        error!("[R28-5] hmac_legacy gauge update failed: {e}");
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'hmac_legacy_gauge' registered (every 10 min)");
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
    async fn should_run_now(
        db: &PgPool,
        event_type: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
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

        // 統一用台灣時間（notification_routing.hour_of_day 以台灣時間為準），
        // 避免 `Local::now()` 依容器 TZ 環境變數而誤判排程時刻。
        let now = crate::time::now_taiwan();
        let current_hour = now.hour() as i16;
        let current_dow = now.weekday().num_days_from_sunday() as i16;
        let current_day = now.day();

        for (frequency, hour_of_day, day_of_week) in rows {
            let matches = match frequency.as_str() {
                "daily" => hour_of_day == current_hour,
                "weekly" => hour_of_day == current_hour && day_of_week == Some(current_dow),
                "monthly" => hour_of_day == current_hour && current_day == 1,
                _ => false,
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

        let users = service.stock_email_recipients("low_stock_alert").await?;
        let alerts_html = Self::build_low_stock_html(&alerts.data);
        let email_count =
            Self::send_low_stock_emails(db, config, &users, &alerts_html, alerts.data.len()).await;

        info!(
            "Low stock check completed: {} alerts, {} emails sent",
            alerts.data.len(),
            email_count
        );
        Ok(())
    }

    /// 檢查效期並發送通知（含月度閾值邏輯）
    async fn check_expiry(db: &PgPool, config: &Config) -> SchedulerResult {
        use crate::services::notification::expiry_monthly::{current_ym, previous_ym};

        let service = NotificationService::new(db.clone());
        let cfg = service.get_expiry_notification_config().await?;

        let alerts: Vec<crate::models::ExpiryAlert> =
            sqlx::query_as("SELECT * FROM fn_expiry_alerts($1, $2)")
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
                    Ok(diff) => match service
                        .send_monthly_expiry_comparison(&diff, &ym, threshold)
                        .await
                    {
                        Ok(c) => info!("Monthly expiry comparison notifications: {} sent", c),
                        Err(e) => tracing::warn!("月度效期比較通知發送失敗: {e}"),
                    },
                    Err(e) => tracing::warn!("月度快照比較失敗: {e}"),
                }
            }
        }

        // 一般效期通知（排除月度閾值範圍的品項）
        let threshold_days = cfg.monthly_threshold_days.unwrap_or(i16::MAX);
        let regular_alerts: Vec<_> = alerts
            .into_iter()
            .filter(|a| a.days_until_expiry >= -(threshold_days as i32))
            .collect();

        if regular_alerts.is_empty() {
            return Ok(());
        }

        let expired_count = regular_alerts
            .iter()
            .filter(|a| a.expiry_status == "expired")
            .count();
        let expiring_count = regular_alerts
            .iter()
            .filter(|a| a.expiry_status == "expiring_soon")
            .count();

        // R82-11：in-app 通知改用同一份 config-aware 的 regular_alerts（與下方 email 一致），
        // 不再獨立查寫死視窗的 v_expiry_alerts，確保管理員設定的 warn/cutoff 天數對兩通道皆生效。
        match service.send_expiry_notifications(&regular_alerts).await {
            Ok(count) => info!("Expiry in-app notifications: {} sent", count),
            Err(e) => tracing::warn!("發送效期預警站內通知失敗: {e}"),
        }

        let users = service.stock_email_recipients("expiry_alert").await?;
        let alerts_html = Self::build_expiry_html(&regular_alerts);
        let email_count = Self::send_expiry_emails(
            db,
            config,
            &users,
            &alerts_html,
            expired_count,
            expiring_count,
        )
        .await;

        info!(
            "Expiry check completed: {} alerts ({} expired, {} expiring), {} emails sent",
            regular_alerts.len(),
            expired_count,
            expiring_count,
            email_count
        );
        Ok(())
    }

    /// 排入低庫存 email（員工通知 → 經時間窗 / 請假 chokepoint），回傳已排入數。
    async fn send_low_stock_emails(
        db: &PgPool,
        config: &Config,
        users: &[(uuid::Uuid, String, String)],
        alerts_html: &str,
        alert_count: usize,
    ) -> usize {
        let service = NotificationService::new(db.clone());
        let actor = ActorContext::System {
            reason: "low_stock_notification",
        };
        let mut email_count = 0;
        for (user_id, email, name) in users {
            let rendered = EmailService::render_low_stock_alert_email(
                &config.app_url,
                name,
                alerts_html,
                alert_count,
            );
            match service
                .dispatch_staff_email(
                    &actor,
                    ("inventory", *user_id),
                    StaffEmail {
                        to_email: email.clone(),
                        to_name: name.clone(),
                        recipient_user_id: Some(*user_id),
                        email: rendered,
                    },
                )
                .await
            {
                Ok(o) if o != DispatchOutcome::SkippedOnLeave => email_count += 1,
                Ok(_) => {}
                Err(e) => error!("Failed to dispatch low stock email to {}: {}", email, e),
            }
        }
        email_count
    }

    /// 排入效期預警 email（員工通知 → 經時間窗 / 請假 chokepoint），回傳已排入數。
    async fn send_expiry_emails(
        db: &PgPool,
        config: &Config,
        users: &[(uuid::Uuid, String, String)],
        alerts_html: &str,
        expired_count: usize,
        expiring_count: usize,
    ) -> usize {
        let service = NotificationService::new(db.clone());
        let actor = ActorContext::System {
            reason: "expiry_notification",
        };
        let mut email_count = 0;
        for (user_id, email, name) in users {
            let rendered = EmailService::render_expiry_alert_email(
                &config.app_url,
                name,
                alerts_html,
                expired_count,
                expiring_count,
            );
            match service
                .dispatch_staff_email(
                    &actor,
                    ("inventory", *user_id),
                    StaffEmail {
                        to_email: email.clone(),
                        to_name: name.clone(),
                        recipient_user_id: Some(*user_id),
                        email: rendered,
                    },
                )
                .await
            {
                Ok(o) if o != DispatchOutcome::SkippedOnLeave => email_count += 1,
                Ok(_) => {}
                Err(e) => error!("Failed to dispatch expiry email to {}: {}", email, e),
            }
        }
        email_count
    }

    /// 清理過期通知
    async fn cleanup_notifications(db: &PgPool) -> SchedulerResult {
        let service = NotificationService::new(db.clone());
        let deleted = service.cleanup_old_notifications().await?;
        info!(
            "Notification cleanup completed: {} old notifications deleted",
            deleted
        );
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
            <tbody>"#,
        );

        for alert in alerts.iter().take(20) {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{} {}</td><td>{}</td></tr>",
                alert.product_sku,
                alert.product_name,
                alert.warehouse_name,
                alert.qty_on_hand,
                alert.base_uom,
                alert
                    .safety_stock
                    .map(|s| s.to_string())
                    .unwrap_or("-".to_string()),
            ));
        }

        html.push_str("</tbody></table>");
        if alerts.len() > 20 {
            html.push_str(&format!(
                "<p>...另外還有 {} 項，請登入系統查看完整列表</p>",
                alerts.len() - 20
            ));
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
            <tbody>"#,
        );

        for alert in alerts.iter().take(20) {
            let status_class = if alert.expiry_status == "expired" {
                "expired"
            } else {
                "expiring"
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td>\
                 <td class=\"{}\">{}</td><td>{} {}</td><td>{} {}</td></tr>",
                alert.sku,
                alert.product_name,
                alert.batch_no.as_deref().unwrap_or("-"),
                alert.expiry_date,
                status_class,
                alert.days_until_expiry,
                alert.on_hand_qty,
                alert.base_uom,
                alert.total_qty,
                alert.base_uom,
            ));
        }

        html.push_str("</tbody></table>");
        if alerts.len() > 20 {
            html.push_str(&format!(
                "<p>...另外還有 {} 項，請登入系統查看完整列表</p>",
                alerts.len() - 20
            ));
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
        info!(
            "PO pending receipt check completed: {} notifications sent",
            count
        );
        Ok(())
    }

    /// 手動觸發採購單未入庫檢查（供 API 使用）
    pub async fn trigger_po_pending_receipt_check(db: &PgPool) -> SchedulerResult {
        Self::check_po_pending_receipt(db).await
    }

    /// 稽核有手術但缺對應銷貨單據的計畫，通知 SD + 倉管
    async fn check_surgery_sales_compliance(db: &PgPool) -> SchedulerResult {
        let service = NotificationService::new(db.clone());
        let count = service.notify_surgery_missing_sales().await?;
        info!(
            "Surgery-sales compliance audit completed: {} notifications sent",
            count
        );
        Ok(())
    }

    /// 手動觸發手術缺銷貨單據稽核（供 API 使用）
    pub async fn trigger_surgery_sales_audit(db: &PgPool) -> SchedulerResult {
        Self::check_surgery_sales_compliance(db).await
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
            &month_str,
            po_count,
            &po_amount,
            so_count,
            &so_amount,
            &blood_test_stats,
        );

        let count = Self::send_report_notifications(db, &month_str, &content).await?;

        info!(
            "[Monthly Report] {}報表已產出並發送給 {} 位使用者（PO: {}, SO: {}, 血檢項: {}）",
            month_str,
            count,
            po_count,
            so_count,
            blood_test_stats.len()
        );
        Ok(())
    }

    /// 計算上月的起迄日期
    fn compute_previous_month_range() -> Result<
        (chrono::NaiveDate, chrono::NaiveDate, String),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        use chrono::{Datelike, NaiveDate};

        let now = crate::time::today_taiwan_naive();
        let year = if now.month() == 1 {
            now.year() - 1
        } else {
            now.year()
        };
        let month = if now.month() == 1 {
            12
        } else {
            now.month() - 1
        };
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
                .ok_or_else(|| {
                    format!(
                        "failed to get last day of {}-{}",
                        now.year(),
                        now.month() - 1
                    )
                })?
        };
        let month_str = format!("{}年{}月", year, month);
        Ok((first_day, last_day, month_str))
    }

    /// 查詢採購彙總
    async fn query_purchase_summary(
        db: &PgPool,
        first_day: chrono::NaiveDate,
        last_day: chrono::NaiveDate,
    ) -> Result<(i64, Option<rust_decimal::Decimal>), Box<dyn std::error::Error + Send + Sync>>
    {
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
    ) -> Result<(i64, Option<rust_decimal::Decimal>), Box<dyn std::error::Error + Send + Sync>>
    {
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
            month_str,
            po_count,
            po_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
            so_count,
            so_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
        );

        if !blood_test_stats.is_empty() {
            content.push_str("\n=== 血液檢查統計 ===\n");
            for (iacuc_no, item_name, cnt) in blood_test_stats {
                content.push_str(&format!(
                    "計畫 {}：{} × {} 次\n",
                    iacuc_no.as_deref().unwrap_or("-"),
                    item_name,
                    cnt,
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
        let mut recipients = service
            .get_users_by_role(crate::constants::ROLE_WAREHOUSE_MANAGER)
            .await?;
        let admins = service.get_admin_users().await?;
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

    /// R36 backup 例行檢查提醒：每天 09:00 UTC（17:00 台灣時間）跑一次，
    /// 依日期決定是否發 SystemAlert 通知所有 SYSTEM_ADMIN。
    /// 月度 / 季度 / 年度 / 3 年 / 5 年 5 種週期見下方 helper。
    async fn register_backup_admin_reminder_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        // 09:00 UTC = 17:00 Taipei，業務時間結束前提醒
        let job = Job::new_async("0 0 9 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] backup_admin_reminders skipped during shutdown");
                    return;
                }
                if let Err(e) = Self::check_backup_admin_reminders(&db).await {
                    error!("[Scheduler] backup_admin_reminders failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'backup_admin_reminders' registered (daily 09:00 UTC / 17:00 Taipei)");
        *count += 1;
        Ok(())
    }

    /// 檢查今天日期，決定要發哪些 backup 維運提醒給 admin。
    /// 對應 docs/runbooks/backup-private-key-handling.md §7「例行檢查」。
    async fn check_backup_admin_reminders(db: &PgPool) -> SchedulerResult {
        let taipei = chrono::FixedOffset::east_opt(8 * 3600).expect("valid offset");
        let today = chrono::Utc::now().with_timezone(&taipei).date_naive();
        let day = today.day();
        let month = today.month();

        let service = NotificationService::new(db.clone());
        let admins = service.get_admin_users().await?;
        // 這條防呆在修好角色代碼之前是**恆真**的（`get_users_by_role("SYSTEM_ADMIN")`
        // 永遠回空），所以備份提醒從來沒發出去過。保留它——修好之後它才第一次
        // 真正發揮作用（系統真的沒有管理員時才跳過）。
        if admins.is_empty() {
            info!("[BackupReminder] no admin users to notify, skipping");
            return Ok(());
        }

        let mut reminders: Vec<(&str, String)> = Vec::new();

        // 月度（每月 1 號）— heartbeat 健康檢查
        if day == 1 {
            reminders.push((
                "[Backup] 月度健康檢查",
                "請確認：\n1) Grafana backup_last_success_timestamp_seconds 持續更新中（< 25h）\n2) 抽 R2 / NAS-B 任一處最新 backup 確認檔案存在\n3) 詳見 docs/runbooks/backup-private-key-handling.md §7".to_string(),
            ));
        }

        // 季度 DR Drill（每季第一個月 1 號 = 1/1, 4/1, 7/1, 10/1）
        if day == 1 && [1, 4, 7, 10].contains(&month) {
            reminders.push((
                "[Backup] 季度 DR Drill",
                "請執行完整 backup restore 演練（依 docs/runbooks/backup-setup.md Step 6）：\n- R2 下載 → 解密 → pg_restore → row-count 比對\n- 完成後紀錄到 docs/runbooks/dr-drill-records.md §5".to_string(),
            ));
        }

        // 年度 — 遠端 USB 可讀性（每年 1/1）
        if day == 1 && month == 1 {
            reminders.push((
                "[Backup] 年度遠端 USB 可讀性檢查",
                "請聯繫離家 USB 保管人，請其插入 USB 並回報是否能 ls 出 backup_gpg_privkey.asc。USB 接頭 2-3 年起可能氧化讀不到。".to_string(),
            ));
        }

        // 3 年 USB 輪替提醒（GPG keypair 產於 2026-05-08；建議 2029-05 起每月 1 號提醒到換完）
        let usb_rotation_due = chrono::NaiveDate::from_ymd_opt(2029, 5, 1).expect("valid date");
        if today >= usb_rotation_due && day == 1 {
            reminders.push((
                "[Backup] USB 已使用 3 年，建議更換",
                "GPG keypair 產於 2026-05-08，USB 已使用 ≥ 3 年。建議：採購新 USB → 從舊 USB 複製私鑰 → 驗證可讀 → 物理銷毀舊 USB。完成後在 backup-private-key-handling.md §9 寫變更紀錄。".to_string(),
            ));
        }

        // 5 年 passphrase 輪替提醒（建議 2031-05 起每月 1 號）
        let passphrase_rotation_due =
            chrono::NaiveDate::from_ymd_opt(2031, 5, 1).expect("valid date");
        if today >= passphrase_rotation_due && day == 1 {
            reminders.push((
                "[Backup] GPG passphrase 已 5 年，建議輪替",
                "GPG passphrase 5 年未換，依 NIST 800-63 建議輪替。流程：插 USB → gpg --passwd → 重新匯出到兩支 USB → Bitwarden 同步更新。詳見 backup-private-key-handling.md §6。".to_string(),
            ));
        }

        if reminders.is_empty() {
            return Ok(());
        }

        info!(
            "[BackupReminder] 今日 {} 觸發 {} 條提醒，發送給 {} 位 admin",
            today,
            reminders.len(),
            admins.len()
        );

        for (title, body) in reminders {
            for (admin_id, _email, _name) in &admins {
                if let Err(e) = service
                    .create_notification(crate::models::CreateNotificationRequest {
                        user_id: *admin_id,
                        notification_type: crate::models::NotificationType::SystemAlert,
                        title: title.to_string(),
                        content: Some(body.clone()),
                        related_entity_type: Some("backup_routine_check".to_string()),
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!(
                        "[BackupReminder] create_notification 失敗 admin_id={}: {}",
                        admin_id,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// 檢查最近 150 分鐘內新送審的 IACUC 計畫書，發送 Email 通知
    async fn check_iacuc_new_submissions(db: &PgPool) -> SchedulerResult {
        // 僅在台灣時間（UTC+8）平日 07:00–15:00 執行
        let taipei = chrono::FixedOffset::east_opt(8 * 3600).expect("valid offset");
        let now_taipei = chrono::Utc::now().with_timezone(&taipei);
        let hour = now_taipei.hour();
        let is_workday = matches!(
            now_taipei.weekday(),
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri
        );
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
            SELECT p.protocol_no, p.title, u.display_name
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
        let case_list_html: String = rows
            .iter()
            .map(|(no, title, pi)| {
                format!(
                    "<li><strong>{no}</strong> — {title}（申請人：{}）</li>",
                    pi.as_deref().unwrap_or("—")
                )
            })
            .collect();
        let case_list_plain: String = rows
            .iter()
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

        let recipients: Vec<&str> = notify_emails
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        // IACUC 收件人為 system_settings 設定字串（非 users 表）→ 只套寄送時間窗，不做請假判斷。
        let service = NotificationService::new(db.clone());
        let actor = ActorContext::System {
            reason: "iacuc_submission_notification",
        };
        let rendered = crate::services::RenderedEmail {
            subject: subject.clone(),
            plain_body: body_plain.clone(),
            html_body: body_html.clone(),
        };
        for addr in &recipients {
            if let Err(e) = service
                .dispatch_staff_email(
                    &actor,
                    ("iacuc_submission", crate::middleware::SYSTEM_USER_ID),
                    StaffEmail {
                        to_email: addr.to_string(),
                        to_name: "IACUC 執行秘書".to_string(),
                        recipient_user_id: None,
                        email: rendered.clone(),
                    },
                )
                .await
            {
                error!("[IACUC] 分派通知至 {} 失敗: {}", addr, e);
            }
        }

        info!(
            "[IACUC] 送審通知已發送：{} 件新案 → {} 位收件人",
            count,
            recipients.len()
        );
        Ok(())
    }

    /// 2026-05-18: 每 5 分鐘掃描 idle 過久的 sessions 並標記 ended_reason='timeout'。
    /// 過去此 job 從未註冊，server-side idle 強制登出形同虛設。配套 migration 068
    /// 把 system_settings.session_timeout_minutes 設成 480 (8h)。
    async fn register_session_cleanup_job(
        sched: &JobScheduler,
        db: &PgPool,
        token: &CancellationToken,
        count: &mut u32,
    ) -> SchedulerResult {
        let db_clone = db.clone();
        let token_outer = token.clone();
        let job = Job::new_async(SESSION_CLEANUP_CRON, move |_uuid, _l| {
            let db = db_clone.clone();
            let token = token_outer.clone();
            Box::pin(async move {
                if token.is_cancelled() {
                    info!("[Scheduler] session_cleanup skipped during shutdown");
                    return;
                }
                // 讀 system_settings — admin 可在 UI 動態調整 idle window
                let timeout_min = sqlx::query_scalar::<_, serde_json::Value>(
                    "SELECT value FROM system_settings WHERE key = 'session_timeout_minutes'",
                )
                .fetch_optional(&db)
                .await
                .ok()
                .flatten()
                .and_then(|v| v.as_str().and_then(|s| s.parse::<i32>().ok()))
                .unwrap_or(SESSION_CLEANUP_FALLBACK_MINUTES);

                match SessionManager::cleanup_expired(&db, timeout_min).await {
                    Ok(n) if n > 0 => info!(
                        "[Scheduler] session_cleanup: ended {} idle session(s) (idle ≥ {} min)",
                        n, timeout_min
                    ),
                    Ok(_) => {}
                    Err(e) => error!("[Scheduler] session_cleanup failed: {}", e),
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'session_cleanup' registered (every 5 min)");
        *count += 1;
        Ok(())
    }
}
