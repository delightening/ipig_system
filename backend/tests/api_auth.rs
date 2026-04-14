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

// ── Validation error message ─────────────────────────────────

#[tokio::test]
#[serial]
async fn login_validation_error_does_not_leak_field_names() {
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

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON");
    let message = body["message"].as_str().unwrap_or("");

    // The response must NOT expose field-level details like "email: is not a valid..."
    // which would leak our API schema to attackers.
    assert!(
        !message.contains("email:"),
        "Error message leaks field name 'email:' — got: {:?}",
        message
    );
    assert!(
        !message.to_lowercase().contains("is not a valid email"),
        "Error message leaks validator detail — got: {:?}",
        message
    );
}

// ── 2FA rate limiting ─────────────────────────────────────────

#[tokio::test]
#[serial]
async fn two_factor_verify_locks_after_5_failures() {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let app = common::TestApp::spawn().await;

    // Fetch admin user (guaranteed to exist after spawn)
    let (user_id_str, email): (String, String) = sqlx::query_as(
        "SELECT id::text, email FROM users WHERE is_active = true ORDER BY created_at LIMIT 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch test user");

    // Build a valid 2FA temp token using the same ES256 private key as the running server
    // (common::TestApp::spawn() sets JWT_EC_PRIVATE_KEY in env)
    let private_pem = std::env::var("JWT_EC_PRIVATE_KEY")
        .expect("JWT_EC_PRIVATE_KEY must be set (done by TestApp::spawn)");
    let encoding_key = EncodingKey::from_ec_pem(private_pem.as_bytes())
        .expect("Failed to parse test EC private key");

    let now = Utc::now();
    let exp = (now + Duration::seconds(300)).timestamp() as usize;
    let claims = serde_json::json!({
        "sub": user_id_str,
        "purpose": "2fa_pending",
        "exp": exp,
        "iat": now.timestamp(),
    });

    let temp_token = encode(
        &Header::new(Algorithm::ES256),
        &claims,
        &encoding_key,
    )
    .expect("Failed to sign temp token");

    // Pre-seed 5 2fa_failure events for this email within the 5-minute window
    for _ in 0..5 {
        sqlx::query(
            r#"INSERT INTO login_events (id, user_id, email, event_type, created_at)
               VALUES (gen_random_uuid(), $1::uuid, $2, '2fa_failure', NOW())"#,
        )
        .bind(&user_id_str)
        .bind(&email)
        .execute(&app.db_pool)
        .await
        .expect("Failed to insert 2fa_failure event");
    }

    // The next attempt (6th failure) must be rate-limited
    let res = app
        .client
        .post(app.url("/api/v1/auth/2fa/verify"))
        .json(&serde_json::json!({
            "temp_token": temp_token,
            "code": "123456"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    // Cleanup before asserting so the DB is clean even if the test fails
    sqlx::query(
        "DELETE FROM login_events WHERE email = $1 AND event_type = '2fa_failure'",
    )
    .bind(&email)
    .execute(&app.db_pool)
    .await
    .expect("Failed to clean up 2fa_failure events");

    assert_eq!(
        res.status(),
        429,
        "Expected 429 TooManyRequests after 5 pre-seeded 2FA failures"
    );
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
