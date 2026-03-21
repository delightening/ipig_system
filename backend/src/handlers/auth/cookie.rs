use axum::{
    http::{header, HeaderMap, StatusCode},
    response::Response,
};

use crate::{config::Config, models::LoginResponse, AppError, Result};

/// 建構認證 Cookie 的 Set-Cookie header 值
pub(crate) fn build_set_cookie(
    name: &str,
    value: &str,
    max_age_secs: i64,
    config: &Config,
) -> String {
    let mut cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        name, value, max_age_secs
    );
    if config.cookie_secure {
        cookie.push_str("; Secure");
    }
    if let Some(ref domain) = config.cookie_domain {
        cookie.push_str(&format!("; Domain={}", domain));
    }
    cookie
}

/// 建構清除 Cookie 的 Set-Cookie header 值（Max-Age=0）
pub(crate) fn build_clear_cookie(name: &str, config: &Config) -> String {
    build_set_cookie(name, "", 0, config)
}

/// 從請求的 Cookie header 中提取指定名稱的 cookie 值
pub(super) fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(&format!("{}=", name)))
        .map(|s| s[name.len() + 1..].to_string())
}

/// 將 LoginResponse 附加 Set-Cookie headers 回傳
pub(super) fn login_response_with_cookies(
    response: &LoginResponse,
    config: &Config,
) -> Result<Response> {
    let access_cookie = build_set_cookie(
        "access_token",
        &response.access_token,
        response.expires_in,
        config,
    );
    let refresh_cookie = build_set_cookie(
        "refresh_token",
        &response.refresh_token,
        7 * 24 * 3600, // 7 天
        config,
    );

    let body = serde_json::to_string(response)
        .map_err(|e| AppError::Internal(format!("JSON 序列化失敗: {}", e)))?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, access_cookie)
        .header(header::SET_COOKIE, refresh_cookie)
        .body(body.into())
        .map_err(|e| AppError::Internal(format!("Response 建構失敗: {e}")))
}
