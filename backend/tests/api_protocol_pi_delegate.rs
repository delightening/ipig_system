//! PI 代理授權（`protocol_pi_delegates`，migration 010）回歸測試。
//!
//! 涵蓋決策表（見 `services/protocol/pi_delegate.rs` 檔案頂端說明）：
//! - 核准他人：僅現任 SD。
//! - SD 核准自己：拒絕，須改由 IACUC_STAFF/admin。
//! - 撤銷：現任 SD 或 IACUC_STAFF/admin 皆可。
//! - 資格 / 重複核准 / PI 非外部 / 無 SD 的邊界情況。
//! - SD 變更時舊授權自動撤銷。
//! - 結案雙簽 `dual_signature_ready` 接受代理簽章（migration 010 新增的條件 6 分支）。
//! - `can_write_amendment` 含生效中代理人。
//!
//! 直接呼叫 service 層（不經 HTTP），與 `api_protocol_closure_dual_sign.rs` 同構——
//! 這裡要驗的是授權邏輯本身，不是路由層。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::services::protocol::closure::{dual_signature_ready, CLOSURE_ENTITY_TYPE};
use erp_backend::services::{
    access, protocol_closure_sign, AuthService, ClosureSigner, ProtocolService,
};
use erp_backend::AppError;

const TEST_PASSWORD: &str = "iPig$ecure1";

fn actor(id: Uuid, roles: &[&str]) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: format!("{id}@test.local"),
        roles: roles.iter().map(|r| r.to_string()).collect(),
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

async fn seed_user(app: &TestApp, role_code: Option<&str>) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("delegate-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("delegate-{}", &Uuid::new_v4().to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    if let Some(role_code) = role_code {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
        )
        .bind(id)
        .bind(role_code)
        .execute(&app.db_pool)
        .await
        .expect("assign role");
    }
    id
}

/// 建一份外部 PI 計畫：`pi_is_external=true`、`pi_user_id` 借位建立者、指定 SD。
async fn seed_external_pi_protocol(app: &TestApp, created_by: Uuid, sd: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by,
              study_director_user_id, import_pending, pi_is_external)
           VALUES ($1, $2, $3, 'APPROVED'::protocol_status, $4, $4, $5, false, true)"#,
    )
    .bind(id)
    .bind(format!("DELEGATE-{}", &Uuid::new_v4().to_string()[..8]))
    .bind("PI 代理授權測試計畫")
    .bind(created_by)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

async fn active_delegate_row(app: &TestApp, protocol_id: Uuid) -> Option<(Uuid, Uuid)> {
    sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT id, delegate_user_id FROM protocol_pi_delegates \
         WHERE protocol_id = $1 AND revoked_at IS NULL",
    )
    .bind(protocol_id)
    .fetch_optional(&app.db_pool)
    .await
    .expect("query active delegate")
}

// ── 核准 ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn sd_can_authorize_other_as_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await;

    assert!(result.is_ok(), "SD 核准他人應成功：{result:?}");
    let row = active_delegate_row(&app, protocol).await;
    assert_eq!(row.map(|(_, d)| d), Some(delegate));
}

#[tokio::test]
#[serial]
async fn sd_cannot_self_authorize() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        sd,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "SD 核准自己應被拒絕（自簽自證）：{result:?}"
    );
    assert!(active_delegate_row(&app, protocol).await.is_none());
}

#[tokio::test]
#[serial]
async fn iacuc_staff_can_authorize_sd_as_self_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let staff = seed_user(&app, Some("IACUC_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(staff, &["IACUC_STAFF"]),
        protocol,
        sd,
        Some("SD 本人熟悉案情，執秘核准其兼任代理人"),
    )
    .await;

    assert!(result.is_ok(), "執秘核准 SD 本人為代理人應成功：{result:?}");
    let row = active_delegate_row(&app, protocol).await;
    assert_eq!(row.map(|(_, d)| d), Some(sd));
}

