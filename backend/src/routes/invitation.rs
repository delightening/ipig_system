use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{handlers, AppState};

/// 需要認證的邀請管理路由（IACUC_STAFF / SYSTEM_ADMIN）
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(handlers::create_invitation))
        .route("/invitations", get(handlers::list_invitations))
        .route("/invitations/:id", delete(handlers::revoke_invitation))
        .route(
            "/invitations/:id/resend",
            post(handlers::resend_invitation),
        )
}

/// 公開邀請路由（無需認證，由外層 rate limiter 保護）
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/invitations/verify/:token",
            get(handlers::verify_invitation),
        )
        .route("/invitations/accept", post(handlers::accept_invitation))
}
