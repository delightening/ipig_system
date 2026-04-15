use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Method, Request, Response, StatusCode},
    middleware::Next,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use sqlx::PgPool;

use crate::constants::{
    AUTH_RATE_LIMIT_PER_MINUTE, API_RATE_LIMIT_PER_MINUTE, RATE_LIMIT_CLEANUP_INTERVAL_SECS,
    RATE_LIMIT_WINDOW_SECS, UPLOAD_RATE_LIMIT_PER_MINUTE, WRITE_RATE_LIMIT_PER_MINUTE,
    SEC_EVENT_RATE_LIMIT_AUTH, SEC_EVENT_RATE_LIMIT_API, SEC_EVENT_RATE_LIMIT_WRITE,
    SEC_EVENT_RATE_LIMIT_UPLOAD,
};
use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::services::AuditService;
use crate::AppState;

/// 速率限制器配置
#[derive(Clone)]
pub struct RateLimiterConfig {
    /// 時間窗口內允許的最大請求數
    pub max_requests: u32,
    /// 時間窗口長度
    pub window: Duration,
}

/// 共享的速率限制器狀態
#[derive(Clone)]
pub struct RateLimiterState {
    records: Arc<DashMap<String, Vec<Instant>>>,
    config: RateLimiterConfig,
}

impl RateLimiterState {
    pub fn new(config: RateLimiterConfig) -> Self {
        let records = Arc::new(DashMap::new());

        let cleanup_records = Arc::clone(&records);
        let cleanup_window = config.window;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(RATE_LIMIT_CLEANUP_INTERVAL_SECS)).await;
                let now = Instant::now();
                cleanup_records.retain(|_ip: &String, timestamps: &mut Vec<Instant>| {
                    timestamps.retain(|t| now.duration_since(*t) < cleanup_window);
                    !timestamps.is_empty()
                });
                // SEC-M3: 防止 HashMap 無限成長（DDoS 大量不同 IP）
                // 超過上限時清除最舊的條目
                const MAX_TRACKED_IPS: usize = 50_000;
                if cleanup_records.len() > MAX_TRACKED_IPS {
                    let overflow = cleanup_records.len() - MAX_TRACKED_IPS;
                    let keys_to_remove: Vec<String> = cleanup_records
                        .iter()
                        .take(overflow)
                        .map(|entry| entry.key().clone())
                        .collect();
                    for key in keys_to_remove {
                        cleanup_records.remove(&key);
                    }
                    tracing::warn!(
                        "[RateLimit] IP 追蹤數超過上限，已清除 {} 筆舊紀錄",
                        overflow
                    );
                }
            }
        });

        Self { records, config }
    }

    /// 檢查 IP 是否超過速率限制，回傳 (是否允許, 剩餘配額)
    fn check_rate(&self, ip: &str) -> (bool, u32) {
        let now = Instant::now();
        let window = self.config.window;

        let mut entry = self.records.entry(ip.to_string()).or_default();
        let timestamps = entry.value_mut();

        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() as u32 >= self.config.max_requests {
            (false, 0)
        } else {
            timestamps.push(now);
            let remaining = self.config.max_requests - timestamps.len() as u32;
            (true, remaining)
        }
    }
}

/// 認證端點速率限制中間件（嚴格：每分鐘 30 次）
pub async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    static AUTH_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
    let limiter = AUTH_LIMITER.get_or_init(|| {
        RateLimiterState::new(RateLimiterConfig {
            max_requests: AUTH_RATE_LIMIT_PER_MINUTE,
            window: Duration::from_secs(RATE_LIMIT_WINDOW_SECS),
        })
    });
    let ip = extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers);
    apply_rate_limit(limiter, &ip, "認證端點", SEC_EVENT_RATE_LIMIT_AUTH, Some(state.db.clone()), request, next).await
}

