//! 申請須知 schema + repository 整合測試（PR-A）。
//!
//! - 須知版本：insert / find / list / activate（至多一個生效，partial unique index）。
//! - 簽署紀錄：upsert（一計劃一筆，重簽覆寫）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::models::{NewApplicationNotice, NewNoticeAcknowledgement};
use erp_backend::repositories::application_notice::{
    ApplicationNoticeRepository, NoticeAcknowledgementRepository,
};

async fn seed_user(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'notice user', true, false)"#,
    )
    .bind(id)
    .bind(format!("notice-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_protocol(app: &TestApp, pi: Uuid) -> Uuid {
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '須知測試計畫', 'DRAFT', $4, $4)",
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

fn new_notice(label: &str, by: Uuid) -> NewApplicationNotice {
    NewApplicationNotice {
        version_label: label.to_string(),
        title: "動物試驗申請須知".to_string(),
        content: "# 須知正文\n請詳閱。".to_string(),
        effective_from: chrono::NaiveDate::from_ymd_opt(2025, 9, 15).expect("valid date"),
        attachment_id: None,
        created_by: by,
    }
}

// ── insert / find_by_id / find_active / activate ──
#[tokio::test]
#[serial]
async fn insert_and_activate_notice() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let label = format!("v-{}", &Uuid::new_v4().to_string()[..6]);

    let notice = ApplicationNoticeRepository::insert(&app.db_pool, &new_notice(&label, by))
        .await
        .expect("insert notice");
    assert!(!notice.is_active, "新版本預設未生效");

    let found = ApplicationNoticeRepository::find_by_id(&app.db_pool, notice.id)
        .await
        .expect("find_by_id")
        .expect("notice exists");
    assert_eq!(found.version_label, label);

    ApplicationNoticeRepository::activate(&app.db_pool, notice.id)
        .await
        .expect("activate");
    let active = ApplicationNoticeRepository::find_active(&app.db_pool)
        .await
        .expect("find_active")
        .expect("has active");
    assert_eq!(active.id, notice.id);
    assert!(
        ApplicationNoticeRepository::exists_version_label(&app.db_pool, &label)
            .await
            .expect("exists"),
    );
}

// ── 切換生效版本：舊版自動停用，至多一筆生效 ──
#[tokio::test]
#[serial]
async fn activate_switches_single_active() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let l1 = format!("a-{}", &Uuid::new_v4().to_string()[..6]);
    let l2 = format!("b-{}", &Uuid::new_v4().to_string()[..6]);

    let n1 = ApplicationNoticeRepository::insert(&app.db_pool, &new_notice(&l1, by))
        .await
        .expect("insert n1");
    let n2 = ApplicationNoticeRepository::insert(&app.db_pool, &new_notice(&l2, by))
        .await
        .expect("insert n2");

    ApplicationNoticeRepository::activate(&app.db_pool, n1.id)
        .await
        .expect("activate n1");
    ApplicationNoticeRepository::activate(&app.db_pool, n2.id)
        .await
        .expect("activate n2");

    let active = ApplicationNoticeRepository::find_active(&app.db_pool)
        .await
        .expect("find_active")
        .expect("has active");
    assert_eq!(active.id, n2.id, "最新啟用者為生效版本");

    let active_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM application_notices WHERE is_active = true")
            .fetch_one(&app.db_pool)
            .await
            .expect("count active");
    assert_eq!(active_count, 1, "至多一個生效版本");
}

async fn seed_two_notices(
    app: &TestApp,
    by: Uuid,
) -> (
    erp_backend::models::ApplicationNotice,
    erp_backend::models::ApplicationNotice,
) {
    let n1 = ApplicationNoticeRepository::insert(
        &app.db_pool,
        &new_notice(&format!("x-{}", &Uuid::new_v4().to_string()[..6]), by),
    )
    .await
    .expect("insert n1");
    let n2 = ApplicationNoticeRepository::insert(
        &app.db_pool,
        &new_notice(&format!("y-{}", &Uuid::new_v4().to_string()[..6]), by),
    )
    .await
    .expect("insert n2");
    (n1, n2)
}

// ── 簽署紀錄 upsert：一計劃一筆，重簽覆寫 notice_id ──
#[tokio::test]
#[serial]
async fn acknowledgement_upsert_replaces() {
    let app = TestApp::spawn().await;
    let by = seed_user(&app).await;
    let protocol_id = seed_protocol(&app, by).await;
    let (n1, n2) = seed_two_notices(&app, by).await;

    let ack1 = NoticeAcknowledgementRepository::upsert(
        &app.db_pool,
        &NewNoticeAcknowledgement {
            protocol_id,
            notice_id: n1.id,
            signer_id: by,
            signature_id: None,
            notice_attachment_id: None,
        },
    )
    .await
    .expect("first ack");
    assert_eq!(ack1.notice_id, n1.id);

    let ack2 = NoticeAcknowledgementRepository::upsert(
        &app.db_pool,
        &NewNoticeAcknowledgement {
            protocol_id,
            notice_id: n2.id,
            signer_id: by,
            signature_id: None,
            notice_attachment_id: None,
        },
    )
    .await
    .expect("re-ack");
    assert_eq!(ack2.id, ack1.id, "同計劃同一筆（upsert 非新增）");
    assert_eq!(ack2.notice_id, n2.id, "重簽覆寫為新版本");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM protocol_notice_acknowledgements WHERE protocol_id = $1",
    )
    .bind(protocol_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count");
    assert_eq!(count, 1, "一計劃一筆");

    let found = NoticeAcknowledgementRepository::find_by_protocol(&app.db_pool, protocol_id)
        .await
        .expect("find_by_protocol")
        .expect("ack exists");
    assert_eq!(found.notice_id, n2.id);
}
