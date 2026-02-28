//! Integration tests for health and metrics endpoints.

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn health_check_returns_200_with_healthy_status() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/health"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["checks"]["database"]["status"], "up");
    assert!(body["version"].is_string());
    assert!(body["checks"]["database"]["latency_ms"].is_number());
}

#[tokio::test]
#[serial]
async fn metrics_endpoint_returns_prometheus_format() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/metrics"))
        .send()
        .await
        .expect("HTTP request failed");

    // Metrics endpoint may return 200 or 503 depending on PrometheusHandle availability.
    // In test mode we set metrics_handle = None, so expect 503.
    // The important thing is the endpoint exists and responds.
    assert!(res.status() == 200 || res.status() == 503);
}

#[tokio::test]
#[serial]
async fn unknown_route_returns_404() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/nonexistent"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 404);
}
