// 真實 IP 提取中間件（SEC-30）
//
// 從已知的反向代理 header 中提取客戶端的真實 IP。
// 新增 trust_proxy 參數：
// - true: 信任 proxy header（適用於 Cloudflare Tunnel / nginx 後方部署）
// - false: 直接使用 socket addr（適用於直接面向外網的部署）

use axum::http::HeaderMap;
use std::net::SocketAddr;

/// 從 header 取值的輔助函式
fn get_header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 提取真實 IP（支援 trust_proxy 設定）
///
/// - `trust_proxy = true`：依序檢查 CF-Connecting-IP、X-Real-IP、X-Forwarded-For
/// - `trust_proxy = false`：僅使用 socket addr，忽略所有 proxy header（防偽造）
pub fn extract_real_ip(headers: &HeaderMap, fallback: &SocketAddr) -> String {
    extract_real_ip_with_trust(headers, fallback, true)
}

/// 提取真實 IP（含信任策略參數）
pub fn extract_real_ip_with_trust(
    headers: &HeaderMap,
    fallback: &SocketAddr,
    trust_proxy: bool,
) -> String {
    // SEC-30: 不信任 proxy header 時，直接使用 socket IP
    if !trust_proxy {
        return fallback.ip().to_string();
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn fallback() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 12345)
    }

    // === trust_proxy = false ===

    #[test]
    fn test_no_trust_ignores_all_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", "1.2.3.4".parse().unwrap());
        headers.insert("x-real-ip", "5.6.7.8".parse().unwrap());
        headers.insert("x-forwarded-for", "9.10.11.12".parse().unwrap());
        assert_eq!(
            extract_real_ip_with_trust(&headers, &fallback(), false),
            "192.168.1.100"
        );
    }

    // === trust_proxy = true: 優先級 ===

    #[test]
    fn test_cf_connecting_ip_highest_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", "1.1.1.1".parse().unwrap());
        headers.insert("x-real-ip", "2.2.2.2".parse().unwrap());
        headers.insert("x-forwarded-for", "3.3.3.3".parse().unwrap());
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "1.1.1.1");
    }

    #[test]
    fn test_x_real_ip_second_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "2.2.2.2".parse().unwrap());
        headers.insert("x-forwarded-for", "3.3.3.3".parse().unwrap());
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "2.2.2.2");
    }

    #[test]
    fn test_x_forwarded_for_first_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "10.0.0.1, 10.0.0.2, 10.0.0.3".parse().unwrap());
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "10.0.0.1");
    }

    #[test]
    fn test_x_forwarded_for_single() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.50".parse().unwrap());
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "203.0.113.50");
    }

    #[test]
    fn test_fallback_when_no_headers() {
        let headers = HeaderMap::new();
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "192.168.1.100");
    }

    #[test]
    fn test_empty_header_skipped() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", "".parse().unwrap());
        headers.insert("x-real-ip", "4.4.4.4".parse().unwrap());
        assert_eq!(extract_real_ip_with_trust(&headers, &fallback(), true), "4.4.4.4");
    }

    // === extract_real_ip (預設 trust wrapper) ===

    #[test]
    fn test_default_trusts_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "8.8.8.8".parse().unwrap());
        assert_eq!(extract_real_ip(&headers, &fallback()), "8.8.8.8");
    }

    #[test]
    fn test_whitespace_trimmed() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "  7.7.7.7  ".parse().unwrap());
        assert_eq!(extract_real_ip(&headers, &fallback()), "7.7.7.7");
    }
}
