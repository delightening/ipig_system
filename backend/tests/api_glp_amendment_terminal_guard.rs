//! C2-extra (GLP §11.10(e))：Amendment terminal status guard
//!
//! 驗證 record_decision 的終態守衛 — APPROVED / REJECTED / ADMIN_APPROVED 後
//! 不可再記錄審查決定，避免：
//! 1. status 翻轉（已核准 → 強制改決定 → 變 REJECTED）
//! 2. （PR #205 合併後）approved/rejected_signature_id 被覆寫
//! 3. 違反 21 CFR §11.10(e) 「audit trail 不得遮蔽先前記錄」
//!
//! 關聯 review：CodeRabbit PR #205 R7（Outside-diff Major）
//!
//! 因 main 上 amendments 表尚未有 signature FK 欄位，此 PR 僅驗證 status
//! 防覆寫即可；PR #205 合併後 signature 防覆寫會自動繼承本守衛保護。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

/// 建立 protocol + admin user，回傳 (protocol_id, admin_id)。
async fn seed_protocol(app: &TestApp) -> (Uuid, Uuid) {
    let admin_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch admin");

    let protocol_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"
        INSERT INTO protocols (
            id, protocol_no, iacuc_no, title, status,
            pi_user_id, created_by, created_at, updated_at
        ) VALUES (
            $1, $2, $3, 'C2 Extra Guard Test', 'APPROVED'::protocol_status,
            $4, $4, NOW(), NOW()
        )
        "#,
    )
    .bind(protocol_id)
    .bind(format!("PR-{unique}"))
    .bind(format!("IACUC-{unique}"))
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");

    (protocol_id, admin_id)
}

/// 建立一個 UNDER_REVIEW 狀態 amendment，指派 1 位 reviewer。
async fn seed_amendment_with_reviewer(
    app: &TestApp,
    protocol_id: Uuid,
    admin_id: Uuid,
) -> (Uuid, Uuid) {
    let amendment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO amendments (
            id, protocol_id, amendment_no, revision_number, amendment_type,
            status, title, description, change_items, changes_content,
            created_by, created_at, updated_at
        ) VALUES (
            $1, $2, 'AM-001', 1, 'MAJOR'::amendment_type,
            'UNDER_REVIEW'::amendment_status, 'guard test', 'desc', '{}'::varchar[], '{}'::jsonb,
            $3, NOW(), NOW()
        )
        "#,
    )
    .bind(amendment_id)
    .bind(protocol_id)
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert amendment");

    let reviewer_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
        VALUES ($1, $2, 'fake', 'guard reviewer', true, false)
        "#,
    )
    .bind(reviewer_id)
    .bind(format!("guard-reviewer-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert reviewer");

    sqlx::query(
        r#"
        INSERT INTO amendment_review_assignments (id, amendment_id, reviewer_id, assigned_by, assigned_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(amendment_id)
    .bind(reviewer_id)
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("assign reviewer");

    (amendment_id, reviewer_id)
}

#[tokio::test]
#[serial]
async fn record_decision_rejected_after_approved_returns_conflict() {
    let app = TestApp::spawn().await;
    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let (amendment_id, reviewer_id) =
        seed_amendment_with_reviewer(&app, protocol_id, admin_id).await;

    // 模擬 amendment 已被前一輪審查 APPROVED
    sqlx::query("UPDATE amendments SET status = 'APPROVED' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("force APPROVED");

    let req = erp_backend::models::RecordAmendmentDecisionRequest {
        decision: "REJECT".into(),
        comment: Some("試圖翻轉終態".into()),
    };
    let err = erp_backend::services::AmendmentService::record_decision(
        &app.db_pool,
        amendment_id,
        reviewer_id,
        &req,
    )
    .await
    .expect_err("APPROVED 終態下 record_decision 應拒絕");

    match err {
        erp_backend::AppError::Conflict(msg) => {
            assert!(msg.contains("終態"), "錯誤訊息應提及終態；實得: {msg}");
        }
        other => panic!("應為 Conflict，實得: {other:?}"),
    }

    // 確認 status 沒被翻轉
    let status: (String,) = sqlx::query_as("SELECT status::text FROM amendments WHERE id = $1")
        .bind(amendment_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(status.0, "APPROVED", "守衛失效時 status 會被翻轉");
}

#[tokio::test]
#[serial]
async fn record_decision_rejected_after_rejected_returns_conflict() {
    let app = TestApp::spawn().await;
    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let (amendment_id, reviewer_id) =
        seed_amendment_with_reviewer(&app, protocol_id, admin_id).await;

    sqlx::query("UPDATE amendments SET status = 'REJECTED' WHERE id = $1")
        .bind(amendment_id)
        .execute(&app.db_pool)
        .await
        .expect("force REJECTED");

    let req = erp_backend::models::RecordAmendmentDecisionRequest {
        decision: "APPROVE".into(),
        comment: Some("試圖把 REJECTED 翻成 APPROVED".into()),
    };
    let err = erp_backend::services::AmendmentService::record_decision(
        &app.db_pool,
        amendment_id,
        reviewer_id,
        &req,
    )
    .await
    .expect_err("REJECTED 終態下 record_decision 應拒絕");

    assert!(matches!(err, erp_backend::AppError::Conflict(_)));
}

#[tokio::test]
#[serial]
async fn record_decision_under_review_succeeds() {
    let app = TestApp::spawn().await;
    let (protocol_id, admin_id) = seed_protocol(&app).await;
    let (amendment_id, reviewer_id) =
        seed_amendment_with_reviewer(&app, protocol_id, admin_id).await;

    // status = UNDER_REVIEW（seed 預設）→ 可正常記錄
    let req = erp_backend::models::RecordAmendmentDecisionRequest {
        decision: "APPROVE".into(),
        comment: Some("normal vote".into()),
    };
    erp_backend::services::AmendmentService::record_decision(
        &app.db_pool,
        amendment_id,
        reviewer_id,
        &req,
    )
    .await
    .expect("UNDER_REVIEW 時 record_decision 應成功");
}
