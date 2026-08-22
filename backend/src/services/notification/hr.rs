// HR 通知（請假 + 加班申請提交）

use uuid::Uuid;

use crate::{error::AppError, models::NotificationType};

use super::{EventContext, NotificationPayload, NotificationService};

impl NotificationService {
    /// 通知請假申請進入審核關，待審核者做出動作（核准/駁回）——屬「待處理」，
    /// 走置頂待辦而非一般通知（`dispatch_pinned_event`）。
    ///
    /// 收件人由 `notification_routing` 表的 `leave_submitted` 規則決定，該規則現在指向
    /// resolver `leave_current_stage_approvers`（依假單目前狀態動態算出單位主管/負責人/
    /// 卡關代批管理員），不再是「admin + DIRECTOR 角色廣播、不分審核關卡」的舊設定
    /// ——後者會讓還輪不到的角色也收到「待處理」，點下去卻 403（見 migration 145）。
    ///
    /// 每次呼叫都會經過 `EventContext.entity_id = leave_id`，resolver 才讀得到最新狀態；
    /// 因此本函式必須在**每一次**進入新審核關時被呼叫（送審後代理確認完、以及單位主管
    /// 核准過關後），而非只在第一次送審時呼叫一次——後者正是本函式修復前的既有缺口
    /// （單位主管核准後，負責人從未被通知）。
    ///
    /// 申請人姓名由 `applicant_id` 反查（僅在職、未軟刪除；查無則以「申請人」代稱），
    /// 避免呼叫端寫死名稱、並符 GDPR（不外洩停用/已刪除帳號資料）。
    pub async fn notify_leave_submitted(
        &self,
        leave_id: Uuid,
        applicant_id: Uuid,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<i32, AppError> {
        let applicant_name = sqlx::query_scalar::<_, String>(
            "SELECT display_name FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
        )
        .bind(applicant_id)
        .fetch_optional(&self.db)
        .await?
        .unwrap_or_else(|| "申請人".to_string());
        let title = format!("[iPig] 新請假申請 - {}", applicant_name);
        let content = format!(
            "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
            applicant_name, leave_type, start_date, end_date
        );

        let count = self
            .dispatch_pinned_event(
                "leave_submitted",
                &EventContext {
                    entity_id: Some(leave_id),
                    ..Default::default()
                },
                NotificationPayload {
                    notification_type: NotificationType::LeaveApproval,
                    title,
                    content: Some(content),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(leave_id),
                },
                Some(crate::models::RECIPIENT_ROLE_APPROVER),
            )
            .await?;

        tracing::info!("[Notification] 請假待審置頂待辦已發送給 {} 位審核者", count);
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
        let title = format!("[iPig] 已核准請假已取消 - {}", applicant_name);
        let reason_text = cancellation_reason
            .map(|r| format!("\n取消原因：{}", r))
            .unwrap_or_default();
        let content = format!(
            "先前核准的請假已被取消。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}{}",
            applicant_name, leave_type, start_date, end_date, reason_text
        );

        // 統一派送：收件人由 resolver leave_request_approvers（曾核准此假單的經手人）決定。
        let count = self
            .dispatch_event(
                "leave_cancelled",
                &EventContext {
                    entity_id: Some(leave_id),
                    ..Default::default()
                },
                NotificationPayload {
                    notification_type: NotificationType::LeaveApproval,
                    title,
                    content: Some(content),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(leave_id),
                },
            )
            .await?;

        tracing::info!("[Notification] 請假取消通知已發送給 {} 位核准經手人", count);
        Ok(count)
    }

    /// 代理人退回申請後通知申請人「代理人退回，請重新指定代理人」。
    /// 收件人＝申請人本人（resolver event_subject，由 subject_user_id 提供並驗證 active）。
    pub async fn notify_leave_proxy_rejected(
        &self,
        leave_id: Uuid,
        applicant_id: Uuid,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
        reason: Option<&str>,
    ) -> Result<i32, AppError> {
        let leave_type_display = crate::models::LeaveType::from_db_str(leave_type)
            .map_or(leave_type, |t| t.display_name());
        let reason_text = reason
            .filter(|r| !r.is_empty())
            .map(|r| format!("\n退回原因：{}", r))
            .unwrap_or_default();
        let content = format!(
            "您指定的職務代理人已退回此請假申請，已退回草稿，請重新指定代理人後再送審。\n\n假別：{}\n期間：{} ~ {}{}",
            leave_type_display, start_date, end_date, reason_text
        );
        self.dispatch_event(
            "leave_proxy_rejected",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::LeaveApproval,
                title: "[iPig] 職務代理人退回您的請假".to_string(),
                content: Some(content),
                related_entity_type: Some("leave_request".to_string()),
                related_entity_id: Some(leave_id),
            },
        )
        .await
    }

