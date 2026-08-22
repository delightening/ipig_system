// 預警通知（低庫存 + 效期）

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, ExpiryAlert, LowStockAlert, NotificationType, PaginatedResponse,
    },
};

use super::NotificationService;

impl NotificationService {
    /// 庫存類 email 收件人：來源同站內通知（notification_routing 角色型規則），
    /// 且個人偏好（notification_settings）開啟。收斂 scheduler 原本寫死角色的分歧。
    /// `event_type`：'low_stock_alert' | 'expiry_alert'（同時決定對應個人偏好欄位）。
    pub async fn stock_email_recipients(
        &self,
        event_type: &str,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let rows: Vec<(Uuid, String, String)> = match event_type {
            "low_stock_alert" => {
                sqlx::query_as(
                    r#"
                    SELECT DISTINCT u.id, u.email, u.display_name
                    FROM notification_routing nr
                    JOIN roles r ON nr.role_code = r.code
                    JOIN user_roles ur ON ur.role_id = r.id
                    JOIN users u ON u.id = ur.user_id
                    JOIN notification_settings ns ON ns.user_id = u.id
                    WHERE nr.event_type = $1
                      AND nr.is_active = true
                      AND nr.target_kind = 'role'
                      AND u.is_active = true
                      AND u.deleted_at IS NULL
                      AND ns.email_low_stock = true
                    "#,
                )
                .bind(event_type)
                .fetch_all(&self.db)
                .await?
            }
            "expiry_alert" => {
                sqlx::query_as(
                    r#"
                    SELECT DISTINCT u.id, u.email, u.display_name
                    FROM notification_routing nr
                    JOIN roles r ON nr.role_code = r.code
                    JOIN user_roles ur ON ur.role_id = r.id
                    JOIN users u ON u.id = ur.user_id
                    JOIN notification_settings ns ON ns.user_id = u.id
                    WHERE nr.event_type = $1
                      AND nr.is_active = true
                      AND nr.target_kind = 'role'
                      AND u.is_active = true
                      AND u.deleted_at IS NULL
                      AND ns.email_expiry_warning = true
                    "#,
                )
                .bind(event_type)
                .fetch_all(&self.db)
                .await?
            }
            _ => Vec::new(),
        };
        Ok(rows)
    }

