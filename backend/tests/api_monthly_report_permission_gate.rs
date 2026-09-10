//! 回歸測試：工時月報端點（`GET /hr/attendance/monthly-report` 與 `/export`）
//! 缺少 `hr.attendance.view` 基準門檻（CodeRabbit PR #35 指出）。
//!
//! `resolve_monthly_report_scope` 只在**沒有** `hr.attendance.view_all` 時把
//! `user_id` 收斂到自己，從未拒絕過請求——`PI` 這類不具 `hr.attendance.view`
//! 的角色（見 `startup/permissions.rs`，本系統 14 個角色裡有 8 個不含此碼）
//! 呼叫本端點只會拿到自己（不存在）的紀錄，不會被 403。
//!
//! 鎖住修補後的行為：
//! - `PI`（無 `hr.attendance.view`）→ 403
//! - `EXPERIMENT_STAFF`（有 `hr.attendance.view`，內部員工基本權限）→ 200

mod common;

use common::TestApp;
use erp_backend::services::AuthService;
use serial_test::serial;
use uuid::Uuid;

/// 建立一個可登入使用者並指派指定角色，回傳 email（供登入用）。
///
/// ⚠️ 密碼固定用 `iPig$ecure1`——與本檔其餘 login 測試一致（見
/// `api_director_role_privesc_regression.rs`），不是隨機值。
async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> String {
    let id = Uuid::new_v4();
    let email = format!(
        "monthly-report-gate-{label}-{}@test.local",
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
    .bind(format!("monthly report gate {label}"))
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
    email
}

#[tokio::test]
#[serial]
async fn pi_without_view_permission_is_forbidden() {
    let app = TestApp::spawn().await;
    let email = seed_login_user(&app, "pi", "PI").await;
    let token = app
        .login(&email, "iPig$ecure1")
        .await
        .expect("PI login should succeed");

    let res = app
        .auth_get(
            "/api/v1/hr/attendance/monthly-report?year=2026&month=1",
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        reqwest::StatusCode::FORBIDDEN,
        "PI 不具 hr.attendance.view，應被擋在權限門檻，不是拿到自己（空）的紀錄"
    );

    let res_export = app
        .auth_get(
            "/api/v1/hr/attendance/monthly-report/export?year=2026&month=1",
            &token,
        )
        .await;
    assert_eq!(
        res_export.status(),
        reqwest::StatusCode::FORBIDDEN,
        "匯出端點的授權門檻要與 JSON 端點一致"
    );
}

#[tokio::test]
#[serial]
async fn experiment_staff_with_view_permission_is_allowed() {
    let app = TestApp::spawn().await;
    let email = seed_login_user(&app, "staff", "EXPERIMENT_STAFF").await;
    let token = app
        .login(&email, "iPig$ecure1")
        .await
        .expect("EXPERIMENT_STAFF login should succeed");

    let res = app
        .auth_get(
            "/api/v1/hr/attendance/monthly-report?year=2026&month=1",
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        reqwest::StatusCode::OK,
        "EXPERIMENT_STAFF 持有 hr.attendance.view（內部員工基本權限），加門檻後仍要能查自己的月報"
    );
}
