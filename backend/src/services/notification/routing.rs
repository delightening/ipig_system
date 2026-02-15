// 通知路由規則 CRUD 操作

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        NotificationRouting, CreateNotificationRoutingRequest,
        UpdateNotificationRoutingRequest,
    },
};

use super::NotificationService;

impl NotificationService {
    /// 列出所有通知路由規則
    pub async fn list_notification_routing(&self) -> Result<Vec<NotificationRouting>, AppError> {
        let rules: Vec<NotificationRouting> = sqlx::query_as(
            r#"
            SELECT id, event_type, role_code, channel, is_active, description, created_at, updated_at
            FROM notification_routing
            ORDER BY event_type, role_code
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rules)
    }

    /// 建立通知路由規則
    pub async fn create_notification_routing(
        &self,
        request: CreateNotificationRoutingRequest,
    ) -> Result<NotificationRouting, AppError> {
        let channel = request.channel.unwrap_or_else(|| "in_app".to_string());

        // 驗證 channel 值
        if !["in_app", "email", "both"].contains(&channel.as_str()) {
            return Err(AppError::BadRequest(
                "channel 必須是 'in_app'、'email' 或 'both'".to_string(),
            ));
        }

        // 驗證 role_code 存在
        let role_exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM roles WHERE code = $1",
        )
        .bind(&request.role_code)
        .fetch_optional(&self.db)
        .await?;

        if role_exists.is_none() {
            return Err(AppError::BadRequest(
                format!("角色代碼 '{}' 不存在", request.role_code),
            ));
        }

        let rule: NotificationRouting = sqlx::query_as(
            r#"
            INSERT INTO notification_routing (event_type, role_code, channel, description)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&request.event_type)
        .bind(&request.role_code)
        .bind(&channel)
        .bind(&request.description)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::BadRequest(
                    format!("事件 '{}' 與角色 '{}' 的路由規則已存在", request.event_type, request.role_code),
                )
            } else {
                AppError::from(e)
            }
        })?;

        tracing::info!(
            "[NotificationRouting] 新增路由規則: {} → {} ({})",
            request.event_type,
            request.role_code,
            channel
        );

        Ok(rule)
    }

    /// 更新通知路由規則
    pub async fn update_notification_routing(
        &self,
        id: Uuid,
        request: UpdateNotificationRoutingRequest,
    ) -> Result<NotificationRouting, AppError> {
        // 驗證 channel 值
        if let Some(ref channel) = request.channel {
            if !["in_app", "email", "both"].contains(&channel.as_str()) {
                return Err(AppError::BadRequest(
                    "channel 必須是 'in_app'、'email' 或 'both'".to_string(),
                ));
            }
        }

        let rule: NotificationRouting = sqlx::query_as(
            r#"
            UPDATE notification_routing
            SET channel = COALESCE($2, channel),
                is_active = COALESCE($3, is_active),
                description = COALESCE($4, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&request.channel)
        .bind(request.is_active)
        .bind(&request.description)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("通知路由規則不存在".to_string()))?;

        tracing::info!(
            "[NotificationRouting] 更新路由規則 {}: {} → {}",
            id,
            rule.event_type,
            rule.role_code
        );

        Ok(rule)
    }

    /// 刪除通知路由規則
    pub async fn delete_notification_routing(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM notification_routing WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("通知路由規則不存在".to_string()));
        }

        tracing::info!("[NotificationRouting] 刪除路由規則 {}", id);
        Ok(())
    }
}
