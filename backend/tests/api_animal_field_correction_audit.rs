//! R71-1 動物欄位修正核准：交易原子性 + 稽核軌跡回歸測試。
//!
//! 修復前：`review` 核准直改動物身分欄位（`apply_correction`）卻無 tx、無 audit、
//! 無 FOR UPDATE，權限用 `is_admin()` 硬編碼。本測試斷言：
//! - 核准後動物身分欄位確實更新
//! - 核准留下 `FIELD_CORRECTION_APPROVE` 稽核（修復前完全沒有）
//! - 拒絕不改動物欄位

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

/// 種一個 staff 帳號當 animals.created_by FK。
async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'fc requester', true, false)"#,
    )
    .bind(id)
    .bind(format!("fc-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 種一隻耳號 001 的動物。
async fn seed_animal(app: &TestApp, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, created_by)
           VALUES ($1, '001', 'miniature', 'male', '2026-01-01', $2)"#,
    )
    .bind(id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    id
}

async fn fc_approve_audit_count(app: &TestApp, animal_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'FIELD_CORRECTION_APPROVE' AND entity_type = 'animal' AND entity_id = $1",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count FIELD_CORRECTION_APPROVE audit")
}

async fn create_field_correction(
    app: &TestApp,
    animal_id: Uuid,
    field_name: &str,
    new_value: &str,
    token: &str,
) -> String {
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/field-corrections"),
            &serde_json::json!({ "field_name": field_name, "new_value": new_value, "reason": "建檔誤植" }),
            token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "建立修正申請應成功");
    let body: serde_json::Value = res.json().await.expect("parse create body");
    body["id"].as_str().expect("request id").to_string()
}

async fn create_correction(app: &TestApp, animal_id: Uuid, new_value: &str, token: &str) -> String {
    create_field_correction(app, animal_id, "ear_tag", new_value, token).await
}

// ── 核准：動物欄位更新 + 留稽核 ──
#[tokio::test]
#[serial]
async fn approve_field_correction_changes_animal_and_writes_audit() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let u = seed_user(&app).await;
    let aid = seed_animal(&app, u).await;

    let request_id = create_correction(&app, aid, "002", &token).await;

    let review = app
        .auth_post(
            &format!("/api/v1/animals/animal-field-corrections/{request_id}/review"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert_eq!(review.status().as_u16(), 200, "核准應成功");

    let ear_tag: String = sqlx::query_scalar("SELECT ear_tag FROM animals WHERE id = $1")
        .bind(aid)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch ear_tag");
    assert_eq!(ear_tag, "002", "核准後動物耳號應更新為 002");

    assert_eq!(
        fc_approve_audit_count(&app, aid).await,
        1,
        "核准動物欄位修正必須留下一筆 FIELD_CORRECTION_APPROVE 稽核（修復前完全沒有）"
    );
}

// ── 進場日期可申請修正（migration 025 放寬 chk_field_name）──
//
// 動物走 API 建立（才會自動寫入初始體重紀錄）。要釘住：
// - 申請記下舊值、核准後動物的進場日期更新、留稽核
// - 初始體重紀錄的量測日**不連動**（使用者 2026-11-12 裁定，體重紀錄是獨立的量測事實）
#[tokio::test]
#[serial]
async fn approve_entry_date_correction_changes_animal_but_not_initial_weight() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let created = common::create_animal_with_free_ear_tag(&app, &token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "gender": "male",
            "breed": "minipig",
            "entry_date": "2026-07-13",
            "entry_weight": 25.5,
            "pen_location": "A-01",
            "force_create": true,
        })
    })
    .await;
    let aid = Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid");

    let weight_date_before: String =
        sqlx::query_scalar("SELECT measure_date::text FROM animal_weights WHERE animal_id = $1")
            .bind(aid)
            .fetch_one(&app.db_pool)
            .await
            .expect("建立動物時應自動寫入一筆初始體重");
    assert_eq!(weight_date_before, "2026-07-13");

    let request_id = create_field_correction(&app, aid, "entry_date", "2026-07-01", &token).await;

    let old_value: Option<String> =
        sqlx::query_scalar("SELECT old_value FROM animal_field_correction_requests WHERE id = $1")
            .bind(Uuid::parse_str(&request_id).expect("request uuid"))
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch old_value");
    assert_eq!(
        old_value.as_deref(),
        Some("2026-07-13"),
        "申請應記下修正前的進場日期"
    );

    let review = app
        .auth_post(
            &format!("/api/v1/animals/animal-field-corrections/{request_id}/review"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert_eq!(review.status().as_u16(), 200, "核准應成功");

    let entry_date: String =
        sqlx::query_scalar("SELECT entry_date::text FROM animals WHERE id = $1")
            .bind(aid)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch entry_date");
    assert_eq!(entry_date, "2026-07-01", "核准後動物進場日期應更新");

    let weight_date_after: String =
        sqlx::query_scalar("SELECT measure_date::text FROM animal_weights WHERE animal_id = $1")
            .bind(aid)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch weight date");
    assert_eq!(
        weight_date_after, "2026-07-13",
        "初始體重紀錄的量測日不應被連動修改"
    );

    assert_eq!(
        fc_approve_audit_count(&app, aid).await,
        1,
        "核准必須留下稽核"
    );
}

#[tokio::test]
#[serial]
async fn entry_date_correction_rejects_bad_format() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let u = seed_user(&app).await;
    let aid = seed_animal(&app, u).await;

    let res = app
        .auth_post(
            &format!("/api/v1/animals/{aid}/field-corrections"),
            &serde_json::json!({ "field_name": "entry_date", "new_value": "01/05/2026", "reason": "建檔誤植" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "日期格式不對應回 400");
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("進場日期格式"),
        "錯誤訊息應說明原因，實際 {text}"
    );
}

// ── 拒絕：不改動物欄位 ──
#[tokio::test]
#[serial]
async fn reject_field_correction_keeps_animal_unchanged() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let u = seed_user(&app).await;
    let aid = seed_animal(&app, u).await;

    let request_id = create_correction(&app, aid, "099", &token).await;

    let review = app
        .auth_post(
            &format!("/api/v1/animals/animal-field-corrections/{request_id}/review"),
            &serde_json::json!({ "approved": false, "reject_reason": "不予更正" }),
            &token,
        )
        .await;
    assert_eq!(review.status().as_u16(), 200, "拒絕應成功");

    let ear_tag: String = sqlx::query_scalar("SELECT ear_tag FROM animals WHERE id = $1")
        .bind(aid)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch ear_tag");
    assert_eq!(ear_tag, "001", "拒絕不應改動物耳號");
}
