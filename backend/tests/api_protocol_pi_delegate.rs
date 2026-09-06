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

use chrono::{Duration, Utc};
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

// ── 授權到期（migration 010 的 expires_at）─────────────────────────
//
// 代理授權的典型情境是「PI 出國兩週」，但撤銷是純手動、沒有任何提醒機制。
// 少了期限，一筆為兩週開的授權會安靜地活到有人想起來為止——而它同時握有
// 結案簽署、安樂死核准/暫緩、修正案寫入、須知簽署全部五項權限。
//
// ⚠️ 期限**不追溯**：只回答「此刻還能不能用它做新的事」，不用來否定過去已做成的
// 行為（同 `revoked_at` 的原則，見 `dual_signature_ready` 條件 6）。

/// 直接把某筆授權的 expires_at 改成過去（模擬「時間走到了」，不必真的等）。
async fn expire_delegation(app: &TestApp, delegation_id: Uuid) {
    sqlx::query(
        "UPDATE protocol_pi_delegates SET expires_at = now() - interval '1 hour' WHERE id = $1",
    )
    .bind(delegation_id)
    .execute(&app.db_pool)
    .await
    .expect("expire delegation");
}

#[tokio::test]
#[serial]
async fn expired_delegation_loses_all_authority() {
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
        None,
    )
    .await
    .expect("authorize")
    .id;

    let delegate_user = CurrentUser {
        id: delegate,
        email: format!("{delegate}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };

    // 過期前：兩條授權判準都放行
    assert!(
        access::can_write_amendment(&app.db_pool, &delegate_user, protocol)
            .await
            .expect("can_write_amendment"),
        "未過期的代理人應有修正案寫入權"
    );
    assert!(
        access::can_sign_notice(&app.db_pool, protocol, delegate)
            .await
            .expect("can_sign_notice"),
        "未過期的代理人應可簽須知"
    );

    expire_delegation(&app, delegation_id).await;

    // 過期後：全部收回，而且不需要任何人動手撤銷
    assert!(
        !access::can_write_amendment(&app.db_pool, &delegate_user, protocol)
            .await
            .expect("can_write_amendment"),
        "過期後修正案寫入權應自動失效"
    );
    assert!(
        !access::can_sign_notice(&app.db_pool, protocol, delegate)
            .await
            .expect("can_sign_notice"),
        "過期後須知簽署權應自動失效"
    );
    assert!(
        access::active_pi_delegate_id(&app.db_pool, protocol, delegate)
            .await
            .expect("active_pi_delegate_id")
            .is_none(),
        "過期後不該再被解析成生效中代理人"
    );
}

/// `active_pi_delegate` 的 SELECT 投影必須涵蓋 `PiDelegateInfo` 的每一個欄位。
///
/// 這支查詢用的是 runtime 的 `sqlx::query_as::<_, PiDelegateInfo>`，不是編譯期巨集——
/// SELECT 少一欄**不會編譯失敗**，只會在解碼時回 `ColumnNotFound`。而
/// `GET /protocols/{id}` 一律呼叫它，所以少一欄的後果是「凡是有生效代理人的計畫
/// 全部打不開」。既有測試都走 `access::active_pi_delegate_id`（另一支查詢），
/// 這條路徑先前完全沒被覆蓋。
#[tokio::test]
#[serial]
async fn active_pi_delegate_projects_every_field() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // 兩個名字給成可區分的確定值：查詢是兩個 JOIN（du 取代理人、au 取核准人），
    // 兩邊接反了 id 仍然對得上，只有名字會互換。`seed_user` 的隨機名字驗不出這個。
    for (uid, name) in [(delegate, "代理甲"), (sd, "核准乙")] {
        sqlx::query("UPDATE users SET display_name = $2 WHERE id = $1")
            .bind(uid)
            .bind(name)
            .execute(&app.db_pool)
            .await
            .expect("set display_name");
    }

    let expires_at = Utc::now() + Duration::days(30);
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        Some("代理出國期間事務"),
        Some(expires_at),
    )
    .await
    .expect("authorize");

    let info = ProtocolService::active_pi_delegate(&app.db_pool, protocol)
        .await
        .expect("active_pi_delegate 必須能解碼——SELECT 漏欄會在這裡炸")
        .expect("剛核准的代理應為生效中");

    assert_eq!(info.delegate_user_id, delegate);
    assert_eq!(info.authorized_by, sd);
    assert_eq!(info.reason.as_deref(), Some("代理出國期間事務"));
    assert!(
        info.expires_at.is_some(),
        "設了到期日就必須帶回前端；漏掉會讓畫面顯示成「未設期限」，比沒有這個欄位更糟"
    );
    // 前端直接把這兩個字串顯示出來，接反了就是「誰授權誰」整個顛倒。
    assert_eq!(info.delegate_name, "代理甲", "delegate_name 必須取自代理人");
    assert_eq!(
        info.authorized_by_name, "核准乙",
        "authorized_by_name 必須取自核准人"
    );
}