#[tokio::test]
#[serial]
async fn admin_can_authorize_sd_as_self_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let admin_id = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(admin_id, &["SYSTEM_ADMIN"]),
        protocol,
        sd,
        None,
    )
    .await;

    assert!(
        result.is_ok(),
        "admin 核准 SD 本人為代理人應成功：{result:?}"
    );
}

#[tokio::test]
#[serial]
async fn iacuc_staff_cannot_authorize_other_as_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let staff = seed_user(&app, Some("IACUC_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // 執秘核准「他人」不在授權規則內——這條路徑限現任 SD 本人。
    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(staff, &["IACUC_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "執秘核准「他人」為代理人應被拒絕（限現任 SD 本人）：{result:?}"
    );
}

#[tokio::test]
#[serial]
async fn authorize_rejects_when_pi_not_external() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    sqlx::query("UPDATE protocols SET pi_is_external = false WHERE id = $1")
        .bind(protocol)
        .execute(&app.db_pool)
        .await
        .expect("flip pi_is_external");

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::BusinessRule(_))),
        "PI 已有真帳號的計畫不該能核准代理人：{result:?}"
    );
}

#[tokio::test]
#[serial]
async fn authorize_rejects_when_no_sd() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, None).await;

    // 沒有 SD 時，連 admin 都無法核准——沒有人是「現任 SD」，也沒有 SD 自己這個組合。
    let admin_id = seed_user(&app, None).await;
    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(admin_id, &["SYSTEM_ADMIN"]),
        protocol,
        delegate,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::BusinessRule(_))),
        "尚未指派 SD 的計畫不該能核准代理人：{result:?}"
    );
}

#[tokio::test]
#[serial]
async fn authorize_rejects_inactive_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let inactive = seed_user(&app, None).await;
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(inactive)
        .execute(&app.db_pool)
        .await
        .expect("deactivate user");
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        inactive,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Validation(_))),
        "停用帳號不得核准為代理人：{result:?}"
    );
}

#[tokio::test]
#[serial]
async fn authorize_rejects_when_already_has_active_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate_a = seed_user(&app, None).await;
    let delegate_b = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate_a,
        None,
    )
    .await
    .expect("first authorize should succeed");

    let result = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate_b,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::BusinessRule(_))),
        "已有生效中代理人時應拒絕再次核准（須先撤銷）：{result:?}"
    );
    let row = active_delegate_row(&app, protocol).await;
    assert_eq!(
        row.map(|(_, d)| d),
        Some(delegate_a),
        "既有代理人不應被隱性覆蓋"
    );
}

// ── 撤銷 ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn revoke_by_current_sd_succeeds() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize");

    ProtocolService::revoke_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        None,
    )
    .await
    .expect("SD 應可撤銷");
    assert!(active_delegate_row(&app, protocol).await.is_none());
}

#[tokio::test]
#[serial]
async fn revoke_by_unrelated_user_rejected() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let stranger = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize");

    let result = ProtocolService::revoke_pi_delegate(
        &app.db_pool,
        &actor(stranger, &["EXPERIMENT_STAFF"]),
        protocol,
        None,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "無關人員不應能撤銷代理人：{result:?}"
    );
    assert!(active_delegate_row(&app, protocol).await.is_some());
}

#[tokio::test]
#[serial]
async fn revoke_without_active_delegate_returns_not_found() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let result = ProtocolService::revoke_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        None,
    )
    .await;

    assert!(matches!(result, Err(AppError::NotFound(_))), "{result:?}");
}

