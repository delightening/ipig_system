// 效期通知範圍設定 CRUD

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{ExpiryNotificationConfig, UpdateExpiryNotificationConfigRequest},
};

use super::NotificationService;

impl NotificationService {
    /// 取得效期通知範圍設定（全系統單一列）
    pub async fn get_expiry_notification_config(
        &self,
    ) -> Result<ExpiryNotificationConfig, AppError> {
        let config: ExpiryNotificationConfig = sqlx::query_as(
            r#"
            SELECT id, warn_days, cutoff_days, monthly_threshold_days, updated_at, updated_by
            FROM expiry_notification_config
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::Internal("效期通知設定列不存在".to_string()))?;

        Ok(config)
    }

    /// 更新效期通知範圍設定
    pub async fn update_expiry_notification_config(
        &self,
        request: UpdateExpiryNotificationConfigRequest,
        updated_by: Uuid,
    ) -> Result<ExpiryNotificationConfig, AppError> {
        if let Some(threshold) = request.monthly_threshold_days {
            let cutoff = request.cutoff_days.unwrap_or(90);
            if threshold > cutoff as i16 {
                return Err(AppError::BadRequest(
                    "monthly_threshold_days 不能大於 cutoff_days".to_string(),
                ));
            }
        }

        let config: ExpiryNotificationConfig = sqlx::query_as(
            r#"
            UPDATE expiry_notification_config
            SET warn_days              = COALESCE($1, warn_days),
                cutoff_days            = COALESCE($2, cutoff_days),
                monthly_threshold_days = $3,
                updated_at             = NOW(),
                updated_by             = $4
            RETURNING id, warn_days, cutoff_days, monthly_threshold_days, updated_at, updated_by
            "#,
        )
        .bind(request.warn_days)
        .bind(request.cutoff_days)
        .bind(request.monthly_threshold_days)
        .bind(updated_by)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::Internal("效期通知設定列不存在".to_string()))?;

        tracing::info!(
            "[ExpiryConfig] 更新效期通知設定: warn={}d, cutoff={}d, monthly={:?}d",
            config.warn_days,
            config.cutoff_days,
            config.monthly_threshold_days
        );

        Ok(config)
    }
}
