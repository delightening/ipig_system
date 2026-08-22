use axum::{
    body::Body,
    extract::{ConnectInfo, MatchedPath, State},
    http::{Method, Request, Response, StatusCode},
    middleware::Next,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use sqlx::PgPool;

use uuid::Uuid;

use crate::config::Config;
use crate::constants::{
    API_RATE_LIMIT_PER_MINUTE, AUTH_RATE_LIMIT_PER_MINUTE, FORGOT_PASSWORD_RATE_LIMIT,
    FORGOT_PASSWORD_RATE_WINDOW_SECS, RATE_LIMIT_CLEANUP_INTERVAL_SECS, RATE_LIMIT_WINDOW_SECS,
    SEC_EVENT_RATE_LIMIT_API, SEC_EVENT_RATE_LIMIT_AUTH, SEC_EVENT_RATE_LIMIT_FORGOT_PW,
    SEC_EVENT_RATE_LIMIT_UPLOAD, SEC_EVENT_RATE_LIMIT_WRITE, UPLOAD_RATE_LIMIT_PER_MINUTE,
    WRITE_RATE_LIMIT_PER_MINUTE,
};
use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::services::{
    AlertThresholdService, AuditService, IpBlocklistService, SecurityNotification, SecurityNotifier,
};
use crate::AppState;

/// Gemini #1: 安全事件記錄的 per-IP 頻率限制（同一 IP 每 60 秒最多記錄 1 次）
/// 避免大規模攻擊時 spawn 太多 DB 寫入導致資源耗盡
static SEC_LOG_THROTTLE: std::sync::LazyLock<DashMap<String, Instant>> =
    std::sync::LazyLock::new(DashMap::new);

fn should_log_security_event(ip: &str) -> bool {
    let now = Instant::now();
    let throttle_window = Duration::from_secs(60);
    if let Some(last) = SEC_LOG_THROTTLE.get(ip) {
        if now.duration_since(*last) < throttle_window {
            return false;
        }
    }
    SEC_LOG_THROTTLE.insert(ip.to_string(), now);
    // 防止 throttle map 無限成長
    if SEC_LOG_THROTTLE.len() > 10_000 {
        let keys: Vec<String> = SEC_LOG_THROTTLE
            .iter()
            .filter(|e| now.duration_since(*e.value()) > throttle_window)
            .take(1000)
            .map(|e| e.key().clone())
            .collect();
        for k in keys {
            SEC_LOG_THROTTLE.remove(&k);
        }
    }
    true
}

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
    // R35-14: auth tier 維持 IP-only — escalation 邏輯（check_auth_rate_limit_escalation）
    // 在 DB 端以 ip_address 聚合判定持續攻擊，per-pattern 切分會破壞此 signal。
    apply_rate_limit(
        limiter,
        &ip,
        &ip,
        "認證端點",
        SEC_EVENT_RATE_LIMIT_AUTH,
        Some(state.db.clone()),
        Some(state.config.clone()),
        request,
        next,
    )
    .await
}

/// E-5: 忘記密碼端點專屬速率限制（嚴格：5 次 / 10 分鐘，防 email flooding）
/// 不觸發 IP 封鎖升級（threat model 與暴力登入不同）
pub async fn forgot_password_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    static FORGOT_PW_LIMITER: OnceLock<RateLimiterState> = OnceLock::new();
    let limiter = FORGOT_PW_LIMITER.get_or_init(|| {
        RateLimiterState::new(RateLimiterConfig {
            max_requests: FORGOT_PASSWORD_RATE_LIMIT,
            window: Duration::from_secs(FORGOT_PASSWORD_RATE_WINDOW_SECS),
        })
    });
    let ip = extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers);
    // R35-14: forgot-password tier 路徑已限定為 password_reset_routes，本身就是 per-endpoint
    apply_rate_limit(
        limiter,
        &ip,
        &ip,
        "忘記密碼端點",
        SEC_EVENT_RATE_LIMIT_FORGOT_PW,
        Some(state.db.clone()),
        None,
        request,
        next,
    )
    .await
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

/// R35-14: 從 request.extensions 取出 axum router 已匹配的路徑模板。
/// 例如 `/api/v1/animals/123e4567-...` → `/api/v1/animals/:id`。
/// 若 middleware 在 router 完成 routing 前執行（極少見），回傳 None；
/// 呼叫端可 fallback 到 IP-only 限流。
fn matched_route_pattern(request: &Request<Body>) -> Option<&str> {
    request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
}

