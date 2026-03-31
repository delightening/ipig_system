// 效期通知範圍設定 Handler（Admin only）

use axum::{
    extract::State,
    Extension, Json,
};
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{ExpiryNotificationConfig, UpdateExpiryNotificationConfigRequest},
    services::NotificationService,
    AppState,
};

/// GET /api/admin/expiry-config
pub async fn get_expiry_config(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<ExpiryNotificationConfig>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可存取效期通知設定".into()));
    }

    let service = NotificationService::new(state.db.clone());
    let config = service.get_expiry_notification_config().await?;
    Ok(Json(config))
}

/// PUT /api/admin/expiry-config
pub async fn update_expiry_config(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<UpdateExpiryNotificationConfigRequest>,
) -> Result<Json<ExpiryNotificationConfig>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可修改效期通知設定".into()));
    }

    request.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let service = NotificationService::new(state.db.clone());
    let config = service.update_expiry_notification_config(request, current_user.id).await?;
    Ok(Json(config))
}
