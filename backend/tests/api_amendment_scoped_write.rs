//! R75-P4：amendment 補齊——`Scoped<AmendmentWrite>` 授權契約測試。
//!
//! create / update / submit 變更申請改吃 `Scoped<AmendmentWrite>`
//! （= `require_amendment_write`：SYSTEM_ADMIN 短路 或 計畫 PI〔`user_protocols` PI 角色〕）。
//! 鎖定唯一建構路徑 `authorize` 的授權契約（handler / service 串接由編譯保證）：
//! 無關使用者 → Forbidden、計畫 PI → Ok、管理者 → Ok。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use erp_backend::AppError;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "amd-scoped@test.local".to_string(),
        roles: roles.iter().map(|s| s.to_string()).collect(),
        permissions: vec![],
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
    .bind(format!("amdscoped-{}-{}@test.local", suffix, &id.to_string()[..8]))
    .bind(format!("AmdScoped {suffix}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'amendment scoped write protocol', 'APPROVED'::protocol_status, $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-AW-{unique}"))
    .bind(format!("IACUC-AW-{unique}"))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

/// 授權判定依 `user_protocols` PI 角色（非 `protocols.pi_user_id`）→ 須顯式建 PI 成員列。
async fn add_pi_membership(app: &TestApp, protocol_id: Uuid, user_id: Uuid) {
    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'PI')"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols PI");
}

// ── 無關使用者 → Forbidden ───────────────────────────────────────────────────
#[tokio::test]
#[serial]
async fn amendment_write_scope_denies_unrelated_user() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol(&app, owner).await;

    let outsider = make_user(seed_user_row(&app, "outsider").await, &[]);
    let result =
        access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &outsider, pid).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "非 PI / 非管理者應得 Forbidden，實得：{:?}",
        result.err()
    );
}

// ── 計畫 PI（user_protocols PI 角色）→ Ok ───────────────────────────────────
#[tokio::test]
#[serial]
async fn amendment_write_scope_allows_protocol_pi() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol(&app, owner).await;

    let pi_id = seed_user_row(&app, "pi").await;
    add_pi_membership(&app, pid, pi_id).await;
    let pi = make_user(pi_id, &[]);
    access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &pi, pid)
        .await
        .expect("計畫 PI 應可取得 AmendmentWrite 證明");
}

// ── SYSTEM_ADMIN → Ok（短路，免成員列）─────────────────────────────────────
#[tokio::test]
#[serial]
async fn amendment_write_scope_allows_admin() {
    let app = TestApp::spawn().await;
    let owner = seed_user_row(&app, "owner").await;
    let pid = seed_protocol(&app, owner).await;

    let admin = make_user(seed_user_row(&app, "admin").await, &["SYSTEM_ADMIN"]);
    access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &admin, pid)
        .await
        .expect("管理者應可取得 AmendmentWrite 證明（短路）");
}
