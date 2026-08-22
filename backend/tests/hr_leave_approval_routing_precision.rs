//! 請假批准接上「待處理」，且不再是「不分關卡的角色廣播」。
//!
//! 背景：`leave_submitted` 原本的通知路由是角色廣播（`admin` + `DIRECTOR`），不分審核
//! 關卡——假單卡在單位主管（L1）關時，DIRECTOR 也會收到，但 `can_user_approve_leave`
//! 在這個階段會拒絕他。若直接把這個通知改成置頂待辦，會出現「驚嘆號告訴你要做事，
//! 點下去卻 403」——正是這次要根除的問題。migration 145 把路由改成 resolver 型
//! （`leave_current_stage_approvers`），依假單目前狀態動態算出該通知誰。
//!
//! 本檔鎖住三件事：
//! 1. 只有「這一關真的能點下去核准」的人會被 pin，不是整批角色。
//! 2. 核准每過一關都要解除當關的待辦；中途過關（L1→終審）要為下一關的人建立新待辦
//!    ——這是修復前的既有缺口（單位主管核准後，負責人從未被通知過）。
//! 3. 卡關（沒有合法審核人）時退回管理員後備名單，不是靜默漏通知。

use chrono::NaiveDate;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::{ROLE_ADMIN_LEGACY, ROLE_DIRECTOR, ROLE_EXPERIMENT_STAFF};
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateLeaveRequest, LeaveStatus};
use erp_backend::services::{AuditService, HrService, NotificationService};

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

