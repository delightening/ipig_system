//! 批次建立欄位（`POST /api/v1/facilities/pens/batch`）回歸測試。
//!
//! 重現的 bug：`batch_create_pens` 的 INSERT 用 `ON CONFLICT (zone_id, code)`，
//! 但 pens 的唯一索引是 partial（`uq_pens_zone_code_active ... WHERE is_active = true`，
//! 見 migrations/005）。PostgreSQL 推斷 partial unique index 時，conflict target
//! 必須帶上同一個 WHERE 謂詞，否則計畫階段直接丟 42P10 → 整批失敗，使用者只看到
//! 通用「資料庫操作失敗」。修法：`ON CONFLICT (zone_id, code) WHERE is_active = true`。

mod common;

use serde_json::{json, Value};
use serial_test::serial;

/// 建一個全新的 zone（避開 seed 區與其既有欄位），回傳其 id。
/// zone code 帶隨機後綴，避免同一測試 DB 反覆跑撞 zones 的 partial unique index。
async fn create_fresh_zone(app: &common::TestApp, token: &str) -> String {
    let building_id: uuid::Uuid = sqlx::query_scalar(
        "SELECT id FROM buildings WHERE is_active = true ORDER BY sort_order LIMIT 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("seed 應至少有一棟 building");

    let unique = uuid::Uuid::new_v4().simple().to_string();
    let zone_code = format!("BATCH-{}", &unique[..8]);

    let res = app
        .auth_post(
            "/api/v1/facilities/zones",
            &json!({ "building_id": building_id, "code": zone_code, "name": "批次測試區" }),
            token,
        )
        .await;
    assert_eq!(res.status(), 201, "建立測試 zone 應成功");
    let zone: Value = common::TestApp::json(res).await;
    zone["id"].as_str().expect("zone id").to_string()
}

/// 修 bug 前：整批 INSERT 撞 42P10 → 500「資料庫操作失敗」。
/// 修 bug 後：回 201，並實際建立 `count` 個欄位。
#[tokio::test]
#[serial]
async fn batch_create_pens_succeeds_in_empty_zone() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone_id = create_fresh_zone(&app, &token).await;

    let res = app
        .auth_post(
            "/api/v1/facilities/pens/batch",
            &json!({ "zone_id": zone_id, "prefix": "Q", "count": 3, "layout": "single" }),
            &token,
        )
        .await;

    assert_eq!(
        res.status(),
        201,
        "批次建立欄位應回 201；若為 500 代表 ON CONFLICT 無法推斷 partial index（回歸）"
    );
    let pens: Vec<Value> = common::TestApp::json(res).await;
    assert_eq!(pens.len(), 3, "應建立 3 個欄位");
    let mut codes: Vec<&str> = pens
        .iter()
        .map(|p| p["code"].as_str().expect("pen code"))
        .collect();
    codes.sort_unstable();
    assert_eq!(codes, vec!["Q01", "Q02", "Q03"]);
}

/// `ON CONFLICT DO NOTHING` 的去重語意仍要成立：對同一 zone 重跑相同 prefix，
/// 已存在的 code 應被跳過，回 201 且不重複建立。
#[tokio::test]
#[serial]
async fn batch_create_pens_is_idempotent_on_duplicate_codes() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone_id = create_fresh_zone(&app, &token).await;

    let body = json!({ "zone_id": zone_id, "prefix": "Q", "count": 2, "layout": "single" });

    let first_res = app
        .auth_post("/api/v1/facilities/pens/batch", &body, &token)
        .await;
    assert_eq!(first_res.status(), 201, "首次建立應成功");
    let first: Vec<Value> = common::TestApp::json(first_res).await;
    assert_eq!(first.len(), 2, "首次應建立 2 個欄位");

    let res = app
        .auth_post("/api/v1/facilities/pens/batch", &body, &token)
        .await;
    assert_eq!(res.status(), 201, "重跑仍應回 201（不是衝突錯誤）");
    let second: Vec<Value> = common::TestApp::json(res).await;
    assert!(
        second.is_empty(),
        "重複 code 應被 ON CONFLICT DO NOTHING 跳過，回傳 0 筆，實得 {}",
        second.len()
    );
}
