//! CSO 第二次 20-round sweep（2026-05-30）HIGH findings 回歸測試。
//!
//! 對應報告 `.gstack/security-reports/2026-05-30-20round-sweep-r2.json`：
//! - #1 ADMIN_STAFF 自指派 legacy `admin` 提權 → `UserService` 角色指派授權守衛
//! - #2 protocol `change_status` 無 terminal lock → 已核准計畫不可退回 review pipeline
//! - #4 controlled-document `update` 直接設 approved → 繞過 `dms.document.approve` SoD
//!
//! （#3 JSON 醫療匯出 IDOR 為 handler 層守衛，沿用與已測之 export-pdf C3 相同的
//!   `access::require_animal_access` / `require_iacuc_protocol_access`。）

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::glp_compliance::{
    UpdateChangeRequestRequest, UpdateControlledDocumentRequest, UpdateStudyReportRequest,
};
use erp_backend::models::{
    ChangeStatusRequest, ImportApprovedProtocolRequest, ProtocolStatus, UpdateUserRequest,
};
use erp_backend::services::{GlpComplianceService, ProtocolService, UserService};
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "cso_r2_regression",
};

fn empty_update_user(role_ids: Vec<Uuid>) -> UpdateUserRequest {
    UpdateUserRequest {
        email: None,
        display_name: None,
        phone: None,
        phone_ext: None,
        organization: None,
        entry_date: None,
        position: None,
        aup_roles: None,
        years_experience: None,
        trainings: None,
        is_internal: None,
        is_active: None,
        role_ids: Some(role_ids),
        expires_at: None,
        version: None, // None 跳過 optimistic-lock 版本檢查
        force_role_change: None,
    }
}

fn update_doc_status(status: &str) -> UpdateControlledDocumentRequest {
    UpdateControlledDocumentRequest {
        title: None,
        category: None,
        status: Some(status.to_string()),
        effective_date: None,
        review_due_date: None,
        retention_years: None,
        file_path: None,
    }
}

async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("cso-r2-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("cso r2 {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

// ── #1：非管理員 actor 不得指派 legacy `admin` 角色 ───────────────────────────
#[tokio::test]
#[serial]
async fn assign_legacy_admin_role_blocked_for_non_admin_actor() {
    let app = TestApp::spawn().await;

    let admin_role_id: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'admin'")
        .fetch_one(&app.db_pool)
        .await
        .expect("legacy admin role 必須存在（migration 002 seed）");

    // actor：一個「沒有任何管理員層級角色」的使用者（模擬 ADMIN_STAFF 持 admin.user.edit）
    let actor_id = seed_user(&app, "actor").await;
    let target_id = seed_user(&app, "target").await;

    let actor = ActorContext::User(CurrentUser {
        id: actor_id,
        email: "actor@test.local".into(),
        roles: vec![], // 守衛改查 DB user_roles，actor 在 DB 無 admin-tier 角色
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    });

    let err = UserService::update(
        &app.db_pool,
        &actor,
        target_id,
        &empty_update_user(vec![admin_role_id]),
    )
    .await
    .expect_err("非管理員 actor 指派 legacy admin 應被拒絕");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（指派 admin 角色屬真授權檢查），實得：{err:?}"
    );

    // target 不應因失敗的呼叫而被指派到 admin
    let has_admin: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM user_roles ur
           INNER JOIN roles r ON ur.role_id = r.id
           WHERE ur.user_id = $1 AND r.code = 'admin')"#,
    )
    .bind(target_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch target roles");
    assert!(!has_admin, "守衛失效時 target 會被提權為 admin");

    // 對照：System actor（受信任）指派 admin 應成功
    UserService::update(
        &app.db_pool,
        &SYSTEM_TEST,
        target_id,
        &empty_update_user(vec![admin_role_id]),
    )
    .await
    .expect("System actor 指派 admin 應放行");
}

// ── #2：已核准計畫不可經泛型 change_status 退回 / 刪除 ─────────────────────────
async fn seed_approved_protocol(app: &TestApp) -> Uuid {
    let admin_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'CSO r2 status guard', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR-{unique}"))
    .bind(format!("IACUC-{unique}"))
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert approved protocol");
    id
}

