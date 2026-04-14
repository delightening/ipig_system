// 伺服器組裝與 Shutdown Signal 模組

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::config::Config;
use crate::AppState;

/// 依設定建立 CORS 層
pub fn build_cors(config: &Config) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    tracing::info!("[CORS] 允許的 Origin: {:?}", config.cors_allowed_origins);
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
}

/// 組裝 Axum Router，套用 CORS / Trace / 安全標頭 / 請求 ID 等 Middleware
pub fn build_app(state: AppState, config: &Config) -> Router {
    let cors = build_cors(config);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // SEC-27: API 層安全回應標頭（defense-in-depth）
    let nosniff = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let frame_deny = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let no_cache_api = SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );

    // SEC-AUDIT-011: Content-Security-Policy 防禦 XSS 與資料注入攻擊
    let csp = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"),
    );
    // SEC-AUDIT-015: 移除 Server header 防止洩漏框架資訊
    let referrer_policy = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let mut app = crate::routes::api_routes(state)
        .layer(cors)
        .layer(trace_layer)
        .layer(nosniff)
        .layer(frame_deny)
        .layer(no_cache_api)
        .layer(csp)
        .layer(referrer_policy)
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024)) // SEC-36: 全域 30MB 請求大小限制
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id());

    // R16-9: Production 環境不掛載 Swagger UI
    if !config.cookie_secure {
        tracing::info!("[Swagger] 開發模式：掛載 Swagger UI 於 /swagger-ui");
        app = app.merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url(
            "/api-docs/openapi.json",
            <crate::openapi::ApiDoc as utoipa::OpenApi>::openapi(),
        ));
    } else {
        tracing::info!("[Swagger] Production 模式：Swagger UI 已停用");
    }

    // R16-12: Production 環境啟用 HSTS header
    if config.cookie_secure {
        app = app.layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    app
}

/// Graceful Shutdown：監聽 Ctrl+C（及 Unix SIGTERM）
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, starting graceful shutdown..."),
        _ = terminate => tracing::info!("Received SIGTERM, starting graceful shutdown..."),
    }
}
