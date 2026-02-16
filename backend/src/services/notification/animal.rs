// 動物相關通知（獸醫建議 + 緊急給藥）

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知獸醫師建議
    /// - 一般建議：僅站內通知 PI/Coeditor
    /// - 緊急建議：站內通知 + Email 給 PI/Coeditor
    pub async fn notify_vet_recommendation(
        &self,
        animal_id: Uuid,
        ear_tag: &str,
        protocol_id: Option<Uuid>,  // 透過計畫 ID 找 PI/Coeditor
        record_type: &str,
        recommendation_content: &str,
        is_urgent: bool,            // 緊急標記
        config: Option<&crate::config::Config>,  // 用於發送 Email
    ) -> Result<i32, AppError> {
        // 取得該計畫的 PI 和 Coeditor
        let recipients: Vec<(Uuid, String, String)> = if let Some(protocol_id) = protocol_id {
            sqlx::query_as(
                r#"
                SELECT DISTINCT u.id, u.email, u.display_name
                FROM users u
                JOIN user_protocols up ON u.id = up.user_id
                WHERE up.protocol_id = $1 
                  AND up.role IN ('PI', 'COEDITOR')
                  AND u.is_active = true
                "#,
            )
            .bind(protocol_id)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        if recipients.is_empty() {
            tracing::warn!(
                "No PI/Coeditor found for animal {} (protocol_id: {:?}), skipping notification",
                ear_tag,
                protocol_id
            );
            return Ok(0);
        }

        // 取得 IACUC NO（用於通知內容）
        let iacuc_no: Option<String> = if let Some(protocol_id) = protocol_id {
            sqlx::query_scalar(
                "SELECT iacuc_no FROM protocols WHERE id = $1"
            )
            .bind(protocol_id)
            .fetch_optional(&self.db)
            .await?
            .flatten()
        } else {
            None
        };

        let urgency_prefix = if is_urgent { "🚨 [緊急] " } else { "" };
        let notification_title = format!("{}[iPig] 獸醫師建議 - 耳號 {}", urgency_prefix, ear_tag);
        let content = format!(
            "獸醫師已對以下動物新增照護建議，請查閱並執行。\n\n耳號：{}\nIACUC No.：{}\n紀錄類型：{}\n建議內容：{}",
            ear_tag,
            iacuc_no.as_deref().unwrap_or("-"),
            record_type,
            recommendation_content
        );

        let mut count = 0;
        for (user_id, email, display_name) in &recipients {
            // 1. 永遠發送站內通知
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::VetRecommendation,
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("animal".to_string()),
                    related_entity_id: Some(animal_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;

            // 2. 緊急建議才發送 Email
            if is_urgent {
                if let Some(cfg) = config {
                    if let Err(e) = crate::services::EmailService::send_vet_recommendation_email(
                        cfg,
                        email,
                        display_name,
                        ear_tag,
                        iacuc_no.as_deref(),
                        record_type,
                        recommendation_content,
                        true,  // is_urgent
                    )
                    .await {
                        tracing::warn!("發送獸醫建議通知郵件失敗: {e}");
                    }
                }
            }
        }

        tracing::info!(
            "Vet recommendation notification sent to {} recipients for animal {} (urgent: {})",
            count,
            ear_tag,
            is_urgent
        );

        Ok(count)
    }

    /// 緊急給藥通知（發送給 VET 和 PI）
    /// 當實驗工作人員在獸醫不在時緊急執行給藥，系統將發送紅色警報
    pub async fn notify_emergency_medication(
        &self,
        animal_id: Uuid,
        observation_id: Uuid,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        operator_name: &str,
        emergency_reason: &str,
    ) -> Result<i32, AppError> {
        // 取得所有 VET 和擁有該計畫的 PI
        let mut recipients: Vec<(Uuid, String)> = Vec::new();

        // 取得所有獸醫師
        let vets: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.display_name
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true AND r.code = 'VET'
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        recipients.extend(vets);

        // 如果有 IACUC NO，也通知該計畫的 PI
        if let Some(iacuc) = iacuc_no {
            let pis: Vec<(Uuid, String)> = sqlx::query_as(
                r#"
                SELECT DISTINCT u.id, u.display_name
                FROM users u
                JOIN user_protocols up ON u.id = up.user_id
                JOIN protocols p ON up.protocol_id = p.id
                WHERE p.iacuc_no = $1 AND up.role = 'PI' AND u.is_active = true
                "#,
            )
            .bind(iacuc)
            .fetch_all(&self.db)
            .await?;
            recipients.extend(pis);
        }

        // 去除重複的收件者
        recipients.sort_by_key(|(id, _)| *id);
        recipients.dedup_by_key(|(id, _)| *id);

        let notification_title = format!("🚨 [緊急] 緊急給藥 - 耳號 {}", ear_tag);
        let content = format!(
            "緊急給藥執行通知\n\n此紀錄需要補簽審核。\n\n耳號：{}\nIACUC No.：{}\n執行者：{}\n緊急原因：{}\n\n請儘速審核此緊急給藥紀錄。",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            operator_name,
            emergency_reason
        );

        let mut count = 0;
        for (user_id, _name) in recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::VetRecommendation, // 使用既有類型，或考慮新增 EmergencyMedication 類型
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("animal".to_string()),
                    related_entity_id: Some(animal_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::warn!(
            "[Emergency Medication] Alert sent to {} recipients for animal {} (observation {})",
            count,
            ear_tag,
            observation_id
        );

        Ok(count)
    }
}
