// 設備維護相關通知

use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::ActorContext,
    models::{CreateNotificationRequest, NotificationType},
    services::{EmailService, RenderedEmail},
};

use super::dispatch::StaffEmail;
use super::{EventContext, NotificationPayload, NotificationService};

impl NotificationService {
    /// 設備通知 email 分派共用 helper（DRY）：統一 actor / source / 收件人 / 錯誤記錄，
    /// 避免三處設備通知的時間窗 + 請假 + outbox 串接邏輯各自漂移。
    async fn dispatch_equipment_email(
        &self,
        reason: &'static str,
        user_id: Uuid,
        email: &str,
        name: &str,
        rendered: RenderedEmail,
    ) {
        let actor = ActorContext::System { reason };
        if let Err(e) = self
            .dispatch_staff_email(
                &actor,
                ("equipment", user_id),
                StaffEmail {
                    to_email: email.to_string(),
                    to_name: name.to_string(),
                    recipient_user_id: Some(user_id),
                    email: rendered,
                },
            )
            .await
        {
            tracing::warn!(reason, "分派設備通知 email 失敗: {e}");
        }
    }

    /// 發送設備校正/確效逾期通知（排程用）
    pub async fn send_equipment_overdue_notifications(&self) -> Result<i32, AppError> {
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
                        related_entity_type: Some("equipment".to_string()),
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立設備逾期通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                let rendered = EmailService::render_equipment_overdue_email(
                    name,
                    &overdue_html,
                    overdue.len(),
                );
                self.dispatch_equipment_email(
                    "equipment_overdue_notification",
                    *user_id,
                    email,
                    name,
                    rendered,
                )
                .await;
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
                        related_entity_type: Some("equipment".to_string()),
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立無法維修通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                let rendered = EmailService::render_equipment_unrepairable_email(
                    name,
                    equipment_name,
                    serial_number,
                    problem,
                );
                self.dispatch_equipment_email(
                    "equipment_unrepairable_notification",
                    *user_id,
                    email,
                    name,
                    rendered,
                )
                .await;
            }
        }

        Ok(())
    }

    /// 發送設備報修通知（通知維修人員）
    pub async fn send_equipment_repair_notification(
        &self,
        equipment_name: &str,
        reporter_name: &str,
        problem: &str,
    ) -> Result<(), AppError> {
        let recipients = self
            .get_recipients_by_event("equipment_maintenance_review")
            .await?;

        for (user_id, _email, _name, channel) in &recipients {
            if Self::should_send_in_app(channel) {
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::SystemAlert,
                        title: format!("設備報修：{}", equipment_name),
                        content: Some(format!(
                            "{} 提報設備「{}」需要維修，問題描述：{}",
                            reporter_name, equipment_name, problem
                        )),
                        related_entity_type: Some("equipment".to_string()),
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立報修通知失敗: {e}");
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
                        related_entity_type: Some("equipment".to_string()),
                        related_entity_id: None,
                    })
                    .await
                {
                    tracing::warn!("建立報廢通知失敗: {e}");
                }
            }

            if Self::should_send_email(channel) {
                let rendered = EmailService::render_equipment_disposal_email(
                    name,
                    equipment_name,
                    applicant_name,
                    reason,
                );
                self.dispatch_equipment_email(
                    "equipment_disposal_notification",
                    *user_id,
                    email,
                    name,
                    rendered,
                )
                .await;
            }
        }

        Ok(())
    }

    /// 依 id 反查設備名稱（**查無此列**以「設備」代稱），供審核結果通知組字串用。
    /// DB 錯誤以 `?` 傳播（禁 unwrap_or 靜默降級）：暫時性查詢失敗不得無聲退化成
    /// 泛稱送出，讓呼叫端（核准後通知，best-effort 層）決定如何記錄。
    async fn equipment_name_by_id(&self, equipment_id: Uuid) -> Result<String, AppError> {
        let name = sqlx::query_scalar::<_, String>("SELECT name FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_optional(&self.db)
            .await?;
        Ok(name.unwrap_or_else(|| "設備".to_string()))
    }

    /// 設備報廢審核結果通知申請人（核准 / 駁回都通知）。
    /// 收件人＝申請人本人（resolver event_subject，由 subject_user_id 提供並驗證 active）。
    pub async fn notify_equipment_disposal_result(
        &self,
        disposal_id: Uuid,
        applicant_id: Uuid,
        equipment_id: Uuid,
        approved: bool,
        rejection_reason: Option<&str>,
    ) -> Result<i32, AppError> {
        let equipment_name = self.equipment_name_by_id(equipment_id).await?;
        let (title, result_word) = if approved {
            ("[iPig] 您的設備報廢申請已核准", "核准")
        } else {
            ("[iPig] 您的設備報廢申請已駁回", "駁回")
        };
        let reason_text = rejection_reason
            .filter(|r| !r.is_empty())
            .map(|r| format!("\n原因：{}", r))
            .unwrap_or_default();
        let content = format!(
            "您提出的設備報廢申請已{}。\n\n設備：{}{}",
            result_word, equipment_name, reason_text
        );
        self.dispatch_event(
            "equipment_disposal_result",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::SystemAlert,
                title: title.to_string(),
                content: Some(content),
                related_entity_type: Some("equipment".to_string()),
                related_entity_id: Some(disposal_id),
            },
        )
        .await
    }

    /// 設備維修/保養驗收結果通知登錄人（通過 / 退回都通知）。
    /// 收件人＝登錄人本人（resolver event_subject，由 subject_user_id 提供並驗證 active）。
    pub async fn notify_equipment_maintenance_result(
        &self,
        record_id: Uuid,
        applicant_id: Uuid,
        equipment_id: Uuid,
        approved: bool,
        review_notes: Option<&str>,
    ) -> Result<i32, AppError> {
        let equipment_name = self.equipment_name_by_id(equipment_id).await?;
        let (title, result_word) = if approved {
            ("[iPig] 您登錄的維修/保養已驗收通過", "驗收通過")
        } else {
            ("[iPig] 您登錄的維修/保養已退回", "退回")
        };
        let note_text = review_notes
            .filter(|r| !r.is_empty())
            .map(|r| format!("\n驗收意見：{}", r))
            .unwrap_or_default();
        let content = format!(
            "您登錄的設備維修/保養紀錄已{}。\n\n設備：{}{}",
            result_word, equipment_name, note_text
        );
        self.dispatch_event(
            "equipment_maintenance_result",
            &EventContext {
                subject_user_id: Some(applicant_id),
                ..Default::default()
            },
            NotificationPayload {
                notification_type: NotificationType::SystemAlert,
                title: title.to_string(),
                content: Some(content),
                related_entity_type: Some("equipment".to_string()),
                related_entity_id: Some(record_id),
            },
        )
        .await
    }
}
