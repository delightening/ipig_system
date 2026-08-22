//! R71-2 PI 開通信核准寄送：交易/冪等 + 稽核軌跡回歸測試。
//!
//! 修復前：approve_send_pi_invite 業務邏輯 + raw SQL 混在 handler，無 tx、無 audit、
//! is_admin() 硬編碼。修復後：下沉 service + tx + FOR UPDATE 冪等 + PI_INVITE_SEND 稽核。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

/// 種一個外部 PI 使用者，回傳 (id, email)。
async fn seed_user(app: &TestApp) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!("pi-{}@external.example", &Uuid::new_v4().to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', '外部 PI', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    (id, email)
}

async fn seed_protocol(app: &TestApp, pi: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let uniq = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'PI 邀請測試計畫', 'APPROVED', $4, $4)"#,
    )
    .bind(id)
    .bind(format!("P-PI-{uniq}"))
    .bind(format!("PIG-{uniq}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// 種一筆 pending 開通信；email 須等同 PI 使用者 email（forgot_password 以 email 查 user）。
async fn seed_pending_invite(app: &TestApp, protocol_id: Uuid, pi: Uuid, email: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO pi_account_invites (id, protocol_id, pi_user_id, email, status, provisioned_by)
           VALUES ($1, $2, $3, $4, 'pending', $3)"#,
    )
    .bind(id)
    .bind(protocol_id)
    .bind(pi)
    .bind(email)
    .execute(&app.db_pool)
    .await
    .expect("insert pi_account_invite");
    id
}

async fn pi_invite_send_audit_count(app: &TestApp, invite_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'PI_INVITE_SEND' AND entity_type = 'pi_account_invite' AND entity_id = $1",
    )
    .bind(invite_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count PI_INVITE_SEND audit")
}

// ── 核准寄送：invite→sent、留稽核 ──
#[tokio::test]
#[serial]
async fn approve_send_pi_invite_marks_sent_and_writes_audit() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (pi, email) = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi).await;
    let invite_id = seed_pending_invite(&app, protocol_id, pi, &email).await;

    let res = app
        .auth_post(
            &format!("/api/v1/pi-account-invites/{invite_id}/approve-send"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "核准寄送應成功");

    let status: String = sqlx::query_scalar("SELECT status FROM pi_account_invites WHERE id = $1")
        .bind(invite_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch invite status");
    assert_eq!(status, "sent", "核准後 invite 狀態應為 sent");

    assert_eq!(
        pi_invite_send_audit_count(&app, invite_id).await,
        1,
        "核准寄送 PI 開通信必須留下一筆 PI_INVITE_SEND 稽核（修復前完全沒有）"
    );
}

// ── 冪等：第二次核准 → 422，不重寫稽核 ──
#[tokio::test]
#[serial]
async fn approve_send_pi_invite_is_idempotent() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (pi, email) = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi).await;
    let invite_id = seed_pending_invite(&app, protocol_id, pi, &email).await;

    let first = app
        .auth_post(
            &format!("/api/v1/pi-account-invites/{invite_id}/approve-send"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(first.status().as_u16(), 200, "首次核准應成功");

    let second = app
        .auth_post(
            &format!("/api/v1/pi-account-invites/{invite_id}/approve-send"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(
        second.status().as_u16(),
        422,
        "已寄送的開通信不應重複寄送（BusinessRule）"
    );

    assert_eq!(
        pi_invite_send_audit_count(&app, invite_id).await,
        1,
        "重複核准不應再寫稽核"
    );
}

/// 種一個可登入的使用者（真實密碼雜湊）；grant_perm 為 Some 時，建臨時角色授予該權限。
async fn seed_login_user(app: &TestApp, password: &str, grant_perm: Option<&str>) -> String {
    let uid = Uuid::new_v4();
    let email = format!("approver-{}@test.local", &Uuid::new_v4().to_string()[..8]);
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, '核准者', true, false)"#,
    )
    .bind(uid)
    .bind(&email)
    .bind(&hash)
    .execute(&app.db_pool)
    .await
    .expect("insert login user");

    if let Some(perm) = grant_perm {
        let role_id = Uuid::new_v4();
        sqlx::query("INSERT INTO roles (id, code, name) VALUES ($1, $2, '測試核准角色')")
            .bind(role_id)
            .bind(format!("TEST_ROLE_{}", &Uuid::new_v4().to_string()[..8]))
            .execute(&app.db_pool)
            .await
            .expect("insert role");
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) SELECT $1, id FROM permissions WHERE code = $2",
        )
        .bind(role_id)
        .bind(perm)
        .execute(&app.db_pool)
        .await
        .expect("grant permission");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
            .bind(uid)
            .bind(role_id)
            .execute(&app.db_pool)
            .await
            .expect("assign role");
    }
    email
}

// ── 非 admin 但具 aup.pi_invite.approve 權限 → 200（驗證可委派） ──
#[tokio::test]
#[serial]
async fn delegated_approver_can_send_pi_invite() {
    let app = TestApp::spawn().await;
    let (pi, email) = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi).await;
    let invite_id = seed_pending_invite(&app, protocol_id, pi, &email).await;

    let pw = "TestPassword123!";
    let approver_email = seed_login_user(&app, pw, Some("aup.pi_invite.approve")).await;
    let token = app
        .login(&approver_email, pw)
        .await
        .expect("approver login");

    let res = app
        .auth_post(
            &format!("/api/v1/pi-account-invites/{invite_id}/approve-send"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        200,
        "具 aup.pi_invite.approve 之非 admin 應可寄送"
    );
}

// ── 非 admin 且無權限 → 403（驗證守衛拒絕） ──
#[tokio::test]
#[serial]
async fn user_without_permission_is_forbidden() {
    let app = TestApp::spawn().await;
    let (pi, email) = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi).await;
    let invite_id = seed_pending_invite(&app, protocol_id, pi, &email).await;

    let pw = "TestPassword123!";
    let plain_email = seed_login_user(&app, pw, None).await;
    let token = app.login(&plain_email, pw).await.expect("user login");

    let res = app
        .auth_post(
            &format!("/api/v1/pi-account-invites/{invite_id}/approve-send"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "無 aup.pi_invite.approve 權限者應被拒（403）"
    );
}
