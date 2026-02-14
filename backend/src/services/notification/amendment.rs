// 修正案通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 修正案進度通知
    pub async fn notify_amendment_progress(
        &self,
        amendment_id: Uuid,
        protocol_id: Uuid,
        protocol_no: &str,
        amendment_title: &str,
        event: &str,
        operator_id: Uuid,
        reason: Option<&str>,
        config: Option<&crate::config::Config>,
    ) -> Result<i32, AppError> {
        let mut count = 0;
        let event_text = match event {
            "submitted" => "修正案已提交",
            "classified" | "under_review" => "修正案開始審查",
            "decision_recorded" => "審查委員已記錄決定",
            "revision_required" => "修正案需要修正",
            "approved" => "修正案已核准",
            "rejected" => "修正案已駁回",
            _ => event,
        };

        let notification_title = format!("[iPig] {} - {}", event_text, protocol_no);
        let content = format!(
            "{}。\n\n計畫編號：{}\n修正案：{}\n{}",
            event_text,
            protocol_no,
            amendment_title,
            reason.map(|r| format!("說明：{}", r)).unwrap_or_default()
        );

        let needs_email = matches!(
            event,
            "revision_required" | "approved" | "rejected"
        );

        match event {
            // 提交 → 通知 IACUC_STAFF
            "submitted" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                for (user_id, _email, _name) in &staff {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 開始審查 → 通知 REVIEWER
            "classified" | "under_review" => {
                // 取得修正案被指派的審查委員
                let reviewers: Vec<(Uuid, String, String)> = sqlx::query_as(
                    r#"
                    SELECT DISTINCT u.id, u.email, u.display_name
                    FROM users u
                    JOIN amendment_review_assignments ara ON u.id = ara.reviewer_id
                    WHERE ara.amendment_id = $1 AND u.is_active = true
                    "#,
                )
                .bind(amendment_id)
                .fetch_all(&self.db)
                .await?;

                for (user_id, _email, _name) in &reviewers {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ReviewAssignment,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 審查委員記錄決定 → 通知 IACUC_STAFF
            "decision_recorded" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                for (user_id, _email, _name) in &staff {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 要求修正 → 通知 PI + Coeditor（+ Email）
            "revision_required" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, email, display_name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg, email, display_name, protocol_no,
                                amendment_title, event_text,
                                &chrono::Utc::now().to_rfc3339(), reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 核准/駁回 → 通知 PI + Coeditor + IACUC_CHAIR（非操作者）
            "approved" | "rejected" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                let chairs = self.get_users_by_role("IACUC_CHAIR").await?;
                let mut all_recipients = pi_coeditors;
                all_recipients.extend(chairs);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, email, display_name) in &all_recipients {
                    if *user_id == operator_id {
                        continue;
                    }
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg, email, display_name, protocol_no,
                                amendment_title, event_text,
                                &chrono::Utc::now().to_rfc3339(), reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        tracing::info!(
            "[Notification] 修正案 {} 事件 {}，通知已發送給 {} 人",
            amendment_id,
            event,
            count
        );
        Ok(count)
    }
}
