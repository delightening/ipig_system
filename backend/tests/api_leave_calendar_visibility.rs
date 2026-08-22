//! 請假行事曆（`GET /api/v1/hr/leaves/calendar`）的可見範圍。
//!
//! 這是本功能唯一的資安面：行事曆會揭露「誰哪幾天不在、由誰代理」，
//! 範圍算錯就是把全場人事動態洩漏給不該看的人。規則（見
//! `services/hr/leave_calendar.rs`）：
//!
//! - 管理員 / `hr.leave.view_all` / **任一部門的主管** → 全場
//! - 其餘 → 自己所屬部門 + 上層 + 下層部門
//! - 不受部門限制的兩個例外：自己請的假、自己被指派為代理人的假
//!
//! 特別要鎖住的是「部門主管」那條：系統裡**沒有** DEPT_MANAGER 這個角色，
//! 主管身分是 `departments.manager_id` 的資料關係。若哪天有人把判斷改回
//! 用權限碼，所有主管會瞬間退化成只看得到自己部門，而且不會有任何錯誤訊息。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

const PW: &str = "iPig$ecure1";

async fn seed_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    let email = format!("leavecal-{label}-{unique}@test.local");
    let hash = AuthService::hash_password(PW).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("行事曆{label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    (id, email)
}

async fn seed_department(app: &TestApp, label: &str, parent: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query("INSERT INTO departments (id, code, name, parent_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(format!("LC_{label}_{unique}"))
        .bind(format!("行事曆測試部門{label}"))
        .bind(parent)
        .execute(&app.db_pool)
        .await
        .expect("insert department");
    id
}

async fn set_department(app: &TestApp, user_id: Uuid, dept_id: Uuid) {
    sqlx::query("UPDATE users SET department_id = $2 WHERE id = $1")
        .bind(user_id)
        .bind(dept_id)
        .execute(&app.db_pool)
        .await
        .expect("set department");
}

async fn set_manager(app: &TestApp, dept_id: Uuid, user_id: Uuid) {
    sqlx::query("UPDATE departments SET manager_id = $2 WHERE id = $1")
        .bind(dept_id)
        .bind(user_id)
        .execute(&app.db_pool)
        .await
        .expect("set manager");
}

/// 建一張假單。`status` 直接指定，繞過送審流程——本測試要驗的是讀取範圍，
/// 不是狀態機。
async fn seed_leave(
    app: &TestApp,
    user_id: Uuid,
    status: &str,
    proxy: Option<Uuid>,
    day_offset: i64,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO leave_requests
             (id, user_id, proxy_user_id, leave_type, start_date, end_date,
              total_days, reason, status)
           VALUES ($1, $2, $3, 'ANNUAL'::leave_type,
                   CURRENT_DATE + $5::int, CURRENT_DATE + $5::int,
                   1, '測試', $4::leave_status)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(proxy)
    .bind(status)
    .bind(day_offset as i32)
    .execute(&app.db_pool)
    .await
    .expect("insert leave");
    id
}

/// 撈行事曆，回傳 (可見的假單 id 集合, scope)。
async fn fetch_calendar(app: &TestApp, token: &str) -> (Vec<String>, String) {
    let res = app
        .auth_get(
            "/api/v1/hr/leaves/calendar?start_date=2000-01-01&end_date=2000-12-31",
            token,
        )
        .await;
    // 先用一個確定不含資料的區間確認端點可用，再打真正的區間。
    assert_eq!(res.status().as_u16(), 200, "行事曆端點應回 200");

    let res = app
        .auth_get(
            &format!(
                "/api/v1/hr/leaves/calendar?start_date={}&end_date={}",
                chrono::Utc::now().date_naive() - chrono::Duration::days(30),
                chrono::Utc::now().date_naive() + chrono::Duration::days(60),
            ),
            token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "行事曆端點應回 200");
    let body: serde_json::Value = TestApp::json(res).await;
    let ids = body["entries"]
        .as_array()
        .expect("entries 應為陣列")
        .iter()
        .map(|e| e["id"].as_str().unwrap_or_default().to_string())
        .collect();
    let scope = body["scope"].as_str().unwrap_or_default().to_string();
    (ids, scope)
}

/// 一般員工只看得到自己部門 + 上下層，看不到不相干的平行部門。
#[tokio::test]
#[serial]
async fn plain_staff_sees_own_and_adjacent_departments_only() {
    let app = TestApp::spawn().await;

    let parent = seed_department(&app, "parent", None).await;
    let child = seed_department(&app, "child", Some(parent)).await;
    let sibling = seed_department(&app, "sibling", None).await;

    let (viewer, viewer_email) = seed_user(&app, "viewer", "EXPERIMENT_STAFF").await;
    set_department(&app, viewer, parent).await;

    // 同部門「他人」。⚠️ 不能只用 viewer 自己的假單來驗部門規則——
    // 那筆會被 `l.user_id = $1`（本人一律可見）這條旁路放行，
    // 就算部門規則整個壞掉斷言也照樣會過。
    let (peer, _) = seed_user(&app, "peer", "EXPERIMENT_STAFF").await;
    set_department(&app, peer, parent).await;

    let (childy, _) = seed_user(&app, "childy", "EXPERIMENT_STAFF").await;
    set_department(&app, childy, child).await;

    let (outsider, _) = seed_user(&app, "outsider", "EXPERIMENT_STAFF").await;
    set_department(&app, outsider, sibling).await;

    let own_leave = seed_leave(&app, viewer, "APPROVED", None, 1).await;
    let peer_leave = seed_leave(&app, peer, "APPROVED", None, 2).await;
    let child_leave = seed_leave(&app, childy, "APPROVED", None, 3).await;
    let outsider_leave = seed_leave(&app, outsider, "APPROVED", None, 4).await;

    let token = app.login(&viewer_email, PW).await.expect("login 應成功");
    let (ids, scope) = fetch_calendar(&app, &token).await;

    assert_eq!(scope, "department", "一般員工的 scope 應為 department");
    assert!(ids.contains(&own_leave.to_string()), "應看得到自己的假單");
    assert!(
        ids.contains(&peer_leave.to_string()),
        "應看得到同部門他人的假單——這才是部門規則真正的斷言"
    );
    assert!(
        ids.contains(&child_leave.to_string()),
        "應看得到下層部門的假單（實習部門掛在試驗部下，屬同一工作團隊）"
    );
    assert!(
        !ids.contains(&outsider_leave.to_string()),
        "不該看得到不相干平行部門的假單——這是本功能的資料外洩面"
    );
}

/// 部門主管看全場。主管身分來自 `departments.manager_id`，不是角色。
#[tokio::test]
#[serial]
async fn department_manager_sees_all_departments() {
    let app = TestApp::spawn().await;

    let own = seed_department(&app, "mgrown", None).await;
    let far = seed_department(&app, "mgrfar", None).await;

    let (manager, manager_email) = seed_user(&app, "mgr", "EXPERIMENT_STAFF").await;
    set_department(&app, manager, own).await;
    set_manager(&app, own, manager).await;

    let (stranger, _) = seed_user(&app, "stranger", "EXPERIMENT_STAFF").await;
    set_department(&app, stranger, far).await;
    let far_leave = seed_leave(&app, stranger, "APPROVED", None, 4).await;

    let token = app.login(&manager_email, PW).await.expect("login 應成功");
    let (ids, scope) = fetch_calendar(&app, &token).await;

    assert_eq!(scope, "all", "部門主管的 scope 應為 all");
    assert!(
        ids.contains(&far_leave.to_string()),
        "部門主管應看得到其他部門的假單（全場）"
    );
}

/// 代理人穿透：跨部門、且不具任何全場權限，仍看得到自己要代理的假單。
#[tokio::test]
#[serial]
async fn proxy_sees_cross_department_leave_they_cover() {
    let app = TestApp::spawn().await;

    let a = seed_department(&app, "proxya", None).await;
    let b = seed_department(&app, "proxyb", None).await;

    let (proxy, proxy_email) = seed_user(&app, "proxy", "EXPERIMENT_STAFF").await;
    set_department(&app, proxy, a).await;

    let (applicant, _) = seed_user(&app, "applicant", "EXPERIMENT_STAFF").await;
    set_department(&app, applicant, b).await;

    let covered = seed_leave(&app, applicant, "PENDING_L1", Some(proxy), 5).await;
    let not_covered = seed_leave(&app, applicant, "APPROVED", None, 6).await;

    let token = app.login(&proxy_email, PW).await.expect("login 應成功");
    let (ids, scope) = fetch_calendar(&app, &token).await;

    assert_eq!(scope, "department", "代理人本身不因代理而升級為全場");
    assert!(
        ids.contains(&covered.to_string()),
        "應看得到自己要代理的跨部門假單——要頂班就得知道對方哪天不在"
    );
    assert!(
        !ids.contains(&not_covered.to_string()),
        "同一人在同部門的其他假單，若與自己無代理關係則不該可見"
    );
}

/// 只顯示已核准與審核中；草稿、已駁回、已取消不入曆。
#[tokio::test]
#[serial]
async fn draft_and_terminal_statuses_are_excluded() {
    let app = TestApp::spawn().await;

    let dept = seed_department(&app, "status", None).await;
    let (viewer, viewer_email) = seed_user(&app, "statusviewer", "EXPERIMENT_STAFF").await;
    set_department(&app, viewer, dept).await;

    let approved = seed_leave(&app, viewer, "APPROVED", None, 7).await;
    let pending = seed_leave(&app, viewer, "PENDING_DIRECTOR", None, 8).await;
    let draft = seed_leave(&app, viewer, "DRAFT", None, 9).await;
    let rejected = seed_leave(&app, viewer, "REJECTED", None, 10).await;
    let cancelled = seed_leave(&app, viewer, "CANCELLED", None, 11).await;

    let token = app.login(&viewer_email, PW).await.expect("login 應成功");
    let (ids, _) = fetch_calendar(&app, &token).await;

    assert!(ids.contains(&approved.to_string()), "已核准應顯示");
    assert!(
        ids.contains(&pending.to_string()),
        "審核中應顯示（這是原生行事曆相對 Google 同步的主要價值）"
    );
    for (id, label) in [(draft, "草稿"), (rejected, "已駁回"), (cancelled, "已取消")] {
        assert!(
            !ids.contains(&id.to_string()),
            "{label}不該入曆——那幾天人其實會在，畫上去反而誤導排班"
        );
    }
}

/// 沒有 `hr.leave.view_calendar` 的角色（例如 VET）應被擋在門外。
#[tokio::test]
#[serial]
async fn role_without_permission_is_forbidden() {
    let app = TestApp::spawn().await;

    let (_, vet_email) = seed_user(&app, "vet", "VET").await;
    let token = app.login(&vet_email, PW).await.expect("login 應成功");

    let res = app
        .auth_get(
            "/api/v1/hr/leaves/calendar?start_date=2026-01-01&end_date=2026-01-31",
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "未獲授權的角色應吃 403，實得 {}",
        res.status()
    );
}

/// 區間上限：避免有人改 query string 把整張表撈走。
#[tokio::test]
#[serial]
async fn oversized_range_is_rejected() {
    let app = TestApp::spawn().await;

    let (_, email) = seed_user(&app, "range", "EXPERIMENT_STAFF").await;
    let token = app.login(&email, PW).await.expect("login 應成功");

    let res = app
        .auth_get(
            "/api/v1/hr/leaves/calendar?start_date=2020-01-01&end_date=2030-01-01",
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        400,
        "超過上限的區間應回 400，實得 {}",
        res.status()
    );
}
