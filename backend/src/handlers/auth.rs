use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    Extension, Json,
};
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        ChangeOwnPasswordRequest, ForgotPasswordRequest, LoginRequest, LoginResponse,
        RefreshTokenRequest, ResetPasswordWithTokenRequest, UpdateUserRequest, User, UserResponse,
    },
    services::{AuthService, UserService, EmailService, LoginTracker, SessionManager},
    AppError, AppState, Result,
};


/// 登入
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // 提取 IP 和 User-Agent
    let ip = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 嘗試登入
    let response = match AuthService::login(&state.db, &state.config, &req).await {
        Ok(resp) => {
            // 登入成功：記錄事件和建立 session
            let db = state.db.clone();
            let user_id = resp.user.id;
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
                ).await {
                    tracing::error!("Failed to log login success for {}: {}", email, e);
                }
                
                if let Err(e) = SessionManager::create_session(
                    &db,
                    user_id,
                    Some(&ip_clone),
                    ua_clone.as_deref(),
                ).await {
                    tracing::error!("Failed to create session for {}: {}", email, e);
                }
            });
            
            resp
        }
        Err(e) => {
            // 登入失敗：記錄失敗事件（會自動檢測暴力破解）
            let db = state.db.clone();
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
                ).await {
                    tracing::error!("Failed to log login failure for {}: {}", email, log_err);
                }
            });
            
            return Err(e);
        }
    };
    
    Ok(Json(response))
}


/// 重新整理 Token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>> {
    let response = AuthService::refresh_token(&state.db, &state.config, &req.refresh_token).await?;
    Ok(Json(response))
}

/// 登出
pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    // 記錄登出事件
    let _ = LoginTracker::log_logout(
        &state.db,
        current_user.id,
        &current_user.email,
        Some(&addr.ip().to_string()),
    ).await;
    
    // 結束所有 sessions
    let _ = SessionManager::end_all_sessions(
        &state.db,
        current_user.id,
        "logout",
    ).await;
    
    AuthService::logout(&state.db, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Logged out successfully" })))
}


/// 取得當前使用者資訊
pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UserResponse>> {
    // 查詢當前使用者資訊
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(current_user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let (roles, permissions) = AuthService::get_user_roles_permissions(&state.db, current_user.id).await?;
    
    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        phone: user.phone,
        organization: user.organization,
        is_internal: user.is_internal,
        is_active: user.is_active,
        must_change_password: user.must_change_password,
        theme_preference: user.theme_preference,
        language_preference: user.language_preference,
        last_login_at: user.last_login_at,
        entry_date: user.entry_date,
        position: user.position,
        aup_roles: user.aup_roles,
        years_experience: user.years_experience,
        trainings: user.trainings.0,
        roles,
        permissions,
    }))
}

/// 更新自己的資訊
pub async fn update_me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(mut req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 限制使用者只能修改特定欄位
    // 預防安全性問題，強制將不允許修改的欄位設為 None
    req.is_active = None;
    req.is_internal = None;
    req.role_ids = None;
    
    let user = UserService::update(&state.db, current_user.id, current_user.id, &req).await?;
    Ok(Json(user))
}

/// 變更自己的密碼
pub async fn change_own_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ChangeOwnPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AuthService::change_own_password(
        &state.db,
        current_user.id,
        &req.current_password,
        &req.new_password,
    ).await?;
    
    Ok(Json(serde_json::json!({ "message": "Password changed successfully" })))
}

/// 忘記密碼 - 發送重設連結
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 產生重設 token
    if let Some((user_id, token)) = AuthService::forgot_password(&state.db, &req.email).await? {
        // 查詢使用者資訊
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

        // 非同步發送重設密碼郵件
        let config = state.config.clone();
        let email = user.email.clone();
        let display_name = user.display_name.clone();
        tokio::spawn(async move {
            if let Err(e) = EmailService::send_password_reset_email(&config, &email, &display_name, &token).await {
                tracing::error!("Failed to send password reset email to {}: {}", email, e);
            }
        });
    }
    
    // 無論使用者是否存在，都返回相同訊息，避免洩露使用者資訊
    Ok(Json(serde_json::json!({ 
        "message": "If the email exists, a password reset link has been sent" 
    })))
}

/// 使用 token 重設密碼
pub async fn reset_password_with_token(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordWithTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AuthService::reset_password_with_token(&state.db, &req.token, &req.new_password).await?;
    
    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}
