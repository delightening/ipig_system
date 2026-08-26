//! `protocols.is_glp` 獨立欄位的回歸測試（migration 006 / 裁定 14）。
//!
//! 這一組守的是**判定來源本身**。舊實作直接讀
//! `working_content -> basic -> is_glp`，而那是設計上就可編輯的工作中內容——
//! 規則寫得再嚴，判定來源可被任意編輯的話，整組規則都只是裝飾。
//!
//! ⚠️ 舊實作**已經**防了單一 request 的繞過（取「更新後生效值」），
//! 擋不住的是跨 request：先送一個 request 關掉 is_glp（不帶 SD，於是 GLP 鎖
//! 那段根本不執行），再送第二個 request 換 SD。本檔的第一支測試就是為此而寫。

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
    .bind(format!("glp-test-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("glp-test-{role_code}"))
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

fn create_req(pi: Uuid, sd: Option<Uuid>, is_glp: bool) -> CreateProtocolRequest {
    CreateProtocolRequest {
        title: "is_glp 欄位測試計劃".to_string(),
        pi_user_id: Some(pi),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": is_glp } })),
        start_date: None,
        end_date: None,
        study_director_user_id: sd,
    }
}

fn update_content(is_glp: bool) -> UpdateProtocolRequest {
    UpdateProtocolRequest {
        title: None,
        working_content: Some(serde_json::json!({ "basic": { "is_glp": is_glp } })),
        start_date: None,
        end_date: None,
        study_director_user_id: None,
        version: None,
        source_form_version: None,
    }
}

/// 把計畫改成指定狀態。測試用捷徑，不走 status service 的完整審查流程——
/// 本檔受測的是 is_glp 的鎖定條件，不是狀態機轉換。
async fn force_status(app: &TestApp, id: Uuid, status: &str, import_pending: bool) {
    sqlx::query(
        "UPDATE protocols SET status = $2::protocol_status, import_pending = $3 WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(import_pending)
    .execute(&app.db_pool)
    .await
    .expect("force status");
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

/// 🔴 本 PR 的核心：GLP 的 SD 鎖讀的是欄位，不是 JSON。
///
/// 直接用 SQL 把 `working_content` 的 is_glp 竄改成 false（繞過應用層，
/// 等同「先送一個 request 關掉它」之後的資料狀態），欄位維持 true。
/// 此時換 SD 仍須被擋——判定來源若還是 JSON，這裡會放行。
#[tokio::test]
#[serial]
async fn glp_sd_lock_reads_column_not_working_content() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd_a), true),
        secretary,
    )
    .await
    .expect("create glp");
    assert!(p.is_glp, "建立 GLP 計畫時欄位就該是 true");

    sqlx::query(
        r#"UPDATE protocols
           SET working_content = jsonb_set(working_content, '{basic,is_glp}', 'false')
           WHERE id = $1"#,
    )
    .bind(p.id)
    .execute(&app.db_pool)
    .await
    .expect("tamper json");

    let req = UpdateProtocolRequest {
        title: None,
        working_content: None, // 不帶內容，只換 SD
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd_b),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect_err("JSON 被竄改成 false 但欄位仍為 true，換 SD 應被擋");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

/// 建立時就把表單的勾選寫進欄位——否則新計畫的欄位一律 false，
/// 與 working_content 立刻不一致，而不一致的那一刻就是規則失效的起點。
#[tokio::test]
#[serial]
async fn create_writes_is_glp_column() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let glp = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None, true), secretary)
        .await
        .expect("create glp");
    let non_glp = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, None, false),
        secretary,
    )
    .await
    .expect("create non-glp");

    assert!(glp.is_glp, "勾了 GLP 的計畫欄位要是 true");
    assert!(!non_glp.is_glp, "沒勾的要是 false");
}

