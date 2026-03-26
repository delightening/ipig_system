use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::error::ErrorResponse;
use crate::{
    middleware::CurrentUser,
    models::UserResponse,
    services::{AuditService, AuthService, SessionManager, UserService},
    AppError, AppState, Result,
};

use super::cookie::build_clear_cookie;

/// GDPR 匯出：單一偏好項目
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PreferenceExport {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: Option<DateTime<Utc>>,
}

/// GDPR 匯出：個人資料完整回應
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GdprExportResponse {
    pub exported_at: DateTime<Utc>,
    pub user: UserResponse,
    pub preferences: Vec<PreferenceExport>,
    pub notification_settings: Option<crate::models::NotificationSettings>,
}

/// GDPR：匯出個人資料（存取權、可攜權）
#[utoipa::path(
    get,
    path = "/api/me/export",
    responses(
        (status = 200, description = "個人資料 JSON", body = GdprExportResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn export_me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<GdprExportResponse>> {
    let user = UserService::get_by_id(&state.db, current_user.id).await?;

    let preferences = crate::repositories::user_preference::list_preferences_by_user(
        &state.db, current_user.id,
    ).await?;

    let preferences_export: Vec<PreferenceExport> = preferences
        .into_iter()
        .map(|p| PreferenceExport {
            key: p.preference_key,
            value: p.preference_value,
            updated_at: p.updated_at,
        })
        .collect();

    let notification_settings = crate::repositories::notification::find_notification_settings_by_user(
        &state.db, current_user.id,
    ).await?;

    Ok(Json(GdprExportResponse {
        exported_at: chrono::Utc::now(),
        user,
        preferences: preferences_export,
        notification_settings,
    }))
}

/// GDPR：刪除帳號請求（刪除權）- 軟刪除，需 X-Reauth-Token
#[utoipa::path(
    delete,
    path = "/api/me/account",
    responses(
        (status = 200, description = "帳號已停用"),
        (status = 401, description = "未認證", body = ErrorResponse),
        (status = 403, description = "需帶 X-Reauth-Token 重新確認密碼", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn delete_me_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
) -> Result<Response> {
    crate::handlers::user::require_reauth_token(&headers, &state, &current_user)?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "SECURITY",
        "GDPR_ACCOUNT_DELETE",
        Some("user"),
        Some(current_user.id),
        None,
        None,
        None,
        None,
        None,
    )
    .await {
        tracing::error!("寫入審計日誌失敗 (GDPR_ACCOUNT_DELETE): {}", e);
    }

    UserService::deactivate_self(&state.db, current_user.id).await?;

    // 將當前 JWT 加入黑名單
    if !current_user.jti.is_empty() {
        state
            .jwt_blacklist
            .revoke(current_user.jti.clone(), current_user.exp, &state.db)
            .await;
    }

    // 結束所有 sessions、清除 refresh tokens
    let _ = SessionManager::end_all_sessions(&state.db, current_user.id, "gdpr_account_delete").await;
    let _ = AuthService::logout(&state.db, current_user.id).await;

    let body = serde_json::json!({
        "message": "帳號已停用。您已登出，如需恢復請聯絡管理員。"
    });
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, build_clear_cookie("access_token", &state.config))
        .header(header::SET_COOKIE, build_clear_cookie("refresh_token", &state.config))
        .body(
            serde_json::to_string(&body)
                .map_err(|e| AppError::Internal(format!("JSON error: {}", e)))?
                .into(),
        )
        .map_err(|e| AppError::Internal(format!("Response build error: {e}")))?;
    Ok(response)
}
