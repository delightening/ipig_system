//! R71-6 管理審查結案守衛（軟性 SoD）回歸測試。
//!
//! 修復前：update_management_review 的 status=COALESCE($4,status) 無守衛，任何持
//! glp.management_review.manage 編輯權者可經泛型 PUT 直接設 completed/closed 結案，
//! 繞過簽核。修復後：結案級狀態須具 glp.management_review.approve 權限。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::glp_compliance::UpdateManagementReviewRequest;
use erp_backend::services::GlpComplianceService;
use erp_backend::AppError;

/// 種一個真實 user（audit 的 actor_user_id FK 需存在），回傳 id。
async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'mgmt reviewer', true, false)"#,
    )
    .bind(id)
    .bind(format!("mr-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

fn actor_with(id: Uuid, perms: &[&str]) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: "x@test.local".into(),
        roles: vec![],
        permissions: perms.iter().map(|s| s.to_string()).collect(),
        jti: "t".into(),
        exp: 0,
        impersonated_by: None,
    })
}

fn req_status(s: &str) -> UpdateManagementReviewRequest {
    UpdateManagementReviewRequest {
        title: None,
        review_date: None,
        status: Some(s.to_string()),
        agenda: None,
        attendees: None,
        minutes: None,
        decisions: None,
        action_items: None,
        chaired_by: None,
    }
}

async fn seed_review(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO management_reviews (id, review_number, title, review_date, status)
           VALUES ($1, $2, '管理審查測試', '2026-06-01', 'planned')"#,
    )
    .bind(id)
    .bind(format!("MR-{}", &id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert management review");
    id
}

// ── manage-only 不能結案（completed） ──
#[tokio::test]
#[serial]
async fn manage_only_cannot_close_management_review() {
    let app = TestApp::spawn().await;
    let uid = seed_user(&app).await;
    let id = seed_review(&app).await;
    let actor = actor_with(uid, &["glp.management_review.manage"]);

    let err = GlpComplianceService::update_management_review(
        &app.db_pool,
        &actor,
        id,
        &req_status("completed"),
    )
    .await
    .expect_err("manage-only 設 completed 應被拒");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應回 Forbidden，實得 {err:?}"
    );
}

// ── manage-only 仍可做非終態更新（in_progress） ──
#[tokio::test]
#[serial]
async fn manage_only_can_progress_non_terminal() {
    let app = TestApp::spawn().await;
    let uid = seed_user(&app).await;
    let id = seed_review(&app).await;
    let actor = actor_with(uid, &["glp.management_review.manage"]);

    let after = GlpComplianceService::update_management_review(
        &app.db_pool,
        &actor,
        id,
        &req_status("in_progress"),
    )
    .await
    .expect("manage-only 設 in_progress 應成功");
    assert_eq!(after.status, "in_progress", "非終態更新不應被守衛擋下");
}

// ── 具 approve 權者可結案 ──
#[tokio::test]
#[serial]
async fn approver_can_close_management_review() {
    let app = TestApp::spawn().await;
    let uid = seed_user(&app).await;
    let id = seed_review(&app).await;
    let actor = actor_with(
        uid,
        &[
            "glp.management_review.manage",
            "glp.management_review.approve",
        ],
    );

    let after = GlpComplianceService::update_management_review(
        &app.db_pool,
        &actor,
        id,
        &req_status("completed"),
    )
    .await
    .expect("approver 設 completed 應成功");
    assert_eq!(after.status, "completed", "具 approve 權者應可結案");
}

// ── 已結案後 manage-only 不能修改內容（status=None 也擋） ──
#[tokio::test]
#[serial]
async fn manage_only_cannot_modify_completed_review() {
    let app = TestApp::spawn().await;
    let uid = seed_user(&app).await;
    let id = seed_review(&app).await;

    // approver 先結案
    let approver = actor_with(
        uid,
        &[
            "glp.management_review.manage",
            "glp.management_review.approve",
        ],
    );
    GlpComplianceService::update_management_review(
        &app.db_pool,
        &approver,
        id,
        &req_status("completed"),
    )
    .await
    .expect("結案應成功");

    // manage-only 嘗試改 title（不帶 status）
    let manage_only = actor_with(uid, &["glp.management_review.manage"]);
    let req = UpdateManagementReviewRequest {
        title: Some("竄改標題".to_string()),
        review_date: None,
        status: None,
        agenda: None,
        attendees: None,
        minutes: None,
        decisions: None,
        action_items: None,
        chaired_by: None,
    };
    let err = GlpComplianceService::update_management_review(&app.db_pool, &manage_only, id, &req)
        .await
        .expect_err("已結案審查不應允許 manage-only 修改內容");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應回 Forbidden，實得 {err:?}"
    );
}

// ── 已結案後 manage-only 不能降級狀態（completed → in_progress） ──
#[tokio::test]
#[serial]
async fn manage_only_cannot_demote_completed_review() {
    let app = TestApp::spawn().await;
    let uid = seed_user(&app).await;
    let id = seed_review(&app).await;

    let approver = actor_with(
        uid,
        &[
            "glp.management_review.manage",
            "glp.management_review.approve",
        ],
    );
    GlpComplianceService::update_management_review(
        &app.db_pool,
        &approver,
        id,
        &req_status("completed"),
    )
    .await
    .expect("結案應成功");

    let manage_only = actor_with(uid, &["glp.management_review.manage"]);
    let err = GlpComplianceService::update_management_review(
        &app.db_pool,
        &manage_only,
        id,
        &req_status("in_progress"),
    )
    .await
    .expect_err("已結案審查不應允許 manage-only 降級");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應回 Forbidden，實得 {err:?}"
    );
}
