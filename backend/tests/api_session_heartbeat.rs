//! Regression test for sliding session heartbeat (F1).
//!
//! Before 2026-05-18 the heartbeat handler erroneously passed `user_id` to
//! `SessionManager::update_activity(session_id, ip)`, so the SQL
//! `WHERE id = $1` never matched any session and sliding session was silently
//! broken. This test asserts that a real heartbeat call updates the most
//! recently started session's `last_activity_at` AND increments
//! `page_view_count` — both proxies for "the UPDATE actually affected rows".
//!
//! If this test ever fails again, sliding session is dead and users will get
//! kicked at the absolute timeout / on any backend cleanup_expired tick.

mod common;

use serial_test::serial;
use std::time::Duration;

#[derive(sqlx::FromRow, Debug)]
struct SessionRow {
    last_activity_at: chrono::DateTime<chrono::Utc>,
    page_view_count: i32,
}

#[tokio::test]
#[serial]
async fn heartbeat_updates_last_activity_and_page_view_count() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // Snapshot the just-created session
    let before: SessionRow = sqlx::query_as(
        r#"
        SELECT last_activity_at, page_view_count
        FROM user_sessions
        WHERE is_active = true
        ORDER BY started_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("freshly logged-in session must exist");

    // Ensure measurable time passes so DESC comparison is unambiguous
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Call heartbeat
    let res = app
        .auth_post("/api/v1/auth/heartbeat", &serde_json::json!({}), &token)
        .await;
    assert_eq!(
        res.status(),
        200,
        "heartbeat must return 200, got {}",
        res.status()
    );

    let after: SessionRow = sqlx::query_as(
        r#"
        SELECT last_activity_at, page_view_count
        FROM user_sessions
        WHERE is_active = true
        ORDER BY started_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("session must still exist after heartbeat");

    assert!(
        after.last_activity_at > before.last_activity_at,
        "heartbeat must move last_activity_at forward (before={:?}, after={:?})",
        before.last_activity_at,
        after.last_activity_at
    );
    assert!(
        after.page_view_count > before.page_view_count,
        "heartbeat must increment page_view_count (before={}, after={})",
        before.page_view_count,
        after.page_view_count
    );
}
