//! 申請須知簽署 + 送審門檻 + SD 編輯權整合測試（PR-B）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::NewApplicationNotice;
use erp_backend::repositories::application_notice::{
    ApplicationNoticeRepository, NoticeAcknowledgementRepository,
};
use erp_backend::services::{access, ProtocolService};
use erp_backend::AppError;

fn user_actor(id: Uuid) -> ActorContext {
    ActorContext::User(current_user(id))
}

fn current_user(id: Uuid) -> CurrentUser {
    CurrentUser {
        id,
        email: "signer@test.local".to_string(),
        roles: vec!["EXPERIMENT_STAFF".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

/// R75-P4：取得須知簽署證明（成功路徑用）。授權契約本身由
/// `acknowledge_notice_forbidden_for_outsider` 直接驗 `authorize`。
async fn notice_scope(app: &TestApp, uid: Uuid, pid: Uuid) -> access::Scoped<access::NoticeSign> {
    access::Scoped::<access::NoticeSign>::authorize(&app.db_pool, &current_user(uid), pid)
        .await
        .expect("notice authorize")
}

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', 'signer', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("sig-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建 DRAFT 計畫，pi_user_id / study_director_user_id 可選。
async fn seed_protocol(app: &TestApp, pi: Uuid, sd: Option<Uuid>, status: &str) -> Uuid {
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, study_director_user_id, created_by) \
         VALUES ($1, $2, $3, '須知簽署測試', $4::protocol_status, $5, $6, $5)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("APIG-{unique}"))
    .bind(status)
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// 建並啟用一個須知版本（activate 會停用其他，避免跨測試殘留）。
async fn seed_active_notice(app: &TestApp, by: Uuid) -> Uuid {
    let n = ApplicationNoticeRepository::insert(
        &app.db_pool,
        &NewApplicationNotice {
            version_label: format!("v-{}", &Uuid::new_v4().to_string()[..6]),
            title: "動物試驗申請須知".to_string(),
            content: "# 須知\n請詳閱。".to_string(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2025, 9, 15).expect("date"),
            attachment_id: None,
            created_by: by,
        },
    )
    .await
    .expect("insert notice");
    ApplicationNoticeRepository::activate(&app.db_pool, n.id)
        .await
        .expect("activate");
    n.id
}

async fn deactivate_all_notices(app: &TestApp) {
    sqlx::query("UPDATE application_notices SET is_active = false WHERE is_active = true")
        .execute(&app.db_pool)
        .await
        .expect("deactivate");
}

// ── PI 簽署 DRAFT → 成功，建簽章 + ack ──
#[tokio::test]
#[serial]
async fn acknowledge_notice_pi_ok() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    let notice_id = seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, None, "DRAFT").await;

    let scope = notice_scope(&app, pi, protocol_id).await;
    let ack =
        ProtocolService::acknowledge_notice(&app.db_pool, &user_actor(pi), scope, "<svg/>", None)
            .await
            .expect("PI 簽署應成功");
    assert_eq!(ack.notice_id, notice_id);
    assert!(ack.signature_id.is_some(), "應有電子簽章");

    let sig_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM electronic_signatures WHERE id = $1 AND meaning = 'ACKNOWLEDGE')",
    )
    .bind(ack.signature_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("check sig");
    assert!(sig_exists, "簽章 meaning 應為 ACKNOWLEDGE");

    let found = NoticeAcknowledgementRepository::find_by_protocol(&app.db_pool, protocol_id)
        .await
        .expect("find")
        .expect("ack exists");
    assert_eq!(found.signer_id, pi);
}

// ── SD 簽署（非 PI）→ 成功 ──
#[tokio::test]
#[serial]
async fn acknowledge_notice_sd_ok() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    let sd = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, Some(sd), "DRAFT").await;

    let scope = notice_scope(&app, sd, protocol_id).await;
    let ack =
        ProtocolService::acknowledge_notice(&app.db_pool, &user_actor(sd), scope, "<svg/>", None)
            .await
            .expect("SD 簽署應成功");
    assert_eq!(ack.signer_id, sd);
}

// ── 非 PI/SD → Forbidden ──
#[tokio::test]
#[serial]
async fn acknowledge_notice_forbidden_for_outsider() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    let outsider = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, None, "DRAFT").await;

    // R75-P4：授權契約移至 Scoped<NoticeSign>::authorize —— 外人無法取得簽署證明。
    // Scoped 未實作 Debug（同既有 marker 測試），故以 matches! 斷言而非 expect_err。
    let result = access::Scoped::<access::NoticeSign>::authorize(
        &app.db_pool,
        &current_user(outsider),
        protocol_id,
    )
    .await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "非 PI/SD 應 Forbidden，實得：{:?}",
        result.err()
    );
}

