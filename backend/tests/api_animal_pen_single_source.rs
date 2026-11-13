//! 動物欄位的寫入端以 `pen_id` 為唯一來源（B 方案第 2 階段）。
//!
//! 背景：前端移動／編輯動物只送 `pen_location` 文字，過去 `pen_id` 與 `pens.current_count`
//! 因此不會跟著動，造成「畫面對、資料庫過期」。現在後端把代碼解析成 `pen_id`，並以解析出的
//! 欄位代碼回寫文字欄，兩欄在寫入當下一致；找不到或有歧義的代碼回 400，不猜。
//!
//! 測試用的欄位代碼一律用隨機的 `T` 開頭代碼，避免與種子欄位或其他測試撞名；
//! 所有資料都是合成的。

mod common;

use common::TestApp;
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

/// 在既有的區底下建一個啟用中的測試欄位，回傳 (pen_id, code)。
async fn make_pen(app: &TestApp, zone_id: Uuid, capacity: i32) -> (Uuid, String) {
    // `T` 不在 A–G 範圍，後端不會對它補零／改寫；後綴取 UUID 前 6 碼，同一個丟棄庫內不會撞名
    let id = Uuid::new_v4();
    let code = format!("T{}", id.simple().to_string()[..6].to_uppercase());
    sqlx::query(
        "INSERT INTO pens (id, zone_id, code, capacity, current_count) VALUES ($1, $2, $3, $4, 0)",
    )
    .bind(id)
    .bind(zone_id)
    .bind(&code)
    .bind(capacity)
    .execute(&app.db_pool)
    .await
    .expect("insert pen");
    (id, code)
}

async fn any_zone(app: &TestApp) -> (Uuid, Uuid) {
    // 測試庫的區可能被全庫匯入測試清掉，改用 harness 自帶的固定區（冪等）
    common::ensure_fixture_pen(&app.db_pool).await
}

async fn pen_row(app: &TestApp, pen_id: Uuid) -> i32 {
    sqlx::query_scalar("SELECT current_count FROM pens WHERE id = $1")
        .bind(pen_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("pen current_count")
}

async fn create_in(app: &TestApp, token: &str, pen_code: &str) -> Value {
    common::create_animal_with_free_ear_tag(app, token, |ear_tag| {
        json!({
            "ear_tag": ear_tag,
            "breed": "white",
            "gender": "female",
            "entry_date": "2026-01-15",
            "entry_weight": 25.5,
            "pen_location": pen_code,
            "force_create": true
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn create_with_only_pen_location_sets_pen_id_and_count() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, _) = any_zone(&app).await;
    let (pen, code) = make_pen(&app, zone, 4).await;

    let created = create_in(&app, &token, &code).await;

    assert_eq!(created["pen_id"], pen.to_string());
    assert_eq!(created["pen_location"], code.as_str());
    assert_eq!(pen_row(&app, pen).await, 1, "建立後欄位頭數應為 1");
}

#[tokio::test]
#[serial]
async fn update_with_only_pen_location_moves_pen_id_and_counts() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, _) = any_zone(&app).await;
    let (from_pen, from_code) = make_pen(&app, zone, 4).await;
    let (to_pen, to_code) = make_pen(&app, zone, 4).await;
    let created = create_in(&app, &token, &from_code).await;
    let id = created["id"].as_str().expect("animal id");

    // 前端現況：只送 pen_location
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": to_code }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);

    let after: Value = app
        .auth_get(&format!("/api/v1/animals/{id}"), &token)
        .await
        .json()
        .await
        .expect("animal json");
    assert_eq!(after["pen_id"], to_pen.to_string(), "pen_id 應跟著移動");
    assert_eq!(after["pen_location"], to_code.as_str());
    assert_eq!(pen_row(&app, from_pen).await, 0, "原欄位頭數應扣掉");
    assert_eq!(pen_row(&app, to_pen).await, 1, "新欄位頭數應加上");
}

#[tokio::test]
#[serial]
async fn echoing_the_current_pen_is_not_blocked_by_capacity() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, _) = any_zone(&app).await;
    // 容量 1：動物自己就把欄位佔滿。整份表單原樣回填同一欄不該被當成「欄位已滿」
    let (_, code) = make_pen(&app, zone, 1).await;

    let created = create_in(&app, &token, &code).await;
    let id = created["id"].as_str().expect("animal id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": code }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);
}

#[tokio::test]
#[serial]
async fn unknown_pen_code_is_rejected() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, _) = any_zone(&app).await;
    let (_, code) = make_pen(&app, zone, 4).await;
    let created = create_in(&app, &token, &code).await;
    let id = created["id"].as_str().expect("animal id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": "NOPE99" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "不存在的欄位代碼應回 400");
}

#[tokio::test]
#[serial]
async fn ambiguous_pen_code_is_rejected_instead_of_guessed() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, building) = any_zone(&app).await;
    let (_, code) = make_pen(&app, zone, 4).await;

    // 在另一個區建立「同代碼」的啟用欄位 → 代碼對到兩個欄位，無法判定
    let other_zone = Uuid::new_v4();
    sqlx::query("INSERT INTO zones (id, building_id, code) VALUES ($1, $2, $3)")
        .bind(other_zone)
        .bind(building)
        .bind(format!("ZT{}", &other_zone.simple().to_string()[..6]))
        .execute(&app.db_pool)
        .await
        .expect("insert zone");
    sqlx::query(
        "INSERT INTO pens (id, zone_id, code, capacity, current_count) VALUES ($1, $2, $3, 4, 0)",
    )
    .bind(Uuid::new_v4())
    .bind(other_zone)
    .bind(&code)
    .execute(&app.db_pool)
    .await
    .expect("insert duplicate-code pen");

    let res = app
        .auth_post(
            "/api/v1/animals",
            &json!({
                "ear_tag": common::free_ear_tag(&app.db_pool).await,
                "breed": "white",
                "gender": "female",
                "entry_date": "2026-01-15",
                "entry_weight": 25.5,
                "pen_location": code,
                "force_create": true
            }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "歧義的欄位代碼應回 400，不猜");
}

#[tokio::test]
#[serial]
async fn conflicting_pen_id_and_pen_location_is_rejected() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (zone, _) = any_zone(&app).await;
    let (_, code_a) = make_pen(&app, zone, 4).await;
    let (pen_b, _) = make_pen(&app, zone, 4).await;
    let created = create_in(&app, &token, &code_a).await;
    let id = created["id"].as_str().expect("animal id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": code_a, "pen_id": pen_b }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "兩個來源互相矛盾應回 400");
}
