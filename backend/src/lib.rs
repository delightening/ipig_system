pub mod config;
pub mod constants;
mod error;
pub mod time;
pub mod utils;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod startup;

pub use error::{AppError, Result};

pub use middleware::{ActorContext, JwtBlacklist, SYSTEM_USER_ID};
pub use services::GeoIpService;
pub use services::{GotenbergClient, ImageProcessorClient, PdfServiceClient, TemplateService};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: std::sync::Arc<config::Config>,
    pub geoip: GeoIpService,
    pub jwt_blacklist: JwtBlacklist,
    pub metrics_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
    pub gotenberg: GotenbergClient,
    pub image_processor: ImageProcessorClient,
    pub pdf_service: PdfServiceClient,
    pub templates: TemplateService,
    /// CRIT-03 / H2：使用者權限快取，減少每次 API 請求對 DB 的 4-table JOIN。
    /// Key: user_id, Value: 該使用者的 permission codes。
    ///
    /// H2 改 moka::future::Cache：內建 TTL（PERMISSION_CACHE_TTL_SECS, 5 分鐘）+
    /// `try_get_with` single-flight — cache miss 時保證僅一個 task 執行 loader，
    /// 其餘併發請求等同一個結果，避免 stampede 導致重複 4-table JOIN 浪費 DB 資源。
    /// loader 含 user 狀態檢查（is_active + expires_at），因此 cache hit 即代表
    /// 該使用者於 TTL 內狀態驗證 + 權限載入皆已完成。角色/權限異動時呼叫
    /// `invalidate(&user_id)` 或 `invalidate_all()`。
    pub permission_cache: moka::future::Cache<uuid::Uuid, Vec<String>>,
    /// graceful shutdown 訊號：main 收到 SIGTERM/Ctrl+C 後 cancel，
    /// 所有背景任務（scheduler cron job、JwtBlacklist cleanup 等）觀測此 token 優雅收尾。
    pub shutdown_token: tokio_util::sync::CancellationToken,
}

/// 統一建構 permission cache：main.rs / tests/common 共用，避免 TTL/capacity 漂移。
///
/// 配置：
/// - TTL：`constants::PERMISSION_CACHE_TTL_SECS`（5 分鐘）
/// - capacity：10,000 entries
///
/// 未來若新增 `eviction_listener` / `weigher` 等設定，集中此處避免兩端 builder
/// 漏更新（CodeRabbit review #210 採納）。
pub fn build_permission_cache() -> moka::future::Cache<uuid::Uuid, Vec<String>> {
    moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(
            constants::PERMISSION_CACHE_TTL_SECS,
        ))
        .max_capacity(10_000)
        .build()
}
