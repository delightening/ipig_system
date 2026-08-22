use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use erp_backend::config;
use erp_backend::middleware::JwtBlacklist;
use erp_backend::services::scheduler::SchedulerService;
use erp_backend::services::{AuditService, FileService, GeoIpService, PdfServiceClient};
use erp_backend::startup::server::{build_app, shutdown_signal};
use erp_backend::startup::{
    check_jwt_key_file_permissions, create_database_pool_with_retry, ensure_admin_user,
    ensure_all_role_permissions, ensure_required_permissions, ensure_schema, init_tracing,
    is_production, log_startup_config_check, run_db_self_test, run_migrations, seed_dev_users,
};
use erp_backend::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = config::Config::from_env()?;
    config.warn_if_idle_window_unusable();
    let config = Arc::new(config);

    // 資料庫連線（含重試）
    let pool = create_database_pool_with_retry(&config)
        .await
        .map_err(|e| {
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

    // R30-23：config 檢查回傳 warn_count，本檔依環境決定 fail-fast。
    // 把 exit 邏輯留在 main.rs 與 run_db_self_test 一致；config_check 純印 + 回傳，
    // 整合測試也能直接斷言 warn_count 而不會中斷 runner。
    let config_warn_count = log_startup_config_check(&config);
    if config_warn_count > 0 && is_production() {
        tracing::error!(
            "[R30-23] production 環境啟動前必須清掉所有 config 警告（{} 項）。\
             退出（exit code 1）。設 APP_ENV=staging 或修正配置可避開。",
            config_warn_count
        );
        std::process::exit(1);
    }
    // H7：JWT 私鑰檔權限檢查（檔案模式提供時）
    check_jwt_key_file_permissions(config.jwt_ec_private_key_file.as_deref());

    // R30-24：DB schema / role / permission self-test。
    // 啟動成功 ≠ DB 正確；備份還原 / migration 漏跑 / seed 失敗等情境會讓
    // 後端啟動但業務 broken。Production 任一檢查失敗 → fail-fast。
    let self_test_failures = run_db_self_test(&pool).await.unwrap_or_else(|e| {
        tracing::error!("[R30-24] DB self-test query failed: {}", e);
        // query 本身失敗（DB 連不上 / 表不存在）視為一項 failure
        1
    });
    if self_test_failures > 0 && is_production() {
        tracing::error!(
            "[R30-24] production 環境啟動前必須通過 DB self-test（{} 項失敗）。退出。",
            self_test_failures
        );
        std::process::exit(1);
    }

    // 全域 graceful shutdown 訊號：所有背景任務觀測此 token 優雅收尾
    let shutdown_token = CancellationToken::new();

    // 統一派送層渲染通知 email 用的全域 app_url（dispatch_event 對 email channel 使用）。
    // 須在啟動背景排程服務之前設定，避免啟動期 job 提早觸發 dispatch_event 時 email 被略過。
    erp_backend::services::init_notification_app_url(config.app_url.clone());

    // 背景排程服務（必須保留到 server 關閉）
    let _scheduler =
        match SchedulerService::start(pool.clone(), config.clone(), shutdown_token.clone()).await {
            Ok(sched) => {
                tracing::info!("Background scheduler started");
                Some(sched)
            }
            Err(e) => {
                tracing::warn!("Failed to start scheduler (non-fatal): {}", e);
                None
            }
        };

    // 台灣國定假日服務（員工通知寄送時間窗用）：建構 → 設為全域 → 暖機 + 每日刷新。
    // 失敗非致命：全域未設時，時間窗退化為僅週末判斷（見 services/holiday）。
    match erp_backend::services::holiday::HolidayService::from_settings(&pool).await {
        Ok(svc) => {
            let svc = std::sync::Arc::new(svc);
            erp_backend::services::holiday::init_global(svc.clone());
            let holiday_token = shutdown_token.clone();
            tokio::spawn(async move {
                use chrono::Datelike;
                loop {
                    // refresh_year 強制重抓並覆寫；失敗保留原快取（不清空），避免 API 故障時退化。
                    let year = erp_backend::time::now_taiwan().year();
                    svc.refresh_year(year).await;
                    svc.refresh_year(year + 1).await;
                    tokio::select! {
                        _ = holiday_token.cancelled() => break,
                        _ = tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)) => {}
                    }
                }
            });
            tracing::info!("台灣國定假日服務已初始化（每日刷新）");
        }
        Err(e) => tracing::warn!(
            "國定假日服務初始化失敗（員工通知時間窗將退化為僅週末判斷）: {}",
            e
        ),
    }

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
    // R28-M5：init 失敗時用 tracing::error（高可見度），且 health check
    // 會回 503 degraded（見 handlers/health.rs metrics 欄位）。
    // 不 fail-fast 是為了讓核心服務（API + DB）仍可運作；觀測能力降級
    // 但業務功能不受影響。
    let metrics_handle =
        match metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder() {
            Ok(handle) => {
                tracing::info!("✅ Prometheus 指標收集器已啟動");
                Some(handle)
            }
            Err(e) => {
                tracing::error!(
                    "❌ Prometheus 指標收集器初始化失敗: {} \
                     (所有 metrics::* 呼叫將靜默；/api/health 將回 503 degraded)",
                    e
                );
                None
            }
        };

    // R39 後 backend 不再渲染 HTML — print-pdf 統一管理範本（HTML / docx / xlsx）。
    let pdf_service = PdfServiceClient::new(&config.pdf_service_url, &config.pdf_service_token);

    let state = AppState {
        db: pool,
        config: config.clone(),
        geoip,
        jwt_blacklist,
        metrics_handle,
        pdf_service,
        // H2：moka::future::Cache 取代 DashMap，內建 TTL + try_get_with single-flight
        permission_cache: erp_backend::build_permission_cache(),
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
