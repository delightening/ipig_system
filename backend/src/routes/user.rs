use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// 使用者、角色、權限路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // Users
        .route(
            "/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route(
            "/users/:id",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .route("/users/:id/delete", post(handlers::delete_user))
        .route("/users/:id/password", put(handlers::reset_user_password))
        .route("/users/:id/impersonate", post(handlers::impersonate_user))
        // Roles
        .route(
            "/roles",
            get(handlers::list_roles).post(handlers::create_role),
        )
        .route(
            "/roles/:id",
            get(handlers::get_role)
                .put(handlers::update_role)
                .delete(handlers::delete_role),
        )
        .route("/roles/:id/delete", post(handlers::delete_role))
        .route("/permissions", get(handlers::list_permissions))
}
