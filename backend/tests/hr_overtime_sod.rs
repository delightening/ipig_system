// 回歸測試：加班審核的職責分離（SoD）。
//
// 背景：`approve_overtime` 原本只比對 status，既沒有「不可自審」也沒有
// 「兩關不得同人」的檢查。自審防護只做在列表的 can_approve 旗標上
// （且該處註解宣稱「與 approve_overtime handler 一致」，實際不然），
// 直接呼叫服務／API 即可核准自己的加班。
//
// 規則對齊請假模組（services/hr/leave.rs）：
//   - 不可審核自己：任何關卡、任何角色皆不放寬。
//   - 兩關不得同人：僅在「確實無其他合格終審者」時放寬代批，
//     否則單一審批人組織會把加班單卡死。

use chrono::{NaiveDate, NaiveTime};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::{
    ROLE_ADMIN_LEGACY, ROLE_ADMIN_STAFF, ROLE_EXPERIMENT_STAFF, ROLE_SYSTEM_ADMIN,
};
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::CreateOvertimeRequest;
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

/// 把角色寫進 user_roles —— `final_stage_has_other_approver` 是查 DB 而非 JWT，
/// 只給 `CurrentUser.roles` 不夠。
///
/// **查無該角色即 panic**：`SELECT ... WHERE code = $2` 在角色不存在時會靜默
/// 插入 0 列，測試前提（存在其他合格終審者）就無聲落空，而斷言反而「通過」。
/// 實例：`roles` 表裡沒有 `SYSTEM_ADMIN` 這一列，只有 legacy 的 `admin`。
async fn grant_role(pool: &PgPool, user_id: Uuid, role_code: &str) {
    let affected = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) \
         SELECT $1, id FROM roles WHERE code = $2 \
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("grant role")
    .rows_affected();
    assert_eq!(
        affected, 1,
        "roles 表查無 code = {role_code}，測試前提不成立（角色授予會靜默失效）"
    );
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

fn payload(day: u32) -> CreateOvertimeRequest {
    CreateOvertimeRequest {
        overtime_date: NaiveDate::from_ymd_opt(2026, 7, day).expect("valid date"),
        start_time: NaiveTime::from_hms_opt(18, 0, 0).expect("valid time"),
        end_time: NaiveTime::from_hms_opt(20, 0, 0).expect("valid time"),
        // DB 檢查約束 `chk_overtime_type` 只允許 A/B/C/D。"A" 對應
        // `services/hr/overtime.rs` 的 `OVERTIME_TYPE_WEEKDAY`（平日加班）；
        // 該常數所在的 `services::hr` 是私有模組，整合測試取不到，故寫字面值。
        overtime_type: "A".to_string(),
        reason: "SoD 測試".to_string(),
    }
}

/// 建立並送出一張加班單，回傳其 id（狀態為 pending_admin_staff）。
async fn submitted_record(pool: &PgPool, applicant: Uuid, day: u32) -> Uuid {
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_overtime(pool, &actor, &payload(day))
        .await
        .expect("create overtime");
    HrService::submit_overtime(pool, &actor, created.id)
        .await
        .expect("submit overtime");
    created.id
}

/// 不可審核自己的加班單——這是本次修復的核心，先前直接呼叫服務即可繞過。
#[tokio::test]
#[serial]
async fn cannot_approve_own_overtime() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    // 申請人本身具備行政與 admin 權限（最寬鬆情境），仍不得自審
    let applicant = mk_user(&pool, &format!("selfapp-{s}@t.com")).await;
    grant_role(&pool, applicant, ROLE_ADMIN_STAFF).await;
    grant_role(&pool, applicant, ROLE_ADMIN_LEGACY).await;

    let id = submitted_record(&pool, applicant, 6).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_ADMIN_STAFF, ROLE_ADMIN_LEGACY]));
    let err = HrService::approve_overtime(&pool, &actor, id, "admin_staff")
        .await
        .expect_err("自審應被擋下");
    assert!(
        format!("{err}").contains("不可審核自己"),
        "錯誤訊息應指出自審，實際：{err}"
    );
}

