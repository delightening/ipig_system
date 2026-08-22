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
    handlers::{auth::build_set_cookie, user::require_reauth_token},
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{
        TwoFactorConfirmRequest, TwoFactorDisableRequest, TwoFactorLoginRequest,
        TwoFactorSetupResponse,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        AuditService, AuthService, LoginTracker, SessionManager, UserService,
    },
    AppError, AppState, Result,
};

/// POST /api/auth/2fa/setup — 產生 TOTP secret（僅限管理員）
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/setup",
    responses(
        (status = 200, description = "TOTP 設定資訊", body = TwoFactorSetupResponse),
        (status = 403, description = "僅管理員可啟用", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn setup_2fa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<TwoFactorSetupResponse>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可啟用兩步驟驗證".into()));
    }
    // C3 (GLP §11.200(a))：啟用 2FA 是 credential 異動，要求 X-Reauth-Token
    // 確保操作者於近期重新輸入密碼確認身分（防 XSS / session hijack 後攻擊者
    // 直接重設受害者的 2FA 設定）。
    require_reauth_token(&headers, &state, &current_user)?;

    let user = UserService::get_user_raw(&state.db, current_user.id).await?;
    if user.totp_enabled {
        return Err(AppError::BusinessRule("2FA 已經啟用".into()));
    }

    let (otpauth_uri, backup_codes) =
        AuthService::generate_totp_setup(&state.db, &state.config, current_user.id, &user.email)
            .await?;

    Ok(Json(TwoFactorSetupResponse {
        otpauth_uri,
        backup_codes,
    }))
}

/// POST /api/auth/2fa/confirm — 驗證第一次 TOTP code 並正式啟用（僅限管理員）
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/confirm",
    request_body = TwoFactorConfirmRequest,
    responses(
        (status = 200, description = "2FA 已啟用"),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
        (status = 403, description = "僅管理員可啟用", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn confirm_2fa_setup(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<TwoFactorConfirmRequest>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可啟用兩步驟驗證".into()));
    }
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let actor = ActorContext::User(current_user.clone());
    AuthService::confirm_totp_setup(
        &state.db,
        &state.config,
        &actor,
        current_user.id,
        &req.code,
        Some(&ip),
        user_agent,
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "2FA 已成功啟用" })))
}

/// POST /api/auth/2fa/disable — 停用 2FA（需密碼 + TOTP code）
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/disable",
    request_body = TwoFactorDisableRequest,
    responses(
        (status = 200, description = "2FA 已停用"),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
    ),
    tag = "認證",
    security(("bearer" = []))
)]
pub async fn disable_2fa(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<TwoFactorDisableRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate()?;

    AuthService::verify_password_by_id(&state.db, current_user.id, &req.password).await?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let actor = ActorContext::User(current_user.clone());
    AuthService::disable_totp(
        &state.db,
        &state.config,
        &actor,
        current_user.id,
        &req.code,
        Some(&ip),
        user_agent,
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "2FA 已停用" })))
}

/// POST /api/auth/2fa/verify — 使用 temp_token + TOTP code 完成登入
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/verify",
    request_body = TwoFactorLoginRequest,
    responses(
        (status = 200, description = "登入成功，回傳 Set-Cookie"),
        (status = 400, description = "驗證失敗", body = ErrorResponse),
    ),
    tag = "認證"
)]
pub async fn verify_2fa_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<TwoFactorLoginRequest>,
) -> Result<Response> {
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // High-3 (#207)：先驗證 2FA 取得 user，再 session-before-token——
    // create_session + end_excess_sessions（SEC-28 併發上限）同步執行、失敗即中止，
    // 最後才簽發 token，與密碼登入路徑（handlers/auth/login.rs）一致。
    let user = AuthService::verify_2fa_and_load_user(
        &state.db,
        &state.config,
        &req.temp_token,
        &req.code,
        Some(&ip),
    )
    .await?;
    let user_id = user.id;

    SessionManager::create_session(&state.db, user_id, Some(&ip), user_agent.as_deref()).await?;
    SessionManager::end_excess_sessions(&state.db, user_id, state.config.max_sessions_per_user)
        .await?;

    let response = AuthService::issue_login_tokens(&state.db, &state.config, &user).await?;

    // bot review #628 / R26-15：2FA 登入成功也要寫 LOGIN_SUCCESS 進 user_activity_logs
    // （與密碼登入路徑一致）。admin 強制 2FA，若漏寫則最高權限登入不在不可竄改稽核鏈。
    {
        let actor = ActorContext::User(CurrentUser {
            id: user.id,
            email: user.email.clone(),
            roles: vec![],
            permissions: vec![],
            jti: String::new(),
            exp: 0,
            impersonated_by: None,
        });
        AuditService::log_activity_oneshot(
            &state.db,
            &actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "LOGIN_SUCCESS",
                entity: Some(AuditEntity::new("user", user.id, &user.display_name)),
                data_diff: None,
                request_context: Some(RequestContext {
                    ip_address: Some(&ip),
                    user_agent: user_agent.as_deref(),
                }),
            },
        )
        .await?;
    }

    // 登入成功事件（純遙測，維持 fire-and-forget）
    let db = state.db.clone();
    let geoip = state.geoip.clone();
    let email = user.email.clone();
    let ip_clone = ip.clone();
    let ua_clone = user_agent.clone();
    tokio::spawn(async move {
        let _ = LoginTracker::log_success(
            &db,
            user_id,
            &email,
            Some(&ip_clone),
            ua_clone.as_deref(),
            &geoip,
        )
        .await;
    });

    let access_cookie = build_set_cookie(
        "access_token",
        &response.access_token,
        response.expires_in,
        &state.config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token",
        &response.refresh_token,
        7 * 24 * 3600,
        &state.config,
    );

    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("JSON 序列化失敗: {}", e)))?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, access_cookie)
        .header(header::SET_COOKIE, refresh_cookie)
        .body(body.into())
        .map_err(|e| AppError::Internal(format!("Response 建構失敗: {e}")))
}
