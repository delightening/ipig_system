use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use uuid::Uuid;
use validator::Validate;

use crate::error::ErrorResponse;
use crate::{
    middleware::{ActorContext, CurrentUser, extract_real_ip_with_trust},
    models::{AuditAction, CreateUserRequest, PaginationParams, ResetPasswordRequest, UpdateUserRequest, UserResponse},
    require_permission,
    services::{AuthService, AuditService, SessionManager, UserService, EmailService},
    AppError, AppState, Result,
};

use super::auth::build_set_cookie;

#[derive(Debug, serde::Deserialize)]
pub struct UserQuery {
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 建立使用者
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "建立成功", body = UserResponse),
        (status = 400, description = "驗證錯誤", body = ErrorResponse),
        (status = 409, description = "信箱已存在", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "admin.user.create");
    req.validate()?;

    let actor = ActorContext::User(current_user.clone());
    let user = UserService::create(&state.db, &actor, &req).await?;
    let response = UserService::get_by_id(&state.db, user.id).await?;

    // 生成密碼重設 token (改為發送重設連結而非明文密碼)
    let reset_result = AuthService::forgot_password(&state.db, &response.email).await?;

    // 非同步發送包含重設連結的歡迎郵件
    let config = state.config.clone();
    let email = response.email.clone();
    let display_name = response.display_name.clone();

    tokio::spawn(async move {
        if let Some((_, token)) = reset_result {
            if let Err(e) = EmailService::send_welcome_email(&config, &email, &display_name, &token).await {
                tracing::error!("Failed to send welcome email to {}: {}", email, e);
            }
        } else {
            tracing::error!("Failed to generate password reset token for new user: {}", email);
        }
    });
    
    Ok(Json(response))
}

/// 列出所有使用者
#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "使用者清單", body = Vec<UserResponse>),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<UserResponse>>> {
    require_permission!(current_user, "admin.user.view");
    
    let pagination = PaginationParams { page: query.page, per_page: query.per_page };
    let users = UserService::list(&state.db, query.keyword.as_deref(), &pagination).await?;
    Ok(Json(users))
}

/// 取得單個使用者
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "使用者 ID")
    ),
    responses(
        (status = 200, description = "使用者資訊", body = UserResponse),
        (status = 404, description = "使用者不存在", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    if current_user.id != id {
        require_permission!(current_user, "admin.user.view");
    }
    
    let user = UserService::get_by_id(&state.db, id).await?;
    Ok(Json(user))
}

/// 更新使用者
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "使用者 ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "更新成功", body = UserResponse),
        (status = 400, description = "驗證錯誤", body = ErrorResponse),
        (status = 404, description = "使用者不存在", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn update_user(
    State(state): State<AppState>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "admin.user.edit");
    req.validate()?;

    // 取 before 快照供 post-update session 撤銷判斷；audit 由 service 層處理
    let before_user = UserService::get_by_id(&state.db, id).await.ok();

    let actor = ActorContext::User(current_user.clone());
    let user = UserService::update(&state.db, &actor, id, &req).await?;

    // H-01: 角色或帳號狀態變更後立即清除該使用者的權限快取，避免舊快取在 TTL 到期前繼續生效
    if let Some(ref before) = before_user {
        if before.roles != user.roles || before.is_active != user.is_active {
            state.permission_cache.remove(&id);
        }

        // BIZ-16: 帳號停用時立即撤銷所有 session 和 refresh token，不等 TTL
        // 場景：人員異動或安全事件下，5 分鐘視窗不可接受
        if before.is_active && !user.is_active {
            // 撤銷 refresh tokens — 阻止 token refresh
            if let Err(e) = AuthService::revoke_all_user_tokens(&state.db, id).await {
                tracing::error!("[Security] 停用帳號 {} 時撤銷 refresh tokens 失敗: {}", id, e);
            }
            // 終止所有 sessions
            if let Err(e) = SessionManager::end_all_sessions(&state.db, id, "account_deactivated").await {
                tracing::error!("[Security] 停用帳號 {} 時終止 sessions 失敗: {}", id, e);
            }
            tracing::warn!("[Security] 帳號 {} 已停用，所有 session/refresh token 已撤銷", id);
        }
    }

    Ok(Json(user))
}

/// SEC-33：敏感操作需帶 X-Reauth-Token（先 POST /auth/confirm-password 取得）
/// 供 user 與 role 等敏感 handler 共用
pub(crate) fn require_reauth_token(
    headers: &HeaderMap,
    state: &AppState,
    current_user: &CurrentUser,
) -> Result<()> {
    let token = headers
        .get("x-reauth-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("敏感操作請先重新輸入密碼確認".to_string()))?;
    AuthService::verify_reauth_token(&state.config, token, current_user.id)
}

/// 刪除使用者
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "使用者 ID")
    ),
    responses(
        (status = 200, description = "刪除成功"),
        (status = 403, description = "需帶 X-Reauth-Token 重新確認密碼", body = ErrorResponse),
        (status = 404, description = "使用者不存在", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "admin.user.delete");
    require_reauth_token(&headers, &state, &current_user)?;

    // H5: 禁止刪除自己
    if id == current_user.id {
        return Err(AppError::Validation("無法刪除自己的帳號".to_string()));
    }

    // H5: 末代管理員保護 — 確保刪除後至少仍有一位啟用中的管理員
    let (admin_count,): (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM users u
           INNER JOIN user_roles ur ON u.id = ur.user_id
           INNER JOIN roles r ON r.id = ur.role_id
           WHERE r.code IN ('SYSTEM_ADMIN', 'admin')
             AND u.is_active = true
             AND u.id != $1"#,
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if admin_count == 0 {
        return Err(AppError::Validation("無法刪除最後一位管理員帳號".to_string()));
    }

    let actor = ActorContext::User(current_user.clone());
    UserService::delete(&state.db, &actor, id).await?;
    // H-01: 使用者刪除後清除其權限快取
    state.permission_cache.remove(&id);
    Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
}

/// Admin 重設其他使用者密碼
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}/password",
    params(
        ("id" = Uuid, Path, description = "使用者 ID")
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "密碼重設成功"),
        (status = 403, description = "權限不足", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn reset_user_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // 檢查權限，必須是 Admin 角色
    if !current_user.is_admin() {
        return Err(AppError::BusinessRule("Only admin can reset other user's password".to_string()));
    }
    require_reauth_token(&headers, &state, &current_user)?;
    
    // 不允許重設自己的密碼，應使用 /me/password
    if id == current_user.id {
        return Err(AppError::Validation("Use /me/password to change your own password".to_string()));
    }
    
    req.validate()?;
    
    // 重設密碼
    AuthService::reset_user_password(&state.db, id, &req.new_password).await?;
    
    // 原有稽核日誌
    AuditService::log(
        &state.db,
        current_user.id,
        AuditAction::PasswordReset,
        "user",
        id,
        None,
        Some(serde_json::json!({
            "target_user_id": id,
            "reset_by": current_user.id,
        })),
    ).await?;

    // SEC: 敏感操作二級審計 — 管理員重設密碼
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ua = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let db = state.db.clone();
    let actor = current_user.id;
    tokio::spawn(async move {
        if let Err(e) = AuditService::log_activity(
            &db, actor, "SECURITY", "PASSWORD_ADMIN_RESET",
            Some("user"), Some(id), None,
            None, Some(serde_json::json!({ "target_user_id": id })),
            Some(&ip), ua.as_deref(),
        ).await { tracing::error!("審計日誌寫入失敗: {e}"); }
    });
    
    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}

