// CSRF 防護中介層（SEC-24）
//
// 使用 Double Submit Cookie 模式：
// 1. 對每個回應設定 csrf_token Cookie（非 HttpOnly，讓前端可讀取）
// 2. 對 POST/PUT/DELETE/PATCH 請求驗證 X-CSRF-Token header 與 Cookie 是否匹配
// 3. GET/HEAD/OPTIONS 請求免驗證

use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, Response, StatusCode},
    middleware::Next,
};
use uuid::Uuid;

/// 需要 CSRF 驗證的 HTTP 方法
fn requires_csrf_check(method: &Method) -> bool {
    matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH)
}

/// 不需要 CSRF 驗證的路徑（公開端點）
fn is_exempt_path(path: &str) -> bool {
    let exempt_paths = [
        "/api/auth/login",
        "/api/auth/refresh",
        "/api/auth/forgot-password",
        "/api/auth/reset-password",
    ];
    exempt_paths.iter().any(|p| path == *p)
}

/// 從 Cookie header 提取 csrf_token 值
fn extract_csrf_cookie(request: &Request) -> Option<String> {
    let cookie_header = request.headers().get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("csrf_token="))
        .map(|s| s.trim_start_matches("csrf_token=").to_string())
}

/// CSRF 防護中介層
pub async fn csrf_middleware(
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // 只有需要 CSRF 的方法且非豁免路徑才檢查
    if requires_csrf_check(&method) && !is_exempt_path(&path) {
        let cookie_token = extract_csrf_cookie(&request);
        let header_token = request
            .headers()
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        match (cookie_token, header_token) {
            (Some(cookie), Some(header_val)) if !cookie.is_empty() && cookie == header_val => {
                // 驗證通過
            }
            _ => {
                tracing::warn!(
                    "[CSRF] 驗證失敗 - Method: {}, Path: {}",
                    method,
                    path
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    let mut response = next.run(request).await;

    // 如果回應中沒有 csrf_token cookie，產生一個新的
    // 檢查是否已存在 csrf cookie（透過回應的 Set-Cookie）
    let has_csrf_cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().map(|s| s.starts_with("csrf_token=")).unwrap_or(false));

    if !has_csrf_cookie {
        let csrf_token = Uuid::new_v4().to_string();
        let cookie_value = format!(
            "csrf_token={}; Path=/; SameSite=Lax; Max-Age=86400",
            csrf_token
        );
        if let Ok(val) = cookie_value.parse() {
            response.headers_mut().append(header::SET_COOKIE, val);
        }
    }

    Ok(response)
}
