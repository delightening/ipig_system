use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
};
use sqlx;
use tower_http::{
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod startup;

use services::scheduler::SchedulerService;
use services::GeoIpService;
use middleware::JwtBlacklist;
use startup::{
    create_database_pool_with_retry,
    ensure_admin_user, ensure_schema, seed_dev_users,
    ensure_required_permissions, ensure_all_role_permissions,
};

pub use error::{AppError, Result};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<config::Config>,
    pub geoip: GeoIpService,
    /// JWT 黑名單，用於主動撤銷已簽發的 token（SEC-23）
    pub jwt_blacklist: JwtBlacklist,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 載入環境變數
    dotenvy::dotenv().ok();

    // 初始化 tracing 日誌
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "erp_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 載入設定
    let config = config::Config::from_env()?;
    let config = Arc::new(config);

    // 建立資料庫連線池（含重試機制）
    let pool = match create_database_pool_with_retry(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(
                "\n╔═══════════════════════════════════════════════════════════════╗\n\
                 ║              API STARTUP FAILED - DATABASE ERROR                ║\n\
                 ╠═══════════════════════════════════════════════════════════════╣\n\
                 ║ The API server cannot start because database connection       ║\n\
                 ║ failed. Please check the error messages above for details.    ║\n\
                 ║                                                                 ║\n\
                 ║ Database Status: ❌ UNAVAILABLE                                ║\n\
                 ║ Error: {}                                                       ║\n\
                 ╚═══════════════════════════════════════════════════════════════╝",
                e
            );
            return Err(e);
        }
    };

    // 執行資料庫 migration
    tracing::info!("[Database] Running migrations...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {
            tracing::info!("[Database] ✓ Migrations completed successfully");
        }
        Err(e) => {
            tracing::error!(
                "\n╔═══════════════════════════════════════════════════════════════╗\n\
                 ║           API STARTUP FAILED - MIGRATION ERROR                   ║\n\
                 ╠═══════════════════════════════════════════════════════════════╣\n\
                 ║ Database connection: ✓ ESTABLISHED                              ║\n\
                 ║ Database migrations: ❌ FAILED                                 ║\n\
                 ║ Error: {}                                                       ║\n\
                 ╚═══════════════════════════════════════════════════════════════╝",
                e
            );
            return Err(anyhow::anyhow!("Database migration failed: {}", e));
        }
    }

    tracing::info!("[Database] ✓ Connection established and migrations completed");

    // 啟動時初始化（非致命錯誤不中斷啟動）
    if let Err(e) = ensure_schema(&pool).await {
        tracing::warn!("Failed to ensure schema (non-fatal): {}", e);
    }

    if let Err(e) = ensure_admin_user(&pool).await {
        tracing::warn!("Failed to ensure admin user (non-fatal): {}", e);
    }

    if let Err(e) = ensure_required_permissions(&pool).await {
        tracing::warn!("Failed to ensure required permissions (non-fatal): {}", e);
    }

    if let Err(e) = ensure_all_role_permissions(&pool).await {
        tracing::warn!("Failed to ensure role permissions (non-fatal): {}", e);
    }

    if config.seed_dev_users {
        tracing::info!("[DevUser] SEED_DEV_USERS is enabled, seeding development users...");
        if let Err(e) = seed_dev_users(&pool).await {
            tracing::warn!("Failed to seed dev users (non-fatal): {}", e);
        }
    }

    // SEC-22: 啟動安全配置檢查
    if !config.cookie_secure {
        tracing::warn!(
            "╔════════════════════════════════════════════════════════════╗\n\
             ║  ⚠️  COOKIE_SECURE=false - Token 將在明文 HTTP 中傳送    ║\n\
             ║     正式環境請設定 COOKIE_SECURE=true 並啟用 HTTPS       ║\n\
             ╚════════════════════════════════════════════════════════════╝"
        );
    }
    if config.seed_dev_users {
        tracing::warn!(
            "╔════════════════════════════════════════════════════════════╗\n\
             ║  ⚠️  SEED_DEV_USERS=true - 已啟用開發測試帳號            ║\n\
             ║     正式環境請確保此設定為 false                          ║\n\
             ╚════════════════════════════════════════════════════════════╝"
        );
    }
    // SEC-26: 正式環境（COOKIE_SECURE=true）禁止啟用開發帳號
    if config.seed_dev_users && config.cookie_secure {
        tracing::error!(
            "╔════════════════════════════════════════════════════════════╗\n\
             ║  ❌  COOKIE_SECURE=true 但 SEED_DEV_USERS=true            ║\n\
             ║     正式環境不得啟用開發帳號！拒絕啟動。                    ║\n\
             ╚════════════════════════════════════════════════════════════╝"
        );
        return Err(anyhow::anyhow!("SEC-26: 正式環境不得啟用 SEED_DEV_USERS"));
    }

    // 啟動背景排程服務
    let scheduler_result = SchedulerService::start(pool.clone(), config.clone()).await;
    match scheduler_result {
        Ok(_scheduler) => {
            tracing::info!("Background scheduler started");
        }
        Err(e) => {
            tracing::warn!("Failed to start scheduler (non-fatal): {}", e);
        }
    }

    // 建立應用程式狀態
    // 初始化 GeoIP 服務
    let geoip_path = std::env::var("GEOIP_DB_PATH")
        .unwrap_or_else(|_| "/app/geoip/GeoLite2-City.mmdb".to_string());
    let geoip = GeoIpService::new(&geoip_path);

    // 初始化 JWT 黑名單（SEC-23）
    let jwt_blacklist = JwtBlacklist::new();

    let state = AppState {
        db: pool,
        config: config.clone(),
        geoip,
        jwt_blacklist,
    };

    // 建立 CORS 層
    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:8080"),
            HeaderValue::from_static("http://10.0.4.34:8080"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::HeaderName::from_static("x-csrf-token")])
        .allow_credentials(true);

    // 建立 Trace 層
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

    // 組裝 Router
    let app = routes::api_routes(state)
        .layer(cors)
        .layer(trace_layer)
        .layer(nosniff)
        .layer(frame_deny)
        .layer(no_cache_api);

    // 啟動伺服器
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;

    Ok(())
}
