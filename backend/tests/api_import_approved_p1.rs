//! 匯入已核准計劃「完整補登」P1 回歸測試。
//!
//! - 匯入即標記 import_pending=true + 存 application_no（申請編號）。
//! - import_pending（APPROVED）期間允許編輯 working_content。
//! - 完成補登清旗標 + 建 protocol_versions v1 + 記 original_version_label；之後恢復鎖定。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{ImportApprovedProtocolRequest, ProtocolStatus, UpdateProtocolRequest};
use erp_backend::services::{access, ProtocolService};
use erp_backend::AppError;

/// R75-P4：update 改吃 `Scoped<ProtocolEdit>`。import 補登測試以一個帶標準 edit
/// 授權的 User 模擬編輯者：補登中計劃的合法編輯者＝`aup.protocol.import_approved` 權限持有者
/// （`can_manage_import_pending` 放行）。草稿編輯權收緊後（執秘唯讀），import-pending 編輯改以
/// 補登權限把關，故此處用 import_approved holder 而非 IACUC_STAFF + edit。
async fn edit_user(app: &TestApp) -> CurrentUser {
    // 須實際 seed 一筆 users 列：update 走 User actor 會寫 protocol_activities，
    // actor_id 有 FK → users，隨機未存在的 id 會違反外鍵。
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'import editor', true, false)"#,
    )
    .bind(id)
    .bind(format!("import-editor-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert editor user");
    CurrentUser {
        id,
        email: "import-editor@test.local".to_string(),
        roles: vec!["EXPERIMENT_STAFF".to_string()],
        permissions: vec!["aup.protocol.import_approved".to_string()],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "import_p1_regression",
};

/// 建立一個具 PI 角色、啟用中的使用者（import_approved 要求 pi_user_id 為合法 PI）。
async fn seed_pi(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'import p1 pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("import-p1-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert pi user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    id
}

/// 建立一個具 EXPERIMENT_STAFF 角色、啟用中的使用者（import_approved 要求 SD 為合法試驗工作人員）。
async fn seed_sd(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'import p1 sd', true, false)"#,
    )
    .bind(id)
    .bind(format!("import-p1-sd-{}@test.local", &Uuid::new_v4().to_string()[..6]))
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

fn import_req(pi: Uuid, sd: Uuid, iacuc_no: &str) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "P1 補登測試計劃".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no: iacuc_no.to_string(),
        application_no: Some("APIG-103001".to_string()),
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

fn content_update(content: serde_json::Value) -> UpdateProtocolRequest {
    UpdateProtocolRequest {
        title: None,
        working_content: Some(content),
        start_date: None,
        end_date: None,
        study_director_user_id: None,
        version: None,
        source_form_version: None,
    }
}

// ── 外部 PI（pi_user_id=None）：pi_user_id 記為匯入者、跳過 PI-role 檢查（P5）──
#[tokio::test]
#[serial]
async fn import_external_pi_defaults_to_importer() {
    let app = TestApp::spawn().await;
    let sd = seed_sd(&app).await;
    // 匯入者本人（非 PI 角色亦可，因外部 PI 不檢查角色）
    let importer = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let mut req = import_req(sd, sd, &format!("PIG-{unique}"));
    req.pi_user_id = None; // 外部 PI
    req.working_content = Some(serde_json::json!({
        "basic": { "pi": { "name": "外部王教授" }, "sponsor": { "name": "委託單位 A" } }
    }));

    let p = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, importer)
        .await
        .expect("外部 PI 匯入應成功（不檢查 PI 角色）");
    assert_eq!(p.pi_user_id, importer, "外部 PI 時 pi_user_id 應記為匯入者");
    assert!(p.import_pending, "匯入即補登中");
}

// ── 外部 PI 缺姓名 → Validation（後端守 API contract）──
#[tokio::test]
#[serial]
async fn import_external_pi_requires_name() {
    let app = TestApp::spawn().await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let mut req = import_req(sd, sd, &format!("PIG-{unique}"));
    req.pi_user_id = None; // 外部 PI 但未提供 working_content.basic.pi.name
    req.working_content = None;

    let err = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, sd)
        .await
        .expect_err("外部 PI 缺姓名應拒絕");
    assert!(
        matches!(err, AppError::Validation(_)),
        "應 Validation，實得：{err:?}"
    );
}

// ── 版本名冊：import_approved 帶入的 source_form_version 應寫入 protocols ──
#[tokio::test]
#[serial]
async fn import_persists_source_form_version() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;

    let mut req = import_req(pi, sd, &format!("PIG-{}", &Uuid::new_v4().to_string()[..8]));
    req.source_form_version = Some("C".to_string());
    let p = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, sd)
        .await
        .expect("匯入（帶版本 C）應成功");
    assert_eq!(
        p.source_form_version.as_deref(),
        Some("C"),
        "source_form_version 應寫入為 C"
    );

    // 未帶版本 → NULL（新案「最新版」語意由前端 normalize 處理）
    let mut req2 = import_req(pi, sd, &format!("PIG-{}", &Uuid::new_v4().to_string()[..8]));
    req2.source_form_version = None;
    let p2 = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req2, sd)
        .await
        .expect("匯入（未帶版本）應成功");
    assert_eq!(p2.source_form_version, None, "未帶版本應為 NULL");
}

