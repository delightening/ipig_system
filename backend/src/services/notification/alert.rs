// 預警通知（低庫存 + 效期）

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, ExpiryAlert, LowStockAlert,
        NotificationType, PaginatedResponse,
    },
};

use super::NotificationService;

impl NotificationService {
    /// 取得低庫存預警列表
    pub async fn list_low_stock_alerts(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<LowStockAlert>, AppError> {
        let offset = (page - 1) * per_page;

        let alerts: Vec<LowStockAlert> = sqlx::query_as(
            r#"
            SELECT * FROM v_low_stock_alerts
            ORDER BY stock_status, product_name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM v_low_stock_alerts"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    /// 取得效期預警列表
    pub async fn list_expiry_alerts(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<ExpiryAlert>, AppError> {
        let offset = (page - 1) * per_page;

        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"
            SELECT * FROM v_expiry_alerts
            ORDER BY days_until_expiry, product_name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM v_expiry_alerts"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    /// 發送低庫存通知（批次作業用）
    pub async fn send_low_stock_notifications(&self) -> Result<i32, AppError> {
        // 取得需要通知的使用者及其設定
        let users_with_settings: Vec<(Uuid, bool)> = sqlx::query_as(
            r#"
            SELECT u.id, ns.email_low_stock
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_low_stock = true
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // 取得低庫存項目
        let alerts: Vec<LowStockAlert> = sqlx::query_as(
            r#"SELECT * FROM v_low_stock_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (user_id, _) in users_with_settings {
            // 建立通知
            let title = format!("低庫存預警：{} 項產品需要補貨", alerts.len());
            let content = alerts
                .iter()
                .take(5)
                .map(|a| format!("- {} ({}) 庫存: {}", a.product_name, a.product_sku, a.qty_on_hand))
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::LowStock,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    /// 發送效期預警通知（批次作業用）
    pub async fn send_expiry_notifications(&self) -> Result<i32, AppError> {
        // 取得需要通知的使用者及其設定
        let users_with_settings: Vec<(Uuid, i32)> = sqlx::query_as(
            r#"
            SELECT u.id, ns.expiry_warning_days
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_expiry_warning = true
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // 取得效期預警項目
        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"SELECT * FROM v_expiry_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (user_id, _) in users_with_settings {
            // 建立通知
            let expired_count = alerts.iter().filter(|a| a.expiry_status == "expired").count();
            let expiring_count = alerts.iter().filter(|a| a.expiry_status == "expiring_soon").count();

            let title = format!(
                "效期預警：{} 項已過期，{} 項即將到期",
                expired_count, expiring_count
            );
            let content = alerts
                .iter()
                .take(5)
                .map(|a| {
                    format!(
                        "- {} ({}) 批號:{} 效期:{} ({}天)",
                        a.product_name, a.sku, 
                        a.batch_no.as_deref().unwrap_or("-"),
                        a.expiry_date,
                        a.days_until_expiry
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ExpiryWarning,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }
}
