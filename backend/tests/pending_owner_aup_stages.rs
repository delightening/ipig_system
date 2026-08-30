//! AUP 待處理人：三種「由資料決定負責人」的關卡形狀。
//!
//! ## 最重要的一條
//!
//! **委員會審查（`UNDER_REVIEW`）的委員姓名只給 IACUC 行政方，其餘所有人
//! 連 tooltip 都沒有。** 2026-08-27 使用者裁定，判準是 `aup.protocol.change_status`。
//!
//! ⚠️ 這條原本是「一律不列名、只給人數」，後來收斂成現在這樣。判準因此從
//! 「與觀看者無關」變成「與觀看者有關」——**多了一個以前不存在的失敗模式**：
//! 權限判斷寫錯的話，姓名會漏給不該看的人，而漏給誰取決於誰在看，
//! 不會有任何一個固定的觀察點能發現。
//!
//! 所以本檔兩側都釘：有權者**必須**看得到姓名、無權者**必須**連 `pending_owner`
//! 都拿不到（不是拿到空名單——那會讓前端仍然畫出一個空 tooltip）。

mod common;
use chrono::{Duration, Utc};
use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::models::PendingOwnerKind;
use erp_backend::services::pending_owner;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 建一個指定狀態的計畫，回傳 (protocol_id, pi_user_id)。
async fn seed_protocol(pool: &PgPool, status: &str) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let pi = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(pi)
    .bind(format!("po-pi-{}@example.com", &suffix[..8]))
    .bind(format!("計畫主持人-{}", &suffix[..4]))
    .execute(pool)
    .await
    .expect("seed PI");

    let protocol = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by, status, submitted_at) \
         VALUES ($1, $2, $3, $4, $4, $5::protocol_status, NOW())",
    )
    .bind(protocol)
    .bind(format!("PO-{}", &suffix[..8]))
    .bind("待處理人測試計畫")
    .bind(pi)
    .bind(status)
    .execute(pool)
    .await
    .expect("seed protocol");

    (protocol, pi)
}

/// 指派 n 位審查委員給計畫。
async fn assign_reviewers(pool: &PgPool, protocol: Uuid, assigner: Uuid, n: usize) {
    for i in 0..n {
        let suffix = Uuid::new_v4().simple().to_string();
        let reviewer = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
             VALUES ($1, $2, $3, 'x', true, true)",
        )
        .bind(reviewer)
        .bind(format!("po-rev{i}-{}@example.com", &suffix[..8]))
        .bind(format!("審查委員{i}-{}", &suffix[..4]))
        .execute(pool)
        .await
        .expect("seed reviewer");

        sqlx::query(
            "INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(protocol)
        .bind(reviewer)
        .bind(assigner)
        .execute(pool)
        .await
        .expect("assign reviewer");
    }
}

/// 造一個檢視者。`perms` 直接給權限碼——本檔測的是「解析器怎麼用權限」，
/// 不是「哪個角色有那個權限」（後者由授予表與 `pending_owner_candidates.rs` 負責）。
fn viewer(perms: &[&str]) -> CurrentUser {
    CurrentUser {
        id: Uuid::new_v4(),
        email: "viewer@example.com".into(),
        roles: vec![],
        permissions: perms.iter().map(|p| (*p).to_string()).collect(),
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

const PERM_CHANGE_STATUS: &str = "aup.protocol.change_status";

#[tokio::test]
#[serial]
async fn committee_review_names_reviewers_for_iacuc_staff() {
    let app = TestApp::spawn().await;
    let (protocol, pi) = seed_protocol(&app.db_pool, "UNDER_REVIEW").await;
    assign_reviewers(&app.db_pool, protocol, pi, 3).await;

    let owners = pending_owner::resolve_for_protocols(
        &app.db_pool,
        &[protocol],
        &viewer(&[PERM_CHANGE_STATUS]),
    )
    .await
    .expect("resolve");
    let owner = owners.get(&protocol).expect("審查中的計畫必須有待處理人");

    assert_eq!(owner.kind, PendingOwnerKind::Role);
    assert_eq!(owner.role_code.as_deref(), Some("REVIEWER"));
    assert_eq!(
        owner.candidates.len() + owner.overflow as usize,
        3,
        "已指派 3 位委員，總人數要對得上。實際 {:?} + overflow {}",
        owner.candidates,
        owner.overflow
    );
    assert!(
        owner.candidates.iter().all(|n| n.starts_with("審查委員")),
        "列出的應該是委員姓名。實際：{:?}",
        owner.candidates
    );
}

#[tokio::test]
#[serial]
async fn committee_review_gives_no_tooltip_at_all_to_everyone_else() {
    let app = TestApp::spawn().await;
    let (protocol, pi) = seed_protocol(&app.db_pool, "UNDER_REVIEW").await;
    assign_reviewers(&app.db_pool, protocol, pi, 3).await;

    // 獸醫與審查委員持有 `aup.review.identity_view`（審查流程內部彼此可見），
    // 但 2026-08-27 裁定他們在這個 tooltip 上看不到委員名單——刻意拿它當反例：
    // 帶著那個權限也不該過。
    for perms in [
        &[][..],
        &["aup.review.identity_view"][..],
        &["aup.protocol.view_all"][..],
    ] {
        let owners =
            pending_owner::resolve_for_protocols(&app.db_pool, &[protocol], &viewer(perms))
                .await
                .expect("resolve");
        assert!(
            !owners.contains_key(&protocol),
            "委員身分外洩給 perms={:?}。無 `{}` 者必須**完全拿不到** pending_owner\n\
             ——不是拿到空名單（那會讓前端畫出一個空 tooltip），是整個不存在。\n\
             實際：{:?}",
            perms,
            PERM_CHANGE_STATUS,
            owners.get(&protocol)
        );
    }
}

#[tokio::test]
#[serial]
async fn vet_review_names_the_assigned_vet() {
    let app = TestApp::spawn().await;
    let (protocol, assigner) = seed_protocol(&app.db_pool, "VET_REVIEW").await;

    // 指派獸醫。這一關卡在特定一位身上，與委員會不同，要列名。
    let suffix = Uuid::new_v4().simple().to_string();
    let vet = Uuid::new_v4();
    let vet_name = format!("獸醫-{}", &suffix[..6]);
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(vet)
    .bind(format!("po-vet-{}@example.com", &suffix[..8]))
    .bind(&vet_name)
    .execute(&app.db_pool)
    .await
    .expect("seed vet");

    sqlx::query(
        "INSERT INTO vet_review_assignments (id, protocol_id, vet_id, assigned_by, assigned_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(protocol)
    .bind(vet)
    .bind(assigner)
    .execute(&app.db_pool)
    .await
    .expect("assign vet");

    let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol], &viewer(&[]))
        .await
        .expect("resolve");
    let owner = owners
        .get(&protocol)
        .expect("獸醫審查中的計畫必須有待處理人");

    assert_eq!(owner.kind, PendingOwnerKind::Person, "指派給特定一位獸醫");
    assert_eq!(owner.candidates, vec![vet_name]);
    assert!(
        owner.role_code.is_none(),
        "綁人不綁角色時不該顯示角色前綴（2026-08-26 裁定）"
    );
}

#[tokio::test]
#[serial]
async fn revision_required_points_back_at_the_applicant() {
    let app = TestApp::spawn().await;

    // 三個 *_REVISION_REQUIRED 都是把球踢回 PI，只是關卡不同。
    for status in [
        "REVISION_REQUIRED",
        "PRE_REVIEW_REVISION_REQUIRED",
        "VET_REVISION_REQUIRED",
    ] {
        let (protocol, pi) = seed_protocol(&app.db_pool, status).await;
        let pi_name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
            .bind(pi)
            .fetch_one(&app.db_pool)
            .await
            .expect("pi name");

        let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol], &viewer(&[]))
            .await
            .expect("resolve");
        let owner = owners
            .get(&protocol)
            .unwrap_or_else(|| panic!("{status} 必須有待處理人——球在申請人身上也是一種「卡住」"));

        assert_eq!(
            owner.kind,
            PendingOwnerKind::Applicant,
            "{status} 卡在申請人，不是在等審查"
        );
        assert_eq!(owner.candidates, vec![pi_name], "{status} 應指向 PI");
    }
}

