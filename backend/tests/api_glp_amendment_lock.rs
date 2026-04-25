//! C2 (GLP) amendment 核准/否決簽章 + update 防呆守衛 integration tests
//!
//! 驗證 21 CFR §11.50/§11.70（非否認性）：amendment 進入終態時必須有電子簽章
//! 記錄；後續 update 即使狀態守衛被繞開也應被簽章 FK 檢查擋下。
//!
//! ## 測試範圍
//!
//! 1. **`classify_minor_inserts_admin_signature_and_blocks_update`**
//!    - classify(Minor) → ADMIN_APPROVED → amendments.approved_signature_id NOT NULL
//!    - 即使把 status 改回 DRAFT（直接 SQL UPDATE 模擬狀態守衛失效），update 仍回 409
//!
//! 2. **`record_decision_unanimous_approve_inserts_signature`**
//!    - 兩位 reviewer 都 APPROVE → APPROVED + approved_signature_id NOT NULL
//!    - signer_id = 最後一位 tipping reviewer
//!
//! 3. **`record_decision_any_reject_inserts_rejection_signature`**
//!    - 任一 reviewer REJECT → REJECTED + rejected_signature_id NOT NULL
//!
//! 4. **`record_decision_revision_no_signature`**
//!    - 任一 reviewer REVISION → REVISION_REQUIRED → 無簽章（非終態）

mod common;

use common::TestApp;
use serial_test::serial;
use sqlx::Row;
use uuid::Uuid;

/// 直接以 raw SQL 建立 protocol（status=APPROVED，PI=admin），
/// 回傳 (protocol_id, admin_user_id)。避免複雜的 protocol service setup。
async fn seed_protocol(app: &TestApp) -> (Uuid, Uuid) {
    let admin_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(
            std::env::var("ADMIN_EMAIL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "admin@ipigsystem.asia".to_string()),
        )
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin id");

    let protocol_id = Uuid::new_v4();
    let suffix = &protocol_id.to_string()[..8];
    sqlx::query(
        r#"
        INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
        VALUES ($1, $2, $3, $4, 'APPROVED', $5, $5)
        "#,
    )
    .bind(protocol_id)
    .bind(format!("P-TEST-{}", suffix))
    .bind(format!("IACUC-{}", suffix))
    .bind("GLP C2 amendment lock test protocol")
    .bind(admin_id.0)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");

    (protocol_id, admin_id.0)
}

/// 直接 INSERT amendment，回傳 amendment_id。狀態為 SUBMITTED 以便 classify。
async fn seed_amendment(app: &TestApp, protocol_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO amendments (
            id, protocol_id, amendment_no, revision_number,
            amendment_type, status, title, description, created_by
        )
        VALUES ($1, $2, $3, 1, 'PENDING', 'SUBMITTED', 'C2 test amendment', 'lock test', $4)
        "#,
    )
    .bind(id)
    .bind(protocol_id)
    .bind(format!("AMD-{}", &id.to_string()[..8]))
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert amendment");
    id
}

async fn fetch_amendment_signatures(
    app: &TestApp,
    id: Uuid,
) -> (Option<Uuid>, Option<Uuid>) {
    let row = sqlx::query(
        "SELECT approved_signature_id, rejected_signature_id FROM amendments WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch amendment sigs");
    (row.try_get(0).ok(), row.try_get(1).ok())
}

async fn assign_reviewer(app: &TestApp, amendment_id: Uuid, reviewer_id: Uuid, assigned_by: Uuid) {
    sqlx::query(
        r#"
        INSERT INTO amendment_review_assignments (amendment_id, reviewer_id, assigned_by)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(amendment_id)
    .bind(reviewer_id)
    .bind(assigned_by)
    .execute(&app.db_pool)
    .await
    .expect("insert review assignment");
}

/// 建立第二個測試使用者（reviewer 用），回傳 user_id。
async fn seed_test_reviewer(app: &TestApp, suffix: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
        VALUES ($1, $2, 'fake-hash-for-test', $3, true, false)
        "#,
    )
    .bind(id)
    .bind(format!("reviewer-{}@test.local", suffix))
    .bind(format!("Reviewer {}", suffix))
    .execute(&app.db_pool)
    .await
    .expect("insert reviewer user");
    id
}

