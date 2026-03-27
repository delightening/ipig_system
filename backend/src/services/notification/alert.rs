// 預警通知（低庫存 + 效期）



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

    /// 發送低庫存通知（批次作業用）— 依 notification_routing 表判斷收件角色
    /// 同一使用者當天已有相同類型的未讀通知時，跳過不重複建立
    pub async fn send_low_stock_notifications(&self) -> Result<i32, AppError> {
        // 取得低庫存項目
        let alerts: Vec<LowStockAlert> = sqlx::query_as(
            r#"SELECT * FROM v_low_stock_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("low_stock_alert").await?;

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
            // 檢查使用者個人設定
            let user_email_enabled: Option<(bool,)> = sqlx::query_as(
                "SELECT email_low_stock FROM notification_settings WHERE user_id = $1",
            )
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;

            if let Some((enabled,)) = user_email_enabled {
                if !enabled {
                    continue;
                }
            }

            // 當天去重：若該使用者今天已有同類型未讀通知，跳過
            if self.has_today_notification(*user_id, "low_stock").await? {
                continue;
            }

            let title = format!("低庫存預警：{} 項產品需要補貨", alerts.len());
            let content = alerts
                .iter()
                .take(5)
                .map(|a| format!("- {} ({}) 庫存: {}", a.product_name, a.product_sku, a.qty_on_hand))
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::LowStock,
                    title,
                    content: Some(content),
                    related_entity_type: Some("low_stock".to_string()),
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    /// 發送效期預警通知（批次作業用）— 依 notification_routing 表判斷收件角色
    /// 同一使用者當天已有相同類型的未讀通知時，跳過不重複建立
    pub async fn send_expiry_notifications(&self) -> Result<i32, AppError> {
        // 取得效期預警項目
        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"SELECT * FROM v_expiry_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("expiry_alert").await?;

        let expired_count = alerts.iter().filter(|a| a.expiry_status == "expired").count();
        let expiring_count = alerts.iter().filter(|a| a.expiry_status == "expiring_soon").count();

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
            // 檢查使用者個人設定
            let user_email_enabled: Option<(bool,)> = sqlx::query_as(
                "SELECT email_expiry_warning FROM notification_settings WHERE user_id = $1",
            )
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;

            if let Some((enabled,)) = user_email_enabled {
                if !enabled {
                    continue;
                }
            }

            // 當天去重：若該使用者今天已有同類型未讀通知，跳過
            if self.has_today_notification(*user_id, "expiry_warning").await? {
                continue;
            }

            let title = format!(
                "效期預警：{} 項已過期，{} 項即將到期",
                expired_count, expiring_count
            );
            let content = alerts
                .iter()
                .take(5)
                .map(|a| {
                    format!(
                        "- {} ({}) 批號:{} 效期:{} ({}天) 近效期量:{} / 總量:{}",
                        a.product_name, a.sku,
                        a.batch_no.as_deref().unwrap_or("-"),
                        a.expiry_date,
                        a.days_until_expiry,
                        a.on_hand_qty,
                        a.total_qty,
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::ExpiryWarning,
                    title,
                    content: Some(content),
                    related_entity_type: Some("expiry_warning".to_string()),
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    /// 檢查使用者今天是否已有指定類型的通知
    async fn has_today_notification(
        &self,
        user_id: uuid::Uuid,
        related_entity_type: &str,
    ) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notifications
                WHERE user_id = $1
                  AND related_entity_type = $2
                  AND created_at >= CURRENT_DATE
            )
            "#,
        )
        .bind(user_id)
        .bind(related_entity_type)
        .fetch_one(&self.db)
        .await?;

        Ok(exists.0)
    }
}
