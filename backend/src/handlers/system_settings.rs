use axum::{
    extract::State,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    services::{email::EmailService, SystemSettingsService},
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
        if val.as_str().is_some_and(|s| !s.is_empty()) {
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
        if val.as_str().is_some_and(|s| !s.is_empty()) {
            settings.insert("smtp_password".into(), Value::String(SMTP_PASSWORD_MASK.into()));
        }
    }

    Ok(Json(settings))
}

#[derive(Deserialize)]
pub struct TestEmailRequest {
    pub to_email: String,
}

/// POST /api/admin/system-settings/test-email
pub async fn send_test_email(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<TestEmailRequest>,
) -> Result<Json<Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可發送測試信件".into()));
    }

    if body.to_email.is_empty() || !body.to_email.contains('@') {
        return Err(AppError::BadRequest("請提供有效的收件人 Email".into()));
    }

    let smtp = EmailService::resolve_smtp(&state.db, &state.config).await;
    if !smtp.is_email_enabled() {
        return Err(AppError::BadRequest(
            "SMTP 尚未設定，請先填寫 SMTP 伺服器資訊".into(),
        ));
    }

    EmailService::send_test_email(&smtp, &body.to_email)
        .await
        .map_err(|e| {
            tracing::error!("Test email failed: {e}");
            AppError::Internal(format!("發送測試信件失敗：{e}"))
        })?;

    tracing::info!(
        "Test email sent to {} by user {}",
        body.to_email,
        current_user.id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("測試信件已成功發送至 {}", body.to_email)
    })))
}
