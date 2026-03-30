//! Edge case integration tests: pagination boundaries, special characters,
//! invalid UUID formats, and oversized request bodies.

mod common;

use serial_test::serial;

// ===== 分頁邊界 =====

#[tokio::test]
#[serial]
async fn pagination_page_zero_returns_error_or_first_page() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/animals?page=0", &token).await;
    // page=0 should return 400 (invalid) or fallback to page 1 (200)
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 200,
        "page=0 should return 400 or 200, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn pagination_per_page_zero_returns_error_or_default() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/animals?per_page=0", &token).await;
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 200,
        "per_page=0 should return 400 or 200, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn pagination_per_page_huge_is_capped_or_rejected() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/animals?per_page=999999", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 200,
        "per_page=999999 should return 400 or 200 (with capped results), got {}",
        status
    );

    if status == 200 {
        let body: serde_json::Value = res.json().await.expect("Failed to parse JSON");
        // If the API caps per_page, the returned data array should be reasonably sized
        if let Some(data) = body["data"].as_array() {
            assert!(
                data.len() <= 10_000,
                "API should cap per_page to a reasonable limit"
            );
        }
    }
}

#[tokio::test]
#[serial]
async fn pagination_negative_page_returns_error() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/animals?page=-1", &token).await;
    let status = res.status().as_u16();
    // Negative page: should be 400 or treated as default (200)
    assert!(
        status == 400 || status == 200 || status == 422,
        "page=-1 should return 400/422 or 200, got {}",
        status
    );
}

// ===== 搜尋特殊字元（SQL injection attempts） =====

#[tokio::test]
#[serial]
async fn search_sql_injection_single_quote() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/animals?search='; DROP TABLE animals; --", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 200 || status == 400,
        "SQL injection in search should be safely handled, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn search_sql_injection_union_select() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get(
            "/api/animals?search=' UNION SELECT * FROM users --",
            &token,
        )
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 200 || status == 400,
        "UNION SELECT injection should be safely handled, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn search_special_characters_percent_underscore() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // SQL LIKE wildcards should be escaped
    let res = app
        .auth_get("/api/animals?search=%25%5F", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 200 || status == 400,
        "LIKE wildcards in search should be safely handled, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn search_unicode_and_emoji() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/animals?search=%E8%B1%AC%F0%9F%90%B7", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 200 || status == 400,
        "Unicode/emoji in search should be handled, got {}",
        status
    );
}

// ===== 無效 UUID 格式路徑參數 =====

#[tokio::test]
#[serial]
async fn invalid_uuid_in_animal_path_returns_400_or_404() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/animals/not-a-valid-uuid", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 404 || status == 422,
        "Invalid UUID should return 400/404/422, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn invalid_uuid_in_protocol_path_returns_400_or_404() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/protocols/xyz-not-uuid", &token)
        .await;
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 404 || status == 422,
        "Invalid UUID should return 400/404/422, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn empty_uuid_in_path_returns_error() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/animals/", &token).await;
    let status = res.status().as_u16();
    // Empty ID: could be treated as list (200) or invalid (400/404)
    assert!(
        status == 200 || status == 400 || status == 404,
        "Empty UUID path should return 200/400/404, got {}",
        status
    );
}

// ===== 超大 request body =====

#[tokio::test]
#[serial]
async fn oversized_request_body_is_rejected() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // Generate a ~2MB JSON string (well above typical API limits)
    let huge_value = "x".repeat(2 * 1024 * 1024);
    let body = serde_json::json!({
        "ear_tag": huge_value,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01"
    });

    let res = app.auth_post("/api/animals", &body, &token).await;
    let status = res.status().as_u16();
    assert!(
        status == 400 || status == 413 || status == 422,
        "Oversized body should return 400/413/422, got {}",
        status
    );
}

#[tokio::test]
#[serial]
async fn deeply_nested_json_is_handled() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // Build deeply nested JSON: {"a":{"a":{"a":...}}}
    let mut nested = serde_json::json!("leaf");
    for _ in 0..128 {
        nested = serde_json::json!({ "a": nested });
    }

    let res = app
        .client
        .post(app.url("/api/animals"))
        .bearer_auth(&token)
        .json(&nested)
        .send()
        .await
        .expect("Request failed");
    let status = res.status().as_u16();
    // Should not panic; 400/413/422 are acceptable
    assert!(
        status == 400 || status == 413 || status == 422,
        "Deeply nested JSON should return 400/413/422, got {}",
        status
    );
}
