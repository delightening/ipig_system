//! 代理人確認 → 置頂待辦（驚嘆號入口）的生命週期。
//!
//! 使用者 2026-08-07 原始回報：「代理人的確認、請假批准也跟巡場報告一樣放在
//! "待處理"裡」——當時分析發現代理人確認走的是一般通知（`kind='info'`），
//! 從未進入「待處理」。本檔鎖住補齊後的行為：送審建立置頂待辦、代理人
//! 確認/退回/申請人取消三條路徑都要能解除它。
//!
//! 只做代理人確認這一段（不含 L1/負責人核准階段）——後者的通知路由目前是
//! 角色廣播（`admin` + `DIRECTOR`），不分審核關卡，會讓還輪不到的角色也看到
//! 一顆點下去必定 403 的「待處理」，需要先修路由精度才能安全接上，
//! 屬另案（見 docs/design/features/notification-vs-action-required-2026-08-07.md）。

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::ROLE_EXPERIMENT_STAFF;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateLeaveRequest, LeaveStatus};
use erp_backend::services::{AuditService, HrService};

#[path = "common/test_db.rs"]
mod test_db;

async fn pool() -> PgPool {
    let pool = test_db::connect_disposable(5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    AuditService::init_hmac_key(Some(
        "test-hmac-key-do-not-use-in-prod-base64-padding".to_string(),
    ));
    pool
}

async fn mk_user(pool: &PgPool, email: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active) \
         VALUES ($1, $2, 'x', $3, true)",
    )
    .bind(id)
    .bind(email)
    .bind(email)
    .execute(pool)
    .await
    .expect("insert user");
    id
}

async fn assign_role(pool: &PgPool, user_id: Uuid, role_code: &str) {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("assign role");
}

fn cur(id: Uuid, roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: format!("{id}@t.com"),
        roles: roles.iter().map(|r| r.to_string()).collect(),
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    }
}

fn draft_payload(proxy: Option<Uuid>, leave_type: &str) -> CreateLeaveRequest {
    CreateLeaveRequest {
        proxy_user_id: proxy,
        leave_type: leave_type.to_string(),
        start_date: NaiveDate::from_ymd_opt(2026, 7, 3).expect("valid date"),
        end_date: NaiveDate::from_ymd_opt(2026, 7, 3).expect("valid date"),
        start_time: None,
        end_time: None,
        total_days: 0.5,
        total_hours: Some(4.0),
        reason: Some("測試".to_string()),
        supporting_documents: None,
        is_urgent: None,
        is_retroactive: None,
    }
}

/// 待辦列的 (kind, priority)。`None` 表示該實體上沒有任何通知列
/// （用於驗證「不該建立」的情境，而非誤讀成一列已解除的待辦）。
async fn pin_state(pool: &PgPool, leave_id: Uuid, proxy_id: Uuid) -> Option<(String, i16)> {
    sqlx::query_as::<_, (String, i16)>(
        "SELECT kind, priority FROM notifications \
         WHERE related_entity_type = 'leave_request' AND related_entity_id = $1 AND user_id = $2",
    )
    .bind(leave_id)
    .bind(proxy_id)
    .fetch_optional(pool)
    .await
    .expect("query notification")
}

/// 送審後代理人立刻拿到一則置頂待辦（kind='action', priority>0）。
#[tokio::test]
async fn submit_creates_pinned_todo_for_proxy() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(Some(proxy), "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");

    let (kind, priority) = pin_state(&pool, created.id, proxy)
        .await
        .expect("proxy 應收到一則通知");
    assert_eq!(kind, "action", "代理人確認是待辦，不是單純告知");
    assert!(priority > 0, "送審後應立即置頂，而非等下一次輪詢才出現");
}

/// 代理人確認後，待辦降級（離開驚嘆號清單），但仍以 kind='action' 留在鈴鐺歷史裡。
#[tokio::test]
async fn proxy_confirm_resolves_the_todo() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(Some(proxy), "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");

    let (kind, priority) = pin_state(&pool, created.id, proxy)
        .await
        .expect("已解除的待辦仍應留著（鈴鐺歷史），不是被刪除");
    assert_eq!(kind, "action");
    assert_eq!(priority, 0, "確認後應離開待處理清單");
}

/// 代理人退回同樣算「已做出動作」，待辦要解除——不是只有確認才算數。
#[tokio::test]
async fn proxy_reject_resolves_the_todo() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(Some(proxy), "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    HrService::proxy_reject_leave(&pool, &proxy_actor, created.id, Some("同天也請假"))
        .await
        .expect("proxy_reject");

    let (_, priority) = pin_state(&pool, created.id, proxy)
        .await
        .expect("退回後待辦仍應留著（鈴鐺歷史）");
    assert_eq!(priority, 0, "退回後應離開待處理清單");
}

/// 申請人在代理人確認前就取消請假：代理人不該永遠卡著一則對著空事項的待辦
/// （待辦不可手動已讀，若這裡不解除，代理人將無法自救）。
#[tokio::test]
async fn cancel_during_pending_proxy_resolves_the_todo() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(Some(proxy), "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");
    assert_eq!(
        pin_state(&pool, created.id, proxy).await.map(|(_, p)| p),
        Some(1),
        "取消前應先確認待辦真的存在，否則下面的解除斷言是空歡喜"
    );

    HrService::cancel_leave(&pool, &actor, created.id, Some("行程取消"))
        .await
        .expect("cancel_leave");

    let (_, priority) = pin_state(&pool, created.id, proxy)
        .await
        .expect("取消後待辦仍應留著（鈴鐺歷史）");
    assert_eq!(priority, 0, "取消後代理人不再需要確認，待辦應解除");
}

/// 負責人本人請假走報備制（送出即核准，見 submit_leave 內 director_self_report）：
/// 沒有代理人需要確認，不該憑空建立一則待辦。
#[tokio::test]
async fn director_self_report_creates_no_todo() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let director = mk_user(&pool, &format!("dir-{s}@t.com")).await;
    assign_role(&pool, director, "DIRECTOR").await;

    let actor = ActorContext::User(cur(director, &["DIRECTOR"]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(None, "PERSONAL"))
        .await
        .expect("create_leave");
    let submitted = HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave（負責人報備制送出即核准）");
    assert_eq!(
        submitted.status,
        LeaveStatus::Approved.as_str(),
        "報備制應直接核准，不進 PENDING_PROXY"
    );

    let any_notification: Option<(String, i16)> = sqlx::query_as(
        "SELECT kind, priority FROM notifications \
         WHERE related_entity_type = 'leave_request' AND related_entity_id = $1",
    )
    .bind(created.id)
    .fetch_optional(&pool)
    .await
    .expect("query notification");
    assert_eq!(
        any_notification, None,
        "報備制沒有代理人可確認，不該有任何通知列"
    );
}
