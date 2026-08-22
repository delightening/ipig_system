//! PI 帳號開通守衛回歸測試（code-review H2 / H3）。
//!
//! H2：provision_pi_account 建新外部 PI 帳號時未走 UserService::create，
//!     原本不寫 USER_CREATE 稽核 → 帳號在 user 稽核軸上隱形。修復後補記。
//! H3：relink 會關聯到任意 email 相符的既有帳號（含內部 admin）→ 可被用於
//!     對既有帳號觸發密碼重設信、或把計畫 PI 誤掛內部高權帳號。修復後拒絕
//!     relink 到內部或具非 PI 高權角色的帳號。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::ImportApprovedProtocolRequest;
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "pi_provision_guard_regression",
};

async fn seed_sd(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'guard sd', true, false)"#,
    )
    .bind(id)
    .bind(format!("guard-sd-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert sd user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign EXPERIMENT_STAFF role");
    id
}

/// 種一個指定 email / is_internal / 角色的帳號。
async fn seed_account(
    app: &TestApp,
    email: &str,
    is_internal: bool,
    role_code: Option<&str>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'existing acct', $3, true, false)"#,
    )
    .bind(id)
    .bind(email)
    .bind(is_internal)
    .execute(&app.db_pool)
    .await
    .expect("insert account");
    if let Some(code) = role_code {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
        )
        .bind(id)
        .bind(code)
        .execute(&app.db_pool)
        .await
        .expect("assign role");
    }
    id
}

/// 建立一個 import_pending（外部 PI）計劃，working_content.basic.pi.email = `pi_email`。
async fn seed_import_protocol(app: &TestApp, pi_email: &str) -> Uuid {
    let importer = seed_sd(app).await;
    let sd = seed_sd(app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let req = ImportApprovedProtocolRequest {
        title: "PI 開通守衛測試計劃".to_string(),
        pi_user_id: None, // 外部 PI
        study_director_user_id: sd,
        iacuc_no: format!("PIG-{unique}"),
        application_no: Some(format!("APIG-{unique}")),
        working_content: Some(serde_json::json!({
            "basic": { "pi": { "name": "外部李教授", "email": pi_email } }
        })),
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
    };
    let p = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, importer)
        .await
        .expect("外部 PI 匯入應成功");
    p.id
}

async fn user_create_audit_count(app: &TestApp, user_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'USER_CREATE' AND entity_type = 'user' AND entity_id = $1",
    )
    .bind(user_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count USER_CREATE audit")
}

// ── H2：開通新外部 PI 帳號須留 USER_CREATE 稽核 ──
#[tokio::test]
#[serial]
async fn provision_new_pi_writes_user_create_audit() {
    let app = TestApp::spawn().await;
    let pi_email = format!(
        "new-pi-{}@external.example",
        &Uuid::new_v4().to_string()[..8]
    );
    let protocol_id = seed_import_protocol(&app, &pi_email).await;

    let (pi_user_id, _email, created_new) =
        ProtocolService::provision_pi_account(&app.db_pool, &SYSTEM_TEST, protocol_id)
            .await
            .expect("開通新 PI 帳號應成功");

    assert!(created_new, "email 不存在時應建立新帳號");
    assert_eq!(
        user_create_audit_count(&app, pi_user_id).await,
        1,
        "新建 PI 帳號必須留下一筆 USER_CREATE 稽核（修復前完全沒有）"
    );
}

// ── H3：email 對應內部帳號 → 拒絕 relink ──
#[tokio::test]
#[serial]
async fn provision_rejects_relink_to_internal_account() {
    let app = TestApp::spawn().await;
    let email = format!(
        "internal-{}@ipigsystem.asia",
        &Uuid::new_v4().to_string()[..8]
    );
    seed_account(&app, &email, true, None).await; // 內部帳號
    let protocol_id = seed_import_protocol(&app, &email).await;

    let err = ProtocolService::provision_pi_account(&app.db_pool, &SYSTEM_TEST, protocol_id)
        .await
        .expect_err("relink 到內部帳號應被拒絕");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應回 BusinessRule(422)（email 衝突屬業務規則，非權限拒絕），實得：{err:?}"
    );
}

// ── H3：email 對應具非 PI 高權角色的帳號 → 拒絕 relink ──
#[tokio::test]
#[serial]
async fn provision_rejects_relink_to_privileged_account() {
    let app = TestApp::spawn().await;
    let email = format!(
        "staff-{}@external.example",
        &Uuid::new_v4().to_string()[..8]
    );
    seed_account(&app, &email, false, Some("IACUC_STAFF")).await; // 外部但具高權角色
    let protocol_id = seed_import_protocol(&app, &email).await;

    let err = ProtocolService::provision_pi_account(&app.db_pool, &SYSTEM_TEST, protocol_id)
        .await
        .expect_err("relink 到高權帳號應被拒絕");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應回 BusinessRule(422)（email 衝突屬業務規則，非權限拒絕），實得：{err:?}"
    );
}

// ── 對照：email 對應外部、僅具 PI 角色的帳號 → 允許 relink（不新建） ──
#[tokio::test]
#[serial]
async fn provision_relinks_to_external_pi_only_account() {
    let app = TestApp::spawn().await;
    let email = format!(
        "ext-pi-{}@external.example",
        &Uuid::new_v4().to_string()[..8]
    );
    let existing = seed_account(&app, &email, false, Some("PI")).await;
    let protocol_id = seed_import_protocol(&app, &email).await;

    let (pi_user_id, _email, created_new) =
        ProtocolService::provision_pi_account(&app.db_pool, &SYSTEM_TEST, protocol_id)
            .await
            .expect("relink 到外部 PI 帳號應成功");

    assert!(!created_new, "email 已存在外部 PI 帳號時不應新建");
    assert_eq!(pi_user_id, existing, "應 relink 到既有外部 PI 帳號");
}

// ── M4：finalize 後（import_pending=false）admin 仍可開通 PI 帳號 ──
#[tokio::test]
#[serial]
async fn admin_can_provision_after_finalize() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    let pi_email = format!(
        "m4-pi-{}@external.example",
        &Uuid::new_v4().to_string()[..8]
    );
    let protocol_id = seed_import_protocol(&app, &pi_email).await;

    // 模擬完成補登：import_pending=false 後 can_manage_import_pending 對所有人回 false
    sqlx::query("UPDATE protocols SET import_pending = false WHERE id = $1")
        .bind(protocol_id)
        .execute(&app.db_pool)
        .await
        .expect("set import_pending=false");

    // M4：admin 不受 import_pending 限制，finalize 後仍可補開通（修復前回 403）
    let res = app
        .auth_post(
            &format!("/api/v1/protocols/{protocol_id}/provision-pi"),
            &serde_json::json!({}),
            &admin_token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        200,
        "admin 在 finalize 後仍應可開通 PI 帳號（M4），實得 {}",
        res.status()
    );
}
