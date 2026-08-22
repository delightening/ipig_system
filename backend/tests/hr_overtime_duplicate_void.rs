// 回歸測試：加班補登防重（R86-2）與已核准加班單的作廢通道。
//
// 背景：`overtime_records` 只有 pkey(id)、`create_overtime` 也沒有重複檢查，
// 而補登腳本（docs/ops/overtime-backfill-template.js）明寫「中斷可重新執行」。
// 重跑會把同一時段建成兩筆，兩筆各自核准就各授一份補休 → 餘額翻倍；
// 而 `delete_overtime` 只受理 draft，已核准的重複單完全沒有撤銷路徑。
//
// 本檔鎖住兩件事：
//   1. 同一人／同一天／同一起訖時間不得重複建單（駁回的不佔位，可重新申請）。
//   2. 作廢：ADMIN 單簽 + 理由必填、不得作廢自己的單、補休已被使用者擋下；
//      作廢成功則收回補休餘額且原單保留為 voided。

use chrono::{NaiveDate, NaiveTime};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::{ROLE_ADMIN_LEGACY, ROLE_EXPERIMENT_STAFF};
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

/// 授予角色。查無該角色即 panic——`INSERT ... SELECT` 在角色不存在時會靜默插入 0 列，
/// 測試前提會無聲落空（教訓見 hr_overtime_sod.rs）。
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

/// `overtime_type` 只能是 A/B/C/D（DB 檢查約束 `chk_overtime_type`）。
/// C＝國定假日加班，固定授 8 小時補休——作廢要驗補休收回，故用 C。
fn payload(day: u32, overtime_type: &str) -> CreateOvertimeRequest {
    CreateOvertimeRequest {
        overtime_date: NaiveDate::from_ymd_opt(2026, 7, day).expect("valid date"),
        start_time: NaiveTime::from_hms_opt(18, 0, 0).expect("valid time"),
        end_time: NaiveTime::from_hms_opt(20, 0, 0).expect("valid time"),
        overtime_type: overtime_type.to_string(),
        reason: "防重／作廢測試".to_string(),
    }
}

/// 建單 → 送審 → 兩關核准（兩位不同負責人，避開 SoD），回傳加班單 id。
async fn approved_record(
    pool: &PgPool,
    applicant: Uuid,
    admin_a: Uuid,
    admin_b: Uuid,
    day: u32,
) -> Uuid {
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let created = HrService::create_overtime(pool, &actor, &payload(day, "C"))
        .await
        .expect("create overtime");
    HrService::submit_overtime(pool, &actor, created.id)
        .await
        .expect("submit overtime");

    let a = ActorContext::User(cur(admin_a, &[ROLE_ADMIN_LEGACY]));
    HrService::approve_overtime(pool, &a, created.id, "admin_staff")
        .await
        .expect("第一關核准");
    let b = ActorContext::User(cur(admin_b, &[ROLE_ADMIN_LEGACY]));
    let approved = HrService::approve_overtime(pool, &b, created.id, "admin")
        .await
        .expect("終審核准");
    assert_eq!(approved.status, "approved");
    created.id
}

async fn comp_time_rows(pool: &PgPool, overtime_id: Uuid) -> i64 {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM comp_time_balances WHERE overtime_record_id = $1")
            .bind(overtime_id)
            .fetch_one(pool)
            .await
            .expect("count comp_time_balances");
    n
}

/// 補登腳本重跑的核心情境：同一人同一天同一時段建第二筆應被擋下。
#[tokio::test]
#[serial]
async fn duplicate_overtime_is_rejected() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("dup-{s}@t.com")).await;
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));

    HrService::create_overtime(&pool, &actor, &payload(9, "A"))
        .await
        .expect("第一筆應建立成功");

    let err = HrService::create_overtime(&pool, &actor, &payload(9, "A"))
        .await
        .expect_err("同一時段第二筆應被擋下");
    assert!(
        format!("{err}").contains("已有一筆加班紀錄"),
        "錯誤訊息應指出重複，實際：{err}"
    );
}

/// 同一天不同時段仍可分別開單——防重不該擋掉正常的分段加班。
#[tokio::test]
#[serial]
async fn same_day_different_time_range_is_allowed() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("dup2-{s}@t.com")).await;
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));

    HrService::create_overtime(&pool, &actor, &payload(10, "A"))
        .await
        .expect("第一筆");

    let mut other = payload(10, "A");
    other.start_time = NaiveTime::from_hms_opt(21, 0, 0).expect("valid time");
    other.end_time = NaiveTime::from_hms_opt(22, 0, 0).expect("valid time");
    HrService::create_overtime(&pool, &actor, &other)
        .await
        .expect("同日不同時段應可建立");
}

/// 被駁回的單不佔位：同一時段可以重新申請。
#[tokio::test]
#[serial]
async fn rejected_record_does_not_block_recreate() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("rej-{s}@t.com")).await;
    let admin = mk_user(&pool, &format!("rejadm-{s}@t.com")).await;
    grant_role(&pool, admin, ROLE_ADMIN_LEGACY).await;

    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let first = HrService::create_overtime(&pool, &actor, &payload(11, "A"))
        .await
        .expect("建立");
    HrService::submit_overtime(&pool, &actor, first.id)
        .await
        .expect("送審");
    let admin_actor = ActorContext::User(cur(admin, &[ROLE_ADMIN_LEGACY]));
    HrService::reject_overtime(&pool, &admin_actor, first.id, "時段填錯")
        .await
        .expect("駁回");

    HrService::create_overtime(&pool, &actor, &payload(11, "A"))
        .await
        .expect("駁回後應可重新申請同一時段");
}

