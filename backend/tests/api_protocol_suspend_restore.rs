//! 「已暫停」計畫的復原路徑回歸測試。
//!
//! 背景：前端「變更計畫狀態」選單曾提供 SUSPENDED → UNDER_REVIEW（後端從未允許，
//! 見 `models::protocol::ProtocolStatus::can_change_status_to`），且 SUSPENDED →
//! APPROVED / APPROVED_WITH_CONDITIONS 雖在叢集層級白名單合法，卻被 `change_status_tx`
//! 的 entry-guard（寫死「必須從 UNDER_REVIEW 進入」）擋下，形同無法復原。
//!
//! 本檔驗證：
//! 1. SUSPENDED → UNDER_REVIEW 仍應被拒絕（前端選單已移除此選項，此處鎖住後端行為）。
//! 2. SUSPENDED → APPROVED / APPROVED_WITH_CONDITIONS 應成功（沿用原審查意見 + 原電子簽章）。
//! 3. APPROVED_WITH_CONDITIONS → APPROVED 應成功（同一 entry-guard 補的第三個來源，
//!    修復「附條件核准轉正式核准」原本也會被同一守衛擋下的既有缺陷）。
//! 4. 上述復原/轉正皆沿用既有 PIG- 編號，不重新生成（避免動物 iacuc_no / 客戶代碼綁定變孤兒）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{ChangeStatusRequest, ProtocolStatus};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "suspend_restore_regression",
};

/// 建立一個指定狀態的計畫，並補齊「核准前置守衛」所需的歷史資料
/// （正式審查委員指派 + 已發表意見 + 有效電子簽章），模擬「先核准、再暫停/附條件」
/// 後的狀態。iacuc_no 使用既有 PIG- 格式，供測試驗證復原/轉正時不重新生成。
async fn seed_protocol_with_approval_history(
    app: &TestApp,
    status: ProtocolStatus,
) -> (Uuid, String) {
    let admin_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");

    let protocol_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    // per-run 唯一：用完整 8-hex uuid（勿再 [..3] 截斷 → 值域僅 4096、共用測試 DB 累積會撞
    // protocols_iacuc_no_key 而 flake）。保留 PIG-115 語意前綴。
    let iacuc_no = format!("PIG-115-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'Suspend restore regression', $5, $4, $4, NOW(), NOW())"#,
    )
    .bind(protocol_id)
    .bind(format!("PR-{unique}"))
    .bind(&iacuc_no)
    .bind(admin_id)
    .bind(status)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol with approval history");

    let reviewer_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'suspend restore reviewer', true, false)"#,
    )
    .bind(reviewer_id)
    .bind(format!("suspend-restore-rev-{}@test.local", &Uuid::new_v4().to_string()[..6]))
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

    sqlx::query(
        r#"INSERT INTO review_comments (id, protocol_id, reviewer_id, content, created_at, updated_at)
           VALUES ($1, $2, $3, 'reviewed before original approval', NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id)
    .bind(reviewer_id)
    .execute(&app.db_pool)
    .await
    .expect("insert review comment");

    // 原核准時留下的電子簽章：暫停不會作廢它（作廢是 admin 另呼叫
    // /signatures/:id/invalidate 的獨立動作），故復原時仍應視為有效。
    sqlx::query(
        r#"INSERT INTO electronic_signatures
           (id, entity_type, entity_id, signer_id, signature_type, content_hash, signature_data, is_valid, meaning)
           VALUES ($1, 'protocol', $2, $3, 'approve', 'deadbeef', 'sigdata', true, 'APPROVE')"#,
    )
    .bind(Uuid::new_v4())
    .bind(protocol_id.to_string())
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert electronic signature");

    (protocol_id, iacuc_no)
}

fn change_status_req(to_status: ProtocolStatus) -> ChangeStatusRequest {
    ChangeStatusRequest {
        to_status,
        remark: None,
        reviewer_ids: None,
        vet_id: None,
    }
}

