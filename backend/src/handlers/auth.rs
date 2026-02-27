use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    config::Config,
    middleware::{extract_real_ip_with_trust, CurrentUser},
    models::{
        ChangeOwnPasswordRequest, ConfirmPasswordRequest, ForgotPasswordRequest, LoginRequest,
        LoginResponse, RefreshTokenRequest, ResetPasswordWithTokenRequest, UpdateUserRequest,
        UserResponse,
    },
    services::{
        AuditService, AuthService, EmailService, LoginTracker, SessionManager, UserService,
    },
    AppError, AppState, Result,
};

// ============================================
// Cookie 輔助函式
// ============================================

/// 建構認證 Cookie 的 Set-Cookie header 值
pub(crate) fn build_set_cookie(
    name: &str,
    value: &str,
    max_age_secs: i64,
    config: &Config,
) -> String {
    let mut cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        name, value, max_age_secs
    );
    if config.cookie_secure {
        cookie.push_str("; Secure");
    }
    if let Some(ref domain) = config.cookie_domain {
        cookie.push_str(&format!("; Domain={}", domain));
    }
    cookie
}

/// 建構清除 Cookie 的 Set-Cookie header 值（Max-Age=0）
pub(crate) fn build_clear_cookie(name: &str, config: &Config) -> String {
    build_set_cookie(name, "", 0, config)
}

/// 從請求的 Cookie header 中提取指定名稱的 cookie 值
fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(&format!("{}=", name)))
        .map(|s| s[name.len() + 1..].to_string())
}

/// 將 LoginResponse 附加 Set-Cookie headers 回傳
fn login_response_with_cookies(response: &LoginResponse, config: &Config) -> Response {
    let access_cookie = build_set_cookie(
        "access_token",
        &response.access_token,
        response.expires_in,
        config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token",
        &response.refresh_token,
        7 * 24 * 3600, // 7 天
        config,
    );

    let body = serde_json::to_string(response).expect("LoginResponse 序列化不應失敗");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, access_cookie)
        .header(header::SET_COOKIE, refresh_cookie)
        .body(body.into())
        .expect("建構 HTTP Response 不應失敗")
}

// ============================================
// Auth Handlers
// ============================================

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

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 嘗試登入
    let response = match AuthService::login(&state.db, &state.config, &req).await {
        Ok(resp) => {
            // 登入成功：記錄事件和建立 session
            let db = state.db.clone();
            let geoip = state.geoip.clone();
            let broadcaster = state.alert_broadcaster.clone();
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
                    &broadcaster,
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
                let max_sessions = max_sess;
                match SessionManager::get_active_session_count(&db, user_id).await {
                    Ok(count) if count > max_sessions => {
                        let excess = count - max_sessions;
                        tracing::info!(
                            "[Session] 使用者 {} 有 {} 個活躍 session，超過上限 {}，將結束 {} 個最舊的 session",
                            email, count, max_sessions, excess
                        );
                        // 結束最舊的多餘 session
                        let _ = sqlx::query(
                            r#"
                            UPDATE user_sessions
                            SET is_active = false,
                                ended_at = NOW(),
                                ended_reason = 'session_limit'
                            WHERE id IN (
                                SELECT id FROM user_sessions
                                WHERE user_id = $1 AND is_active = true
                                ORDER BY started_at ASC
                                LIMIT $2
                            )
                            "#,
                        )
                        .bind(user_id)
                        .bind(excess)
                        .execute(&db)
                        .await;
                    }
                    _ => {}
                }
            });

            resp
        }
        Err(e) => {
            // 登入失敗：記錄失敗事件（會自動檢測暴力破解）
            let db = state.db.clone();
            let geoip = state.geoip.clone();
            let broadcaster = state.alert_broadcaster.clone();
            let email = req.email.clone();
            let ip_clone = ip.clone();
            let ua_clone = user_agent.clone();
            let err_msg = e.to_string();

            tokio::spawn(async move {
                if let Err(log_err) = LoginTracker::log_failure(
                    &db,
                    &email,
                    Some(&ip_clone),
                    ua_clone.as_deref(),
                    &err_msg,
                    &geoip,
                    &broadcaster,
                )
                .await
                {
                    tracing::error!("Failed to log login failure for {}: {}", email, log_err);
                }
            });

            return Err(e);
        }
    };

    // 回傳 JSON + Set-Cookie headers
    Ok(login_response_with_cookies(&response, &state.config))
}

