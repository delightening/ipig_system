use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRequest, CreateScheduledReportRequest, ExpiryAlert, LowStockAlert, Notification, NotificationItem, NotificationQuery,
        NotificationSettings, NotificationType, PaginatedResponse, ReportHistory,
        ScheduledReport, UpdateNotificationSettingsRequest,
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
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ProtocolSubmitted,
                    title: notification_title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
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
    /// - 一般建議：僅站內通知 PI/Coeditor
    /// - 緊急建議：站內通知 + Email 給 PI/Coeditor
    pub async fn notify_vet_recommendation(
        &self,
        pig_id: Uuid,
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
                "No PI/Coeditor found for pig {} (protocol_id: {:?}), skipping notification",
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
            "獸醫師已對以下豬隻新增照護建議，請查閱並執行。\n\n耳號：{}\nIACUC NO.：{}\n紀錄類型：{}\n建議內容：{}",
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
                    related_entity_type: Some("pig".to_string()),
                    related_entity_id: Some(pig_id),
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
            "Vet recommendation notification sent to {} recipients for pig {} (urgent: {})",
            count,
            ear_tag,
            is_urgent
        );

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

    /// 緊急給藥通知（發送給 VET 和 PI）
    /// 當實驗工作人員在獸醫不在時緊急執行給藥，系統將發送紅色警報
    pub async fn notify_emergency_medication(
        &self,
        pig_id: Uuid,
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
            "緊急給藥執行通知\n\n此紀錄需要補簽審核。\n\n耳號：{}\nIACUC NO.：{}\n執行者：{}\n緊急原因：{}\n\n請儘速審核此緊急給藥紀錄。",
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
                    related_entity_type: Some("pig".to_string()),
                    related_entity_id: Some(pig_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::warn!(
            "[Emergency Medication] Alert sent to {} recipients for pig {} (observation {})",
            count,
            ear_tag,
            observation_id
        );

        Ok(count)
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

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::LowStock,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
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

            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id,
                    notification_type: NotificationType::ExpiryWarning,
                    title,
                    content: Some(content),
                    related_entity_type: None,
                    related_entity_id: None,
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        Ok(count)
    }

    // ============================================
    // 安樂死相關通知
    // ============================================

    /// 通知 PI 有新的安樂死單據
    pub async fn notify_euthanasia_order(
        &self,
        order_id: Uuid,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        reason: &str,
        pi_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = format!("[緊急] 豬隻 #{} 安樂死執行通知", ear_tag);
        let content = format!(
            "獸醫已開立安樂死單。\n\n耳號：{}\nIACUC NO.：{}\n原因：{}\n\n執行時間：系統將於 24 小時後自動解鎖執行權限。\n\n請登入系統選擇「同意執行」或「申請暫緩」。",
            ear_tag,
            iacuc_no.unwrap_or("-"),
            reason
        );

        self.create_notification(CreateNotificationRequest {
            user_id: pi_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        Ok(())
    }

    /// 通知獸醫：PI 已同意執行安樂死
    pub async fn notify_euthanasia_approved(
        &self,
        order_id: Uuid,
        vet_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = "[已核准] PI 已同意執行安樂死".to_string();
        let content = "PI 已同意執行安樂死。您現在可以進行安樂死操作。".to_string();

        self.create_notification(CreateNotificationRequest {
            user_id: vet_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        Ok(())
    }

    /// 通知 CHAIR：有新的安樂死暫緩申請需要仲裁
    pub async fn notify_euthanasia_appeal(
        &self,
        appeal_id: Uuid,
        order_id: Uuid,
        chair_user_id: Uuid,
        appeal_reason: &str,
    ) -> Result<(), AppError> {
        let title = "[仲裁請求] 安樂死暫緩申請".to_string();
        let content = format!(
            "PI 已申請暫緩安樂死執行，請進行仲裁。\n\n暫緩理由：{}\n\n請於 24 小時內做出裁決，否則系統將自動核准執行安樂死。",
            appeal_reason
        );

        self.create_notification(CreateNotificationRequest {
            user_id: chair_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_appeal".to_string()),
            related_entity_id: Some(appeal_id),
        })
        .await?;

        tracing::info!(
            "[Euthanasia Appeal] Notification sent to CHAIR for order {} appeal {}",
            order_id,
            appeal_id
        );

        Ok(())
    }

    /// 通知獸醫：因超時自動核准執行權限
    pub async fn notify_euthanasia_timeout_approved(
        &self,
        order_id: Uuid,
        vet_user_id: Uuid,
    ) -> Result<(), AppError> {
        let title = "[超時核准] 安樂死執行權限已解鎖".to_string();
        let content = "因 PI/CHAIR 超時未回應，系統已自動解鎖安樂死執行權限。您現在可以進行安樂死操作。".to_string();

        self.create_notification(CreateNotificationRequest {
            user_id: vet_user_id,
            notification_type: NotificationType::SystemAlert,
            title,
            content: Some(content),
            related_entity_type: Some("euthanasia_order".to_string()),
            related_entity_id: Some(order_id),
        })
        .await?;

        tracing::warn!(
            "[Euthanasia Timeout] Order {} auto-approved due to timeout",
            order_id
        );

        Ok(())
    }

    // ========== 輔助方法 ==========

    /// 取得計畫的 PI 和 Coeditor 使用者
    pub async fn get_protocol_pi_and_coeditors(
        &self,
        protocol_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_protocols up ON u.id = up.user_id
            WHERE up.protocol_id = $1
              AND up.role_in_protocol IN ('PI', 'CO_EDITOR')
              AND u.is_active = true
            "#,
        )
        .bind(protocol_id)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 取得被指派的審查委員
    pub async fn get_assigned_reviewers(
        &self,
        protocol_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN review_assignments ra ON u.id = ra.reviewer_id
            WHERE ra.protocol_id = $1
              AND u.is_active = true
            "#,
        )
        .bind(protocol_id)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 依角色代碼取得使用者
    pub async fn get_users_by_role(
        &self,
        role_code: &str,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.is_active = true AND r.code = $1
            "#,
        )
        .bind(role_code)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    // ========== HR 通知 ==========

    /// 通知請假申請已提交（給 ADMIN_STAFF + admin）
    pub async fn notify_leave_submitted(
        &self,
        leave_id: Uuid,
        applicant_name: &str,
        leave_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<i32, AppError> {
        // 取得 ADMIN_STAFF 角色使用者
        let mut recipients = self.get_users_by_role("ADMIN_STAFF").await?;
        // 也取得 admin 角色使用者
        let admins = self.get_users_by_role("SYSTEM_ADMIN").await?;
        recipients.extend(admins);
        // 去重
        recipients.sort_by_key(|(id, _, _)| *id);
        recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新請假申請 - {}", applicant_name);
        let content = format!(
            "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
            applicant_name, leave_type, start_date, end_date
        );

        let mut count = 0;
        for (user_id, _email, _name) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::LeaveApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(leave_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] 請假申請通知已發送給 {} 位審核者", count);
        Ok(count)
    }

    /// 通知加班申請已提交（給 ADMIN_STAFF + admin）
    pub async fn notify_overtime_submitted(
        &self,
        overtime_id: Uuid,
        applicant_name: &str,
        overtime_date: &str,
        hours: f64,
    ) -> Result<i32, AppError> {
        let mut recipients = self.get_users_by_role("ADMIN_STAFF").await?;
        let admins = self.get_users_by_role("SYSTEM_ADMIN").await?;
        recipients.extend(admins);
        recipients.sort_by_key(|(id, _, _)| *id);
        recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新加班申請 - {}", applicant_name);
        let content = format!(
            "有新的加班申請待審核。\n\n申請人：{}\n加班日期：{}\n加班時數：{} 小時",
            applicant_name, overtime_date, hours
        );

        let mut count = 0;
        for (user_id, _email, _name) in &recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::OvertimeApproval,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("overtime_record".to_string()),
                    related_entity_id: Some(overtime_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!("[Notification] 加班申請通知已發送給 {} 位審核者", count);
        Ok(count)
    }

    // ========== AUP 審查流程通知 ==========

    /// AUP 審查進度通知 — 依新狀態決定通知對象
    /// 同時處理需要 Email 的重要節點
    pub async fn notify_protocol_review_progress(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        protocol_title: &str,
        new_status: &str,
        operator_id: Uuid,
        reason: Option<&str>,
        config: Option<&crate::config::Config>,
    ) -> Result<i32, AppError> {
        let mut count = 0;
        let status_text = match new_status {
            "pre_review" => "行政預審中",
            "vet_review" => "獸醫審查中",
            "pre_review_revision_required" => "行政退回修正",
            "vet_revision_required" => "獸醫退回修正",
            "under_review" => "委員審查中",
            "revision_required" => "要求修正",
            "resubmitted" => "已重新提交",
            "approved" => "已核准",
            "approved_with_conditions" => "有條件核准",
            "rejected" => "已駁回",
            _ => new_status,
        };

        let notification_title = format!("[iPig] 計畫狀態更新 - {}", protocol_no);
        let content = format!(
            "計畫狀態已更新。\n\n計畫編號：{}\n計畫名稱：{}\n新狀態：{}\n{}",
            protocol_no,
            protocol_title,
            status_text,
            reason.map(|r| format!("說明：{}", r)).unwrap_or_default()
        );

        // 判斷需求修正或最終決定 → 發 Email
        let needs_email = matches!(
            new_status,
            "pre_review_revision_required"
                | "vet_revision_required"
                | "revision_required"
                | "approved"
                | "approved_with_conditions"
                | "rejected"
        );

        match new_status {
            // 進入獸醫審查 → 通知 VET
            "vet_review" => {
                let vets = self.get_users_by_role("VET").await?;
                for (user_id, _email, _name) in &vets {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 退回修正 → 通知 PI + Coeditor（+ Email）
            "pre_review_revision_required" | "vet_revision_required" | "revision_required" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, email, display_name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    // 寄 Email
                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg,
                                email,
                                display_name,
                                protocol_no,
                                protocol_title,
                                status_text,
                                &chrono::Utc::now().to_rfc3339(),
                                reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 進入委員審查 → 通知 IACUC_STAFF + REVIEWER
            "under_review" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                let reviewers = self.get_assigned_reviewers(protocol_id).await?;
                let mut all_recipients = staff;
                all_recipients.extend(reviewers);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, _email, _name) in &all_recipients {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 重新提交 → 通知 IACUC_STAFF + 原審查委員
            "resubmitted" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                let reviewers = self.get_assigned_reviewers(protocol_id).await?;
                let mut all_recipients = staff;
                all_recipients.extend(reviewers);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, _email, _name) in &all_recipients {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 核准/駁回 → 通知 PI + Coeditor + IACUC_CHAIR（非操作者）
            "approved" | "approved_with_conditions" | "rejected" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                let chairs = self.get_users_by_role("IACUC_CHAIR").await?;
                let mut all_recipients = pi_coeditors;
                all_recipients.extend(chairs);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, email, display_name) in &all_recipients {
                    // 排除操作者本人
                    if *user_id == operator_id {
                        continue;
                    }
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    // 寄 Email
                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg,
                                email,
                                display_name,
                                protocol_no,
                                protocol_title,
                                status_text,
                                &chrono::Utc::now().to_rfc3339(),
                                reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 其他狀態 → 通知 PI
            _ => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, _email, _name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("protocol".to_string()),
                            related_entity_id: Some(protocol_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
        }

        tracing::info!(
            "[Notification] 計畫 {} 狀態變更為 {}，通知已發送給 {} 人",
            protocol_no,
            new_status,
            count
        );
        Ok(count)
    }

    /// 審查意見通知 — 通知 PI + Coeditor + IACUC_STAFF
    pub async fn notify_review_comment_created(
        &self,
        protocol_id: Uuid,
        protocol_no: &str,
        commenter_name: &str,
        comment_excerpt: &str,
    ) -> Result<i32, AppError> {
        let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
        let staff = self.get_users_by_role("IACUC_STAFF").await?;
        let mut all_recipients = pi_coeditors;
        all_recipients.extend(staff);
        all_recipients.sort_by_key(|(id, _, _)| *id);
        all_recipients.dedup_by_key(|(id, _, _)| *id);

        let title = format!("[iPig] 新審查意見 - {}", protocol_no);
        let excerpt = if comment_excerpt.chars().count() > 100 {
            format!("{}...", comment_excerpt.chars().take(100).collect::<String>())
        } else {
            comment_excerpt.to_string()
        };
        let content = format!(
            "計畫 {} 收到新的審查意見。\n\n審查委員：{}\n意見摘要：{}",
            protocol_no, commenter_name, excerpt
        );

        let mut count = 0;
        for (user_id, _email, _name) in &all_recipients {
            if let Err(e) = self
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::ReviewComment,
                    title: title.clone(),
                    content: Some(content.clone()),
                    related_entity_type: Some("protocol".to_string()),
                    related_entity_id: Some(protocol_id),
                })
                .await {
                tracing::warn!("建立通知失敗: {e}");
            }
            count += 1;
        }

        tracing::info!(
            "[Notification] 審查意見通知已發送給 {} 人（計畫 {}）",
            count,
            protocol_no
        );
        Ok(count)
    }

    // ========== 修正案通知 ==========

    /// 修正案進度通知
    pub async fn notify_amendment_progress(
        &self,
        amendment_id: Uuid,
        protocol_id: Uuid,
        protocol_no: &str,
        amendment_title: &str,
        event: &str,
        operator_id: Uuid,
        reason: Option<&str>,
        config: Option<&crate::config::Config>,
    ) -> Result<i32, AppError> {
        let mut count = 0;
        let event_text = match event {
            "submitted" => "修正案已提交",
            "classified" | "under_review" => "修正案開始審查",
            "decision_recorded" => "審查委員已記錄決定",
            "revision_required" => "修正案需要修正",
            "approved" => "修正案已核准",
            "rejected" => "修正案已駁回",
            _ => event,
        };

        let notification_title = format!("[iPig] {} - {}", event_text, protocol_no);
        let content = format!(
            "{}。\n\n計畫編號：{}\n修正案：{}\n{}",
            event_text,
            protocol_no,
            amendment_title,
            reason.map(|r| format!("說明：{}", r)).unwrap_or_default()
        );

        let needs_email = matches!(
            event,
            "revision_required" | "approved" | "rejected"
        );

        match event {
            // 提交 → 通知 IACUC_STAFF
            "submitted" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                for (user_id, _email, _name) in &staff {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 開始審查 → 通知 REVIEWER
            "classified" | "under_review" => {
                // 取得修正案被指派的審查委員
                let reviewers: Vec<(Uuid, String, String)> = sqlx::query_as(
                    r#"
                    SELECT DISTINCT u.id, u.email, u.display_name
                    FROM users u
                    JOIN amendment_review_assignments ara ON u.id = ara.reviewer_id
                    WHERE ara.amendment_id = $1 AND u.is_active = true
                    "#,
                )
                .bind(amendment_id)
                .fetch_all(&self.db)
                .await?;

                for (user_id, _email, _name) in &reviewers {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ReviewAssignment,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 審查委員記錄決定 → 通知 IACUC_STAFF
            "decision_recorded" => {
                let staff = self.get_users_by_role("IACUC_STAFF").await?;
                for (user_id, _email, _name) in &staff {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;
                }
            }
            // 要求修正 → 通知 PI + Coeditor（+ Email）
            "revision_required" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                for (user_id, email, display_name) in &pi_coeditors {
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg, email, display_name, protocol_no,
                                amendment_title, event_text,
                                &chrono::Utc::now().to_rfc3339(), reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            // 核准/駁回 → 通知 PI + Coeditor + IACUC_CHAIR（非操作者）
            "approved" | "rejected" => {
                let pi_coeditors = self.get_protocol_pi_and_coeditors(protocol_id).await?;
                let chairs = self.get_users_by_role("IACUC_CHAIR").await?;
                let mut all_recipients = pi_coeditors;
                all_recipients.extend(chairs);
                all_recipients.sort_by_key(|(id, _, _)| *id);
                all_recipients.dedup_by_key(|(id, _, _)| *id);

                for (user_id, email, display_name) in &all_recipients {
                    if *user_id == operator_id {
                        continue;
                    }
                    if let Err(e) = self
                        .create_notification(CreateNotificationRequest {
                            user_id: *user_id,
                            notification_type: NotificationType::ProtocolStatus,
                            title: notification_title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("amendment".to_string()),
                            related_entity_id: Some(amendment_id),
                        })
                        .await {
                        tracing::warn!("建立通知失敗: {e}");
                    }
                    count += 1;

                    if needs_email {
                        if let Some(cfg) = config {
                            if let Err(e) = crate::services::EmailService::send_protocol_status_change_email(
                                cfg, email, display_name, protocol_no,
                                amendment_title, event_text,
                                &chrono::Utc::now().to_rfc3339(), reason,
                            )
                            .await {
                                tracing::warn!("發送計畫狀態變更郵件失敗: {e}");
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        tracing::info!(
            "[Notification] 修正案 {} 事件 {}，通知已發送給 {} 人",
            amendment_id,
            event,
            count
        );
        Ok(count)
    }

    // ========== ERP 採購單通知 ==========

    /// 通知採購單已提交（給 WAREHOUSE_MANAGER）
    pub async fn notify_document_submitted(
        &self,
        document_id: Uuid,
        document_no: &str,
        doc_type: &str,
        creator_name: &str,
    ) -> Result<i32, AppError> {
        let managers = self.get_users_by_role("WAREHOUSE_MANAGER").await?;

        let type_text = match doc_type {
            "PO" => "採購單",
            "PR" => "採購退貨",
            "SO" => "銷售單",
            "DO" => "銷售出庫",
            _ => doc_type,
        };
        let title = format!("[iPig] 新{}待審核 - {}", type_text, document_no);
        let content = format!(
            "有新的{}待審核。\n\n單據編號：{}\n建立者：{}",
            type_text, document_no, creator_name
        );

        let mut count = 0;
        for (user_id, _email, _name) in &managers {
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

    /// 通知採購單已審核/駁回（給建立者）
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
            "SO" => "銷售單",
            "DO" => "銷售出庫",
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
