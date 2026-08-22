//! 回歸測試：置頂待辦（`priority > 0`）的孤兒對帳。
//!
//! 2026-08-07 事故：巡場報告的 `retract_to_draft` / `delete`（軟刪）兩條路徑沒有呼叫
//! `resolve_pinned_notifications`，導致置頂通知綁在一份已軟刪的報告上永久卡死——
//! 而待辦依設計不可手動已讀，使用者完全無法自救。
//!
//! 本測試鎖住對帳作業的兩個對稱契約：
//! 1. 關聯實體已刪 / 已完成 / 不存在 → **必須**降級
//! 2. 關聯實體仍在途（等使用者動作）→ **絕不可**降級
//!
//! 第 2 點比第 1 點重要：誤清真正待處理的事項，比留下一筆多餘待辦嚴重得多。

use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::NotificationService;

#[path = "common/test_db.rs"]
mod test_db;

/// 本檔的測試會跑 non-dry-run 對帳（真的 `UPDATE notifications`）並跑 migration，
/// 所以「連錯 DB」的代價是污染正式資料，不是測試失敗而已——隔離護欄改用
/// `common/test_db.rs`（判斷依據是目標資料庫本身的身分，不是它的名字或哪個
/// 環境變數帶的值；見該檔文件註解）。
async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'pinned reconcile test', true, false)"#,
    )
    .bind(id)
    .bind(format!("pinned-reconcile-{}@test.local", &id.to_string()[..8]))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

/// 建一份巡場報告。`deleted` 為 true 時同時軟刪。
async fn seed_patrol_report(pool: &PgPool, vet_id: Uuid, status: &str, deleted: bool) -> Uuid {
    seed_patrol_report_with_follower(pool, vet_id, status, deleted, None).await
}

/// 同上，但可指定 `follow_up_user_id`（撤回情境需要它明確為 NULL）。
async fn seed_patrol_report_with_follower(
    pool: &PgPool,
    vet_id: Uuid,
    status: &str,
    deleted: bool,
    follow_up_user_id: Option<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO vet_patrol_reports
             (id, patrol_date, status, created_by, follow_up_user_id, deleted_at)
           VALUES ($1, CURRENT_DATE, $2, $3, $4,
                   CASE WHEN $5 THEN NOW() ELSE NULL END)"#,
    )
    .bind(id)
    .bind(status)
    .bind(vet_id)
    .bind(follow_up_user_id)
    .bind(deleted)
    .execute(pool)
    .await
    .expect("seed patrol report");
    id
}

/// 建一則置頂待辦（`priority = 1`）。
async fn seed_pinned_notification(pool: &PgPool, user_id: Uuid, report_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, related_entity_type, related_entity_id, priority)
           VALUES ($1, $2, 'vet_recommendation'::notification_type, '需您填寫追蹤改善',
                   'vet_patrol_reports', $3, 1)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(report_id)
    .execute(pool)
    .await
    .expect("seed pinned notification");
    id
}

async fn priority_of(pool: &PgPool, notification_id: Uuid) -> i16 {
    sqlx::query_scalar("SELECT priority FROM notifications WHERE id = $1")
        .bind(notification_id)
        .fetch_one(pool)
        .await
        .expect("read priority")
}

