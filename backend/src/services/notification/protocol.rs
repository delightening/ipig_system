// AUP 計畫相關通知

use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::ActorContext,
    models::{CreateNotificationRequest, NotificationType},
};

use super::dispatch::StaffEmail;
use super::{EventContext, NotificationPayload, NotificationService};

impl NotificationService {
    /// 通知計畫提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_protocol_submitted(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        title: &str,
        pi_name: &str,
    ) -> Result<i32, AppError> {
        let notification_title = format!("[iPig] 新計畫提交 - {}", protocol_no);
        let content = format!(
            "新計畫已提交，請進行行政預審。\n\n計畫編號：{}\n計畫名稱：{}\n計畫主持人：{}",
            protocol_no, title, pi_name
        );

        // 統一派送：收件人與管道由 notification_routing 決定（protocol_submitted → 角色型）。
        let count = self
            .dispatch_event(
                "protocol_submitted",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::ProtocolSubmitted,
                    title: notification_title,
                    content: Some(content),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                },
            )
            .await?;

        Ok(count)
    }

    /// 通知計畫狀態變更（直接通知 PI，非路由表管理）
    pub async fn notify_protocol_status_change(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        title: &str,
        new_status: &str,
        pi_user_id: Uuid,
        reason: Option<&str>,
    ) -> Result<(), AppError> {
        let notification_title = format!("[iPig] 計畫狀態更新 - {}", protocol_no);
        let content = format!(
            "您的計畫狀態已更新。\n\n計畫編號：{}\n計畫名稱：{}\n新狀態：{}\n{}",
            protocol_no,
            title,
            new_status,
            reason
                .map(|r| format!("變更原因：{}", r))
                .unwrap_or_default()
        );

        // 通知 PI（直接傳入 user_id，非路由表管理）
        self.create_notification(CreateNotificationRequest {
            user_id: pi_user_id,
            notification_type: NotificationType::ProtocolStatus,
            title: notification_title,
            content: Some(content),
            related_entity_type: Some("protocol".to_string()),
            related_entity_id: Some(protocol_id),
        })
        .await?;

        Ok(())
    }

    /// 通知審查指派（直接通知審查委員，非路由表管理）。
    ///
    /// 站內通知一律建立；email 為「可切換的委員通知類型」——僅當 notification_routing
    /// 的 `protocol_under_review`（委員審查）事件啟用 email 時才寄，且寄送套用時間窗、
    /// 但**不做請假判斷**（委員無請假功能，`recipient_user_id` 傳 None）。
    pub async fn notify_review_assignment(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        title: &str,
        pi_name: &str,
        reviewer_id: Uuid,
        due_date: Option<&str>,
        config: Option<&crate::config::Config>,
    ) -> Result<(), AppError> {
        let notification_title = format!("[iPig] 審查指派 - {}", protocol_no);
        let content = format!(
            "您已被指派審查以下計畫，請於期限內完成審查。\n\n計畫編號：{}\n計畫名稱：{}\n計畫主持人：{}\n審查期限：{}",
            protocol_no,
            title,
            pi_name,
            due_date.unwrap_or("待定")
        );

        self.create_notification(CreateNotificationRequest {
            user_id: reviewer_id,
            notification_type: NotificationType::ReviewAssignment,
            title: notification_title,
            content: Some(content),
            related_entity_type: Some("protocol".to_string()),
            related_entity_id: Some(protocol_id),
        })
        .await?;

        // 委員 email（可切換）：路由啟用 + 取得 active 委員聯絡資料時才寄。
        if let Some(cfg) = config {
            if self
                .is_email_enabled_for_event("protocol_under_review")
                .await
            {
                if let Some((email, name)) = self.find_active_user_contact(reviewer_id).await? {
                    let rendered = crate::services::EmailService::render_review_assignment_email(
                        &cfg.app_url,
                        &name,
                        protocol_no,
                        title,
                        pi_name,
                        due_date,
                    );
                    let actor = ActorContext::System {
                        reason: "review_assignment_notification",
                    };
                    // 委員無請假功能 → recipient_user_id None（套時間窗、不查請假）。
                    if let Err(e) = self
                        .dispatch_staff_email(
                            &actor,
                            ("protocol", protocol_id),
                            StaffEmail {
                                to_email: email,
                                to_name: name,
                                recipient_user_id: None,
                                email: rendered,
                            },
                        )
                        .await
                    {
                        tracing::warn!("分派審查指派 email 失敗: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    /// notify_protocol_review_progress 是否會直接通知 PI/Coeditor（終局決定 / 退回修正）。
    /// 呼叫端（handlers/protocol/crud.rs）據此判斷是否還需另呼叫 notify_protocol_status_change，
    /// 避免同一狀態變更對 PI 發出兩則相同通知。new_status 為小寫狀態字串。
    pub(crate) fn review_progress_notifies_pi(new_status: &str) -> bool {
        matches!(
            new_status,
            "pre_review_revision_required"
                | "vet_revision_required"
                | "revision_required"
                | "approved"
                | "approved_with_conditions"
                | "rejected"
        )
    }

    /// AUP 審查進度通知 — 依新狀態決定通知對象
    /// 「角色驅動」的部分走 notification_routing 表
    /// 「實體相關」的部分（PI/Coeditor）維持程式碼邏輯
    /// 同時處理需要 Email 的重要節點
    pub async fn notify_protocol_review_progress(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        protocol_title: &str,
        new_status: &str,
        operator_id: Uuid,
        reason: Option<&str>,
        config: Option<&crate::config::Config>,
    ) -> Result<i32, AppError> {
        let mut count = 0;
        let status_text = match new_status {
            "pre_review" => "行政預審中",
            "vet_review" => "獸醫審查中",
            "pre_review_revision_required" => "行政退回修正",
            "vet_revision_required" => "獸醫退回修正",
            "under_review" => "委員審查中",
            "revision_required" => "要求修正",
            "resubmitted" => "已重新提交",
            "approved" => "已核准",
            "approved_with_conditions" => "有條件核准",
            "rejected" => "已駁回",
            _ => new_status,
        };

        let notification_title = format!("[iPig] 計畫狀態更新 - {}", protocol_no);
        let content = format!(
            "計畫狀態已更新。\n\n計畫編號：{}\n計畫名稱：{}\n新狀態：{}\n{}",
            protocol_no,
            protocol_title,
            status_text,
            reason.map(|r| format!("說明：{}", r)).unwrap_or_default()
        );

        // 映射 protocol 狀態到路由表的 event_type
        let event_type = match new_status {
            "vet_review" => Some("protocol_vet_review"),
            "under_review" => Some("protocol_under_review"),
            "resubmitted" => Some("protocol_resubmitted"),
            "approved" | "approved_with_conditions" => Some("protocol_approved"),
            "rejected" => Some("protocol_rejected"),
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
                        title: notification_title.clone(),
                        content: Some(content.clone()),
                        related_entity_type: Some("protocol".to_string()),
                        related_entity_id: Some(protocol_id),
                    })
                    .await
                {
                    tracing::warn!("建立通知失敗: {e}");
                }
                count += 1;
            }
        }

        // 判斷需要通知 PI/Coeditor 的狀態（退回修正或最終決定）
        let needs_pi_notification = Self::review_progress_notifies_pi(new_status);

        let needs_email = needs_pi_notification;

        // 實體驅動：通知 PI/Coeditor
        if needs_pi_notification {
            let pi_coeditors = self.get_protocol_pi_and_sd(protocol_id).await?;
            for (user_id, email, display_name) in &pi_coeditors {
                if *user_id == operator_id {
                    continue;
                }
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::ProtocolStatus,
                        title: notification_title.clone(),
                        content: Some(content.clone()),
                        related_entity_type: Some("protocol".to_string()),
                        related_entity_id: Some(protocol_id),
                    })
                    .await
                {
                    tracing::warn!("建立通知失敗: {e}");
                }
                count += 1;

                // 寄 Email（員工通知 → 經時間窗 / 請假 chokepoint）
                if needs_email {
                    if let Some(cfg) = config {
                        let rendered =
                            crate::services::EmailService::render_protocol_status_change_email(
                                &cfg.app_url,
                                display_name,
                                protocol_no,
                                protocol_title,
                                status_text,
                                &crate::time::now_taiwan().to_rfc3339(),
                                reason,
                            );
                        let actor = ActorContext::System {
                            reason: "protocol_status_change_notification",
                        };
                        if let Err(e) = self
                            .dispatch_staff_email(
                                &actor,
                                ("protocol", protocol_id),
                                StaffEmail {
                                    to_email: email.clone(),
                                    to_name: display_name.clone(),
                                    recipient_user_id: Some(*user_id),
                                    email: rendered,
                                },
                            )
                            .await
                        {
                            tracing::warn!("分派計畫狀態變更郵件失敗: {e}");
                        }
                    }
                }
            }
        }

        // 進入委員審查或重新提交時，也需要通知已指派的審查委員（非路由表管理）
        if matches!(new_status, "under_review" | "resubmitted") {
            let reviewers = self.get_assigned_reviewers(protocol_id).await?;
            for (user_id, _email, _name) in &reviewers {
                if *user_id == operator_id {
                    continue;
                }
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::ProtocolStatus,
                        title: notification_title.clone(),
                        content: Some(content.clone()),
                        related_entity_type: Some("protocol".to_string()),
                        related_entity_id: Some(protocol_id),
                    })
                    .await
                {
                    tracing::warn!("建立通知失敗: {e}");
                }
                count += 1;
            }
        }

        tracing::info!(
            "[Notification] 計畫 {} 狀態變更為 {}，通知已發送給 {} 人",
            protocol_no,
            new_status,
            count
        );
        Ok(count)
    }

    /// 審查意見通知 — 角色驅動的部分走路由表，PI/Coeditor 維持程式碼邏輯
    pub async fn notify_review_comment_created(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        commenter_name: &str,
        comment_excerpt: &str,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 新審查意見 - {}", protocol_no);
        let excerpt = if comment_excerpt.chars().count() > 100 {
            format!(
                "{}...",
                comment_excerpt.chars().take(100).collect::<String>()
            )
        } else {
            comment_excerpt.to_string()
        };
        let content = format!(
            "計畫 {} 收到新的審查意見。\n\n審查委員：{}\n意見摘要：{}",
            protocol_no, commenter_name, excerpt
        );

        // 統一派送：角色（IACUC_STAFF）+ resolver（protocol_pi_sd → 計畫 PI/SD）皆由
        // notification_routing 決定；dispatch_event 解析後去重。
        let count = self
            .dispatch_event(
                "review_comment_created",
                &EventContext {
                    protocol_id: Some(protocol_id),
                    ..Default::default()
                },
                NotificationPayload {
                    notification_type: NotificationType::ReviewComment,
                    title,
                    content: Some(content),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] 審查意見通知已發送給 {} 人（計畫 {}）",
            count,
            protocol_no
        );
        Ok(count)
    }

    /// 通知所有審查意見已送出（所有指定委員都已發表意見）
    /// 依 notification_routing 表查詢 `all_reviews_completed` 事件的收件者
    pub async fn notify_all_reviews_completed(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        protocol_title: &str,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 所有審查意見已送出 - {}", protocol_no);
        let content = format!(
            "所有指定委員已完成意見提交，請進行後續處理。\n\n計畫編號：{}\n計畫名稱：{}",
            protocol_no, protocol_title
        );

        // 統一派送：收件人與管道由 notification_routing 決定（all_reviews_completed → 角色型）。
        let count = self
            .dispatch_event(
                "all_reviews_completed",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::ProtocolStatus,
                    title,
                    content: Some(content),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] 所有審查意見已送出通知已發送給 {} 人（計畫 {}）",
            count,
            protocol_no
        );
        Ok(count)
    }

    /// 通知所有審查意見已解決
    /// 依 notification_routing 表查詢 `all_comments_resolved` 事件的收件者
    pub async fn notify_all_comments_resolved(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        protocol_title: &str,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 所有審查意見已解決 - {}", protocol_no);
        let content = format!(
            "所有審查意見已被標記為解決，可進行最終審查決定。\n\n計畫編號：{}\n計畫名稱：{}",
            protocol_no, protocol_title
        );

        // 統一派送：收件人與管道由 notification_routing 決定（all_comments_resolved → 角色型）。
        let count = self
            .dispatch_event(
                "all_comments_resolved",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::ProtocolStatus,
                    title,
                    content: Some(content),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] 所有意見已解決通知已發送給 {} 人（計畫 {}）",
            count,
            protocol_no
        );
        Ok(count)
    }
}
