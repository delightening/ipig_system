//! R121：Google 帳號登入的 HTTP 端點。流程與綁定規則見 `services/auth/google_oidc.rs`。
//!
//! 三支端點都在公開路由（無 JWT、無 CSRF；皆為 GET）：
//! - `GET /auth/google/status`：前端決定要不要顯示 Google 按鈕。
//! - `GET /auth/google/start`：設 flow cookie，302 到 Google。
//! - `GET /auth/google/callback`：Google 導回來的地方。**一律以 302 導回前端登入頁**收尾，
//!   結果放在網址上（`?google=success`／`#google_2fa=<temp token>`／`?google_error=<code>`），
//!   因為這一步是瀏覽器的頂層導覽，不是前端的 fetch，沒有 JSON 可以接。

use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use chrono::Utc;

use super::cookie::{build_set_cookie, build_set_cookie_with_path, extract_cookie_value};
use crate::{
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{GoogleCallbackQuery, GoogleLoginStatus, User},
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        google_oidc::{
            GoogleAuthResult, GoogleLoginFailure, VerifiedGoogleIdentity, GOOGLE_FLOW_COOKIE,
            GOOGLE_FLOW_COOKIE_PATH, GOOGLE_FLOW_TTL_SECS,
        },
        AuditService, AuthService, LoginTracker, SessionManager,
    },
    AppError, AppState, Result,
};

/// 前端登入頁路徑；callback 以相對路徑導回，與 callback 本身同源。
const LOGIN_PAGE: &str = "/login";
/// 與 `cookie::login_response_with_cookies`、`two_factor::verify_2fa_login` 相同的 refresh cookie 期限。
const REFRESH_COOKIE_MAX_AGE_SECS: i64 = 7 * 24 * 3600;
/// 稽核 `entity_display_name`：尚未驗明 Google 身分就失敗時沒有 email 可記。
const UNIDENTIFIED_GOOGLE_ACCOUNT: &str = "(未識別的 Google 帳號)";

/// 查詢是否開放 Google 登入
#[utoipa::path(
    get,
    path = "/api/v1/auth/google/status",
    responses((status = 200, description = "Google 登入是否啟用", body = GoogleLoginStatus)),
    tag = "認證"
)]
pub async fn google_login_status(State(state): State<AppState>) -> Json<GoogleLoginStatus> {
    Json(GoogleLoginStatus {
        enabled: state.config.google_oauth.is_some(),
    })
}

/// 開始 Google 登入：設 flow cookie 並導向 Google
#[utoipa::path(
    get,
    path = "/api/v1/auth/google/start",
    responses((status = 302, description = "導向 Google 授權頁；未設定時導回登入頁並帶錯誤代碼")),
    tag = "認證"
)]
pub async fn google_login_start(State(state): State<AppState>) -> Result<Response> {
    let Some(google) = state.config.google_oauth.as_ref() else {
        return redirect(&failure_location(GoogleLoginFailure::NotConfigured), &[]);
    };

    let start =
        AuthService::begin_google_login(google, &state.config.csrf_secret, Utc::now().timestamp())?;
    let flow_cookie = build_set_cookie_with_path(
        GOOGLE_FLOW_COOKIE,
        &start.flow_cookie_value,
        GOOGLE_FLOW_TTL_SECS,
        GOOGLE_FLOW_COOKIE_PATH,
        &state.config,
    );
    redirect(&start.authorization_url, &[flow_cookie])
}

