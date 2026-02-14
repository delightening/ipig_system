// AUP 計畫相關通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知計畫提交（給 IACUC_STAFF）
    pub async fn notify_protocol_submitted(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        title: &str,
        pi_name: &str,
    ) -> Result<i32, AppError> {
        // 取得所有 IACUC_STAFF 使用者
        let staff_users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT u.id, u.email, u.display_name
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true AND r.code = 'IACUC_STAFF'
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        let mut count = 0;
        let notification_title = format!("[iPig] 新計畫提交 - {}", protocol_no);
        let content = format!(
            "新計畫已提交，請進行行政預審。\n\n計畫編號：{}\n計畫名稱：{}\n計畫主持人：{}",
            protocol_no, title, pi_name
        );

        for (user_id, _email, _name) in staff_users {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ProtocolSubmitted,
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    /// 通知計畫狀態變更
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
            reason.map(|r| format!("變更原因：{}", r)).unwrap_or_default()
        );

        // 通知 PI
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

    /// 通知審查指派
    pub async fn notify_review_assignment(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        title: &str,
        pi_name: &str,
        reviewer_id: Uuid,
        due_date: Option<&str>,
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

        Ok(())
    }

    /// AUP 審查進度通知 — 依新狀態決定通知對象
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

        // 判斷需求修正或最終決定 → 發 Email
        let needs_email = matches!(
            new_status,
            "pre_review_revision_required"
                | "vet_revision_required"
                | "revision_required"
                | "approved"
                | "approved_with_conditions"
                | "rejected"
        );

        match new_status {
            // 進入獸醫審查 → 通知 VET
            "vet_review" => {
                let vets = self.get_users_by_role("VET").await?;
                for (user_id, _email, _name) in &vets {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 退回修正 → 通知 PI + Coeditor（+ Email）
            "pre_review_revision_required" | "vet_revision_required" | "revision_required" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, email, display_name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    // 寄 Email
                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg,
                                email,
                                display_name,
                                protocol_no,
                                protocol_title,
                                status_text,
                                &chrono::Utc::now().to_rfc3339(),
                                reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 進入委員審查 → 通知 IACUC_STAFF + REVIEWER
            "under_review" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                let reviewers = self.get_assigned_reviewers(protocol_id).await?;
                let mut all_recipients = staff;
                all_recipients.extend(reviewers);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, _email, _name) in &all_recipients {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 重新提交 → 通知 IACUC_STAFF + 原審查委員
            "resubmitted" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                let reviewers = self.get_assigned_reviewers(protocol_id).await?;
                let mut all_recipients = staff;
                all_recipients.extend(reviewers);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, _email, _name) in &all_recipients {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 核准/駁回 → 通知 PI + Coeditor + IACUC_CHAIR（非操作者）
            "approved" | "approved_with_conditions" | "rejected" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                let chairs = self.get_users_by_role("IACUC_CHAIR").await?;
                let mut all_recipients = pi_coeditors;
                all_recipients.extend(chairs);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, email, display_name) in &all_recipients {
                    // 排除操作者本人
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
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    // 寄 Email
                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg,
                                email,
                                display_name,
                                protocol_no,
                                protocol_title,
                                status_text,
                                &chrono::Utc::now().to_rfc3339(),
                                reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 其他狀態 → 通知 PI
            _ => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, _email, _name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
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

    /// 審查意見通知 — 通知 PI + Coeditor + IACUC_STAFF
    pub async fn notify_review_comment_created(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        commenter_name: &str,
        comment_excerpt: &str,
    ) -> Result<i32, AppError> {
        let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
        let staff = self.get_users_by_role("IACUC_STAFF").await?;
        let mut all_recipients = pi_coeditors;
        all_recipients.extend(staff);
        all_recipients.sort_by_key(|(id, _, _)| *id);
        all_recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新審查意見 - {}", protocol_no);
        let excerpt = if comment_excerpt.chars().count() > 100 {
            format!("{}...", comment_excerpt.chars().take(100).collect::<String>())
        } else {
            comment_excerpt.to_string()
        };
        let content = format!(
            "計畫 {} 收到新的審查意見。\n\n審查委員：{}\n意見摘要：{}",
            protocol_no, commenter_name, excerpt
        );

        let mut count = 0;
        for (user_id, _email, _name) in &all_recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::ReviewComment,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!(
            "[Notification] 審查意見通知已發送給 {} 人（計畫 {}）",
            count,
            protocol_no
        );
        Ok(count)
    }
}