fn rate_limit_response(limiter: &RateLimiterState) -> Response<Body> {
    // HIGH-04: 使用實際設定的時間窗口，而非硬編碼 "60"
    let window_secs = limiter.config.window.as_secs();
    let body = serde_json::json!({
        "error": "Too Many Requests",
        "message": "請求過於頻繁，請稍後再試",
        "retry_after_seconds": window_secs
    });

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("Retry-After", window_secs.to_string())
        .header("X-RateLimit-Limit", limiter.config.max_requests.to_string())
        .header("X-RateLimit-Remaining", "0")
        .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        })
}

/// 共用速率限制執行邏輯（check → warn → response or next）
/// R22-1: 觸發時同步寫入 user_activity_logs（fire-and-forget）
async fn apply_rate_limit(
    limiter: &RateLimiterState,
    ip: &str,
    label: &str,
    event_type: &str,
    db: Option<PgPool>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let (allowed, remaining) = limiter.check_rate(ip);
    if !allowed {
        tracing::warn!(
            "[RateLimit] {} 速率限制觸發 - IP: {}, 限制: {}/min",
            label,
            ip,
            limiter.config.max_requests
        );

        // R22-1: 記錄 rate limit 事件到 DB
        if let Some(db) = db {
            let ip_owned = ip.to_string();
            let event_type_owned = event_type.to_string();
            let path = request.uri().path().to_string();
            let method = request.method().to_string();
            tokio::spawn(async move {
                if let Err(e) = AuditService::log_security_event(
                    &db,
                    &event_type_owned,
                    Some(&ip_owned),
                    None,
                    Some(&path),
                    Some(&method),
                    serde_json::json!({
                        "ip": ip_owned,
                        "tier": event_type_owned,
                    }),
                )
                .await
                {
                    tracing::error!("[R22] Failed to log rate limit event: {e}");
                }
            });
        }

        return Ok(rate_limit_response(limiter));
    }
    let mut response = next.run(request).await;
    if let Ok(val) = remaining.to_string().parse() {
        response.headers_mut().insert("X-RateLimit-Remaining", val);
    }
    Ok(response)
}

/// 一般 API 速率限制中間件（每分鐘 600 次）
pub async fn api_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    static API_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
    let limiter = API_LIMITER.get_or_init(|| {
        RateLimiterState::new(RateLimiterConfig {
            max_requests: API_RATE_LIMIT_PER_MINUTE,
            window: Duration::from_secs(RATE_LIMIT_WINDOW_SECS),
        })
    });
    let ip = extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers);
    apply_rate_limit(limiter, &ip, "API", SEC_EVENT_RATE_LIMIT_API, Some(state.db.clone()), request, next).await
}

/// 寫入端點速率限制（POST/PUT/PATCH/DELETE：每分鐘 120 次）
/// GET/HEAD/OPTIONS 直接放行
pub async fn write_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if matches!(request.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(request).await);
    }
    static WRITE_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
    let limiter = WRITE_LIMITER.get_or_init(|| {
        RateLimiterState::new(RateLimiterConfig {
            max_requests: WRITE_RATE_LIMIT_PER_MINUTE,
            window: Duration::from_secs(RATE_LIMIT_WINDOW_SECS),
        })
    });
    let ip = extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers);
    apply_rate_limit(limiter, &ip, "寫入端點", SEC_EVENT_RATE_LIMIT_WRITE, Some(state.db.clone()), request, next).await
}

/// 檔案上傳端點速率限制（每分鐘 30 次）
pub async fn upload_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    static UPLOAD_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
    let limiter = UPLOAD_LIMITER.get_or_init(|| {
        RateLimiterState::new(RateLimiterConfig {
            max_requests: UPLOAD_RATE_LIMIT_PER_MINUTE,
            window: Duration::from_secs(RATE_LIMIT_WINDOW_SECS),
        })
    });
    let ip = extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers);
    apply_rate_limit(limiter, &ip, "檔案上傳", SEC_EVENT_RATE_LIMIT_UPLOAD, Some(state.db.clone()), request, next).await
}
