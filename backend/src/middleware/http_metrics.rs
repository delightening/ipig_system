use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn http_metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().to_string();
    let path = normalize_path(request.uri().path());
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    let duration = start.elapsed().as_secs_f64();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path"   => path.clone(),
        "status" => status,
    )
    .increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path"   => path,
    )
    .record(duration);

    response
}

// 只保留前 4 個 path segment，避免 UUID 造成高基數
// /api/v1/animals/uuid/obs/123 → /api/v1/animals
fn normalize_path(path: &str) -> String {
    let parts: Vec<&str> = path.splitn(5, '/').take(4).collect();
    let joined = parts.join("/");
    if joined.is_empty() {
        "/".to_string()
    } else {
        joined
    }
}
