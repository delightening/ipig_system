// ERP 採購單通知

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::{EventContext, NotificationPayload, NotificationService};

impl NotificationService {
    /// 通知採購單已提交（依 notification_routing 表判斷收件角色）
    pub async fn notify_document_submitted(
        &self,
        document_id: Uuid,
        document_no: &str,
        doc_type: &str,
        creator_name: &str,
    ) -> Result<i32, AppError> {
        let type_text = match doc_type {
            "PO" => "採購單",
            "PR" => "採購退貨",
            "SO" => "銷貨單",
            _ => doc_type,
        };
        let title = format!("[iPig] 新{}待審核 - {}", type_text, document_no);
        let content = format!(
            "有新的{}待審核。\n\n單據編號：{}\n建立者：{}",
            type_text, document_no, creator_name
        );

        // 統一派送：收件人與管道全由 notification_routing 決定（document_submitted → 角色型）。
        let count = self
            .dispatch_event(
                "document_submitted",
                &EventContext::default(),
                NotificationPayload {
                    notification_type: NotificationType::DocumentApproval,
                    title,
                    content: Some(content),
                    related_entity_type: Some("document".to_string()),
                    related_entity_id: Some(document_id),
                },
            )
            .await?;

        tracing::info!(
            "[Notification] {}提交通知已發送給 {} 位收件者",
            type_text,
            count
        );
        Ok(count)
    }

