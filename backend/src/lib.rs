pub mod config;
mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod routes;
pub mod services;
pub mod startup;

pub use error::{AppError, Result};

pub use middleware::JwtBlacklist;
pub use services::GeoIpService;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: std::sync::Arc<config::Config>,
    pub geoip: GeoIpService,
    pub jwt_blacklist: JwtBlacklist,
    pub alert_broadcaster: handlers::sse::AlertBroadcaster,
    pub metrics_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
}
