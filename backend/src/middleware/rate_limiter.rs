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

use crate::constants::{
    AUTH_RATE_LIMIT_PER_MINUTE, API_RATE_LIMIT_PER_MINUTE, RATE_LIMIT_CLEANUP_INTERVAL_SECS,
    RATE_LIMIT_WINDOW_SECS, UPLOAD_RATE_LIMIT_PER_MINUTE, WRITE_RATE_LIMIT_PER_MINUTE,
};
use crate::middleware::real_ip::extract_real_ip_with_trust;
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
                cleanup_records.retain(|_ip, timestamps: &mut Vec<Instant>| {
                    timestamps.retain(|t| now.duration_since(*t) < cleanup_window);
                    !timestamps.is_empty()
                });
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
    apply_rate_limit(limiter, &ip, "認證端點", request, next).await
}

fn rate_limit_response(limiter: &RateLimiterState) -> Response<Body> {
    let body = serde_json::json!({
        "error": "Too Many Requests",
        "message": "請求過於頻繁，請稍後再試",
        "retry_after_seconds": 60
    });

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("Retry-After", "60")
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
async fn apply_rate_limit(
    limiter: &RateLimiterState,
    ip: &str,
    label: &str,
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
    apply_rate_limit(limiter, &ip, "API", request, next).await
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
    apply_rate_limit(limiter, &ip, "寫入端點", request, next).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_limiter(max_requests: u32) -> RateLimiterState {
        // Create without spawning the cleanup task for tests
        RateLimiterState {
            records: Arc::new(DashMap::new()),
            config: RateLimiterConfig {
                max_requests,
                window: Duration::from_secs(60),
            },
        }
    }

    #[test]
    fn test_check_rate_allows_under_limit() {
        let limiter = test_limiter(5);
        let (allowed, remaining) = limiter.check_rate("192.168.1.1");
        assert!(allowed);
        assert_eq!(remaining, 4);
    }

    #[test]
    fn test_check_rate_decrements_remaining() {
        let limiter = test_limiter(3);
        let (_, remaining) = limiter.check_rate("10.0.0.1");
        assert_eq!(remaining, 2);

        let (_, remaining) = limiter.check_rate("10.0.0.1");
        assert_eq!(remaining, 1);

        let (_, remaining) = limiter.check_rate("10.0.0.1");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_check_rate_blocks_at_limit() {
        let limiter = test_limiter(2);
        limiter.check_rate("10.0.0.1");
        limiter.check_rate("10.0.0.1");

        let (allowed, remaining) = limiter.check_rate("10.0.0.1");
        assert!(!allowed);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_check_rate_isolates_ips() {
        let limiter = test_limiter(1);
        let (allowed1, _) = limiter.check_rate("1.1.1.1");
        assert!(allowed1);

        let (allowed2, _) = limiter.check_rate("2.2.2.2");
        assert!(allowed2);

        // First IP is now blocked
        let (blocked, _) = limiter.check_rate("1.1.1.1");
        assert!(!blocked);

        // Second IP is also blocked
        let (blocked2, _) = limiter.check_rate("2.2.2.2");
        assert!(!blocked2);
    }

    #[test]
    fn test_check_rate_single_request_limit() {
        let limiter = test_limiter(1);
        let (allowed, remaining) = limiter.check_rate("ip");
        assert!(allowed);
        assert_eq!(remaining, 0);

        let (blocked, _) = limiter.check_rate("ip");
        assert!(!blocked);
    }
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
    apply_rate_limit(limiter, &ip, "檔案上傳", request, next).await
}