/// 已暫停計畫不可直接轉「審查中」——這是使用者回報的原始 bug：前端選單曾提供
/// 此選項，但後端叢集層級白名單從未允許（`protocol.rs` `can_change_status_to`）。
#[tokio::test]
#[serial]
async fn suspended_protocol_cannot_go_to_under_review() {
    let app = TestApp::spawn().await;
    let (protocol_id, _iacuc_no) =
        seed_protocol_with_approval_history(&app, ProtocolStatus::Suspended).await;

    let err = ProtocolService::change_status(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol_id,
        &change_status_req(ProtocolStatus::UnderReview),
    )
    .await
    .expect_err("SUSPENDED 不可直接轉 UNDER_REVIEW");

    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應為 BusinessRule，實得：{err:?}"
    );

    let status: (String,) = sqlx::query_as("SELECT status::text FROM protocols WHERE id = $1")
        .bind(protocol_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(
        status.0,
        ProtocolStatus::Suspended.as_str(),
        "守衛失效時 status 會被轉成 UNDER_REVIEW"
    );
}

/// 已暫停計畫可復原為「已核准」，且沿用既有 PIG- 編號（不重新生成）。
#[tokio::test]
#[serial]
async fn suspended_protocol_can_restore_to_approved_reusing_iacuc_no() {
    let app = TestApp::spawn().await;
    let (protocol_id, original_iacuc_no) =
        seed_protocol_with_approval_history(&app, ProtocolStatus::Suspended).await;

    let updated = ProtocolService::change_status(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol_id,
        &change_status_req(ProtocolStatus::Approved),
    )
    .await
    .expect("SUSPENDED → APPROVED 應成功（沿用原審查意見與電子簽章）");

    assert_eq!(updated.status, ProtocolStatus::Approved);
    assert_eq!(
        updated.iacuc_no,
        Some(original_iacuc_no),
        "復原核准不應重新生成 IACUC 編號，否則既有動物/客戶綁定會變孤兒"
    );
}

/// 已暫停計畫可復原為「附條件核准」，同樣沿用既有 PIG- 編號。
#[tokio::test]
#[serial]
async fn suspended_protocol_can_restore_to_approved_with_conditions_reusing_iacuc_no() {
    let app = TestApp::spawn().await;
    let (protocol_id, original_iacuc_no) =
        seed_protocol_with_approval_history(&app, ProtocolStatus::Suspended).await;

    let updated = ProtocolService::change_status(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol_id,
        &change_status_req(ProtocolStatus::ApprovedWithConditions),
    )
    .await
    .expect("SUSPENDED → APPROVED_WITH_CONDITIONS 應成功");

    assert_eq!(updated.status, ProtocolStatus::ApprovedWithConditions);
    assert_eq!(updated.iacuc_no, Some(original_iacuc_no));
}

/// 附條件核准可轉正為「已核准」，且沿用既有 PIG- 編號（不重新生成）。
///
/// 修復同一個 entry-guard 的既有缺陷：叢集層級白名單本已允許
/// `APPROVED_WITH_CONDITIONS → APPROVED`，但 entry-guard 原本寫死「必須從
/// UNDER_REVIEW 進入」，導致此轉換在單元測試（`can_change_status_to`）層級判定
/// 合法，實際打 API 卻會失敗（PR #921 review 意見發現）。
#[tokio::test]
#[serial]
async fn approved_with_conditions_can_restore_to_approved_reusing_iacuc_no() {
    let app = TestApp::spawn().await;
    let (protocol_id, original_iacuc_no) =
        seed_protocol_with_approval_history(&app, ProtocolStatus::ApprovedWithConditions).await;

    let updated = ProtocolService::change_status(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol_id,
        &change_status_req(ProtocolStatus::Approved),
    )
    .await
    .expect("APPROVED_WITH_CONDITIONS → APPROVED 應成功");

    assert_eq!(updated.status, ProtocolStatus::Approved);
    assert_eq!(
        updated.iacuc_no,
        Some(original_iacuc_no),
        "附條件轉正不應重新生成 IACUC 編號"
    );
}
