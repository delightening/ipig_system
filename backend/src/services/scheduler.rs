use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use sqlx::PgPool;
use tracing::{info, error};

use crate::{
    config::Config,
    services::{EmailService, NotificationService},
};

pub struct SchedulerService;

impl SchedulerService {
    /// 啟動排程服務
    pub async fn start(db: PgPool, config: Arc<Config>) -> Result<JobScheduler, Box<dyn std::error::Error + Send + Sync>> {
        let sched = JobScheduler::new().await?;

        // 每日 08:00 執行低庫存檢查
        let db_clone = db.clone();
        let config_clone = config.clone();
        sched.add(Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("Running daily low stock check...");
                if let Err(e) = Self::check_low_stock(&db, &config).await {
                    error!("Low stock check failed: {}", e);
                }
            })
        })?).await?;

        // 每日 08:00 執行效期檢查
        let db_clone = db.clone();
        let config_clone = config.clone();
        sched.add(Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let db = db_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("Running daily expiry check...");
                if let Err(e) = Self::check_expiry(&db, &config).await {
                    error!("Expiry check failed: {}", e);
                }
            })
        })?).await?;

        // 每週日 03:00 清理過期通知
        let db_clone = db.clone();
        sched.add(Job::new_async("0 0 3 * * 0", move |_uuid, _l| {
            let db = db_clone.clone();
            Box::pin(async move {
                info!("Running weekly notification cleanup...");
                if let Err(e) = Self::cleanup_notifications(&db).await {
                    error!("Notification cleanup failed: {}", e);
                }
            })
        })?).await?;

        // 啟動排程器
        sched.start().await?;
        info!("Scheduler started successfully");

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
        for (user_id, email, name) in users {
            // 建立站內通知
            let _ = service.send_low_stock_notifications().await;
            
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
        for (user_id, email, name) in users {
            // 建立站內通知
            let _ = service.send_expiry_notifications().await;
            
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
}
