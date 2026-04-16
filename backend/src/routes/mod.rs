use axum::{middleware, Router};
use tower::ServiceBuilder;

use crate::middleware::etag::etag_middleware;
use crate::middleware::rate_limiter::{
    api_rate_limit_middleware, auth_rate_limit_middleware, upload_rate_limit_middleware,
    write_rate_limit_middleware,
};
use crate::{
    handlers,
    middleware::{auth_middleware, csrf_middleware, guest_guard_middleware, security_response_logger},
    AppState,
};

mod admin;
mod ai;
mod animal;
mod auth;
mod erp;
mod honeypot;
mod hr;
mod invitation;
mod mcp;
mod notification;
mod protocol;
mod report;
mod upload;
mod user;

pub fn api_routes(state: AppState) -> Router {
    let public_routes = auth::public_routes()
        .merge(invitation::public_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        ))
        .with_state(state.clone());

    // 使用 ServiceBuilder 合併 middleware 層（避免 .route_layer 逐路由包裝導致記憶體暴漲）
    // R22-3: security_response_logger 必須放在 auth_middleware 內層（最後一個 .layer()），
    // 確保 auth_middleware 先執行並設定 CurrentUser，logger 才能讀取到 user_id。
    // ServiceBuilder 中第一個 .layer() 是最外層（最先執行），最後一個是最內層（最後執行）。
    let auth_middleware_stack = ServiceBuilder::new()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            write_rate_limit_middleware,
        ))
        .layer(middleware::from_fn(guest_guard_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_response_logger,
        ));

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
        .layer(auth_middleware_stack)
        .with_state(state.clone());

    let upload_middleware_stack = ServiceBuilder::new()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            upload_rate_limit_middleware,
        ))
        .layer(middleware::from_fn(guest_guard_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let upload_routes = upload::routes()
        .layer(upload_middleware_stack)
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

    // MCP 路由（使用個人 MCP API Key 認證，不走 JWT / CSRF）
    let mcp_routes = mcp::routes().with_state(state.clone());

    // R22-16: 蜜罐端點（/api/v1 外層，不需 auth）
    let honeypot_routes = honeypot::routes().with_state(state.clone());

    let api_middleware_stack = ServiceBuilder::new()
        .layer(middleware::from_fn(etag_middleware))
        .layer(middleware::from_fn_with_state(
            state,
            api_rate_limit_middleware,
        ));

    // P1-M1: API 版本路徑 — /api/v1 為唯一前綴
    let api_v1 = public_routes
        .merge(protected_routes)
        .merge(upload_routes)
        .merge(ai_query_routes)
        .merge(mcp_routes);

    health_route
        .merge(honeypot_routes)
        .merge(
            Router::new()
                .nest("/api/v1", api_v1)
                .layer(api_middleware_stack),
        )
}