/// 🔴 **跨 request 繞過的回歸測試——本檔最重要的一支。**
///
/// 這支測試的前身是「漏洞存在證明」：先前它會**通過**，證明
/// 「核准後鎖死」的設計堵不到真正的洞。洞在核准**之前**（DRAFT／補件中／補登中），
/// 而那些狀態下改 is_glp 曾被視為正常編輯。
///
/// 使用者 2026-08-25 裁定改為**雙向鎖定**後，第 1 步就會被擋，
/// 這支測試也就從「證明」變成「回歸測試」。
#[tokio::test]
#[serial]
async fn cross_request_bypass_is_blocked_at_step_one() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // GLP 計畫，已指派 SD=A，狀態 DRAFT（可編輯）
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd_a), true),
        secretary,
    )
    .await
    .expect("create glp");
    assert!(p.is_glp);

    // request 1：只關掉 is_glp，不帶 SD。
    // 這一步是整條繞過路徑的入口——擋住它，後面兩步就沒有意義。
    let scope = scope_for(&app, secretary, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &update_content(false))
        .await
        .expect_err("關掉 is_glp 應被擋——這是繞過路徑的第一步");
    assert!(
        matches!(&err, AppError::BusinessRule(m) if m.contains("GLP 屬性")),
        "應被 GLP 屬性鎖擋下（而不是別的 BusinessRule），實得：{err:?}"
    );

    // 欄位確實沒被動到，SD 也還是原本那位
    let (still_glp, still_sd): (bool, Option<Uuid>) =
        sqlx::query_as("SELECT is_glp, study_director_user_id FROM protocols WHERE id = $1")
            .bind(p.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("read back");
    assert!(still_glp, "被拒之後仍是 GLP");
    assert_eq!(still_sd, Some(sd_a), "SD 未被更動");

    // 補一刀：就算真的走到第 2 步（換 SD），GLP 鎖也會擋
    let req2 = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd_b),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &req2)
        .await
        .expect_err("GLP 案換 SD 本來就該被擋");
}

/// 雙向鎖：非 GLP 也不能改成 GLP（使用者 2026-08-25 裁定「都不能改」）。
///
/// ⚠️ 這個方向同樣要擋，否則「非 GLP 案先指派 SD → 翻成 GLP」等於用兩步繞過
/// 「GLP 案的 SD 必須從一開始就確定」這個規則。
#[tokio::test]
#[serial]
async fn non_glp_cannot_become_glp() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, None, false),
        secretary,
    )
    .await
    .expect("create non-glp");
    assert!(!p.is_glp);

    let scope = scope_for(&app, secretary, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &update_content(true))
        .await
        .expect_err("非 GLP 不可改成 GLP");
    assert!(
        matches!(&err, AppError::BusinessRule(m) if m.contains("GLP 屬性")),
        "應被 GLP 屬性鎖擋下，實得：{err:?}"
    );
}

/// 送出「與現值相同」的 is_glp 不該報錯——那等於沒改。
///
/// ⚠️ 這支測的是**不要過度阻擋**。前端編輯計畫時會整份送出 working_content，
/// 若只要帶到 is_glp 就拒絕，等於任何欄位都不能編輯，功能形同凍結。
#[tokio::test]
#[serial]
async fn same_value_is_not_a_change() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    for is_glp in [true, false] {
        let p = ProtocolService::create(
            &app.db_pool,
            &actor,
            &create_req(pi, None, is_glp),
            secretary,
        )
        .await
        .expect("create");
        let scope = scope_for(&app, secretary, p.id).await;
        let updated = ProtocolService::update(&app.db_pool, &actor, scope, &update_content(is_glp))
            .await
            .unwrap_or_else(|e| panic!("送相同值不該被擋（is_glp={is_glp}）：{e:?}"));
        assert_eq!(updated.is_glp, is_glp);
    }
}

/// 沒帶 working_content 的 request（例如只換 SD）不受 GLP 屬性鎖影響。
#[tokio::test]
#[serial]
async fn update_without_working_content_is_unaffected() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let sd_a = seed_user(&app, "EXPERIMENT_STAFF").await;
    let sd_b = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // 非 GLP，換 SD 應該照常可行
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(pi, Some(sd_a), false),
        secretary,
    )
    .await
    .expect("create");
    let req = UpdateProtocolRequest {
        title: Some("改個標題".to_string()),
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd_b),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    let updated = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect("非 GLP 換 SD 應成功");
    assert_eq!(updated.study_director_user_id, Some(sd_b));
    assert!(!updated.is_glp, "is_glp 不受影響");
}

/// 補登中（APPROVED + import_pending）同樣鎖住。
///
/// ⚠️ 這條值得單獨測：補登中是**可編輯**狀態，而實測正式庫多數計畫都在這個狀態。
/// 若鎖漏掉這裡，繞過路徑對絕大多數計畫依然通行。
#[tokio::test]
#[serial]
async fn import_pending_is_also_locked() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None, true), secretary)
        .await
        .expect("create glp");
    force_status(&app, p.id, "APPROVED", true).await;

    let scope = scope_for(&app, secretary, p.id).await;
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &update_content(false))
        .await
        .expect_err("補登中也不可改 is_glp");
    assert!(
        matches!(&err, AppError::BusinessRule(m) if m.contains("GLP 屬性")),
        "應被 GLP 屬性鎖擋下，實得：{err:?}"
    );
}

