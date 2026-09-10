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

    /// 領用被儲位庫存擋下 → 通知倉管與當下操作的人。
    ///
    /// 🔴 **只能在 `tx.commit()`／rollback 之後呼叫。** 擋下的當下整張核准會回滾，
    /// 寫在那個 tx 裡的通知會一起消失——這正是本函式獨立於 `StockService` 存在的理由。
    ///
    /// 為什麼要通知而不是只回錯誤給操作者：使用者裁定的規則是「儲藏室平常不盤，
    /// 缺貨或有異狀時才盤」，而**領不出來就是那個異狀**。只有操作者看到錯誤的話，
    /// 這件事止於他當下的挫折；倉管要知道才會去盤那個儲位。
    ///
    /// dedup：同一 (儲位, 品項) 24 小時內只發一則。不去重的話同一個人連按三次就三則，
    /// 倉管會被淹沒而不再看——那等於這則通知沒有存在過。
    pub async fn notify_stock_blocked(
        &self,
        storage_location_id: Uuid,
        product_id: Uuid,
        message: &str,
        doc_no: &str,
        operator_id: Option<Uuid>,
    ) -> Result<i32, AppError> {
        let (sku, product_name, loc_code) =
            self.blocked_labels(storage_location_id, product_id).await;
        let title = format!("[iPig] 領用卡關 - {} @ {}", sku, loc_code);
        let content = format!(
            "{}\n\n單據：{}\n品項：{} {}\n儲位：{}",
            message, doc_no, sku, product_name, loc_code
        );
        self.dispatch_storage_alert(storage_location_id, operator_id, title, content)
            .await
    }

    /// 儲位帳失真（領用時該儲位根本沒有庫存列，系統靜默放行）→ 通知倉管與操作者。
    ///
    /// 🔴 同樣只能在 commit 之後呼叫，理由與 `notify_stock_blocked` 相同。
    ///
    /// 這條補的是「領不出來就通知」的另一半：缺 row 的品項**根本卡不住**，
    /// 領用照樣過，於是儲位帳一路失真而沒有任何人會發現。行為刻意不改
    /// （不擋，避免上線當天大面積卡住現場），改成讓它出聲。
    pub async fn notify_storage_drift(
        &self,
        storage_location_id: Uuid,
        product_id: Uuid,
        qty: rust_decimal::Decimal,
        doc_no: &str,
        operator_id: Option<Uuid>,
    ) -> Result<i32, AppError> {
        let (sku, product_name, loc_code) =
            self.blocked_labels(storage_location_id, product_id).await;
        let title = format!("[iPig] 儲位帳失真 - {} @ {}", sku, loc_code);
        // `normalize()` 的理由同 `ledger.rs` 的 `insufficient_stock_message`：
        // qty 來自 numeric 欄位，不去尾隨零會印成「領出了 3.0000 個單位」。
        // 兩處都是給現場看的文字，要一起處理才不會只修一半（CodeRabbit 在 MR !3 同時點名兩處）。
        let content = format!(
            "單據 {} 從{}領出了 {} 個單位的「{} {}」，但系統在該儲位查無這個品項的庫存列，\
             領用仍照常放行（倉庫層級的總量檢查有守住，所以倉庫總數沒錯）。\n\n\
             這代表該儲位的帳與實體已經對不起來，且不會自己恢復。請安排盤點該儲位以建立正確的基準。",
            doc_no,
            loc_code,
            qty.normalize(),
            sku,
            product_name
        );
        self.dispatch_storage_alert(storage_location_id, operator_id, title, content)
            .await
    }

    /// 兩則儲位警示共用：查人看得懂的標示。查不到就退回 UUID——
    /// 這是輔助資訊，不該讓它的失敗蓋掉「有東西不對勁」這件事本身。
    async fn blocked_labels(
        &self,
        storage_location_id: Uuid,
        product_id: Uuid,
    ) -> (String, String, String) {
        let prod: Option<(String, String)> =
            sqlx::query_as("SELECT sku, name FROM products WHERE id = $1")
                .bind(product_id)
                .fetch_optional(&self.db)
                .await
                .ok()
                .flatten();
        let loc: Option<(String, Option<String>)> =
            sqlx::query_as("SELECT code, name FROM storage_locations WHERE id = $1")
                .bind(storage_location_id)
                .fetch_optional(&self.db)
                .await
                .ok()
                .flatten();
        let (sku, product_name) =
            prod.unwrap_or_else(|| (product_id.to_string(), String::from("(查無品項)")));
        let loc_code = match loc {
            Some((code, Some(name))) => format!("{code} {name}"),
            Some((code, None)) => code,
            None => storage_location_id.to_string(),
        };
        (sku, product_name, loc_code)
    }

    /// 兩則儲位警示共用的派送：收件人 = 全體倉管 + 當下操作的人（去重），
    /// 同一 (儲位, 品項) 24 小時內只發一則。
    ///
    /// dedup 用 `title` 精確比對而非 `LIKE`：title 已含 SKU 與儲位，本身就是那組 key
    /// 的唯一表示。用 `LIKE '%...%'` 會在某個 SKU 恰為另一個的前綴時把兩者當成同一則，
    /// 於是其中一個永遠收不到通知。
    async fn dispatch_storage_alert(
        &self,
        storage_location_id: Uuid,
        operator_id: Option<Uuid>,
        title: String,
        content: String,
    ) -> Result<i32, AppError> {
        let managers = self
            .get_users_by_role(crate::constants::ROLE_WAREHOUSE_MANAGER)
            .await?;
        let mut recipients: Vec<Uuid> = managers.iter().map(|(id, ..)| *id).collect();
        // 操作者也要收到——他才是站在架子前面的人。他可能同時是倉管，故去重。
        if let Some(op) = operator_id {
            if !recipients.contains(&op) {
                recipients.push(op);
            }
        }
        if recipients.is_empty() {
            tracing::warn!("[Notification] 儲位警示無收件者（查無倉管且無操作者），跳過：{title}");
            return Ok(0);
        }

        let already: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT user_id
               FROM notifications
               WHERE user_id = ANY($1)
                 AND related_entity_type = 'storage_location'
                 AND related_entity_id = $2
                 AND title = $3
                 AND created_at > NOW() - INTERVAL '24 hours'"#,
        )
        .bind(&recipients)
        .bind(storage_location_id)
        .bind(&title)
        .fetch_all(&self.db)
        .await?;
        let notified: std::collections::HashSet<Uuid> =
            already.into_iter().map(|(id,)| id).collect();

        let mut count = 0;
        for user_id in &recipients {
            if notified.contains(user_id) {
                continue;
            }
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::SystemAlert,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("storage_location".to_string()),
                    related_entity_id: Some(storage_location_id),
                })
                .await
            {
                tracing::warn!("建立儲位警示通知失敗 user={user_id}: {e}");
                continue;
            }
            count += 1;
        }

        tracing::info!(
            "[Notification] {title}：{} 筆新通知（收件者 {} 位，24 小時內已通知過 {} 位）",
            count,
            recipients.len(),
            notified.len()
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