#[tokio::test]
#[serial]
async fn authorize_rejects_expiry_in_the_past() {
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
        Some(chrono::Utc::now() - chrono::Duration::hours(1)),
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Validation(_))),
        "到期時間早於現在應被拒絕：{result:?}"
    );
    assert!(
        active_delegate_row(&app, protocol).await.is_none(),
        "被拒絕時不得留下授權列"
    );
}

#[tokio::test]
#[serial]
async fn expired_delegation_is_auto_revoked_when_approving_a_new_one() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let first = seed_user(&app, None).await;
    let second = seed_user(&app, None).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    let first_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        first,
        None,
        None,
    )
    .await
    .expect("authorize first")
    .id;
    expire_delegation(&app, first_id).await;

    // 「一份計畫一位生效代理人」的部分唯一索引以 revoked_at IS NULL 為準
    // （索引述詞不能用 now()），所以已過期但未撤銷的列仍佔著位置。
    // 核准新代理人時應自動把它關掉，而不是逼 SD 先手動撤銷一筆早就無效的紀錄。
    let second_row = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        second,
        None,
        None,
    )
    .await
    .expect("已過期的舊授權不應擋住新的核准");

    assert_eq!(
        active_delegate_row(&app, protocol).await.map(|(_, d)| d),
        Some(second),
        "生效中的應該是新代理人"
    );

    let (revoked_at, revoked_reason): (Option<chrono::DateTime<chrono::Utc>>, Option<String>) =
        sqlx::query_as(
            "SELECT revoked_at, revoked_reason FROM protocol_pi_delegates WHERE id = $1",
        )
        .bind(first_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("read first delegation");
    assert!(revoked_at.is_some(), "過期的舊授權應被自動撤銷");
    assert!(
        revoked_reason.is_some_and(|r| r.contains("到期")),
        "自動撤銷要留下可歸責的理由，不是靜默關閉"
    );
    assert_ne!(second_row.id, first_id);
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
        None,
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

// ── 修正案寫入也要留下代簽證據（migration 010）───────────────────
//
// `can_write_amendment` 放行代理人，但 `amendments` 的 created_by / submitted_by
// 與 `euthanasia_appeals.pi_user_id` 踩到同一個坑：本次改動之前它們必然是計畫 PI
// （判準只認 admin 與 PI），之後可能是代理人。而變更申請的簽章是**審查方的決定簽**，
// 不是提交方的簽章，所以借不到 `electronic_signatures.delegation_id` 那條證據鏈。

async fn amendment_delegation_ids(
    app: &TestApp,
    amendment_id: Uuid,
) -> (Option<Uuid>, Option<Uuid>) {
    sqlx::query_as::<_, (Option<Uuid>, Option<Uuid>)>(
        "SELECT created_delegation_id, submitted_delegation_id FROM amendments WHERE id = $1",
    )
    .bind(amendment_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("read amendment")
}

async fn make_amendment(
    app: &TestApp,
    protocol: Uuid,
    writer: Uuid,
    writer_user: &CurrentUser,
    title: &str,
) -> erp_backend::models::Amendment {
    use erp_backend::models::CreateAmendmentRequest;
    use erp_backend::services::AmendmentService;

    // `seed_external_pi_protocol` 不設 iacuc_no，而變更申請編號產生器要求要有。
    // 這是 fixture 的附帶前提，不是本測試要驗的東西，所以就地補上。
    sqlx::query("UPDATE protocols SET iacuc_no = COALESCE(iacuc_no, $2) WHERE id = $1")
        .bind(protocol)
        .bind(format!("IACUC-DLG-{}", &protocol.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("set iacuc_no");

    let scope =
        access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, writer_user, protocol)
            .await
            .expect("authorize amendment write");
    let delegation = access::amendment_writer_delegation(&app.db_pool, writer_user, protocol)
        .await
        .expect("resolve delegation");
    AmendmentService::create(
        &app.db_pool,
        scope,
        &CreateAmendmentRequest {
            protocol_id: protocol,
            title: title.to_string(),
            description: None,
            change_items: None,
            changes_content: None,
        },
        writer,
        delegation,
    )
    .await
    .expect("create amendment")
}

#[tokio::test]
#[serial]
async fn amendment_created_by_delegate_records_delegation_evidence() {
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
        None,
    )
    .await
    .expect("authorize")
    .id;

    let delegate_user = CurrentUser {
        id: delegate,
        email: format!("{delegate}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let amendment =
        make_amendment(&app, protocol, delegate, &delegate_user, "代理人建立的變更").await;

    assert_eq!(
        amendment_delegation_ids(&app, amendment.id).await.0,
        Some(delegation_id),
        "代理人建立的變更申請必須綁上那筆授權，否則稽核上看起來就是 PI 本人提的"
    );
}

#[tokio::test]
#[serial]
async fn amendment_created_in_person_leaves_delegation_null() {
    let app = TestApp::spawn().await;
    let creator = seed_user(&app, None).await;
    let sd = seed_user(&app, Some("EXPERIMENT_STAFF")).await;
    let protocol = seed_external_pi_protocol(&app, creator, Some(sd)).await;

    // creator 就是借位的 pi_user_id，本人有資格；即使同時也被指定為代理人，
    // 本人身分優先，不得標成代簽（假的可歸責資訊比沒有更糟）。
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        creator,
        None,
        None,
    )
    .await
    .expect("authorize");

    let pi_user = CurrentUser {
        id: creator,
        email: format!("{creator}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };
    let amendment = make_amendment(&app, protocol, creator, &pi_user, "PI 本人建立的變更").await;

    assert_eq!(
        amendment_delegation_ids(&app, amendment.id).await.0,
        None,
        "本人有資格時一律以個人名義落帳"
    );
}

// ── 修正案寫入：解析與寫入之間的競態（待決 53.3 的裁定）─────────────
//
// handler 解析授權時沒有對該列下鎖，所以「撤銷交易在解出之後、寫入之前 commit」
// 是成立的時序。真的發生時，`created_delegation_id` / `submitted_delegation_id`
// 會指向一筆當下已失效的授權——那是假的可歸責證據，比不記錄更糟。
//
// 下面四條把那個視窗做成確定性的：先取得 scope 與 delegation（模擬 handler 解析
// 完成），**接著**才讓授權失效，然後才呼叫 service。這不是人為刁難——真實時序
// 就是這樣，只是視窗窄。

enum Invalidate {
    Revoke,
    Expire,
}

async fn invalidate_delegation(
    app: &TestApp,
    sd: Uuid,
    protocol: Uuid,
    delegation_id: Uuid,
    how: Invalidate,
) {
    match how {
        Invalidate::Revoke => {
            ProtocolService::revoke_pi_delegate(
                &app.db_pool,
                &actor(sd, &["EXPERIMENT_STAFF"]),
                protocol,
                Some("測試：解析之後撤銷"),
            )
            .await
            .expect("revoke");
        }
        Invalidate::Expire => expire_delegation(app, delegation_id).await,
    }
}

/// 佈場：外部 PI 計畫 + 一位生效中的代理人，並把 iacuc_no 補上（編號產生器要求）。
/// 回傳 (creator, sd, delegate, protocol, delegation_id, delegate_user)。
async fn seed_delegate_amendment_scene(
    app: &TestApp,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, CurrentUser) {
    let creator = seed_user(app, None).await;
    let sd = seed_user(app, Some("EXPERIMENT_STAFF")).await;
    let delegate = seed_user(app, None).await;
    let protocol = seed_external_pi_protocol(app, creator, Some(sd)).await;

    sqlx::query("UPDATE protocols SET iacuc_no = COALESCE(iacuc_no, $2) WHERE id = $1")
        .bind(protocol)
        .bind(format!("IACUC-TOC-{}", &protocol.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("set iacuc_no");

    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &actor(sd, &["EXPERIMENT_STAFF"]),
        protocol,
        delegate,
        None,
        None,
    )
    .await
    .expect("authorize")
    .id;

    let delegate_user = CurrentUser {
        id: delegate,
        email: format!("{delegate}@test.local"),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    };

    (
        creator,
        sd,
        delegate,
        protocol,
        delegation_id,
        delegate_user,
    )
}

async fn amendment_count(app: &TestApp, protocol: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM amendments WHERE protocol_id = $1")
        .bind(protocol)
        .fetch_one(&app.db_pool)
        .await
        .expect("count amendments")
}

async fn create_after_invalidation(app: &TestApp, how: Invalidate) {
    use erp_backend::models::CreateAmendmentRequest;
    use erp_backend::services::AmendmentService;

    let (_creator, sd, delegate, protocol, delegation_id, delegate_user) =
        seed_delegate_amendment_scene(app).await;

    // ── handler 的解析步驟（此刻授權有效）
    let scope =
        access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &delegate_user, protocol)
            .await
            .expect("authorize amendment write");
    let delegation = access::amendment_writer_delegation(&app.db_pool, &delegate_user, protocol)
        .await
        .expect("resolve delegation");
    assert_eq!(
        delegation,
        Some(delegation_id),
        "前提：解析當下授權有效，否則這條測不到競態"
    );

    // ── 競態視窗：解析之後、寫入之前失效
    invalidate_delegation(app, sd, protocol, delegation_id, how).await;

    let before = amendment_count(app, protocol).await;
    let result = AmendmentService::create(
        &app.db_pool,
        scope,
        &CreateAmendmentRequest {
            protocol_id: protocol,
            title: "解析後授權才失效".to_string(),
            description: None,
            change_items: None,
            changes_content: None,
        },
        delegate,
        delegation,
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "授權在寫入前已失效，不得建立——實際：{result:?}"
    );
    assert_eq!(
        amendment_count(app, protocol).await,
        before,
        "被拒的建立不得留下任何一列（交易要整個回滾）"
    );
}

#[tokio::test]
#[serial]
async fn amendment_create_rejects_delegation_revoked_after_resolution() {
    let app = TestApp::spawn().await;
    create_after_invalidation(&app, Invalidate::Revoke).await;
}

#[tokio::test]
#[serial]
async fn amendment_create_rejects_delegation_expired_after_resolution() {
    let app = TestApp::spawn().await;
    create_after_invalidation(&app, Invalidate::Expire).await;
}

async fn submit_after_invalidation(app: &TestApp, how: Invalidate) {
    use erp_backend::services::AmendmentService;

    let (_creator, sd, delegate, protocol, delegation_id, delegate_user) =
        seed_delegate_amendment_scene(app).await;

    // 先在授權有效時把草稿建起來——這一條要測的是送審，不是建立。
    let amendment = make_amendment(app, protocol, delegate, &delegate_user, "待送審的變更").await;

    let scope =
        access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &delegate_user, protocol)
            .await
            .expect("authorize amendment write");
    let delegation = access::amendment_writer_delegation(&app.db_pool, &delegate_user, protocol)
        .await
        .expect("resolve delegation");
    assert_eq!(delegation, Some(delegation_id), "前提：解析當下授權有效");

    invalidate_delegation(app, sd, protocol, delegation_id, how).await;

    let result =
        AmendmentService::submit(&app.db_pool, scope, amendment.id, delegate, delegation).await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "授權在寫入前已失效，不得送審——實際：{result:?}"
    );

    // 狀態必須留在 DRAFT，且不得留下版本快照或狀態歷程的孤兒。
    let (status, submitted_delegation): (String, Option<Uuid>) = sqlx::query_as(
        "SELECT status::text, submitted_delegation_id FROM amendments WHERE id = $1",
    )
    .bind(amendment.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("read amendment");
    assert_eq!(status, "DRAFT", "被拒的送審不得改動狀態");
    assert_eq!(submitted_delegation, None, "不得寫入已失效的授權作為證據");

    let versions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM amendment_versions WHERE amendment_id = $1",
    )
    .bind(amendment.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count versions");
    assert_eq!(versions, 0, "送審被拒卻留下版本快照＝交易沒有整個回滾");
}

#[tokio::test]
#[serial]
async fn amendment_submit_rejects_delegation_revoked_after_resolution() {
    let app = TestApp::spawn().await;
    submit_after_invalidation(&app, Invalidate::Revoke).await;
}

#[tokio::test]
#[serial]
async fn amendment_submit_rejects_delegation_expired_after_resolution() {
    let app = TestApp::spawn().await;
    submit_after_invalidation(&app, Invalidate::Expire).await;
}

/// `submit` 讀狀態時必須持有列鎖（CodeRabbit #53 第六輪）。
///
/// 交易化只保證「這批寫入同生同滅」，不保證「讀到的狀態還算數」。少了
/// `FOR UPDATE`，狀態守衛讀到的 `Draft` 可能在 UPDATE 之前就被別人改掉，
/// 而那個 UPDATE 只 match `id`、不帶狀態條件，於是照樣寫成 `Submitted`，
/// 並再產一份版本快照與狀態歷程——守衛看似擋著，實際上擋不住。
///
/// ⚠️ **不用 `tokio::join!` 兩個 submit**：那樣寫過（第一版），mutation 存活。
/// current-thread runtime 下兩個 future 只是交錯，未必在關鍵區間重疊——
/// 第一個常常一路跑完，第二個才開始，於是守衛正常擋下、測試假通過。
/// 改成由測試自己持有鎖、自己決定何時放開，時序就不再靠排程碰運氣：
///   1. `tx1` 鎖住該列並**不** commit
///   2. 另一個 task 呼叫 `submit`——有鎖版會停在它的 `FOR UPDATE`；
///      無鎖版的普通 SELECT 不受阻，直接讀到尚未變更的 `Draft`
///   3. `tx1` 把狀態推進到 `SUBMITTED` 後 commit
///   4. 有鎖版這時才讀到 `SUBMITTED`，被守衛擋下；
///      無鎖版早就過了守衛，UPDATE 一等到鎖就寫下去
#[tokio::test]
#[serial]
async fn submit_reads_status_under_row_lock() {
    use erp_backend::services::AmendmentService;
    use std::time::Duration;

    let app = TestApp::spawn().await;
    let (_creator, _sd, delegate, protocol, _delegation_id, delegate_user) =
        seed_delegate_amendment_scene(&app).await;
    let amendment = make_amendment(&app, protocol, delegate, &delegate_user, "並發送審").await;
    let amendment_id = amendment.id;

    let scope =
        access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &delegate_user, protocol)
            .await
            .expect("authorize amendment write");

    // 1. 先鎖住該列，不放。
    let mut tx1 = app.db_pool.begin().await.expect("begin tx1");
    sqlx::query("SELECT id FROM amendments WHERE id = $1 FOR UPDATE")
        .bind(amendment_id)
        .fetch_one(&mut *tx1)
        .await
        .expect("lock row in tx1");

    // 2. 讓 submit 在鎖被持有期間開始。
    let pool = app.db_pool.clone();
    let handle = tokio::spawn(async move {
        AmendmentService::submit(&pool, scope, amendment_id, delegate, None).await
    });
    tokio::time::sleep(Duration::from_millis(400)).await;

    // 3. 在 submit 還卡著（或已讀過舊值）時把狀態推進，然後放開鎖。
    sqlx::query("UPDATE amendments SET status = 'SUBMITTED'::amendment_status WHERE id = $1")
        .bind(amendment_id)
        .execute(&mut *tx1)
        .await
        .expect("advance status in tx1");
    tx1.commit().await.expect("commit tx1");

    // 4. 有鎖才會讀到已提交的新狀態並被守衛擋下。
    let result = handle.await.expect("join submit task");
    assert!(
        result.is_err(),
        "狀態已被別的交易推進到 SUBMITTED，這次 submit 必須讀到新狀態並被擋下；\
         成功代表它讀的是過期的 DRAFT（少了 FOR UPDATE）——實際：{result:?}"
    );

    // 而且不得留下任何送審副作用。
    let versions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM amendment_versions WHERE amendment_id = $1",
    )
    .bind(amendment_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count versions");
    assert_eq!(versions, 0, "被擋下的 submit 不該留下版本快照");
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

/// 同一個人同時是 `pi_user_id` 與 `study_director_user_id` 時，**本人直簽**也要擋。
///
/// 這條補的是上一支測試涵蓋不到的入口（CodeRabbit #53 第六輪）：守衛原本多了一個
/// `delegation_id.is_some()`，只擋代理那條路；但 `dual_signature_ready` 的
/// `pi_signer_authorized` 第一分支是 `pi_signer == pi_user_id`——本人直簽根本不需要
/// 授權，`delegation_id` 是 `None`，守衛整個失效。於是繞遠路的被擋、最直接的放行。
///
/// 後果與代理那條完全相同：PI 欄被佔住、條件 7（兩簽不同人）永遠回 false，
/// 計畫再也進不了 `CLOSED`。
#[tokio::test]
#[serial]
async fn sd_who_is_also_pi_cannot_sign_pi_closure_slot_in_person() {
    let app = TestApp::spawn().await;
    let both = seed_signer(&app, Some("EXPERIMENT_STAFF")).await;
    // pi_user_id 與 study_director_user_id 指向同一人（存量資料可能長成這樣，
    // 見 `dual_signature_ready` 條件 7 的註解）。
    let protocol = seed_external_pi_protocol(&app, both, Some(both)).await;

    let err = protocol_closure_sign(
        &app.db_pool,
        &actor(both, &["EXPERIMENT_STAFF"]),
        protocol,
        ClosureSigner::Pi,
        both,
        None, // ← 本人直簽，沒有代理授權
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("同時是 SD 的人本人直簽 PI 欄，一樣要被擋下");
    assert!(matches!(err, AppError::BusinessRule(_)), "{err:?}");

    let pi_slot: Option<Uuid> =
        sqlx::query_scalar("SELECT close_pi_signature_id FROM protocols WHERE id = $1")
            .bind(protocol)
            .fetch_one(&app.db_pool)
            .await
            .expect("read protocol");
    assert!(
        pi_slot.is_none(),
        "被擋下時不得寫入 PI 欄簽章——寫下去就是同一個不可回復的卡死狀態"
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
