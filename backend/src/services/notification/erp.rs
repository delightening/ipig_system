// ERP 採購單通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

impl NotificationService {
    /// 通知採購單已提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_document_submitted(
        &self,
        document_id: Uuid,
        document_no: &str,
        doc_type: &str,
        creator_name: &str,
    ) -> Result<i32, AppError> {
        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("document_submitted").await?;

        let type_text = match doc_type {
            "PO" => "採購單",
            "PR" => "採購退貨",
            "SO" => "銷貨單",
            "DO" => "銷貨出庫",
            _ => doc_type,
        };
        let title = format!("[iPig] 新{}待審核 - {}", type_text, document_no);
        let content = format!(
            "有新的{}待審核。\n\n單據編號：{}\n建立者：{}",
            type_text, document_no, creator_name
        );

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
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

    /// 通知倉管人員：已核准的採購單尚未建立入庫單（未入庫提醒）
    /// 查詢 status='approved' 且沒有任何已核准 GRN 的 PO
    pub async fn notify_po_pending_receipt(&self) -> Result<i32, AppError> {
        // 查詢已核准、但尚無任何已核准 GRN 入庫的 PO
        let pending_pos: Vec<(uuid::Uuid, String, String, chrono::NaiveDate)> = sqlx::query_as(
            r#"
            SELECT po.id, po.doc_no, COALESCE(p.name, '-') as partner_name, po.doc_date
            FROM documents po
            LEFT JOIN partners p ON po.partner_id = p.id
            WHERE po.doc_type = 'PO'
              AND po.status = 'approved'
              AND NOT EXISTS (
                  SELECT 1 FROM documents grn
                  WHERE grn.source_doc_id = po.id
                    AND grn.doc_type = 'GRN'
                    AND grn.status = 'approved'
              )
            ORDER BY po.doc_date ASC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        if pending_pos.is_empty() {
            tracing::info!("[Notification] 無已核准但未入庫的採購單");
            return Ok(0);
        }

        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("po_pending_receipt").await?;
        if recipients.is_empty() {
            tracing::warn!("[Notification] po_pending_receipt 事件無設定收件者，跳過通知");
            return Ok(0);
        }

        let title = format!(
            "[iPig] 採購單未入庫提醒：{} 筆採購單待入庫",
            pending_pos.len()
        );
        let content = {
            let list: Vec<String> = pending_pos
                .iter()
                .take(10)
                .map(|(_, doc_no, partner, date)| {
                    format!("• {} / {} / 單據日期：{}", doc_no, partner, date)
                })
                .collect();
            let mut text = format!(
                "以下採購單已核准但尚未完成入庫，請儘速處理。\n\n{}",
                list.join("\n")
            );
            if pending_pos.len() > 10 {
                text.push_str(&format!("\n\n...另有 {} 筆，請登入系統查看", pending_pos.len() - 10));
            }
            text
        };

        let mut count = 0;
        for (user_id, _email, _name, _channel) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::DocumentApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("document".to_string()),
                    related_entity_id: None, // 多筆 PO，不指定單一 entity
                })
                .await
            {
                tracing::warn!("建立未入庫通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!(
            "[Notification] 採購單未入庫提醒已發送給 {} 位倉管（共 {} 筆待入庫 PO）",
            count,
            pending_pos.len()
        );
        Ok(count)
    }

    /// 通知採購單已審核/駁回（給建立者，非路由表管理）
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
            "SO" => "銷貨單",
            "DO" => "銷貨出庫",
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
