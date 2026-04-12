use axum::{
    extract::State,
    response::Response,
    Extension,
};

use crate::error::ErrorResponse;
use crate::{
    middleware::CurrentUser,
    models::AuditAction,
    services::{AuditService, AuthService},
    AppError, AppState, Result,
};

use super::cookie::login_response_with_cookies;

/// 停止模擬登入，恢復管理員 session
/// 從 JWT 的 impersonated_by 欄位取得管理員 ID，重新簽發管理員的正常 token
#[utoipa::path(
    post,
    path = "/api/v1/auth/stop-impersonate",
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
    login_response_with_cookies(&login_response, &state.config)
}
