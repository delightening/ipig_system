//! T1 (review-followup): R34-3 amendment IDOR widening regression test.
//!
//! PR #340 / R34-3 changed `get_amendment` IDOR check from inline
//! `user_protocols` SELECT EXISTS to `access::require_protocol_related_access`.
//! The new helper accepts: PI (protocols.pi_user_id), user_protocols members,
//! review_assignments reviewers, AND vet_review_assignments vet_reviewers —
//! strictly broader than the original.
//!
//! This test locks the contract so future refactors of
//! `require_protocol_related_access` can't silently re-narrow (regression to
//! reviewers losing access) or over-widen (unrelated users gaining access).
//!
//! Approach: avoid the password-hash + login flow by calling the service
//! helper directly with constructed `CurrentUser` instances + seeded DB rows.

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use serial_test::serial;
use uuid::Uuid;

fn make_user(id: Uuid, email: &str) -> CurrentUser {
    CurrentUser {
        id,
        email: email.to_string(),
        roles: vec![],
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
    .bind(format!("idor-{}-{}@test.local", suffix, &id.to_string()[..8]))
    .bind(format!("IDOR Test {}", suffix))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol_with_pi(app: &TestApp, pi_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'IDOR test protocol', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-IDOR-{}", &pid.to_string()[..8]))
    .bind(format!("IACUC-IDOR-{}", &pid.to_string()[..8]))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

#[tokio::test]
#[serial]
async fn require_protocol_related_access_accepts_pi_via_pi_user_id() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    let pi = make_user(pi_id, "pi@test.local");
    let res = access::require_protocol_related_access(&app.db_pool, &pi, protocol_id).await;
    assert!(
        res.is_ok(),
        "PI via protocols.pi_user_id should be accepted: {:?}",
        res
    );
}

#[tokio::test]
#[serial]
async fn require_protocol_related_access_accepts_user_protocols_member() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let coinv_id = seed_user_row(&app, "co").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(coinv_id)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols");

    let coinv = make_user(coinv_id, "co@test.local");
    let res = access::require_protocol_related_access(&app.db_pool, &coinv, protocol_id).await;
    assert!(
        res.is_ok(),
        "CLIENT member in user_protocols should be accepted: {:?}",
        res
    );
}

#[tokio::test]
#[serial]
async fn require_protocol_related_access_accepts_review_assignment() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let reviewer_id = seed_user_row(&app, "rev").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    sqlx::query(
        r#"INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by)
           VALUES (gen_random_uuid(), $1, $2, $3)"#,
    )
    .bind(protocol_id)
    .bind(reviewer_id)
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert review_assignments");

    let reviewer = make_user(reviewer_id, "rev@test.local");
    let res = access::require_protocol_related_access(&app.db_pool, &reviewer, protocol_id).await;
    assert!(
        res.is_ok(),
        "Reviewer should be accepted (R34-3 widening): {:?}",
        res
    );
}

#[tokio::test]
#[serial]
async fn require_protocol_related_access_rejects_unrelated_user() {
    let app = TestApp::spawn().await;
    let pi_id = seed_user_row(&app, "pi").await;
    let stranger_id = seed_user_row(&app, "stranger").await;
    let protocol_id = seed_protocol_with_pi(&app, pi_id).await;

    let stranger = make_user(stranger_id, "x@test.local");
    let res = access::require_protocol_related_access(&app.db_pool, &stranger, protocol_id).await;
    assert!(
        res.is_err(),
        "Unrelated user must be rejected (no widening leak): {:?}",
        res
    );
}

#[tokio::test]
#[serial]
async fn require_protocol_related_access_rejects_user_on_different_protocol() {
    // 防止 vet_reviewer 跨 protocol IDOR：assigned 到 protocol A 不能讀 protocol B
    let app = TestApp::spawn().await;
    let pi_a = seed_user_row(&app, "pia").await;
    let pi_b = seed_user_row(&app, "pib").await;
    let reviewer = seed_user_row(&app, "rev").await;
    let protocol_a = seed_protocol_with_pi(&app, pi_a).await;
    let protocol_b = seed_protocol_with_pi(&app, pi_b).await;

    sqlx::query(
        r#"INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by)
           VALUES (gen_random_uuid(), $1, $2, $3)"#,
    )
    .bind(protocol_a)
    .bind(reviewer)
    .bind(pi_a)
    .execute(&app.db_pool)
    .await
    .expect("insert review_assignments");

    let user = make_user(reviewer, "rev@test.local");
    let res_a = access::require_protocol_related_access(&app.db_pool, &user, protocol_a).await;
    let res_b = access::require_protocol_related_access(&app.db_pool, &user, protocol_b).await;
    assert!(res_a.is_ok(), "Reviewer on protocol A should pass A");
    assert!(res_b.is_err(), "Reviewer on protocol A must NOT pass B");
}
