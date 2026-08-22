//! `GET /protocols/assignable-users` 回歸測試。
//!
//! 背景：匯入頁 PI / Study Director 下拉原本打 `/users`（需 `admin.user.view`），
//! 導致 EXPERIMENT_STAFF（具 `aup.protocol.import_approved` 但無 `admin.user.view`）
//! 取不到系統內 PI 名單。新端點門檻改為「同匯入權限」，讓匯入者皆能下拉選系統內 PI。
//!
//! 驗收：
//! - EXPERIMENT_STAFF（有 import_approved）→ 200，名單含系統內 PI。
//! - WAREHOUSE_MANAGER（無 import_approved、非 admin）→ 403。
//! - service 層 `list_assignable_users` 排除停用使用者並帶出角色。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::models::AssignableUser;
use erp_backend::services::{AuthService, UserService};

/// 建立一個「可登入」使用者並指派指定角色，回傳 (id, email)。
async fn seed_login_user(
    app: &TestApp,
    label: &str,
    role_code: &str,
    password: &str,
) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "assignable-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("assignable {label}"))
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
    (id, email)
}

/// 建立一個具 PI 角色、啟用中的最小使用者（PI 候選），回傳 id。
async fn seed_pi(app: &TestApp, active: bool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'assignable pi', $3, false)"#,
    )
    .bind(id)
    .bind(format!("assignable-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(active)
    .execute(&app.db_pool)
    .await
    .expect("insert pi user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    id
}

/// EXPERIMENT_STAFF（具 import_approved，無 admin.user.view）可取得名單，且名單含系統內 PI。
#[tokio::test]
#[serial]
async fn experiment_staff_can_list_assignable_users() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app, true).await;
    let pw = "Assignable$Pw1";
    let (_staff_id, email) = seed_login_user(&app, "staff", "EXPERIMENT_STAFF", pw).await;

    let token = app.login(&email, pw).await.expect("staff login");
    let res = app
        .auth_get("/api/v1/protocols/assignable-users", &token)
        .await;
    assert_eq!(res.status(), 200, "EXPERIMENT_STAFF 應可取得可指派名單");

    let users: Vec<AssignableUser> = TestApp::json(res).await;
    assert!(
        users
            .iter()
            .any(|u| u.id == pi && u.roles.iter().any(|r| r == "PI")),
        "名單應含系統內 PI（含角色）"
    );
}

/// 無 import_approved 權限且非 admin 的角色 → 403。
#[tokio::test]
#[serial]
async fn user_without_import_permission_is_forbidden() {
    let app = TestApp::spawn().await;
    let pw = "Assignable$Pw2";
    let (_id, email) = seed_login_user(&app, "wh", "WAREHOUSE_MANAGER", pw).await;

    let token = app.login(&email, pw).await.expect("warehouse login");
    let res = app
        .auth_get("/api/v1/protocols/assignable-users", &token)
        .await;
    assert_eq!(res.status(), 403, "無匯入權限者應被拒絕");
}

/// service 層：排除停用使用者，帶出角色。
#[tokio::test]
#[serial]
async fn list_assignable_users_excludes_inactive() {
    let app = TestApp::spawn().await;
    let active_pi = seed_pi(&app, true).await;
    let inactive_pi = seed_pi(&app, false).await;

    let users = UserService::list_assignable_users(&app.db_pool)
        .await
        .expect("list assignable users");

    assert!(users.iter().any(|u| u.id == active_pi), "啟用 PI 應在名單");
    assert!(
        !users.iter().any(|u| u.id == inactive_pi),
        "停用使用者不應在名單"
    );
}