/// 建一張請假單。`status` 直接指定，供測試擺出各種關卡狀態。
///
/// `proxy_user_id` 用 `Option`：代理人待辦的情境必須有代理人（傳 `Some`），
/// 審核人待辦的情境則刻意留 `None`（負責人自行報備、或舊資料沒有代理人）——
/// 對帳查詢用「收件人是不是 proxy_user_id」區分兩種待辦，所以這個欄位是不是
/// NULL 直接決定測試打到哪一支 UNION 分支。
///
/// 欄位真值來源：`migrations/008_hr_system.sql:131`（NOT NULL 為 user_id /
/// leave_type / start_date / end_date / total_days）、`001_enums.sql:55`
/// （leave_type 的 'ANNUAL'）、`models/hr.rs:286`（leave_status 的 'PENDING_PROXY'）。
async fn seed_leave_request(
    pool: &PgPool,
    user_id: Uuid,
    proxy_user_id: Option<Uuid>,
    status: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO leave_requests
             (id, user_id, proxy_user_id, leave_type, start_date, end_date,
              total_days, reason, status)
           VALUES ($1, $2, $3, 'ANNUAL'::leave_type, CURRENT_DATE, CURRENT_DATE,
                   1, 'pinned reconcile test', $4::leave_status)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(proxy_user_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("seed leave request");
    id
}

/// 建一則**代理人**的「待確認」置頂待辦，對應 `submit_leave` 實際產生的那一則。
///
/// 與 [`seed_pinned_leave_approval_notification`] 的差別只在收件人是誰——兩者的
/// `related_entity_type` 都是 `leave_request`，對帳靠「收件人是不是該假單的
/// `proxy_user_id`」區分，所以呼叫端傳進來的 `proxy_user_id` 必須與假單上的一致。
///
/// `kind='action'` 與 `create_pinned_notification_tx` 一致（`crud.rs:365`
/// 依 priority 決定 kind）——對帳查詢雖只看 `priority > 0`，但 fixture 若與
/// 正式流程產出的形狀不同，保護的就是不存在的資料。
async fn seed_pinned_leave_proxy_notification(
    pool: &PgPool,
    proxy_user_id: Uuid,
    leave_request_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, related_entity_type, related_entity_id,
              priority, kind, recipient_role)
           VALUES ($1, $2, 'leave_approval'::notification_type,
                   '[iPig] 測試申請人 指定您為職務代理人，待您確認',
                   'leave_request', $3, 1, 'action', 'proxy')"#,
    )
    .bind(id)
    .bind(proxy_user_id)
    .bind(leave_request_id)
    .execute(pool)
    .await
    .expect("seed pinned leave proxy notification");
    id
}

/// 建一則**審核人**（L1／終審）的置頂待辦。
///
/// 收件人刻意不是假單的 `proxy_user_id`，才會落到對帳的審核人分支。
async fn seed_pinned_leave_approval_notification(
    pool: &PgPool,
    approver_id: Uuid,
    leave_request_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, related_entity_type, related_entity_id,
              priority, kind, recipient_role)
           VALUES ($1, $2, 'leave_approval'::notification_type, '新請假申請待審核',
                   'leave_request', $3, 1, 'action', 'approver')"#,
    )
    .bind(id)
    .bind(approver_id)
    .bind(leave_request_id)
    .execute(pool)
    .await
    .expect("seed pinned leave approval notification");
    id
}

// 本檔全部測試都對「整張 notifications 表」跑對帳＝共享狀態，必須序列化：
// 併發下另一支測試的非 dry-run 對帳會把本測試的列一起降級，dry-run 那支尤其會偽紅。
//
// ⚠️ `#[serial]` 只在**同一測試 binary 內**生效。目前這是安全的，因為
// `reconcile_pinned_notifications` 只有本檔呼叫 —— 其他 test binary 不會動到
// 別人的置頂列。**若日後有第二個 test binary 呼叫對帳（非 dry-run），這個前提就破了**，
// 屆時需改用跨行程機制（各自獨立的 test database，或 advisory lock），
// 而不是再加 `#[serial]`（那擋不住跨 binary 的併發）。
#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_report_was_soft_deleted() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    // fixture 必須**只**滿足「軟刪」這一個 disjunct，否則測不出軟刪規則：
    // 若用 draft + follow_up_user_id=NULL，會同時命中「已撤回」那條，
    // 把 SQL 裡的 `deleted_at IS NOT NULL` 整條拿掉、測試仍然會綠。
    // 故用「在途狀態 + 有指派追蹤者 + 已軟刪」—— 這也正是 2026-08-07 事故的真實狀態。
    let report = seed_patrol_report_with_follower(
        &pool,
        vet,
        "awaiting_acknowledgement",
        true,
        Some(follower),
    )
    .await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        report_out.resolved.iter().any(|r| r.id == notif),
        "報告已軟刪，其置頂待辦應被對帳作業降級"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        0,
        "降級後 priority 應為 0，否則仍會出現在待處理清單"
    );
}

