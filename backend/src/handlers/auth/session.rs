use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use validator::Validate;

use crate::error::ErrorResponse;
use crate::{
    middleware::{extract_real_ip_with_trust, CurrentUser},
    models::{ConfirmPasswordRequest, LoginResponse, RefreshTokenRequest},
    services::{AuthService, SessionManager},
    AppError, AppState, Result,
};

use super::cookie::{extract_cookie_value, login_response_with_cookies};

/// 重新整理 Token
/// 支援從 JSON body 或 Cookie 讀取 refresh_token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token 更新成功", body = LoginResponse),
        (status = 401, description = "Refresh token 無效或過期", body = ErrorResponse),
    ),
    tag = "認證"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response> {
    // 優先從 JSON body 讀取，其次從 Cookie 讀取
    let refresh_token_value = body
        .map(|Json(req)| req.refresh_token)
        .or_else(|| extract_cookie_value(&headers, "refresh_token"))
        .ok_or_else(|| AppError::InvalidCredentials("Missing refresh token".to_string()))?;

    // R46-1/2: rotation metadata — 寫入 normal_rotation 後該 token 的 last_ip / UA，
    // 供未來 reuse detection 比對裝置同一性。
    let client_ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let response = AuthService::refresh_token(
        &state.db,
        &state.config,
        &refresh_token_value,
        &client_ip,
        user_agent.as_deref(),
    )
    .await?;
    login_response_with_cookies(&response, &state.config)
}

/// SEC-33：敏感操作二級認證 — 以密碼換取短期 reauth token
#[utoipa::path(
    post,
    path = "/api/v1/auth/confirm-password",
    request_body = ConfirmPasswordRequest,
    responses(
        (status = 200, description = "驗證成功，回傳 reauth_token 供後續敏感操作使用"),
        (status = 401, description = "密碼錯誤", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn confirm_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ConfirmPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate()?;
    AuthService::verify_password_by_id(&state.db, current_user.id, &req.password).await?;
    let (reauth_token, expires_in) =
        AuthService::generate_reauth_token(&state.config, current_user.id)?;
    Ok(Json(serde_json::json!({
        "reauth_token": reauth_token,
        "expires_in": expires_in
    })))
}

/// Heartbeat - 更新使用者 session 的最後活動時間與 IP
#[utoipa::path(
    post,
    path = "/api/v1/auth/heartbeat",
    responses(
        (status = 200, description = "心跳更新成功"),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn heartbeat(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);

    // F1 fix (2026-05-18): 原本錯誤地呼叫 update_activity(session_id) 但傳 user_id，
    // SQL `WHERE id = $1` 永遠 0 rows affected → sliding session 完全失效。
    // 改呼叫 update_activity_by_user，依 user_id 找最新 active session 更新。
    if let Err(e) =
        SessionManager::update_activity_by_user(&state.db, current_user.id, Some(&ip)).await
    {
        tracing::warn!(
            "Failed to update session activity for user {}: {}",
            current_user.id,
            e
        );
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