/// 重新整理 Token
/// 支援從 JSON body 或 Cookie 讀取 refresh_token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token 更新成功", body = LoginResponse),
        (status = 401, description = "Refresh token 無效或過期", body = ErrorResponse),
    ),
    tag = "認證"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response> {
    // 優先從 JSON body 讀取，其次從 Cookie 讀取
    let refresh_token_value = body
        .map(|Json(req)| req.refresh_token)
        .or_else(|| extract_cookie_value(&headers, "refresh_token"))
        .ok_or_else(|| AppError::Validation("Missing refresh token".to_string()))?;

    let response =
        AuthService::refresh_token(&state.db, &state.config, &refresh_token_value).await?;
    Ok(login_response_with_cookies(&response, &state.config))
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
                .expect("登出 JSON 序列化不應失敗")
                .into(),
        )
        .expect("建構登出 Response 不應失敗");

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

/// 變更自己的密碼
#[utoipa::path(
    put,
    path = "/api/me/password",
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
    let response = AuthService::change_own_password(
        &state.db,
        &state.config,
        current_user.id,
        &req.current_password,
        &req.new_password,
    )
    .await?;

    // SEC: 敏感操作二級審計 — 密碼自行變更
    let db = state.db.clone();
    let user_id = current_user.id;
    let ip_clone = ip.clone();
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    tokio::spawn(async move {
        let _ = AuditService::log_activity(
            &db,
            user_id,
            "SECURITY",
            "PASSWORD_SELF_CHANGE",
            Some("user"),
            Some(user_id),
            None,
            None,
            None,
            Some(&ip_clone),
            ua.as_deref(),
        )
        .await;
    });

    // 回傳新 tokens 的 Set-Cookie headers，保持用戶登入狀態
    Ok(login_response_with_cookies(&response, &state.config))
}

/// 忘記密碼 - 發送重設連結
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
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

    // 不管帳號存不存在都回覆相同訊息（防止帳號枚舉攻擊）
    Ok(Json(
        serde_json::json!({ "message": "If the email exists, a reset link has been sent" }),
    ))
}

/// 使用 token 重設密碼
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
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

/// SEC-33：敏感操作二級認證 — 以密碼換取短期 reauth token
#[utoipa::path(
    post,
    path = "/api/auth/confirm-password",
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
    path = "/api/auth/heartbeat",
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

    if let Err(e) = SessionManager::update_activity(&state.db, current_user.id, Some(&ip)).await {
        tracing::warn!(
            "Failed to update session activity for user {}: {}",
            current_user.id,
            e
        );
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// 停止模擬登入，恢復管理員 session
/// 從 JWT 的 impersonated_by 欄位取得管理員 ID，重新簽發管理員的正常 token
#[utoipa::path(
    post,
    path = "/api/auth/stop-impersonate",
    responses(
        (status = 200, description = "恢復管理員 session"),
        (status = 403, description = "非模擬登入狀態", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn stop_impersonate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response> {
    // 檢查是否為模擬登入狀態
    let admin_id = current_user
        .impersonated_by
        .ok_or_else(|| AppError::BusinessRule("目前不在模擬登入狀態".to_string()))?;

    // SEC-23: 將當前模擬登入的 JWT 加入黑名單
    if !current_user.jti.is_empty() {
        state
            .jwt_blacklist
            .revoke(current_user.jti.clone(), current_user.exp, &state.db)
            .await;
    }

    // 用管理員 ID 重新建立正常的 LoginResponse（不含 impersonated_by）
    let login_response =
        AuthService::impersonate_restore(&state.db, &state.config, admin_id).await?;

    // 記錄稽核日誌
    use crate::models::AuditAction;
    use crate::services::AuditService;
    AuditService::log(
        &state.db,
        admin_id,
        AuditAction::StopImpersonate,
        "user",
        current_user.id,
        None,
        Some(serde_json::json!({
            "impersonated_user_id": current_user.id,
            "admin_user_id": admin_id,
            "reason": "Admin stopped impersonation",
        })),
    )
    .await?;

    tracing::info!(
        "[Security] 停止模擬登入 - 管理員 {} 恢復身分（原模擬用戶 {}）",
        admin_id,
        current_user.id
    );

    // 回傳管理員的 token + Set-Cookie headers
    Ok(login_response_with_cookies(&login_response, &state.config))
}
