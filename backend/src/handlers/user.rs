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

/// 撱箇??冽
pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "dev.user.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 靽???撖Ⅳ?冽撖隞?
    let plain_password = req.password.clone();
    
    let user = UserService::create(&state.db, &req).await?;
    let response = UserService::get_by_id(&state.db, user.id).await?;
    
    // 撖迭餈縑隞塚??唳郊?瑁?嚗??餃???嚗?
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

/// ???冽?”
pub async fn list_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<UserResponse>>> {
    require_permission!(current_user, "dev.user.view");
    
    let users = UserService::list(&state.db, query.keyword.as_deref()).await?;
    Ok(Json(users))
}

/// ???桐??冽
pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "dev.user.view");
    
    let user = UserService::get_by_id(&state.db, id).await?;
    Ok(Json(user))
}

/// ?湔?冽
pub async fn update_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    require_permission!(current_user, "dev.user.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let user = UserService::update(&state.db, id, &req).await?;
    Ok(Json(user))
}

/// ?芷?冽
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dev.user.delete");
    
    UserService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
}

/// Admin ?身隞犖撖Ⅳ
pub async fn reset_user_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // 瑼Ｘ甈?嚗?? Admin 閫
    if !current_user.roles.contains(&"admin".to_string()) {
        return Err(AppError::BusinessRule("Only admin can reset other user's password".to_string()));
    }
    
    // 銝?身?芸楛??蝣潘??蝙??/me/password嚗?
    if id == current_user.id {
        return Err(AppError::Validation("Use /me/password to change your own password".to_string()));
    }
    
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // ?身撖Ⅳ
    AuthService::reset_user_password(&state.db, id, &req.new_password).await?;
    
    // 閮?蝔賣?亥?
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

