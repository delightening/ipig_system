// ERP 採購單通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知採購單已提交（給 WAREHOUSE_MANAGER）
    pub async fn notify_document_submitted(
        &self,
        document_id: Uuid,
        document_no: &str,
        doc_type: &str,
        creator_name: &str,
    ) -> Result<i32, AppError> {
        let managers = self.get_users_by_role("WAREHOUSE_MANAGER").await?;

        let type_text = match doc_type {
            "PO" => "採購單",
            "PR" => "採購退貨",
            "SO" => "銷售單",
            "DO" => "銷售出庫",
            _ => doc_type,
        };
        let title = format!("[iPig] 新{}待審核 - {}", type_text, document_no);
        let content = format!(
            "有新的{}待審核。\n\n單據編號：{}\n建立者：{}",
            type_text, document_no, creator_name
        );

        let mut count = 0;
        for (user_id, _email, _name) in &managers {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::DocumentApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("document".to_string()),
                    related_entity_id: Some(document_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] {}提交通知已發送給 {} 位倉管", type_text, count);
        Ok(count)
    }

    /// 通知採購單已審核/駁回（給建立者）
    pub async fn notify_document_decided(
        &self,
        document_id: Uuid,
        document_no: &str,
        doc_type: &str,
        is_approved: bool,
        creator_id: Uuid,
    ) -> Result<(), AppError> {
        let type_text = match doc_type {
            "PO" => "採購單",
            "PR" => "採購退貨",
            "SO" => "銷售單",
            "DO" => "銷售出庫",
            _ => doc_type,
        };
        let decision = if is_approved { "已核准" } else { "已駁回" };
        let title = format!("[iPig] {}{} - {}", type_text, decision, document_no);
        let content = format!(
            "您的{}已{}。\n\n單據編號：{}",
            type_text, decision, document_no
        );

        self.create_notification(CreateNotificationRequest {
            user_id: creator_id,
            notification_type: NotificationType::DocumentApproval,
            title,
            content: Some(content),
            related_entity_type: Some("document".to_string()),
            related_entity_id: Some(document_id),
        })
        .await?;

        tracing::info!(
            "[Notification] {}{} 通知已發送給建立者",
            type_text,
            decision
        );
        Ok(())
    }
}
