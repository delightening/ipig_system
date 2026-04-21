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
    /// CRIT-03: 使用者權限快取，減少每次 API 請求對 DB 的 4-table JOIN。
    /// Key: user_id, Value: (permission_codes, cached_at)
    /// TTL 由 PERMISSION_CACHE_TTL_SECS 控制，角色異動時應呼叫 invalidate_permission_cache。
    pub permission_cache: std::sync::Arc<dashmap::DashMap<uuid::Uuid, (Vec<String>, std::time::Instant)>>,
    /// graceful shutdown 訊號：main 收到 SIGTERM/Ctrl+C 後 cancel，
    /// 所有背景任務（scheduler cron job、JwtBlacklist cleanup 等）觀測此 token 優雅收尾。
    pub shutdown_token: tokio_util::sync::CancellationToken,
}