#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_report_is_completed() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    // completed 報告必定帶指派的追蹤者（complete_followup 只能由他本人執行）。
    // fixture 要符合正式流程產得出來的狀態，否則保護的是不可能存在的資料。
    let report =
        seed_patrol_report_with_follower(&pool, vet, "completed", false, Some(follower)).await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    svc.reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert_eq!(
        priority_of(&pool, notif).await,
        0,
        "報告已完成，追蹤者已無事可做，置頂待辦應降級"
    );
}

/// 撤回留下 `status='draft'` + `follow_up_user_id IS NULL`。
///
/// 置頂待辦只在 `submit_for_followup` 建立（該處必定同時設 `awaiting_acknowledgement`
/// 與 `follow_up_user_id`），所以這個組合唯一的來源就是撤回。撤回正是 2026-08-07
/// 事故的觸發路徑 —— 若 service 內的解除將來回歸，安全網必須接得住。
#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_report_was_retracted() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    let report = seed_patrol_report_with_follower(&pool, vet, "draft", false, None).await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    let hit = report_out.resolved.iter().find(|r| r.id == notif);
    assert!(
        hit.is_some(),
        "報告已撤回（draft 且無指派追蹤者），其置頂待辦應被降級"
    );
    assert!(
        hit.expect("hit").reason.contains("撤回"),
        "降級理由應明確指出是撤回，維運者才知道哪條路徑漏接"
    );
    assert_eq!(priority_of(&pool, notif).await, 0);
}

/// 對照組：`draft` 但**仍有**指派追蹤者 —— 這不是撤回造成的狀態，不得誤降。
/// 沒有這一例的話，上一個測試可以靠「只要是 draft 就降級」這種過寬的條件通過。
#[tokio::test]
#[serial]
async fn reconcile_leaves_draft_with_assigned_follower_untouched() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    let report = seed_patrol_report_with_follower(&pool, vet, "draft", false, Some(follower)).await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == notif),
        "draft 但仍有指派追蹤者 ≠ 撤回，不得降級"
    );
    assert_eq!(priority_of(&pool, notif).await, 1);
}

/// `related_entity_id IS NULL` ＝ **無從判斷**，不是「實體不存在」。
///
/// 初版 SQL 沒有這個條件，於是 `NOT EXISTS (... WHERE d.id = NULL)` 恆為真、
/// `LEFT JOIN` 的 `r.id IS NULL` 也恆成立 → 這類列被**無條件降級**，
/// 還印出「關聯的實體已不存在」這個假理由。
/// 2026-08-07 prod 實查：7 筆置頂中有 4 筆正是這種（舊聚合式採購提醒，本來就不帶 entity id）。
#[tokio::test]
#[serial]
async fn reconcile_leaves_null_entity_id_untouched() {
    let pool = setup_pool().await;
    let follower = seed_user(&pool).await;

    let notif = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, related_entity_type, related_entity_id, priority)
           VALUES ($1, $2, 'document_approval'::notification_type,
                   '[iPig] 採購單未入庫提醒：1 筆採購單待入庫', 'document', NULL, 1)"#,
    )
    .bind(notif)
    .bind(follower)
    .execute(&pool)
    .await
    .expect("seed null-entity pinned notification");

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == notif),
        "related_entity_id 為 NULL＝無從判斷，不得降級"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        1,
        "NULL entity 的置頂待辦必須保持原狀"
    );
    assert!(
        report_out.null_entity_id >= 1,
        "NULL entity 的列必須被單獨列出讓維運者看見，實得 {}",
        report_out.null_entity_id
    );
}

#[tokio::test]
#[serial]
async fn reconcile_leaves_in_flight_todo_untouched() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    // 仍在途：指派給追蹤者、等他確認收到。
    // 必須帶 follow_up_user_id —— submit_for_followup 一定同時設 status 與追蹤者，
    // 用 None 會讓這個 fixture 保護一個正式流程根本產不出來的狀態。
    let report = seed_patrol_report_with_follower(
        &pool,
        vet,
        "awaiting_acknowledgement",
        false,
        Some(follower),
    )
    .await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == notif),
        "在途待辦不得被對帳作業降級——誤清真正待處理的事項比留下多餘待辦嚴重得多"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        1,
        "在途待辦的 priority 必須維持 1"
    );
}

