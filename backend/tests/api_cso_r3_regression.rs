//! CSO 第三次 comprehensive sweep（2026-05-30）HIGH findings 回歸測試。
//!
//! 對應報告 `.gstack/security-reports/2026-05-30-comprehensive-r3.json`：
//! - #1 附件下載 / 列表 IDOR → `check_attachment_permission` 補物件層級存取檢查
//!   （持全域權限但非該計畫成員者不得跨計畫讀取附件）。
//! - #2 邀請流程旁路 `validate_role_assignment` → 邀請建立 / 接受端共用
//!   `access::require_authority_to_assign_roles`，非管理員不得指派 legacy `admin`。

mod common;

use chrono::{Duration, Utc};
use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::models::{AcceptInvitationRequest, CreateInvitationRequest};
use erp_backend::services::{AuthService, InvitationService};
use erp_backend::AppError;

// ── fixtures ──────────────────────────────────────────────────────────────

/// 建立一個「最小」使用者（不可登入，僅供 FK 參照，如 pi_user_id / invited_by）。
async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("cso-r3-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("cso r3 {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建立一個「可登入」使用者並指派指定角色，回傳 (id, email)。
async fn seed_login_user(
    app: &TestApp,
    label: &str,
    role_code: &str,
    password: &str,
) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "cso-r3-{label}-{}@test.local",
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
    .bind(format!("cso r3 {label}"))
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

/// 建立一個 APPROVED 計畫，pi_user_id = `pi`（即 `pi` 為該計畫成員）。
async fn seed_protocol(app: &TestApp, pi: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'CSO r3 attachment idor', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR3-{unique}"))
    .bind(format!("IACUC3-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// 在 attachments 表插入一筆 protocol 附件（entity_id = protocol_id）。
async fn seed_protocol_attachment(app: &TestApp, protocol_id: Uuid, uploaded_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO attachments (id, category, entity_type, entity_id, file_name, file_path, file_size, mime_type, uploaded_by)
           VALUES ($1, 'protocol_attachment', 'protocol', $2, 'doc.pdf', '/tmp/cso-r3-none.pdf', 10, 'application/pdf', $3)"#,
    )
    .bind(id)
    .bind(protocol_id)
    .bind(uploaded_by)
    .execute(&app.db_pool)
    .await
    .expect("insert attachment");
    id
}

// ── #1：附件下載 / 列表 IDOR — 非計畫成員不得跨計畫存取 ───────────────────────
#[tokio::test]
#[serial]
async fn attachment_access_blocks_cross_protocol_idor() {
    let app = TestApp::spawn().await;
    let pw = "iPig$ecure1";

    // attacker：PI 角色（持 aup.protocol.edit，但非 view_all），僅為 protocol_a 成員。
    let (attacker, attacker_email) = seed_login_user(&app, "attacker", "PI", pw).await;
    let victim = seed_user(&app, "victim").await;

    let protocol_a = seed_protocol(&app, attacker).await; // attacker 為成員
    let protocol_b = seed_protocol(&app, victim).await; // attacker 非成員

    let att_a = seed_protocol_attachment(&app, protocol_a, attacker).await;
    let att_b = seed_protocol_attachment(&app, protocol_b, victim).await;

    let token = app
        .login(&attacker_email, pw)
        .await
        .expect("attacker login 應成功");

    // 跨計畫下載 → 403（守衛失效時為 200）
    let res = app
        .auth_get(&format!("/api/v1/attachments/{att_b}"), &token)
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 403,
        "跨計畫附件下載必須 403，實得 {status}，body={body}"
    );

    // 跨計畫列表 → 403
    let res = app
        .auth_get(
            &format!("/api/v1/attachments?entity_type=protocol&entity_id={protocol_b}"),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "跨計畫附件列表必須 403，實得 {}",
        res.status()
    );

    // 對照：自己計畫的列表 → 200，且包含該附件（證明守衛未過度封鎖合法存取）
    let res = app
        .auth_get(
            &format!("/api/v1/attachments?entity_type=protocol&entity_id={protocol_a}"),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "自己計畫附件列表應 200");
    let body: Vec<serde_json::Value> = TestApp::json(res).await;
    assert!(
        body.iter()
            .any(|a| a["id"].as_str() == Some(att_a.to_string().as_str())),
        "自己計畫的列表應包含該附件"
    );
}

// ── #2：邀請建立端不得讓非管理員指派 legacy `admin` ─────────────────────────
#[tokio::test]
#[serial]
async fn invitation_create_blocks_admin_role_for_non_admin_inviter() {
    let app = TestApp::spawn().await;
    let config = erp_backend::config::Config::from_env().expect("config from env");

    let inviter = seed_user(&app, "inviter").await; // 無管理員層級角色
    let admin_role: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'admin'")
        .fetch_one(&app.db_pool)
        .await
        .expect("legacy admin role 必須存在");

    let email = format!(
        "cso-r3-invitee-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let req = CreateInvitationRequest {
        email: email.clone(),
        display_name: "Invitee".into(),
        organization: "Org".into(),
        phone: None,
        position: None,
        role_ids: vec![admin_role],
        is_internal: true,
    };
    let err = InvitationService::create(&app.db_pool, &config, &req, inviter)
        .await
        .expect_err("非管理員邀請人指派 admin 應被拒絕");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );

    // 守衛在 tx 前生效 → 不應殘留任何 admin 邀請
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.db_pool)
        .await
        .expect("count invitations");
    assert_eq!(count, 0, "守衛失效時會殘留 backdoor admin 邀請");

    // 對照：非管理員邀請人指派 PI（非管理員層級）角色應成功
    let pi_role = app.pi_role_id().await;
    let email_ok = format!(
        "cso-r3-invitee-ok-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let req_ok = CreateInvitationRequest {
        email: email_ok,
        display_name: "Invitee OK".into(),
        organization: "Org".into(),
        phone: None,
        position: None,
        role_ids: vec![pi_role],
        // PI 是外部角色，照實填 false。本測試驗的是角色指派授權、與 is_internal
        // 無關，但值仍應與情境一致，避免日後被當成「PI 可以是內部人員」的依據。
        is_internal: false,
    };
    InvitationService::create(&app.db_pool, &config, &req_ok, inviter)
        .await
        .expect("非管理員邀請人指派 PI 角色應成功");
}

// ── #2 深度防禦：接受端不得實現 admin 角色（原邀請人非管理員） ──────────────────
#[tokio::test]
#[serial]
async fn invitation_accept_blocks_admin_role_from_non_admin_inviter() {
    let app = TestApp::spawn().await;
    let config = erp_backend::config::Config::from_env().expect("config from env");

    let inviter = seed_user(&app, "inviter2").await; // 非管理員
    let admin_role: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'admin'")
        .fetch_one(&app.db_pool)
        .await
        .expect("legacy admin role 必須存在");

    // 直接寫入 pending 邀請 + admin 角色（模擬繞過 create 端，或邀請人建立後被降級）
    let token = format!("cso-r3-tok-{}", Uuid::new_v4());
    let email = format!(
        "cso-r3-accept-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let expires = Utc::now() + Duration::days(7);
    let inv_id: Uuid = sqlx::query_scalar(
        // is_internal 必須明確帶值（migration 154 撤掉了 DEFAULT）。本案斷言的是
        // 「接受端不得實現 admin 角色」與「不得建立 backdoor 帳號」，內外部無關，取 false。
        r#"INSERT INTO invitations (email, display_name, invitation_token, invited_by, expires_at, is_internal)
           VALUES ($1, 'Accept Test', $2, $3, $4, false) RETURNING id"#,
    )
    .bind(&email)
    .bind(&token)
    .bind(inviter)
    .bind(expires)
    .fetch_one(&app.db_pool)
    .await
    .expect("insert invitation");
    sqlx::query("INSERT INTO invitation_roles (invitation_id, role_id) VALUES ($1, $2)")
        .bind(inv_id)
        .bind(admin_role)
        .execute(&app.db_pool)
        .await
        .expect("insert invitation role");

    let req = AcceptInvitationRequest {
        invitation_token: token,
        display_name: "Accept Test".into(),
        phone: "0900000000".into(),
        organization: "Org".into(),
        password: "iPig$ecure1".into(),
        position: None,
        agree_terms: true,
    };
    let err = InvitationService::accept(&app.db_pool, &config, &req)
        .await
        .expect_err("接受端不可實現 admin 角色（原邀請人非管理員）");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );

    // 守衛在建立帳號 tx 前生效 → 不應建立 backdoor 帳號
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.db_pool)
        .await
        .expect("count users");
    assert_eq!(count, 0, "守衛失效時會建立 backdoor admin 帳號");
}
