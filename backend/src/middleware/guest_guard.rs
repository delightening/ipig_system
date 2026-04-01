use axum::{
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::CurrentUser;

/// 訪客防護 middleware
/// - 攔截 GUEST 角色的所有 POST/PUT/PATCH/DELETE 請求（排除 auth 端點）
/// - 攔截 GUEST 角色的 GET 請求，回傳空資料（排除白名單端點）
pub async fn guest_guard_middleware(request: Request, next: Next) -> Response {
    let is_guest = request
        .extensions()
        .get::<CurrentUser>()
        .map(|u| u.is_guest())
        .unwrap_or(false);

    if !is_guest {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();

    // 寫入防護：攔截所有 mutation（排除 auth）
    if is_mutation(&request) {
        if path.ends_with("/auth/logout")
            || path.ends_with("/auth/refresh")
            || path.ends_with("/auth/heartbeat")
        {
            return next.run(request).await;
        }
        return (StatusCode::FORBIDDEN, "訪客模式無法修改資料").into_response();
    }

    // 讀取防護：GET 請求白名單放行，其餘回傳空資料
    if *request.method() == Method::GET && !is_guest_allowed_path(&path) {
        return empty_json_response();
    }

    next.run(request).await
}

/// Guest GET 白名單 — 這些路徑允許回傳真實資料
fn is_guest_allowed_path(path: &str) -> bool {
    // 去掉 API 版本前綴做比對
    let p = path
        .strip_prefix("/api/v1")
        .or_else(|| path.strip_prefix("/api"))
        .unwrap_or(path);

    // 允許：自身使用者資訊
    if p == "/me" || p.starts_with("/me/preferences") {
        return true;
    }
    // 允許：auth 相關
    if p.starts_with("/auth/") {
        return true;
    }
    // 允許：通知未讀數（避免 UI 報錯）
    if p == "/notifications/unread-count" {
        return true;
    }
    // 允許：amendment pending count（sidebar badge）
    if p == "/amendments/pending-count" {
        return true;
    }
    // 允許：健康檢查和指標
    if p == "/health" || p == "/metrics" {
        return true;
    }

    false
}

/// 回傳空 JSON 陣列 — 前端會用 useGuestQuery 的 demo data 覆蓋
fn empty_json_response() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        "[]",
    )
        .into_response()
}

fn is_mutation(request: &Request) -> bool {
    matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}
