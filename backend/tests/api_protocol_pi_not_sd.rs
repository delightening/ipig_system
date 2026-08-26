//! PI 不可兼任同一計畫的 SD（裁定 16）。
//!
//! ⚠️ 這條規則不是形式上的職稱分離，而是**結案雙簽的正確性前提**：
//! 裁定 9 要求 PI 與 SD 各簽一次才能結案，同一人的話兩張簽章的 signer_id
//! 會是同一個 UUID——稽核鏈上看起來是「雙方認可」，實際是自簽自證。
//!
//! 本檔同時守住一個容易做錯的**放寬**：PI 沒有系統帳號時，
//! `protocols.pi_user_id` 記的是匯入者本人（佔位），拿它比對 SD 會誤擋
//! 「執秘替外部 PI 建計畫並自任 SD」——那是裁定 10／11 明確允許的操作。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateProtocolRequest, UpdateProtocolRequest};
use erp_backend::services::{access, ProtocolService};
use erp_backend::AppError;

async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("pisd-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("pisd-{role_code}"))
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

/// 同時具備多個角色的使用者（例如執秘本身也是 EXPERIMENT_STAFF）。
async fn add_role(app: &TestApp, user_id: Uuid, role_code: &str) {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("add role");
}

fn user_cu(id: Uuid, roles: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "actor@example.com".to_string(),
        roles: roles.iter().map(|r| r.to_string()).collect(),
        permissions: vec!["aup.protocol.edit".to_string()],
        jti: Uuid::new_v4().to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

fn user_actor(id: Uuid, roles: &[&str]) -> ActorContext {
    ActorContext::User(user_cu(id, roles))
}

fn create_req(pi: Option<Uuid>, sd: Option<Uuid>) -> CreateProtocolRequest {
    CreateProtocolRequest {
        title: "PI/SD 分離測試計劃".to_string(),
        pi_user_id: pi,
        working_content: Some(serde_json::json!({ "basic": { "is_glp": false } })),
        start_date: None,
        end_date: None,
        study_director_user_id: sd,
    }
}

async fn scope_for(
    app: &TestApp,
    actor_id: Uuid,
    protocol_id: Uuid,
) -> access::Scoped<access::ProtocolEdit> {
    access::Scoped::<access::ProtocolEdit>::authorize_update(
        &app.db_pool,
        &user_cu(actor_id, &["IACUC_STAFF"]),
        protocol_id,
        false,
    )
    .await
    .expect("authorize")
}

/// 🔴 核心：建立計畫時把 PI 指派為 SD → 擋。
#[tokio::test]
#[serial]
async fn create_rejects_pi_as_sd() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    // 同一人同時具 PI 與 EXPERIMENT_STAFF——若沒有本規則，他會通過所有既有檢查
    let pi = seed_user(&app, "PI").await;
    add_role(&app, pi, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let err = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(Some(pi), Some(pi)),
        secretary,
    )
    .await
    .expect_err("PI 兼任 SD 應被擋");
    assert!(
        matches!(&err, AppError::Validation(m) if m.contains("不可兼任")),
        "應是「不可兼任」的驗證錯誤（而非角色不符），實得：{err:?}"
    );
}

/// 🔴 update 換 SD 時把 PI 指派為 SD → 擋。
#[tokio::test]
#[serial]
async fn update_rejects_pi_as_sd() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    add_role(&app, pi, "EXPERIMENT_STAFF").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(Some(pi), Some(sd_a)),
        secretary,
    )
    .await
    .expect("create");

    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(pi), // 想把 SD 換成 PI 本人
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect_err("把 SD 換成 PI 應被擋");
    assert!(
        matches!(&err, AppError::Validation(m) if m.contains("不可兼任")),
        "應是「不可兼任」的驗證錯誤，實得：{err:?}"
    );

    // 確認 SD 沒被改掉
    let after: Option<Uuid> =
        sqlx::query_scalar("SELECT study_director_user_id FROM protocols WHERE id = $1")
            .bind(p.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("read back");
    assert_eq!(after, Some(sd_a), "被拒之後 SD 應維持原值");
}