/// Google 導回的 callback：驗證後登入，並導回前端登入頁
#[utoipa::path(
    get,
    path = "/api/v1/auth/google/callback",
    responses((status = 302, description = "導回前端登入頁，結果在網址參數或 fragment")),
    tag = "認證"
)]
pub async fn google_login_callback(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<Response> {
    // flow cookie 一次性：不論結果如何都清掉，避免同一組 state 被重送
    let clear_flow_cookie = build_set_cookie_with_path(
        GOOGLE_FLOW_COOKIE,
        "",
        0,
        GOOGLE_FLOW_COOKIE_PATH,
        &state.config,
    );
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let identity = match verify_callback(&state, &headers, &query).await {
        Ok(identity) => identity,
        Err(failure) => {
            tracing::warn!(reason = failure.code(), ip = %ip, "[Google OIDC] callback 被拒");
            // 身分尚未驗明（state 不符、交換失敗、email 未驗證…），稽核上沒有 email 可記
            audit_login_failure(&state, None, failure, &ip, user_agent.as_deref()).await;
            return redirect(&failure_location(failure), &[clear_flow_cookie]);
        }
    };

    let request_context = RequestContext {
        ip_address: Some(&ip),
        user_agent: user_agent.as_deref(),
    };
    let resolved = AuthService::resolve_google_user(&state.db, &identity, request_context).await?;
    let user = match resolved {
        GoogleAuthResult::Authenticated(user) => *user,
        GoogleAuthResult::Rejected(failure) => {
            audit_login_failure(
                &state,
                Some(&identity.email),
                failure,
                &ip,
                user_agent.as_deref(),
            )
            .await;
            return redirect(&failure_location(failure), &[clear_flow_cookie]);
        }
    };

    // 有開 TOTP：Google 只取代密碼這一關，第二關照走既有 2FA。
    // temp token 放 fragment（`#`）而非 query：fragment 不會送到任何伺服器、不進 Referer，
    // 前端讀出後立即從網址列移除。它本身只能拿來「嘗試輸入驗證碼」，5 分鐘失效。
    if user.totp_enabled {
        let temp_token = AuthService::generate_2fa_temp_token(&state.config, user.id)?;
        return redirect(
            &format!("{LOGIN_PAGE}#google_2fa={temp_token}"),
            &[clear_flow_cookie],
        );
    }

    // 與密碼登入同一順序：session（含併發上限）先建好，最後才簽 token（見 login.rs H4 註解）
    SessionManager::create_session(&state.db, user.id, Some(&ip), user_agent.as_deref()).await?;
    SessionManager::end_excess_sessions(&state.db, user.id, state.config.max_sessions_per_user)
        .await?;
    let tokens = AuthService::issue_login_tokens(&state.db, &state.config, &user).await?;

    audit_login_success(&state, &user, &ip, user_agent.as_deref()).await?;
    spawn_login_success_telemetry(&state, &user, ip, user_agent);

    let access_cookie = build_set_cookie(
        "access_token",
        &tokens.access_token,
        tokens.expires_in,
        &state.config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token",
        &tokens.refresh_token,
        REFRESH_COOKIE_MAX_AGE_SECS,
        &state.config,
    );
    redirect(
        &format!("{LOGIN_PAGE}?google=success"),
        &[clear_flow_cookie, access_cookie, refresh_cookie],
    )
}

/// 驗 flow cookie 與 state → 換 token → 驗 ID token。任何一步失敗都回拒絕原因。
async fn verify_callback(
    state: &AppState,
    headers: &HeaderMap,
    query: &GoogleCallbackQuery,
) -> std::result::Result<VerifiedGoogleIdentity, GoogleLoginFailure> {
    let google = state
        .config
        .google_oauth
        .as_ref()
        .ok_or(GoogleLoginFailure::NotConfigured)?;

    let flow = extract_cookie_value(headers, GOOGLE_FLOW_COOKIE)
        .and_then(|raw| {
            AuthService::open_google_flow_state(
                &state.config.csrf_secret,
                &raw,
                Utc::now().timestamp(),
            )
        })
        .ok_or(GoogleLoginFailure::InvalidState)?;
    // state 必須先比對，連「使用者取消」也要：否則第三方可以偽造 callback 讓人看到錯誤訊息
    let returned_state = query
        .state
        .as_deref()
        .ok_or(GoogleLoginFailure::InvalidState)?;
    if !AuthService::google_state_matches(&flow, returned_state) {
        return Err(GoogleLoginFailure::InvalidState);
    }

    if let Some(error) = query.error.as_deref() {
        if error == "access_denied" {
            return Err(GoogleLoginFailure::Cancelled);
        }
        tracing::warn!(error, "[Google OIDC] Google 回報授權錯誤");
        return Err(GoogleLoginFailure::ProviderError);
    }
    let code = query
        .code
        .as_deref()
        .filter(|c| !c.is_empty())
        .ok_or(GoogleLoginFailure::ProviderError)?;

    let id_token = AuthService::exchange_google_code(google, code, &flow.code_verifier).await?;
    AuthService::verify_google_id_token(&id_token, google, &flow.nonce, Utc::now().timestamp())
}

fn failure_location(failure: GoogleLoginFailure) -> String {
    format!("{LOGIN_PAGE}?google_error={}", failure.code())
}

fn redirect(location: &str, cookies: &[String]) -> Result<Response> {
    let mut builder = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, location)
        // 帶 Set-Cookie 的導向回應不得被任何快取保存
        .header(header::CACHE_CONTROL, "no-store");
    for cookie in cookies {
        builder = builder.header(header::SET_COOKIE, cookie);
    }
    builder
        .body(Body::empty())
        .map_err(|e| AppError::Internal(format!("Response 建構失敗: {e}")))
}

