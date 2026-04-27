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
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{LoginRequest, LoginResponse, TwoFactorRequiredResponse, UpdateUserRequest, UserResponse},
    services::{AuthService, LoginTracker, SessionManager, UserService},
    AppError, AppState, Result,
};

use super::cookie::{build_clear_cookie, login_response_with_cookies};

/// 登入
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
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

    // Phase 1: 驗證帳密（CRIT-01: ip 傳入服務層，失敗事件在 advisory lock 事務內原子性寫入）
    let user = match AuthService::validate_credentials(&state.db, &state.config, &req, Some(&ip)).await {
        Ok(u) => u,
        Err(e) => {
            // R22-7: 火忘式觸發暴力破解偵測（GeoIP 異常偵測 + 計數）
            let db = state.db.clone();
            let config = state.config.clone();
            let email = req.email.clone();
            let ip_clone = ip.clone();
            let ua_clone = user_agent.clone();
            let geoip = state.geoip.clone();
            tokio::spawn(async move {
                if let Err(e) = LoginTracker::log_failure(
                    &db,
                    &config,
                    &email,
                    Some(&ip_clone),
                    ua_clone.as_deref(),
                    "invalid_credentials",
                    &geoip,
                )
                .await
                {
                    tracing::error!("Failed to log login failure for {}: {}", email, e);
                }
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
    //
    // H4 (併發審查): session 建立 + 併發上限檢查必須在 token 發出之**前**完成。
    // 原順序為先 issue_login_tokens（INSERT refresh_tokens 列）→ 再 create_session；
    // 若 create_session 失敗，refresh_tokens 留下孤兒列（client 雖未取得，但仍占
    // unique 索引、需 cleanup cron 才清）。改為 session 先建好再簽 token，
    // 失敗路徑不留 DB 副作用，並確保 SEC-28 併發 session 上限始終被執行。
    let user_id = user.id;
    SessionManager::create_session(
        &state.db,
        user_id,
        Some(&ip),
        user_agent.as_deref(),
    )
    .await?;
    SessionManager::end_excess_sessions(&state.db, user_id, state.config.max_sessions_per_user)
        .await?;

    let response = AuthService::issue_login_tokens(&state.db, &state.config, &user).await?;

    // Fire-and-forget: GeoIP 異常偵測屬遙測，非安全強制，允許非同步執行
    {
        let db = state.db.clone();
        let geoip = state.geoip.clone();
        let email = req.email.clone();
        let ip_clone = ip.clone();
        let ua_clone = user_agent.clone();
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
        });
    }

    // 回傳 JSON + Set-Cookie headers
    login_response_with_cookies(&response, &state.config)
}

/// 登出
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
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

    // 清除所有認證 Cookie（含 csrf_token，MED-01）
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
        .header(
            header::SET_COOKIE,
            build_clear_cookie("csrf_token", &state.config),
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
    path = "/api/v1/me",
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
    path = "/api/v1/me",
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
    // SEC-PRIV: 一般使用者不能修改自己的權限相關欄位，防止自我提權
    req.is_active = None;
    req.is_internal = None;
    req.role_ids = None;
    req.expires_at = None;
    let actor = ActorContext::User(current_user.clone());
    let user = UserService::update(&state.db, &actor, current_user.id, &req).await?;
    Ok(Json(user))
}
