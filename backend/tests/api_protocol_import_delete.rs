//! 刪除誤匯的匯入計劃（R64-5c）整合測試。
//!
//! - 僅匯入計劃（imported_at 非 NULL）可刪；硬刪 + scaffold 級聯。
//! - 下游守衛：有 amendment 拒絕。
//! - 非匯入計劃 / Anonymous 拒絕。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::ImportApprovedProtocolRequest;
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "import_delete_test",
};

async fn seed_pi(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'del pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("del-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
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
           VALUES ($1, $2, 'fake', 'del sd', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("del-sd-{}@test.local", &Uuid::new_v4().to_string()[..6]))
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

fn import_req(pi: Uuid, sd: Uuid, iacuc_no: &str) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "刪除測試計劃".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no: iacuc_no.to_string(),
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
        notice_version_label: None,
        notice_attachment_id: None,
        notice_acknowledged_at: None,
        source_form_version: None,
    }
}

async fn seed_import(app: &TestApp) -> (Uuid, Uuid) {
    let pi = seed_pi(app).await;
    let sd = seed_sd(app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}")),
        pi,
    )
    .await
    .expect("import");
    (p.id, pi)
}

async fn exists_protocol(app: &TestApp, id: Uuid) -> bool {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)")
        .bind(id)
        .fetch_one(&app.db_pool)
        .await
        .expect("check protocol")
}

// ── 新鮮匯入計劃可刪 + scaffold 級聯 ──
#[tokio::test]
#[serial]
async fn delete_fresh_imported_ok() {
    let app = TestApp::spawn().await;
    let (id, _pi) = seed_import(&app).await;

    ProtocolService::delete_imported_protocol(&app.db_pool, &SYSTEM_TEST, id)
        .await
        .expect("刪除新鮮匯入計劃應成功");

    assert!(!exists_protocol(&app, id).await, "計劃應已硬刪");
    let up: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_protocols WHERE protocol_id = $1)")
            .bind(id)
            .fetch_one(&app.db_pool)
            .await
            .expect("check user_protocols");
    assert!(!up, "user_protocols 應 FK CASCADE 連帶刪");
}

// ── 有 amendment → 拒絕（CASCADE 會靜默刪，守衛明確擋）──
#[tokio::test]
#[serial]
async fn delete_rejected_with_amendment() {
    let app = TestApp::spawn().await;
    let (id, pi) = seed_import(&app).await;
    sqlx::query(
        "INSERT INTO amendments (id, protocol_id, amendment_no, title, created_by) VALUES ($1, $2, $3, '測試變更', $4)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind("PIG-X-R01")
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert amendment");

    let err = ProtocolService::delete_imported_protocol(&app.db_pool, &SYSTEM_TEST, id)
        .await
        .expect_err("有 amendment 應拒絕刪除");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
    assert!(exists_protocol(&app, id).await, "拒絕後計劃應仍存在");
}

// ── 非匯入計劃（imported_at NULL）→ 拒絕 ──
#[tokio::test]
#[serial]
async fn delete_rejected_non_imported() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) VALUES ($1, $2, $3, '一般計劃', 'APPROVED', $4, $4)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("PIG-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert normal protocol");

    let err = ProtocolService::delete_imported_protocol(&app.db_pool, &SYSTEM_TEST, id)
        .await
        .expect_err("非匯入計劃應拒絕刪除");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── 非匯入但「已駁回」→ 可硬刪（R64-5c 擴充：匯入 / 已駁回 / 草稿）──
#[tokio::test]
#[serial]
async fn delete_rejected_protocol_succeeds() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) VALUES ($1, $2, $3, '已駁回計劃', 'REJECTED', $4, $4)",
    )
    .bind(id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("PIG-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert rejected protocol");

    ProtocolService::delete_imported_protocol(&app.db_pool, &SYSTEM_TEST, id)
        .await
        .expect("非匯入但已駁回計劃應可硬刪");

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)")
        .bind(id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query exists");
    assert!(!exists, "刪除後計劃不應存在");
}

// ── Anonymous → Forbidden ──
#[tokio::test]
#[serial]
async fn delete_rejects_anonymous() {
    let app = TestApp::spawn().await;
    let (id, _pi) = seed_import(&app).await;
    let err = ProtocolService::delete_imported_protocol(&app.db_pool, &ActorContext::Anonymous, id)
        .await
        .expect_err("Anonymous 應拒絕");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
}
