use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use erp_backend::config;
use erp_backend::middleware::JwtBlacklist;
use erp_backend::services::scheduler::SchedulerService;
use erp_backend::services::{AuditService, FileService, GeoIpService, GotenbergClient, ImageProcessorClient, PdfServiceClient, TemplateService};
use erp_backend::startup::{
    check_jwt_key_file_permissions, create_database_pool_with_retry, ensure_admin_user,
    ensure_all_role_permissions, ensure_required_permissions, ensure_schema, init_tracing,
    log_startup_config_check, run_migrations, seed_dev_users,
};
use erp_backend::startup::server::{build_app, shutdown_signal};
use erp_backend::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = config::Config::from_env()?;
    let config = Arc::new(config);

    // 資料庫連線（含重試）
    let pool = create_database_pool_with_retry(&config).await.map_err(|e| {
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
        e
    })?;

    run_migrations(&pool, &config).await?;
    tracing::info!("[Database] ✓ Connection established and migrations completed");

    // 啟動時初始化（非致命）
    if let Err(e) = ensure_schema(&pool).await {
        tracing::warn!("Failed to ensure schema (non-fatal): {}", e);
    }
    if let Err(e) = ensure_admin_user(&pool, &config).await {
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
        if let Err(e) = seed_dev_users(&pool, &config).await {
            tracing::warn!("Failed to seed dev users (non-fatal): {}", e);
        }
    }

    // SEC-22 / SEC-26: 安全配置檢查
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
    if config.seed_dev_users && config.cookie_secure {
        tracing::error!(
            "╔════════════════════════════════════════════════════════════╗\n\
             ║  ❌  COOKIE_SECURE=true 但 SEED_DEV_USERS=true            ║\n\
             ║     正式環境不得啟用開發帳號！拒絕啟動。                    ║\n\
             ╚════════════════════════════════════════════════════════════╝"
        );
        return Err(anyhow::anyhow!("SEC-26: 正式環境不得啟用 SEED_DEV_USERS"));
    }

    log_startup_config_check(&config);
    // H7：JWT 私鑰檔權限檢查（檔案模式提供時）
    check_jwt_key_file_permissions();

    // 全域 graceful shutdown 訊號：所有背景任務觀測此 token 優雅收尾
    let shutdown_token = CancellationToken::new();

    // 背景排程服務（必須保留到 server 關閉）
    let _scheduler = match SchedulerService::start(
        pool.clone(),
        config.clone(),
        shutdown_token.clone(),
    )
    .await
    {
        Ok(sched) => {
            tracing::info!("Background scheduler started");
            Some(sched)
        }
        Err(e) => {
            tracing::warn!("Failed to start scheduler (non-fatal): {}", e);
            None
        }
    };

    // 靜態服務初始化
    FileService::init_upload_dir(&config.upload_dir);
    AuditService::init_hmac_key(config.audit_hmac_key.clone());

    let geoip = GeoIpService::new(&config.geoip_db_path);

    // JWT 黑名單（SEC-23 + SEC-33）
    let jwt_blacklist = JwtBlacklist::new();
    jwt_blacklist.load_from_db(&pool).await;
    let jwt_cleanup_handle = jwt_blacklist
        .clone()
        .start_cleanup_task(pool.clone(), shutdown_token.clone());

    // SEC-30
    if config.trust_proxy_headers {
        tracing::info!("[Security] TRUST_PROXY_HEADERS=true - 信任反向代理 header");
    } else {
        tracing::info!(
            "[Security] TRUST_PROXY_HEADERS=false - 僅使用 socket IP（已忽略 proxy header）"
        );
    }

    // Prometheus 指標收集器
    let metrics_handle =
        match metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder() {
            Ok(handle) => {
                tracing::info!("✅ Prometheus 指標收集器已啟動");
                Some(handle)
            }
            Err(e) => {
                tracing::warn!("⚠️ Prometheus 指標收集器初始化失敗: {}", e);
                None
            }
        };

    // 初始化 PDF 模板引擎
    let templates = TemplateService::new().expect("Failed to initialize PDF template service");

    // 初始化 Gotenberg PDF 生成服務
    let gotenberg = GotenbergClient::new(&config.gotenberg_url);

    // 初始化 Image Processor 微服務（方案 D）
    let image_processor = ImageProcessorClient::new(&config.image_processor_url);

    // 初始化 PDF Service 微服務（FastAPI + Jinja2 + Gotenberg）
    let pdf_service = PdfServiceClient::new(&config.pdf_service_url, &config.pdf_service_token);

    let state = AppState {
        db: pool,
        config: config.clone(),
        geoip,
        jwt_blacklist,
        metrics_handle,
        gotenberg,
        image_processor,
        pdf_service,
        templates,
        permission_cache: std::sync::Arc::new(dashmap::DashMap::new()),
        shutdown_token: shutdown_token.clone(),
    };

    let app = build_app(state, &config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    // shutdown 流程：接到訊號 → cancel token → axum 停收新連線 → 等背景任務收尾
    let shutdown_token_for_signal = shutdown_token.clone();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        tracing::info!("Shutdown signal received — cancelling background tasks");
        shutdown_token_for_signal.cancel();
    })
    .await?;

    // 等背景任務收尾（timeout 保底避免永久卡死）
    //
    // 目前只等 jwt_blacklist cleanup join handle。Scheduler 的 cron job 在
    // shutdown_token.cancel() 後跳過下一輪觸發，正在執行中的 job 會跑完當輪
    // 才結束（`is_cancelled()` 在 job body 開頭檢查）。執行中 job 的 join 留給
    // tokio runtime 自動 drop 處理。
    //
    // 長 job 的明確 shutdown 協調（select! 中斷 + scheduler grace period）留待
    // R26-1 升級，屆時此處會加 `scheduler.shutdown().await` 等 in-flight
    // cron runtime 結束。
    const JWT_CLEANUP_TIMEOUT: Duration = Duration::from_secs(10);
    tracing::info!(
        "Waiting for jwt_blacklist cleanup to finish (up to {}s)...",
        JWT_CLEANUP_TIMEOUT.as_secs()
    );
    match tokio::time::timeout(JWT_CLEANUP_TIMEOUT, jwt_cleanup_handle).await {
        Ok(Ok(())) => tracing::info!("jwt_blacklist cleanup joined cleanly"),
        Ok(Err(e)) => tracing::warn!("jwt_blacklist cleanup task panicked: {}", e),
        Err(_) => tracing::warn!(
            "jwt_blacklist cleanup did not finish within {}s — forcing shutdown",
            JWT_CLEANUP_TIMEOUT.as_secs()
        ),
    }

    tracing::info!("Server shut down gracefully");
    Ok(())
}
