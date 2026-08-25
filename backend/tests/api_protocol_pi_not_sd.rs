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