/// R35-14: 組 per-route rate-limit bucket key。
/// 形式 `ip|pattern` — 同一 IP 對不同 endpoint 有獨立配額。
/// pattern 缺省（router 未匹配）時退回純 IP key 維持原行為。
fn ip_route_key(ip: &str, request: &Request<Body>) -> String {
    match matched_route_pattern(request) {
        Some(pattern) => format!("{ip}|{pattern}"),
        None => ip.to_string(),
    }
}

/// 共用速率限制執行邏輯（check → warn → response or next）
/// R22-1: 觸發時同步寫入 user_activity_logs（fire-and-forget）
///
/// `bucket_key` 是 rate-limit 配額用的 key（可能是 `ip` 或 `ip|pattern`）；
/// `ip` 仍須單獨傳入用於 security event log + escalation throttle（這兩者
/// 都是 per-IP 不分 endpoint，不可改成 bucket_key）。
async fn apply_rate_limit(
    limiter: &RateLimiterState,
    bucket_key: &str,
    ip: &str,
    label: &str,
    event_type: &str,
    db: Option<PgPool>,
    config: Option<Arc<Config>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let (allowed, remaining) = limiter.check_rate(bucket_key);
    if !allowed {
        tracing::warn!(
            "[RateLimit] {} 速率限制觸發 - IP: {}, 限制: {}/min",
            label,
            ip,
            limiter.config.max_requests
        );

        // R22-1: 記錄 rate limit 事件到 DB（Gemini #1: per-IP 頻率限制）
        if let Some(db) = db {
            if !should_log_security_event(ip) {
                return Ok(rate_limit_response(limiter));
            }
            let ip_owned = ip.to_string();
            let event_type_owned = event_type.to_string();
            let path = request.uri().path().to_string();
            let method = request.method().to_string();
            tokio::spawn(async move {
                if let Err(e) = AuditService::log_security_event(
                    &db,
                    &event_type_owned,
                    None, // no actor for rate limit events
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

                // R22-5: Auth rate limit → escalation alert + dispatch notification
                if event_type_owned == SEC_EVENT_RATE_LIMIT_AUTH {
                    if let Err(e) =
                        check_auth_rate_limit_escalation(&db, &ip_owned, config.as_deref()).await
                    {
                        tracing::error!("[R22] Auth rate limit escalation check failed: {e}");
                    }
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
    // R35-14: api tier 是 /api/v1/* 整層 IP cap，作為 per-pattern 限流的 outer backstop —
    // 防 attacker 在 pattern 間 rotate 規避 per-pattern quota。維持 IP-only。
    apply_rate_limit(
        limiter,
        &ip,
        &ip,
        "API",
        SEC_EVENT_RATE_LIMIT_API,
        Some(state.db.clone()),
        None,
        request,
        next,
    )
    .await
}

/// 寫入端點速率限制（POST/PUT/PATCH/DELETE：每分鐘 120 次）
/// GET/HEAD/OPTIONS 直接放行
pub async fn write_rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if matches!(
        request.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
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
    // R35-14: write tier 改 per IP × matched route pattern。同一 IP 對不同 endpoint
    // 持有獨立配額（120/min each），防止單一熱門端點被打爆 → 全 tier 寫入癱瘓。
    let key = ip_route_key(&ip, &request);
    apply_rate_limit(
        limiter,
        &key,
        &ip,
        "寫入端點",
        SEC_EVENT_RATE_LIMIT_WRITE,
        Some(state.db.clone()),
        None,
        request,
        next,
    )
    .await
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
    // R35-14: upload tier 同 write tier，per IP × pattern keying
    let key = ip_route_key(&ip, &request);
    apply_rate_limit(
        limiter,
        &key,
        &ip,
        "檔案上傳",
        SEC_EVENT_RATE_LIMIT_UPLOAD,
        Some(state.db.clone()),
        None,
        request,
        next,
    )
    .await
}

/// R22-5: 檢查 auth rate limit 是否需要升級為 security_alert
async fn check_auth_rate_limit_escalation(
    pool: &PgPool,
    ip: &str,
    config: Option<&Config>,
) -> std::result::Result<(), sqlx::Error> {
    let threshold = AlertThresholdService::auth_rate_limit_threshold(pool).await;
    let window_mins = AlertThresholdService::auth_rate_limit_window_mins(pool).await;
    let dedup_mins = AlertThresholdService::alert_escalation_dedup_mins(pool).await;

    // Gemini #4: 加 partition_date 條件啟用 partition pruning
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_activity_logs
        WHERE event_type = 'RATE_LIMIT_AUTH'
          AND ip_address = $1::inet
          AND partition_date >= (NOW() - make_interval(mins => $2::integer))::date
          AND created_at > NOW() - make_interval(mins => $2::integer)
        "#,
    )
    .bind(ip)
    .bind(window_mins as i32)
    .fetch_one(pool)
    .await?;

    if count < threshold {
        return Ok(());
    }

    // Dedup: skip if recent open alert exists for this IP
    let (existing,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM security_alerts
        WHERE alert_type = 'rate_limit_escalation'
          AND context_data->>'ip' = $1
          AND created_at > NOW() - make_interval(mins => $2::integer)
          AND status = 'open'
        "#,
    )
    .bind(ip)
    .bind(dedup_mins as i32)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Ok(());
    }

    let alert_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            context_data, created_at, updated_at, status
        ) VALUES (
            $1, 'rate_limit_escalation', 'critical',
            '認證端點遭持續速率限制攻擊',
            $2, $3, NOW(), NOW(), 'open'
        )
        "#,
    )
    .bind(alert_id)
    .bind(format!(
        "IP {ip} 在過去 {window_mins} 分鐘內觸發認證速率限制 {count} 次"
    ))
    .bind(serde_json::json!({ "ip": ip, "count": count, "window_mins": window_mins }))
    .execute(pool)
    .await?;

    tracing::warn!("[R22-5] Auth rate limit escalation alert created for IP {ip}");

    // R24-1: 封 IP 1 小時（auth ratelimit 升級 = 持續攻擊）
    IpBlocklistService::auto_block(
        pool,
        ip,
        "R22-1_ratelimit",
        Some(alert_id),
        &format!("Auth rate limit 升級：IP {ip} 在 {window_mins} 分內觸發 {count} 次"),
        Some(1),
    )
    .await;

    // Dispatch notification
    if let Some(cfg) = config {
        let notification = SecurityNotification {
            alert_id,
            alert_type: "rate_limit_escalation".to_string(),
            severity: "critical".to_string(),
            title: "認證端點遭持續速率限制攻擊".to_string(),
            description: Some(format!(
                "IP {ip} 在過去 {window_mins} 分鐘內觸發認證速率限制 {count} 次"
            )),
            context_data: Some(serde_json::json!({ "ip": ip, "count": count })),
            created_at: chrono::Utc::now(),
        };
        SecurityNotifier::dispatch(pool, cfg, &notification).await;
    }

    Ok(())
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

        let (blocked, _) = limiter.check_rate("1.1.1.1");
        assert!(!blocked);

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

    /// R35-14: 同 IP 對不同 pattern 配額獨立 — 一個端點打爆不影響另一個。
    #[test]
    fn test_per_pattern_buckets_are_isolated() {
        let limiter = test_limiter(2);
        let key_a = "10.0.0.1|/api/v1/animals";
        let key_b = "10.0.0.1|/api/v1/protocols";

        // 把 endpoint A 打爆
        limiter.check_rate(key_a);
        limiter.check_rate(key_a);
        let (allowed_a, _) = limiter.check_rate(key_a);
        assert!(!allowed_a, "endpoint A 已達配額應該被擋");

        // endpoint B 不受影響
        let (allowed_b, remaining_b) = limiter.check_rate(key_b);
        assert!(allowed_b, "同 IP 的 endpoint B 仍應有獨立配額");
        assert_eq!(remaining_b, 1);
    }

    /// R35-14: 不同 IP 對相同 pattern 互不影響（既有行為 regression check）。
    #[test]
    fn test_same_pattern_different_ips_isolated() {
        let limiter = test_limiter(1);
        let (allowed1, _) = limiter.check_rate("1.1.1.1|/api/v1/animals");
        assert!(allowed1);
        let (allowed2, _) = limiter.check_rate("2.2.2.2|/api/v1/animals");
        assert!(allowed2, "不同 IP 即使打同一 pattern 也應各自有配額");
    }
}
