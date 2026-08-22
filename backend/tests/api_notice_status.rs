//! 須知簽署狀態查詢（PR-D）整合測試。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::NewApplicationNotice;
use erp_backend::repositories::application_notice::ApplicationNoticeRepository;
use erp_backend::services::access::{NoticeSign, ProtocolId, Scoped};
use erp_backend::services::ProtocolService;

fn user_actor(id: Uuid) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: "pi@test.local".to_string(),
        roles: vec!["PI".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 構造已授權的 `Scoped<ProtocolId>`：viewer 持 `aup.protocol.view_all` 權限，
/// 故走真實 `authorize` 必過、無需額外 DB 角色 setup（R75-P4 Phase 2 pilot）。
async fn scope(app: &TestApp, pid: Uuid) -> Scoped<ProtocolId> {
    let viewer = CurrentUser {
        id: Uuid::new_v4(),
        email: "viewer@test.local".to_string(),
        roles: vec![],
        permissions: vec!["aup.protocol.view_all".to_string()],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    };
    Scoped::<ProtocolId>::authorize(&app.db_pool, &viewer, pid)
        .await
        .expect("authorize scope")
}

/// R75-P4：取得須知簽署證明（signer 為計畫 pi_user_id → `can_sign_notice` 過）。
async fn notice_scope(app: &TestApp, uid: Uuid, pid: Uuid) -> Scoped<NoticeSign> {
    let signer = CurrentUser {
        id: uid,
        email: "pi@test.local".to_string(),
        roles: vec![],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    };
    Scoped::<NoticeSign>::authorize(&app.db_pool, &signer, pid)
        .await
        .expect("notice authorize")
}

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("ns-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_draft_protocol(app: &TestApp, pi: Uuid) -> Uuid {
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '狀態測試', 'DRAFT'::protocol_status, $4, $4)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("APIG-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

async fn seed_active_notice(app: &TestApp, by: Uuid) -> Uuid {
    let n = ApplicationNoticeRepository::insert(
        &app.db_pool,
        &NewApplicationNotice {
            version_label: format!("v-{}", &Uuid::new_v4().to_string()[..6]),
            title: "申請須知".to_string(),
            content: "正文".to_string(),
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

async fn deactivate_all(app: &TestApp) {
    sqlx::query("UPDATE application_notices SET is_active = false WHERE is_active = true")
        .execute(&app.db_pool)
        .await
        .expect("deactivate");
}

// ── 無生效須知 → active_notice None / acknowledged false ──
#[tokio::test]
#[serial]
async fn status_no_active_notice() {
    let app = TestApp::spawn().await;
    deactivate_all(&app).await;
    let pi = seed_user(&app).await;
    let pid = seed_draft_protocol(&app, pi).await;

    let s = ProtocolService::get_notice_status(&app.db_pool, scope(&app, pid).await)
        .await
        .expect("status");
    assert!(s.active_notice.is_none());
    assert!(!s.acknowledged);
}

// ── 有生效須知未簽 → acknowledged false；簽後 → true + acknowledged_at ──
#[tokio::test]
#[serial]
async fn status_reflects_acknowledgement() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app).await;
    let notice_id = seed_active_notice(&app, pi).await;
    let pid = seed_draft_protocol(&app, pi).await;

    let before = ProtocolService::get_notice_status(&app.db_pool, scope(&app, pid).await)
        .await
        .expect("status");
    assert_eq!(before.active_notice.as_ref().map(|n| n.id), Some(notice_id));
    assert!(!before.acknowledged, "未簽應為 false");

    let nscope = notice_scope(&app, pi, pid).await;
    ProtocolService::acknowledge_notice(&app.db_pool, &user_actor(pi), nscope, "<svg/>", None)
        .await
        .expect("sign");

    let after = ProtocolService::get_notice_status(&app.db_pool, scope(&app, pid).await)
        .await
        .expect("status");
    assert!(after.acknowledged, "簽後應為 true");
    assert!(after.acknowledged_at.is_some());
}
