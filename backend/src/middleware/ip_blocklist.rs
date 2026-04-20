//! R24-1: IP 黑名單 middleware
//!
//! 掛於 auth_middleware_stack 與 upload_middleware_stack 的最外層（ServiceBuilder 第一個 layer）。
//! 封 IP 請求於此 short-circuit，不消耗後續 CSRF / rate_limiter / handler CPU。
//! /metrics 與 /api/health 路由不掛此 middleware（監控探針信任網段）。

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::net::SocketAddr;

use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::services::IpBlocklistService;
use crate::AppState;

/// 451 Unavailable For Legal Reasons：語義上代表「因政策封鎖」。
/// 回傳 JSON body，方便前端辨識。
fn blocked_response() -> Response<Body> {
    let body = serde_json::json!({
        "error": "IP blocked",
        "message": "您的來源 IP 已被系統封鎖"
    });
    Response::builder()
        .status(StatusCode::from_u16(451).unwrap_or(StatusCode::FORBIDDEN))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&body).unwrap_or_else(|_| String::from("{}")),
        ))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        })
}

pub async fn ip_blocklist_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let ip_str = extract_real_ip_with_trust(
        request.headers(),
        &addr,
        state.config.trust_proxy_headers,
    );

    // 非法 IP 字串直接放行（middleware 不應卡主流量；實務上 real_ip 必回傳合法字串）
    let Ok(ip) = ip_str.parse::<std::net::IpAddr>() else {
        return next.run(request).await;
    };

    if IpBlocklistService::is_blocked(&state.db, ip).await {
        IpBlocklistService::spawn_record_hit(state.db.clone(), ip);
        tracing::warn!("[R24-1] Blocked request from {ip_str}");
        return blocked_response();
    }

    next.run(request).await
}