/// 🔴 **不要過度阻擋**：PI 沒有系統帳號時，pi_user_id 是匯入者佔位，
/// 此時匯入者自任 SD 是合法的（裁定 10／11）。
///
/// 這支測試守的是「放寬」而不是「限制」。若日後有人把檢查改成比對
/// `pi_user_id.unwrap_or(created_by)`，這裡會紅——那個改法看起來更嚴謹，
/// 實際上會擋掉執秘的日常操作。
#[tokio::test]
#[serial]
async fn create_allows_creator_as_sd_when_pi_is_external() {
    let app = TestApp::spawn().await;
    // 執秘同時具 EXPERIMENT_STAFF（實際資料就是這樣：執秘本人是主要試驗人員）
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    add_role(&app, secretary, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // PI 留空 → pi_user_id 會退回 created_by（＝執秘），但那只是佔位
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(None, Some(secretary)),
        secretary,
    )
    .await
    .expect("外部 PI + 建立者自任 SD 應該可行");

    assert_eq!(p.pi_user_id, secretary, "PI 欄位是佔位值");
    assert_eq!(
        p.study_director_user_id,
        Some(secretary),
        "SD 就是建立者本人"
    );
}

/// 🔴 **create 也要用角色判準，不能只看 `req.pi_user_id` 有沒有填**
/// （CodeRabbit #26 第 2 輪指出，2026-08-26 修）。
///
/// 攻擊路徑：同時具 `PI` 與 `EXPERIMENT_STAFF` 的人，建立計畫時**不填**
/// `pi_user_id`、把自己設成 SD。第一版的 create 直接傳 `req.pi_user_id`（= None），
/// PI≠SD 檢查整個跳過，結果存進去的 `pi_user_id` 與 `study_director_user_id`
/// 都是他本人——**裁定 16 在資料落地的那一刻就被繞過了**。
///
/// ⚠️ 而 `update` 的啟發式**抓不到它**：那人有 PI 角色、不算佔位，
/// 所以 update 只會擋住之後的變更，違規狀態已經寫進去了。
/// 這正是「同一個規則在兩條路徑上判準不一致」的典型後果。
///
/// 2026-08-26 實測正式庫：同時具兩個角色的使用者 0 人，所以目前無法觸發。
/// 這支測試守的是「角色指派之後」——那是例行管理動作。
#[tokio::test]
#[serial]
async fn create_blocks_pi_role_creator_naming_self_as_sd() {
    let app = TestApp::spawn().await;
    // 同時具 PI（所以不是佔位）與 EXPERIMENT_STAFF（所以有資格當 SD）
    let pi = seed_user(&app, "PI").await;
    add_role(&app, pi, "EXPERIMENT_STAFF").await;
    let actor = user_actor(pi, &["PI"]);

    // 關鍵：pi_user_id 留空，SD 填自己
    let err = ProtocolService::create(&app.db_pool, &actor, &create_req(None, Some(pi)), pi)
        .await
        .expect_err("具 PI 角色者不填 pi_user_id、自任 SD，應被擋");

    assert!(
        matches!(&err, AppError::Validation(m) if m.contains("不可兼任")),
        "應是「不可兼任」的驗證錯誤，實得：{err:?}"
    );
}

/// 對照組：**沒有** PI 角色的建立者做同樣的事 → 放行。
///
/// 這支跟上面那支只差一個變因（建立者有沒有 PI 角色），
/// 用來證明擋下的原因確實是角色判準，不是「留空 pi_user_id 一律擋」——
/// 後者會擋掉執秘替外部 PI 建計畫並自任 SD，那是裁定 10／11 允許的日常操作。
#[tokio::test]
#[serial]
async fn create_allows_non_pi_role_creator_naming_self_as_sd() {
    let app = TestApp::spawn().await;
    let staff = seed_user(&app, "EXPERIMENT_STAFF").await;
    add_role(&app, staff, "IACUC_STAFF").await;
    let actor = user_actor(staff, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(None, Some(staff)), staff)
        .await
        .expect("無 PI 角色的建立者自任 SD 應該可行（外部 PI 佔位）");

    assert_eq!(p.study_director_user_id, Some(staff));
}