// ── SD 變更自動撤銷 ──────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn sd_change_auto_revokes_delegate() {
    use erp_backend::models::UpdateProtocolRequest;
    use erp_backend::services::access::{ProtocolEdit, Scoped};

    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let new_sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let staff = seed_user(&app, Some("IACUC_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    // `core.rs::update` 是全系統唯一會寫 `study_director_user_id` 的路徑，而它擋住
    // 非 DRAFT/需修訂的計畫——唯一的例外是「APPROVED + import_pending」（補登中）。
    // 外部 PI 計畫幾乎都是匯入進來的 APPROVED 件，補登期間換 SD 正是這條 fallback
    // 存在的實際情境，所以 fixture 補上 import_pending 而不是把狀態改成 DRAFT。
    sqlx::query("UPDATE protocols SET import_pending = true WHERE id = $1")
        .bind(protocol)
        .execute(&app.db_pool)
        .await
        .expect("set import_pending");
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize");
    assert!(active_delegate_row(&app, protocol).await.is_some());

    // 走真正的 update() 流程換 SD（比直接戳 tx-internal 函式更貼近實際整合路徑）：
    // 純 SD 指派（touches_content=false）需 admin/IACUC_STAFF/可編輯者，這裡用執秘。
    let staff_user = CurrentUser {
        id: staff,
        email: format!("{staff}@test.local"),
        roles: vec!["IACUC_STAFF".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let scope =
        Scoped::<ProtocolEdit>::authorize_update(&app.db_pool, &staff_user, protocol, false)
            .await
            .expect("staff should be authorized to reassign SD");
    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(new_sd),
        version: None,
        source_form_version: None,
    };
    ProtocolService::update(&app.db_pool, &actor(staff, &["IACUC_STAFF"]), scope, &req)
        .await
        .expect("update SD");

    assert!(
        active_delegate_row(&app, protocol).await.is_none(),
        "SD 變更後舊代理授權應自動撤銷"
    );
}

// ── 結案雙簽：接受代理簽章 ───────────────────────────────────────

async fn seed_signature_with_delegation(
    app: &TestApp,
    entity_type: &str,
    entity_id: &str,
    signer_id: Uuid,
    delegation_id: Option<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO electronic_signatures
             (id, entity_type, entity_id, signer_id, signature_type,
              content_hash, signature_data, signature_method, meaning, is_valid, delegation_id)
           VALUES ($1, $2, $3, $4, 'CONFIRM', 'hash', 'data', 'password',
                   'CONFIRM'::signature_meaning, true, $5)"#,
    )
    .bind(id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(signer_id)
    .bind(delegation_id)
    .execute(&app.db_pool)
    .await
    .expect("insert signature");
    id
}

#[tokio::test]
#[serial]
async fn dual_signature_ready_accepts_delegate_signature() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize")
    .id;

    let eid = protocol.to_string();
    let pi_sig = seed_signature_with_delegation(
        &app,
        CLOSURE_ENTITY_TYPE,
        &eid,
        delegate,
        Some(delegation_id),
    )
    .await;
    let sd_sig = seed_signature_with_delegation(&app, CLOSURE_ENTITY_TYPE, &eid, sd, None).await;

    let mut tx = app.db_pool.begin().await.expect("begin");
    // pi_user_id 仍是借位的 creator（外部 PI 情境），代理人簽的是「PI 那一欄」。
    let ready = dual_signature_ready(
        &mut tx,
        protocol,
        creator,
        Some(sd),
        Some(pi_sig),
        Some(sd_sig),
    )
    .await
    .expect("gate");
    tx.rollback().await.expect("rollback");

    assert!(
        ready,
        "持有效代理授權的代理人簽章應被視為 PI 那一欄的合法簽署"
    );
}

#[tokio::test]
#[serial]
async fn dual_signature_ready_rejects_delegation_from_another_protocol() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    let other_protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // 代理授權核准在**別份計畫**上。
    let other_delegation = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        other_protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize on other protocol")
    .id;

    let eid = protocol.to_string();
    let pi_sig = seed_signature_with_delegation(
        &app,
        CLOSURE_ENTITY_TYPE,
        &eid,
        delegate,
        Some(other_delegation),
    )
    .await;
    let sd_sig = seed_signature_with_delegation(&app, CLOSURE_ENTITY_TYPE, &eid, sd, None).await;

    let mut tx = app.db_pool.begin().await.expect("begin");
    let ready = dual_signature_ready(
        &mut tx,
        protocol,
        creator,
        Some(sd),
        Some(pi_sig),
        Some(sd_sig),
    )
    .await
    .expect("gate");
    tx.rollback().await.expect("rollback");

    assert!(!ready, "代理授權屬於別份計畫時，不該讓這份計畫的雙簽齊備");
}