fn draft_payload(proxy: Uuid) -> CreateLeaveRequest {
    CreateLeaveRequest {
        proxy_user_id: Some(proxy),
        leave_type: "PERSONAL".to_string(),
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

/// `(kind, priority)`；`None` 表示這個人在這則假單上完全沒有通知列。
async fn pin_state(pool: &PgPool, leave_id: Uuid, user_id: Uuid) -> Option<(String, i16)> {
    sqlx::query_as::<_, (String, i16)>(
        "SELECT kind, priority FROM notifications \
         WHERE related_entity_type = 'leave_request' AND related_entity_id = $1 AND user_id = $2",
    )
    .bind(leave_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .expect("query notification")
}

/// 送到 PENDING_L1：申請人 + 代理人 + 部門主管，送審並代理確認。
async fn submit_to_pending_l1(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid) {
    let s = Uuid::new_v4();
    let applicant = mk_user(pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(pool, &format!("proxy-{s}@t.com")).await;
    let mgr = mk_user(pool, &format!("mgr-{s}@t.com")).await;
    let dept = mk_department(pool, mgr, &s).await;
    assign_department(pool, applicant, dept).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(pool, &actor, &draft_payload(proxy))
        .await
        .expect("create_leave");
    HrService::submit_leave(pool, &actor, created.id)
        .await
        .expect("submit_leave");
    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let after_proxy = HrService::proxy_confirm_leave(pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(after_proxy.status, LeaveStatus::PendingL1.as_str());

    (created.id, applicant, mgr, dept)
}

/// resolver 只 pin 真正這一關能核准的人：PENDING_L1 只 pin 部門主管，
/// 不 pin 一個八竿子打不著的 DIRECTOR（舊的角色廣播會兩個都通知）。
#[tokio::test]
#[serial]
async fn leave_current_stage_approvers_pins_only_dept_manager_at_l1() {
    let pool = pool().await;
    let (leave_id, applicant, mgr, _dept) = submit_to_pending_l1(&pool).await;
    let unrelated_director = mk_user(&pool, &format!("dir-{}@t.com", Uuid::new_v4())).await;
    assign_role(&pool, unrelated_director, ROLE_DIRECTOR).await;

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(leave_id, applicant, "PERSONAL", "2026-07-03", "2026-07-03")
        .await
        .expect("notify_leave_submitted");

    let (kind, priority) = pin_state(&pool, leave_id, mgr)
        .await
        .expect("部門主管應收到待辦");
    assert_eq!(kind, "action", "請假批准是待辦，不是單純告知");
    assert!(priority > 0, "應立即置頂");

    assert_eq!(
        pin_state(&pool, leave_id, unrelated_director).await,
        None,
        "還輪不到終審關的 DIRECTOR 不該被通知——這正是舊角色廣播的問題"
    );
}

/// PENDING_DIRECTOR 只 pin 合法的 DIRECTOR，不 pin 別人部門的主管。
#[tokio::test]
#[serial]
async fn leave_current_stage_approvers_pins_only_director_at_final_stage() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    // 申請人無部門 → 代理確認後自動跳關直接進 PENDING_DIRECTOR。
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let director = mk_user(&pool, &format!("dir-{s}@t.com")).await;
    assign_role(&pool, director, ROLE_DIRECTOR).await;
    let unrelated_mgr = mk_user(&pool, &format!("mgr-{s}@t.com")).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");
    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    let after_proxy = HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm");
    assert_eq!(after_proxy.status, LeaveStatus::PendingDirector.as_str());

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(
        created.id,
        applicant,
        "PERSONAL",
        "2026-07-03",
        "2026-07-03",
    )
    .await
    .expect("notify_leave_submitted");

    let (_, priority) = pin_state(&pool, created.id, director)
        .await
        .expect("負責人應收到待辦");
    assert!(priority > 0);
    assert_eq!(
        pin_state(&pool, created.id, unrelated_mgr).await,
        None,
        "與此假單無關的部門主管不該被通知"
    );
}

/// 沒有任何合法 DIRECTOR 時（卡關）退回管理員後備名單，不是靜默漏通知。
///
/// ⚠️ `director_eligible_directors` 是**全域**查詢（同 `hr_overtime_sod.rs` 的
/// `final_stage_has_other_approver`），本例必須營造「全庫沒有合法 DIRECTOR」，
/// 否則平行跑的其他測試留下的 DIRECTOR 會讓 fallback 條件不成立、測試假紅。
/// 比照該檔 `sole_admin_may_approve_both_stages` 的作法：先記下、暫時停用其他
/// DIRECTOR，中途不 panic，最後一定還原；並掛 `#[serial]` 序列化同一 binary
/// 內的案例（跨 binary 的殘留風險見該檔註解，本 session 一律單一 binary 執行）。
#[tokio::test]
#[serial]
async fn leave_current_stage_approvers_falls_back_to_admin_when_no_director() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let admin = mk_user(&pool, &format!("admin-{s}@t.com")).await;
    // 本測試庫的 roles 表只 seed 了 legacy 'admin'，沒有 SYSTEM_ADMIN
    // （同 hr_overtime_sod.rs 的既有發現）；admin_roster 兩者都認，這裡選一個即可。
    assign_role(&pool, admin, ROLE_ADMIN_LEGACY).await;
    // 刻意不建立任何 DIRECTOR，但要先把全庫既有的 DIRECTOR 暫時停用。
    //
    // ⚠️ 這是**全庫範圍的暫時性改動**，不是本測試自己造的資料（codeant-ai PR #67 review）。
    // 無法避免：`director_eligible_directors` 查的是「全庫有沒有在職 DIRECTOR」，
    // 「無合法 DIRECTOR」這個前提本身就是全域性質，沒有 per-leave 的造法。
    //
    // 已有的防護與殘留風險，寫清楚免得日後有人以為這裡是安全的：
    // - 還原（本區塊底下的 UPDATE ... is_active = true）在**所有邏輯路徑**都會執行：
    //   中間的步驟包在回傳 `Result` 的 async block 裡、用 `map_err` 不 panic，
    //   失敗時先還原再 `result.expect(...)`。
    // - `#[serial]` 只在同一測試 binary 內生效，擋不住跨 binary 併發。
    // - **殘留風險**：行程被硬中止（Ctrl-C / OOM kill）時還原不會執行，會留下被停用的
    //   帳號。影響範圍限於同一次執行——CI 每輪用全新的 postgres service container，
    //   本機依 PARALLEL_SESSIONS.md §5 也是各 session 自己的丟棄庫，不會跨輪汙染。

    let other_directors: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT u.id FROM users u \
         JOIN user_roles ur ON ur.user_id = u.id \
         JOIN roles r ON r.id = ur.role_id \
         WHERE r.code = $1 AND u.is_active = true",
    )
    .bind(ROLE_DIRECTOR)
    .fetch_all(&pool)
    .await
    .expect("query existing directors");
    let other_ids: Vec<Uuid> = other_directors.into_iter().map(|(id,)| id).collect();
    sqlx::query("UPDATE users SET is_active = false WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("deactivate other directors");

    // 中途不 panic，確保底下的還原一定會執行。
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let result: Result<(), String> = async {
        let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy))
            .await
            .map_err(|e| e.to_string())?;
        HrService::submit_leave(&pool, &actor, created.id)
            .await
            .map_err(|e| e.to_string())?;
        let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
        HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
            .await
            .map_err(|e| e.to_string())?;

        let svc = NotificationService::new(pool.clone());
        svc.notify_leave_submitted(
            created.id,
            applicant,
            "PERSONAL",
            "2026-07-03",
            "2026-07-03",
        )
        .await
        .map_err(|e| e.to_string())?;

        let (_, priority) = pin_state(&pool, created.id, admin)
            .await
            .ok_or("無合法 DIRECTOR 時，管理員應收到卡關代批的待辦")?;
        if priority <= 0 {
            return Err(format!("priority 應 > 0，實際 {priority}"));
        }
        Ok(())
    }
    .await;

    sqlx::query("UPDATE users SET is_active = true WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("restore other directors");

    result.expect("admin fallback should have pinned the admin");
}

/// 卡關代批 fallback 必須排除申請人本人：申請人自己持有管理員角色、且該關卡關
/// （無合法主管）時，`admin_roster` 不得把申請人本人放進待辦名單。
///
/// 若不排除，申請人會收到自己請假的置頂待辦，但 `can_user_approve_leave` 的
/// 第一條規則是「不可審核自己的假單，任何角色皆不放寬」——點下去必定 403，
/// 正是「請假批准接上待處理」這整個 PR 要根除的失效模式（CodeRabbit PR #67
/// review）。
#[tokio::test]
#[serial]
async fn admin_roster_excludes_applicant_even_when_stuck() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("admin-app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    // 申請人本人是唯一的管理員——admin_roster 的候選名單裡只有他自己。
    assign_role(&pool, applicant, ROLE_ADMIN_LEGACY).await;
    // 申請人沒有部門（未指派 department_id）→ l1_has_eligible_approver 為 false，
    // proxy_confirm_leave 會直接跳關到 PENDING_DIRECTOR（見該函式的卡關自動跳關）。
    // 要讓終審關也卡住、真正落到 admin_roster fallback，須比照既有測試停用全庫
    // 既有 DIRECTOR，否則殘留的其他 DIRECTOR 會被通知，測試等於沒驗到這條路徑。

    let other_directors: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT u.id FROM users u \
         JOIN user_roles ur ON ur.user_id = u.id \
         JOIN roles r ON r.id = ur.role_id \
         WHERE r.code = $1 AND u.is_active = true",
    )
    .bind(ROLE_DIRECTOR)
    .fetch_all(&pool)
    .await
    .expect("query existing directors");
    let other_ids: Vec<Uuid> = other_directors.into_iter().map(|(id,)| id).collect();
    sqlx::query("UPDATE users SET is_active = false WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("deactivate other directors");

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let result: Result<(), String> = async {
        let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy))
            .await
            .map_err(|e| e.to_string())?;
        HrService::submit_leave(&pool, &actor, created.id)
            .await
            .map_err(|e| e.to_string())?;
        let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
        let after_confirm = HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
            .await
            .map_err(|e| e.to_string())?;
        if after_confirm.status != LeaveStatus::PendingDirector.as_str() {
            return Err(format!(
                "應已自動跳關到 PENDING_DIRECTOR，實際 {}",
                after_confirm.status
            ));
        }

        let svc = NotificationService::new(pool.clone());
        svc.notify_leave_submitted(
            created.id,
            applicant,
            "PERSONAL",
            "2026-07-03",
            "2026-07-03",
        )
        .await
        .map_err(|e| e.to_string())?;

        match pin_state(&pool, created.id, applicant).await {
            None => Ok(()),
            Some((_, priority)) => Err(format!(
                "申請人不應被 admin_roster fallback 通知自己的假單，實際 priority={priority}"
            )),
        }
    }
    .await;

    sqlx::query("UPDATE users SET is_active = true WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("restore other directors");

    result.expect("admin_roster should exclude the applicant even when stuck");
}

/// 核准過關解除當關待辦；中途過關（L1→終審）要為下一關的人建立新待辦
/// ——這是修復前的既有缺口：單位主管核准後，負責人從未被通知過。
///
/// CodeRabbit PR #67 review 後更新：下一關待辦的建立已下沉至
/// `HrService::approve_leave` 自己的 tx（與當關待辦的解除原子化），不再是
/// handler 端 commit 後 best-effort 的 `tokio::spawn`。本測試因此直接斷言
/// `approve_leave` 回傳後負責人的待辦已存在，不再另外手動呼叫
/// `notify_leave_submitted` 模擬 handler 端動作——那個模擬呼叫舊版必要，
/// 現在反而會遮住「`approve_leave` 本身到底有沒有原子建立待辦」這件事
/// （若拿掉手動呼叫測試仍然通過，才是真的驗證到這次修復）。
#[tokio::test]
#[serial]
async fn approve_leave_resolves_current_stage_and_next_stage_can_be_notified() {
    let pool = pool().await;
    let (leave_id, applicant, mgr, dept) = submit_to_pending_l1(&pool).await;
    let director = mk_user(&pool, &format!("dir-{}@t.com", Uuid::new_v4())).await;
    assign_role(&pool, director, ROLE_DIRECTOR).await;
    let _ = dept;

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(leave_id, applicant, "PERSONAL", "2026-07-03", "2026-07-03")
        .await
        .expect("notify_leave_submitted（模擬 proxy_confirm handler 呼叫）");
    assert_eq!(
        pin_state(&pool, leave_id, mgr).await.map(|(_, p)| p),
        Some(1),
        "核准前應先確認主管的待辦真的存在"
    );
    assert_eq!(
        pin_state(&pool, leave_id, director).await,
        None,
        "核准前負責人不應有任何通知列"
    );

    let mgr_actor = ActorContext::User(cur(mgr, &[ROLE_EXPERIMENT_STAFF]));
    let after_mgr = HrService::approve_leave(&pool, &mgr_actor, leave_id, None)
        .await
        .expect("manager approve");
    assert_eq!(after_mgr.status, LeaveStatus::PendingDirector.as_str());

    let (_, mgr_priority) = pin_state(&pool, leave_id, mgr)
        .await
        .expect("主管的待辦仍應留著（鈴鐺歷史）");
    assert_eq!(mgr_priority, 0, "主管核准後，這關的待辦應解除");

    // 不再手動呼叫 notify_leave_submitted——approve_leave 回傳時，下一關
    // （負責人）的待辦應已在同一 tx 內原子建立完成，直接查得到。
    let (_, dir_priority) = pin_state(&pool, leave_id, director)
        .await
        .expect("approve_leave 回傳後，負責人的待辦應已原子建立完成");
    assert!(dir_priority > 0, "負責人的待辦應為置頂狀態");
}

/// 最終核准解除負責人的待辦，不再建立新的。
#[tokio::test]
#[serial]
async fn approve_leave_final_resolves_directors_pin() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let proxy = mk_user(&pool, &format!("proxy-{s}@t.com")).await;
    let director = mk_user(&pool, &format!("dir-{s}@t.com")).await;
    assign_role(&pool, director, ROLE_DIRECTOR).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_leave(&pool, &actor, &draft_payload(proxy))
        .await
        .expect("create_leave");
    HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave");
    let proxy_actor = ActorContext::User(cur(proxy, &[ROLE_EXPERIMENT_STAFF]));
    HrService::proxy_confirm_leave(&pool, &proxy_actor, created.id)
        .await
        .expect("proxy_confirm（無部門，自動跳關至 PENDING_DIRECTOR）");

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(
        created.id,
        applicant,
        "PERSONAL",
        "2026-07-03",
        "2026-07-03",
    )
    .await
    .expect("notify_leave_submitted");

    let dir_actor = ActorContext::User(cur(director, &[ROLE_DIRECTOR]));
    let after_dir = HrService::approve_leave(&pool, &dir_actor, created.id, None)
        .await
        .expect("director approve");
    assert_eq!(after_dir.status, LeaveStatus::Approved.as_str());

    let (_, priority) = pin_state(&pool, created.id, director)
        .await
        .expect("待辦仍應留著（鈴鐺歷史）");
    assert_eq!(priority, 0, "最終核准後待辦應解除");
}