// ── 匯入即標記 import_pending + 存 application_no ──
#[tokio::test]
#[serial]
async fn import_sets_pending_and_application_no() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}")),
        pi,
    )
    .await
    .expect("import_approved 應成功");

    assert!(p.import_pending, "匯入應標記 import_pending=true");
    assert_eq!(
        p.application_no.as_deref(),
        Some("APIG-103001"),
        "應存申請編號"
    );
    assert!(
        matches!(p.status, ProtocolStatus::Approved),
        "匯入即 APPROVED"
    );
}

// ── Service 層拒絕 Anonymous 觸發匯入 / 完成補登（CSO bot fix）──
#[tokio::test]
#[serial]
async fn import_and_finalize_reject_anonymous() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    // 匿名不可匯入
    let err = ProtocolService::import_approved(
        &app.db_pool,
        &ActorContext::Anonymous,
        &import_req(pi, sd, &format!("PIG-{unique}")),
        pi,
    )
    .await
    .expect_err("Anonymous 不可匯入已核准計劃");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );

    // 先以 System 匯入，再以匿名嘗試完成補登 → 應 Forbidden
    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}-b")),
        pi,
    )
    .await
    .expect("system import");
    let err2 = ProtocolService::finalize_import(&app.db_pool, &ActorContext::Anonymous, p.id, None)
        .await
        .expect_err("Anonymous 不可完成補登");
    assert!(
        matches!(err2, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err2:?}"
    );
}

// ── 補登中可編輯 → 完成補登清旗標 + 建 v1 snapshot + 恢復鎖定 ──
#[tokio::test]
#[serial]
async fn finalize_clears_pending_and_snapshots_v1() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}")),
        pi,
    )
    .await
    .expect("import");

    // 補登中（APPROVED + import_pending）允許編輯 working_content
    let editor = edit_user(&app).await;
    let editor_actor = ActorContext::User(editor.clone());
    let scope = access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &editor, p.id)
        .await
        .expect("authorize edit");
    ProtocolService::update(
        &app.db_pool,
        &editor_actor,
        scope,
        &content_update(serde_json::json!({ "basic": { "study_title": "補登內容" } })),
    )
    .await
    .expect("import_pending 期間應可編輯 working_content");

    // 完成補登
    let finalized = ProtocolService::finalize_import(
        &app.db_pool,
        &SYSTEM_TEST,
        p.id,
        Some("v2.1".to_string()),
    )
    .await
    .expect("finalize_import 應成功");
    assert!(
        !finalized.import_pending,
        "完成補登後 import_pending 應清除"
    );
    assert_eq!(
        finalized.original_version_label.as_deref(),
        Some("v2.1"),
        "應記原始版本號"
    );

    // 建立了 v1 版本快照
    let vcount: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM protocol_versions WHERE protocol_id = $1")
            .bind(p.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count versions");
    assert_eq!(vcount, 1, "完成補登應建 1 筆 protocol_versions");

    // 完成後恢復鎖定：import_pending 已清，import_approved 編輯者不再具編輯權
    // （can_manage_import_pending → false；非 PI/SD/admin）→ 授權層即 Forbidden。
    let denied =
        access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &editor, p.id).await;
    assert!(
        matches!(denied, Err(AppError::Forbidden(_))),
        "完成補登後 import 編輯者應於授權層被拒，實得：{:?}",
        denied.err()
    );

    // 服務層鎖：即使持有有效 ProtocolEdit 證明（PI 仍可授權），import_pending 清除後內容更新
    // 仍被業務規則拒絕（Scoped 僅承載 protocol_id、不保證業務狀態，故需服務層獨立把關）。
    let pi_cu = CurrentUser {
        id: pi,
        email: "import-pi@test.local".to_string(),
        roles: vec!["PI".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    };
    let pi_scope = access::Scoped::<access::ProtocolEdit>::authorize(&app.db_pool, &pi_cu, p.id)
        .await
        .expect("PI 應可取得 ProtocolEdit 證明");
    let lock_err = ProtocolService::update(
        &app.db_pool,
        &ActorContext::User(pi_cu),
        pi_scope,
        &content_update(serde_json::json!({ "basic": { "study_title": "鎖後不可改" } })),
    )
    .await
    .expect_err("完成補登後 APPROVED 內容更新應被服務層業務規則拒絕");
    assert!(
        matches!(lock_err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{lock_err:?}"
    );

    // 再次完成補登應被拒（非補登中）
    let err2 = ProtocolService::finalize_import(&app.db_pool, &SYSTEM_TEST, p.id, None)
        .await
        .expect_err("非補登中不可完成補登");
    assert!(
        matches!(err2, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err2:?}"
    );
}
