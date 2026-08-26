//! AUP 待處理人：三種「由資料決定負責人」的關卡形狀。
//!
//! ## 最重要的一條
//!
//! **委員會審查（`UNDER_REVIEW`）一律不列名。** 2026-08-26 使用者裁定：IACUC 審查委員
//! 的身分對所有人一律不揭露，只給人數。
//!
//! 這條是用「狀態」擋而不是用「誰在看」擋——後者會出現 A 看得到 B 看不到的一致性問題。
//! 正因為判準與觀看者無關，它**很容易在重構時被順手改掉**（例如有人把
//! `PendingOwner::anonymous` 換成 `from_candidates` 讓文案「更一致」），而改掉之後
//! 沒有任何既有測試會紅。本檔就是那道紅燈。

mod common;
use common::TestApp;
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

#[tokio::test]
#[serial]
async fn committee_review_never_names_the_reviewers() {
    let app = TestApp::spawn().await;
    let (protocol, pi) = seed_protocol(&app.db_pool, "UNDER_REVIEW").await;
    assign_reviewers(&app.db_pool, protocol, pi, 3).await;

    let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol])
        .await
        .expect("resolve");
    let owner = owners.get(&protocol).expect("審查中的計畫必須有待處理人");

    assert_eq!(
        owner.kind,
        PendingOwnerKind::Anonymous,
        "委員會審查關必須是 anonymous"
    );
    assert!(
        owner.candidates.is_empty(),
        "🔴 委員身分外洩。2026-08-26 使用者裁定：IACUC 審查委員一律不列名，只給人數。\n\
         實際列出：{:?}",
        owner.candidates
    );
    assert_eq!(owner.overflow, 3, "人數要給，且要正確（已指派 3 位）");
    assert_eq!(owner.role_code.as_deref(), Some("REVIEWER"));
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

    let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol])
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

        let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol])
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

/// 已核准 / 已結案的計畫不在等任何人。
#[tokio::test]
#[serial]
async fn settled_protocols_have_no_pending_owner() {
    let app = TestApp::spawn().await;
    for status in ["APPROVED", "REJECTED", "CLOSED", "DRAFT"] {
        let (protocol, _) = seed_protocol(&app.db_pool, status).await;
        let owners = pending_owner::resolve_for_protocols(&app.db_pool, &[protocol])
            .await
            .expect("resolve");
        assert!(
            !owners.contains_key(&protocol),
            "{status} 不在等任何人，卻算出了待處理人"
        );
    }
}