/// 🔴 **update 路徑也要放行佔位 PI**（CodeRabbit #26 指出，2026-08-26 修）。
///
/// 上面那支只守 create。實際流程更常見的是「先建計畫、SD 之後再指派」——
/// 而 update 拿不到 `req.pi_user_id`（PI 不可透過 update 變更），只能讀
/// `before.pi_user_id`，那個值對外部 PI 的計畫就是建立者的佔位。
/// 原本直接傳 `Some(before.pi_user_id)`，於是建立者之後想自任 SD 會被誤擋。
///
/// 正式庫實查（2026-08-26）：`pi_user_id = created_by` 且尚未指派 SD 的計畫共 3 筆，
/// 其中 2 筆的建立者具 EXPERIMENT_STAFF（＝擔任 SD 的必要角色）且**無 PI 角色**。
/// 也就是說這不是假想情境，是現存資料就會踩到的。
#[tokio::test]
#[serial]
async fn update_allows_creator_as_sd_when_pi_is_external() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    add_role(&app, secretary, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // 外部 PI（留空 → 佔位成建立者）且**先不指派 SD**
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(None, None), secretary)
        .await
        .expect("create");
    assert_eq!(p.pi_user_id, secretary, "PI 欄位是佔位值");
    assert!(p.study_director_user_id.is_none(), "建立時尚未指派 SD");

    // 之後才把建立者本人指派為 SD
    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(secretary),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect("佔位 PI 的計畫，建立者事後自任 SD 應該可行");

    let after: Option<Uuid> =
        sqlx::query_scalar("SELECT study_director_user_id FROM protocols WHERE id = $1")
            .bind(p.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("read back");
    assert_eq!(after, Some(secretary));
}

/// 🔴 **判別靠的是「有沒有 PI 角色」，不是「形狀」——這支守住那個區別。**
///
/// 真 PI 自己開自己的計畫時，`pi_user_id` 同樣等於 `created_by`，形狀跟上面那支
/// 一模一樣。若只用形狀判斷佔位，這種情形會被一起放過，而它正是裁定 16
/// 要擋的東西（正式庫既有那筆 PI=SD 就是這個形狀，差別在 PI 具 PI 角色）。
#[tokio::test]
#[serial]
async fn update_still_blocks_when_self_created_pi_has_pi_role() {
    let app = TestApp::spawn().await;
    // 具 PI 角色、同時具 EXPERIMENT_STAFF（否則會先被角色檢查擋掉，測不到重點）
    let pi = seed_user(&app, "PI").await;
    add_role(&app, pi, "EXPERIMENT_STAFF").await;
    let actor = user_actor(pi, &["PI"]);

    // PI 自己建自己的計畫 → pi_user_id == created_by == pi（形狀同佔位）
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(None, None), pi)
        .await
        .expect("create");
    assert_eq!(p.pi_user_id, pi);

    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(pi),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, pi, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect_err("具 PI 角色者自任 SD 仍應被擋——形狀相同，靠角色區分");
    assert!(
        matches!(&err, AppError::Validation(m) if m.contains("不可兼任")),
        "應是「不可兼任」，實得：{err:?}"
    );
}

/// 正常情況：PI 與 SD 是不同人 → 放行。
#[tokio::test]
#[serial]
async fn create_allows_distinct_pi_and_sd() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(Some(pi), Some(sd)),
        secretary,
    )
    .await
    .expect("PI 與 SD 不同人應可建立");
    assert_eq!(p.pi_user_id, pi);
    assert_eq!(p.study_director_user_id, Some(sd));
}

/// PI≠SD 的檢查要在角色檢查**之前**——否則錯誤訊息會誤導。
///
/// PI 通常是外部人員、本來就沒有 EXPERIMENT_STAFF 角色。若讓角色檢查先跑，
/// 使用者收到的是「此人不具試驗工作人員角色」，看不出真正的問題是「他是 PI」。
#[tokio::test]
#[serial]
async fn pi_check_precedes_role_check() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    // 這個 PI **沒有** EXPERIMENT_STAFF 角色——兩道檢查都會失敗，看先報哪個
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let err = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(Some(pi), Some(pi)),
        secretary,
    )
    .await
    .expect_err("應被擋");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("不可兼任"),
        "應先報「不可兼任」而非角色不符，實得：{msg}"
    );
    assert!(
        !msg.contains("EXPERIMENT_STAFF"),
        "不該讓角色檢查先報，那會誤導使用者，實得：{msg}"
    );
}
