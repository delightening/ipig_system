use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use sqlx::PgPool;
use tracing::{info, error};

use crate::{
    config::Config,
    services::{EmailService, NotificationService, BalanceExpirationJob, CalendarService, PartitionMaintenanceJob, EuthanasiaService},
};

pub struct SchedulerService;

impl SchedulerService {
    /// 啟動排程服務
    pub async fn start(db: PgPool, config: Arc<Config>) -> Result<JobScheduler, Box<dyn std::error::Error + Send + Sync>> {
        let sched = JobScheduler::new().await?;
        let mut job_count = 0;

        // 每日 08:00 執行低庫存檢查
        let db_clone = db.clone();
        let config_clone = config.clone();
        let job = Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("Running daily low stock check...");
                if let Err(e) = Self::check_low_stock(&db, &config).await {
                    error!("Low stock check failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'low_stock_check' registered");
        job_count += 1;

        // 每日 08:00 執行效期檢查
        let db_clone = db.clone();
        let config_clone = config.clone();
        let job = Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("Running daily expiry check...");
                if let Err(e) = Self::check_expiry(&db, &config).await {
                    error!("Expiry check failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'expiry_check' registered");
        job_count += 1;

        // 每週日 03:00 清理過期通知
        // 使用 "Sun" 作為星期日，避免數字 0 的相容性問題
        let db_clone = db.clone();
        let job = Job::new_async("0 0 3 * * Sun", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running weekly notification cleanup...");
                if let Err(e) = Self::cleanup_notifications(&db).await {
                    error!("Notification cleanup failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'notification_cleanup' registered");
        job_count += 1;

        // 每日 00:30 執行餘額到期檢查
        let db_clone = db.clone();
        let job = Job::new_async("0 30 0 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running daily balance expiration check...");
                match BalanceExpirationJob::run(&db).await {
                    Ok(summary) => {
                        info!("Balance expiration check completed: {} annual, {} comp_time expired", 
                              summary.annual_leave_expired, summary.comp_time_expired);
                    }
                    Err(e) => {
                        error!("Balance expiration check failed: {}", e);
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'balance_expiration' registered");
        job_count += 1;

        // 每日 08:00 執行 Google Calendar 同步（早上）
        let db_clone = db.clone();
        let job = Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running scheduled calendar sync (morning)...");
                match CalendarService::trigger_sync(&db, None).await {
                    Ok(history) => {
                        info!("Calendar sync completed: {:?}", history.status);
                    }
                    Err(e) => {
                        error!("Calendar sync failed: {}", e);
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'calendar_sync_morning' registered");
        job_count += 1;
        
        // 每日 18:00 執行 Google Calendar 同步（傍晚）
        let db_clone = db.clone();
        let job = Job::new_async("0 0 18 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running scheduled calendar sync (evening)...");
                match CalendarService::trigger_sync(&db, None).await {
                    Ok(history) => {
                        info!("Calendar sync completed: {:?}", history.status);
                    }
                    Err(e) => {
                        error!("Calendar sync failed: {}", e);
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'calendar_sync_evening' registered");
        job_count += 1;

        // 每年 12 月 1 日 03:00 執行分區表維護
        // 確保 user_activity_logs 表有未來 2 年的季度分區
        let db_clone = db.clone();
        let job = Job::new_async("0 0 3 1 12 *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running annual partition maintenance...");
                match PartitionMaintenanceJob::ensure_partitions(&db).await {
                    Ok(result) => {
                        info!("Partition maintenance completed: {} checked, {} created, {} existing",
                              result.checked, result.created, result.existing);
                    }
                    Err(e) => {
                        error!("Partition maintenance failed: {}", e);
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'partition_maintenance' registered");
        job_count += 1;

        // 每 5 分鐘檢查安樂死單據超時
        // 處理 PI 超時未回應和 CHAIR 仲裁超時
        // 使用 "0/5" 語法表示從 0 開始每 5 分鐘
        let db_clone = db.clone();
        let job = Job::new_async("0 0/5 * * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                match EuthanasiaService::check_expired_orders(&db).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Euthanasia timeout check: {} orders auto-approved", count);
                        }
                    }
                    Err(e) => {
                        error!("Euthanasia timeout check failed: {}", e);
                    }
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'euthanasia_timeout' registered");
        job_count += 1;

        // 每月 1 號 06:00 產出上月進銷貨+血液檢查報表
        let db_clone = db.clone();
        let job = Job::new_async("0 0 6 1 * *", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running monthly report generation...");
                if let Err(e) = Self::generate_monthly_report(&db).await {
                    error!("Monthly report generation failed: {}", e);
                }
            })
        })?;
        sched.add(job).await?;
        info!("[Scheduler] ✓ Job 'monthly_report' registered");
        job_count += 1;

        // 啟動排程器
        sched.start().await?;
        info!("[Scheduler] ✓ All {} jobs registered and scheduler started successfully", job_count);

        Ok(sched)
    }


    /// 檢查低庫存並發送通知
    async fn check_low_stock(db: &PgPool, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = NotificationService::new(db.clone());
        
        // 取得低庫存項目
        let alerts = service.list_low_stock_alerts(1, 100).await?;
        
        if alerts.data.is_empty() {
            info!("No low stock alerts found");
            return Ok(());
        }

        // 取得需要通知的使用者
        let users: Vec<(uuid::Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_low_stock = true
            "#,
        )
        .fetch_all(db)
        .await?;

        // 建構 HTML 表格
        let alerts_html = Self::build_low_stock_html(&alerts.data);

        // 發送通知
        let mut notification_count = 0;
        for (_user_id, email, name) in users {
            // 建立站內通知
            if let Err(e) = service.send_low_stock_notifications().await {
                tracing::warn!("發送庫存不足通知失敗: {e}");
            }

            
            // 發送 Email
            if let Err(e) = EmailService::send_low_stock_alert_email(
                config,
                &email,
                &name,
                &alerts_html,
                alerts.data.len(),
            ).await {
                error!("Failed to send low stock email to {}: {}", email, e);
            } else {
                notification_count += 1;
            }
        }

        info!("Low stock check completed: {} alerts, {} notifications sent", alerts.data.len(), notification_count);
        Ok(())
    }

    /// 檢查效期並發送通知
    async fn check_expiry(db: &PgPool, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = NotificationService::new(db.clone());
        
        // 取得效期預警項目
        let alerts = service.list_expiry_alerts(1, 100).await?;
        
        if alerts.data.is_empty() {
            info!("No expiry alerts found");
            return Ok(());
        }

        let expired_count = alerts.data.iter().filter(|a| a.expiry_status == "expired").count();
        let expiring_count = alerts.data.iter().filter(|a| a.expiry_status == "expiring_soon").count();

        // 取得需要通知的使用者
        let users: Vec<(uuid::Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_expiry_warning = true
            "#,
        )
        .fetch_all(db)
        .await?;

        // 建構 HTML 表格
        let alerts_html = Self::build_expiry_html(&alerts.data);

        // 發送通知
        let mut notification_count = 0;
        for (_user_id, email, name) in users {
            // 建立站內通知
            if let Err(e) = service.send_expiry_notifications().await {
                tracing::warn!("發送效期預警通知失敗: {e}");
            }

            
            // 發送 Email
            if let Err(e) = EmailService::send_expiry_alert_email(
                config,
                &email,
                &name,
                &alerts_html,
                expired_count,
                expiring_count,
            ).await {
                error!("Failed to send expiry email to {}: {}", email, e);
            } else {
                notification_count += 1;
            }
        }

        info!("Expiry check completed: {} alerts ({} expired, {} expiring), {} notifications sent", 
              alerts.data.len(), expired_count, expiring_count, notification_count);
        Ok(())
    }

    /// 清理過期通知
    async fn cleanup_notifications(db: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                    <th>SKU</th>
                    <th>品名</th>
                    <th>倉庫</th>
                    <th>現有量</th>
                    <th>安全庫存</th>
                </tr>
            </thead>
            <tbody>"#
        );

        for alert in alerts.iter().take(20) {
            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{} {}</td>
                    <td>{}</td>
                </tr>"#,
                alert.product_sku,
                alert.product_name,
                alert.warehouse_name,
                alert.qty_on_hand,
                alert.base_uom,
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
                    <th>SKU</th>
                    <th>品名</th>
                    <th>批號</th>
                    <th>效期</th>
                    <th>剩餘天數</th>
                    <th>現有量</th>
                </tr>
            </thead>
            <tbody>"#
        );

        for alert in alerts.iter().take(20) {
            let status_class = if alert.expiry_status == "expired" { "expired" } else { "expiring" };
            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td class="{}">{}</td>
                    <td>{} {}</td>
                </tr>"#,
                alert.sku,
                alert.product_name,
                alert.batch_no.as_deref().unwrap_or("-"),
                alert.expiry_date,
                status_class,
                alert.days_until_expiry,
                alert.on_hand_qty,
                alert.base_uom,
            ));
        }

        html.push_str("</tbody></table>");

        if alerts.len() > 20 {
            html.push_str(&format!("<p>...另外還有 {} 項，請登入系統查看完整列表</p>", alerts.len() - 20));
        }

        html
    }

    /// 手動觸發低庫存檢查（供 API 使用）
    pub async fn trigger_low_stock_check(db: &PgPool, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::check_low_stock(db, config).await
    }

    /// 手動觸發效期檢查（供 API 使用）
    pub async fn trigger_expiry_check(db: &PgPool, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::check_expiry(db, config).await
    }

    /// 產出每月進銷貨+血液檢查報表
    async fn generate_monthly_report(db: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use chrono::{Datelike, NaiveDate, Utc};

        let now = Utc::now().naive_utc().date();
        // 上月的第一天和最後一天
        let year = if now.month() == 1 { now.year() - 1 } else { now.year() };
        let month = if now.month() == 1 { 12 } else { now.month() - 1 };
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("上月第一天應為有效日期");
        let last_day = if now.month() == 1 {
            NaiveDate::from_ymd_opt(now.year(), 1, 1).expect("當年 1/1 應為有效日期").pred_opt().expect("1/1 前一天應存在")
        } else {
            NaiveDate::from_ymd_opt(now.year(), now.month(), 1).expect("當月第一天應為有效日期").pred_opt().expect("當月第一天前一天應存在")
        };

        info!("[Monthly Report] 統計期間：{} ~ {}", first_day, last_day);

        // 1. 採購彙總
        let purchase_summary: Option<(i64, Option<rust_decimal::Decimal>)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as cnt,
                   SUM(dl.qty * COALESCE(dl.unit_price, 0)) as total_amount
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.doc_type = 'PO'
              AND d.status = 'approved'
              AND d.doc_date BETWEEN $1 AND $2
            "#,
        )
        .bind(first_day)
        .bind(last_day)
        .fetch_optional(db)
        .await?;

        let (po_count, po_amount) = purchase_summary.unwrap_or((0, None));

        // 2. 銷售彙總
        let sales_summary: Option<(i64, Option<rust_decimal::Decimal>)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as cnt,
                   SUM(dl.qty * COALESCE(dl.unit_price,
                       (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                        WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL),
                       0)) as total_amount
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.doc_type = 'SO'
              AND d.status = 'approved'
              AND d.doc_date BETWEEN $1 AND $2
            "#,
        )
        .bind(first_day)
        .bind(last_day)
        .fetch_optional(db)
        .await?;

        let (so_count, so_amount) = sales_summary.unwrap_or((0, None));

        // 3. 各計畫血液檢查項目統計
        let blood_test_stats: Vec<(Option<String>, String, i64)> = sqlx::query_as(
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
        .unwrap_or_default();

        // 4. 構建報表內容
        let month_str = format!("{}年{}月", year, month);
        let mut content = format!(
            "{}月度報表\n\n=== 進銷貨彙總 ===\n採購單（已核准）：{} 筆，金額 ${}\n銷售單（已核准）：{} 筆，金額 ${}\n",
            month_str,
            po_count,
            po_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
            so_count,
            so_amount.map(|a| a.to_string()).unwrap_or("0".to_string()),
        );

        if !blood_test_stats.is_empty() {
            content.push_str("\n=== 血液檢查統計 ===\n");
            for (iacuc_no, item_name, cnt) in &blood_test_stats {
                content.push_str(&format!(
                    "計畫 {}：{} × {} 次\n",
                    iacuc_no.as_deref().unwrap_or("-"),
                    item_name,
                    cnt,
                ));
            }
        }

        // 5. 通知相關使用者
        let service = NotificationService::new(db.clone());
        let mut recipients = service.get_users_by_role("WAREHOUSE_MANAGER").await?;
        let admins = service.get_users_by_role("SYSTEM_ADMIN").await?;
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
                    content: Some(content.clone()),
                    related_entity_type: Some("report".to_string()),
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("create_notification 失敗: {e}");
            }

            count += 1;
        }

        info!(
            "[Monthly Report] {}報表已產出並發送給 {} 位使用者（PO: {}, SO: {}, 血檢項: {}）",
            month_str, count, po_count, so_count, blood_test_stats.len()
        );
        Ok(())
    }
}
