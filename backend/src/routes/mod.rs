use axum::{middleware, Router};

use crate::middleware::etag::etag_middleware;
use crate::middleware::rate_limiter::{
    api_rate_limit_middleware, auth_rate_limit_middleware, upload_rate_limit_middleware,
    write_rate_limit_middleware,
};
use crate::{handlers, middleware::auth_middleware, middleware::csrf_middleware, AppState};

mod admin;
mod ai;
mod animal;
mod auth;
mod erp;
mod hr;
mod invitation;
mod notification;
mod protocol;
mod report;
mod upload;
mod user;

pub fn api_routes(state: AppState) -> Router {
    let public_routes = auth::public_routes()
        .merge(invitation::public_routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        ))
        .with_state(state.clone());

    let protected_routes = auth::protected_routes()
        .merge(user::routes())
        .merge(erp::routes())
        .merge(report::routes())
        .merge(protocol::routes())
        .merge(animal::routes())
        .merge(notification::routes())
        .merge(admin::routes())
        .merge(hr::routes())
        .merge(invitation::admin_routes())
        .merge(ai::admin_routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            write_rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    let upload_routes = upload::routes()
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            upload_rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // 健康檢查 + 指標路由（不受 Rate Limiter 影響，確保監控系統可探測）
    let health_route = Router::new()
        .route("/api/health", axum::routing::get(handlers::health::health_check))
        .route("/metrics", axum::routing::get(handlers::metrics::metrics_handler))
        .route(
            "/api/metrics/vitals",
            axum::routing::post(handlers::metrics::vitals_handler),
        )
        .with_state(state.clone());

    // AI 查詢路由（使用獨立的 AI API key 認證，不走 JWT / CSRF）
    let ai_query_routes = ai::ai_routes(state.clone())
        .with_state(state.clone());

    // P1-M1: API 版本路徑 — 引入 /api/v1/ 前綴；/api 保留為 deprecated 向後相容
    let api_v1 = public_routes
        .merge(protected_routes)
        .merge(upload_routes)
        .merge(ai_query_routes);
    health_route.merge(
        Router::new()
            .nest("/api/v1", api_v1.clone())
            .nest("/api", api_v1)
            .route_layer(middleware::from_fn(etag_middleware))
            .route_layer(middleware::from_fn_with_state(
                state,
                api_rate_limit_middleware,
            )),
    )
}
