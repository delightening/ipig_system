//! 加班終審關的候選名單必須與 `approve_overtime` 的職務分離守衛同源。
//!
//! ## 修的是什麼
//!
//! `resolve_for_overtime` 原本只排除申請人。但 `approve_overtime`（`overtime.rs`）
//! 的終審關另有一條：**批過前關的人不得再批終審**——而且**只在還有其他人可簽時
//! 才收緊**，否則單一審批人組織會把加班單卡死。
//!
//! 淨結果：tooltip 會列出「已批過前關、按下去必定被擋」的人。這正是本機制唯一
//! 硬規則（名單與守衛同源）要防的事（CodeRabbit 於 #30 指出）。
//!
//! 修法是照 `leave.rs::director_eligible_directors` 的形狀，把判準抽成
//! `HrService::final_stage_eligible_approvers`，**守衛與名單建在同一個查詢上**，
//! 而不是在解析器裡複製一份條件——複製出來的那份遲早會跟本尊分岔。
//!
//! ## 另外釘住「已等待 N 天」的起點
//!
//! 終審關的等待從**第一關核准那一刻**起算，不是從送出起算。用 `submitted_at`
//! 會把第一關的等待時間也算進去，讓每一筆看起來都比實際更急。

mod common;
use common::TestApp;
use erp_backend::services::pending_owner::resolve_for_overtime;
use erp_backend::services::HrService;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 管理員在 `roles` 表裡的真實代碼——是 `admin` 不是 `SYSTEM_ADMIN`。
/// （`RULES_BACKEND.md` §7.1 記載過這個坑；`constants.rs` 兩個都定義，DB 只有這個。）
const ADMIN_ROLE: &str = "admin";

async fn seed_user_with_role(pool: &PgPool, label: &str, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(id)
    .bind(format!("{label}-{}@example.com", &suffix[..8]))
    .bind(format!("{label}-{}", &suffix[..6]))
    .execute(pool)
    .await
    .expect("seed user");

    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("grant role");

    id
}

/// 建一筆待終審的加班單（`pending_admin`）。
///
/// `day_offset` 讓每筆的 (user, date, start, end) 不同——
/// `idx_overtime_records_no_duplicate` 對非 rejected/voided 的列是唯一索引。
async fn seed_pending_admin_overtime(pool: &PgPool, applicant: Uuid, day_offset: i32) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO overtime_records
            (id, user_id, overtime_date, start_time, end_time, hours, overtime_type,
             comp_time_hours, comp_time_expires_at, reason, status,
             submitted_at, approved_at)
        VALUES
            ($1, $2, DATE '2026-01-01' + $3,
             (DATE '2026-01-01' + $3)::timestamptz + interval '18 hours',
             (DATE '2026-01-01' + $3)::timestamptz + interval '20 hours',
             2, 'A', 2, DATE '2026-12-31', '加班測試', 'pending_admin',
             now() - interval '10 days', now() - interval '2 days')
        "#,
    )
    .bind(id)
    .bind(applicant)
    .bind(day_offset)
    .execute(pool)
    .await
    .expect("seed overtime record");
    id
}

/// 記一筆「這個人核准過本單」——SoD 判定讀的就是這張表。
async fn record_approval(pool: &PgPool, overtime_id: Uuid, approver: Uuid, level: &str) {
    sqlx::query(
        "INSERT INTO overtime_approvals (overtime_record_id, approver_id, approval_level, action) \
         VALUES ($1, $2, $3, 'APPROVE')",
    )
    .bind(overtime_id)
    .bind(approver)
    .bind(level)
    .execute(pool)
    .await
    .expect("record approval");
}

/// 目前在職、非申請人的管理員（不論批過與否）。終審關「代批」時的完整名單。
async fn all_admins_except(pool: &PgPool, applicant: Uuid) -> Vec<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT u.id FROM users u \
         JOIN user_roles ur ON ur.user_id = u.id \
         JOIN roles r ON r.id = ur.role_id \
         WHERE r.code IN ('SYSTEM_ADMIN', 'admin') \
           AND u.is_active = true AND u.deleted_at IS NULL AND u.id <> $1",
    )
    .bind(applicant)
    .fetch_all(pool)
    .await
    .expect("query admins")
}

/// 候選總人數（`candidates` 只列前 3 個，其餘記在 `overflow`）。
fn total(owner: &erp_backend::models::PendingOwner) -> usize {
    owner.candidates.len() + owner.overflow as usize
}

/// 權威來源本身：批過前關的人不得再列為終審關合法核准人。
#[tokio::test]
#[serial]
async fn eligible_approvers_exclude_whoever_already_approved() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let applicant = seed_user_with_role(pool, "ot-applicant", "EXPERIMENT_STAFF").await;
    let admin_a = seed_user_with_role(pool, "ot-admin-a", ADMIN_ROLE).await;
    let admin_b = seed_user_with_role(pool, "ot-admin-b", ADMIN_ROLE).await;
    let ot = seed_pending_admin_overtime(pool, applicant, 1).await;

    let before = HrService::final_stage_eligible_approvers(pool, &[ot])
        .await
        .expect("eligible before");
    let before_ids = before.get(&ot).cloned().unwrap_or_default();
    assert!(before_ids.contains(&admin_a) && before_ids.contains(&admin_b));
    assert!(
        !before_ids.contains(&applicant),
        "申請人永遠不得列入終審關（`approve_overtime` 的 `before.user_id == approver_id` 硬擋）"
    );

    record_approval(pool, ot, admin_a, "admin_staff").await;

    let after = HrService::final_stage_eligible_approvers(pool, &[ot])
        .await
        .expect("eligible after");
    let after_ids = after.get(&ot).cloned().unwrap_or_default();
    assert!(
        !after_ids.contains(&admin_a),
        "admin_a 批過前關，終審關的守衛會擋他（在還有其他人可簽時）。\n\
         把他留在名單裡＝tooltip 叫使用者去催一個按下去會拿 403 的人。\n\
         實際：{after_ids:?}"
    );
    assert!(
        after_ids.contains(&admin_b),
        "admin_b 沒批過，必須留著。實際：{after_ids:?}"
    );
}