/// 駁回是終態，這一關的待辦要解除。
#[tokio::test]
async fn reject_leave_resolves_the_todo() {
    let pool = pool().await;
    let (leave_id, applicant, mgr, _dept) = submit_to_pending_l1(&pool).await;

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(leave_id, applicant, "PERSONAL", "2026-07-03", "2026-07-03")
        .await
        .expect("notify_leave_submitted");

    let mgr_actor = ActorContext::User(cur(mgr, &[ROLE_EXPERIMENT_STAFF]));
    let rejected = HrService::reject_leave(&pool, &mgr_actor, leave_id, "不核准")
        .await
        .expect("manager reject");
    assert_eq!(rejected.status, LeaveStatus::Rejected.as_str());

    let (_, priority) = pin_state(&pool, leave_id, mgr)
        .await
        .expect("待辦仍應留著（鈴鐺歷史）");
    assert_eq!(priority, 0, "駁回後應離開待處理清單");
}

/// 非審核中狀態（例如已核准）不該讓 resolver 誤判成「還有人要審」。
#[tokio::test]
#[serial]
async fn current_stage_approvers_empty_for_non_pending_status() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let director = mk_user(&pool, &format!("dir-{s}@t.com")).await;
    assign_role(&pool, director, ROLE_DIRECTOR).await;

    let actor = ActorContext::User(cur(director, &[ROLE_DIRECTOR]));
    // 負責人本人請假走報備制，送出即核准（見 HrService::submit_leave）。
    let created = HrService::create_leave(
        &pool,
        &actor,
        &CreateLeaveRequest {
            proxy_user_id: None,
            ..draft_payload(director)
        },
    )
    .await
    .expect("create_leave");
    let submitted = HrService::submit_leave(&pool, &actor, created.id)
        .await
        .expect("submit_leave（報備制）");
    assert_eq!(submitted.status, LeaveStatus::Approved.as_str());

    let approvers = HrService::current_stage_approvers(&pool, created.id)
        .await
        .expect("current_stage_approvers");
    assert!(
        approvers.is_empty(),
        "已核准的假單不該有任何「這關該通知誰」——不存在審核中的這一關"
    );
}
