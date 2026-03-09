// HR 通知（請假 + 加班申請提交）

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知請假申請已提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_leave_submitted(
        &self,
        leave_id: Uuid,
        applicant_name: &str,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<i32, AppError> {
        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("leave_submitted").await?;

        let title = format!("[iPig] 新請假申請 - {}", applicant_name);
        let content = format!(
            "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
            applicant_name, leave_type, start_date, end_date
        );

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
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

    /// 通知已核准請假被取消（通知該請假的所有核准經手人）
    pub async fn notify_leave_cancelled(
        &self,
        leave_id: Uuid,
        applicant_name: &str,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
        cancellation_reason: Option<&str>,
    ) -> Result<i32, AppError> {
        // 從 leave_approvals 取得曾經核准此假單的經手人
        let approvers: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT la.approver_id, u.display_name
            FROM leave_approvals la
            JOIN users u ON u.id = la.approver_id
            WHERE la.leave_request_id = $1
              AND la.action = 'APPROVE'
            "#,
        )
        .bind(leave_id)
        .fetch_all(&self.db)
        .await?;

        if approvers.is_empty() {
            tracing::info!("[Notification] 請假 {} 無核准經手人，跳過取消通知", leave_id);
            return Ok(0);
        }

        let title = format!("[iPig] 已核准請假已取消 - {}", applicant_name);
        let reason_text = cancellation_reason
            .map(|r| format!("\n取消原因：{}", r))
            .unwrap_or_default();
        let content = format!(
            "先前核准的請假已被取消。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}{}",
            applicant_name, leave_type, start_date, end_date, reason_text
        );

        let mut count = 0;
        for (approver_id, _name) in &approvers {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *approver_id,
                    notification_type: NotificationType::LeaveApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(leave_id),
                })
                .await
            {
                tracing::warn!("建立請假取消通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!(
            "[Notification] 請假取消通知已發送給 {} 位核准經手人",
            count
        );
        Ok(count)
    }

    /// 通知加班申請已提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_overtime_submitted(
        &self,
        overtime_id: Uuid,
        applicant_name: &str,
        overtime_date: &str,
        hours: f64,
    ) -> Result<i32, AppError> {
        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("overtime_submitted").await?;

        let title = format!("[iPig] 新加班申請 - {}", applicant_name);
        let content = format!(
            "有新的加班申請待審核。\n\n申請人：{}\n加班日期：{}\n加班時數：{} 小時",
            applicant_name, overtime_date, hours
        );

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
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
