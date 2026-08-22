//! 回歸測試：DIRECTOR 角色指派提權缺口（2026-08-13 發現於組織/人事授權設計討論）。
//!
//! `ADMIN_STAFF` 持有 `admin.user.edit`（`PUT /api/v1/users/{id}` 的守衛權限），但
//! `validate_role_assignment` 修補前只擋 SYSTEM_ADMIN / legacy `admin` 兩種管理員層級，
//! 完全不擋 DIRECTOR——等於任何行政人員都能把終審簽核權（DIRECTOR）指派給任意使用者、
//! 包括自己。本測試鎖住修補後的行為：
//! - ADMIN_STAFF 指派 DIRECTOR → 403
//! - DIRECTOR 指派 DIRECTOR → 成功（使用者裁定：可解「全系統只有一位負責人」的單點問題）
//! - admin（legacy）指派 DIRECTOR → 成功（既有管理員層級路徑不受影響）

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::UpdateUserRequest;
use erp_backend::services::{AuthService, UserService};
use erp_backend::AppError;

fn actor(id: Uuid) -> CurrentUser {
    CurrentUser {
        id,
        email: format!("director-privesc-{id}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test-jti".to_string(),
        exp: 9999999999,
        impersonated_by: None,
    }
}

/// 建立一個可登入使用者並指派指定角色，回傳 id。
async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!(
        "director-privesc-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("director privesc {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    id
}

/// 建立一個無角色的目標使用者（被指派 DIRECTOR 的對象）。
async fn seed_target_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!(
        "director-privesc-target-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("director privesc target {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert target user");
    id
}

async fn director_role_id(app: &TestApp) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM roles WHERE code = 'DIRECTOR' AND is_active = true LIMIT 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("DIRECTOR role must exist after startup migrations")
}

fn role_ids_req(role_ids: Vec<Uuid>) -> UpdateUserRequest {
    UpdateUserRequest {
        email: None,
        display_name: None,
        phone: None,
        phone_ext: None,
        organization: None,
        entry_date: None,
        position: None,
        aup_roles: None,
        years_experience: None,
        trainings: None,
        is_internal: None,
        is_active: None,
        role_ids: Some(role_ids),
        expires_at: None,
        version: None,
        force_role_change: None,
    }
}

// ── ADMIN_STAFF 不得指派 DIRECTOR（本次修補的缺口本體）───────────────────────
#[tokio::test]
#[serial]
async fn admin_staff_cannot_assign_director_role() {
    let app = TestApp::spawn().await;

    let admin_staff = seed_login_user(&app, "adminstaff", "ADMIN_STAFF").await;
    let target = seed_target_user(&app, "victim1").await;
    let director_role = director_role_id(&app).await;

    let req = role_ids_req(vec![director_role]);
    let err = UserService::update(
        &app.db_pool,
        &ActorContext::User(actor(admin_staff)),
        target,
        &req,
    )
    .await
    .expect_err("ADMIN_STAFF 指派 DIRECTOR 應被拒絕（守衛失效時會成功）");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );

    let has_director: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.code = 'DIRECTOR'
        )"#,
    )
    .bind(target)
    .fetch_one(&app.db_pool)
    .await
    .expect("check target roles");
    assert!(
        !has_director,
        "守衛失效時目標使用者會被靜默授予 DIRECTOR 角色"
    );
}

// ── DIRECTOR 可指派 DIRECTOR（使用者裁定：解單點問題）──────────────────────
#[tokio::test]
#[serial]
async fn director_can_assign_director_role() {
    let app = TestApp::spawn().await;

    let director = seed_login_user(&app, "director", "DIRECTOR").await;
    let target = seed_target_user(&app, "successor").await;
    let director_role = director_role_id(&app).await;

    let req = role_ids_req(vec![director_role]);
    UserService::update(
        &app.db_pool,
        &ActorContext::User(actor(director)),
        target,
        &req,
    )
    .await
    .expect("DIRECTOR 指派 DIRECTOR 給別人應成功（使用者裁定可解單點）");

    let has_director: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.code = 'DIRECTOR'
        )"#,
    )
    .bind(target)
    .fetch_one(&app.db_pool)
    .await
    .expect("check target roles");
    assert!(has_director, "目標使用者應已被授予 DIRECTOR 角色");
}

// ── 對照：legacy admin 指派 DIRECTOR 不受本次修補影響（既有管理員層級路徑）──
#[tokio::test]
#[serial]
async fn legacy_admin_can_still_assign_director_role() {
    let app = TestApp::spawn().await;

    let admin = seed_login_user(&app, "legacyadmin", "admin").await;
    let target = seed_target_user(&app, "victim2").await;
    let director_role = director_role_id(&app).await;

    let req = role_ids_req(vec![director_role]);
    UserService::update(
        &app.db_pool,
        &ActorContext::User(actor(admin)),
        target,
        &req,
    )
    .await
    .expect("legacy admin 指派 DIRECTOR 應維持成功，不受本次修補影響");
}
