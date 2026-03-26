use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use validator::Validate;

use crate::error::ErrorResponse;
use crate::{
    middleware::{extract_real_ip_with_trust, CurrentUser},
    models::{LoginRequest, LoginResponse, TwoFactorRequiredResponse, UpdateUserRequest, UserResponse},
    services::{AuthService, LoginTracker, SessionManager, UserService},
    AppError, AppState, Result,
};

use super::cookie::{build_clear_cookie, login_response_with_cookies};

/// 登入
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登入成功", body = LoginResponse),
        (status = 401, description = "帳號或密碼錯誤", body = ErrorResponse),
        (status = 423, description = "帳號已鎖定", body = ErrorResponse),
    ),
    tag = "認證"
)]
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    // 從 proxy header 提取真實客戶端 IP
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    req.validate()?;

    // Phase 1: 驗證帳密
    let user = match AuthService::validate_credentials(&state.db, &state.config, &req).await {
        Ok(u) => u,
        Err(e) => {
            let db = state.db.clone();
            let geoip = state.geoip.clone();
            let email = req.email.clone();
            let ip_clone = ip.clone();
            let ua_clone = user_agent.clone();
            let err_msg = e.to_string();
            tokio::spawn(async move {
                let _ = LoginTracker::log_failure(
                    &db, &email, Some(&ip_clone), ua_clone.as_deref(), &err_msg, &geoip,
                ).await;
            });
            return Err(e);
        }
    };

    // Phase 2: 2FA 檢查 — 若啟用，回傳 temp token 給前端進行第二步驗證
    if user.totp_enabled {
        let temp_token = AuthService::generate_2fa_temp_token(&state.config, user.id)?;
        let body = serde_json::to_string(&TwoFactorRequiredResponse {
            requires_2fa: true,
            temp_token,
        }).map_err(|e| AppError::Internal(format!("JSON 序列化失敗: {}", e)))?;
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.into())
            .map_err(|e| AppError::Internal(format!("Response 建構失敗: {e}")));
    }

    // Phase 3: 正常登入（無 2FA）
    let response = match AuthService::issue_login_tokens(&state.db, &state.config, &user).await {
        Ok(resp) => {
            // 登入成功：記錄事件和建立 session
            let db = state.db.clone();
            let geoip = state.geoip.clone();
            let user_id = resp.user.id;
            let email = req.email.clone();
            let ip_clone = ip.clone();
            let ua_clone = user_agent.clone();
            let max_sess = state.config.max_sessions_per_user;

            tokio::spawn(async move {
                if let Err(e) = LoginTracker::log_success(
                    &db,
                    user_id,
                    &email,
                    Some(&ip_clone),
                    ua_clone.as_deref(),
                    &geoip,
                )
                .await
                {
                    tracing::error!("Failed to log login success for {}: {}", email, e);
                }

                if let Err(e) = SessionManager::create_session(
                    &db,
                    user_id,
                    Some(&ip_clone),
                    ua_clone.as_deref(),
                )
                .await
                {
                    tracing::error!("Failed to create session for {}: {}", email, e);
                }

                // SEC-28: Session 併發限制，超過上限時自動結束最舊的 session
                if let Err(e) = SessionManager::end_excess_sessions(&db, user_id, max_sess).await {
                    tracing::warn!("[Session] 清理超額 session 失敗 for {}: {}", email, e);
                }
            });

            resp
        }
        Err(e) => {
            return Err(e);
        }
    };

    // 回傳 JSON + Set-Cookie headers
    login_response_with_cookies(&response, &state.config)
}

/// 登出
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "登出成功"),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response> {
    // SEC-23: 將當前 JWT 加入黑名單，使其立即失效
    if !current_user.jti.is_empty() {
        state
            .jwt_blacklist
            .revoke(current_user.jti.clone(), current_user.exp, &state.db)
            .await;
    }

    // 記錄登出事件
    if let Err(e) = LoginTracker::log_logout(
        &state.db,
        current_user.id,
        &current_user.email,
        Some(&extract_real_ip_with_trust(
            &headers,
            &addr,
            state.config.trust_proxy_headers,
        )),
    )
    .await
    {
        tracing::warn!("記錄登出事件失敗: {e}");
    }

    // 結束所有 sessions
    if let Err(e) = SessionManager::end_all_sessions(&state.db, current_user.id, "logout").await {
        tracing::warn!("結束所有 sessions 失敗: {e}");
    }

    AuthService::logout(&state.db, current_user.id).await?;

    // 清除所有認證 Cookie
    let body = serde_json::json!({ "message": "Logged out successfully" });
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::SET_COOKIE,
            build_clear_cookie("access_token", &state.config),
        )
        .header(
            header::SET_COOKIE,
            build_clear_cookie("refresh_token", &state.config),
        )
        .body(
            serde_json::to_string(&body)
                .map_err(|e| AppError::Internal(format!("JSON 序列化失敗: {}", e)))?
                .into(),
        )
        .map_err(|e| AppError::Internal(format!("Response 建構失敗: {e}")))?;

    Ok(response)
}

/// 取得當前使用者資訊
#[utoipa::path(
    get,
    path = "/api/me",
    responses(
        (status = 200, description = "目前使用者資訊", body = UserResponse),
        (status = 401, description = "未認證", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UserResponse>> {
    let user = UserService::get_by_id(&state.db, current_user.id).await?;
    Ok(Json(user))
}

/// 更新自己的資訊
#[utoipa::path(
    put,
    path = "/api/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "更新成功", body = UserResponse),
        (status = 400, description = "驗證錯誤", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(mut req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    // 一般使用者不能修改自己的 active/internal 狀態
    req.is_active = None;
    let user = UserService::update(&state.db, current_user.id, current_user.id, &req).await?;
    Ok(Json(user))
}
