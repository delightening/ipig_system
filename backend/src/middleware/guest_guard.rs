//! Guest demo 寫入防護 — defense-in-depth 補強
//!
//! 背景：guest 帳號（GUEST_DEMO_EMAIL）登入後 frontend axios interceptor
//! 會把寫入請求短路成假 200，但僅是 UI 層保護。攻擊者繞過 axios（curl / Postman /
//! 直接呼叫 fetch）即可寫入後端，完全依賴每個 mutation handler 都有 has_permission
//! 檢查 — 漏一個就破功。
//!
//! 本 middleware 在 auth_middleware 之後執行：
//! - 從 extensions 讀 CurrentUser
//! - 若 is_guest() 且 method 不是 GET/HEAD/OPTIONS → 直接 403
//! - 非 guest 或安全 method → pass through
//!
//! 與 has_permission(false for guest) 互為雙保險。

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::middleware::auth::CurrentUser;

/// 安全 method 白名單：唯讀請求允許 guest 通過，由 has_permission / handler
/// 內部邏輯決定能否讀取特定資源（前端 interceptor 也會回 demo data）。
fn is_safe_method(method: &Method) -> bool {
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

pub async fn guest_guard_middleware(request: Request, next: Next) -> Response {
    if !is_safe_method(request.method()) {
        if let Some(user) = request.extensions().get::<CurrentUser>() {
            if user.is_guest() {
                tracing::warn!(
                    "[guest_guard] guest 嘗試寫入：method={} path={}",
                    request.method(),
                    request.uri().path()
                );
                return (
                    StatusCode::FORBIDDEN,
                    "Guest 示範模式：唯讀，無法執行寫入操作",
                )
                    .into_response();
            }
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_methods_are_recognized() {
        assert!(is_safe_method(&Method::GET));
        assert!(is_safe_method(&Method::HEAD));
        assert!(is_safe_method(&Method::OPTIONS));
        assert!(!is_safe_method(&Method::POST));
        assert!(!is_safe_method(&Method::PUT));
        assert!(!is_safe_method(&Method::PATCH));
        assert!(!is_safe_method(&Method::DELETE));
    }
}
