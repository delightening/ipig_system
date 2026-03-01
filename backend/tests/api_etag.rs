//! Integration tests for ETag middleware.

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn get_returns_etag_header() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/users", &token).await;

    assert_eq!(res.status(), 200);
    let etag = res.headers().get("etag");
    assert!(etag.is_some(), "GET response should include ETag header");
    let etag_val = etag.unwrap().to_str().unwrap();
    assert!(etag_val.starts_with('"') && etag_val.ends_with('"'));
    assert!(
        res.headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap().contains("private"))
            .unwrap_or(false),
        "Should include Cache-Control: private"
    );
}

#[tokio::test]
#[serial]
async fn get_with_if_none_match_returns_304_when_match() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res1 = app.auth_get("/api/users", &token).await;
    assert_eq!(res1.status(), 200);
    let etag = res1
        .headers()
        .get("etag")
        .expect("First GET should have ETag")
        .to_str()
        .unwrap()
        .to_string();

    let res2 = app
        .client
        .get(app.url("/api/users"))
        .bearer_auth(&token)
        .header("If-None-Match", etag)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res2.status(), 304);
    let body = res2.text().await.unwrap();
    assert!(body.is_empty(), "304 response should have empty body");
}

#[tokio::test]
#[serial]
async fn post_unaffected_no_etag() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let body = serde_json::json!({
        "name": "ETag Test Role",
        "description": "For ETag test"
    });

    let res = app
        .client
        .post(app.url("/api/roles"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("Request failed");

    assert!(res.status().is_success() || res.status().as_u16() == 409);
    let etag = res.headers().get("etag");
    assert!(etag.is_none(), "POST response should not include ETag");
}

#[tokio::test]
#[serial]
async fn excluded_paths_no_etag() {
    let app = common::TestApp::spawn().await;

    let res = app.client.get(app.url("/api/health")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(
        res.headers().get("etag").is_none(),
        "/api/health should not have ETag"
    );
}
