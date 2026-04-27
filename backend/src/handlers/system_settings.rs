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
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    service.update_settings(&actor, &body).await?;

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

#[derive(Debug, Deserialize)]
pub struct IacucTestNotifyRequest {
    /// 逗號分隔收件人，留空則讀 system_settings.iacuc_notify_emails
    pub to_emails: Option<String>,
}

/// POST /api/admin/iacuc/test-notification
/// 手動觸發 IACUC 新案通知信（測試用，繞過時間限制）
pub async fn send_iacuc_test_notification(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<IacucTestNotifyRequest>,
) -> Result<Json<Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可執行測試通知".into()));
    }

    // 決定收件人：優先用 request body，否則讀 system_settings
    let notify_raw = match body.to_emails.filter(|s| !s.is_empty()) {
        Some(v) => v,
        None => {
            sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT value FROM system_settings WHERE key = 'iacuc_notify_emails'",
            )
            .fetch_optional(&state.db)
            .await?
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
        }
    };

    let recipients: Vec<&str> = notify_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if recipients.is_empty() {
        return Err(AppError::BadRequest(
            "未指定收件人，且系統通知信箱（iacuc_notify_emails）尚未設定".into(),
        ));
    }

    // 查詢最近送審案件（無時間限制，最多取 5 筆供預覽）
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT p.protocol_no, p.title, u.name
        FROM protocols p
        LEFT JOIN users u ON u.id = p.pi_user_id
        WHERE p.status = 'SUBMITTED'
        ORDER BY p.updated_at DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let case_list_html: String = if rows.is_empty() {
        "<li>（目前無送審案件，此為測試信件）</li>".to_string()
    } else {
        rows.iter()
            .map(|(no, title, pi)| {
                format!(
                    "<li><strong>{no}</strong> — {title}（申請人：{}）</li>",
                    pi.as_deref().unwrap_or("—")
                )
            })
            .collect()
    };

    let subject = "【iPig IACUC】測試通知信".to_string();
    let body_html = format!(
        r#"<html><body style="font-family:Microsoft JhengHei,sans-serif;max-width:600px;margin:0 auto">
<h2 style="color:#1e40af">IACUC 新送審案件通知（測試）</h2>
<p style="color:#dc2626;font-weight:bold">⚠ 此為測試信件，由管理員手動觸發</p>
<p>以下為系統中最近的送審案件範例：</p>
<ul style="line-height:2">{case_list_html}</ul>
<p style="margin-top:24px">
  <a href="https://ipigsystem.asia" style="background:#2563eb;color:#fff;padding:8px 16px;border-radius:6px;text-decoration:none">
    前往 iPig 系統
  </a>
</p>
<hr style="margin-top:32px"/>
<p style="color:#94a3b8;font-size:12px">此信由 iPig 系統管理員觸發發送</p>
</body></html>"#
    );
    let body_plain = "【iPig IACUC】測試通知信\n\n此為測試信件，請忽略。\n\n前往 https://ipigsystem.asia".to_string();

    let smtp = EmailService::resolve_smtp(&state.db, &state.config).await;
    if !smtp.is_email_enabled() {
        return Err(AppError::BadRequest("SMTP 尚未設定，請先填寫 SMTP 伺服器資訊".into()));
    }

    let mut sent = vec![];
    let mut failed = vec![];
    for addr in &recipients {
        match EmailService::send_email_smtp(
            &smtp, addr, "IACUC 執行秘書", &subject, &body_plain, &body_html,
        )
        .await
        {
            Ok(_) => sent.push(*addr),
            Err(e) => {
                tracing::error!("IACUC test notify to {} failed: {e}", addr);
                failed.push(*addr);
            }
        }
    }

    tracing::info!(
        "IACUC test notification by {}: sent={:?} failed={:?}",
        current_user.id, sent, failed
    );

    Ok(Json(serde_json::json!({
        "success": failed.is_empty(),
        "sent": sent,
        "failed": failed,
    })))
}
