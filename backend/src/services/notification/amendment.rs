// 修正案通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知修正案提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_amendment_submitted(
        &self,
        amendment_id: Uuid,
        protocol_no: &str,
        amendment_title: &str,
        _pi_name: &str,
    ) -> Result<i32, AppError> {
        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("amendment_submitted").await?;

        let title = format!("[iPig] 新修正案 - {} / {}", protocol_no, amendment_title);
        let content = format!(
            "有新的修正案提交，請進行審查。\n\n所屬計畫：{}\n修正案名稱：{}\n提交者：{}",
            protocol_no, amendment_title, _pi_name
        );

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::ProtocolStatus,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("amendment".to_string()),
                    related_entity_id: Some(amendment_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] 修正案提交通知已發送給 {} 人", count);
        Ok(count)
    }

    /// 修正案狀態變更通知 — 角色驅動走路由表，PI/Coeditor 維持程式碼邏輯
    /// 函數名稱保留為 notify_amendment_progress 以維持與現有 caller 的相容性
    pub async fn notify_amendment_progress(
        &self,
        amendment_id: Uuid,
        protocol_id: Uuid,
        protocol_no: &str,
        amendment_title: &str,
        new_status: &str,
        operator_id: Uuid,
        reason: Option<&str>,
        _config: Option<&crate::config::Config>,
    ) -> Result<i32, AppError> {
        let status_text = match new_status {
            "submitted" => "已提交",
            "under_review" => "審查中",
            "revision_required" => "要求修正",
            "resubmitted" => "已重新提交",
            "approved" => "已核准",
            "rejected" => "已駁回",
            "classified" => "已分類",
            _ => new_status,
        };

        let title = format!("[iPig] 修正案更新 - {} / {}", protocol_no, amendment_title);
        let content = format!(
            "修正案狀態已更新。\n\n所屬計畫：{}\n修正案：{}\n新狀態：{}\n{}",
            protocol_no,
            amendment_title,
            status_text,
            reason.map(|r| format!("說明：{}", r)).unwrap_or_default()
        );

        let mut count = 0;

        // 映射修正案狀態到路由表 event_type
        let event_type = match new_status {
            "submitted" => Some("amendment_submitted"),
            "approved" => Some("amendment_approved"),
            "rejected" => Some("amendment_rejected"),
            _ => None,
        };

        // 角色驅動：從路由表取得收件者
        if let Some(evt) = event_type {
            let role_recipients = self.get_recipients_by_event(evt).await?;
            for (user_id, _email, _name, _channel) in &role_recipients {
                if *user_id == operator_id {
                    continue;
                }
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::ProtocolStatus,
                        title: title.clone(),
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

        // 判斷是否需要通知 PI/Coeditor（退回修正或最終決定）
        let needs_pi_notification = matches!(
            new_status,
            "revision_required" | "approved" | "rejected"
        );

        // 實體驅動：通知 PI/Coeditor
        if needs_pi_notification {
            let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
            for (user_id, _email, _display_name) in &pi_coeditors {
                if *user_id == operator_id {
                    continue;
                }
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::ProtocolStatus,
                        title: title.clone(),
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

        tracing::info!(
            "[Notification] 修正案 {} 狀態變更為 {}，通知已發送給 {} 人",
            amendment_title,
            new_status,
            count
        );
        Ok(count)
    }
}
