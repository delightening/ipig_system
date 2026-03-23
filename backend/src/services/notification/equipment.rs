// 設備維護相關通知

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
    services::EmailService,
};

use super::NotificationService;

fn load_config() -> Option<crate::config::Config> {
    match crate::config::Config::from_env() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::warn!("無法載入設定，跳過 email 通知: {e}");
            None
        }
    }
}

impl NotificationService {
    /// 發送設備校正/確效逾期通知（排程用）
    pub async fn send_equipment_overdue_notifications(
        &self,
    ) -> Result<i32, AppError> {
        let overdue: Vec<(String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT e.name, COALESCE(e.serial_number, '-'),
                   ec.calibration_type::text, ec.next_due_at::text
            FROM equipment_calibrations ec
            INNER JOIN equipment e ON ec.equipment_id = e.id
            WHERE ec.next_due_at < CURRENT_DATE
              AND e.status = 'active'
              AND ec.id = (
                  SELECT id FROM equipment_calibrations ec2
                  WHERE ec2.equipment_id = ec.equipment_id
                    AND ec2.calibration_type = ec.calibration_type
                  ORDER BY ec2.calibrated_at DESC
                  LIMIT 1
              )
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        if overdue.is_empty() {
            return Ok(0);
        }

        let recipients = self.get_recipients_by_event("equipment_overdue").await?;

        let overdue_html = format!(
            "<ul>{}</ul>",
            overdue
                .iter()
                .map(|(name, sn, cal_type, due)| {
                    format!("<li>{} ({}) - {} 到期：{}</li>", name, sn, cal_type, due)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        let smtp = match load_config() {
            Some(config) => Some(EmailService::resolve_smtp(&self.db, &config).await),
            None => None,
        };

        let mut count = 0;
        for (user_id, email, name, channel) in &recipients {
            if Self::should_send_in_app(channel) {
                let title = format!("設備校正/確效逾期：{} 項待處理", overdue.len());
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::SystemAlert,
                        title,
                        content: Some(format!("共 {} 項設備校正/確效已逾期", overdue.len())),
                        related_entity_type: None,
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立設備逾期通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                if let Some(smtp) = &smtp {
                    if let Err(e) = EmailService::send_equipment_overdue_email(
                        smtp, email, name, &overdue_html, overdue.len(),
                    )
                    .await
                    {
                        tracing::warn!("發送設備逾期 email 失敗: {e}");
                    }
                }
            }
            count += 1;
        }

        Ok(count)
    }

    /// 發送設備無法維修通知
    pub async fn send_equipment_unrepairable_notification(
        &self,
        equipment_name: &str,
        serial_number: &str,
        problem: &str,
    ) -> Result<(), AppError> {
        let recipients = self
            .get_recipients_by_event("equipment_unrepairable")
            .await?;

        let smtp = match load_config() {
            Some(config) => Some(EmailService::resolve_smtp(&self.db, &config).await),
            None => None,
        };

        for (user_id, email, name, channel) in &recipients {
            if Self::should_send_in_app(channel) {
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::SystemAlert,
                        title: format!("設備無法維修：{}", equipment_name),
                        content: Some(format!(
                            "設備「{}」（{}）經維修後判定無法修復，請安排報廢流程。",
                            equipment_name, serial_number
                        )),
                        related_entity_type: None,
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立無法維修通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                if let Some(smtp) = &smtp {
                    if let Err(e) = EmailService::send_equipment_unrepairable_email(
                        smtp, email, name, equipment_name, serial_number, problem,
                    )
                    .await
                    {
                        tracing::warn!("發送無法維修 email 失敗: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    /// 發送設備報廢申請通知
    pub async fn send_equipment_disposal_notification(
        &self,
        equipment_name: &str,
        applicant_name: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        let recipients = self.get_recipients_by_event("equipment_disposal").await?;

        let smtp = match load_config() {
            Some(config) => Some(EmailService::resolve_smtp(&self.db, &config).await),
            None => None,
        };

        for (user_id, email, name, channel) in &recipients {
            if Self::should_send_in_app(channel) {
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::SystemAlert,
                        title: format!("設備報廢申請：{}", equipment_name),
                        content: Some(format!(
                            "{} 申請報廢設備「{}」，原因：{}",
                            applicant_name, equipment_name, reason
                        )),
                        related_entity_type: None,
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立報廢通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                if let Some(smtp) = &smtp {
                    if let Err(e) = EmailService::send_equipment_disposal_email(
                        smtp, email, name, equipment_name, applicant_name, reason,
                    )
                    .await
                    {
                        tracing::warn!("發送報廢 email 失敗: {e}");
                    }
                }
            }
        }

        Ok(())
    }
}
