use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;

use crate::error::ErrorResponse;
use crate::{
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{
        ChangeOwnPasswordRequest, ForgotPasswordRequest, ResetPasswordWithTokenRequest,
    },
    services::{AuthService, EmailService, UserService},
    AppState, Result,
};

use super::cookie::login_response_with_cookies;

/// 變更自己的密碼
#[utoipa::path(
    put,
    path = "/api/v1/me/password",
    request_body = ChangeOwnPasswordRequest,
    responses(
        (status = 200, description = "密碼變更成功"),
        (status = 400, description = "密碼不符合要求", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn change_own_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ChangeOwnPasswordRequest>,
) -> Result<Response> {
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    // Audit 已收進 service 層（PASSWORD_SELF_CHANGE，tx 內，含 IP/UA）
    let actor = ActorContext::User(current_user.clone());
    let response = AuthService::change_own_password(
        &state.db,
        &state.config,
        &actor,
        current_user.id,
        &req.current_password,
        &req.new_password,
        Some(&ip),
        user_agent,
    )
    .await?;

    // 回傳新 tokens 的 Set-Cookie headers，保持用戶登入狀態
    login_response_with_cookies(&response, &state.config)
}

/// 忘記密碼 - 發送重設連結
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "若信箱存在則已寄出重設連結"),
    ),
    tag = "認證"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    use std::time::Instant;
    let start = Instant::now();

    match AuthService::forgot_password(&state.db, &req.email).await? {
        Some((user_id, token)) => {
            // 非同步發送重設密碼郵件
            let config = state.config.clone();
            let email = req.email.clone();

            // 查詢使用者名稱
            let user = UserService::get_by_id(&state.db, user_id).await?;
            let display_name = user.display_name.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    EmailService::send_password_reset_email(&config, &email, &display_name, &token)
                        .await
                {
                    tracing::error!("Failed to send password reset email to {}: {}", email, e);
                }
            });
        }
        None => {
            tracing::info!(
                "Password reset requested for non-existent email: {}",
                req.email
            );
        }
    }

    // 固定延遲 200ms 防止 timing attack (存在/不存在的帳號回應時間一致)
    let elapsed = start.elapsed();
    const MIN_RESPONSE_TIME: std::time::Duration = std::time::Duration::from_millis(200);
    if elapsed < MIN_RESPONSE_TIME {
        tokio::time::sleep(MIN_RESPONSE_TIME - elapsed).await;
    }

    // 不管帳號存不存在都回覆相同訊息（防止帳號枚舉攻擊）
    Ok(Json(
        serde_json::json!({ "message": "If the email exists, a reset link has been sent" }),
    ))
}

/// 使用 token 重設密碼
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordWithTokenRequest,
    responses(
        (status = 200, description = "密碼重設成功"),
        (status = 400, description = "Token 無效或已過期", body = ErrorResponse),
    ),
    tag = "認證"
)]
pub async fn reset_password_with_token(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordWithTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    AuthService::reset_password_with_token(&state.db, &req.token, &req.new_password).await?;
    Ok(Json(
        serde_json::json!({ "message": "Password has been reset successfully" }),
    ))
}