/// 🔴 字串 `"true"` 要跟 JSON 布林 `true` 得到同一個結論（CodeRabbit #25）。
///
/// migration 006 的回填用 `->> 'is_glp' = 'true'`，`->>` 回傳 text，
/// 所以 JSON 布林 `true` 與字串 `"true"` 在那裡是**同一件事**。
/// 若 Rust 端只認 `as_bool()`，同一份內容兩邊結論相反：欄位說非 GLP、
/// 而前端 `basic.is_glp ? …` 走 JS truthiness 會把字串 `"true"` 顯示成 GLP。
///
/// 前端型別是 `is_glp: boolean` 送不出字串，但 `working_content` 在後端是
/// 未驗證的 `serde_json::Value`——直接打 API 就進得來。
#[tokio::test]
#[serial]
async fn string_true_is_treated_as_glp() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let mut req = create_req(pi, None, false);
    req.working_content = Some(serde_json::json!({ "basic": { "is_glp": "true" } }));

    let p = ProtocolService::create(&app.db_pool, &actor, &req, secretary)
        .await
        .expect("create");

    let is_glp: bool = sqlx::query_scalar("SELECT is_glp FROM protocols WHERE id = $1")
        .bind(p.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query is_glp");
    assert!(
        is_glp,
        "字串 \"true\" 應與 migration 的 ->> 判準一致，視為 GLP"
    );
}

/// 不可辨識的值仍是「沒表態」，不是 false。
///
/// ⚠️ 這支釘住「不要為了寬容而多認」：判準是**與 migration 對齊**，
/// 而 migration 的 `= 'true'` 不認 `"TRUE"` / `"1"` / `"yes"`。
/// 多認就是在原本的分歧之外再造一條新的。
/// 回 `None` 而非 `Some(false)` 也很重要——`update` 的雙向鎖靠 `None`
/// 判斷「這次沒動 GLP」，若誤判成 `Some(false)`，GLP 案的任何編輯都會被鎖擋死。
#[tokio::test]
#[serial]
async fn unrecognised_value_is_not_a_statement() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // GLP 計畫
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None, true), secretary)
        .await
        .expect("create glp");

    // 帶一個認不得的 is_glp 值 → 視為沒表態，鎖不該觸發
    let req = UpdateProtocolRequest {
        title: Some("改個標題".to_string()),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": "YES" } })),
        start_date: None,
        end_date: None,
        study_director_user_id: None,
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect("認不得的值＝沒表態，不該被鎖擋");

    let is_glp: bool = sqlx::query_scalar("SELECT is_glp FROM protocols WHERE id = $1")
        .bind(p.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query is_glp");
    assert!(is_glp, "欄位不該被動到");
}

/// 🔴 複製要繼承來源的**欄位**，不從 working_content 重新推導。
///
/// 情境：來源是 migration 006 回填來的——欄位 `is_glp = true`，
/// 而 `working_content` 裡是字串 `"true"`（甚至根本沒有 is_glp）。
/// 舊實作從 JSON 推導，複本會拿到 false，等於複製一份「同一個計畫但不是 GLP」。
///
/// 而裁定 14 的雙向鎖對 DRAFT 一樣生效，複本的 is_glp 一設下去就再也改不了——
/// 錯了就得找管理員。
#[tokio::test]
#[serial]
async fn copy_inherits_is_glp_column_not_json() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let pi = seed_user(&app, "PI").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(pi, None, true), secretary)
        .await
        .expect("create glp");

    // 模擬 migration 回填後的資料形狀：欄位 true，JSON 裡沒有 is_glp。
    // 直接用 SQL 繞過應用層——這是既有資料的狀態，不是應用層造得出來的。
    sqlx::query(
        r#"UPDATE protocols
           SET working_content = jsonb_set(
                 working_content::jsonb, '{basic}', '{}'::jsonb
               )::json
           WHERE id = $1"#,
    )
    .bind(p.id)
    .execute(&app.db_pool)
    .await
    .expect("清掉 JSON 裡的 is_glp");

    let source_scope = access::Scoped::<access::ProtocolId>::authorize(
        &app.db_pool,
        &user_cu(secretary, &["IACUC_STAFF"]),
        p.id,
    )
    .await
    .expect("authorize copy source");
    let copy = ProtocolService::copy(&app.db_pool, &actor, source_scope, secretary)
        .await
        .expect("copy");

    let is_glp: bool = sqlx::query_scalar("SELECT is_glp FROM protocols WHERE id = $1")
        .bind(copy.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query copy is_glp");
    assert!(
        is_glp,
        "複本應繼承來源欄位的 true，即使 working_content 裡沒有 is_glp"
    );
}
