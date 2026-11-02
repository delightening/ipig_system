//! PDF 服務告警信的收件者查詢（handlers::health::find_alert_admin_email）。
//!
//! 回歸：這條查詢曾寫成 `users.role = 'ADMIN'`，但 `users` 表沒有 `role` 欄位
//! （角色在 user_roles／roles），查詢一律出錯，告警信永遠寄不出去。
//! 斷言：查詢可執行；有啟用中管理員時回傳的 email 確實屬於持有管理員角色的帳號。

mod common;

use common::TestApp;
use erp_backend::handlers::health::find_alert_admin_email;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn alert_recipient_query_runs_and_returns_an_admin() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let id = Uuid::new_v4();
    let email = format!("alert-{}@test.local", &Uuid::new_v4().to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'alert admin', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .execute(pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) \
         SELECT $1, id FROM roles WHERE code IN ('admin', 'SYSTEM_ADMIN') LIMIT 1",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("assign admin role");

    let found = find_alert_admin_email(pool)
        .await
        .expect("query must not error")
        .expect("an active admin exists");

    let is_admin: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users u \
           JOIN user_roles ur ON ur.user_id = u.id \
           JOIN roles r ON r.id = ur.role_id \
          WHERE u.email = $1 AND r.code IN ('admin', 'SYSTEM_ADMIN'))",
    )
    .bind(&found)
    .fetch_one(pool)
    .await
    .expect("role lookup");
    assert!(is_admin, "returned recipient must hold an admin role");
}
