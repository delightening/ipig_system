//! Integration tests for calendar endpoint access control.
//! Verifies that calendar management routes enforce admin-only access.

mod common;

use serial_test::serial;

/// A non-admin user must receive 403 when accessing the calendar status endpoint.
/// This guards against the require_calendar_admin() check being removed or bypassed.
#[tokio::test]
#[serial]
async fn calendar_status_returns_403_for_non_admin() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // Create a regular user with no roles
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time error")
        .subsec_nanos();
    let email = format!("calendar_nonadmin_{}@test.local", ts);

    let create_res = app
        .auth_post(
            "/api/v1/users",
            &serde_json::json!({
                "email": email,
                "display_name": "Calendar Non-Admin Test",
                "password": "TestPassword123!",
                "role_id": null
            }),
            &admin_token,
        )
        .await;

    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned unexpected status: {}",
        create_res.status()
    );

    // Login as the non-admin user
    let user_token = app
        .login(&email, "TestPassword123!")
        .await
        .expect("Non-admin user login failed");

    // Calendar status must be forbidden for users without SYSTEM_ADMIN or ADMIN_STAFF role
    let res = app
        .auth_get("/api/v1/hr/calendar/status", &user_token)
        .await;

    assert_eq!(
        res.status(),
        403,
        "Expected 403 Forbidden for non-admin user on calendar status, got {}",
        res.status()
    );
}

/// An unauthenticated request must be rejected before reaching the role check.
#[tokio::test]
#[serial]
async fn calendar_status_returns_401_without_token() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/hr/calendar/status"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        res.status(),
        401,
        "Expected 401 Unauthorized for unauthenticated request, got {}",
        res.status()
    );
}

/// Admin users (SYSTEM_ADMIN role) must be able to reach the calendar status endpoint.
/// This confirms the allow-list is not over-restrictive.
#[tokio::test]
#[serial]
async fn calendar_status_is_accessible_to_admin() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/hr/calendar/status", &token).await;

    // Admin must not receive 401 or 403 — service errors (500) are acceptable
    // since the Google Calendar integration may not be configured in CI.
    assert_ne!(
        res.status(),
        401,
        "Admin should not be denied authentication on calendar status"
    );
    assert_ne!(
        res.status(),
        403,
        "Admin should not be forbidden from calendar status"
    );
}
