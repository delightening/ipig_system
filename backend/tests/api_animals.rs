//! Integration tests for animal CRUD operations.

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_animals_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/animals", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
#[serial]
async fn list_animals_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/animals"))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn create_and_get_animal() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 耳號用「配置 + 撞號重試」而非亂數：三位數只有 900 個值，抽籤在共用測試 DB 上
    // 必然碰撞而回 409（force_create 跳不過重複守衛）；而純配置與建立之間還有
    // 跨 process 的 TOCTOU 空隙。詳見 common::create_animal_with_free_ear_tag。
    let created = common::create_animal_with_free_ear_tag(&app, &token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "white",
            "gender": "female",
            "entry_date": "2026-01-15",
            "entry_weight": 25.5,
            "pen_location": "A-01",
            "force_create": true
        })
    })
    .await;
    let animal_id = created["id"]
        .as_str()
        .expect("Created animal should have id");
    // 耳號取自建立回應，不是外部變數——helper 撞號時會換一個重試，
    // 最終用的是哪一個只有回應知道。
    let created_ear_tag = created["ear_tag"]
        .as_str()
        .expect("Created animal should have ear_tag");

    // Fetch the created animal
    let get_res = app
        .auth_get(&format!("/api/v1/animals/{}", animal_id), &token)
        .await;
    assert_eq!(get_res.status(), 200);

    let fetched: serde_json::Value = get_res
        .json()
        .await
        .expect("Failed to parse animal response");
    assert_eq!(fetched["ear_tag"], created_ear_tag);
}

#[tokio::test]
#[serial]
async fn create_animal_with_invalid_data_returns_400() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let bad_body = serde_json::json!({
        "ear_tag": "",
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01"
    });

    let res = app.auth_post("/api/v1/animals", &bad_body, &token).await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn get_nonexistent_animal_returns_404() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let res = app
        .auth_get(&format!("/api/v1/animals/{}", fake_id), &token)
        .await;

    assert_eq!(res.status(), 404);
}