/// 兩關不得同人：有其他合格終審者時，批過第一關者不得再批終審。
#[tokio::test]
#[serial]
async fn same_person_cannot_approve_both_stages_when_another_admin_exists() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app-{s}@t.com")).await;
    let admin_a = mk_user(&pool, &format!("admina-{s}@t.com")).await;
    let admin_b = mk_user(&pool, &format!("adminb-{s}@t.com")).await;
    grant_role(&pool, admin_a, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_b, ROLE_ADMIN_LEGACY).await;

    let id = submitted_record(&pool, applicant, 7).await;
    let a = ActorContext::User(cur(admin_a, &[ROLE_ADMIN_LEGACY]));

    HrService::approve_overtime(&pool, &a, id, "admin_staff")
        .await
        .expect("第一關應可核准");

    // admin_b 尚未參與本單 → 存在其他合格終審者 → SoD 應收緊
    let err = HrService::approve_overtime(&pool, &a, id, "admin")
        .await
        .expect_err("同一人不得包辦兩關");
    assert!(
        format!("{err}").contains("職責分離"),
        "錯誤訊息應指出職責分離，實際：{err}"
    );

    // 換另一位負責人終審 → 應成功
    let b = ActorContext::User(cur(admin_b, &[ROLE_ADMIN_LEGACY]));
    let approved = HrService::approve_overtime(&pool, &b, id, "admin")
        .await
        .expect("其他負責人終審應成功");
    assert_eq!(approved.status, "approved");
}

/// 代批放寬：確實沒有其他合格終審者時，同一人可完成兩關，
/// 否則單一審批人組織的加班單會永久卡住。
///
/// ⚠️ 本例必須營造「全庫只剩一位合格終審者」，而
/// `final_stage_has_other_approver` 是**全域**查詢，因此無可避免要暫時停用
/// 其他 admin。為了不把共用測試 DB 留在髒狀態（會讓平行執行的其他測試
/// 隨機失敗），這裡：
///   1. 先記下被動到的 user id；
///   2. 兩次核准都用 `Result` 承接、**不在中途 panic**；
///   3. 還原 `is_active` 之後才做斷言；
///   4. 本檔三例皆掛 `#[serial]`——否則本例的停用會與
///      `same_person_cannot_approve_both_stages_*`（需要 admin_b 保持啟用）
///      在同一 binary 內對撞，變成間歇性紅燈。
///
/// ⚠️ 殘留風險：`#[serial]` 只序列化**同一測試 binary 內**的案例，擋不住
/// cargo 平行執行的其他 binary。上面第 1–3 點把窗口壓到最小，但要根治得等
/// 測試 DB 改成每個 binary 獨立（另案）。
#[tokio::test]
#[serial]
async fn sole_admin_may_approve_both_stages() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("app2-{s}@t.com")).await;
    let admin = mk_user(&pool, &format!("solo-{s}@t.com")).await;
    grant_role(&pool, admin, ROLE_ADMIN_LEGACY).await;

    let id = submitted_record(&pool, applicant, 8).await;

    // 先取出「除了本例的 admin 之外、目前仍啟用」的合格終審者
    let others: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT u.id FROM users u \
         JOIN user_roles ur ON ur.user_id = u.id \
         JOIN roles r ON r.id = ur.role_id \
         WHERE r.code IN ($1, $2) AND u.is_active = true AND u.id <> $3",
    )
    .bind(ROLE_SYSTEM_ADMIN)
    .bind(ROLE_ADMIN_LEGACY)
    .bind(admin)
    .fetch_all(&pool)
    .await
    .expect("query other admins");
    let other_ids: Vec<Uuid> = others.into_iter().map(|(id,)| id).collect();

    sqlx::query("UPDATE users SET is_active = false WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("deactivate other admins");

    // 中途不 panic，確保底下的還原一定會執行
    let a = ActorContext::User(cur(admin, &[ROLE_ADMIN_LEGACY]));
    let first = HrService::approve_overtime(&pool, &a, id, "admin_staff").await;
    let second = HrService::approve_overtime(&pool, &a, id, "admin").await;

    sqlx::query("UPDATE users SET is_active = true WHERE id = ANY($1)")
        .bind(&other_ids)
        .execute(&pool)
        .await
        .expect("restore other admins");

    first.expect("第一關應可核准");
    let approved = second.expect("無其他合格終審者時應放寬代批");
    assert_eq!(approved.status, "approved");
}
