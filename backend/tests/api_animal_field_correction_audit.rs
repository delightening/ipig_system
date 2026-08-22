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

async fn create_correction(app: &TestApp, animal_id: Uuid, new_value: &str, token: &str) -> String {
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/field-corrections"),
            &serde_json::json!({ "field_name": "ear_tag", "new_value": new_value, "reason": "建檔誤植" }),
            token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "建立修正申請應成功");
    let body: serde_json::Value = res.json().await.expect("parse create body");
    body["id"].as_str().expect("request id").to_string()
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
