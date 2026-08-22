// 輔助方法

use uuid::Uuid;

use crate::error::AppError;

use super::NotificationService;

impl NotificationService {
    /// 取得計畫的 PI 與計劃負責人（SD）使用者（站內通知對象）。
    ///
    /// CO_EDITOR 角色拆除後，內部協作者由計劃負責人（SD, `study_director_user_id`）後繼，
    /// 故通知對象＝PI 成員 ＋ SD（取代原「PI + CO_EDITOR」）。
    pub async fn get_protocol_pi_and_sd(
        &self,
        protocol_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, AppError> {
        let users: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT u.id, u.email, u.display_name
            FROM users u
            JOIN user_protocols up ON u.id = up.user_id
            WHERE up.protocol_id = $1
              AND up.role_in_protocol = 'PI'
              AND u.is_active = true
              AND u.deleted_at IS NULL
            UNION
            SELECT u.id, u.email, u.display_name
            FROM users u
            JOIN protocols p ON u.id = p.study_director_user_id
            WHERE p.id = $1
              AND u.is_active = true
              AND u.deleted_at IS NULL
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

    /// 該事件是否已在 notification_routing 啟用 email（任一 active 列 channel 為 email/both）。
    /// 供「非角色路由」的通知（如直接指派的審查委員）做可切換的 email 開關。
    /// 查詢失敗或無設定 → false（預設不寄，admin 在路由設定開啟後才寄）。
    pub async fn is_email_enabled_for_event(&self, event_type: &str) -> bool {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notification_routing
                WHERE event_type = $1 AND is_active = true AND channel IN ('email', 'both')
            )
            "#,
        )
        .bind(event_type)
        .fetch_one(&self.db)
        .await
        .unwrap_or(false)
    }

    /// 取得使用者的 email + display_name（僅 active 且未軟刪除）；不存在回 None。
    pub async fn find_active_user_contact(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(String, String)>, AppError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT email, display_name FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
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