/// 解析器必須真的用到那個權威來源——不是只有守衛用。
#[tokio::test]
#[serial]
async fn final_stage_candidates_drop_the_prior_approver() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let applicant = seed_user_with_role(pool, "ot2-applicant", "EXPERIMENT_STAFF").await;
    let admin_a = seed_user_with_role(pool, "ot2-admin-a", ADMIN_ROLE).await;
    seed_user_with_role(pool, "ot2-admin-b", ADMIN_ROLE).await;
    let ot = seed_pending_admin_overtime(pool, applicant, 2).await;

    let before = resolve_for_overtime(pool, &[ot]).await.expect("resolve");
    let before_total = total(before.get(&ot).expect("待終審必須有 pending_owner"));

    record_approval(pool, ot, admin_a, "admin_staff").await;

    let after = resolve_for_overtime(pool, &[ot]).await.expect("resolve");
    let after_total = total(after.get(&ot).expect("待終審必須有 pending_owner"));

    assert_eq!(
        after_total,
        before_total - 1,
        "批過前關的 admin_a 應從候選名單消失。\n\
         用總人數（candidates + overflow）而非人名比對，因為名單只列前 3 個，\n\
         測試庫裡可能還有其他種子管理員。before={before_total} after={after_total}"
    );
}

/// 卡關代批：沒有其他人可簽時，SoD 放寬，名單必須把人放回來。
///
/// ⚠️ 這是這次修正最容易寫錯的一半。只做「剔除批過的人」會讓單一審批人組織的
/// 加班單顯示成「沒有任何人能處理」，而實際上守衛會放行代批——那是**反過來的**
/// 誤導：使用者以為卡死，其實按下去就過。
#[tokio::test]
#[serial]
async fn final_stage_restores_candidates_when_nobody_else_can_sign() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let applicant = seed_user_with_role(pool, "ot3-applicant", "EXPERIMENT_STAFF").await;
    // 自己建足前提，不依賴種子資料裡剛好有管理員（CodeRabbit 於 #30 指出）。
    // 原本是斷言「測試庫有管理員」——那不會假綠，但 fixture 一改就會紅在錯的地方，
    // 讀的人會以為是 resolver 壞了。
    let seeded_admin = seed_user_with_role(pool, "ot3-admin", ADMIN_ROLE).await;
    let ot = seed_pending_admin_overtime(pool, applicant, 3).await;

    let admins = all_admins_except(pool, applicant).await;
    assert!(
        admins.contains(&seeded_admin),
        "自建的管理員必須被 all_admins_except 撈到，否則後面的「讓所有人都批過」不成立"
    );

    // 讓「所有」合格終審者都批過前關 → 權威來源回空 → 必須退回全體。
    for admin in &admins {
        record_approval(pool, ot, *admin, "admin_staff").await;
    }

    let eligible = HrService::final_stage_eligible_approvers(pool, &[ot])
        .await
        .expect("eligible");
    // ⚠️ 是「有這個 key 且清單為空」，不是「沒有這個 key」。
    // 兩者在解析器裡都會退回全體，所以行為看不出差別——但意思不同：
    // 空清單＝判準跑過了、答案是沒人；缺席＝這筆不存在。權威來源必須表達前者，
    // 否則「無人可簽」與「查無此單」會走到同一條路。
    // （這個區分原本只寫在註解裡，mutation 打不到那條分支才發現。）
    assert_eq!(
        eligible.get(&ot).map(Vec::len),
        Some(0),
        "所有人都批過之後，權威來源應回**空清單**而非缺席"
    );

    let resolved = resolve_for_overtime(pool, &[ot]).await.expect("resolve");
    let owner = resolved.get(&ot).expect("待終審必須有 pending_owner");
    assert_eq!(
        total(owner),
        admins.len(),
        "沒有其他人可簽時 SoD 放寬（`approve_overtime` 的 `final_stage_has_other_approver` \
         回 false 就不擋），名單必須把管理員放回來，否則會顯示成「沒人能處理」。\n\
         實際 {} 人、應為 {} 人",
        total(owner),
        admins.len()
    );
}

/// 「已等待 N 天」的起點是第一關核准那一刻，不是送出那一刻。
#[tokio::test]
#[serial]
async fn final_stage_waits_from_first_approval_not_submission() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let applicant = seed_user_with_role(pool, "ot4-applicant", "EXPERIMENT_STAFF").await;
    let ot = seed_pending_admin_overtime(pool, applicant, 4).await;

    let (submitted_at, approved_at): (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as("SELECT submitted_at, approved_at FROM overtime_records WHERE id = $1")
        .bind(ot)
        .fetch_one(pool)
        .await
        .expect("read timestamps");

    let resolved = resolve_for_overtime(pool, &[ot]).await.expect("resolve");
    let owner = resolved.get(&ot).expect("待終審必須有 pending_owner");

    assert_eq!(
        owner.since, approved_at,
        "終審關的 since 應為 approved_at（第一關核准寫入的時間戳）"
    );
    assert_ne!(
        owner.since, submitted_at,
        "用 submitted_at 會把第一關的等待時間也算進終審關的「已等待 N 天」。\n\
         本筆刻意讓兩者相差 8 天，所以這個斷言抓得到退化。"
    );
}