    /// 取得低庫存預警列表
    pub async fn list_low_stock_alerts(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<LowStockAlert>, AppError> {
        let page = page.max(1);
        let per_page = per_page.clamp(1, crate::constants::MAX_PAGE_SIZE);
        let offset = (page - 1).saturating_mul(per_page);

        let alerts: Vec<LowStockAlert> = sqlx::query_as(
            r#"
            SELECT * FROM v_low_stock_alerts
            ORDER BY stock_status, product_name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM v_low_stock_alerts"#)
            .fetch_one(&self.db)
            .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    /// 取得效期預警列表
    ///
    /// R35-17: 加 `within_days` 可選參數 — 只回 `days_until_expiry <= within_days`
    /// 的品項（含已過期負值）。Dashboard widget 用 `within_days=7` 取「7 天內到期」摘要。
    /// `None` 時行為與舊版一致（回所有 alerts）。
    pub async fn list_expiry_alerts(
        &self,
        page: i64,
        per_page: i64,
        within_days: Option<i32>,
    ) -> Result<PaginatedResponse<ExpiryAlert>, AppError> {
        let page = page.max(1);
        let per_page = per_page.clamp(1, crate::constants::MAX_PAGE_SIZE);
        let offset = (page - 1).saturating_mul(per_page);

        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"
            SELECT * FROM v_expiry_alerts
            WHERE $1::INT IS NULL OR days_until_expiry <= $1
            ORDER BY days_until_expiry, product_name
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(within_days)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM v_expiry_alerts
            WHERE $1::INT IS NULL OR days_until_expiry <= $1
            "#,
        )
        .bind(within_days)
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    /// 發送低庫存通知（批次作業用）— 依 notification_routing 表判斷收件角色
    /// 同一使用者當天已有相同類型的未讀通知時，跳過不重複建立
    pub async fn send_low_stock_notifications(&self) -> Result<i32, AppError> {
        // 取得低庫存項目
        let alerts: Vec<LowStockAlert> = sqlx::query_as(r#"SELECT * FROM v_low_stock_alerts"#)
            .fetch_all(&self.db)
            .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("low_stock_alert").await?;

        // R82-9：批次化 — 先收斂唯一收件者，再兩次批次查詢取代 per-user 2N 次探測。
        let mut seen = std::collections::HashSet::new();
        let user_ids: Vec<uuid::Uuid> = recipients
            .iter()
            .filter_map(|(user_id, ..)| seen.insert(*user_id).then_some(*user_id))
            .collect();

        // 無任何收件者 → 提前返回，省下後續兩次批次查詢的 DB roundtrip
        if user_ids.is_empty() {
            return Ok(0);
        }

        // 個人設定明確關閉 email_low_stock 者（無設定列＝預設開啟，不入此集合）
        let disabled: std::collections::HashSet<uuid::Uuid> = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT user_id FROM notification_settings \
             WHERE user_id = ANY($1) AND email_low_stock = false",
        )
        .bind(&user_ids)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        // 當天已有 low_stock 通知者（一次撈完）
        let notified_today: std::collections::HashSet<uuid::Uuid> =
            sqlx::query_as::<_, (uuid::Uuid,)>(
                "SELECT DISTINCT user_id FROM notifications \
                 WHERE user_id = ANY($1) AND related_entity_type = 'low_stock' \
                   AND created_at >= CURRENT_DATE",
            )
            .bind(&user_ids)
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|(id,)| id)
            .collect();

        // title / content 對所有收件者相同，迴圈外算一次
        let title = format!("低庫存預警：{} 項產品需要補貨", alerts.len());
        let content = alerts
            .iter()
            .take(5)
            .map(|a| {
                // qty_on_hand 為 Decimal，直接 Display 會帶尾數零（46.0000）；
                // normalize() 去除尾零，整數顯示為整數、真正的分數仍保留。
                format!(
                    "- {} ({}) 庫存: {}",
                    a.product_name,
                    a.product_sku,
                    a.qty_on_hand.normalize()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut count = 0;
        for user_id in &user_ids {
            // 個人設定關閉 或 當天已通知 → 跳過
            if disabled.contains(user_id) || notified_today.contains(user_id) {
                continue;
            }

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::LowStock,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("low_stock".to_string()),
                    related_entity_id: None,
                })
                .await
            {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    /// 發送效期預警通知（批次作業用）— 依 notification_routing 表判斷收件角色
    /// 同一使用者當天已有相同類型的未讀通知時，跳過不重複建立
    ///
    /// `alerts`：由呼叫端（scheduler `check_expiry`）依 admin 設定
    /// `fn_expiry_alerts(warn_days, cutoff_days)` 算好的效期清單。in-app 通知與 email
    /// 共用同一 config-aware 來源，確保管理員調整的預警視窗對兩個通道皆生效（R82-11）。
    pub async fn send_expiry_notifications(&self, alerts: &[ExpiryAlert]) -> Result<i32, AppError> {
        if alerts.is_empty() {
            return Ok(0);
        }

        // 從路由表動態取得收件者
        let recipients = self.get_recipients_by_event("expiry_alert").await?;

        let expired_count = alerts
            .iter()
            .filter(|a| a.expiry_status == "expired")
            .count();
        let expiring_count = alerts
            .iter()
            .filter(|a| a.expiry_status == "expiring_soon")
            .count();

        // R82-9：批次化 — 先收斂唯一收件者，再兩次批次查詢取代 per-user 2N 次探測。
        let mut seen = std::collections::HashSet::new();
        let user_ids: Vec<uuid::Uuid> = recipients
            .iter()
            .filter_map(|(user_id, ..)| seen.insert(*user_id).then_some(*user_id))
            .collect();

        // 無任何收件者 → 提前返回，省下後續兩次批次查詢的 DB roundtrip
        if user_ids.is_empty() {
            return Ok(0);
        }

        // 個人設定明確關閉 email_expiry_warning 者（無設定列＝預設開啟，不入此集合）
        let disabled: std::collections::HashSet<uuid::Uuid> = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT user_id FROM notification_settings \
             WHERE user_id = ANY($1) AND email_expiry_warning = false",
        )
        .bind(&user_ids)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        // 當天已有 expiry_warning 通知者（一次撈完）
        let notified_today: std::collections::HashSet<uuid::Uuid> =
            sqlx::query_as::<_, (uuid::Uuid,)>(
                "SELECT DISTINCT user_id FROM notifications \
                 WHERE user_id = ANY($1) AND related_entity_type = 'expiry_warning' \
                   AND created_at >= CURRENT_DATE",
            )
            .bind(&user_ids)
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|(id,)| id)
            .collect();

        // title / content 對所有收件者相同，迴圈外算一次
        let title = format!(
            "效期預警：{} 項已過期，{} 項即將到期",
            expired_count, expiring_count
        );
        let content = alerts
            .iter()
            .take(5)
            .map(|a| {
                format!(
                    "- {} ({}) 批號:{} 效期:{} ({}天) 近效期量:{} / 總量:{}",
                    a.product_name,
                    a.sku,
                    a.batch_no.as_deref().unwrap_or("-"),
                    a.expiry_date,
                    a.days_until_expiry,
                    a.on_hand_qty,
                    a.total_qty,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut count = 0;
        for user_id in &user_ids {
            // 個人設定關閉 或 當天已通知 → 跳過
            if disabled.contains(user_id) || notified_today.contains(user_id) {
                continue;
            }

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::ExpiryWarning,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("expiry_warning".to_string()),
                    related_entity_id: None,
                })
                .await
            {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }
}
