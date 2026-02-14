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
}
