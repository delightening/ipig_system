// 動物相關通知（獸醫建議 + 緊急給藥）

use uuid::Uuid;

use crate::{error::AppError, models::NotificationType};

use super::{EventContext, NotificationPayload, NotificationService};

impl NotificationService {
    // notify_vet_recommendation 已隨 vet_recommendations 功能退役移除。

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
        let notification_title = format!("🚨 [緊急] 緊急給藥 - 耳號 {}", ear_tag);
        let content = format!(
            "緊急給藥執行通知\n\n此紀錄需要補簽審核。\n\n耳號：{}\nIACUC No.：{}\n執行者：{}\n緊急原因：{}\n\n請儘速審核此緊急給藥紀錄。",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            operator_name,
            emergency_reason
        );

        // 解析所屬計畫 id（供 protocol_pi resolver 通知該計畫 PI）。
        let protocol_id = match iacuc_no {
            Some(iacuc) => {
                sqlx::query_scalar::<_, Uuid>(
                    "SELECT id FROM protocols WHERE iacuc_no = $1 LIMIT 1",
                )
                .bind(iacuc)
                .fetch_optional(&self.db)
                .await?
            }
            None => None,
        };

        // 統一派送：VET（角色）+ 該計畫 PI（resolver protocol_pi）皆由 notification_routing 決定。
        let count = self
            .dispatch_event(
                "emergency_medication",
                &EventContext {
                    protocol_id,
                    ..Default::default()
                },
                NotificationPayload {
                    notification_type: NotificationType::VetRecommendation,
                    title: notification_title,
                    content: Some(content),
                    related_entity_type: Some("animal".to_string()),
                    related_entity_id: Some(animal_id),
                },
            )
            .await?;

        tracing::warn!(
            "[Emergency Medication] Alert dispatched to {} recipients for animal {} (observation {})",
            count,
            ear_tag,
            observation_id
        );

        Ok(count)
    }

    /// 通知動物異常紀錄
    /// 依 notification_routing 表查詢 `animal_abnormal_record` 事件的收件者（通常為 VET）
    pub async fn notify_abnormal_record(
        &self,
        animal_id: Uuid,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        record_summary: &str,
        operator_name: &str,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 動物異常紀錄 - 耳號 {}", ear_tag);
        let content = format!(
            "有新的動物異常紀錄需要關注。\n\n耳號：{}\nIACUC No.：{}\n紀錄摘要：{}\n記錄者：{}",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            record_summary,
            operator_name
        );

        // 統一派送：收件人與管道由 notification_routing 決定（animal_abnormal_record → 角色型）。
        let count = self
            .dispatch_event(
                "animal_abnormal_record",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::VetRecommendation,
                    title,
                    content: Some(content),
                    related_entity_type: Some("animal".to_string()),
                    related_entity_id: Some(animal_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] 動物異常紀錄通知已發送給 {} 人（耳號 {}）",
            count,
            ear_tag
        );
        Ok(count)
    }

    /// 通知動物猝死登記（依 notification_routing 的 `animal_sudden_death` 事件 → 角色 VET）。
    /// `reporter_user_id`：登記者 user id，傳入後 dispatch 會排除「通知自己」（登記者本身為 VET 時）。
    pub async fn notify_sudden_death(
        &self,
        animal_id: Uuid,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        reporter_name: &str,
        reporter_user_id: Option<Uuid>,
    ) -> Result<i32, AppError> {
        let title = format!("[iPig] 動物猝死登記 - 耳號 {}", ear_tag);
        let content = format!(
            "有動物猝死登記，請關注。\n\n耳號：{}\nIACUC No.：{}\n登記者：{}",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            reporter_name
        );

        // 統一派送：收件人與管道由 notification_routing 決定（animal_sudden_death → 角色 VET）。
        let count = self
            .dispatch_event(
                "animal_sudden_death",
                &EventContext {
                    actor_id: reporter_user_id,
                    ..Default::default()
                },
                NotificationPayload {
                    notification_type: NotificationType::SystemAlert,
                    title,
                    content: Some(content),
                    related_entity_type: Some("animal".to_string()),
                    related_entity_id: Some(animal_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] 動物猝死通知已發送給 {} 人（耳號 {}）",
            count,
            ear_tag
        );
        Ok(count)
    }
}
