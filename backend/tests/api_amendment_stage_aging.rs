//! CodeRabbit #31：變更申請的 `since` 要算「進入目前這一關的時間」，不是原始送審時間。
//!
//! 一份變更申請可能被退回補件、重送、再進委員會審查好幾輪。`submitted_at`
//! 是最初送出的時間，從頭到尾不變；`resolve_amendment_assignees` 修復前
//! 把它當 `UNDER_REVIEW` / `REVISION_REQUIRED` 共用的起算點，等於案子繞了
//! 幾圈就多算幾圈的等待時間，觸發（誤導性）催辦。

mod common;
use chrono::{Duration, Utc};
use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::pending_owner;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

const PERM_CHANGE_STATUS: &str = "aup.protocol.change_status";

fn viewer_with_change_status() -> CurrentUser {
    CurrentUser {
        id: Uuid::new_v4(),
        email: "viewer@example.com".into(),
        roles: vec![],
        permissions: vec![PERM_CHANGE_STATUS.to_string()],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

/// 建一份很久以前送審、目前是 `UNDER_REVIEW` 的變更申請。回傳 amendment_id。
async fn seed_old_amendment_under_review(pool: &PgPool) -> Uuid {
    let suffix = Uuid::new_v4().simple().to_string();
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(user_id)
    .bind(format!("amd-aging-{}@example.com", &suffix[..8]))
    .bind(format!("時效測試-{}", &suffix[..4]))
    .execute(pool)
    .await
    .expect("seed user");

    let protocol_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by) \
         VALUES ($1, $2, $3, $4, $4)",
    )
    .bind(protocol_id)
    .bind(format!("AMDAGE-{}", &suffix[..8]))
    .bind("時效測試計畫")
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed protocol");

    let amendment_id = Uuid::new_v4();
    let long_ago = Utc::now() - Duration::days(30);
    sqlx::query(
        "INSERT INTO amendments (id, protocol_id, amendment_no, title, status, created_by, submitted_at) \
         VALUES ($1, $2, $3, $4, 'UNDER_REVIEW', $5, $6)",
    )
    .bind(amendment_id)
    .bind(protocol_id)
    .bind(format!("AMDAGE-{}-01", &suffix[..8]))
    .bind("時效測試變更申請")
    .bind(user_id)
    .bind(long_ago)
    .execute(pool)
    .await
    .expect("seed amendment");

    // 最近才轉進 UNDER_REVIEW（模擬：送審 → 分類 → 退回補件 → 重送 → 再進審查）。
    let recent_transition = Utc::now() - Duration::hours(2);
    sqlx::query(
        "INSERT INTO amendment_status_history (id, amendment_id, from_status, to_status, changed_by, created_at) \
         VALUES ($1, $2, 'RESUBMITTED'::amendment_status, 'UNDER_REVIEW'::amendment_status, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(amendment_id)
    .bind(user_id)
    .bind(recent_transition)
    .execute(pool)
    .await
    .expect("seed amendment_status_history transition");

    amendment_id
}

#[tokio::test]
#[serial]
async fn amendment_stage_aging_uses_last_transition_not_original_submission() {
    let app = TestApp::spawn().await;
    let amendment_id = seed_old_amendment_under_review(&app.db_pool).await;
    let recent_transition = Utc::now() - Duration::hours(2);
    let long_ago = Utc::now() - Duration::days(30);

    let owners = pending_owner::resolve_for_amendments(
        &app.db_pool,
        &[amendment_id],
        &viewer_with_change_status(),
    )
    .await
    .expect("resolve");
    let owner = owners
        .get(&amendment_id)
        .expect("UNDER_REVIEW 必須有待處理人");

    let since = owner.since.expect("since 應算得出來");
    let diff_from_transition = (since - recent_transition).num_seconds().abs();
    assert!(
        diff_from_transition < 5,
        "since 應該是最近一次轉進 UNDER_REVIEW 的時間，實際差了 {diff_from_transition} 秒：{since}"
    );
    let diff_from_submission = (since - long_ago).num_seconds().abs();
    assert!(
        diff_from_submission > 29 * 24 * 3600,
        "since 不該算回 30 天前的原始送審時間：{since}"
    );
}
