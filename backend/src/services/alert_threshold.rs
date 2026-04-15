//! R22-8: Security alert threshold configuration
//!
//! Reads threshold values from `security_alert_config` table with 60-second in-memory cache.

use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Cached alert thresholds
type ThresholdCache = Option<(HashMap<String, String>, Instant)>;
static CACHE: std::sync::LazyLock<Mutex<ThresholdCache>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

const CACHE_TTL: Duration = Duration::from_secs(60);

pub struct AlertThresholdService;

impl AlertThresholdService {
    /// Get a threshold value by key, with 60s cache. Returns default if key not found.
    async fn get(pool: &PgPool, key: &str, default: i64) -> i64 {
        if let Ok(guard) = CACHE.lock() {
            if let Some((map, ts)) = guard.as_ref() {
                if ts.elapsed() < CACHE_TTL {
                    return map
                        .get(key)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(default);
                }
            }
        }

        // Cache miss — reload from DB
        let rows: Vec<(String, String)> =
            match sqlx::query_as("SELECT key, value FROM security_alert_config")
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    tracing::warn!("[R22] Failed to load alert thresholds: {e}");
                    return default;
                }
            };

        let map: HashMap<String, String> = rows.into_iter().collect();
        let result = map
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default);

        if let Ok(mut guard) = CACHE.lock() {
            *guard = Some((map, Instant::now()));
        }

        result
    }

    pub async fn auth_rate_limit_threshold(pool: &PgPool) -> i64 {
        Self::get(pool, "auth_rate_limit_threshold", 10).await
    }

    pub async fn auth_rate_limit_window_mins(pool: &PgPool) -> i64 {
        Self::get(pool, "auth_rate_limit_window_mins", 5).await
    }

    pub async fn idor_403_threshold(pool: &PgPool) -> i64 {
        Self::get(pool, "idor_403_threshold", 20).await
    }

    pub async fn idor_403_window_mins(pool: &PgPool) -> i64 {
        Self::get(pool, "idor_403_window_mins", 5).await
    }

    pub async fn alert_escalation_dedup_mins(pool: &PgPool) -> i64 {
        Self::get(pool, "alert_escalation_dedup_mins", 30).await
    }
}