/// `LOGIN_SUCCESS` 進 HMAC 稽核鏈；與密碼登入同事件名，`data_diff.method = "google"` 區分。
async fn audit_login_success(
    state: &AppState,
    user: &User,
    ip: &str,
    user_agent: Option<&str>,
) -> Result<()> {
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
            data_diff: Some(AuthService::google_login_success_audit()),
            request_context: Some(RequestContext {
                ip_address: Some(ip),
                user_agent,
            }),
        },
    )
    .await?;
    Ok(())
}

/// Google 登入被拒時寫 `LOGIN_FAILED`（Anonymous actor），`data_diff` 帶 `method` 與原因代碼。
/// `email` 只有在已驗明 Google 身分之後才有；之前的失敗（state 不符等）記為未識別。
/// 寫入失敗不擋導向（使用者本來就是被拒），只留 log——與密碼登入失敗路徑一致。
///
/// 刻意**不**呼叫 `LoginTracker::log_failure`：它寫的 `login_failure` 是密碼帳號鎖定的計數來源，
/// Google 端的拒絕不應讓同 email 的密碼登入被鎖。
async fn audit_login_failure(
    state: &AppState,
    email: Option<&str>,
    failure: GoogleLoginFailure,
    ip: &str,
    user_agent: Option<&str>,
) {
    if !failure.is_security_relevant() {
        return;
    }
    let result = AuditService::log_activity_oneshot(
        &state.db,
        &ActorContext::Anonymous,
        ActivityLogEntry {
            event_category: "SECURITY",
            event_type: "LOGIN_FAILED",
            entity: Some(AuditEntity::new(
                "user",
                uuid::Uuid::nil(),
                email.unwrap_or(UNIDENTIFIED_GOOGLE_ACCOUNT),
            )),
            data_diff: Some(AuthService::google_login_failure_audit(failure)),
            request_context: Some(RequestContext {
                ip_address: Some(ip),
                user_agent,
            }),
        },
    )
    .await;
    if let Err(e) = result {
        tracing::warn!("[Google OIDC] LOGIN_FAILED 稽核寫入失敗: {e}");
    }
}

/// GeoIP 異常偵測屬遙測，與密碼登入相同採 fire-and-forget。
fn spawn_login_success_telemetry(
    state: &AppState,
    user: &User,
    ip: String,
    user_agent: Option<String>,
) {
    let db = state.db.clone();
    let geoip = state.geoip.clone();
    let user_id = user.id;
    let email = user.email.clone();
    tokio::spawn(async move {
        if let Err(e) = LoginTracker::log_success(
            &db,
            user_id,
            &email,
            Some(&ip),
            user_agent.as_deref(),
            &geoip,
        )
        .await
        {
            tracing::error!("[Google OIDC] 登入成功遙測寫入失敗: {e}");
        }
    });
}
