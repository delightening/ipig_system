// 通知 CRUD 操作 + 設定

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, Notification, NotificationItem, NotificationQuery,
        NotificationSettings, PaginatedResponse, UpdateNotificationSettingsRequest, ENTRY_BELL,
        ENTRY_TODO, KIND_ACTION, KIND_INFO, PRIORITY_NORMAL, PRIORITY_PINNED,
    },
};

use super::NotificationService;

/// 解除置頂待辦的唯一 SQL 定義，pool 版與 tx 版共用。
///
/// 以 `related_entity_type + related_entity_id` 定位（同一業務實體上僅一則置頂待辦），
/// 不比對標題字串——避免文字微調 / 多語系翻譯導致 `LIKE` 失效而漏解除。
/// `priority > $3` 保證只動置頂列、不誤降既有一般通知。
const RESOLVE_PINNED_SQL: &str = r#"
    UPDATE notifications
    SET priority = $3,
        is_read  = true,
        read_at  = COALESCE(read_at, NOW())
    WHERE related_entity_type = $1
      AND related_entity_id   = $2
      AND priority > $3
"#;

impl NotificationService {
    /// 取得使用者通知列表
    pub async fn list_notifications(
        &self,
        user_id: Uuid,
        query: &NotificationQuery,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<NotificationItem>, AppError> {
        // SEC: clamp 防無上限 per_page 拉爆 DB + saturating 防 page 溢位 panic
        let page = page.max(1);
        let per_page = per_page.clamp(1, crate::constants::MAX_PAGE_SIZE);
        let offset = (page - 1).saturating_mul(per_page);

        // 建立基本查詢（使用 QueryBuilder 避免 SQL injection）
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            r#"
            SELECT id, type::TEXT, title, content, is_read, read_at,
                   related_entity_type, related_entity_id, created_at, priority, kind
            FROM notifications
            WHERE user_id = "#,
        );
        qb.push_bind(user_id);

        let mut count_qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE user_id = "#,
        );
        count_qb.push_bind(user_id);

        if let Some(is_read) = query.is_read {
            qb.push(" AND is_read = ").push_bind(is_read);
            count_qb.push(" AND is_read = ").push_bind(is_read);
        }

        if let Some(ref notification_type) = &query.notification_type {
            qb.push(" AND type::TEXT = ")
                .push_bind(notification_type.clone());
            count_qb
                .push(" AND type::TEXT = ")
                .push_bind(notification_type.clone());
        }

        // 入口篩選。兩個入口的判準只寫在這裡一處（與兩個計數共用同一句 SQL 述詞），
        // 前端只說「我是哪個入口」，不自己拼條件——否則清單與紅點會各算各的而對不起來。
        //
        // 鈴鐺不是 kind='info'：已完成的待辦（kind='action' 但 priority=0）也屬鈴鐺，
        // 那正是「我當初處理過哪些事」的歷史。
        //
        // 未帶 entry ＝ 不篩選，維持舊前端相容（部署期間新舊前端並存）。
        match query.entry.as_deref() {
            Some(ENTRY_TODO) => {
                for b in [&mut qb, &mut count_qb] {
                    b.push(" AND kind = ")
                        .push_bind(KIND_ACTION)
                        .push(" AND priority > ")
                        .push_bind(PRIORITY_NORMAL);
                }
            }
            Some(ENTRY_BELL) => {
                for b in [&mut qb, &mut count_qb] {
                    b.push(" AND NOT (kind = ")
                        .push_bind(KIND_ACTION)
                        .push(" AND priority > ")
                        .push_bind(PRIORITY_NORMAL)
                        .push(")");
                }
            }
            // 無法辨識的值一律當「未帶」，不因打錯字回空清單而讓使用者以為東西不見了
            _ => {}
        }

        // 緊急置頂（priority=1）永遠排在最上方，其餘按時間新到舊。
        qb.push(" ORDER BY priority DESC, created_at DESC LIMIT ")
            .push_bind(per_page);
        qb.push(" OFFSET ").push_bind(offset);

