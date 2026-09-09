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

/// 直接從 DB 撈簽章列——**不經 service**，因為要驗的正是 service 有沒有正確寫進去。
/// 用 `fetch_one` 而非 `fetch_optional`：呼叫端都是「簽署成功之後」才查，
/// 查不到就是缺陷，該在這裡當場炸，而不是回 `None` 讓後面的斷言拿空值繼續跑。
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
    assign_role(app, id, role_code).await;
    id
}

/// 額外把某個既有使用者也指派一個角色（用於「SD 同時也是 QAU／admin」的 SoD 測試）。
///
/// 🔴 **一定要斷言 `rows_affected() == 1`。**
/// 這是 `INSERT … SELECT … FROM roles WHERE code = $2`：角色碼不存在時它插入 0 列、
/// 回傳 `Ok`，**完全不會報錯**。2026-09-09 因此踩到一次——本檔原本用 `"SYSTEM_ADMIN"`
/// 造「SD 兼 admin」的交集，而 `003_seed.sql` 的 15 個角色裡**根本沒有 SYSTEM_ADMIN**
///（只有 legacy 的 `admin`；`is_admin()` 兩個都認，見 `middleware/auth.rs:70-74`）。
/// 結果是那個使用者從頭到尾都不是 admin，`admin_who_is_the_reports_sd_still_cannot_attest`
/// 退化成它上面那支的複本——**對舊碼一樣是綠的，證明不了裁定 102.1 的改動**。
/// 有了這道斷言，日後角色碼被改名或漏 seed 會立刻紅，而不是靜默把測試掏空。
async fn assign_role(app: &TestApp, user_id: Uuid, role_code: &str) {
    let res = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    assert_eq!(
        res.rows_affected(),
        1,
        "角色 `{role_code}` 未被指派——`roles` 表沒有這個代碼（seed 漏了或被改名）。\
         這會讓依賴該角色的測試靜默失去鑑別力，故直接失敗。"
    );
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

/// 包成 `ActorContext::User`。本組 service 一律 `actor.require_user()?`，
/// 用 `ActorContext::System` 會直接回錯——身分即授權需要一個「人」，
/// 系統身分沒有 SD 資格可言（`api_cso_r2_regression.rs` 就是為此改用 `sd_actor`）。
async fn actor(app: &TestApp, id: Uuid) -> ActorContext {
    ActorContext::User(current_user(app, id).await)
}

/// 建立報告用的最小 payload：只填必要欄位，其餘留 `None`。
/// 刻意不填滿——後續的編輯測試靠「這些欄位原本是 None」來斷言 `COALESCE` 的行為。
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
    // ⚠️ 用 seed 真的有的 legacy `admin`，不是 `SYSTEM_ADMIN`——後者不在 `003_seed.sql`
    // 的 15 個角色裡，指派會靜默插 0 列（`is_admin()` 兩個代碼都認，auth.rs:70-74）。
    let admin = seed_user(&app, "admin").await;
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

    let after: (String, Option<Uuid>) =
        sqlx::query_as("SELECT status, signed_by FROM study_final_reports WHERE id = $1")
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

/// 🔴 已簽署的報告不得再改內容——否則電子簽章會掛在簽署後才被改過的內容上。
///
/// # 為什麼既有測試擋不住這個
///
/// `update_study_report` 原有的守衛只擋「把 `status` **設成** approved/signed」。
/// 這條路徑完全繞過它：報告**已經**是 signed，請求只帶 `summary`、不帶 `status`，
/// 於是 `status = COALESCE(NULL, status)` 讓它維持 signed，
/// 而 `signed_by` / `signed_at` / `signature_id` 一個都不會被清掉。
/// 2026-09-09 由 CodeRabbit 在 MR !10 指出（Data Integrity，Major）。
///
/// # 鑑別力
///
/// 對舊碼會紅：舊碼那次 update 會**成功**，`expect_err` 因此失敗。
/// 下面同時斷言內容真的沒被改動，避免「擋下了但已經寫進去一半」。
#[tokio::test]
#[serial]
async fn signed_report_content_cannot_be_edited() {
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

    let signed = GlpComplianceService::sign_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("SD 簽署");
    assert_eq!(signed.status, "signed");
    let signature_id_before = signed.signature_id;
    assert!(signature_id_before.is_some(), "簽署後應留下 signature_id");

    // 關鍵：不帶 status，只改內容——舊碼會讓它通過且 status 維持 signed。
    let err = GlpComplianceService::update_study_report(
        &app.db_pool,
        &actor(&app, sd).await,
        report.id,
        &UpdateStudyReportRequest {
            title: None,
            status: None,
            summary: Some("簽署之後偷改的摘要".to_string()),
            methods: None,
            results: None,
            conclusions: None,
            deviations: None,
        },
    )
    .await
    .expect_err("已簽署報告的內容不應可再編輯");
    assert!(
        err.to_string().contains("已簽署"),
        "錯誤訊息應說明報告已簽署，實際：{err}"
    );

    // 被擋下時不得留下任何痕跡：內容、簽章欄位都要與簽署當下一致。
    let after: (String, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT status, summary, signature_id FROM study_final_reports WHERE id = $1",
    )
    .bind(report.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("query report");
    assert_eq!(after.0, "signed", "狀態應維持 signed");
    assert_ne!(
        after.1.as_deref(),
        Some("簽署之後偷改的摘要"),
        "被擋下的編輯不得落地"
    );
    assert_eq!(after.2, signature_id_before, "簽章不應被動到");
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

/// 🔴 **admin 身分不構成 SoD 例外**（使用者 2026-09-06 裁定，待決事項 102.1）。
///
/// # 為什麼要單獨一支，上面那支不夠
///
/// `qau_can_attest_but_reports_own_sd_cannot` 的 SD 是 `EXPERIMENT_STAFF`（非 admin），
/// 所以它在**移除 admin 例外之前就已經是綠的**——它證明不了這次的改動。
/// 舊碼寫的是 `!user.is_admin() && sd == Some(user.id)`，缺口只在「SD 同時是 admin」
/// 這個交集上，要測到它就必須讓同一個人同時具備兩者。
///
/// 這支對舊碼會紅：admin 走 `!user.is_admin()` 短路繞過 SoD，`update_qau_statement`
/// 會成功回傳，`expect_err` 因此失敗。
#[tokio::test]
#[serial]
async fn admin_who_is_the_reports_sd_still_cannot_attest() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    // SD 資格來自 EXPERIMENT_STAFF；再疊上 admin 造出「SD 兼 admin」這個交集。
    // ⚠️ 這裡本來寫 `SYSTEM_ADMIN`，而 seed 沒有那個角色碼 → 指派靜默插 0 列，
    // 這支測試因此**整整失去鑑別力**（2026-09-09 由 CodeRabbit 在 MR !10 指出）。
    // `assign_role` 現在會斷言 rows_affected == 1，同一個錯不會再靜默發生。
    let sd_admin = seed_user(&app, "EXPERIMENT_STAFF").await;
    assign_role(&app, sd_admin, "admin").await;
    let protocol_id = seed_protocol(&app, pi, Some(sd_admin)).await;

    let report = GlpComplianceService::create_study_report(
        &app.db_pool,
        &actor(&app, sd_admin).await,
        &create_req(protocol_id),
    )
    .await
    .expect("SD（兼 admin）建立報告");

    let err = GlpComplianceService::update_qau_statement(
        &app.db_pool,
        &actor(&app, sd_admin).await,
        report.id,
        "admin 想幫自己擔任 SD 的報告寫品保聲明。",
    )
    .await
    .expect_err("admin 身分不得成為 SoD 例外——簽署人＝被稽核對象本人的品保聲明沒有意義");
    assert!(
        err.to_string().contains("職責分離"),
        "錯誤訊息應說明是職責分離（SoD），實際：{err}"
    );

    // 擋下時不得留下任何簽署痕跡
    let after: (Option<Uuid>, Option<String>) = sqlx::query_as(
        "SELECT qau_signed_by, qau_statement FROM study_final_reports WHERE id = $1",
    )
    .bind(report.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("query report");
    assert!(after.0.is_none(), "被擋下時不該寫入 qau_signed_by");
    assert!(after.1.is_none(), "被擋下時不該寫入 qau_statement");
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