    /// 通知倉管人員：已核准的採購單尚未建立入庫單（未入庫提醒）
    ///
    /// 每張 PO **每一輪**只通知一次：dedup 只跳過置頂中（`priority > 0`）的既有通知，
    /// 入庫核准時 hook 會把它降級（`document/workflow.rs` 的 `resolve_pinned_notifications`），
    /// 該 PO 於是自然退出提醒；若之後入庫單被沖銷、PO 實質回到未入庫，就會重新提醒。
    pub async fn notify_po_pending_receipt(&self) -> Result<i32, AppError> {
        // R84-18：`NOT EXISTS (有效 GRN)` 必須排除已被沖銷者——原單沖銷後**仍是 `approved`**，
        // 不排除的話該 PO 永遠不會回到未入庫清單，倉管收不到「要重開一張正確的」的提醒。
        // 述詞取自 `document/grn.rs` 的 `exclude_reversed_grn!`（別名固定為 `g` 是它的前提），
        // 與入庫進度那四處同源。
        let pending_pos: Vec<(Uuid, String, String, chrono::NaiveDate)> = sqlx::query_as(concat!(
            r#"
            SELECT po.id, po.doc_no, COALESCE(p.name, '-') as partner_name, po.doc_date
            FROM documents po
            LEFT JOIN partners p ON po.partner_id = p.id
            WHERE po.doc_type = 'PO'
              AND po.status = 'approved'
              AND NOT EXISTS (
                  SELECT 1 FROM documents g
                  WHERE g.source_doc_id = po.id
                    AND g.doc_type = 'GRN'
                    AND g.status = 'approved'
                "#,
            crate::exclude_reversed_grn!(),
            r#"
              )
            ORDER BY po.doc_date ASC
            "#
        ))
        .fetch_all(&self.db)
        .await?;

        if pending_pos.is_empty() {
            tracing::info!("[Notification] 無已核准但未入庫的採購單");
            return Ok(0);
        }

        let recipients = self.get_recipients_by_event("po_pending_receipt").await?;
        if recipients.is_empty() {
            tracing::warn!("[Notification] po_pending_receipt 事件無設定收件者，跳過通知");
            return Ok(0);
        }

        let po_ids: Vec<Uuid> = pending_pos.iter().map(|(id, ..)| *id).collect();
        let recipient_ids: Vec<Uuid> = recipients.iter().map(|(id, ..)| *id).collect();

        // R84-18：dedup 的判準是「這一輪還在置頂嗎」，不是「這張 PO 這輩子通知過嗎」。
        //
        // 原本不帶 `priority > 0`，於是通知列一旦寫下去就永久擋住同一張 PO 的後續提醒——
        // 即使上一則早已被入庫 hook 降級（`RESOLVE_PINNED_SQL` 只改 priority，列仍留著）。
        // 沖銷之後 PO 實質回到未入庫，卻因為那筆歷史列而不再提醒，光修上面的 SQL 沒有用。
        //
        // 用 priority 而不是自己記一個時間基準：置頂與否本來就是「這件事還沒處理完」的
        // 單一事實來源（`chk_notifications_priority`／`idx_notifications_action_pending`），
        // 再造第二個判準就會有兩邊不同步的老問題。
        let already_notified: Vec<(Uuid, Uuid)> = sqlx::query_as(
            r#"SELECT user_id, related_entity_id
               FROM notifications
               WHERE user_id = ANY($1)
                 AND related_entity_type = 'document'
                 AND related_entity_id = ANY($2)
                 AND title LIKE '%未入庫提醒%'
                 AND priority > 0"#,
        )
        .bind(&recipient_ids)
        .bind(&po_ids)
        .fetch_all(&self.db)
        .await?;

        let notified_set: std::collections::HashSet<(Uuid, Uuid)> =
            already_notified.into_iter().collect();

        let mut count = 0;
        for (po_id, doc_no, partner, date) in &pending_pos {
            for (user_id, _email, _name, _channel) in &recipients {
                if notified_set.contains(&(*user_id, *po_id)) {
                    continue;
                }
                let title = format!("[iPig] 採購單未入庫提醒 - {}", doc_no);
                let content = format!(
                    "以下採購單已核准但尚未完成入庫，請儘速處理。\n\n單據編號：{}\n供應商：{}\n單據日期：{}",
                    doc_no, partner, date
                );
                // urgent：置頂顯示直到入庫完成（GRN 核准時由 hook 解除，見 document/workflow.rs）
                if let Err(e) = self
                    .create_pinned_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::DocumentApproval,
                        title,
                        content: Some(content),
                        related_entity_type: Some("document".to_string()),
                        related_entity_id: Some(*po_id),
                    })
                    .await
                {
                    tracing::warn!("建立未入庫通知失敗 PO={doc_no}: {e}");
                }
                count += 1;
            }
        }

        tracing::info!(
            "[Notification] 採購單未入庫提醒：{} 筆新通知（{} 筆待入庫 PO，{} 筆已通知過跳過）",
            count,
            pending_pos.len(),
            pending_pos.len() * recipients.len() - count as usize
        );
        Ok(count)
    }

    /// 稽核：有手術但時間窗內缺對應銷貨單據(DO/SO)的計畫 → 通知 SD + 倉管。
    ///
    /// 規則：手術日前後 `WINDOW_DAYS` 天內須有已核准 DO/SO；只看近 `LOOKBACK_DAYS` 天、
    /// 且已過窗（≥WINDOW_DAYS 天前）的手術，避免剛開完刀尚在補單期就誤報。
    /// 手術經 `animals.iacuc_no = protocols.iacuc_no` 串回計畫（animal_surgeries 無 protocol_id）。
    /// 以 notifications 表 dedup（每計畫每收件者只通知一次），避免每日重複轟炸。
    pub async fn notify_surgery_missing_sales(&self) -> Result<i32, AppError> {
        const LOOKBACK_DAYS: i32 = 60;
        const WINDOW_DAYS: i32 = 7;

        let non_compliant: Vec<(Uuid, String, Option<Uuid>, i64, chrono::NaiveDate)> =
            sqlx::query_as(
                r#"
                SELECT p.id, p.protocol_no, sd.id AS study_director_user_id,
                       COUNT(*) AS unmatched_count, MIN(s.surgery_date) AS earliest
                FROM protocols p
                JOIN animals a ON a.iacuc_no = p.iacuc_no AND a.deleted_at IS NULL
                JOIN animal_surgeries s ON s.animal_id = a.id AND s.deleted_at IS NULL
                -- SD 僅在使用者啟用時納入通知（停用/缺值 → NULL，下游略過）
                LEFT JOIN users sd ON sd.id = p.study_director_user_id AND sd.is_active = true
                WHERE s.surgery_date BETWEEN CURRENT_DATE - $1::int AND CURRENT_DATE - $2::int
                  AND NOT EXISTS (
                      SELECT 1 FROM documents d
                      WHERE d.protocol_id = p.id
                        AND d.doc_type = 'SO'
                        AND d.status = 'approved'
                        AND d.doc_date BETWEEN s.surgery_date - $2::int AND s.surgery_date + $2::int
                  )
                GROUP BY p.id, p.protocol_no, sd.id
                ORDER BY MIN(s.surgery_date) ASC
                "#,
            )
            .bind(LOOKBACK_DAYS)
            .bind(WINDOW_DAYS)
            .fetch_all(&self.db)
            .await?;

        if non_compliant.is_empty() {
            tracing::info!("[Notification] 無手術缺銷貨單據的計畫");
            return Ok(0);
        }

        // 倉管（全體）
        let warehouse_managers = self
            .get_users_by_role(crate::constants::ROLE_WAREHOUSE_MANAGER)
            .await?;
        let wm_ids: Vec<Uuid> = warehouse_managers.iter().map(|(id, ..)| *id).collect();

        // dedup：已就該計畫通知過的 (user, protocol)
        let protocol_ids: Vec<Uuid> = non_compliant.iter().map(|(id, ..)| *id).collect();
        let already: Vec<(Uuid, Uuid)> = sqlx::query_as(
            r#"SELECT user_id, related_entity_id
               FROM notifications
               WHERE related_entity_type = 'protocol'
                 AND related_entity_id = ANY($1)
                 AND title LIKE '%手術缺銷貨單據%'"#,
        )
        .bind(&protocol_ids)
        .fetch_all(&self.db)
        .await?;
        let notified: std::collections::HashSet<(Uuid, Uuid)> = already.into_iter().collect();

        let mut count = 0;
        for (protocol_id, protocol_no, sd_id, unmatched, earliest) in &non_compliant {
            // 收件者 = 該計畫 SD（若有）+ 全體倉管，去重
            let mut recipients: Vec<Uuid> = Vec::new();
            if let Some(sd) = sd_id {
                recipients.push(*sd);
            }
            for id in &wm_ids {
                if !recipients.contains(id) {
                    recipients.push(*id);
                }
            }

            let title = format!("[iPig] 計畫 {} 手術缺銷貨單據", protocol_no);
            let content = format!(
                "計畫 {} 有 {} 筆手術在前後 {} 天內查無已核准的銷貨單據(DO/SO)，最早一筆手術日 {}。\n請確認是否漏開銷貨單據。",
                protocol_no, unmatched, WINDOW_DAYS, earliest
            );

            for user_id in &recipients {
                if notified.contains(&(*user_id, *protocol_id)) {
                    continue;
                }
                if let Err(e) = self
                    .create_notification(CreateNotificationRequest {
                        user_id: *user_id,
                        notification_type: NotificationType::SystemAlert,
                        title: title.clone(),
                        content: Some(content.clone()),
                        related_entity_type: Some("protocol".to_string()),
                        related_entity_id: Some(*protocol_id),
                    })
                    .await
                {
                    tracing::warn!("建立手術缺銷貨通知失敗 protocol={protocol_no}: {e}");
                }
                count += 1;
            }
        }

        tracing::info!(
            "[Notification] 手術缺銷貨單據稽核：{} 筆新通知（{} 個計畫缺單）",
            count,
            non_compliant.len()
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
            _ => doc_type,
        };
        let decision = if is_approved {
            "已核准"
        } else {
            "已駁回"
        };
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
