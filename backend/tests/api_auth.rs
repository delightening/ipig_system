//! Integration tests for authentication flow:
//! login → me → refresh → logout → verify token revoked

mod common;

use serial_test::serial;

// ── Login ────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_with_valid_credentials_returns_200_and_tokens() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    assert!(!token.is_empty());
}

#[tokio::test]
#[serial]
async fn login_with_wrong_password_returns_401() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());

    let res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": "definitely_wrong_password"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn login_with_invalid_email_format_returns_400() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({
            "email": "not-an-email",
            "password": "anything"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 400);
}

// ── Me ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn me_returns_current_user_info() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body["id"].is_string());
    assert!(body["email"].is_string());
    assert!(body["display_name"].is_string());
}

#[tokio::test]
#[serial]
async fn me_without_token_returns_401() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/me"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 401);
}

// ── Refresh ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn refresh_with_valid_token_returns_new_tokens() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let password = std::env::var("ADMIN_INITIAL_PASSWORD")
        .unwrap_or_else(|_| "iPig$ecure1".to_string());

    // Login to get refresh_token
    let login_res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(login_res.status(), 200);

    let login_body: serde_json::Value = login_res
        .json()
        .await
        .expect("Failed to parse login response");
    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("refresh_token should be present in login response");

    // Use refresh_token to get new tokens
    let refresh_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(refresh_res.status(), 200);
    let refresh_body: serde_json::Value = refresh_res
        .json()
        .await
        .expect("Failed to parse refresh response");
    assert!(refresh_body["access_token"].is_string());
    assert!(refresh_body["refresh_token"].is_string());
}

// ── Logout ───────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn logout_invalidates_token() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // Verify token works
    let me_res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(me_res.status(), 200);

    // Logout
    let logout_res = app.auth_post("/api/v1/auth/logout", &(), &token).await;
    assert!(logout_res.status().is_success() || logout_res.status() == 200);

    // Token should now be blacklisted
    let me_after = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(me_after.status(), 401);
}

// ── Password change ──────────────────────────────────────────

#[tokio::test]
#[serial]
async fn change_password_with_wrong_current_returns_error() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_put(
            "/api/v1/me/password",
            &serde_json::json!({
                "current_password": "wrong_current_password",
                "new_password": "NewPassword123!"
            }),
            &token,
        )
        .await;

    // Should be 400 or 401 (wrong current password)
    assert!(res.status() == 400 || res.status() == 401);
}
