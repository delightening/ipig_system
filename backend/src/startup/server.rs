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
    trace::{DefaultOnResponse, TraceLayer},
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

    // 自訂 make_span：保留 DefaultMakeSpan 的 method/uri/version，額外宣告 user_id 欄位
    // （初始 Empty，由認證中間件在驗證成功後 Span::current().record 填入），
    // 讓 ERROR / 5xx 日誌能對應「誰觸發」（見 PROGRESS 2026-06-03 改善計劃 §2）。
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &axum::http::Request<axum::body::Body>| {
            tracing::info_span!(
                "request",
                method = %req.method(),
                uri = %req.uri(),
                version = ?req.version(),
                user_id = tracing::field::Empty,
            )
        })
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
        // frame-src 'self' blob:：計畫書預覽以 <iframe> 內嵌前端取得的 PDF blob（瀏覽器原生 PDF viewer，正確分頁）。
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self'; frame-src 'self' blob:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"),
    );
    let referrer_policy = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    let cross_domain = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    );

    // F2（pentest 2026-07-05）：prod 的 /api-docs/openapi.json 改為需認證後才可取，
    // 具名身分才能枚舉完整 schema。state 先 clone 供該路由的 auth middleware 使用。
    let openapi_auth_state = state.clone();
    let router = crate::routes::api_routes(state);

    // R16-9 / F2：prod 停 Swagger UI，且 /api-docs/openapi.json 改為**需認證**後才可取——
    // 具名身分（登入 session / Bearer）才能枚舉完整 schema，擋匿名第三方 recon 攻擊面
    //（使用者決策：所有 AI 都要具名、零匿名探索）。搭配 nginx 對 /.well-known 探索檔與
    // /llms.txt 的收斂。
    //
    // 此 gated route 於**全域 layer chain 之前**併入 router，讓它與其他路由一起吃到下方的
    // CORS / 安全標頭 layer（axum 的 .layer 只作用於當下已註冊的路由，故 merge 順序有意義）。
    // dev 的 Swagger UI 因需寬鬆 CSP（inline script/style），改於 layer chain **之後**再 merge，
    // 避免被全域 strict `script-src 'self'` CSP 擋掉。
    let router = if config.cookie_secure {
        tracing::info!("[OpenAPI] Production 模式：/api-docs/openapi.json 需認證後存取（F2）");
        let spec = <crate::openapi::ApiDoc as utoipa::OpenApi>::openapi();
        let openapi_route = axum::Router::new()
            .route(
                "/api-docs/openapi.json",
                axum::routing::get(move || {
                    let spec = spec.clone();
                    async move { axum::Json(spec) }
                }),
            )
            .route_layer(axum::middleware::from_fn_with_state(
                openapi_auth_state,
                crate::middleware::auth_middleware,
            ));
        router.merge(openapi_route)
    } else {
        drop(openapi_auth_state); // dev 不 gate openapi；明確 drop 供 prod 用的 state clone
        router
    };

    let mut app = router
        .layer(cors)
        .layer(trace_layer)
        .layer(nosniff)
        .layer(frame_deny)
        .layer(no_cache_api)
        .layer(csp)
        .layer(referrer_policy)
        .layer(cross_domain)
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024)) // SEC-36: 全域 30MB 請求大小限制
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id());

    // dev：Swagger UI 於全域 layer chain 之後併入（見上，避開 strict CSP 破壞互動 UI）。
    if !config.cookie_secure {
        tracing::info!("[Swagger] 開發模式：掛載 Swagger UI 於 /swagger-ui");
        app = app.merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url(
            "/api-docs/openapi.json",
            <crate::openapi::ApiDoc as utoipa::OpenApi>::openapi(),
        ));
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
