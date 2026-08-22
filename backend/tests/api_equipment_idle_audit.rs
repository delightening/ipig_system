//! R71-3 設備閒置申請核准：交易原子性 + 稽核軌跡回歸測試。
//!
//! 修復前：approve_idle_request 連動改設備狀態 + 寫狀態 log，卻 SELECT→UPDATE×2
//! →INSERT 各自對 pool 散打（無 tx、無 FOR UPDATE、無 audit、未用 ActorContext）。
//! 本測試斷言核准後 (1) 申請=approved (2) 設備=inactive (3) 留 IDLE_REQUEST_APPROVE 稽核。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'idle applicant', true, false)"#,
    )
    .bind(id)
    .bind(format!("idle-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_equipment(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO equipment (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(format!("離心機-{}", &id.to_string()[..6]))
        .execute(&app.db_pool)
        .await
        .expect("insert equipment");
    id
}

/// 種一筆 pending 的閒置申請，回傳 request id。
async fn seed_idle_request(app: &TestApp, equipment_id: Uuid, applied_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO equipment_idle_requests (id, equipment_id, request_type, reason, status, applied_by)
           VALUES ($1, $2, 'idle', '長期停用', 'pending', $3)"#,
    )
    .bind(id)
    .bind(equipment_id)
    .bind(applied_by)
    .execute(&app.db_pool)
    .await
    .expect("insert idle request");
    id
}

/// 取測試 admin 的 user id（自核 SoD 測試需要「申請人 = 核准人」為同一人）。
async fn admin_user_id(app: &TestApp) -> Uuid {
    let email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin user id")
}

async fn idle_approve_audit_count(app: &TestApp, request_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'IDLE_REQUEST_APPROVE' AND entity_type = 'equipment_idle_request' AND entity_id = $1",
    )
    .bind(request_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count IDLE_REQUEST_APPROVE audit")
}

// ── 核准：申請→approved、設備→inactive、留稽核 ──
#[tokio::test]
#[serial]
async fn approve_idle_request_updates_equipment_and_writes_audit() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let applicant = seed_user(&app).await;
    let equipment_id = seed_equipment(&app).await;
    let request_id = seed_idle_request(&app, equipment_id, applicant).await;

    let res = app
        .auth_post(
            &format!("/api/v1/equipment-idle-requests/{request_id}/approve"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "核准閒置申請應成功");

    let req_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment_idle_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch request status");
    assert_eq!(req_status, "approved", "核准後申請狀態應為 approved");

    let equip_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch equipment status");
    assert_eq!(equip_status, "inactive", "核准閒置後設備狀態應為 inactive");

    assert_eq!(
        idle_approve_audit_count(&app, request_id).await,
        1,
        "核准閒置申請必須留下一筆 IDLE_REQUEST_APPROVE 稽核（修復前完全沒有）"
    );
}

// ── 拒絕：申請→rejected、設備狀態不變 ──
#[tokio::test]
#[serial]
async fn reject_idle_request_keeps_equipment_active() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let applicant = seed_user(&app).await;
    let equipment_id = seed_equipment(&app).await;
    let request_id = seed_idle_request(&app, equipment_id, applicant).await;

    let res = app
        .auth_post(
            &format!("/api/v1/equipment-idle-requests/{request_id}/approve"),
            &serde_json::json!({ "approved": false, "rejection_reason": "尚需使用" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "駁回應成功");

    let req_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment_idle_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch request status");
    assert_eq!(req_status, "rejected", "駁回後申請狀態應為 rejected");

    let equip_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch equipment status");
    assert_eq!(equip_status, "active", "駁回不應改設備狀態");
}

// ── SEC-SoD（資安稽核 M-1）：申請人不得核准自己的閒置申請 ──
#[tokio::test]
#[serial]
async fn approve_idle_request_rejects_self_approval() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    // 申請人 = 核准人（admin 本人），觸發職權分離守衛
    let admin_id = admin_user_id(&app).await;
    let equipment_id = seed_equipment(&app).await;
    let request_id = seed_idle_request(&app, equipment_id, admin_id).await;

    let res = app
        .auth_post(
            &format!("/api/v1/equipment-idle-requests/{request_id}/approve"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "申請人核准自己的閒置申請必須被職權分離守衛擋下（403）"
    );

    // 自核被擋後：申請維持 pending、設備維持 active、無核准稽核
    let req_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment_idle_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch request status");
    assert_eq!(req_status, "pending", "自核被擋後申請應維持 pending");

    let equip_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch equipment status");
    assert_eq!(equip_status, "active", "自核被擋後設備狀態不應改變");

    assert_eq!(
        idle_approve_audit_count(&app, request_id).await,
        0,
        "自核被擋不應留下 IDLE_REQUEST_APPROVE 稽核"
    );
}
