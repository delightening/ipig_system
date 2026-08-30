//! CodeRabbit #31：`REVISION_REQUIRED` 的變更申請不該從所有人的待處理數量裡消失。
//!
//! `PENDING_AMENDMENT_STATUSES`（修復前唯一的一份清單）只有
//! `SUBMITTED` / `RESUBMITTED` / `CLASSIFIED` / `UNDER_REVIEW`——這四個對 staff
//! 是對的（`REVISION_REQUIRED` 時球在申請人身上，staff 不用做事，不該算進
//! staff 的 triage badge），但同一份清單也被 `get_pending_count_for_user`
//! （申請人自己的待辦數）拿去用，於是申請人被退回補件的案子從自己的
//! badge 上憑空消失——那正是申請人最需要看到的待辦。
//!
//! ⚠️ **`get_pending_count`（staff 全域版）沒有任何範圍限制，是整張
//! `amendments` 表的 `COUNT(*)`。** `TestApp::spawn()`／`connect_disposable`
//! 只確認連到的是可安全破壞的丟棄庫，**不會**在測試之間 TRUNCATE——同一次
//! `cargo test` 進程裡跑過的其他測試留下的 amendments 列仍在表裡（2026-08-30
//! 實測：先後兩次 `cargo test` 疊加後，同一個斷言從差 1 變差 2）。
//! 所以本檔對 `get_pending_count` 一律比「呼叫前後的差值」，不斷言絕對值；
//! `get_pending_count_for_user` 本身已經用隨機 `user_id` 經 `user_protocols`
//! 過濾，範圍天生不會被其他測試污染，可以斷言絕對值。

mod common;
use common::TestApp;
use erp_backend::services::AmendmentService;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 建一位使用者、一個他是成員的計畫、一筆指定狀態的變更申請。
/// 回傳 (user_id, amendment_id)。
async fn seed_amendment_for_user(pool: &PgPool, status: &str) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(user_id)
    .bind(format!("amd-pc-{}@example.com", &suffix[..8]))
    .bind(format!("待處理數測試-{}", &suffix[..4]))
    .execute(pool)
    .await
    .expect("seed user");

    let protocol_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by) \
         VALUES ($1, $2, $3, $4, $4)",
    )
    .bind(protocol_id)
    .bind(format!("AMDPC-{}", &suffix[..8]))
    .bind("待處理數測試計畫")
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed protocol");

    sqlx::query(
        "INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol) VALUES ($1, $2, 'PI')",
    )
    .bind(user_id)
    .bind(protocol_id)
    .execute(pool)
    .await
    .expect("link member to protocol");

    let amendment_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO amendments (id, protocol_id, amendment_no, title, status, created_by, submitted_at) \
         VALUES ($1, $2, $3, $4, $5::amendment_status, $6, NOW())",
    )
    .bind(amendment_id)
    .bind(protocol_id)
    .bind(format!("AMDPC-{}-01", &suffix[..8]))
    .bind("待處理數測試變更申請")
    .bind(status)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed amendment");

    (user_id, amendment_id)
}

/// 申請人自己的待處理數要算進 `REVISION_REQUIRED`——那正是「我需要交修正版」。
#[tokio::test]
#[serial]
async fn user_pending_count_includes_revision_required() {
    let app = TestApp::spawn().await;
    let (user_id, _amendment) = seed_amendment_for_user(&app.db_pool, "REVISION_REQUIRED").await;

    let count = AmendmentService::get_pending_count_for_user(&app.db_pool, user_id)
        .await
        .expect("get_pending_count_for_user");

    assert_eq!(
        count, 1,
        "REVISION_REQUIRED 對申請人來說是待辦（需要交修正版），不該從自己的待處理數消失"
    );
}

/// staff 的全域 triage 數**不**算 `REVISION_REQUIRED`——球在申請人身上，staff 不用做事。
#[tokio::test]
#[serial]
async fn staff_pending_count_excludes_revision_required() {
    let app = TestApp::spawn().await;
    let before = AmendmentService::get_pending_count(&app.db_pool)
        .await
        .expect("get_pending_count before");

    let (_user_id, _amendment) = seed_amendment_for_user(&app.db_pool, "REVISION_REQUIRED").await;

    let after = AmendmentService::get_pending_count(&app.db_pool)
        .await
        .expect("get_pending_count after");

    assert_eq!(
        after - before,
        0,
        "REVISION_REQUIRED 不需要 staff 動作，不該讓 staff 的全域 triage badge 增加"
    );
}

/// 對照組：`UNDER_REVIEW` 兩邊都要算——staff 要審，申請人也在等結果。
#[tokio::test]
#[serial]
async fn both_pending_counts_include_under_review() {
    let app = TestApp::spawn().await;
    let staff_before = AmendmentService::get_pending_count(&app.db_pool)
        .await
        .expect("get_pending_count before");

    let (user_id, _amendment) = seed_amendment_for_user(&app.db_pool, "UNDER_REVIEW").await;

    let staff_after = AmendmentService::get_pending_count(&app.db_pool)
        .await
        .expect("get_pending_count after");
    let user_count = AmendmentService::get_pending_count_for_user(&app.db_pool, user_id)
        .await
        .expect("get_pending_count_for_user");

    assert_eq!(
        staff_after - staff_before,
        1,
        "UNDER_REVIEW 仍需 staff 審查，應讓 staff 待處理數 +1"
    );
    assert_eq!(
        user_count, 1,
        "UNDER_REVIEW 申請人也在等結果，應算進申請人待處理數"
    );
}
