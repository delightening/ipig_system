//! GLP 最終報告：身分即授權（2026-09-05，`docs/reviews/2026-09-03-code-side-issues.md` P0-1）。
//!
//! `study.report.manage` 過去是「授給零角色，只有 admin 能用」的角色權限碼。
//! 改走身分即授權後（比照 `services/protocol/closure.rs`），這支測試要證明的是
//! **接線接得起來**：SD 身分能建立／編輯／簽署自己計畫的報告，非 SD（含 admin）
//! 不能簽，QAU 品保聲明與報告本文的授權互相獨立且有結構性 SoD。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::glp_compliance::{CreateStudyReportRequest, UpdateStudyReportRequest};
use erp_backend::services::{AuthService, GlpComplianceService};

const TEST_PASSWORD: &str = "iPig$ecure1";

/// 電子簽章列在 DB 裡的實際樣子（同 `api_protocol_closure_sign_path.rs` 的 `SigRow`）。
#[derive(Debug, sqlx::FromRow)]
struct SigRow {
    entity_type: String,
    entity_id: String,
    signature_type: String,
    is_valid: bool,
    signer_id: Uuid,
}

async fn fetch_sig(app: &TestApp, id: Uuid) -> SigRow {
    sqlx::query_as::<_, SigRow>(
        "SELECT entity_type, entity_id, signature_type, is_valid, signer_id
           FROM electronic_signatures WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&app.db_pool)
    .await
    .expect("簽章列應該存在")
}

/// 建一個密碼可用的使用者並指派角色（不能沿用假 hash，`sign_record_tx` 會真的驗密碼）。
async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let hash = AuthService::hash_password(TEST_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name,
                              is_active, is_internal, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(format!(
        "study-report-{}@example.com",
        &Uuid::new_v4().to_string()[..8]
    ))
    .bind(&hash)
    .bind(format!("study-report-{role_code}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    id
}

/// 額外把某個既有使用者也指派一個角色（用於「SD 同時也是 QAU」的 SoD 測試）。
async fn assign_role(app: &TestApp, user_id: Uuid, role_code: &str) {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign extra role");
}

/// 建一份 APPROVED 計畫，`study_director_user_id` 指向傳入的 `sd`。
async fn seed_protocol(app: &TestApp, pi: Uuid, sd: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by,
              study_director_user_id, import_pending)
           VALUES ($1, $2, $3, 'APPROVED'::protocol_status, $4, $4, $5, false)"#,
    )
    .bind(id)
    .bind(format!("SRSIGN-{}", &Uuid::new_v4().to_string()[..8]))
    .bind("最終報告身分即授權測試")
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// 依使用者實際被指派的角色，從 DB 讀出登入後會拿到的角色／權限快照
/// （同鄰檔理由：不硬編權限，矩陣現況決定測試假設）。
async fn current_user(app: &TestApp, id: Uuid) -> CurrentUser {
    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.code FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
    )
    .bind(id)
    .fetch_all(&app.db_pool)
    .await
    .expect("load roles");
    let permissions: Vec<String> = sqlx::query_scalar(
        r#"SELECT DISTINCT p.code
             FROM user_roles ur
             JOIN role_permissions rp ON rp.role_id = ur.role_id
             JOIN permissions p ON p.id = rp.permission_id
            WHERE ur.user_id = $1"#,
    )
    .bind(id)
    .fetch_all(&app.db_pool)
    .await
    .expect("load permissions");

    CurrentUser {
        id,
        email: "study-report-actor@example.com".to_string(),
        roles,
        permissions,
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

async fn actor(app: &TestApp, id: Uuid) -> ActorContext {
    ActorContext::User(current_user(app, id).await)
}

fn create_req(protocol_id: Uuid) -> CreateStudyReportRequest {
    CreateStudyReportRequest {
        protocol_id,
        title: "最終報告".to_string(),
        summary: Some("摘要".to_string()),
        methods: None,
        results: None,
        conclusions: None,
        deviations: None,
    }
}

/// SD 建立 → 編輯 → 簽署，且寫入端產出的簽章列符合預期形狀。
#[tokio::test]
#[serial]
async fn sd_can_create_edit_and_sign_own_report() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD 應能建立自己計畫的最終報告");
    assert_eq!(report.status, "draft");

    let updated = GlpComplianceService::update_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        &UpdateStudyReportRequest {
            title: Some("最終報告（修訂）".to_string()),
            status: None,
            summary: None,
            methods: Some("方法內容".to_string()),
            results: None,
            conclusions: None,
            deviations: None,
        },
    )
    .await
    .expect("SD 應能編輯自己計畫的最終報告");
    assert_eq!(updated.title, "最終報告（修訂）");
    assert_eq!(updated.methods.as_deref(), Some("方法內容"));

    let signed = GlpComplianceService::sign_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("SD 應能簽署自己計畫的最終報告");

    assert_eq!(signed.status, "signed");
    assert_eq!(signed.signed_by, Some(sd));
    assert!(signed.signed_at.is_some());
    let sig_id = signed.signature_id.expect("簽章欄應已寫入");

    let row = fetch_sig(&app, sig_id).await;
    assert_eq!(
        row.entity_type, "study_final_report",
        "entity_type 須與其他簽章實體分開，不能沿用 'protocol' 或 'protocol_closure'"
    );
    assert_eq!(row.entity_id, report.id.to_string());
    assert_eq!(
        row.signature_type, "CONFIRM",
        "SD 對報告具結是 §11.50 responsibility，不是審查核准"
    );
    assert!(row.is_valid);
    assert_eq!(row.signer_id, sd);
}