#[tokio::test]
#[serial]
async fn protocol_change_status_blocks_approved_revert() {
    let app = TestApp::spawn().await;
    let protocol_id = seed_approved_protocol(&app).await;

    // 試圖把 APPROVED 退回 DRAFT（重開 review pipeline）→ 應拒絕
    let req = ChangeStatusRequest {
        to_status: ProtocolStatus::Draft,
        remark: None,
        reviewer_ids: None,
        vet_id: None,
    };
    let err = ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &req)
        .await
        .expect_err("APPROVED 不可退回 DRAFT");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應為 BusinessRule，實得：{err:?}"
    );

    let status: (String,) = sqlx::query_as("SELECT status::text FROM protocols WHERE id = $1")
        .bind(protocol_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(status.0, "APPROVED", "守衛失效時 status 會被退回");

    // 對照：APPROVED → SUSPENDED 為合法生命週期轉移，應成功
    let req_ok = ChangeStatusRequest {
        to_status: ProtocolStatus::Suspended,
        remark: None,
        reviewer_ids: None,
        vet_id: None,
    };
    ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &req_ok)
        .await
        .expect("APPROVED → SUSPENDED 應成功");
}

// ── #4：controlled-document update 不得設定「發布類」狀態（繞過簽核 SoD） ───────
async fn seed_controlled_document(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO controlled_documents (id, doc_number, title, doc_type, status)
           VALUES ($1, $2, 'CSO r2 SoD doc', 'SOP', 'draft')"#,
    )
    .bind(id)
    .bind(format!("DOC-{unique}"))
    .execute(&app.db_pool)
    .await
    .expect("insert controlled document");
    id
}

#[tokio::test]
#[serial]
async fn controlled_document_update_blocks_release_status() {
    let app = TestApp::spawn().await;
    let doc_id = seed_controlled_document(&app).await;

    // 試圖經泛型 update 直接設 approved → 應拒絕（須走 approve_controlled_document）
    let err = GlpComplianceService::update_controlled_document(
        &app.db_pool,
        &SYSTEM_TEST,
        doc_id,
        &update_doc_status("approved"),
    )
    .await
    .expect_err("update 不可直接設 approved");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應為 BusinessRule(422)（發布狀態須走簽核流程，業務規則非權限拒絕），實得：{err:?}"
    );

    let status: (String,) = sqlx::query_as("SELECT status FROM controlled_documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch doc status");
    assert_eq!(status.0, "draft", "守衛失效時文件會被標為 approved");

    // 對照：非發布類狀態（under_review）可正常更新
    GlpComplianceService::update_controlled_document(
        &app.db_pool,
        &SYSTEM_TEST,
        doc_id,
        &update_doc_status("under_review"),
    )
    .await
    .expect("update 設 under_review 應成功");
}

// ── #4 收尾：change_request update 不得直接設 approved（繞過 approve_change_request） ─
fn update_change_request_status(status: &str) -> UpdateChangeRequestRequest {
    UpdateChangeRequestRequest {
        title: None,
        description: None,
        justification: None,
        impact_assessment: None,
        status: Some(status.to_string()),
    }
}

async fn seed_change_request(app: &TestApp, requested_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO change_requests (id, change_number, title, change_type, description, status, requested_by)
           VALUES ($1, $2, 'CSO r2 change', 'process', 'desc', 'draft', $3)"#,
    )
    .bind(id)
    .bind(format!("CR-{unique}"))
    .bind(requested_by)
    .execute(&app.db_pool)
    .await
    .expect("insert change request");
    id
}

