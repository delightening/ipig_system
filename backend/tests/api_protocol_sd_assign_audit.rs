//! SD 指派／變更的專屬稽核事件（裁定 21，配套裁定 11 的自我指派標記）。
//!
//! # 為什麼需要專屬事件
//!
//! 沒有它的話，SD 變更只會落成通用的 `PROTOCOL_UPDATE`——稽核報表上跟改標題、
//! 改日期長得一模一樣，要靠人去比對 before/after 的 JSON 才知道改的是 SD。
//! 而 GLP 稽核會問「這份計畫的 SD 換過幾次、誰換的」。
//!
//! 實測（2026-08-25 正式庫）：`user_activity_logs` 裡 `changed_fields` 真的含
//! `study_director` 的只有 **1 筆**，其餘 153 筆是整份快照剛好帶到這個欄位。
//! 也就是說現況連「換過幾次」都得靠人工判讀。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateProtocolRequest, UpdateProtocolRequest};
use erp_backend::services::{access, ProtocolService};

async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("sdaud-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("sdaud-{role_code}"))
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

fn create_req(pi: Uuid, sd: Option<Uuid>) -> CreateProtocolRequest {
    CreateProtocolRequest {
        title: "SD 稽核事件測試計劃".to_string(),
        pi_user_id: Some(pi),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": false } })),
        start_date: None,
        end_date: None,
        study_director_user_id: sd,
    }
}

fn assign_sd(sd: Uuid) -> UpdateProtocolRequest {
    UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd),
        version: None,
        source_form_version: None,
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

/// 取出該計畫所有 SD_ASSIGNED 活動（含 extra_data）。
async fn sd_events(
    app: &TestApp,
    protocol_id: Uuid,
) -> Vec<(Option<String>, Option<String>, Option<serde_json::Value>)> {
    sqlx::query_as(
        r#"SELECT from_value, to_value, extra_data
           FROM protocol_activities
           WHERE protocol_id = $1 AND activity_type = 'SD_ASSIGNED'
           ORDER BY created_at"#,
    )
    .bind(protocol_id)
    .fetch_all(&app.db_pool)
    .await
    .expect("query activities")
}

/// 🔴 核心：換 SD 會產生專屬事件，帶 from／to。
#[tokio::test]
#[serial]
async fn sd_change_records_dedicated_event() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, Some(sd_a)), secretary)
        .await
        .expect("create");

    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &assign_sd(sd_b))
        .await
        .expect("換 SD");

    let events = sd_events(&app, p.id).await;
    assert_eq!(events.len(), 1, "應恰好一筆 SD_ASSIGNED 事件");
    let (from, to, extra) = &events[0];
    assert_eq!(
        from.as_deref(),
        Some(sd_a.to_string().as_str()),
        "from 應是舊 SD"
    );
    assert_eq!(
        to.as_deref(),
        Some(sd_b.to_string().as_str()),
        "to 應是新 SD"
    );
    let extra = extra.as_ref().expect("extra_data 不該是 NULL");
    assert_eq!(
        extra["assigned_by"],
        serde_json::json!(secretary),
        "要記誰指派的"
    );
    assert_eq!(extra["self_assigned"], serde_json::json!(false));
}