/// CodeRabbit #31：`since` 要算「進入目前這一關的時間」，不是原始送審時間。
///
/// 造一份很久以前送審、但**最近**才轉進 `UNDER_REVIEW` 的計畫（模擬繞了一圈：
/// 送審 → 審查 → 退回補件 → 重送 → 再進審查）。若 `since` 錯拿 `submitted_at`，
/// 會顯示「已經等了 30 天」；正確答案是「剛進審查沒多久」。
#[tokio::test]
#[serial]
async fn stage_aging_uses_last_transition_not_original_submission() {
    let app = TestApp::spawn().await;
    let (protocol, pi) = seed_protocol(&app.db_pool, "UNDER_REVIEW").await;

    // seed_protocol 把 submitted_at 設成 NOW()；改成 30 天前，模擬案子拖了很久。
    let long_ago = Utc::now() - Duration::days(30);
    sqlx::query("UPDATE protocols SET submitted_at = $2 WHERE id = $1")
        .bind(protocol)
        .bind(long_ago.date_naive())
        .execute(&app.db_pool)
        .await
        .expect("backdate submitted_at");

    // 最近才轉進 UNDER_REVIEW 這一關。
    let recent_transition = Utc::now() - Duration::hours(2);
    sqlx::query(
        "INSERT INTO protocol_activities (id, protocol_id, activity_type, actor_id, to_value, created_at) \
         VALUES (gen_random_uuid(), $1, 'STATUS_CHANGED'::protocol_activity_type, $2, 'UNDER_REVIEW', $3)",
    )
    .bind(protocol)
    .bind(pi)
    .bind(recent_transition)
    .execute(&app.db_pool)
    .await
    .expect("seed protocol_activities transition");

    let owners = pending_owner::resolve_for_protocols(
        &app.db_pool,
        &[protocol],
        &viewer(&[PERM_CHANGE_STATUS]),
    )
    .await
    .expect("resolve");
    let owner = owners.get(&protocol).expect("UNDER_REVIEW 必須有待處理人");

    let since = owner.since.expect("since 應算得出來");
    let diff_from_transition = (since - recent_transition).num_seconds().abs();
    assert!(
        diff_from_transition < 5,
        "since 應該是最近一次轉進 UNDER_REVIEW 的時間，實際差了 {diff_from_transition} 秒：{since}"
    );
    let diff_from_submission = (since - long_ago).num_seconds().abs();
    assert!(
        diff_from_submission > 29 * 24 * 3600,
        "since 不該算回 30 天前的原始送審時間：{since}"
    );
}

/// 已核准 / 已結案的計畫不在等任何人。
#[tokio::test]
#[serial]
async fn settled_protocols_have_no_pending_owner() {
    let app = TestApp::spawn().await;
    for status in ["APPROVED", "REJECTED", "CLOSED", "DRAFT"] {
        let (protocol, _) = seed_protocol(&app.db_pool, status).await;
        let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol], &viewer(&[]))
            .await
            .expect("resolve");
        assert!(
            !owners.contains_key(&protocol),
            "{status} 不在等任何人，卻算出了待處理人"
        );
    }
}
