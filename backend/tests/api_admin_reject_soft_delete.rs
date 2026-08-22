//! Admin 駁回通道 + 軟刪除整合測試（2026-06-12）。
//!
//! - 預審階段（SUBMITTED / PRE_REVIEW）駁回 = 系統管理員專用、理由必填。
//! - 軟刪除：限「已否決」計畫，設為 DELETED（保留資料列）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{ChangeStatusRequest, ProtocolStatus};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

fn user_actor(id: Uuid, role: &str) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: format!("{role}@test.local"),
        roles: vec![role.to_string()],
        // 給足泛型 change_status 的 handler 權限（service 層 admin 子守衛才是受測重點）
        permissions: vec!["aup.protocol.change_status".to_string()],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

async fn seed_pi(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'reject pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("rj-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert pi");
    id
}

/// 建立一筆真實 user row 供 audit `actor_user_id` FK（roles 由 CurrentUser 決定 is_admin）。
async fn seed_actor_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', 'actor', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("rj-actor-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert actor user");
    id
}

async fn seed_protocol(app: &TestApp, status: &str) -> Uuid {
    let pi = seed_pi(app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '駁回測試計畫', $4::protocol_status, $5, $5)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("APIG-{unique}"))
    .bind(status)
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

async fn status_of(app: &TestApp, id: Uuid) -> String {
    sqlx::query_scalar("SELECT status::text FROM protocols WHERE id = $1")
        .bind(id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query status")
}

fn reject_req(remark: Option<&str>) -> ChangeStatusRequest {
    ChangeStatusRequest {
        to_status: ProtocolStatus::Rejected,
        remark: remark.map(str::to_string),
        reviewer_ids: None,
        vet_id: None,
    }
}

// ── 預審駁回：非 admin → Forbidden ──
#[tokio::test]
#[serial]
async fn pre_review_reject_blocks_non_admin() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "SUBMITTED").await;
    let staff = user_actor(seed_actor_user(&app).await, "EXPERIMENT_STAFF");

    let err = ProtocolService::change_status(&app.db_pool, &staff, id, &reject_req(Some("理由")))
        .await
        .expect_err("非 admin 預審駁回應 Forbidden");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
    assert_eq!(status_of(&app, id).await, "SUBMITTED", "拒絕後狀態不變");
}

// ── 預審駁回：admin 但無理由 → Validation ──
#[tokio::test]
#[serial]
async fn pre_review_reject_requires_remark() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "PRE_REVIEW").await;
    let admin = user_actor(seed_actor_user(&app).await, "admin");

    let err = ProtocolService::change_status(&app.db_pool, &admin, id, &reject_req(None))
        .await
        .expect_err("admin 駁回未填理由應 Validation");
    assert!(
        matches!(err, AppError::Validation(_)),
        "應 Validation，實得：{err:?}"
    );
}

// ── 預審駁回：admin + 理由 → 成功，狀態 REJECTED ──
#[tokio::test]
#[serial]
async fn pre_review_reject_admin_ok() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "SUBMITTED").await;
    let admin = user_actor(seed_actor_user(&app).await, "admin");

    let p = ProtocolService::change_status(&app.db_pool, &admin, id, &reject_req(Some("誤送審")))
        .await
        .expect("admin 帶理由駁回應成功");
    assert_eq!(p.status, ProtocolStatus::Rejected);
    assert_eq!(status_of(&app, id).await, "REJECTED");
}

// ── 軟刪除：已否決 → 成功，狀態 DELETED，資料列保留 ──
#[tokio::test]
#[serial]
async fn soft_delete_rejected_ok() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "REJECTED").await;
    let admin = user_actor(seed_actor_user(&app).await, "admin");

    ProtocolService::soft_delete_protocol(&app.db_pool, &admin, id)
        .await
        .expect("已否決計畫應可軟刪除");

    assert_eq!(status_of(&app, id).await, "DELETED", "軟刪後狀態為 DELETED");
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)")
        .bind(id)
        .fetch_one(&app.db_pool)
        .await
        .expect("check exists");
    assert!(exists, "軟刪除應保留資料列");
}

// ── 軟刪除：非已否決（APPROVED）→ BusinessRule ──
#[tokio::test]
#[serial]
async fn soft_delete_non_rejected_blocked() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "APPROVED").await;
    let admin = user_actor(seed_actor_user(&app).await, "admin");

    let err = ProtocolService::soft_delete_protocol(&app.db_pool, &admin, id)
        .await
        .expect_err("非已否決計畫應拒絕軟刪除");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
    assert_eq!(status_of(&app, id).await, "APPROVED", "拒絕後狀態不變");
}

// ── 軟刪除：非 admin User → Forbidden（service 層縱深防護）──
#[tokio::test]
#[serial]
async fn soft_delete_blocks_non_admin() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "REJECTED").await;
    let staff = user_actor(seed_actor_user(&app).await, "EXPERIMENT_STAFF");

    let err = ProtocolService::soft_delete_protocol(&app.db_pool, &staff, id)
        .await
        .expect_err("非 admin 應拒絕軟刪除");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
    assert_eq!(status_of(&app, id).await, "REJECTED", "拒絕後狀態不變");
}

// ── 軟刪除：Anonymous → Forbidden ──
#[tokio::test]
#[serial]
async fn soft_delete_blocks_anonymous() {
    let app = TestApp::spawn().await;
    let id = seed_protocol(&app, "REJECTED").await;

    let err = ProtocolService::soft_delete_protocol(&app.db_pool, &ActorContext::Anonymous, id)
        .await
        .expect_err("Anonymous 應拒絕");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
}
