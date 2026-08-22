//! 計畫複製跨客戶 IDOR 回歸測試（R75-1，2026-06-18）。
//!
//! Bug：`copy_protocol` handler 僅檢查 `aup.protocol.create` / PI / admin 角色，
//! **不驗呼叫者能否檢視來源計畫**；service `ProtocolService::copy` 直接讀來源
//! `working_content` 並 `RETURNING *` 回傳 → 具 create 權的 PI 可複製**任一客戶**
//! 計畫並取得其完整內容（跨客戶外洩）。Claude+Codex 雙模型交叉確認。
//!
//! 修復：handler 在 can_create 後加 `access::require_protocol_related_access(source_id)`
//! ——僅「能檢視來源計畫者」（view_all 角色或該計畫成員）可複製。
//!
//! 測試層級：與 sibling `api_protocol_edit_idor.rs` 一致，鎖定 copy 所依賴的
//! `require_protocol_related_access` 授權契約（handler 串接由 code review + 編譯保證）。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, perms: &[&str], roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "copy@test.local".to_string(),
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
    .bind(format!("copyidor-{}-{}@test.local", suffix, &id.to_string()[..8]))
    .bind(format!("CopyIDOR {suffix}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol_with_pi(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'copy IDOR protocol', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-CI-{}", &pid.to_string()[..8]))
    .bind(format!("IACUC-CI-{}", &pid.to_string()[..8]))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

// ── R75-1 核心：具 create 權的 PI 但與來源計畫無關 → 拒絕（修復前 copy 會放行讀內容） ──
#[tokio::test]
#[serial]
async fn create_permission_pi_cannot_copy_unrelated_protocol() {
    let app = TestApp::spawn().await;
    let owner_id = seed_user_row(&app, "owner").await;
    let source_id = seed_protocol_with_pi(&app, owner_id).await;

    // 另一客戶的 PI：有 create 權 + PI 角色，但與 source 計畫無任何關聯
    let attacker = make_user(
        seed_user_row(&app, "attacker").await,
        &["aup.protocol.create"],
        &["PI"],
    );
    let err = access::require_protocol_related_access(&app.db_pool, &attacker, source_id)
        .await
        .expect_err("具 create 權但與來源計畫無關的 PI 應被拒絕");
    assert!(
        matches!(err, erp_backend::AppError::Forbidden(_)),
        "應回傳 Forbidden（而非把無關錯誤誤判為測試通過），實得：{err:?}"
    );
}

// ── view_all 角色 → 允許複製任意計畫 ──
#[tokio::test]
#[serial]
async fn view_all_can_copy_any_protocol() {
    let app = TestApp::spawn().await;
    let owner_id = seed_user_row(&app, "owner").await;
    let source_id = seed_protocol_with_pi(&app, owner_id).await;

    let staff = make_user(
        seed_user_row(&app, "staff").await,
        &["aup.protocol.create", "aup.protocol.view_all"],
        &[],
    );
    access::require_protocol_related_access(&app.db_pool, &staff, source_id)
        .await
        .expect("view_all 應可複製任意計畫");
}

// ── 來源計畫成員（co-editor）→ 允許複製自己相關的計畫 ──
#[tokio::test]
#[serial]
async fn protocol_member_can_copy_own_protocol() {
    let app = TestApp::spawn().await;
    let owner_id = seed_user_row(&app, "owner").await;
    let member_id = seed_user_row(&app, "member").await;
    let source_id = seed_protocol_with_pi(&app, owner_id).await;

    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(member_id)
    .bind(source_id)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols");

    let member = make_user(member_id, &["aup.protocol.create"], &["PI"]);
    access::require_protocol_related_access(&app.db_pool, &member, source_id)
        .await
        .expect("來源計畫成員應可複製自己相關的計畫");
}
