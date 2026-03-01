//! ETag 快取中介層
//!
//! 僅套用到讀取型 GET API：
//! - 計算 response body 的 SHA-256 作為 ETag
//! - 比對 request 的 If-None-Match
//! - 匹配則回傳 304 Not Modified
//! - 排除 /api/auth/*、/api/health、/api/metrics/*

use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, HeaderValue, Response, StatusCode},
    middleware::Next,
};
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};

/// 不套用 ETag 的路徑前綴
fn is_excluded_path(path: &str) -> bool {
    path.starts_with("/api/auth")
        || path.starts_with("/api/v1/auth")
        || path == "/api/health"
        || path.starts_with("/api/metrics")
        || path.starts_with("/api/v1/metrics")
        || path == "/metrics"
}

/// ETag 中介層：僅對 GET 且非排除路徑的 200 回應計算 ETag、支援 304
pub async fn etag_middleware(request: Request, next: Next) -> Response<Body> {
    if request.method() != Method::GET {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();
    if is_excluded_path(&path) {
        return next.run(request).await;
    }

    let if_none_match = request
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let response = next.run(request).await;
    let (mut parts, body) = response.into_parts();

    // 僅對 200 OK 且有 body 的回應計算 ETag
    if parts.status != StatusCode::OK {
        return Response::from_parts(parts, body);
    }

    let Ok(collected) = body.collect().await else {
        return Response::from_parts(parts, Body::empty());
    };

    let bytes = collected.to_bytes();
    if bytes.is_empty() {
        return Response::from_parts(parts, Body::from(bytes));
    }

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let etag = format!("\"{:x}\"", hash);

    // 比對 If-None-Match（支援 "etag" 或 "etag1", "etag2" 格式）
    if let Some(imm) = if_none_match {
        if imm.split(',').any(|s| s.trim().trim_matches('"') == etag.trim_matches('"')) {
            return Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header(header::ETAG, etag)
                .header(header::CACHE_CONTROL, "private, no-cache, must-revalidate")
                .body(Body::empty())
                .expect("build 304 response");
        }
    }

    parts
        .headers
        .insert(header::ETAG, HeaderValue::try_from(etag.as_str()).expect("ETag is valid ASCII"));
    parts
        .headers
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-cache, must-revalidate"));

    Response::from_parts(parts, Body::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded_path() {
        assert!(is_excluded_path("/api/auth/login"));
        assert!(is_excluded_path("/api/auth/refresh"));
        assert!(is_excluded_path("/api/v1/auth/login"));
        assert!(is_excluded_path("/api/health"));
        assert!(is_excluded_path("/api/metrics/vitals"));
        assert!(is_excluded_path("/api/v1/metrics/foo"));
        assert!(is_excluded_path("/metrics"));
        assert!(!is_excluded_path("/api/v1/users"));
        assert!(!is_excluded_path("/api/users"));
        assert!(!is_excluded_path("/api/v1/animals"));
    }

    #[test]
    fn test_etag_format() {
        let mut hasher = Sha256::new();
        hasher.update(b"hello");
        let hash = hasher.finalize();
        let etag = format!("\"{:x}\"", hash);
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        assert_eq!(etag.len(), 66); // " + 64 hex + "
    }
}
