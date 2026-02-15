// 輔助方法

use uuid::Uuid;

use crate::error::AppError;

use super::NotificationService;

impl NotificationService {
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

    /// 依事件類型從 notification_routing 表動態查詢收件者
    /// 回傳 (user_id, email, display_name, channel)
    pub async fn get_recipients_by_event(
        &self,
        event_type: &str,
    ) -> Result<Vec<(Uuid, String, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name, nr.channel
            FROM notification_routing nr
            JOIN roles r ON nr.role_code = r.code
            JOIN user_roles ur ON ur.role_id = r.id
            JOIN users u ON u.id = ur.user_id
            WHERE nr.event_type = $1
              AND nr.is_active = true
              AND u.is_active = true
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// 檢查特定事件是否需要 Email 通知（根據 routing 表的 channel 設定）
    pub fn should_send_email(channel: &str) -> bool {
        channel == "email" || channel == "both"
    }

    /// 檢查特定事件是否需要站內通知
    pub fn should_send_in_app(channel: &str) -> bool {
        channel == "in_app" || channel == "both"
    }
}
