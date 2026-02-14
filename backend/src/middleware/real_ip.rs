// 真實 IP 提取工具
// 從 proxy header 中按優先順序提取客戶端真實 IP
// 優先順序：CF-Connecting-IP > X-Real-IP > X-Forwarded-For > ConnectInfo

use axum::http::HeaderMap;
use std::net::SocketAddr;

/// 從 HTTP header 中提取真實客戶端 IP
/// 
/// 在反向代理（nginx、Cloudflare Tunnel）環境下，
/// `ConnectInfo<SocketAddr>` 拿到的是 proxy 的 IP（如 Docker 內部 IP），
/// 需要從 proxy 注入的 header 中提取真實 IP。
pub fn extract_real_ip(headers: &HeaderMap, fallback: &SocketAddr) -> String {
    // 1. Cloudflare Tunnel 注入的 header（最可信）
    if let Some(ip) = get_header_value(headers, "cf-connecting-ip") {
        return ip;
    }

    // 2. nginx 設定的 X-Real-IP ($remote_addr)
    if let Some(ip) = get_header_value(headers, "x-real-ip") {
        return ip;
    }

    // 3. X-Forwarded-For：取第一個 IP（最原始的客戶端）
    if let Some(forwarded) = get_header_value(headers, "x-forwarded-for") {
        if let Some(first_ip) = forwarded.split(',').next() {
            let trimmed = first_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // 4. Fallback：使用 socket 直連 IP
    fallback.ip().to_string()
}

/// 安全地取得 header 值
fn get_header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
