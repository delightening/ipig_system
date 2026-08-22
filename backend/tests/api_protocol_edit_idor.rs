//! 計畫編輯授權 IDOR 回歸測試（code-review M2 + 草稿編輯權收緊）。
//!
//! 原 bug：update_protocol 僅憑 `aup.protocol.edit` 權限即放行 → IDOR。
//!
//! 現行政策（對齊原始 spec §4.1）：`can_edit_protocol` 收緊為 **admin / PI / SD**
//! （`aup.protocol.view_all` / `aup.protocol.edit` 權限與 CLIENT 成員身分皆不放行）。
//! 本測試鎖定該 helper 契約。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, perms: &[&str], roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "edit@test.local".to_string(),
        roles: roles.iter().map(|s| s.to_string()).collect(),
        permissions: perms.iter().map(|s| s.to_string()).collect(),
        jti: "test-jti".to_string(),
        exp: 9999999999,
        impersonated_by: None,
    }
}

async fn seed_user_row(app: &TestApp, suffix: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake-hash', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("editidor-{}-{}@test.local", suffix, &id.to_string()[..8]))
    .bind(format!("EditIDOR {suffix}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol_with_pi(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'edit IDOR protocol', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-EI-{}", &pid.to_string()[..8]))
    .bind(format!("IACUC-EI-{}", &pid.to_string()[..8]))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

// ── M2 核心：具編輯權限但與計畫無關 → 拒絕（修復前會放行） ──
#[tokio::test]
#[serial]
async fn edit_permission_without_relation_is_rejected() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    // 僅有 aup.protocol.edit、非 view_all、與此計畫無任何關聯
    let editor = make_user(
        seed_user_row(&app, "editor").await,
        &["aup.protocol.edit"],
        &[],
    );
    let allowed = access::can_edit_protocol(&app.db_pool, &editor, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(!allowed, "編輯權限者若與計畫無關，不得編輯（IDOR）");
}

// ── view_all + 編輯權限但非 PI/SD/admin → 現行政策拒絕（執秘唯讀） ──
#[tokio::test]
#[serial]
async fn view_all_without_pi_or_sd_is_rejected() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    // 持 view_all + edit 權限，但非該計畫 PI/SD、非系統管理員（如執秘）。
    let viewer = make_user(
        seed_user_row(&app, "viewer").await,
        &["aup.protocol.edit", "aup.protocol.view_all"],
        &[],
    );
    let allowed = access::can_edit_protocol(&app.db_pool, &viewer, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(
        !allowed,
        "view_all/edit 權限但非 PI/SD/admin 不得編輯內容（執秘唯讀）"
    );
}

// ── CLIENT 成員（唯讀）→ 現行政策拒絕編輯（編輯權收緊為 PI/SD） ──
#[tokio::test]
#[serial]
async fn client_member_is_rejected() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let member_id = seed_user_row(&app, "member").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(member_id)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols");

    let member = make_user(member_id, &["aup.protocol.edit"], &[]);
    let allowed = access::can_edit_protocol(&app.db_pool, &member, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(!allowed, "CLIENT 成員（唯讀）不具編輯權（收緊為 PI/SD）");
}

// ── PI（pi_user_id）即使無 edit 權限仍可編輯自己的計畫 ──
#[tokio::test]
#[serial]
async fn pi_without_edit_permission_is_allowed() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    let pi = make_user(pi_id, &[], &[]);
    let allowed = access::can_edit_protocol(&app.db_pool, &pi, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(allowed, "PI 即使無 edit 權限仍可編輯自己的計畫");
}

// ── SD（study_director_user_id）可編輯其負責的計畫 ──
#[tokio::test]
#[serial]
async fn study_director_is_allowed() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let sd_id = seed_user_row(&app, "sd").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;
    sqlx::query("UPDATE protocols SET study_director_user_id = $1 WHERE id = $2")
        .bind(sd_id)
        .bind(protocol_id)
        .execute(&app.db_pool)
        .await
        .expect("set SD");

    let sd = make_user(sd_id, &[], &[]);
    let allowed = access::can_edit_protocol(&app.db_pool, &sd, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(allowed, "SD 應可編輯其負責的計畫");
}

// ── 完全無關且無權限 → 拒絕 ──
#[tokio::test]
#[serial]
async fn unrelated_user_without_permission_is_rejected() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    let stranger = make_user(seed_user_row(&app, "stranger").await, &[], &[]);
    let allowed = access::can_edit_protocol(&app.db_pool, &stranger, protocol_id)
        .await
        .expect("can_edit_protocol 不應 error");
    assert!(!allowed, "無關聯且無權限者不得編輯");
}