#[tokio::test]
#[serial]
async fn change_request_update_blocks_release_status() {
    let app = TestApp::spawn().await;
    let user = seed_user(&app, "cr-requester").await;
    let cr_id = seed_change_request(&app, user).await;

    let err = GlpComplianceService::update_change_request(
        &app.db_pool,
        &SYSTEM_TEST,
        cr_id,
        &update_change_request_status("approved"),
    )
    .await
    .expect_err("update 不可直接設 approved");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應為 BusinessRule(422)（發布狀態須走簽核流程，業務規則非權限拒絕），實得：{err:?}"
    );

    let status: (String,) = sqlx::query_as("SELECT status FROM change_requests WHERE id = $1")
        .bind(cr_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(
        status.0, "draft",
        "守衛失效時 change_request 會被標為 approved"
    );

    // 對照：非發布類狀態（under_review）可正常更新
    GlpComplianceService::update_change_request(
        &app.db_pool,
        &SYSTEM_TEST,
        cr_id,
        &update_change_request_status("under_review"),
    )
    .await
    .expect("update 設 under_review 應成功");
}

// ── #4 收尾：study_report update 不得直接設 approved/signed（跳過電子簽章） ─────────
fn update_study_report_status(status: &str) -> UpdateStudyReportRequest {
    UpdateStudyReportRequest {
        title: None,
        status: Some(status.to_string()),
        summary: None,
        methods: None,
        results: None,
        conclusions: None,
        deviations: None,
        qau_statement: None,
    }
}

async fn seed_study_report(app: &TestApp, protocol_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO study_final_reports (id, report_number, protocol_id, title, status)
           VALUES ($1, $2, $3, 'CSO r2 report', 'draft')"#,
    )
    .bind(id)
    .bind(format!("RPT-{unique}"))
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert study report");
    id
}

#[tokio::test]
#[serial]
async fn study_report_update_blocks_release_status() {
    let app = TestApp::spawn().await;
    let protocol_id = seed_approved_protocol(&app).await;
    let report_id = seed_study_report(&app, protocol_id).await;

    for bad in ["approved", "signed"] {
        let err = GlpComplianceService::update_study_report(
            &app.db_pool,
            &SYSTEM_TEST,
            report_id,
            &update_study_report_status(bad),
        )
        .await
        .expect_err("update 不可直接設發布類狀態");
        assert!(
            matches!(err, AppError::BusinessRule(_)),
            "{bad} 應為 BusinessRule(422)（發布狀態須走簽核流程，業務規則非權限拒絕），實得：{err:?}"
        );
    }

    let status: (String,) = sqlx::query_as("SELECT status FROM study_final_reports WHERE id = $1")
        .bind(report_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status");
    assert_eq!(status.0, "draft", "守衛失效時報告會被標為 approved/signed");

    GlpComplianceService::update_study_report(
        &app.db_pool,
        &SYSTEM_TEST,
        report_id,
        &update_study_report_status("under_review"),
    )
    .await
    .expect("update 設 under_review 應成功");
}

// ── 匯入授權收緊：pi_user_id 必須 active 且具 PI 類角色 ──────────────────────────
fn import_req(pi_user_id: Uuid, sd_user_id: Uuid, iacuc_no: &str) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "CSO r2 import guard".to_string(),
        pi_user_id: Some(pi_user_id),
        study_director_user_id: sd_user_id,
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

#[tokio::test]
#[serial]
async fn import_approved_requires_pi_role_user() {
    let app = TestApp::spawn().await;
    let unique = &Uuid::new_v4().to_string()[..8];

    // SD：合法 EXPERIMENT_STAFF（PI 驗證先於 SD，故 non_pi 案仍會先因 PI 失敗）
    let sd = seed_user(&app, "import-sd").await;
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF'",
    )
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("assign EXPERIMENT_STAFF role");

    // pi_user_id 指向「無 PI 類角色」的使用者 → 應拒絕（Validation）
    let non_pi = seed_user(&app, "import-nonpi").await;
    let err = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(non_pi, sd, &format!("CSO-IMP-{unique}")),
        non_pi,
    )
    .await
    .expect_err("非 PI 角色不可作為計畫主持人");
    assert!(
        matches!(err, AppError::Validation(_)),
        "應為 Validation，實得：{err:?}"
    );

    // 對照：具 PI 角色且 active 的使用者 → 應可匯入
    let pi = seed_user(&app, "import-pi").await;
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("CSO-IMP-OK-{unique}")),
        pi,
    )
    .await
    .expect("具 PI 角色之啟用使用者應可匯入");
}
