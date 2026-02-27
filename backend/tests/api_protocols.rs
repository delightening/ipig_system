//! Integration tests for protocol (AUP) workflow.

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_protocols_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/protocols", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["data"].is_array());
}

#[tokio::test]
#[serial]
async fn create_protocol_draft() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let body = serde_json::json!({
        "title": "Integration Test Protocol",
        "protocol_number": format!("TEST-{}", rand_num()),
        "pi_name": "Test PI",
        "department": "Testing Dept",
        "species": "Sus scrofa",
        "animal_count": 10,
        "start_date": "2026-03-01",
        "end_date": "2026-12-31",
        "purpose": "Integration test — verifying protocol creation flow"
    });

    let res = app.auth_post("/api/protocols", &body, &token).await;

    // Expect 201 or 200
    assert!(
        res.status() == 201 || res.status() == 200,
        "Create protocol returned: {}",
        res.status()
    );

    let created: serde_json::Value = res.json().await.unwrap();
    assert!(created["id"].is_string());
    assert_eq!(created["title"], "Integration Test Protocol");
}

#[tokio::test]
#[serial]
async fn list_protocols_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/protocols"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

fn rand_num() -> u32 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    d.subsec_nanos() % 10000
}