/// Qodo PR #67 review：新增 leave_request 為對帳作業認得的 entity_type
/// ——修前對帳完全不認得這個類型，孤兒的請假置頂待辦會永遠卡住（見
/// `count_unknown_entity_types` 的 unknown 桶，本來會把 leave_request 算進去）。
#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_leave_was_approved() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let approver = seed_user(&pool).await;

    // 已離開審核關卡（終態）：待辦已無事可做。
    let leave = seed_leave_request(&pool, applicant, None, "APPROVED").await;
    let notif = seed_pinned_leave_approval_notification(&pool, approver, leave).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        report_out.resolved.iter().any(|r| r.id == notif),
        "假單已核准（終態），其置頂待辦應被對帳作業降級"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        0,
        "降級後 priority 應為 0，否則仍會出現在待處理清單"
    );
}

#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_leave_no_longer_exists() {
    let pool = setup_pool().await;
    let approver = seed_user(&pool).await;
    // 不建立對應的 leave_requests 列，模擬 row 不存在（硬刪或從未存在）。
    let phantom_leave_id = Uuid::new_v4();
    let notif = seed_pinned_leave_approval_notification(&pool, approver, phantom_leave_id).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        report_out.resolved.iter().any(|r| r.id == notif),
        "關聯的請假申請已不存在，其置頂待辦應被對帳作業降級"
    );
    assert_eq!(priority_of(&pool, notif).await, 0);
}

#[tokio::test]
#[serial]
async fn reconcile_leaves_leave_in_pending_l1_untouched() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let manager = seed_user(&pool).await;

    // 仍在途：等單位主管審核。
    let leave = seed_leave_request(&pool, applicant, None, "PENDING_L1").await;
    let notif = seed_pinned_leave_approval_notification(&pool, manager, leave).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == notif),
        "在途待辦不得被對帳作業降級——誤清真正待處理的事項比留下多餘待辦嚴重得多"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        1,
        "在途待辦的 priority 必須維持 1"
    );
}

#[tokio::test]
#[serial]
async fn reconcile_dry_run_reports_without_writing() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool).await;
    let follower = seed_user(&pool).await;

    let report = seed_patrol_report(&pool, vet, "draft", true).await;
    let notif = seed_pinned_notification(&pool, follower, report).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(true)
        .await
        .expect("reconcile dry-run");

    assert!(
        report_out.resolved.iter().any(|r| r.id == notif),
        "dry-run 仍應回報將被降級的列，供上 prod 前核對筆數"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        1,
        "dry-run 不得寫入——這是上 prod 前唯一的核對機會"
    );
}

// ── 請假：代理人確認待辦（PR #66 新增的待辦類型）────────────────────────
//
// 這一類的置頂待辦只在 `submit_leave` 的非 director_self_report 分支建立，
// 收件人是 proxy_user_id，建立當下 status 必為 'PENDING_PROXY'。三條解除路徑
// （proxy_confirm / proxy_reject / cancel_leave）都會讓 status 離開該值，
// 故「仍需代理人動作」等價於「仍在 PENDING_PROXY」。

/// 代理人已確認（假單進入 PENDING_L1）→ 代理人已無事可做，必須降級。
#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_leave_left_pending_proxy() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let proxy = seed_user(&pool).await;

    let leave = seed_leave_request(&pool, applicant, Some(proxy), "PENDING_L1").await;
    let notif = seed_pinned_leave_proxy_notification(&pool, proxy, leave).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    let hit = report_out.resolved.iter().find(|r| r.id == notif);
    assert!(
        hit.is_some(),
        "假單已離開 PENDING_PROXY，代理人的置頂待辦應被降級"
    );
    assert!(
        hit.expect("hit").reason.contains("待代理確認"),
        "降級理由應指出是離開待代理確認狀態，維運者才知道是哪條路徑"
    );
    assert_eq!(priority_of(&pool, notif).await, 0);
}