/// 模擬登入使用者（管理員專用的測試功能）
/// 回傳 Response 含 Set-Cookie headers
#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/impersonate",
    params(
        ("id" = Uuid, Path, description = "要模擬的使用者 ID")
    ),
    responses(
        (status = 200, description = "模擬登入成功"),
        (status = 403, description = "僅限管理員", body = ErrorResponse),
    ),
    tag = "使用者管理",
    security(("bearer" = []))
)]
pub async fn impersonate_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    // 檢查權限，必須是 Admin 角色
    if !current_user.is_admin() {
        return Err(AppError::BusinessRule("Only admin can impersonate other users".to_string()));
    }
    require_reauth_token(&headers, &state, &current_user)?;
    
    // 不允許模擬自己
    if id == current_user.id {
        return Err(AppError::Validation("Cannot impersonate yourself".to_string()));
    }
    
    // 執行模擬登入
    let login_response = AuthService::impersonate(&state.db, &state.config, current_user.id, id).await?;
    
    // 原有稽核日誌
    AuditService::log(
        &state.db,
        current_user.id,
        AuditAction::Impersonate,
        "user",
        id,
        None,
        Some(serde_json::json!({
            "impersonated_user_id": id,
            "impersonated_by": current_user.id,
            "reason": "Admin impersonation for testing",
        })),
    ).await?;

    // SEC: 敏感操作二級審計 — 模擬登入
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ua = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let target_name = login_response.user.display_name.clone();
    let db = state.db.clone();
    let actor = current_user.id;
    tokio::spawn(async move {
        if let Err(e) = AuditService::log_activity(
            &db, actor, "SECURITY", "IMPERSONATE_START",
            Some("user"), Some(id), Some(&target_name),
            None, Some(serde_json::json!({ "impersonated_user_id": id })),
            Some(&ip), ua.as_deref(),
        ).await { tracing::error!("審計日誌寫入失敗: {e}"); }
    });
    
    // 回傳 JSON + Set-Cookie headers
    let access_cookie = build_set_cookie(
        "access_token",
        &login_response.access_token,
        login_response.expires_in,
        &state.config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token",
        &login_response.refresh_token,
        7 * 24 * 3600,
        &state.config,
    );

    let body = serde_json::to_string(&login_response)
        .map_err(|e| AppError::Internal(format!("serialize error: {e}")))?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, access_cookie)
        .header(header::SET_COOKIE, refresh_cookie)
        .body(body.into())
        .map_err(|e| AppError::Internal(format!("response build error: {e}")))?;

    Ok(response)
}