/// 🔴 **event_type 必須是 PROTOCOL_SD_ASSIGN，不能落到 PROTOCOL_ACTION。**
///
/// `history.rs` 的 `event_type_for` 有 `_ => "PROTOCOL_ACTION"` catch-all，
/// 是本流程唯一沒有編譯器保護的地方——忘了加對應不會報錯，事件會靜默變成
/// 通用類型，而那要等到有人去查稽核紀錄才會發現（通常是稽核當下）。
/// 這支測試就是那個 catch-all 的替代保護。
#[tokio::test]
#[serial]
async fn sd_assign_has_dedicated_event_type() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None), secretary)
        .await
        .expect("create");
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &assign_sd(sd))
        .await
        .expect("指派 SD");

    let event_types: Vec<String> = sqlx::query_scalar(
        r#"SELECT event_type FROM user_activity_logs
           WHERE entity_type = 'protocol' AND entity_id = $1
           ORDER BY created_at"#,
    )
    .bind(p.id)
    .fetch_all(&app.db_pool)
    .await
    .expect("query audit");

    assert!(
        event_types.iter().any(|e| e == "PROTOCOL_SD_ASSIGN"),
        "稽核事件應含 PROTOCOL_SD_ASSIGN，實得：{event_types:?}"
    );
    assert!(
        !event_types.iter().any(|e| e == "PROTOCOL_ACTION"),
        "不該落到通用的 PROTOCOL_ACTION，實得：{event_types:?}"
    );

    // 🔴 `changed_fields` 必須**恰好**只有 SD 這個欄位。
    //
    // 這是裁定 21 真正要買到的東西：現況查不出「SD 換過幾次」的原因不是
    // 沒有紀錄，而是 `PROTOCOL_UPDATE` 帶整份快照、`changed_fields` 裡
    // 塞滿一起變動的欄位，無法區分「改了 SD」與「改了一堆、SD 剛好在裡面」。
    // 若日後有人把 focused diff 換回 `DataDiff::compute(&before, &updated)`，
    // 編譯照樣過、事件照樣有，只有這個斷言會紅。
    let (changed, before_json, after_json): (
        Vec<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) = sqlx::query_as(
        r#"SELECT changed_fields, before_data, after_data
               FROM user_activity_logs
               WHERE entity_type = 'protocol' AND entity_id = $1
                 AND event_type = 'PROTOCOL_SD_ASSIGN'"#,
    )
    .bind(p.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("query SD audit row");

    assert_eq!(
        changed,
        vec!["study_director_user_id".to_string()],
        "changed_fields 應恰好只有 SD 欄位"
    );
    // 首次指派：before 的 SD 為 null、after 為新 SD
    assert_eq!(
        before_json.expect("before_data")["study_director_user_id"],
        serde_json::Value::Null
    );
    assert_eq!(
        after_json.expect("after_data")["study_director_user_id"],
        serde_json::json!(sd)
    );
}

/// 裁定 11：執秘自我指派要標記出來。
#[tokio::test]
#[serial]
async fn self_assignment_is_flagged() {
    let app = TestApp::spawn().await;
    // 執秘同時具 EXPERIMENT_STAFF（實際資料就是這樣）
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    sqlx::query("INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF' ON CONFLICT DO NOTHING")
        .bind(secretary)
        .execute(&app.db_pool)
        .await
        .expect("add role");
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None), secretary)
        .await
        .expect("create");
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &assign_sd(secretary))
        .await
        .expect("自我指派");

    let events = sd_events(&app, p.id).await;
    assert_eq!(events.len(), 1);
    let extra = events[0].2.as_ref().expect("extra_data");
    assert_eq!(
        extra["self_assigned"],
        serde_json::json!(true),
        "指派者＝被指派者時必須標記 self_assigned"
    );
}

/// 首次指派（原本沒有 SD）也要記，from 為 null。
#[tokio::test]
#[serial]
async fn first_assignment_is_recorded_with_null_from() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None), secretary)
        .await
        .expect("create");
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &assign_sd(sd))
        .await
        .expect("首次指派");

    let events = sd_events(&app, p.id).await;
    assert_eq!(events.len(), 1);
    assert!(events[0].0.is_none(), "首次指派的 from 應為 NULL");
    assert_eq!(events[0].1.as_deref(), Some(sd.to_string().as_str()));
}

/// ⚠️ **不要過度記錄**：SD 沒變的 update 不該產生事件。
///
/// 前端編輯計畫時會整份送出，`study_director_user_id` 每次都帶著同一個值。
/// 若照 req 有沒有帶來判斷，每改一次標題就多一筆假的「SD 變更」，
/// 稽核紀錄會被灌爆，真正的變更反而被淹沒。
#[tokio::test]
#[serial]
async fn unchanged_sd_records_no_event() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, Some(sd)), secretary)
        .await
        .expect("create");

    // 帶同一個 SD + 改標題
    let req = UpdateProtocolRequest {
        title: Some("改個標題".to_string()),
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect("update");

    let events = sd_events(&app, p.id).await;
    assert!(
        events.is_empty(),
        "SD 沒變不該產生事件，實得 {} 筆",
        events.len()
    );
}
