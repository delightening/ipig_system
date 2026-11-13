//! 動物時間軸的「IACUC No. 變更」事件（`GET /api/v1/animals/{id}/events`）。
//!
//! 要釘住的契約：
//! 1. 首次分配並帶了過去的實驗日期 → 動物的 `experiment_date` 採用該日期（不被蓋成今天），
//!    事件的 `before_data/after_data` 有「舊 → 新」，且 `after_data` 帶實際日期。
//! 2. 之後再轉換 IACUC No. → 事件有「舊 → 新」，但**不帶**日期
//!    （`experiment_date` 只有一格，仍是最初分配日，不能當成這次轉換的實際日期）。
//! 3. 修正前寫入的歷史事件沒有 before/after（稽核紀錄不可改），讀取時由顯示文字回補。

mod common;

use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

use common::TestApp;

fn animal_body(ear_tag: &str) -> Value {
    json!({
        "ear_tag": ear_tag,
        "gender": "male",
        "breed": "minipig",
        "entry_date": "2026-07-13",
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true,
    })
}

/// 建立一隻未分配的動物，回傳它的 id。
async fn create_animal(app: &TestApp, token: &str) -> Uuid {
    let created = common::create_animal_with_free_ear_tag(app, token, animal_body).await;
    Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid")
}

async fn put_animal(app: &TestApp, token: &str, id: Uuid, body: Value) -> Value {
    let res = app
        .auth_put(&format!("/api/v1/animals/{id}"), &body, token)
        .await;
    let status = res.status().as_u16();
    let json: Value = res.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 200, "更新動物失敗：{json}");
    json
}

async fn get_events(app: &TestApp, token: &str, id: Uuid) -> Vec<Value> {
    let res = app
        .auth_get(&format!("/api/v1/animals/{id}/events"), token)
        .await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.expect("events json");
    body.as_array().expect("events 應為陣列").clone()
}

#[tokio::test]
#[serial]
async fn first_assignment_with_past_date_is_honored_and_recorded() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let id = create_animal(&app, &token).await;

    let updated = put_animal(
        &app,
        &token,
        id,
        json!({
            "status": "in_experiment",
            "iacuc_no": "PIG-TEST-0001",
            "experiment_date": "2026-09-01",
        }),
    )
    .await;
    assert_eq!(
        updated["experiment_date"], "2026-09-01",
        "首次分配帶的過去日期不該被蓋成今天：{updated}"
    );

    let events = get_events(&app, &token, id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0];
    assert_eq!(e["event_type"], "IACUC_CHANGE");
    assert!(e["before_data"]["iacuc_no"].is_null(), "{e}");
    assert_eq!(e["after_data"]["iacuc_no"], "PIG-TEST-0001", "{e}");
    assert_eq!(e["after_data"]["experiment_date"], "2026-09-01", "{e}");
}

#[tokio::test]
#[serial]
async fn first_assignment_without_date_falls_back_to_today() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let id = create_animal(&app, &token).await;

    let updated = put_animal(
        &app,
        &token,
        id,
        json!({ "status": "in_experiment", "iacuc_no": "PIG-TEST-0002" }),
    )
    .await;
    assert!(
        updated["experiment_date"].is_string(),
        "沒帶日期時仍應以今天為實驗日期：{updated}"
    );
}

#[tokio::test]
#[serial]
async fn later_change_records_old_and_new_without_a_date() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let id = create_animal(&app, &token).await;

    // 狀態維持未分配：先掛上 A 計畫，再轉到 B 計畫（實驗中的動物不可改 IACUC No.）
    put_animal(&app, &token, id, json!({ "iacuc_no": "PIG-TEST-A" })).await;
    put_animal(&app, &token, id, json!({ "iacuc_no": "PIG-TEST-B" })).await;

    let events = get_events(&app, &token, id).await;
    assert_eq!(events.len(), 2, "{events:?}");
    // 新 → 舊排序
    assert_eq!(events[0]["before_data"]["iacuc_no"], "PIG-TEST-A");
    assert_eq!(events[0]["after_data"]["iacuc_no"], "PIG-TEST-B");
    assert!(
        events[0]["after_data"].get("experiment_date").is_none(),
        "轉換計畫不該帶日期：{}",
        events[0]
    );
    assert!(events[1]["before_data"]["iacuc_no"].is_null());
    assert_eq!(events[1]["after_data"]["iacuc_no"], "PIG-TEST-A");
}

#[tokio::test]
#[serial]
async fn legacy_event_without_before_after_is_backfilled_from_display_text() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let id = create_animal(&app, &token).await;

    // 模擬修正前寫入的歷史事件：before/after 為空，只有顯示文字
    let actor: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("any user");
    sqlx::query(
        "SELECT public.log_activity($1, 'ANIMAL', 'IACUC_CHANGE', 'animal', $2, \
         '[(無) → PIG-LEGACY-1] 243', NULL::jsonb, NULL::jsonb, NULL::inet, NULL::text, \
         NULL::uuid, NULL::text[], false)",
    )
    .bind(actor)
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("insert legacy-style audit row");

    let events = get_events(&app, &token, id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0];
    assert!(e["before_data"]["iacuc_no"].is_null(), "{e}");
    assert_eq!(e["after_data"]["iacuc_no"], "PIG-LEGACY-1", "{e}");
    assert!(
        e["after_data"].get("experiment_date").is_none(),
        "歷史事件沒有實際日期：{e}"
    );
}
