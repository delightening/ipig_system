// 回歸測試：請假審核流程（代理確認 → 單位主管 → 負責人）。
//
// 背景：
//   - PR #843 補上 users.department_id，修復送審 500。
//   - 本輪重設計審核鏈：送審後先「代理人確認」，再「單位主管」，最後「負責人（DIRECTOR）」簽核；
//     某關無合法審核人時自動跳關；代理人退回則回草稿。
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::{ROLE_DIRECTOR, ROLE_EXPERIMENT_STAFF};
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

/// 建立部門並指派主管，回傳部門 id。
async fn mk_department(pool: &PgPool, manager_id: Uuid, suffix: &Uuid) -> Uuid {
    let dept = Uuid::new_v4();
    sqlx::query("INSERT INTO departments (id, code, name, manager_id) VALUES ($1, $2, $3, $4)")
        .bind(dept)
        .bind(format!("D{}", &suffix.to_string()[..8]))
        .bind("測試部門")
        .bind(manager_id)
        .execute(pool)
        .await
        .expect("insert dept");
    dept
}

async fn assign_department(pool: &PgPool, user_id: Uuid, dept: Uuid) {
    sqlx::query("UPDATE users SET department_id = $1 WHERE id = $2")
        .bind(dept)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("assign dept");
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

fn draft_payload(proxy: Uuid, leave_type: &str) -> CreateLeaveRequest {
    CreateLeaveRequest {
        proxy_user_id: Some(proxy),
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

/// 送審後第一關為「代理確認」：狀態 PENDING_PROXY，current_approver = 代理人，不得 500。
#[tokio::test]
async fn submit_routes_to_pending_proxy() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    let submitted = HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave should not 500");
    assert_eq!(submitted.status, LeaveStatus::PendingProxy.as_str());
    assert_eq!(submitted.current_approver_id, Some(proxy));
}

/// 代理確認：申請人有部門主管 → 進 PENDING_L1（單位主管關）。
#[tokio::test]
async fn proxy_confirm_with_manager_routes_to_l1() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let mgr = mk_user(&pool, &format!("mgr-{s}@t.com")).await;
    let dept = mk_department(&pool, mgr, &s).await;
    assign_department(&pool, applicant, dept).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let confirmed = HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(confirmed.status, LeaveStatus::PendingL1.as_str());
}

/// 代理確認：申請人未指派部門 → 自動跳關，直接進 PENDING_DIRECTOR（負責人關）。
#[tokio::test]
async fn proxy_confirm_without_department_skips_to_director() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let confirmed = HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(confirmed.status, LeaveStatus::PendingDirector.as_str());
}

/// 完整鏈：送審 → 代理確認 → 單位主管核准 → 負責人核准 → 已核准。
#[tokio::test]
async fn full_chain_reaches_approved() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let mgr = mk_user(&pool, &format!("mgr-{s}@t.com")).await;
    let director = mk_user(&pool, &format!("dir-{s}@t.com")).await;
    let dept = mk_department(&pool, mgr, &s).await;
    assign_department(&pool, applicant, dept).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let after_proxy = HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(after_proxy.status, LeaveStatus::PendingL1.as_str());

    let mgr_actor = ActorContext::User(cur(mgr, &[ROLE_EXPERIMENT_STAFF]));
    let after_mgr = HrService::approve_leave(&pool, &mgr_actor, created.id, None)
        .await
        .expect("manager approve");
    assert_eq!(after_mgr.status, LeaveStatus::PendingDirector.as_str());

    let dir_actor = ActorContext::User(cur(director, &[ROLE_DIRECTOR]));
    let after_dir = HrService::approve_leave(&pool, &dir_actor, created.id, None)
        .await
        .expect("director approve");
    assert_eq!(after_dir.status, LeaveStatus::Approved.as_str());
}

/// 代理人退回 → 回草稿，且保留原 proxy_user_id 供申請人重新指定。
#[tokio::test]
async fn proxy_reject_returns_to_draft() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let rejected =
        HrService::proxy_reject_leave(&pool, &proxy_actor, created.id, Some("我那天也請假"))
            .await
            .expect("proxy_reject");
    assert_eq!(rejected.status, LeaveStatus::Draft.as_str());
    assert_eq!(rejected.proxy_user_id, Some(proxy));
    assert!(rejected.submitted_at.is_none());
}

/// SoD 回歸：單位主管兼任職務代理人時，確認代理後仍能審核 PENDING_L1（不因 SoD 卡關）。
#[tokio::test]
async fn manager_who_is_proxy_can_still_approve_l1() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    // 代理人 = 部門主管（同一人）
    let mgr = mk_user(&pool, &format!("mgr-{s}@t.com")).await;
    let dept = mk_department(&pool, mgr, &s).await;
    assign_department(&pool, applicant, dept).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(mgr, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let mgr_actor = ActorContext::User(cur(mgr, &[ROLE_EXPERIMENT_STAFF]));
    // 主管以「代理人」身分確認
    let confirmed = HrService::proxy_confirm_leave(&pool, &mgr_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(confirmed.status, LeaveStatus::PendingL1.as_str());
    // 同一主管改以「單位主管」身分審核 —— SoD 不應擋（代理確認不算前關核准）
    let approved = HrService::approve_leave(&pool, &mgr_actor, created.id, None)
        .await
        .expect("manager approve should not be blocked by SoD");
    assert_eq!(approved.status, LeaveStatus::PendingDirector.as_str());
}

/// 非指定代理人不得確認他人的待代理請假。
#[tokio::test]
async fn non_proxy_cannot_confirm() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let stranger = mk_user(&pool, &format!("x-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy, "PERSONAL"))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit");

    let stranger_actor = ActorContext::User(cur(stranger, &[ROLE_EXPERIMENT_STAFF]));
    let result = HrService::proxy_confirm_leave(&pool, &stranger_actor, created.id).await;
    assert!(result.is_err(), "非代理人不得確認");
}
