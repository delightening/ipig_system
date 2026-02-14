// HR 通知（請假 + 加班申請提交）

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知請假申請已提交（給 ADMIN_STAFF + admin）
    pub async fn notify_leave_submitted(
        &self,
        leave_id: Uuid,
        applicant_name: &str,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<i32, AppError> {
        // 取得 ADMIN_STAFF 角色使用者
        let mut recipients = self.get_users_by_role("ADMIN_STAFF").await?;
        // 也取得 admin 角色使用者
        let admins = self.get_users_by_role("SYSTEM_ADMIN").await?;
        recipients.extend(admins);
        // 去重
        recipients.sort_by_key(|(id, _, _)| *id);
        recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新請假申請 - {}", applicant_name);
        let content = format!(
            "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
            applicant_name, leave_type, start_date, end_date
        );

        let mut count = 0;
        for (user_id, _email, _name) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::LeaveApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(leave_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] 請假申請通知已發送給 {} 位審核者", count);
        Ok(count)
    }

    /// 通知加班申請已提交（給 ADMIN_STAFF + admin）
    pub async fn notify_overtime_submitted(
        &self,
        overtime_id: Uuid,
        applicant_name: &str,
        overtime_date: &str,
        hours: f64,
    ) -> Result<i32, AppError> {
        let mut recipients = self.get_users_by_role("ADMIN_STAFF").await?;
        let admins = self.get_users_by_role("SYSTEM_ADMIN").await?;
        recipients.extend(admins);
        recipients.sort_by_key(|(id, _, _)| *id);
        recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新加班申請 - {}", applicant_name);
        let content = format!(
            "有新的加班申請待審核。\n\n申請人：{}\n加班日期：{}\n加班時數：{} 小時",
            applicant_name, overtime_date, hours
        );

        let mut count = 0;
        for (user_id, _email, _name) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::OvertimeApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("overtime_record".to_string()),
                    related_entity_id: Some(overtime_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] 加班申請通知已發送給 {} 位審核者", count);
        Ok(count)
    }
}