// ── can_write_amendment 含生效中代理人 ────────────────────────────

#[tokio::test]
#[serial]
async fn can_write_amendment_includes_active_delegate() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize");

    let delegate_user = CurrentUser {
        id: delegate,
        email: format!("{delegate}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let can_write = access::can_write_amendment(&app.db_pool, &delegate_user, protocol)
        .await
        .expect("can_write_amendment");
    assert!(can_write, "生效中代理人應可建立/更新/提交修正案");

    // SD 本人（非代理人）不應因為代理人存在而額外取得這項權限——
    // 這條規則本來就刻意不含 SD（見 access.rs 註解）。
    let sd_user = CurrentUser {
        id: sd,
        email: format!("{sd}@test.local"),
        roles: vec!["EXPERIMENT_STAFF".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let sd_can_write = access::can_write_amendment(&app.db_pool, &sd_user, protocol)
        .await
        .expect("can_write_amendment");
    assert!(!sd_can_write, "SD 本人不因代理人存在而取得修正案寫入權");
}

// ── 結案雙簽：真正的寫入端（不是只測讀取端的 gate）───────────────
//
// 同 `api_protocol_closure_sign_path.rs` 的理由：讀取端 gate 綠燈不代表寫入端
// `sign_closure` 真的寫得出 gate 會接受的列——尤其代理簽署這條新路徑，密碼驗證
// 與 `delegation_id` 綁定都是全新程式碼，必須有測試真的呼叫寫入端。

/// 建一個**密碼可用**的使用者（`sign_record_tx` 一定會驗密碼，`password_hash='fake'`
/// 會讓路徑在簽章寫入前就失敗）。
async fn seed_signer(app: &TestApp, role_code: Option<&str>) -> Uuid {
    let id = Uuid::new_v4();
    let hash = AuthService::hash_password(TEST_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("delegate-sign-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(&hash)
    .bind(format!("delegate-sign-{}", &Uuid::new_v4().to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    if let Some(role_code) = role_code {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
        )
        .bind(id)
        .bind(role_code)
        .execute(&app.db_pool)
        .await
        .expect("assign role");
    }
    id
}

#[derive(Debug, sqlx::FromRow)]
struct SigRow {
    signer_id: Uuid,
    delegation_id: Option<Uuid>,
    is_valid: bool,
}

/// 依使用者**實際被指派的角色**從 DB 讀出角色/權限快照，供需要走
/// `change_status_tx` 完整權限檢查的呼叫使用（單純硬編 permissions 會把權限矩陣
/// 繞過——矩陣哪天不再授予 `aup.protocol.close_own`，測試仍然全綠但正式環境結不了案）。
async fn actor_with_real_permissions(app: &TestApp, id: Uuid) -> ActorContext {
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
    assert!(
        permissions.iter().any(|p| p == "aup.protocol.close_own"),
        "測試前提不成立：使用者 {id} 的角色 {roles:?} 在 DB 裡沒有 aup.protocol.close_own"
    );
    ActorContext::User(CurrentUser {
        id,
        email: format!("{id}@test.local"),
        roles,
        permissions,
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

#[tokio::test]
#[serial]
async fn delegate_can_sign_pi_closure_slot_and_signature_carries_delegation_id() {
    let app = TestApp::spawn().await;
    let creator = seed_signer(&app, None).await;
    let sd = seed_signer(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_signer(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize")
    .id;

    // 代理人以自己的密碼簽 PI 那一欄——這是本測試要證明「寫得出來」的核心動作。
    let updated = protocol_closure_sign(
        &app.db_pool,
        &actor(delegate, &[]),
        protocol,
        ClosureSigner::Pi,
        delegate,
        Some(delegation_id),
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("代理人簽 PI 那一欄應成功");

    assert_eq!(updated.status.as_str(), "APPROVED", "只簽一邊不該結案");
    let pi_sig_id = updated
        .close_pi_signature_id
        .expect("PI 簽章欄應已寫入代理人簽的那張");

    let row = sqlx::query_as::<_, SigRow>(
        "SELECT signer_id, delegation_id, is_valid FROM electronic_signatures WHERE id = $1",
    )
    .bind(pi_sig_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("簽章列應該存在");

    assert_eq!(
        row.signer_id, delegate,
        "signer_id 必須是代理人本人，不是 PI 佔位值"
    );
    assert_eq!(
        row.delegation_id,
        Some(delegation_id),
        "簽章必須留下依哪筆授權代簽的證據，不能看起來像 PI 本人簽的"
    );
    assert!(row.is_valid);

    // ---- 回讀端也要接受：用真正寫出來的列跑一次 gate（不是另外 raw INSERT 的列）----
    let sd_hash = AuthService::hash_password(TEST_PASSWORD).expect("hash");
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
        .bind(sd)
        .bind(&sd_hash)
        .execute(&app.db_pool)
        .await
        .expect("give sd a real password for the second signature");
    let after_sd = protocol_closure_sign(
        &app.db_pool,
        &actor_with_real_permissions(&app, sd).await,
        protocol,
        ClosureSigner::StudyDirector,
        sd,
        None,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("SD 簽第二欄應成功");

    assert_eq!(
        after_sd.status.as_str(),
        "CLOSED",
        "雙簽齊備（代理人簽 PI + SD 本人簽 SD）應自動轉結案，證明代理簽章確實被 gate 接受"
    );
}

// ── SD 自任代理人：授權仍合法，但不得用來簽結案 PI 那一欄 ──────────
//
// `authorize_pi_delegate` 刻意允許「SD 自任代理人」（改由執秘/admin 核准）。
// 那個組合對安樂死核准、修正案寫入等「只需要一個有權責的人」的用途仍然成立，
// 但**結案雙簽要求兩人各自具結**（`dual_signature_ready` 條件 7）。若讓 SD 以
// 代理人身分簽下 PI 那一欄，PI 欄被佔住、gate 卻永遠回 false，計畫再也進不了
// `CLOSED`，脫困要撤銷授權 + 作廢簽章（人工修資料等級）。
//
// 下面兩支測試是一組，缺一不可：一支釘「擋得住」，一支釘「沒有連帶把合法用途
// 一起擋掉」。只留前者的話，日後有人把守衛上移到核准端也會全綠。

#[tokio::test]
#[serial]
async fn sd_as_own_delegate_cannot_sign_pi_closure_slot() {
    let app = TestApp::spawn().await;
    let creator = seed_signer(&app, None).await;
    let sd = seed_signer(&app, Some("EXPERIMENT_STAFF")).await;
    let staff = seed_user(&app, Some("IACUC_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // 前提：這個組合本身仍然核准得下去（不是靠核准端擋）。
    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(staff, &["IACUC_STAFF"]),
        protocol,
        sd,
        None,
    )
    .await
    .expect("執秘應可核准 SD 自任代理人")
    .id;

    let err = protocol_closure_sign(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        ClosureSigner::Pi,
        sd,
        Some(delegation_id),
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("SD 以代理人身分簽 PI 欄應被擋下");
    assert!(matches!(err, AppError::BusinessRule(_)), "{err:?}");

    // 最關鍵的一條：被擋下時**不可以留下半張簽章**。留下就等於把 PI 欄卡死，
    // 而 gate 又永遠不會齊備——那正是這個守衛要防的不可回復狀態。
    let pi_slot: Option<Uuid> =
        sqlx::query_scalar("SELECT close_pi_signature_id FROM protocols WHERE id = $1")
            .bind(protocol)
            .fetch_one(&app.db_pool)
            .await
            .expect("read protocol");
    assert!(
        pi_slot.is_none(),
        "被擋下時不得寫入 PI 欄簽章，否則要人工撤銷授權 + 作廢簽章才脫得了困"
    );
}

#[tokio::test]
#[serial]
async fn sd_as_own_delegate_still_usable_outside_closure() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let staff = seed_user(&app, Some("IACUC_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(staff, &["IACUC_STAFF"]),
        protocol,
        sd,
        None,
    )
    .await
    .expect("執秘應可核准 SD 自任代理人");

    // 修正案寫入權本來刻意不含 SD（見 access.rs），SD 取得這項權限只可能來自
    // 那筆代理授權——所以這條同時證明「授權真的生效」而不只是資料列存在。
    let sd_user = CurrentUser {
        id: sd,
        email: format!("{sd}@test.local"),
        roles: vec!["EXPERIMENT_STAFF".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let can_write = access::can_write_amendment(&app.db_pool, &sd_user, protocol)
        .await
        .expect("can_write_amendment");
    assert!(
        can_write,
        "結案雙簽的守衛只該擋結案那一個動作，不該讓 SD 自任代理人整個失效"
    );
}

// ── 簽章 tx 內重驗代理授權（TOCTOU）────────────────────────────────
//
// handler 解出授權的時間點與簽章落地的時間點之間有空隙，撤銷可以擠進去
// （`handlers/signature/protocol_closure.rs` 的查詢明文不加鎖，而 `sign_closure`
// 的 `FOR UPDATE` 讀的是 protocols，從頭到尾沒再看過 protocol_pi_delegates）。
// 疊上 `dual_signature_ready` 條件 6 刻意不檢查 `revoked_at`，撤銷後才簽出來的
// 那張會被永久當成有效——所以只能在寫入這一側關門。
//
// 下面兩支各驗守衛的一半：陳舊（已撤銷）與跨計畫。傳入「先前解出的」
// delegation_id 正是在模擬 handler 早一步解析、之後狀態才改變的真實時序。

#[tokio::test]
#[serial]
async fn revoked_delegation_cannot_sign_closure_even_if_resolved_earlier() {
    let app = TestApp::spawn().await;
    let creator = seed_signer(&app, None).await;
    let sd = seed_signer(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_signer(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // handler 在 T0 解出授權（此時確實生效中）
    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
    )
    .await
    .expect("authorize")
    .id;

    // T1：授權在簽章落地之前被撤銷
    ProtocolService::revoke_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        Some("測試：簽章落地前撤銷"),
    )
    .await
    .expect("revoke");

    // T2：帶著 T0 解出的那個 id 去簽——正是 handler 不加鎖會發生的事
    let err = protocol_closure_sign(
        &app.db_pool,
        &actor(delegate, &[]),
        protocol,
        ClosureSigner::Pi,
        delegate,
        Some(delegation_id),
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("已撤銷的授權不得簽出簽章");
    assert!(matches!(err, AppError::Forbidden(_)), "{err:?}");

    let pi_slot: Option<Uuid> =
        sqlx::query_scalar("SELECT close_pi_signature_id FROM protocols WHERE id = $1")
            .bind(protocol)
            .fetch_one(&app.db_pool)
            .await
            .expect("read protocol");
    assert!(
        pi_slot.is_none(),
        "撤銷後的代簽不得留下任何簽章——gate 條件 6 不看 revoked_at，留下就永遠算有效"
    );
}

#[tokio::test]
#[serial]
async fn delegation_from_another_protocol_cannot_sign_closure() {
    let app = TestApp::spawn().await;
    let creator = seed_signer(&app, None).await;
    let sd = seed_signer(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_signer(&app, None).await;
    let target = seed_external_pi_protocol(&app, creator, Some(sd)).await;
    let other = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // 授權只掛在 `other` 上
    let other_delegation = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        other,
        delegate,
        None,
    )
    .await
    .expect("authorize on other protocol")
    .id;

    let err = protocol_closure_sign(
        &app.db_pool,
        &actor(delegate, &[]),
        target,
        ClosureSigner::Pi,
        delegate,
        Some(other_delegation),
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("別份計畫的授權不得用來簽這份計畫");
    assert!(matches!(err, AppError::Forbidden(_)), "{err:?}");

    let pi_slot: Option<Uuid> =
        sqlx::query_scalar("SELECT close_pi_signature_id FROM protocols WHERE id = $1")
            .bind(target)
            .fetch_one(&app.db_pool)
            .await
            .expect("read protocol");
    assert!(pi_slot.is_none(), "跨計畫代簽不得留下任何簽章");
}
