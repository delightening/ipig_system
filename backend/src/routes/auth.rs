use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// 公開認證路由（不需登入）
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/forgot-password", post(handlers::forgot_password))
        .route(
            "/auth/reset-password",
            post(handlers::reset_password_with_token),
        )
        .route("/auth/2fa/verify", post(handlers::verify_2fa_login))
}

/// 受保護認證路由 + /me 個人設定
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/confirm-password", post(handlers::confirm_password))
        .route("/auth/2fa/setup", post(handlers::setup_2fa))
        .route("/auth/2fa/confirm", post(handlers::confirm_2fa_setup))
        .route("/auth/2fa/disable", post(handlers::disable_2fa))
        .route("/auth/stop-impersonate", post(handlers::stop_impersonate))
        .route("/auth/heartbeat", post(handlers::heartbeat))
        .route("/me", get(handlers::me).put(handlers::update_me))
        .route("/me/export", get(handlers::export_me))
        .route("/me/account", delete(handlers::delete_me_account))
        .route("/me/account/delete", post(handlers::delete_me_account))
        .route("/me/password", put(handlers::change_own_password))
        // User Preferences
        .route("/me/preferences", get(handlers::get_all_preferences))
        .route(
            "/me/preferences/:key",
            get(handlers::get_preference)
                .put(handlers::upsert_preference)
                .delete(handlers::delete_preference),
        )
        .route(
            "/me/preferences/:key/delete",
            post(handlers::delete_preference),
        )
}
