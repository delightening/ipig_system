//! R75-P4：protocol 族補齊——`Scoped<ProtocolView>` / `Scoped<ProtocolEdit>` 授權契約測試。
//!
//! get_protocol 改 `Scoped<ProtocolView>`（= `require_protocol_view_access`）、
//! update_protocol 改 `Scoped<ProtocolEdit>`（= `require_protocol_edit` = `can_edit_protocol`）。
//! 鎖定兩個 marker 唯一建構路徑 `authorize` 的授權契約（handler 串接由編譯保證）：
//! 無關使用者 → Forbidden、PI/co-editor → Ok、不存在計畫 → NotFound。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use erp_backend::AppError;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, perms: &[&str], roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "scoped-ve@test.local".to_string(),
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
    .bind(format!("scopedve-{}-{}@test.local", suffix, &id.to_string()[..8]))
    .bind(format!("ScopedVE {suffix}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol_with_pi(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'scoped view/edit protocol', 'DRAFT'::protocol_status, $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-VE-{unique}"))
    .bind(format!("IACUC-VE-{unique}"))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

async fn add_client_member(app: &TestApp, protocol_id: Uuid, user_id: Uuid) {
    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols");
}

// ── ProtocolView ─────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn protocol_view_scope_denies_unrelated_user() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol_with_pi(&app, owner).await;

    // 無 view_all、非 PI、非成員 → 不可檢視
    let outsider = make_user(seed_user_row(&app, "outsider").await, &[], &[]);
    let result =
        access::Scoped::<access::ProtocolView>::authorize(&app.db_pool, &outsider, pid).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "無關使用者應得 Forbidden，實得：{:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn protocol_view_scope_allows_pi() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol_with_pi(&app, owner).await;

    // PI 本人（current_user.id == pi_user_id 短路）→ 放行
    let pi = make_user(owner, &[], &["PI"]);
    access::Scoped::<access::ProtocolView>::authorize(&app.db_pool, &pi, pid)
        .await
        .expect("計畫 PI 應可取得 ProtocolView 證明");
}

#[tokio::test]
#[serial]
async fn protocol_view_scope_missing_protocol_is_not_found() {
    let app = TestApp::spawn().await;
    let user = make_user(seed_user_row(&app, "any").await, &[], &["REVIEWER"]);
    let result =
        access::Scoped::<access::ProtocolView>::authorize(&app.db_pool, &user, Uuid::new_v4())
            .await;
    assert!(
        matches!(result, Err(AppError::NotFound(_))),
        "不存在的計畫應得 NotFound，實得：{:?}",
        result.err()
    );
}

// ── ProtocolEdit ─────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn protocol_edit_scope_denies_unrelated_user() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol_with_pi(&app, owner).await;

    // 無 edit 權、非 PI/co-editor/SD、非補登管理者 → 不可編輯
    let outsider = make_user(seed_user_row(&app, "outsider").await, &[], &[]);
    let result =
        access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &outsider, pid).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "無關使用者應得 Forbidden，實得：{:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn protocol_edit_scope_denies_client_member() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol_with_pi(&app, owner).await;

    // 草稿編輯權收緊為 PI/SD/admin → CLIENT 成員（唯讀）不放行（對齊原始 spec §4.1）。
    let editor_id = seed_user_row(&app, "client").await;
    add_client_member(&app, pid, editor_id).await;
    let editor = make_user(editor_id, &[], &[]);
    let result =
        access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &editor, pid).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "CLIENT 成員不應取得 ProtocolEdit 證明，實得：{:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn protocol_edit_scope_allows_pi() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol_with_pi(&app, owner).await;

    // PI（pi_user_id）即使無 aup.protocol.edit 權限 → 可取得 ProtocolEdit 證明。
    let pi = make_user(owner, &[], &[]);
    access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &pi, pid)
        .await
        .expect("PI 應可取得 ProtocolEdit 證明");
}
