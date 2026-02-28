//! Integration tests for user management (admin-only).

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_users_returns_paginated_result() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/users", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body["data"].is_array());
    // At least the admin user should exist
    assert!(
        !body["data"]
            .as_array()
            .expect("data should be array")
            .is_empty()
    );
}

#[tokio::test]
#[serial]
async fn create_user_and_fetch() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let new_user = serde_json::json!({
        "email": format!("testuser_{}@test.local", ts),
        "display_name": "Integration Test User",
        "password": "TestPassword123!",
        "role_id": null
    });

    let create_res = app.auth_post("/api/users", &new_user, &token).await;

    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned: {}",
        create_res.status()
    );

    let created: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create response");
    let user_id = created["id"].as_str().expect("User should have id");

    // Fetch individual user
    let get_res = app
        .auth_get(&format!("/api/users/{}", user_id), &token)
        .await;
    assert_eq!(get_res.status(), 200);

    let fetched: serde_json::Value = get_res
        .json()
        .await
        .expect("Failed to parse user response");
    assert_eq!(fetched["display_name"], "Integration Test User");
}

#[tokio::test]
#[serial]
async fn list_roles_returns_array() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/roles", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body.is_array());
}

#[tokio::test]
#[serial]
async fn list_permissions_returns_array() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/permissions", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body.is_array());
    assert!(
        !body
            .as_array()
            .expect("permissions should be array")
            .is_empty()
    );
}
