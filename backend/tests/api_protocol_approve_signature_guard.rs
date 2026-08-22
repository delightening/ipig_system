//! R71-7（21 CFR §11 / GLP）：Protocol「已核准必有簽章」不變式守衛
//!
//! 驗證 `ProtocolService::change_status` 進入 APPROVED 前，必須已存在
//! 有效的 protocol 電子簽章（IACUC 主席/秘書經 `/signatures/protocol/:id`
//! 簽核）。否則拒絕（做法 A：先簽再核准）。
//!
//! 守衛位置：`services/protocol/status.rs` APPROVED / APPROVED_WITH_CONDITIONS 分支。
//!
//! 防護目標：避免「核准狀態」與「電子簽章」脫鉤 —— 已核准計畫若無對應簽章，
//! 形同 audit trail 缺非否認性（non-repudiation）依據，違反 21 CFR §11.10。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{ChangeStatusRequest, ProtocolStatus};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "r71_7_signature_guard_test",
};

/// 建立一個 UNDER_REVIEW 計畫，並指派 1 位已發表意見的正式審查委員，
/// 使其通過「審查委員均已表意」的前置守衛，僅剩簽章守衛攔截。
/// 回傳 (protocol_id, reviewer_id)。
async fn seed_under_review_protocol(app: &TestApp) -> (Uuid, Uuid) {
    let admin_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");

    let protocol_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'R71-7 signature guard', 'UNDER_REVIEW'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(protocol_id)
    .bind(format!("PR-{unique}"))
    .bind(format!("IACUC-{unique}"))
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert under-review protocol");

    // 正式審查委員
    let reviewer_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'r71-7 reviewer', true, false)"#,
    )
    .bind(reviewer_id)
    .bind(format!("r71-7-rev-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert reviewer");

    sqlx::query(
        r#"INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at, is_primary_reviewer)
           VALUES ($1, $2, $3, $4, NOW(), true)"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id)
    .bind(reviewer_id)
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("assign primary reviewer");

    // 審查委員已發表頂層意見（parent_comment_id NULL）→ 通過「均已表意」守衛
    sqlx::query(
        r#"INSERT INTO review_comments (id, protocol_id, reviewer_id, content, created_at, updated_at)
           VALUES ($1, $2, $3, 'reviewed', NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id)
    .bind(reviewer_id)
    .execute(&app.db_pool)
    .await
    .expect("insert review comment");

    (protocol_id, reviewer_id)
}

/// 寫入一筆有效的 protocol 電子簽章（模擬 IACUC 主席/秘書簽核完成）。
async fn insert_valid_protocol_signature(app: &TestApp, protocol_id: Uuid, signer_id: Uuid) {
    sqlx::query(
        r#"INSERT INTO electronic_signatures
           (id, entity_type, entity_id, signer_id, signature_type, content_hash, signature_data, is_valid, meaning)
           VALUES ($1, 'protocol', $2, $3, 'approve', 'deadbeef', 'sigdata', true, 'APPROVE')"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id.to_string())
    .bind(signer_id)
    .execute(&app.db_pool)
    .await
    .expect("insert electronic signature");
}

fn approve_req() -> ChangeStatusRequest {
    ChangeStatusRequest {
        to_status: ProtocolStatus::Approved,
        remark: None,
        reviewer_ids: None,
        vet_id: None,
    }
}

/// 無簽章時核准應被拒絕，且 status 不得翻成 APPROVED。
#[tokio::test]
#[serial]
async fn approve_without_signature_is_rejected() {
    let app = TestApp::spawn().await;
    let (protocol_id, _reviewer_id) = seed_under_review_protocol(&app).await;

    let err =
        ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &approve_req())
            .await
            .expect_err("無有效簽章時核准應被拒絕");

    match err {
        AppError::BusinessRule(msg) => {
            assert!(msg.contains("簽章"), "錯誤訊息應提及簽章；實得：{msg}");
        }
        other => panic!("應為 BusinessRule，實得：{other:?}"),
    }

    // 守衛失效時 status 會被翻成 APPROVED
    let status: (String,) = sqlx::query_as("SELECT status::text FROM protocols WHERE id = $1")
        .bind(protocol_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(
        status.0, "UNDER_REVIEW",
        "守衛失效時 status 會被翻成 APPROVED"
    );
}

/// 已有有效簽章時核准應成功。
#[tokio::test]
#[serial]
async fn approve_with_valid_signature_succeeds() {
    let app = TestApp::spawn().await;
    let (protocol_id, reviewer_id) = seed_under_review_protocol(&app).await;
    insert_valid_protocol_signature(&app, protocol_id, reviewer_id).await;

    let updated =
        ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &approve_req())
            .await
            .expect("已有有效簽章時核准應成功");

    assert_eq!(updated.status, ProtocolStatus::Approved);
}

/// 簽章存在但已失效（is_valid = false）時，核准仍應被拒絕。
#[tokio::test]
#[serial]
async fn approve_with_invalidated_signature_is_rejected() {
    let app = TestApp::spawn().await;
    let (protocol_id, reviewer_id) = seed_under_review_protocol(&app).await;

    // 失效簽章（is_valid = false）不應滿足守衛
    sqlx::query(
        r#"INSERT INTO electronic_signatures
           (id, entity_type, entity_id, signer_id, signature_type, content_hash, signature_data, is_valid, meaning)
           VALUES ($1, 'protocol', $2, $3, 'approve', 'deadbeef', 'sigdata', false, 'APPROVE')"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id.to_string())
    .bind(reviewer_id)
    .execute(&app.db_pool)
    .await
    .expect("insert invalidated signature");

    let err =
        ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &approve_req())
            .await
            .expect_err("僅有失效簽章時核准應被拒絕");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應為 BusinessRule，實得：{err:?}"
    );
}
