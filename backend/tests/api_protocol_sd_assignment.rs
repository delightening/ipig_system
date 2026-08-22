//! 計劃負責人（SD）角色化指派回歸測試（create / update）。
//!
//! 規則：SD = 啟用中、內部 EXPERIMENT_STAFF；僅 IACUC_STAFF / admin 可指派他人，
//! 其餘登入者只能設自己；GLP 計劃一旦已指派 SD 即鎖定不可變更。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateProtocolRequest, UpdateProtocolRequest};
use erp_backend::services::{access, ProtocolService};
use erp_backend::AppError;

/// 建立啟用中、內部、具指定角色的使用者。
async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("sd-test-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("sd-test-{role_code}"))
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

/// 建構登入使用者 CurrentUser（指定 id + 角色）。帶 `aup.protocol.edit` 權限，
/// 使 R75-P4 後 `Scoped<ProtocolEdit>::authorize` 可通過（本檔測 update 的 SD 指派/GLP
/// 鎖定業務邏輯，edit 授權非受測點）。
fn user_cu(id: Uuid, roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "actor@test.local".to_string(),
        roles: roles.iter().map(|r| r.to_string()).collect(),
        permissions: vec!["aup.protocol.edit".to_string()],
        jti: Uuid::new_v4().to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

/// 建構登入使用者 actor（指定 id + 角色）。
fn user_actor(id: Uuid, roles: &[&str]) -> ActorContext {
    ActorContext::User(user_cu(id, roles))
}

/// 執秘 CurrentUser，**不帶** `aup.protocol.edit` 權限——驗證純 SD 指派僅靠 IACUC_STAFF 角色
/// （`require_protocol_sd_assign`）即可放行，而非舊的 edit 權限旁路。
fn secretary_cu(id: Uuid) -> CurrentUser {
    CurrentUser {
        id,
        email: "secretary@test.local".to_string(),
        roles: vec!["IACUC_STAFF".to_string()],
        permissions: vec![],
        jti: Uuid::new_v4().to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

fn create_req(pi: Uuid, sd: Option<Uuid>, is_glp: bool) -> CreateProtocolRequest {
    CreateProtocolRequest {
        title: "SD 指派測試計劃".to_string(),
        pi_user_id: Some(pi),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": is_glp } })),
        start_date: None,
        end_date: None,
        study_director_user_id: sd,
    }
}

// ── create ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn create_sd_by_secretary_ok() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd), false),
        secretary,
    )
    .await
    .expect("執秘指派 SD 應成功");
    assert_eq!(p.study_director_user_id, Some(sd), "SD 應寫入指派的 staff");
}

#[tokio::test]
#[serial]
async fn create_sd_staff_self_ok() {
    let app = TestApp::spawn().await;
    let staff = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(staff, &["EXPERIMENT_STAFF"]);
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(staff, Some(staff), false),
        staff,
    )
    .await
    .expect("staff 設自己為 SD 應成功");
    assert_eq!(p.study_director_user_id, Some(staff));
}

#[tokio::test]
#[serial]
async fn create_sd_staff_other_forbidden() {
    let app = TestApp::spawn().await;
    let staff1 = seed_user(&app, "EXPERIMENT_STAFF").await;
    let staff2 = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(staff1, &["EXPERIMENT_STAFF"]);
    let err = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(staff1, Some(staff2), false),
        staff1,
    )
    .await
    .expect_err("staff 指派他人為 SD 應被拒");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
}

#[tokio::test]
#[serial]
async fn create_no_sd_ok() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(pi, &["PI"]);
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None, false), pi)
        .await
        .expect("客戶未填 SD 應成功（留待執秘指派）");
    assert_eq!(p.study_director_user_id, None, "未指派時 SD 應為 NULL");
}

#[tokio::test]
#[serial]
async fn create_sd_invalid_role_rejected() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);
    // pi（非 EXPERIMENT_STAFF）不得任 SD
    let err = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(pi), false),
        secretary,
    )
    .await
    .expect_err("非試驗工作人員任 SD 應被拒");
    assert!(
        matches!(err, AppError::Validation(_)),
        "應 Validation，實得：{err:?}"
    );
}

// ── update（GLP 鎖）─────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn update_sd_nonglp_by_secretary_ok() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd_a), false),
        secretary,
    )
    .await
    .expect("create");
    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd_b),
        version: None,
        source_form_version: None,
    };
    // 純 SD 指派（無內容變更）→ 欄位感知授權允許執秘協調指派。
    let scope = access::Scoped::<access::ProtocolEdit>::authorize_update(
        &app.db_pool,
        &secretary_cu(secretary),
        p.id,
        false,
    )
    .await
    .expect("authorize SD assignment");
    let updated = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect("非 GLP 計劃執秘變更 SD 應成功");
    assert_eq!(updated.study_director_user_id, Some(sd_b), "SD 應更新為 B");
}

#[tokio::test]
#[serial]
async fn update_sd_glp_locked() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);
    // GLP 計劃，建立時即指派 SD=A
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd_a), true),
        secretary,
    )
    .await
    .expect("create glp");
    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd_b),
        version: None,
        source_form_version: None,
    };
    // 純 SD 指派（無內容變更）→ 欄位感知授權允許執秘協調指派。
    let scope = access::Scoped::<access::ProtocolEdit>::authorize_update(
        &app.db_pool,
        &secretary_cu(secretary),
        p.id,
        false,
    )
    .await
    .expect("authorize SD assignment");
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect_err("GLP 計劃已指派 SD 後變更應被拒");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}
