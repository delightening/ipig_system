use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, CreateScheduledReportRequest, ExpiryAlert, LowStockAlert,
        MarkNotificationsReadRequest, Notification, NotificationItem, NotificationQuery,
        NotificationSettings, NotificationType, PaginatedResponse, ReportHistory,
        ScheduledReport, UnreadNotificationCount, UpdateNotificationSettingsRequest,
        UpdateScheduledReportRequest,
    },
};

pub struct NotificationService {
    db: PgPool,
}

impl NotificationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

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
            SELECT id, type, title, content, is_read, read_at, 
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
            RETURNING *
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
            let _ = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ProtocolSubmitted,
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                })
                .await;
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

    /// 通知獸醫師建議
    pub async fn notify_vet_recommendation(
        &self,
        pig_id: i32,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        record_type: &str,
        recommendation_content: &str,
    ) -> Result<i32, AppError> {
        // 取得該計畫的 EXPERIMENT_STAFF
        let staff_users: Vec<(Uuid,)> = if let Some(iacuc) = iacuc_no {
            sqlx::query_as(
                r#"
                SELECT DISTINCT u.id
                FROM users u
                JOIN user_roles ur ON u.id = ur.user_id
                JOIN roles r ON ur.role_id = r.id
                WHERE u.is_active = true AND r.code = 'EXPERIMENT_STAFF'
                "#,
            )
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let notification_title = format!("[iPig] 獸醫師建議 - 耳號 {}", ear_tag);
        let content = format!(
            "獸醫師已對以下豬隻新增照護建議，請查閱並執行。\n\n耳號：{}\nIACUC NO.：{}\n紀錄類型：{}\n建議內容：{}",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            record_type,
            recommendation_content
        );

        let mut count = 0;
        for (user_id,) in staff_users {
            let _ = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::VetRecommendation,
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("pig".to_string()),
                    related_entity_id: None, // pig uses i32, not UUID
                })
                .await;
            count += 1;
        }

        Ok(count)
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

    // ============================================
    // 預警相關
    // ============================================

    /// 取得低庫存預警列表
    pub async fn list_low_stock_alerts(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<LowStockAlert>, AppError> {
        let offset = (page - 1) * per_page;

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

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM v_low_stock_alerts"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    /// 取得效期預警列表
    pub async fn list_expiry_alerts(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<ExpiryAlert>, AppError> {
        let offset = (page - 1) * per_page;

        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"
            SELECT * FROM v_expiry_alerts
            ORDER BY days_until_expiry, product_name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM v_expiry_alerts"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(alerts, total.0, page, per_page))
    }

    // ============================================
    // 定期報表相關
    // ============================================

    /// 取得定期報表列表
    pub async fn list_scheduled_reports(&self) -> Result<Vec<ScheduledReport>, AppError> {
        let reports: Vec<ScheduledReport> = sqlx::query_as(
            r#"
            SELECT * FROM scheduled_reports
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// 取得單一定期報表
    pub async fn get_scheduled_report(&self, id: Uuid) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"SELECT * FROM scheduled_reports WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Scheduled report not found".to_string()))?;

        Ok(report)
    }

    /// 建立定期報表
    pub async fn create_scheduled_report(
        &self,
        request: CreateScheduledReportRequest,
        created_by: Uuid,
    ) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"
            INSERT INTO scheduled_reports 
                (id, report_type, schedule_type, day_of_week, day_of_month, 
                 hour_of_day, parameters, recipients, created_by)
            VALUES 
                (gen_random_uuid(), $1::report_type, $2::schedule_type, $3, $4, 
                 $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&request.report_type)
        .bind(&request.schedule_type)
        .bind(request.day_of_week)
        .bind(request.day_of_month)
        .bind(request.hour_of_day)
        .bind(&request.parameters)
        .bind(&request.recipients)
        .bind(created_by)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// 更新定期報表
    pub async fn update_scheduled_report(
        &self,
        id: Uuid,
        request: UpdateScheduledReportRequest,
    ) -> Result<ScheduledReport, AppError> {
        let report: ScheduledReport = sqlx::query_as(
            r#"
            UPDATE scheduled_reports
            SET 
                day_of_week = COALESCE($2, day_of_week),
                day_of_month = COALESCE($3, day_of_month),
                hour_of_day = COALESCE($4, hour_of_day),
                parameters = COALESCE($5, parameters),
                recipients = COALESCE($6, recipients),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(request.day_of_week)
        .bind(request.day_of_month)
        .bind(request.hour_of_day)
        .bind(&request.parameters)
        .bind(&request.recipients)
        .bind(request.is_active)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// 刪除定期報表
    pub async fn delete_scheduled_report(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"DELETE FROM scheduled_reports WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Scheduled report not found".to_string()));
        }

        Ok(())
    }

    /// 取得報表歷史記錄
    pub async fn list_report_history(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<PaginatedResponse<ReportHistory>, AppError> {
        let offset = (page - 1) * per_page;

        let reports: Vec<ReportHistory> = sqlx::query_as(
            r#"
            SELECT * FROM report_history
            ORDER BY generated_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM report_history"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(PaginatedResponse::new(reports, total.0, page, per_page))
    }

    /// 取得單一報表歷史
    pub async fn get_report_history(&self, id: Uuid) -> Result<ReportHistory, AppError> {
        let report: ReportHistory = sqlx::query_as(
            r#"SELECT * FROM report_history WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Report history not found".to_string()))?;

        Ok(report)
    }

    /// 發送低庫存通知（批次作業用）
    pub async fn send_low_stock_notifications(&self) -> Result<i32, AppError> {
        // 取得需要通知的使用者及其設定
        let users_with_settings: Vec<(Uuid, bool)> = sqlx::query_as(
            r#"
            SELECT u.id, ns.email_low_stock
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_low_stock = true
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // 取得低庫存項目
        let alerts: Vec<LowStockAlert> = sqlx::query_as(
            r#"SELECT * FROM v_low_stock_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (user_id, _) in users_with_settings {
            // 建立通知
            let title = format!("低庫存預警：{} 項產品需要補貨", alerts.len());
            let content = alerts
                .iter()
                .take(5)
                .map(|a| format!("- {} ({}) 庫存: {}", a.product_name, a.product_sku, a.qty_on_hand))
                .collect::<Vec<_>>()
                .join("\n");

            let _ = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::LowStock,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await;
            count += 1;
        }

        Ok(count)
    }

    /// 發送效期預警通知（批次作業用）
    pub async fn send_expiry_notifications(&self) -> Result<i32, AppError> {
        // 取得需要通知的使用者及其設定
        let users_with_settings: Vec<(Uuid, i32)> = sqlx::query_as(
            r#"
            SELECT u.id, ns.expiry_warning_days
            FROM users u
            JOIN notification_settings ns ON u.id = ns.user_id
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true
              AND r.code IN ('SYSTEM_ADMIN', 'WAREHOUSE_MANAGER')
              AND ns.email_expiry_warning = true
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // 取得效期預警項目
        let alerts: Vec<ExpiryAlert> = sqlx::query_as(
            r#"SELECT * FROM v_expiry_alerts"#,
        )
        .fetch_all(&self.db)
        .await?;

        if alerts.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (user_id, _) in users_with_settings {
            // 建立通知
            let expired_count = alerts.iter().filter(|a| a.expiry_status == "expired").count();
            let expiring_count = alerts.iter().filter(|a| a.expiry_status == "expiring_soon").count();

            let title = format!(
                "效期預警：{} 項已過期，{} 項即將到期",
                expired_count, expiring_count
            );
            let content = alerts
                .iter()
                .take(5)
                .map(|a| {
                    format!(
                        "- {} ({}) 批號:{} 效期:{} ({}天)",
                        a.product_name, a.sku, 
                        a.batch_no.as_deref().unwrap_or("-"),
                        a.expiry_date,
                        a.days_until_expiry
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let _ = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ExpiryWarning,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await;
            count += 1;
        }

        Ok(count)
    }
}
