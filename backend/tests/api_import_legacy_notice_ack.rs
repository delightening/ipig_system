//! 匯入舊計劃承接歷史須知簽署（PR-E）整合測試。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{ImportApprovedProtocolRequest, NewApplicationNotice};
use erp_backend::repositories::application_notice::{
    ApplicationNoticeRepository, NoticeAcknowledgementRepository,
};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "import_notice_test",
};

async fn seed_pi(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'imp pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("imp-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert pi");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI");
    id
}

async fn seed_sd(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', 'imp sd', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("imp-sd-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert sd");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF'")
        .bind(id)
        .execute(&app.db_pool)
        .await
        .expect("assign EXPERIMENT_STAFF");
    id
}

async fn seed_notice(app: &TestApp, by: Uuid, label: &str) {
    ApplicationNoticeRepository::insert(
        &app.db_pool,
        &NewApplicationNotice {
            version_label: label.to_string(),
            title: "動物試驗申請須知".to_string(),
            content: "（紙本版本，詳見附件 PDF）".to_string(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2024, 11, 26).expect("date"),
            attachment_id: None,
            created_by: by,
        },
    )
    .await
    .expect("seed notice");
}

fn import_req(
    pi: Uuid,
    sd: Uuid,
    iacuc: &str,
    notice_label: Option<&str>,
    ack_date: Option<chrono::NaiveDate>,
) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "須知承接匯入測試".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no: iacuc.to_string(),
        application_no: None,
        working_content: None,
        start_date: None,
        end_date: None,
        submitted_at: None,
        pre_review_at: None,
        vet_review_at: None,
        committee_first_review_at: None,
        revision_required_at: None,
        committee_second_review_at: None,
        approved_at: None,
        remark: None,
        notice_version_label: notice_label.map(str::to_string),
        notice_attachment_id: None,
        notice_acknowledged_at: ack_date,
        source_form_version: None,
    }
}

// ── 匯入帶須知版次 → 記歷史 ack（signature_id NULL + 歷史日期）──
#[tokio::test]
#[serial]
async fn import_with_notice_records_legacy_ack() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let label = format!("v-{}", &Uuid::new_v4().to_string()[..6]);
    seed_notice(&app, pi, &label).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let ack_date = chrono::NaiveDate::from_ymd_opt(2024, 11, 26).expect("date");

    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(
            pi,
            sd,
            &format!("PIG-{unique}"),
            Some(&label),
            Some(ack_date),
        ),
        pi,
    )
    .await
    .expect("匯入應成功");

    let ack = NoticeAcknowledgementRepository::find_by_protocol(&app.db_pool, p.id)
        .await
        .expect("find")
        .expect("應有歷史 ack");
    assert!(ack.signature_id.is_none(), "legacy ack 無電子簽章");
    assert_eq!(ack.signer_id, pi);
    assert_eq!(
        ack.acknowledged_at.date_naive(),
        ack_date,
        "acknowledged_at 應為歷史日期"
    );
}

// ── 匯入帶不存在的須知版次 → Validation ──
#[tokio::test]
#[serial]
async fn import_with_unknown_notice_label_rejected() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let err = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}"), Some("不存在版次"), None),
        pi,
    )
    .await
    .expect_err("未知版次應拒絕");
    assert!(matches!(err, AppError::Validation(_)), "實得：{err:?}");
}

// ── 匯入不帶須知欄位 → 無 ack（既有行為不變）──
#[tokio::test]
#[serial]
async fn import_without_notice_no_ack() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}"), None, None),
        pi,
    )
    .await
    .expect("匯入應成功");

    let ack = NoticeAcknowledgementRepository::find_by_protocol(&app.db_pool, p.id)
        .await
        .expect("find");
    assert!(ack.is_none(), "未帶須知欄位不應建 ack");
}
