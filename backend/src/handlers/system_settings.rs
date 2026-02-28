use axum::{
    extract::State,
    Extension, Json,
};
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    services::SystemSettingsService,
    AppState,
};

const SMTP_PASSWORD_MASK: &str = "********";

/// GET /api/admin/system-settings
pub async fn get_system_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<HashMap<String, Value>>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可存取系統設定".into()));
    }

    let service = SystemSettingsService::new(state.db.clone());
    let mut settings = service.get_all_settings().await?;

    if let Some(val) = settings.get("smtp_password") {
        if val.as_str().map_or(false, |s| !s.is_empty()) {
            settings.insert("smtp_password".into(), Value::String(SMTP_PASSWORD_MASK.into()));
        }
    }

    Ok(Json(settings))
}

/// PUT /api/admin/system-settings
pub async fn update_system_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(mut body): Json<HashMap<String, Value>>,
) -> Result<Json<HashMap<String, Value>>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可修改系統設定".into()));
    }

    if let Some(val) = body.get("smtp_password") {
        if val.as_str() == Some(SMTP_PASSWORD_MASK) {
            body.remove("smtp_password");
        }
    }

    if body.is_empty() {
        return Err(AppError::BadRequest("未提供任何設定值".into()));
    }

    let service = SystemSettingsService::new(state.db.clone());
    service.update_settings(&body, current_user.id).await?;

    let mut settings = service.get_all_settings().await?;
    if let Some(val) = settings.get("smtp_password") {
        if val.as_str().map_or(false, |s| !s.is_empty()) {
            settings.insert("smtp_password".into(), Value::String(SMTP_PASSWORD_MASK.into()));
        }
    }

    Ok(Json(settings))
}
