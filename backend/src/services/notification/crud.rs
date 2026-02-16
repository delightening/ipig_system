// 通知 CRUD 操作 + 設定


use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, Notification, NotificationItem, NotificationQuery,
        NotificationSettings, PaginatedResponse,
        UpdateNotificationSettingsRequest,
    },
};

use super::NotificationService;

impl NotificationService {
    /// 取得使用者通知列表
    pub async fn list_notifications(
        &self,
        user_id: Uuid,
        query: &NotificationQuery,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<NotificationItem>, AppError> {
        let offset = (page - 1) * per_page;

        // 建立基本查詢
        let mut sql = String::from(
            r#"
            SELECT id, type::TEXT, title, content, is_read, read_at, 
                   related_entity_type, related_entity_id, created_at
            FROM notifications
            WHERE user_id = $1
            "#,
        );

        let mut count_sql = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE user_id = $1
            "#,
        );

        // 動態添加篩選條件
        if let Some(is_read) = query.is_read {
            let condition = format!(" AND is_read = {}", is_read);
            sql.push_str(&condition);
            count_sql.push_str(&condition);
        }

        if let Some(ref notification_type) = query.notification_type {
            let condition = format!(" AND type = '{}'", notification_type);
            sql.push_str(&condition);
            count_sql.push_str(&condition);
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT $2 OFFSET $3");

        let notifications: Vec<NotificationItem> = sqlx::query_as(&sql)
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.db)
            .await?;

        let total: (i64,) = sqlx::query_as(&count_sql)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

        Ok(PaginatedResponse::new(notifications, total.0, page, per_page))
    }

    /// 取得未讀通知數量
    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE user_id = $1 AND is_read = false
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(result.0)
    }

    /// 標記通知為已讀
    pub async fn mark_as_read(
        &self,
        user_id: Uuid,
        notification_ids: &[Uuid],
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1 AND id = ANY($2)
            "#,
        )
        .bind(user_id)
        .bind(notification_ids)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 標記所有通知為已讀
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1 AND is_read = false
            "#,
        )
        .bind(user_id)
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

    /// 建立通知
    pub async fn create_notification(
        &self,
        request: CreateNotificationRequest,
    ) -> Result<Notification, AppError> {
        let notification_type = request.notification_type.as_str();

        let notification: Notification = sqlx::query_as(
            r#"
            INSERT INTO notifications (id, user_id, type, title, content, 
                                       related_entity_type, related_entity_id)
            VALUES (gen_random_uuid(), $1, $2::notification_type, $3, $4, $5, $6)
            RETURNING id, user_id, type::TEXT, title, content, is_read, read_at,
                      related_entity_type, related_entity_id, created_at
            "#,
        )
        .bind(request.user_id)
        .bind(notification_type)
        .bind(&request.title)
        .bind(&request.content)
        .bind(&request.related_entity_type)
        .bind(request.related_entity_id)
        .fetch_one(&self.db)
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

    /// 取得通知設定
    pub async fn get_settings(&self, user_id: Uuid) -> Result<NotificationSettings, AppError> {
        let settings: NotificationSettings = sqlx::query_as(
            r#"
            SELECT * FROM notification_settings WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(settings)
    }

    /// 更新通知設定
    pub async fn update_settings(
        &self,
        user_id: Uuid,
        request: UpdateNotificationSettingsRequest,
    ) -> Result<NotificationSettings, AppError> {
        let settings: NotificationSettings = sqlx::query_as(
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

        Ok(settings)
    }
}
