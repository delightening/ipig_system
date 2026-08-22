//! 回歸測試：`backfill_missing_leave_pins` 對多收件人關卡的冪等檢查不能漏人。
//!
//! 背景（Qodo review on PR #133 抓到，2026-08-14）：原本的冪等判斷是單一
//! `(leave_id, recipient_role)` EXISTS 查詢——只要同一關卡的收件人裡有任何一位
//! 已經有 pin，就整批 `continue`，其他還沒補到的人永遠補不到。這在單一收件人
//! 情境（部門主管、單一 DIRECTOR）不會觸發，但只要一關有多位合法審核人
//! （如多位 DIRECTOR，或 admin_roster fallback 命中多位管理員）就會漏人，
//! 且不會有任何錯誤訊息。
//!
//! 本測試模擬「PENDING_DIRECTOR 有兩位合法 DIRECTOR、其中一位已經有 pin（模擬
//! 先前跑過一次不完整的回填）」，驗證重跑後另一位也會補到。

mod common;

use chrono::Utc;
use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::{AuthService, HrService};

async fn seed_director(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!(
        "backfill-multi-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("backfill multi director {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert director");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'DIRECTOR'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign DIRECTOR role");
    id
}

async fn seed_applicant(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'backfill multi applicant', true, false)"#,
    )
    .bind(id)
    .bind(format!(
        "backfill-multi-applicant-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    ))
    .execute(&app.db_pool)
    .await
    .expect("insert applicant");
    id
}

#[tokio::test]
#[serial]
async fn backfill_does_not_skip_other_recipients_when_one_already_pinned() {
    let app = TestApp::spawn().await;

    let director_a = seed_director(&app, "a").await;
    let director_b = seed_director(&app, "b").await;
    let applicant = seed_applicant(&app).await;

    let leave_id = Uuid::new_v4();
    let today = Utc::now().date_naive();
    sqlx::query(
        r#"INSERT INTO leave_requests
             (id, user_id, leave_type, start_date, end_date, total_days, status, submitted_at)
           VALUES ($1, $2, 'ANNUAL'::leave_type, $3, $3, 1, 'PENDING_DIRECTOR'::leave_status, NOW())"#,
    )
    .bind(leave_id)
    .bind(applicant)
    .bind(today)
    .execute(&app.db_pool)
    .await
    .expect("insert leave request");

    // 模擬「先前跑過一次不完整的回填」：director_a 已經有 pin，director_b 沒有。
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, is_read, related_entity_type, related_entity_id,
              priority, kind, recipient_role)
           VALUES (gen_random_uuid(), $1, 'leave_approval'::notification_type,
                   '[iPig] 新請假申請 - backfill multi applicant', false,
                   'leave_request', $2, 1, 'action', 'approver')"#,
    )
    .bind(director_a)
    .bind(leave_id)
    .execute(&app.db_pool)
    .await
    .expect("insert pre-existing pin for director_a");

    HrService::backfill_missing_leave_pins(&app.db_pool, false)
        .await
        .expect("backfill should succeed");

    let director_b_pinned: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 FROM notifications
            WHERE related_entity_type = 'leave_request' AND related_entity_id = $1
              AND user_id = $2 AND recipient_role = 'approver' AND priority > 0
        )"#,
    )
    .bind(leave_id)
    .bind(director_b)
    .fetch_one(&app.db_pool)
    .await
    .expect("check director_b pin");
    assert!(
        director_b_pinned,
        "director_b 應該被補建置頂待辦——同一關卡另一位審核人已有 pin 不該讓 director_b 被跳過"
    );

    let director_a_pin_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM notifications
           WHERE related_entity_type = 'leave_request' AND related_entity_id = $1
             AND user_id = $2 AND recipient_role = 'approver'"#,
    )
    .bind(leave_id)
    .bind(director_a)
    .fetch_one(&app.db_pool)
    .await
    .expect("check director_a pin count");
    assert_eq!(
        director_a_pin_count, 1,
        "director_a 原本就有的 pin 不該被重複建立"
    );
}

/// dry-run **絕對不可寫入資料庫**。
///
/// 這是這支 CLI 的預設模式、而且是對 prod 執行的——「說好只查不寫、結果寫了」是這類
/// 工具最不能出的錯，卻原本零測試覆蓋（2026-08-14 獨立審查抓到）。
#[tokio::test]
#[serial]
async fn dry_run_reports_without_writing_any_notification() {
    let app = TestApp::spawn().await;

    let director = seed_director(&app, "dryrun").await;
    let applicant = seed_applicant(&app).await;

    let leave_id = Uuid::new_v4();
    let today = Utc::now().date_naive();
    sqlx::query(
        r#"INSERT INTO leave_requests
             (id, user_id, leave_type, start_date, end_date, total_days, status, submitted_at)
           VALUES ($1, $2, 'ANNUAL'::leave_type, $3, $3, 1, 'PENDING_DIRECTOR'::leave_status, NOW())"#,
    )
    .bind(leave_id)
    .bind(applicant)
    .bind(today)
    .execute(&app.db_pool)
    .await
    .expect("insert leave request");

    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE related_entity_type = 'leave_request'",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("count before");

    let report = HrService::backfill_missing_leave_pins(&app.db_pool, true)
        .await
        .expect("dry-run should succeed");

    // 有回報「將建立」，證明它確實走完了計算路徑（而不是因為沒事做才沒寫入）。
    assert!(
        report
            .created
            .iter()
            .any(|c| c.leave_id == leave_id && c.recipient_names.iter().any(|n| !n.is_empty())),
        "dry-run 應回報這張單將建立待辦，實得：{:?}",
        report.created
    );

    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE related_entity_type = 'leave_request'",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("count after");
    assert_eq!(
        before, after,
        "dry-run 不得寫入任何 notifications 列（before={before}, after={after}）"
    );

    let director_pinned: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 FROM notifications
            WHERE related_entity_type = 'leave_request' AND related_entity_id = $1
              AND user_id = $2
        )"#,
    )
    .bind(leave_id)
    .bind(director)
    .fetch_one(&app.db_pool)
    .await
    .expect("check director pin");
    assert!(!director_pinned, "dry-run 不得替 director 建立任何通知");
}
