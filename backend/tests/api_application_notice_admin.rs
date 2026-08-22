//! 申請須知版本登記服務（admin）整合測試（PR-C）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::CreateApplicationNoticeRequest;
use erp_backend::services::ApplicationNoticeService;
use erp_backend::AppError;

fn actor(id: Uuid) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: "admin@test.local".to_string(),
        roles: vec!["admin".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'admin', true, false)"#,
    )
    .bind(id)
    .bind(format!("an-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

fn req(label: &str) -> CreateApplicationNoticeRequest {
    CreateApplicationNoticeRequest {
        version_label: label.to_string(),
        title: "動物試驗申請須知".to_string(),
        content: "# 須知\n請詳閱。".to_string(),
        effective_from: chrono::NaiveDate::from_ymd_opt(2025, 9, 15).expect("date"),
        attachment_id: None,
    }
}

// ── create → 未生效；重複版本號 → BusinessRule；activate → get_active 回傳 + audit ──
#[tokio::test]
#[serial]
async fn create_dup_activate_flow() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let label = format!("v-{}", &Uuid::new_v4().to_string()[..6]);

    let n = ApplicationNoticeService::create(&app.db_pool, &actor(by), &req(&label))
        .await
        .expect("create");
    assert!(!n.is_active, "新版本預設未生效");

    let dup = ApplicationNoticeService::create(&app.db_pool, &actor(by), &req(&label))
        .await
        .expect_err("重複版本號應拒絕");
    assert!(matches!(dup, AppError::BusinessRule(_)), "實得：{dup:?}");

    let active = ApplicationNoticeService::activate(&app.db_pool, &actor(by), n.id)
        .await
        .expect("activate");
    assert!(active.is_active);

    let activated_audited: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_activity_logs WHERE event_type = 'APPLICATION_NOTICE_ACTIVATED' AND entity_id = $1)",
    )
    .bind(n.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("activate audit check");
    assert!(activated_audited, "activate 應寫 audit");

    let got = ApplicationNoticeService::get_active(&app.db_pool)
        .await
        .expect("get_active")
        .expect("有生效版本");
    assert_eq!(got.id, n.id);

    let list = ApplicationNoticeService::list(&app.db_pool)
        .await
        .expect("list");
    assert!(list.iter().any(|r| r.id == n.id), "列表應含新版本");

    let audited: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_activity_logs WHERE event_type = 'APPLICATION_NOTICE_CREATED' AND entity_id = $1)",
    )
    .bind(n.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("audit check");
    assert!(audited, "create 應寫 audit");
}

async fn seed_protocol_min(app: &TestApp, by: Uuid) -> Uuid {
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '須知內容測試', 'DRAFT'::protocol_status, $4, $4)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("APIG-{unique}"))
    .bind(by)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

// ── update_content：未簽 → 成功 + audit；已被簽署引用 → BusinessRule（受控文件內容不可改）──
#[tokio::test]
#[serial]
async fn update_content_guarded_by_acknowledgement() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let n = ApplicationNoticeService::create(
        &app.db_pool,
        &actor(by),
        &req(&format!("u-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await
    .expect("create");

    // 未簽 → 可更新正文
    let updated = ApplicationNoticeService::update_content(
        &app.db_pool,
        &actor(by),
        n.id,
        "新正文（純文字）",
    )
    .await
    .expect("未簽應可更新");
    assert_eq!(updated.content, "新正文（純文字）");

    let audited: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_activity_logs WHERE event_type = 'APPLICATION_NOTICE_CONTENT_UPDATED' AND entity_id = $1)",
    )
    .bind(n.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("audit check");
    assert!(audited, "update_content 應寫 audit");

    // 插入一筆簽署引用此版本 → 之後不可改內容
    let proto = seed_protocol_min(&app, by).await;
    sqlx::query(
        "INSERT INTO protocol_notice_acknowledgements (protocol_id, notice_id, signer_id) VALUES ($1, $2, $3)",
    )
    .bind(proto)
    .bind(n.id)
    .bind(by)
    .execute(&app.db_pool)
    .await
    .expect("insert ack");

    let err = ApplicationNoticeService::update_content(&app.db_pool, &actor(by), n.id, "再改")
        .await
        .expect_err("已簽應拒改");
    assert!(matches!(err, AppError::BusinessRule(_)), "實得：{err:?}");
}

// ── activate 不存在的版本 → NotFound ──
#[tokio::test]
#[serial]
async fn activate_missing_not_found() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let err = ApplicationNoticeService::activate(&app.db_pool, &actor(by), Uuid::new_v4())
        .await
        .expect_err("不存在應 NotFound");
    assert!(matches!(err, AppError::NotFound(_)), "實得：{err:?}");
}

// ── 切換生效版本：只有一個 active ──
#[tokio::test]
#[serial]
async fn activate_switches_single_active() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let n1 = ApplicationNoticeService::create(
        &app.db_pool,
        &actor(by),
        &req(&format!("a-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await
    .expect("n1");
    let n2 = ApplicationNoticeService::create(
        &app.db_pool,
        &actor(by),
        &req(&format!("b-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await
    .expect("n2");

    ApplicationNoticeService::activate(&app.db_pool, &actor(by), n1.id)
        .await
        .expect("act n1");
    ApplicationNoticeService::activate(&app.db_pool, &actor(by), n2.id)
        .await
        .expect("act n2");

    let got = ApplicationNoticeService::get_active(&app.db_pool)
        .await
        .expect("get")
        .expect("active");
    assert_eq!(got.id, n2.id);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM application_notices WHERE is_active = true")
            .fetch_one(&app.db_pool)
            .await
            .expect("count");
    assert_eq!(count, 1, "至多一個生效");
}
