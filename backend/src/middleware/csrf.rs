// CSRF 防護中介層（SEC-24）
//
// 使用 Signed Double Submit Cookie 模式：
// 1. 對每個回應設定 csrf_token Cookie（非 HttpOnly，讓前端可讀取）
//    token 格式為 `{nonce}.{hmac_hex}`，其中 HMAC 以 JWT_SECRET 為 key，
//    將 nonce + 使用者 ID（來自 access_token cookie）混合簽署，
//    使 token 綁定至當前 session。
// 2. 對 POST/PUT/DELETE/PATCH 請求驗證 X-CSRF-Token header 與 Cookie 是否匹配，
//    並重新計算 HMAC 確認 token 未被竄改且屬於當前 session。
// 3. GET/HEAD/OPTIONS 請求免驗證
// 4. 依 COOKIE_SECURE 設定動態加入 Secure flag（SEC-29）

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, Method, Response, StatusCode},
    middleware::Next,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

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

/// 從 Cookie header 提取指定 cookie 值
fn extract_cookie(request: &Request, name: &str) -> Option<String> {
    let cookie_header = request.headers().get(header::COOKIE)?.to_str().ok()?;
    let prefix = format!("{}=", name);
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(&prefix))
        .map(|s| s[prefix.len()..].to_string())
}

/// 從 access_token JWT 快速提取 sub（user_id），不做完整驗證
/// （CSRF 層僅需綁定 session，JWT 驗證由 auth middleware 負責）
fn extract_session_id(request: &Request) -> String {
    extract_cookie(request, "access_token")
        .and_then(|jwt| {
            let parts: Vec<&str> = jwt.split('.').collect();
            if parts.len() != 3 {
                return None;
            }
            // JWT payload 是第二段，base64url 編碼
            use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
            let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
            let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
            payload.get("sub").and_then(|v| v.as_str()).map(|s| s.to_string())
        })
        .unwrap_or_default() // 未登入時為空字串，CSRF token 仍可綁定（空 session）
}

/// 產生 HMAC 簽署的 CSRF token：`{nonce}.{hmac_hex}`
fn generate_signed_csrf_token(secret: &str, session_id: &str) -> String {
    let nonce = Uuid::new_v4().to_string();
    let signature = compute_csrf_hmac(secret, &nonce, session_id);
    format!("{}.{}", nonce, signature)
}

/// 驗證 CSRF token 簽章是否匹配當前 session
fn verify_signed_csrf_token(secret: &str, token: &str, session_id: &str) -> bool {
    let Some((nonce, provided_sig)) = token.split_once('.') else {
        return false;
    };
    if nonce.is_empty() || provided_sig.is_empty() {
        return false;
    }
    let expected_sig = compute_csrf_hmac(secret, nonce, session_id);
    // 使用固定時間比較防止 timing attack
    constant_time_eq(provided_sig.as_bytes(), expected_sig.as_bytes())
}

/// 計算 HMAC-SHA256(secret, nonce + session_id)
fn compute_csrf_hmac(secret: &str, nonce: &str, session_id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC 初始化不應失敗（key 長度無限制）");
    mac.update(nonce.as_bytes());
    mac.update(b"|");
    mac.update(session_id.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// 固定時間字串比較（防止 timing side-channel attack）
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
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

    // 在 request 被 move 之前先提取 session_id（供驗證與簽發使用）
    let session_id = extract_session_id(&request);

    // 只有需要 CSRF 的方法且非豁免路徑才檢查
    if !skip_csrf && requires_csrf_check(&method) && !is_exempt_path(&path) {
        let cookie_token = extract_cookie(&request, "csrf_token");
        let header_token = request
            .headers()
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        match (cookie_token, header_token) {
            (Some(cookie), Some(header_val))
                if !cookie.is_empty()
                    && cookie == header_val
                    && verify_signed_csrf_token(
                        &state.config.jwt_secret,
                        &cookie,
                        &session_id,
                    ) =>
            {
                // 驗證通過：header == cookie 且 HMAC 簽章與當前 session 匹配
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
        let csrf_token = generate_signed_csrf_token(&state.config.jwt_secret, &session_id);
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

    // === extract_cookie ===

    #[test]
    fn test_extract_cookie_present() {
        let request = Request::builder()
            .header(
                header::COOKIE,
                "session=abc; csrf_token=my-token-123; other=val",
            )
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(
            extract_cookie(&request, "csrf_token"),
            Some("my-token-123".to_string())
        );
    }

    #[test]
    fn test_extract_cookie_absent() {
        let request = Request::builder()
            .header(header::COOKIE, "session=abc; other=val")
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(extract_cookie(&request, "csrf_token"), None);
    }

    #[test]
    fn test_extract_cookie_no_cookie_header() {
        let request = Request::builder()
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(extract_cookie(&request, "csrf_token"), None);
    }

    #[test]
    fn test_extract_cookie_only_cookie() {
        let request = Request::builder()
            .header(header::COOKIE, "csrf_token=solo-value")
            .body(Body::empty())
            .expect("建構測試 Request 不應失敗");
        assert_eq!(
            extract_cookie(&request, "csrf_token"),
            Some("solo-value".to_string())
        );
    }

    // === Signed CSRF token ===

    #[test]
    fn test_generate_and_verify_signed_token() {
        let secret = "test-secret-key-for-csrf";
        let session_id = "user-123";
        let token = generate_signed_csrf_token(secret, session_id);
        assert!(
            token.contains('.'),
            "簽署 token 應包含 nonce.signature 格式"
        );
        assert!(
            verify_signed_csrf_token(secret, &token, session_id),
            "使用相同 session 驗證應通過"
        );
    }

    #[test]
    fn test_signed_token_wrong_session() {
        let secret = "test-secret-key-for-csrf";
        let token = generate_signed_csrf_token(secret, "user-123");
        assert!(
            !verify_signed_csrf_token(secret, &token, "user-456"),
            "不同 session 驗證應失敗"
        );
    }

    #[test]
    fn test_signed_token_wrong_secret() {
        let token = generate_signed_csrf_token("secret-a", "user-123");
        assert!(
            !verify_signed_csrf_token("secret-b", &token, "user-123"),
            "不同 secret 驗證應失敗"
        );
    }

    #[test]
    fn test_signed_token_tampered() {
        let secret = "test-secret";
        let token = generate_signed_csrf_token(secret, "user-123");
        let tampered = format!("{}x", token);
        assert!(
            !verify_signed_csrf_token(secret, &tampered, "user-123"),
            "竄改後的 token 驗證應失敗"
        );
    }

    #[test]
    fn test_signed_token_invalid_format() {
        let secret = "test-secret";
        assert!(!verify_signed_csrf_token(secret, "no-dot-here", "user"));
        assert!(!verify_signed_csrf_token(secret, ".empty-nonce", "user"));
        assert!(!verify_signed_csrf_token(secret, "empty-sig.", "user"));
        assert!(!verify_signed_csrf_token(secret, "", "user"));
    }

    #[test]
    fn test_signed_token_empty_session() {
        let secret = "test-secret";
        let token = generate_signed_csrf_token(secret, "");
        assert!(
            verify_signed_csrf_token(secret, &token, ""),
            "空 session 應可產生並驗證（未登入狀態）"
        );
        assert!(
            !verify_signed_csrf_token(secret, &token, "user-123"),
            "空 session 的 token 不應通過有 session 的驗證"
        );
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}
