use axum::{extract::State, Extension, Json};
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        ChangeOwnPasswordRequest, ForgotPasswordRequest, LoginRequest, LoginResponse,
        RefreshTokenRequest, ResetPasswordWithTokenRequest, User, UserResponse,
    },
    services::{AuthService, EmailService},
    AppError, AppState, Result,
};

/// 登入
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let response = AuthService::login(&state.db, &state.config, &req).await?;
    Ok(Json(response))
}

/// 刷新 Token
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
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    AuthService::logout(&state.db, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Logged out successfully" })))
}

/// 取得當前用戶資訊
pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UserResponse>> {
    // 取得完整用戶資訊
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
        roles,
        permissions,
    }))
}

/// 修改自己的密碼
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

/// 忘記密碼 - 發送重設信件
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 產生重設 token
    if let Some((user_id, token)) = AuthService::forgot_password(&state.db, &req.email).await? {
        // 取得用戶資訊
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

        // 異步發送重設信件
        let config = state.config.clone();
        let email = user.email.clone();
        let display_name = user.display_name.clone();
        tokio::spawn(async move {
            if let Err(e) = EmailService::send_password_reset_email(&config, &email, &display_name, &token).await {
                tracing::error!("Failed to send password reset email to {}: {}", email, e);
            }
        });
    }
    
    // 不管用戶是否存在都回傳成功（防止帳號枚舉）
    Ok(Json(serde_json::json!({ 
        "message": "If the email exists, a password reset link has been sent" 
    })))
}

/// 透過 token 重設密碼
pub async fn reset_password_with_token(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordWithTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AuthService::reset_password_with_token(&state.db, &req.token, &req.new_password).await?;
    
    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}
