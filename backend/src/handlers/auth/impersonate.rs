use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::Response,
    Extension,
};
use std::net::SocketAddr;

use crate::error::ErrorResponse;
use crate::{
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::AuditAction,
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        AuditService, AuthService,
    },
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
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

    // SEC: 敏感操作二級審計 — 停止模擬（同步寫入 HMAC 鏈，與 IMPERSONATE_START 對稱）。
    // actor = 被模擬使用者、impersonated_by = 管理員；entity 錨在被模擬使用者。
    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ua = headers.get("user-agent").and_then(|v| v.to_str().ok());
    let actor = ActorContext::User(current_user.clone());
    AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "SECURITY",
            event_type: "IMPERSONATE_STOP",
            entity: Some(AuditEntity::new(
                "user",
                current_user.id,
                &current_user.email,
            )),
            data_diff: None,
            request_context: Some(RequestContext {
                ip_address: Some(&ip),
                user_agent: ua,
            }),
        },
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