        let notifications: Vec<NotificationItem> = qb.build_query_as().fetch_all(&self.db).await?;

        let total: (i64,) = count_qb.build_query_as().fetch_one(&self.db).await?;

        Ok(PaginatedResponse::new(
            notifications,
            total.0,
            page,
            per_page,
        ))
    }

    /// 取得未讀**通知**數量（鈴鐺紅點）。
    ///
    /// 排除待辦：待辦有自己的入口與計數（[`Self::get_action_required_count`]），
    /// 兩邊都算會讓同一件事在畫面上被數兩次。
    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE user_id = $1
              AND is_read = false
              AND NOT (kind = $2 AND priority > $3)
            "#,
        )
        .bind(user_id)
        .bind(KIND_ACTION)
        .bind(PRIORITY_NORMAL)
        .fetch_one(&self.db)
        .await?;

        Ok(result.0)
    }

    /// 取得**待處理**數量（驚嘆號紅點）。
    ///
    /// 判準是 `kind='action' AND priority>0`，**不看 `is_read`** ——
    /// 待辦的存否由業務狀態決定，不由使用者看過與否決定。
    pub async fn get_action_required_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE user_id = $1 AND kind = $2 AND priority > $3
            "#,
        )
        .bind(user_id)
        .bind(KIND_ACTION)
        .bind(PRIORITY_NORMAL)
        .fetch_one(&self.db)
        .await?;

        Ok(result.0)
    }

    /// 標記指定通知為已讀。
    ///
    /// ⚠️ 同 [`Self::mark_all_as_read`]，**排除未完成的待辦**。
    /// 前端的待處理清單不提供「標為已讀」按鈕，但 API 是公開端點 ——
    /// 只靠前端不給按鈕擋不住直接呼叫，規則必須落在後端。
    pub async fn mark_as_read(
        &self,
        user_id: Uuid,
        notification_ids: &[Uuid],
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1
              AND id = ANY($2)
              AND NOT (kind = $3 AND priority > $4)
            "#,
        )
        .bind(user_id)
        .bind(notification_ids)
        .bind(KIND_ACTION)
        .bind(PRIORITY_NORMAL)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 標記所有通知為已讀。
    ///
    /// ⚠️ **排除未完成的待辦**（`kind='action' AND priority>0`）。
    ///
    /// 使用者裁定「待辦只能由系統偵測動作完成後自動消失，不可手動略過」。
    /// 若「全部已讀」把待辦也標掉，等於開了一條手動清除的後門。
    ///
    /// 這不是理論顧慮：2026-08-07 查 prod 時，既有的置頂待辦 `is_read` **全部是 true**
    /// —— 使用者早就按過「全部已讀」，只是當時前端用 `priority` 而非 `is_read` 決定
    /// 是否顯示，才沒被掩蓋掉。分家後「待處理」清單若改看 `is_read`，那一按就全清了。
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1
              AND is_read = false
              AND NOT (kind = $2 AND priority > $3)
            "#,
        )
        .bind(user_id)
        .bind(KIND_ACTION)
        .bind(PRIORITY_NORMAL)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 刪除通知
    pub async fn delete_notification(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE user_id = $1 AND id = $2
            "#,
        )
        .bind(user_id)
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Notification not found".to_string()));
        }

        Ok(())
    }

    /// 建立通知（一般 priority=0）
    pub async fn create_notification(
        &self,
        request: CreateNotificationRequest,
    ) -> Result<Notification, AppError> {
        let mut tx = self.db.begin().await?;
        let notification = Self::create_notification_tx(&mut tx, request).await?;
        tx.commit().await?;
        Ok(notification)
    }

    /// 建立「緊急置頂」通知（priority=1）。
    ///
    /// 用於採購未入庫、巡場待填追蹤等「待辦，完成前需置頂」的提醒；完成對應動作後由
    /// [`Self::resolve_pinned_notifications`] 降級回 0。
    pub async fn create_pinned_notification(
        &self,
        request: CreateNotificationRequest,
    ) -> Result<Notification, AppError> {
        self.create_pinned_notification_with_role(request, None)
            .await
    }

    /// [`Self::create_pinned_notification`]，但額外標記收件人身分（見
    /// [`Self::create_pinned_notification_tx_with_role`]）。目前唯一呼叫端是
    /// `dispatch_pinned_event`（`leave_submitted` 事件），供 [`Self::dispatch_pinned_event`]
    /// 把角色一路帶進來。
    pub async fn create_pinned_notification_with_role(
        &self,
        request: CreateNotificationRequest,
        recipient_role: Option<&'static str>,
    ) -> Result<Notification, AppError> {
        let mut tx = self.db.begin().await?;
        let notification = Self::create_notification_tx_with_priority(
            &mut tx,
            request,
            PRIORITY_PINNED,
            recipient_role,
        )
        .await?;
        tx.commit().await?;
        Ok(notification)
    }

    /// 完成對應待辦後，解除該實體關聯的置頂通知（priority→NORMAL + 標記已讀）。
    ///
    /// 以 `related_entity_type + related_entity_id` 定位（同一業務實體上僅一則置頂待辦），
    /// 不比對標題字串——避免文字微調 / 多語系翻譯導致 `LIKE` 失效而漏解除。
    /// `priority > NORMAL` 保證只動置頂列、不誤降既有一般通知。best-effort，回傳受影響列數。
    pub async fn resolve_pinned_notifications(
        &self,
        related_entity_type: &str,
        related_entity_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(RESOLVE_PINNED_SQL)
            .bind(related_entity_type)
            .bind(related_entity_id)
            .bind(PRIORITY_NORMAL)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected())
    }

    /// tx 內版本的 [`Self::create_pinned_notification`]。
    ///
    /// 與 [`Self::resolve_pinned_notifications_tx`] 成對：**建立與解除都應該在
    /// 業務狀態轉換自己的 tx 內**，兩端才一致。
    ///
    /// 只有解除在 tx 內、建立仍是 commit 後 best-effort 的話，會留下兩個洞：
    /// 建立失敗時使用者永遠收不到待辦（而對帳只找殘留、不找漏建）；
    /// 以及「送出 commit → 併發的撤回解除（掃不到尚未建立的列）→ 建立」的孤兒時序。
    pub async fn create_pinned_notification_tx(
        tx: &mut Transaction<'_, Postgres>,
        request: CreateNotificationRequest,
    ) -> Result<Notification, AppError> {
        Self::create_notification_tx_with_priority(tx, request, PRIORITY_PINNED, None).await
    }

    /// [`Self::create_pinned_notification_tx`]，但額外標記這則置頂待辦的收件人身分
    /// （[`crate::models::RECIPIENT_ROLE_PROXY`] / [`crate::models::RECIPIENT_ROLE_APPROVER`]）。
    ///
    /// 只有請假流程需要——同一人可能同時是代理人與下一關核准人，若不記下「這則 pin
    /// 是因為什麼身分建立」，reconcile 對帳只能靠「收件人是誰」猜用途，兩種身分重疊時
    /// 會猜錯而誤清真正待處理的核准 pin（見 `notification/reconcile.rs` 與 R92-1）。
    pub async fn create_pinned_notification_tx_with_role(
        tx: &mut Transaction<'_, Postgres>,
        request: CreateNotificationRequest,
        recipient_role: &'static str,
    ) -> Result<Notification, AppError> {
        Self::create_notification_tx_with_priority(
            tx,
            request,
            PRIORITY_PINNED,
            Some(recipient_role),
        )
        .await
    }

    /// tx 內版本的 [`Self::resolve_pinned_notifications`]。
    ///
    /// **業務狀態轉換一律用這個，不要用 pool 版**——先 commit 狀態、再另外解除待辦
    /// 會開一個競態窗口：
    ///
    /// 1. 撤回 commit（status → draft，follow_up_user_id → NULL）
    /// 2. 獸醫立刻重新送出 → 建立**新的**置頂待辦 N2
    /// 3. 撤回流程這才呼叫解除 → 把 N2 一起降級
    ///
    /// 結果是新指派的追蹤者永遠看不到自己的待辦，而待辦不可手動已讀、他無從自救——
    /// 正好是本 PR 要修的那個失效模式，只是換成競態觸發。
    ///
    /// 放進狀態轉換自己的 tx（該 tx 已對報告列 `SELECT ... FOR UPDATE`）後，
    /// 狀態轉換 rollback 時待辦不會被誤降，且併發的狀態轉換彼此被列鎖序列化。
    ///
    /// ⚠️ **窗口縮小但未完全消滅**，取決於「建立」那一端是否也在 tx 內：
    /// 若某個流程的置頂待辦是在 service tx **commit 之後**才由 handler 建立
    /// （best-effort 模式），則「送出 commit → 撤回取得列鎖並解除（此時 pin 尚未存在）
    /// → 送出的 handler 這才 INSERT pin」這條時序仍會留下孤兒。
    /// 巡場流程已把建立搬進 `submit_for_followup` 的 tx 以關掉這個窗口；
    /// **其他流程日後接上置頂待辦時務必比照辦理**，否則本函式的保證不成立，
    /// 只能靠每日對帳事後兜底（最長 24h）。
    pub async fn resolve_pinned_notifications_tx(
        tx: &mut Transaction<'_, Postgres>,
        related_entity_type: &str,
        related_entity_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(RESOLVE_PINNED_SQL)
            .bind(related_entity_type)
            .bind(related_entity_id)
            .bind(PRIORITY_NORMAL)
            .execute(&mut **tx)
            .await?;

        Ok(result.rows_affected())
    }

    /// R30-3b: tx 內寫站內通知。callers 可把「業務 mutation + audit + notification」
    /// 包進同一個 tx，達成 all-or-nothing。對應 [`OutboxService::enqueue_tx`] 處理
    /// 外部訊息（email / line / webhook）的另一面。
    ///
    /// 詳見 `docs/dev/notification-and-outbox.md`。
    pub async fn create_notification_tx(
        tx: &mut Transaction<'_, Postgres>,
        request: CreateNotificationRequest,
    ) -> Result<Notification, AppError> {
        Self::create_notification_tx_with_priority(tx, request, PRIORITY_NORMAL, None).await
    }

    /// tx 內寫站內通知，並指定 priority（0=一般 / 1=緊急置頂）與收件人身分標記
    /// （見 [`Self::create_pinned_notification_tx_with_role`]；一般通知一律傳 `None`）。
    /// [`Self::create_notification_tx`] 與 [`Self::create_pinned_notification`] 皆委派於此，
    /// 集中 INSERT 唯一定義。
    pub(super) async fn create_notification_tx_with_priority(
        tx: &mut Transaction<'_, Postgres>,
        request: CreateNotificationRequest,
        priority: i16,
        recipient_role: Option<&'static str>,
    ) -> Result<Notification, AppError> {
        let notification_type = request.notification_type.as_str();
        // kind 由 priority 決定：置頂＝待辦，其餘＝一般通知。
        // 兩者在此唯一的 INSERT 點綁定，避免呼叫端各自決定而漂移
        // （例如建了 priority=1 卻標成 info，該列就會同時不出現在兩個入口）。
        let kind = if priority > PRIORITY_NORMAL {
            KIND_ACTION
        } else {
            KIND_INFO
        };
        let notification: Notification = sqlx::query_as(
            r#"
            INSERT INTO notifications (id, user_id, type, title, content,
                                       related_entity_type, related_entity_id, priority, kind,
                                       recipient_role)
            VALUES (gen_random_uuid(), $1, $2::notification_type, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, type::TEXT, title, content, is_read, read_at,
                      related_entity_type, related_entity_id, created_at, priority, kind
            "#,
        )
        .bind(request.user_id)
        .bind(notification_type)
        .bind(&request.title)
        .bind(&request.content)
        .bind(&request.related_entity_type)
        .bind(request.related_entity_id)
        .bind(priority)
        .bind(kind)
        .bind(recipient_role)
        .fetch_one(&mut **tx)
        .await?;
        Ok(notification)
    }

    /// 清理過期通知（90 天前的已讀通知）
    pub async fn cleanup_old_notifications(&self) -> Result<i64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE is_read = true 
              AND read_at < NOW() - INTERVAL '90 days'
            "#,
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// 取得通知設定（若無則建立預設列，相容 migration 前建立的使用者）
    pub async fn get_settings(&self, user_id: Uuid) -> Result<NotificationSettings, AppError> {
        let settings: Option<NotificationSettings> = sqlx::query_as(
            r#"
            SELECT * FROM notification_settings WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match settings {
            Some(s) => Ok(s),
            None => {
                let created: NotificationSettings = sqlx::query_as(
                    r#"
                    INSERT INTO notification_settings (user_id)
                    VALUES ($1)
                    ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
                    RETURNING *
                    "#,
                )
                .bind(user_id)
                .fetch_one(&self.db)
                .await?;
                Ok(created)
            }
        }
    }

    /// 更新通知設定（若無列則先建立，相容 migration 前建立的使用者）
    pub async fn update_settings(
        &self,
        user_id: Uuid,
        request: UpdateNotificationSettingsRequest,
    ) -> Result<NotificationSettings, AppError> {
        let settings: Option<NotificationSettings> = sqlx::query_as(
            r#"
            UPDATE notification_settings
            SET 
                email_low_stock = COALESCE($2, email_low_stock),
                email_expiry_warning = COALESCE($3, email_expiry_warning),
                email_document_approval = COALESCE($4, email_document_approval),
                email_protocol_status = COALESCE($5, email_protocol_status),
                email_monthly_report = COALESCE($6, email_monthly_report),
                expiry_warning_days = COALESCE($7, expiry_warning_days),
                low_stock_notify_immediately = COALESCE($8, low_stock_notify_immediately),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(request.email_low_stock)
        .bind(request.email_expiry_warning)
        .bind(request.email_document_approval)
        .bind(request.email_protocol_status)
        .bind(request.email_monthly_report)
        .bind(request.expiry_warning_days)
        .bind(request.low_stock_notify_immediately)
        .fetch_optional(&self.db)
        .await?;

        match settings {
            Some(s) => Ok(s),
            None => {
                sqlx::query(
                    "INSERT INTO notification_settings (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
                )
                .bind(user_id)
                .execute(&self.db)
                .await?;
                // 插入後再執行一次 UPDATE（此時列已存在）
                let created: NotificationSettings = sqlx::query_as(
                    r#"
                    UPDATE notification_settings
                    SET 
                        email_low_stock = COALESCE($2, email_low_stock),
                        email_expiry_warning = COALESCE($3, email_expiry_warning),
                        email_document_approval = COALESCE($4, email_document_approval),
                        email_protocol_status = COALESCE($5, email_protocol_status),
                        email_monthly_report = COALESCE($6, email_monthly_report),
                        expiry_warning_days = COALESCE($7, expiry_warning_days),
                        low_stock_notify_immediately = COALESCE($8, low_stock_notify_immediately),
                        updated_at = NOW()
                    WHERE user_id = $1
                    RETURNING *
                    "#,
                )
                .bind(user_id)
                .bind(request.email_low_stock)
                .bind(request.email_expiry_warning)
                .bind(request.email_document_approval)
                .bind(request.email_protocol_status)
                .bind(request.email_monthly_report)
                .bind(request.expiry_warning_days)
                .bind(request.low_stock_notify_immediately)
                .fetch_one(&self.db)
                .await?;
                Ok(created)
            }
        }
    }
}