// ── 非草稿 → BusinessRule ──
#[tokio::test]
#[serial]
async fn acknowledge_notice_non_draft_blocked() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, None, "APPROVED").await;

    // PI 可取得簽署證明（status 非 authorize 條件）；非草稿由 service 擋下。
    let scope = notice_scope(&app, pi, protocol_id).await;
    let err =
        ProtocolService::acknowledge_notice(&app.db_pool, &user_actor(pi), scope, "<svg/>", None)
            .await
            .expect_err("非草稿應拒絕");
    assert!(matches!(err, AppError::BusinessRule(_)), "實得：{err:?}");
}

// ── 送審門檻：有生效須知但未簽 → BadRequest ──
#[tokio::test]
#[serial]
async fn submit_blocked_without_ack() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, None, "DRAFT").await;

    let err = ProtocolService::submit(&app.db_pool, &user_actor(pi), protocol_id)
        .await
        .expect_err("未簽須知應擋送審");
    assert!(
        matches!(err, AppError::BadRequest(_)),
        "應 BadRequest（須知門檻），實得：{err:?}"
    );
}

// ── 送審門檻：已簽 → 通過須知門檻（後續才因內容驗證失敗，非 BadRequest）──
#[tokio::test]
#[serial]
async fn submit_passes_notice_gate_after_ack() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    let protocol_id = seed_protocol(&app, pi, None, "DRAFT").await;

    let scope = notice_scope(&app, pi, protocol_id).await;
    ProtocolService::acknowledge_notice(&app.db_pool, &user_actor(pi), scope, "<svg/>", None)
        .await
        .expect("簽署");

    // 已簽 → 須知門檻通過；working_content 為 NULL → 改在內容驗證階段失敗（Validation，非 BadRequest）。
    let err = ProtocolService::submit(&app.db_pool, &user_actor(pi), protocol_id)
        .await
        .expect_err("內容空應於後續驗證失敗");
    assert!(
        matches!(err, AppError::Validation(_)),
        "已簽後應通過須知門檻、改報內容 Validation，實得：{err:?}"
    );
}

// ── 補件重送（*_REVISION_REQUIRED）即使未簽當前生效須知，亦不被須知門檻阻擋（Bug 1 迴歸）──
// 須知為「初次送審」前的一次性簽署；補件狀態無法簽（acknowledge 限 DRAFT），
// 故門檻只在 DRAFT 檢查，否則造成「要簽卻無法簽」死鎖。
#[tokio::test]
#[serial]
async fn submit_revision_resubmit_not_blocked_by_notice_gate() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    seed_active_notice(&app, pi).await;
    // 補件狀態 + 無 ack：若門檻仍對補件檢查 → BadRequest；正確行為應略過門檻、改報內容 Validation。
    let protocol_id = seed_protocol(&app, pi, None, "PRE_REVIEW_REVISION_REQUIRED").await;

    let err = ProtocolService::submit(&app.db_pool, &user_actor(pi), protocol_id)
        .await
        .expect_err("內容空應於內容驗證失敗");
    assert!(
        matches!(err, AppError::Validation(_)),
        "補件重送不應被須知門檻擋（應略過門檻、改報內容 Validation），實得：{err:?}"
    );
}

// ── 無生效須知時，送審不受須知門檻阻擋（直接進內容驗證）──
#[tokio::test]
#[serial]
async fn submit_not_blocked_when_no_active_notice() {
    let app = TestApp::spawn().await;
    deactivate_all_notices(&app).await;
    let pi = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi, None, "DRAFT").await;

    let err = ProtocolService::submit(&app.db_pool, &user_actor(pi), protocol_id)
        .await
        .expect_err("內容空應於內容驗證失敗");
    assert!(
        matches!(err, AppError::Validation(_)),
        "無生效須知時不應被須知門檻擋，實得：{err:?}"
    );
}

// ── SD 對自己負責的計劃取得編輯權（§5.4）──
#[tokio::test]
#[serial]
async fn study_director_can_edit_own_protocol() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    let sd = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, pi, Some(sd), "DRAFT").await;

    let can = access::can_edit_protocol(&app.db_pool, &current_user(sd), protocol_id)
        .await
        .expect("can_edit");
    assert!(can, "SD 應可編輯自己負責的計劃");

    let outsider = seed_user(&app).await;
    let cannot = access::can_edit_protocol(&app.db_pool, &current_user(outsider), protocol_id)
        .await
        .expect("can_edit");
    assert!(!cannot, "無關使用者不應可編輯");
}