/// 非本計畫 SD（另一個 EXPERIMENT_STAFF）不能建立此計畫的報告。
#[tokio::test]
#[serial]
async fn non_sd_cannot_create_report_for_protocol() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let stranger = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let err = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, stranger).await,
        &create_req(protocol_id),
    )
    .await
    .expect_err("非本計畫 SD 不應能建立最終報告");
    assert!(
        err.to_string().contains("計劃負責人"),
        "錯誤訊息應指出只有 SD 可以，實際：{err}"
    );
}

/// 🔴 admin 不可代簽——比照結案簽章：代簽的簽章在稽核上沒有價值。
#[tokio::test]
#[serial]
async fn admin_cannot_sign_on_behalf_of_sd() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let admin = seed_user(&app, "SYSTEM_ADMIN").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD 建立報告");

    let err = GlpComplianceService::sign_study_report(
        &app.db_pool,
        &actor(&app, admin).await,
        report.id,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("admin 不應能代替 SD 簽署最終報告");
    assert!(
        err.to_string().contains("計劃負責人"),
        "錯誤訊息應指出只有 SD 可以，實際：{err}"
    );

    let after: (String, Option<Uuid>) = sqlx::query_as(
        "SELECT status, signed_by FROM study_final_reports WHERE id = $1",
    )
    .bind(report.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("query report");
    assert_eq!(after.0, "draft", "被擋下時不該轉態");
    assert!(after.1.is_none(), "被擋下時不該寫入 signed_by");
}

/// 已簽署的報告不可重複簽署。
#[tokio::test]
#[serial]
async fn signing_twice_is_rejected() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD 建立報告");

    GlpComplianceService::sign_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("第一次簽署應成功");

    let err = GlpComplianceService::sign_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("重複簽署應被擋下");
    assert!(
        err.to_string().contains("已簽署"),
        "錯誤訊息應說明已簽署，實際：{err}"
    );
}

/// QAU（非本計畫 SD）可填寫品保聲明；本計畫 SD 即使同時具備 QAU 角色／權限，
/// 仍不可填寫自己那份報告的品保聲明（結構性 SoD，不只依賴權限碼配置）。
#[tokio::test]
#[serial]
async fn qau_can_attest_but_reports_own_sd_cannot() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let qau = seed_user(&app, "QAU").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD 建立報告");

    let attested = GlpComplianceService::update_qau_statement(
        &app.db_pool,
        &actor(&app, qau).await,
        report.id,
        "已完成品保稽核，內容符合 SOP。",
    )
    .await
    .expect("QAU（非本計畫 SD）應能填寫品保聲明");
    assert_eq!(attested.qau_signed_by, Some(qau));
    assert!(attested.qau_signed_at.is_some());
    assert_eq!(
        attested.qau_statement.as_deref(),
        Some("已完成品保稽核，內容符合 SOP。")
    );

    // 讓本計畫 SD 也具備 QAU 角色／權限，證明擋下的是「本人是這份報告的 SD」，
    // 不是「沒有 qau.report_statement.write 權限」。
    assign_role(&app, sd, "QAU").await;
    let err = GlpComplianceService::update_qau_statement(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        "SD 想幫自己的報告寫品保聲明。",
    )
    .await
    .expect_err("本計畫 SD 即使具 QAU 權限，仍不可填寫自己報告的品保聲明");
    assert!(
        err.to_string().contains("職責分離"),
        "錯誤訊息應說明是職責分離（SoD），實際：{err}"
    );
}

/// 沒有 `study.report.view` / `qau.report_statement.write` 的一般使用者，
/// 看不到別人計畫的報告；本人是 SD 的那份仍看得到（至少要能看到自己要簽的）。
#[tokio::test]
#[serial]
async fn sd_can_view_own_report_without_broad_view_permission() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let stranger = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD 建立報告");

    let sd_user = current_user(&app, sd).await;
    assert!(
        !sd_user.permissions.iter().any(|p| p == "study.report.view"),
        "測試前提：EXPERIMENT_STAFF 不應持有 study.report.view，否則本測試量不到身分即授權"
    );
    GlpComplianceService::get_study_report(&app.db_pool, &sd_user, report.id)
        .await
        .expect("SD 應能看到自己要簽的報告，即使沒有 study.report.view");

    let stranger_user = current_user(&app, stranger).await;
    let err = GlpComplianceService::get_study_report(&app.db_pool, &stranger_user, report.id)
        .await
        .expect_err("非本計畫 SD 且無 study.report.view 者不應看得到");
    assert!(
        err.to_string().contains("沒有權限"),
        "錯誤訊息應說明無權限，實際：{err}"
    );
}
