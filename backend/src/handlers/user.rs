use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{AuditAction, CreateUserRequest, ResetPasswordRequest, UpdateUserRequest, UserResponse},
    require_permission,
    services::{AuthService, AuditService, UserService, EmailService},
    AppError, AppState, Result,
};

#[derive(Debug, serde::Deserialize)]
pub struct UserQuery {
    pub keyword: Option<String>,
}

/// 建立用戶
pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "user.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 保存原始密碼用於寄送郵件
    let plain_password = req.password.clone();
    
    let user = UserService::create(&state.db, &req).await?;
    let response = UserService::get_by_id(&state.db, user.id).await?;
    
    // 寄送歡迎信件（異步執行，不阻塞回應）
    let config = state.config.clone();
    let email = response.email.clone();
    let display_name = response.display_name.clone();
    tokio::spawn(async move {
        if let Err(e) = EmailService::send_welcome_email(&config, &email, &display_name, &plain_password).await {
            tracing::error!("Failed to send welcome email to {}: {}", email, e);
        }
    });
    
    Ok(Json(response))
}

/// 取得用戶列表
pub async fn list_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<UserResponse>>> {
    require_permission!(current_user, "user.read");
    
    let users = UserService::list(&state.db, query.keyword.as_deref()).await?;
    Ok(Json(users))
}

/// 取得單一用戶
pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "user.read");
    
    let user = UserService::get_by_id(&state.db, id).await?;
    Ok(Json(user))
}

/// 更新用戶
pub async fn update_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "user.update");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let user = UserService::update(&state.db, id, &req).await?;
    Ok(Json(user))
}

/// 刪除用戶
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "user.delete");
    
    UserService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
}

/// Admin 重設他人密碼
pub async fn reset_user_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // 檢查權限：必須是 Admin 角色
    if !current_user.roles.contains(&"admin".to_string()) {
        return Err(AppError::BusinessRule("Only admin can reset other user's password".to_string()));
    }
    
    // 不能重設自己的密碼（應使用 /me/password）
    if id == current_user.id {
        return Err(AppError::Validation("Use /me/password to change your own password".to_string()));
    }
    
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 重設密碼
    AuthService::reset_user_password(&state.db, id, &req.new_password).await?;
    
    // 記錄稽核日誌
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
    
    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}