// ============================================================
// Test 1: classify(Minor) → ADMIN_APPROVED 含簽章 + update 防呆
// ============================================================

#[tokio::test]
#[serial]
async fn classify_minor_inserts_admin_signature_and_blocks_update() {
    let app = TestApp::spawn().await;

    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let amendment_id = seed_amendment(&app, protocol_id, admin_id).await;

    // classify Minor → 終態 ADMIN_APPROVED + 簽章
    let req = erp_backend::models::ClassifyAmendmentRequest {
        amendment_type: erp_backend::models::AmendmentType::Minor,
        remark: Some("test minor classification".to_string()),
    };
    let amendment = erp_backend::services::AmendmentService::classify(
        &app.db_pool, amendment_id, &req, admin_id,
    )
    .await
    .expect("classify minor");

    assert_eq!(
        amendment.status,
        erp_backend::models::AmendmentStatus::AdminApproved
    );
    assert!(
        amendment.approved_signature_id.is_some(),
        "ADMIN_APPROVED 應同步寫入 approved_signature_id"
    );

    // 驗證 electronic_signatures 真有此筆
    let sig_id = amendment.approved_signature_id.expect("approved_signature_id missing");
    let sig_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM electronic_signatures WHERE id = $1 AND signer_id = $2 AND signature_type = 'APPROVE'",
    )
    .bind(sig_id)
    .bind(admin_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch sig count");
    assert_eq!(sig_count.0, 1, "簽章記錄應存在且 signer = classifier");

    // 模擬狀態守衛失效：直接 SQL 把 status 改回 DRAFT，但簽章 FK 仍在
    sqlx::query("UPDATE amendments SET status = 'DRAFT' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("force status DRAFT");

    // update 仍應回 409 Conflict（簽章 FK 守衛攔下）
    let update_req = erp_backend::models::UpdateAmendmentRequest {
        title: Some("tampered title".to_string()),
        description: None,
        change_items: None,
        changes_content: None,
    };
    let err = erp_backend::services::AmendmentService::update(
        &app.db_pool, amendment_id, &update_req,
    )
    .await
    .expect_err("update on signed amendment should error");

    match err {
        erp_backend::AppError::Conflict(msg) => assert!(
            msg.contains("簽章"),
            "錯誤訊息應提及簽章；實得: {msg}"
        ),
        other => panic!("應為 Conflict，實得: {other:?}"),
    }
}

// ============================================================
// Test 2: 全體 reviewer APPROVE → APPROVED + signature
// ============================================================

#[tokio::test]
#[serial]
async fn record_decision_unanimous_approve_inserts_signature() {
    let app = TestApp::spawn().await;

    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let amendment_id = seed_amendment(&app, protocol_id, admin_id).await;

    // 設為 UNDER_REVIEW
    sqlx::query("UPDATE amendments SET status = 'UNDER_REVIEW' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("set under review");

    let r1 = seed_test_reviewer(&app, &Uuid::new_v4().to_string()[..6]).await;
    let r2 = seed_test_reviewer(&app, &Uuid::new_v4().to_string()[..6]).await;
    assign_reviewer(&app, amendment_id, r1, admin_id).await;
    assign_reviewer(&app, amendment_id, r2, admin_id).await;

    // r1 APPROVE — 還未到終態（r2 未決）
    let req_approve = erp_backend::models::RecordAmendmentDecisionRequest {
        decision: "APPROVE".into(),
        comment: Some("r1 ok".into()),
    };
    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool, amendment_id, r1, &req_approve,
    )
    .await
    .expect("r1 record_decision");

    let (sig_after_r1, _) = fetch_amendment_signatures(&app, amendment_id).await;
    assert!(sig_after_r1.is_none(), "r1 後尚未終態，不應有簽章");

    // r2 APPROVE — tipping → APPROVED + 簽章 by r2
    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool, amendment_id, r2, &req_approve,
    )
    .await
    .expect("r2 record_decision");

    let (sig_after_r2, _) = fetch_amendment_signatures(&app, amendment_id).await;
    let sig_id = sig_after_r2.expect("r2 後應有 approved_signature_id");

    let sig_row: (Uuid, String) = sqlx::query_as(
        "SELECT signer_id, signature_type FROM electronic_signatures WHERE id = $1",
    )
    .bind(sig_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch sig");
    assert_eq!(sig_row.0, r2, "簽章主體應為最後 tipping reviewer (r2)");
    assert_eq!(sig_row.1, "APPROVE");
}

// ============================================================
// Test 3: 任一 REJECT → REJECTED + signature
// ============================================================

#[tokio::test]
#[serial]
async fn record_decision_any_reject_inserts_rejection_signature() {
    let app = TestApp::spawn().await;

    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let amendment_id = seed_amendment(&app, protocol_id, admin_id).await;
    sqlx::query("UPDATE amendments SET status = 'UNDER_REVIEW' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("set under review");

    let r1 = seed_test_reviewer(&app, &Uuid::new_v4().to_string()[..6]).await;
    let r2 = seed_test_reviewer(&app, &Uuid::new_v4().to_string()[..6]).await;
    assign_reviewer(&app, amendment_id, r1, admin_id).await;
    assign_reviewer(&app, amendment_id, r2, admin_id).await;

    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool, amendment_id, r1,
        &erp_backend::models::RecordAmendmentDecisionRequest {
            decision: "APPROVE".into(),
            comment: None,
        },
    )
    .await
    .expect("r1");

    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool, amendment_id, r2,
        &erp_backend::models::RecordAmendmentDecisionRequest {
            decision: "REJECT".into(),
            comment: Some("not approved".into()),
        },
    )
    .await
    .expect("r2");

    let (approved, rejected) = fetch_amendment_signatures(&app, amendment_id).await;
    assert!(approved.is_none(), "REJECT 不應寫 approved");
    let sig_id = rejected.expect("REJECT 後應寫 rejected_signature_id");

    let sig_row: (Uuid, String) = sqlx::query_as(
        "SELECT signer_id, signature_type FROM electronic_signatures WHERE id = $1",
    )
    .bind(sig_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch sig");
    assert_eq!(sig_row.0, r2);
    assert_eq!(sig_row.1, "REJECT");
}

// ============================================================
// Test 4: REVISION → 非終態，不簽章
// ============================================================

#[tokio::test]
#[serial]
async fn record_decision_revision_no_signature() {
    let app = TestApp::spawn().await;

    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let amendment_id = seed_amendment(&app, protocol_id, admin_id).await;
    sqlx::query("UPDATE amendments SET status = 'UNDER_REVIEW' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("set under review");

    let r1 = seed_test_reviewer(&app, &Uuid::new_v4().to_string()[..6]).await;
    assign_reviewer(&app, amendment_id, r1, admin_id).await;

    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool, amendment_id, r1,
        &erp_backend::models::RecordAmendmentDecisionRequest {
            decision: "REVISION".into(),
            comment: Some("please revise".into()),
        },
    )
    .await
    .expect("r1 revision");

    let (approved, rejected) = fetch_amendment_signatures(&app, amendment_id).await;
    assert!(approved.is_none() && rejected.is_none(), "REVISION 不應寫任一簽章");

    // 狀態應為 REVISION_REQUIRED
    let status: (String,) = sqlx::query_as(
        "SELECT status::text FROM amendments WHERE id = $1",
    )
    .bind(amendment_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch status");
    assert_eq!(status.0, "REVISION_REQUIRED");
}
