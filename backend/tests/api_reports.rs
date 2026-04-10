//! Integration tests for report endpoints and notifications.

mod common;

use serial_test::serial;

// ── Reports ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn stock_on_hand_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/stock-on-hand", &token).await;

    // 200 even if empty data set
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn stock_ledger_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/stock-ledger", &token).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn purchase_lines_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/purchase-lines", &token).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn reports_without_auth_return_401() {
    let app = common::TestApp::spawn().await;

    let endpoints = [
        "/api/v1/reports/stock-on-hand",
        "/api/v1/reports/stock-ledger",
        "/api/v1/reports/purchase-lines",
    ];

    for endpoint in endpoints {
        let res = app
            .client
            .get(app.url(endpoint))
            .send()
            .await
            .expect("HTTP request failed");
        assert_eq!(res.status(), 401, "Endpoint {} should require auth", endpoint);
    }
}

// ── Notifications ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn list_notifications_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/notifications", &token).await;
    assert_eq!(res.status(), 200);
}
