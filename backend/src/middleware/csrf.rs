// CSRF 防護中介層（SEC-24）
//
// 使用 Double Submit Cookie 模式：
// 1. 對每個回應設定 csrf_token Cookie（非 HttpOnly，讓前端可讀取）
// 2. 對 POST/PUT/DELETE/PATCH 請求驗證 X-CSRF-Token header 與 Cookie 是否匹配
// 3. GET/HEAD/OPTIONS 請求免驗證
// 4. 依 COOKIE_SECURE 設定動態加入 Secure flag（SEC-29）

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, Method, Response, StatusCode},
    middleware::Next,
};
use uuid::Uuid;

use crate::AppState;

/// 需要 CSRF 驗證的 HTTP 方法
fn requires_csrf_check(method: &Method) -> bool {
    matches!(
        method,
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    )
}

/// 不需要 CSRF 驗證的路徑（公開端點）
fn is_exempt_path(path: &str) -> bool {
    let exempt_paths = [
        "/api/auth/login",
        "/api/auth/refresh",
        "/api/auth/forgot-password",
        "/api/auth/reset-password",
    ];
    exempt_paths.contains(&path)
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

/// CSRF 防護中介層（接收 AppState 以讀取 cookie_secure 設定）
pub async fn csrf_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let cookie_secure = state.config.cookie_secure;

    // 整合測試時可停用 CSRF（DISABLE_CSRF_FOR_TESTS=true）
    let skip_csrf = state.config.disable_csrf_for_tests;

    // 只有需要 CSRF 的方法且非豁免路徑才檢查
    if !skip_csrf && requires_csrf_check(&method) && !is_exempt_path(&path) {
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
                tracing::warn!("[CSRF] 驗證失敗 - Method: {}, Path: {}", method, path);
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
        .any(|v| {
            v.to_str()
                .map(|s| s.starts_with("csrf_token="))
                .unwrap_or(false)
        });

    if !has_csrf_cookie {
        let csrf_token = Uuid::new_v4().to_string();
        // SEC-29: 依 COOKIE_SECURE 設定動態加入 Secure flag
        let cookie_value = if cookie_secure {
            format!(
                "csrf_token={}; Path=/; SameSite=Lax; Max-Age=86400; Secure",
                csrf_token
            )
        } else {
            format!(
                "csrf_token={}; Path=/; SameSite=Lax; Max-Age=86400",
                csrf_token
            )
        };
        if let Ok(val) = cookie_value.parse() {
            response.headers_mut().append(header::SET_COOKIE, val);
        }
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === requires_csrf_check ===

    #[test]
    fn test_post_requires_csrf() {
        assert!(requires_csrf_check(&Method::POST));
    }

    #[test]
    fn test_put_requires_csrf() {
        assert!(requires_csrf_check(&Method::PUT));
    }

    #[test]
    fn test_delete_requires_csrf() {
        assert!(requires_csrf_check(&Method::DELETE));
    }

    #[test]
    fn test_patch_requires_csrf() {
        assert!(requires_csrf_check(&Method::PATCH));
    }

    #[test]
    fn test_get_no_csrf() {
        assert!(!requires_csrf_check(&Method::GET));
    }

    #[test]
    fn test_head_no_csrf() {
        assert!(!requires_csrf_check(&Method::HEAD));
    }

    #[test]
    fn test_options_no_csrf() {
        assert!(!requires_csrf_check(&Method::OPTIONS));
    }

    // === is_exempt_path ===

    #[test]
    fn test_login_exempt() {
        assert!(is_exempt_path("/api/auth/login"));
    }

    #[test]
    fn test_refresh_exempt() {
        assert!(is_exempt_path("/api/auth/refresh"));
    }

    #[test]
    fn test_forgot_password_exempt() {
        assert!(is_exempt_path("/api/auth/forgot-password"));
    }

    #[test]
    fn test_reset_password_exempt() {
        assert!(is_exempt_path("/api/auth/reset-password"));
    }

    #[test]
    fn test_normal_path_not_exempt() {
        assert!(!is_exempt_path("/api/users"));
        assert!(!is_exempt_path("/api/protocols"));
        assert!(!is_exempt_path("/api/animals"));
    }

    // === extract_csrf_cookie ===

    #[test]
    fn test_extract_csrf_present() {
        let request = Request::builder()
            .header(
                header::COOKIE,
                "session=abc; csrf_token=my-token-123; other=val",
            )
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(
            extract_csrf_cookie(&request),
            Some("my-token-123".to_string())
        );
    }

    #[test]
    fn test_extract_csrf_absent() {
        let request = Request::builder()
            .header(header::COOKIE, "session=abc; other=val")
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(extract_csrf_cookie(&request), None);
    }

    #[test]
    fn test_extract_csrf_no_cookie_header() {
        let request = Request::builder()
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(extract_csrf_cookie(&request), None);
    }

    #[test]
    fn test_extract_csrf_only_cookie() {
        let request = Request::builder()
            .header(header::COOKIE, "csrf_token=solo-value")
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(
            extract_csrf_cookie(&request),
            Some("solo-value".to_string())
        );
    }
}
