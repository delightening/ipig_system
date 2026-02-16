// 安樂死相關通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知 PI 有新的安樂死單據
    pub async fn notify_euthanasia_order(
        &self,
        order_id: Uuid,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        reason: &str,
        pi_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = format!("[緊急] 動物 #{} 安樂死執行通知", ear_tag);
        let content = format!(
            "獸醫已開立安樂死單。\n\n耳號：{}\nIACUC No.：{}\n原因：{}\n\n執行時間：系統將於 24 小時後自動解鎖執行權限。\n\n請登入系統選擇「同意執行」或「申請暫緩」。",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            reason
        );

        self.create_notification(CreateNotificationRequest {
            user_id: pi_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        Ok(())
    }

    /// 通知獸醫：PI 已同意執行安樂死
    pub async fn notify_euthanasia_approved(
        &self,
        order_id: Uuid,
        vet_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = "[已核准] PI 已同意執行安樂死".to_string();
        let content = "PI 已同意執行安樂死。您現在可以進行安樂死操作。".to_string();

        self.create_notification(CreateNotificationRequest {
            user_id: vet_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        Ok(())
    }

    /// 通知 CHAIR：有新的安樂死暫緩申請需要仲裁
    pub async fn notify_euthanasia_appeal(
        &self,
        appeal_id: Uuid,
        order_id: Uuid,
        chair_user_id: Uuid,
        appeal_reason: &str,
    ) -> Result<(), AppError> {
        let title = "[仲裁請求] 安樂死暫緩申請".to_string();
        let content = format!(
            "PI 已申請暫緩安樂死執行，請進行仲裁。\n\n暫緩理由：{}\n\n請於 24 小時內做出裁決，否則系統將自動核准執行安樂死。",
            appeal_reason
        );

        self.create_notification(CreateNotificationRequest {
            user_id: chair_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_appeal".to_string()),
            related_entity_id: Some(appeal_id),
        })
        .await?;

        tracing::info!(
            "[Euthanasia Appeal] Notification sent to CHAIR for order {} appeal {}",
            order_id,
            appeal_id
        );

        Ok(())
    }

    /// 通知獸醫：因超時自動核准執行權限
    pub async fn notify_euthanasia_timeout_approved(
        &self,
        order_id: Uuid,
        vet_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = "[超時核准] 安樂死執行權限已解鎖".to_string();
        let content = "因 PI/CHAIR 超時未回應，系統已自動解鎖安樂死執行權限。您現在可以進行安樂死操作。".to_string();

        self.create_notification(CreateNotificationRequest {
            user_id: vet_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        tracing::warn!(
            "[Euthanasia Timeout] Order {} auto-approved due to timeout",
            order_id
        );

        Ok(())
    }
}