/// 作廢成功：狀態轉 voided、理由留存、補休餘額收回，且該時段可重新建單。
#[tokio::test]
#[serial]
async fn void_marks_record_and_releases_comp_time() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("void-{s}@t.com")).await;
    let admin_a = mk_user(&pool, &format!("voida-{s}@t.com")).await;
    let admin_b = mk_user(&pool, &format!("voidb-{s}@t.com")).await;
    grant_role(&pool, admin_a, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_b, ROLE_ADMIN_LEGACY).await;

    let id = approved_record(&pool, applicant, admin_a, admin_b, 12).await;
    assert_eq!(comp_time_rows(&pool, id).await, 1, "核准後應授出補休");

    let admin_actor = ActorContext::User(cur(admin_a, &[ROLE_ADMIN_LEGACY]));
    let voided = HrService::void_overtime(&pool, &admin_actor, id, "  補登重跑造成的重複單  ")
        .await
        .expect("作廢應成功");

    assert_eq!(voided.status, "voided");
    assert_eq!(
        voided.void_reason.as_deref(),
        Some("補登重跑造成的重複單"),
        "理由應 trim 後留存"
    );
    assert_eq!(comp_time_rows(&pool, id).await, 0, "作廢應收回補休餘額");

    // 作廢的單不佔防重名額
    let actor = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    HrService::create_overtime(&pool, &actor, &payload(12, "C"))
        .await
        .expect("作廢後應可重新建立同一時段");
}

/// 補休已被請假使用時擋下——否則會造出負餘額或默默動到已核准的假單。
#[tokio::test]
#[serial]
async fn cannot_void_when_comp_time_already_used() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("used-{s}@t.com")).await;
    let admin_a = mk_user(&pool, &format!("useda-{s}@t.com")).await;
    let admin_b = mk_user(&pool, &format!("usedb-{s}@t.com")).await;
    grant_role(&pool, admin_a, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_b, ROLE_ADMIN_LEGACY).await;

    let id = approved_record(&pool, applicant, admin_a, admin_b, 13).await;
    sqlx::query("UPDATE comp_time_balances SET used_hours = 4 WHERE overtime_record_id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("mark comp time used");

    let admin_actor = ActorContext::User(cur(admin_a, &[ROLE_ADMIN_LEGACY]));
    let err = HrService::void_overtime(&pool, &admin_actor, id, "想作廢")
        .await
        .expect_err("補休已使用應擋下");
    assert!(
        format!("{err}").contains("已使用"),
        "錯誤訊息應指出補休已使用，實際：{err}"
    );
    assert_eq!(comp_time_rows(&pool, id).await, 1, "擋下時不得動到餘額");
}

/// 不可作廢自己的加班單，即使自己是負責人。
#[tokio::test]
#[serial]
async fn cannot_void_own_overtime() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let owner_admin = mk_user(&pool, &format!("own-{s}@t.com")).await;
    let admin_a = mk_user(&pool, &format!("owna-{s}@t.com")).await;
    let admin_b = mk_user(&pool, &format!("ownb-{s}@t.com")).await;
    grant_role(&pool, owner_admin, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_a, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_b, ROLE_ADMIN_LEGACY).await;

    let id = approved_record(&pool, owner_admin, admin_a, admin_b, 14).await;

    let actor = ActorContext::User(cur(owner_admin, &[ROLE_ADMIN_LEGACY]));
    let err = HrService::void_overtime(&pool, &actor, id, "自己作廢自己")
        .await
        .expect_err("不得作廢自己的單");
    assert!(
        format!("{err}").contains("不可作廢自己"),
        "錯誤訊息應指出不可作廢自己，實際：{err}"
    );
}

/// 非負責人不得作廢；理由空白亦擋下。
#[tokio::test]
#[serial]
async fn void_requires_admin_and_reason() {
    let pool = pool().await;
    let s = Uuid::new_v4();
    let applicant = mk_user(&pool, &format!("perm-{s}@t.com")).await;
    let admin_a = mk_user(&pool, &format!("perma-{s}@t.com")).await;
    let admin_b = mk_user(&pool, &format!("permb-{s}@t.com")).await;
    grant_role(&pool, admin_a, ROLE_ADMIN_LEGACY).await;
    grant_role(&pool, admin_b, ROLE_ADMIN_LEGACY).await;

    let id = approved_record(&pool, applicant, admin_a, admin_b, 15).await;

    let staff = ActorContext::User(cur(applicant, &[ROLE_EXPERIMENT_STAFF]));
    let err = HrService::void_overtime(&pool, &staff, id, "沒有權限")
        .await
        .expect_err("非負責人應被擋下");
    assert!(
        format!("{err}").contains("僅負責人"),
        "錯誤訊息應指出權限不足，實際：{err}"
    );

    let admin_actor = ActorContext::User(cur(admin_a, &[ROLE_ADMIN_LEGACY]));
    let err = HrService::void_overtime(&pool, &admin_actor, id, "   ")
        .await
        .expect_err("空白理由應被擋下");
    assert!(
        format!("{err}").contains("理由"),
        "錯誤訊息應指出理由必填，實際：{err}"
    );
    assert_eq!(comp_time_rows(&pool, id).await, 1, "擋下時不得動到餘額");
}
