//! SEC-IDOR 回歸：MCP `list_protocols` 須對齊使用者可見範圍。
//!
//! 修補前 `list_protocols` 忽略呼叫者（參數 `_user`）、回傳全部計畫 →
//! 任何持 MCP 金鑰者可列舉所有計畫（水平提權）。本測試鎖定：
//! - 非 view_all 使用者只見其關聯計畫（PI / 成員 / 指派審查 / 指派獸醫）
//! - view_all 角色看全部
//!
//! `read_protocol` 走 `access::require_protocol_related_access`，其關聯語意已由
//! `api_amendment_idor_regression.rs` 鎖定，故此處不重複。
//!
//! 作法：直接呼叫 service helper（吃 &PgPool），避免 login / MCP 金鑰認證流程。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::mcp;
use serde_json::Value;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, perms: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: format!("mcpvis-{}@test.local", &id.to_string()[..8]),
        roles: vec![],
        permissions: perms.iter().map(|s| s.to_string()).collect(),
        jti: "test-jti".to_string(),
        exp: 9999999999,
        impersonated_by: None,
    }
}

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake-hash', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("mcpvis-{}@test.local", &id.to_string()[..8]))
    .bind(format!("MCP Vis {}", &id.to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'MCP vis test', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-MCPVIS-{}", &pid.to_string()[..8]))
    .bind(format!("IACUC-MCPVIS-{}", &pid.to_string()[..8]))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

fn ids(list: &[Value]) -> Vec<String> {
    list.iter()
        .filter_map(|v| v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()))
        .collect()
}

#[tokio::test]
#[serial]
async fn list_protocols_non_view_all_only_sees_related() {
    let app = TestApp::spawn().await;
    let pi_a = seed_user(&app).await;
    let pi_b = seed_user(&app).await;
    let prot_a = seed_protocol(&app, pi_a).await;
    let prot_b = seed_protocol(&app, pi_b).await;

    // pi_a 無 view_all：應只見自己的 prot_a，不得見 pi_b 的 prot_b
    let user_a = make_user(pi_a, &[]);
    let list = mcp::list_visible_protocols(&app.db_pool, &user_a, None, 1000)
        .await
        .expect("list");
    let got = ids(&list);
    assert!(got.contains(&prot_a.to_string()), "PI 應見自己的計畫");
    assert!(
        !got.contains(&prot_b.to_string()),
        "非 view_all 使用者不得見他人計畫（IDOR）: {got:?}"
    );
}

#[tokio::test]
#[serial]
async fn list_protocols_view_all_sees_all() {
    let app = TestApp::spawn().await;
    let pi_a = seed_user(&app).await;
    let pi_b = seed_user(&app).await;
    let prot_a = seed_protocol(&app, pi_a).await;
    let prot_b = seed_protocol(&app, pi_b).await;

    let viewer = make_user(Uuid::new_v4(), &["aup.protocol.view_all"]);
    let list = mcp::list_visible_protocols(&app.db_pool, &viewer, None, 1000)
        .await
        .expect("list");
    let got = ids(&list);
    assert!(got.contains(&prot_a.to_string()), "view_all 應見 prot_a");
    assert!(got.contains(&prot_b.to_string()), "view_all 應見 prot_b");
}

#[tokio::test]
#[serial]
async fn list_protocols_includes_assigned_reviewer() {
    let app = TestApp::spawn().await;
    let pi_a = seed_user(&app).await;
    let reviewer = seed_user(&app).await;
    let prot_a = seed_protocol(&app, pi_a).await;

    sqlx::query(
        r#"INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by)
           VALUES (gen_random_uuid(), $1, $2, $3)"#,
    )
    .bind(prot_a)
    .bind(reviewer)
    .bind(pi_a)
    .execute(&app.db_pool)
    .await
    .expect("insert review_assignments");

    let user = make_user(reviewer, &[]);
    let list = mcp::list_visible_protocols(&app.db_pool, &user, None, 1000)
        .await
        .expect("list");
    assert!(
        ids(&list).contains(&prot_a.to_string()),
        "被指派審查委員應見該計畫"
    );
}