    /// 最終核准後通知職務代理人「代理已生效，請接手」。
    pub async fn notify_leave_proxy_effective(
        &self,
        leave_id: Uuid,
        proxy_id: Uuid,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<i32, AppError> {
        let leave_type_display = crate::models::LeaveType::from_db_str(leave_type)
            .map_or(leave_type, |t| t.display_name());
        let content = format!(
            "您的職務代理已生效，請於期間內協助代理。\n\n假別：{}\n期間：{} ~ {}",
            leave_type_display, start_date, end_date
        );
        self.dispatch_event(
            "leave_proxy_effective",
            &EventContext {
                subject_user_id: Some(proxy_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::LeaveApproval,
                title: "[iPig] 職務代理已生效".to_string(),
                content: Some(content),
                related_entity_type: Some("leave_request".to_string()),
                related_entity_id: Some(leave_id),
            },
        )
        .await
    }

    /// 通知申請人「請假已被駁回」（含駁回原因）。
    /// 收件人＝申請人本人（resolver event_subject，由 subject_user_id 提供並驗證 active）。
    pub async fn notify_leave_rejected(
        &self,
        leave_id: Uuid,
        applicant_id: Uuid,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
        reason: Option<&str>,
    ) -> Result<i32, AppError> {
        let leave_type_display = crate::models::LeaveType::from_db_str(leave_type)
            .map_or(leave_type, |t| t.display_name());
        let reason_text = reason
            .filter(|r| !r.is_empty())
            .map(|r| format!("\n駁回原因：{}", r))
            .unwrap_or_default();
        let content = format!(
            "您的請假申請已被駁回。\n\n假別：{}\n期間：{} ~ {}{}",
            leave_type_display, start_date, end_date, reason_text
        );
        self.dispatch_event(
            "leave_rejected",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::LeaveApproval,
                title: "[iPig] 您的請假申請已駁回".to_string(),
                content: Some(content),
                related_entity_type: Some("leave_request".to_string()),
                related_entity_id: Some(leave_id),
            },
        )
        .await
    }

    /// 通知加班申請已提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_overtime_submitted(
        &self,
        overtime_id: Uuid,
        applicant_name: &str,
        overtime_date: &str,
        hours: f64,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 新加班申請 - {}", applicant_name);
        let content = format!(
            "有新的加班申請待審核。\n\n申請人：{}\n加班日期：{}\n加班時數：{} 小時",
            applicant_name, overtime_date, hours
        );

        // 統一派送：收件人與管道由 notification_routing 決定（overtime_submitted → 角色型）。
        let count = self
            .dispatch_event(
                "overtime_submitted",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::OvertimeApproval,
                    title,
                    content: Some(content),
                    related_entity_type: Some("overtime_record".to_string()),
                    related_entity_id: Some(overtime_id),
                },
            )
            .await?;

        tracing::info!("[Notification] 加班申請通知已發送給 {} 位審核者", count);
        Ok(count)
    }

    /// R72-3：通知申請人「請假已最終核准」。
    /// GDPR：僅通知 is_active 且未軟刪除的帳號（停用/已刪除不寄）。
    pub async fn notify_leave_approved(
        &self,
        leave_id: Uuid,
        applicant_id: Uuid,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<(), AppError> {
        // 對申請人顯示中文假別（重用 LeaveType::display_name；未知值回原字串）
        let leave_type_display = crate::models::LeaveType::from_db_str(leave_type)
            .map_or(leave_type, |t| t.display_name());
        let content = format!(
            "您的請假申請已完成核准。\n\n假別：{}\n期間：{} ~ {}",
            leave_type_display, start_date, end_date
        );

        // 統一派送：收件人由 resolver event_subject（申請人本人）決定；
        // resolver 內已驗證 active 且未軟刪除（GDPR：停用 / 已刪除帳號不通知）。
        self.dispatch_event(
            "leave_approved",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::LeaveApproval,
                title: "[iPig] 您的請假申請已核准".to_string(),
                content: Some(content),
                related_entity_type: Some("leave_request".to_string()),
                related_entity_id: Some(leave_id),
            },
        )
        .await?;
        Ok(())
    }

    /// R72-3：通知申請人「加班已最終核准」（GDPR 同上）。
    pub async fn notify_overtime_approved(
        &self,
        overtime_id: Uuid,
        applicant_id: Uuid,
        overtime_date: &str,
        hours: &str,
    ) -> Result<(), AppError> {
        let content = format!(
            "您的加班申請已完成核准。\n\n日期：{}\n時數：{} 小時",
            overtime_date, hours
        );

        // 統一派送：收件人由 resolver event_subject（申請人本人）決定；
        // resolver 內已驗證 active 且未軟刪除（GDPR：停用 / 已刪除帳號不通知）。
        self.dispatch_event(
            "overtime_approved",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::OvertimeApproval,
                title: "[iPig] 您的加班申請已核准".to_string(),
                content: Some(content),
                related_entity_type: Some("overtime_record".to_string()),
                related_entity_id: Some(overtime_id),
            },
        )
        .await?;
        Ok(())
    }
}
