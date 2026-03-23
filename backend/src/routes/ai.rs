use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, middleware::ai_auth::ai_auth_middleware, AppState};

/// AI 管理端路由（需 JWT 管理員認證，走 protected_routes）
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/ai/admin/keys", post(handlers::ai::create_ai_api_key))
        .route("/ai/admin/keys", get(handlers::ai::list_ai_api_keys))
        .route(
            "/ai/admin/keys/:id/toggle",
            put(handlers::ai::toggle_ai_api_key),
        )
        .route(
            "/ai/admin/keys/:id",
            delete(handlers::ai::delete_ai_api_key),
        )
}

/// AI 資料查詢路由（使用 AI API key 認證，獨立的 middleware）
pub fn ai_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ai/overview", get(handlers::ai::ai_system_overview))
        .route("/ai/schema", get(handlers::ai::ai_schema))
        .route("/ai/query", post(handlers::ai::ai_query))
        .route_layer(middleware::from_fn_with_state(
            state,
            ai_auth_middleware,
        ))
}
