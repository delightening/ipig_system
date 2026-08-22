//! `GET /api/v1/my-projects` 純成員制回歸測試。
//!
//! 背景：EXPERIMENT_STAFF 角色帶有 `aup.protocol.view_all`（給「計畫書管理」全覽用），
//! 舊邏輯讓「我的計劃」也回傳全部。改為純成員制後，「我的計劃」只回成員 / SD / 被指派審查的計畫，
//! 不因 view_all 而看到全部（全覽請至「計畫書管理」）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

/// 建立可登入使用者 + 指定角色，回傳 (id, email)。
async fn seed_login_user(
    app: &TestApp,
    label: &str,
    role_code: &str,
    password: &str,
) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "myproj-{label}-{}@test.local",
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
    .bind(format!("myproj {label}"))
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

/// 建立最小 APPROVED 計畫，回傳 id。`sd` 為 study_director_user_id（None 則 NULL）。
async fn seed_protocol(app: &TestApp, pi: Uuid, sd: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, study_director_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'my-projects scope test', 'APPROVED'::protocol_status, $4, $5, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("MP-{unique}"))
    .bind(format!("MPIACUC-{unique}"))
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// EXPERIMENT_STAFF（具 view_all）的「我的計劃」只回自己為 SD 的計畫，不回他人計畫（純成員制）。
#[tokio::test]
#[serial]
async fn my_projects_experiment_staff_sees_only_membership_not_all() {
    let app = TestApp::spawn().await;

    let pw = "MyProj$Pw1";
    let (staff_id, email) = seed_login_user(&app, "staff", "EXPERIMENT_STAFF", pw).await;
    let other_pi = {
        // 另一位 PI（不可登入，僅供他人計畫的 pi_user_id）
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
               VALUES ($1, $2, 'fake', 'other pi', true, false)"#,
        )
        .bind(id)
        .bind(format!("myproj-otherpi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
        .execute(&app.db_pool)
        .await
        .expect("insert other pi");
        id
    };

    // 計畫 A：staff 為 SD（應出現）；計畫 B：與 staff 無關（不應出現）；計畫 C：staff 為 PI（pi_user_id，應出現）
    let mine = seed_protocol(&app, other_pi, Some(staff_id)).await;
    let not_mine = seed_protocol(&app, other_pi, None).await;
    let mine_as_pi = seed_protocol(&app, staff_id, None).await;

    let token = app.login(&email, pw).await.expect("staff login");
    let res = app.auth_get("/api/v1/my-projects", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("parse my-projects");
    let ids: Vec<String> = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    assert!(ids.contains(&mine.to_string()), "應包含 staff 為 SD 的計畫");
    assert!(
        ids.contains(&mine_as_pi.to_string()),
        "應包含 staff 為 PI（pi_user_id）的計畫"
    );
    assert!(
        !ids.contains(&not_mine.to_string()),
        "不應包含與 staff 無關的計畫（純成員制，不因 view_all 看全部）"
    );
}
