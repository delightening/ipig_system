use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    handlers::auth::build_set_cookie,
    middleware::{extract_real_ip_with_trust, CurrentUser},
    models::{
        TwoFactorConfirmRequest, TwoFactorDisableRequest,
        TwoFactorLoginRequest, TwoFactorSetupResponse,
    },
    services::{AuthService, AuditService, LoginTracker, SessionManager, UserService},
    AppError, AppState, Result,
};

/// POST /api/auth/2fa/setup — 產生 TOTP secret（僅限管理員）
pub async fn setup_2fa(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<TwoFactorSetupResponse>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可啟用兩步驟驗證".into()));
    }
    let user = UserService::get_user_raw(&state.db, current_user.id).await?;
    if user.totp_enabled {
        return Err(AppError::BusinessRule("2FA 已經啟用".into()));
    }

    let (otpauth_uri, backup_codes) =
        AuthService::generate_totp_setup(&state.db, current_user.id, &user.email).await?;

    Ok(Json(TwoFactorSetupResponse {
        otpauth_uri,
        backup_codes,
    }))
}

/// POST /api/auth/2fa/confirm — 驗證第一次 TOTP code 並正式啟用（僅限管理員）
pub async fn confirm_2fa_setup(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<TwoFactorConfirmRequest>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可啟用兩步驟驗證".into()));
    }
    req.validate()?;

    AuthService::confirm_totp_setup(&state.db, current_user.id, &req.code).await?;

    let db = state.db.clone();
    let user_id = current_user.id;
    tokio::spawn(async move {
        let _ = AuditService::log_activity(
            &db, user_id, "SECURITY", "2FA_ENABLED",
            Some("user"), Some(user_id), None, None, None, None, None,
        ).await;
    });

    Ok(Json(serde_json::json!({ "message": "2FA 已成功啟用" })))
}

/// POST /api/auth/2fa/disable — 停用 2FA（需密碼 + TOTP code）
pub async fn disable_2fa(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<TwoFactorDisableRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate()?;

    AuthService::verify_password_by_id(&state.db, current_user.id, &req.password).await?;
    AuthService::disable_totp(&state.db, current_user.id, &req.code).await?;

    let db = state.db.clone();
    let user_id = current_user.id;
    tokio::spawn(async move {
        let _ = AuditService::log_activity(
            &db, user_id, "SECURITY", "2FA_DISABLED",
            Some("user"), Some(user_id), None, None, None, None, None,
        ).await;
    });

    Ok(Json(serde_json::json!({ "message": "2FA 已停用" })))
}

/// POST /api/auth/2fa/verify — 使用 temp_token + TOTP code 完成登入
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

    let response = AuthService::complete_2fa_login(
        &state.db, &state.config, &req.temp_token, &req.code,
    ).await?;

    // 登入成功事件
    let db = state.db.clone();
    let geoip = state.geoip.clone();
    let broadcaster = state.alert_broadcaster.clone();
    let user_id = response.user.id;
    let email = response.user.email.clone();
    let ip_clone = ip.clone();
    let ua_clone = user_agent.clone();
    let max_sess = state.config.max_sessions_per_user;

    tokio::spawn(async move {
        let _ = LoginTracker::log_success(
            &db, user_id, &email, Some(&ip_clone),
            ua_clone.as_deref(), &geoip, &broadcaster,
        ).await;
        let _ = SessionManager::create_session(
            &db, user_id, Some(&ip_clone), ua_clone.as_deref(),
        ).await;

        if let Ok(count) = SessionManager::get_active_session_count(&db, user_id).await {
            if count > max_sess {
                let excess = count - max_sess;
                let _ = sqlx::query(
                    r#"UPDATE user_sessions SET is_active = false, ended_at = NOW(), ended_reason = 'session_limit'
                       WHERE id IN (SELECT id FROM user_sessions WHERE user_id = $1 AND is_active = true ORDER BY started_at ASC LIMIT $2)"#,
                )
                .bind(user_id)
                .bind(excess)
                .execute(&db)
                .await;
            }
        }
    });

    let access_cookie = build_set_cookie(
        "access_token", &response.access_token, response.expires_in, &state.config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token", &response.refresh_token, 7 * 24 * 3600, &state.config,
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