/// 假單被硬刪（row 不存在）→ 代理人不可能再確認，必須降級。
#[tokio::test]
#[serial]
async fn reconcile_downgrades_pin_whose_leave_row_is_gone() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let proxy = seed_user(&pool).await;

    let leave = seed_leave_request(&pool, applicant, Some(proxy), "PENDING_PROXY").await;
    let notif = seed_pinned_leave_proxy_notification(&pool, proxy, leave).await;
    sqlx::query("DELETE FROM leave_requests WHERE id = $1")
        .bind(leave)
        .execute(&pool)
        .await
        .expect("hard delete leave request");

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    let hit = report_out.resolved.iter().find(|r| r.id == notif);
    assert!(hit.is_some(), "關聯假單已不存在，置頂待辦應被降級");
    assert!(
        hit.expect("hit").reason.contains("不存在"),
        "降級理由應指出實體已不存在"
    );
    assert_eq!(priority_of(&pool, notif).await, 0);
}

/// 對照組：假單仍在 PENDING_PROXY ＝ 代理人真的還沒動作，**絕不可**降級。
///
/// 這一例比上面兩例重要：少了它，上面兩個測試可以靠「只要是 leave_request
/// 就降級」這種過寬的條件通過，而誤清真正待處理的事項比留下多餘待辦嚴重得多。
#[tokio::test]
#[serial]
async fn reconcile_leaves_in_flight_leave_proxy_todo_untouched() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let proxy = seed_user(&pool).await;

    let leave = seed_leave_request(&pool, applicant, Some(proxy), "PENDING_PROXY").await;
    let notif = seed_pinned_leave_proxy_notification(&pool, proxy, leave).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == notif),
        "假單仍在 PENDING_PROXY＝代理人尚未確認，不得降級"
    );
    assert_eq!(
        priority_of(&pool, notif).await,
        1,
        "在途的代理人待辦 priority 必須維持 1"
    );
}

/// R92-1 回歸測試：代理人剛好也是下一關核准人（例如代理人＝申請人的單位主管）時，
/// confirm 後新建的核准 pin 不可被規則②誤清。
///
/// 修復前：規則②只憑「收件人是不是 proxy_user_id」判斷，這則核准 pin 的收件人恰好
/// 也是 proxy_user_id，狀態又已離開 PENDING_PROXY（現在是 PENDING_L1），會被誤判成
/// 該清除的舊代理待辦而降級——即使核准人真正還沒動作。148 改用 `recipient_role`
/// （建立時寫死的用途標記）後，這則 pin 的 role='approver' 不滿足規則②的
/// `recipient_role = 'proxy'`，改由規則③依狀態判斷（PENDING_L1 → 不可降），維持不動。
#[tokio::test]
#[serial]
async fn reconcile_protects_approval_pin_when_proxy_is_also_current_approver() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool).await;
    let proxy_and_manager = seed_user(&pool).await;

    // 假單已離開 PENDING_PROXY、進到 PENDING_L1——對應代理人已確認、
    // 且該代理人剛好也是申請人的單位主管，因此下一關核准人與代理人是同一人。
    let leave = seed_leave_request(&pool, applicant, Some(proxy_and_manager), "PENDING_L1").await;
    let approval_notif =
        seed_pinned_leave_approval_notification(&pool, proxy_and_manager, leave).await;

    let svc = NotificationService::new(pool.clone());
    let report_out = svc
        .reconcile_pinned_notifications(false)
        .await
        .expect("reconcile");

    assert!(
        !report_out.resolved.iter().any(|r| r.id == approval_notif),
        "代理人兼核准人時，confirm 後新建的核准 pin 不可被誤判成舊代理待辦而降級"
    );
    assert_eq!(
        priority_of(&pool, approval_notif).await,
        1,
        "在途的核准待辦 priority 必須維持 1，即使收件人剛好等於 proxy_user_id"
    );
}
