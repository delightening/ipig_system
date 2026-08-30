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
use erp_backend::services::{access, ProtocolService, UserService};
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

/// 🔴 **已知缺口（CodeRabbit #26 第 3 輪指出，2026-08-27 查證後保留）**
///
/// 角色是**可變**的，而佔位判別讀的是「此刻有沒有 PI 角色」。所以計畫建立之後
/// 才改角色，判別結果就會跟著變——本支與下一支各釘住一個方向。
///
/// 這個方向：**真 PI 事後失去 PI 角色 → 閘對他失效（fail open）**。
/// 他自任 SD 會被放行，而裁定 16 本來要擋這件事。
///
/// ⚠️ 為什麼不修：CodeRabbit 建議「建立時存下不可變的外部 PI 標記」，
/// 那是加 schema 欄位，而使用者 2026-08-26 已明確裁定**用角色啟發式、不加欄位**
/// （理由：既有資料沒有 ground truth，加了欄位也只能用同一套啟發式回填，
/// 對現存計畫的精確度完全一樣）。schema 變更屬必問項，不由本 PR 自行決定。
///
/// 所以這支**斷言現況而非期望**——把缺口變成明寫的、有測試釘住的東西，
/// 而不是沒人知道的。日後若改用欄位，這支會紅，那正是它該做的事。
#[tokio::test]
#[serial]
async fn known_gap_real_pi_losing_pi_role_makes_guard_fail_open() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "PI").await;
    add_role(&app, pi, "EXPERIMENT_STAFF").await;
    let actor = user_actor(pi, &["PI"]);

    // 具 PI 角色時建立自己的計畫（此時若指派自己為 SD 會被擋——見
    // update_still_blocks_when_self_created_pi_has_pi_role）
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(None, None), pi)
        .await
        .expect("create");
    assert_eq!(p.pi_user_id, pi);

    // 事後被拔掉 PI 角色（人事異動、角色重整都會發生）
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = (SELECT id FROM roles WHERE code = 'PI')")
        .bind(pi)
        .execute(&app.db_pool)
        .await
        .expect("remove PI role");

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
    ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect(
            "已知缺口：失去 PI 角色後，同一個人被判成佔位 → 自任 SD 被放行。\
             這支斷言的是現況，不是期望——若這裡改成會擋，表示判別方式換了，請一併更新註解",
        );
}

/// 🔴 **已知缺口的反方向：佔位建立者事後取得 PI 角色 → 被誤擋（fail closed）**
///
/// 執秘替外部 PI 建計畫（`pi_user_id` 佔位成自己）、之後才自任 SD，是裁定 10／11
/// 允許的日常操作。但他若在這中間取得 PI 角色，佔位判別就不再成立，
/// `effective_pi` 變成他本人 → PI == SD → **原本合法的指派被擋下**。
///
/// ⚠️ 影響範圍已量過，比初看小：
/// - 這道閘**只在 `req.study_director_user_id` 為 `Some` 時才跑**（`core.rs` L1260），
///   所以不會擋掉該計畫的其他欄位更新，只擋「設定 SD」這個動作。
/// - 2026-08-27 正式庫實查：**同時具 PI 與 EXPERIMENT_STAFF 角色的使用者 0 人**，
///   目前無法觸發。但角色指派是例行管理動作，數字隨時會變。
/// - 可繞過：指派別人當 SD，或把 PI 欄位改成真正的外部 PI。不是死鎖。
#[tokio::test]
#[serial]
async fn known_gap_placeholder_creator_gaining_pi_role_gets_blocked() {
    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    add_role(&app, secretary, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let p = ProtocolService::create(&app.db_pool, &actor, &create_req(None, None), secretary)
        .await
        .expect("create");
    assert_eq!(p.pi_user_id, secretary, "PI 欄位是佔位值");
    assert!(p.study_director_user_id.is_none());

    // 事後取得 PI 角色
    add_role(&app, secretary, "PI").await;

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
    let err = ProtocolService::update(&app.db_pool, &actor, scope, &req)
        .await
        .expect_err(
            "已知缺口：取得 PI 角色後，同一筆佔位資料被重新判定成真 PI → 自任 SD 被擋。\
             這支斷言的是現況，不是期望",
        );
    assert!(
        matches!(&err, AppError::Validation(m) if m.contains("不可兼任")),
        "應是「不可兼任」，實得：{err:?}"
    );
}

/// 🔴 **指派 SD 與停用帳號必須序列化**（CodeRabbit #27 第 3 輪指出，修在 #26）。
///
/// 競態：指派端讀到 `is_active = true`（無鎖）→ 停用端鎖住 users、檢查「有沒有
/// 未結案 GLP 案以他為 SD」（此刻還沒寫入）→ 通過 → 停用 → 指派端才寫入。
/// 結果是**已停用的人成為未結案 GLP 計畫的 SD，而兩邊的閘都通過了**。
///
/// # 這支測的是「我們的函數有沒有取鎖」，不是 Postgres 的鎖語意
///
/// 做法：外部先持有 `users` 那一列的鎖，然後呼叫真正的
/// `ProtocolService::update`，斷言它卡住。
///
/// # 🔴 blocker 為什麼用 `FOR NO KEY UPDATE` 而不是 `FOR UPDATE`
///
/// 第一版用 `FOR UPDATE`，**mutation 驗證證明那樣寫沒有鑑別力**：
/// 把 `validate_and_authorize_sd` 裡的 `FOR SHARE` 整句拿掉，測試**仍然綠**。
///
/// 原因是 `protocols.study_director_user_id` 有 FK 指向 `users`，
/// 而 Postgres 在寫入 FK 參照時會自動對被參照列取 `FOR KEY SHARE`——
/// 那把鎖與 `FOR UPDATE` 衝突。所以 update 其實是**卡在最後的寫入**，
/// 跟驗證階段有沒有取鎖完全無關。測試看起來在測 A，實際上在測 B。
///
/// `FOR NO KEY UPDATE` 剛好切開這兩者：
///
/// | blocker 持有 | 與驗證的 `FOR SHARE` | 與 FK 寫入的 `FOR KEY SHARE` |
/// |---|---|---|
/// | `FOR UPDATE` | 衝突 | **也衝突** ← 分不出來 |
/// | `FOR NO KEY UPDATE` | 衝突 | **不衝突** ← 分得出來 |
///
/// 所以現在：有那句 `FOR SHARE` → 卡住（綠）；拿掉 → 直接跑完（紅）。已實測兩邊。
///
/// ⚠️ 附帶修正了對這個競態的理解：實際的停用流程用的是 `FOR UPDATE`，
/// 所以 FK 那把鎖**確實**提供了一部分保護——但只在兩個交易時間重疊時。
/// 若停用端在指派端「驗證完、還沒寫入」的空檔整個 commit 完畢，FK 鎖無人可擋，
/// 競態照樣成立。`FOR SHARE` 把取鎖時機提前到驗證那一刻，才真正關掉它。
///
/// 第二段（放鎖後應成功）是必要的對照：沒有它的話，`update` 因為**任何**原因失敗
/// 都會讓第一段通過，測試就變成「只要它慢或壞掉就算過」。
#[tokio::test]
#[serial]
async fn sd_assignment_blocks_while_user_row_is_locked_for_deactivation() {
    use std::time::Duration;

    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    // 指定 PI 為執秘、SD 先留空，之後才指派（走 update 那條路徑）
    let p = ProtocolService::create(
        &app.db_pool,
        &actor,
        &create_req(Some(secretary), None),
        secretary,
    )
    .await
    .expect("create");

    let req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd),
        version: None,
        source_form_version: None,
    };

    // ── 模擬「停用流程正持有該 users 列」──鎖的強度見上面的表 ──
    let mut blocker = app.db_pool.begin().await.expect("begin blocker tx");
    sqlx::query("SELECT is_active FROM users WHERE id = $1 FOR NO KEY UPDATE")
        .bind(sd)
        .fetch_one(&mut *blocker)
        .await
        .expect("blocker 取得 users 的 FOR NO KEY UPDATE");

    // 此時指派應該卡在 validate_and_authorize_sd 的 FOR SHARE 上。
    // ⚠️ 800ms 是「明顯超過正常耗時」而非精確門檻——正常路徑實測是毫秒級，
    // 卡住時則會一直等到 blocker 結束。用 timeout 是為了不讓測試整個掛死。
    let scope = scope_for(&app, secretary, p.id).await;
    let blocked = tokio::time::timeout(
        Duration::from_millis(800),
        ProtocolService::update(&app.db_pool, &actor, scope, &req),
    )
    .await;
    assert!(
        blocked.is_err(),
        "users 列被 FOR NO KEY UPDATE 持有時，指派 SD 應該卡住——\
         沒卡住表示 validate_and_authorize_sd 沒有對該列取 FOR SHARE，競態仍在"
    );

    // ── 放掉鎖，同一個指派應該成功 ──
    blocker.rollback().await.expect("rollback blocker");

    let scope2 = scope_for(&app, secretary, p.id).await;
    tokio::time::timeout(
        Duration::from_secs(10),
        ProtocolService::update(&app.db_pool, &actor, scope2, &req),
    )
    .await
    .expect("放鎖後不該再逾時")
    .expect("放鎖後指派應成功——證明上面卡住的原因是那把鎖，不是別的錯誤");

    let after: Option<Uuid> =
        sqlx::query_scalar("SELECT study_director_user_id FROM protocols WHERE id = $1")
            .bind(p.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("read back");
    assert_eq!(after, Some(sd));
}

/// CodeRabbit #26 第 4 輪建議：既有兩支測試各自只驗證半邊——上面那支驗證
/// 「指派端會等 users 列的鎖」，`api_glp_sd_disable_guard.rs` 驗證「三條停用
/// 路徑都會呼叫 GLP SD 防護」。沒有一支真正同時跑**真的指派**（`ProtocolService::update`）
/// 與**真的停用**（`UserService::deactivate_self`），驗證完整時序：指派先
/// commit、卡住等待的停用取得鎖之後正確被拒。本測試補上這個端對端情境。
///
/// 手法：第三方用 `FOR SHARE` 人工卡住時序——`FOR SHARE` 與指派端自己的
/// `FOR SHARE`（`validate_and_authorize_sd`）相容、不會擋到它，但與停用端的
/// `FOR UPDATE` 互斥、會擋住它。這樣才能保證停用端**確實**在指派 commit
/// 之前就已經卡在鎖上，而不是純粹兩個 async task 恰好跑出這個順序。
#[tokio::test]
#[serial]
async fn end_to_end_pending_deactivation_is_rejected_after_concurrent_sd_assignment_commits() {
    use std::time::Duration;

    let app = TestApp::spawn().await;
    let secretary = seed_user(&app, "IACUC_STAFF").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let actor = user_actor(secretary, &["IACUC_STAFF"]);

    let create_req_glp = CreateProtocolRequest {
        title: "端對端併發測試計劃".to_string(),
        pi_user_id: Some(secretary),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": true } })),
        start_date: None,
        end_date: None,
        study_director_user_id: None,
    };
    let p = ProtocolService::create(&app.db_pool, &actor, &create_req_glp, secretary)
        .await
        .expect("create");
    assert!(
        p.is_glp,
        "測試前提不成立：計畫必須是 GLP 案，否則停用防護不會擋"
    );

    // ── 第三方人工鎖：FOR SHARE，逼停用端先卡住等待 ──
    let mut holder = app.db_pool.begin().await.expect("begin holder tx");
    sqlx::query("SELECT is_active FROM users WHERE id = $1 FOR SHARE")
        .bind(sd)
        .fetch_one(&mut *holder)
        .await
        .expect("holder 取得 FOR SHARE");

    // CodeRabbit #26 第 6 輪指出：`pg_stat_activity` 輪詢只比對 SQL 文字，沒有
    // 綁定到 `deactivate_task` 實際使用的那條連線——`update`／`deactivate_self`／
    // `delete` 三條路徑的鎖 SQL 文字完全相同，而 `#[serial]` 只序列化「同一個
    // 測試執行檔內」的測試，擋不住 `cargo test` 底下其他執行檔同時跑、剛好也在
    // 等同一句 SQL 的鎖，會造成偽陽性。修法：用專屬的單連線 pool 餵給
    // `deactivate_self`，先在這條連線上查一次 `pg_backend_pid()`，因為
    // `max_connections(1)` 保證之後 `pool.begin()` 拿到的一定是同一條實體連線，
    // 所以這個 PID 就是 `deactivate_task` 真正會用的那條——輪詢時直接用
    // `pid = $1` 鎖定，不再需要靠 SQL 文字比對去猜。
    let deactivate_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(
            &std::env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL 必須存在（TestApp::spawn 已驗證過）"),
        )
        .await
        .expect("connect dedicated single-connection pool for deactivate_task");
    let deactivate_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&deactivate_pool)
        .await
        .expect("query pg_backend_pid on dedicated pool");

    // 真的呼叫停用——此刻應該卡在 FOR UPDATE 上（與 holder 的 FOR SHARE 互斥）。
    let deactivate_actor = ActorContext::User(user_cu(sd, &["EXPERIMENT_STAFF"]));
    let deactivate_task = tokio::spawn(async move {
        UserService::deactivate_self(&deactivate_pool, &deactivate_actor, sd).await
    });

    // CodeRabbit #26 第 5 輪指出：固定 sleep + is_finished() 只證明「task 還沒結束」，
    // 證明不了它已經送出 `FOR UPDATE` 並卡在鎖上——在 CI 負載高或排程延遲時，
    // task 可能 200ms 後根本還沒開始執行那句 SQL，測試會在完全沒測到真正時序的
    // 情況下通過。改成輪詢 `pg_stat_activity`，直接觀察那條連線是否真的
    // `wait_event_type = 'Lock'` 卡在這句 SQL 上，而不是猜一個「應該夠久」的時間。
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let waiting: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity
               WHERE pid = $1
                 AND wait_event_type = 'Lock'
                 AND query ILIKE '%FROM users WHERE id = $1 FOR UPDATE%'",
        )
        .bind(deactivate_pid)
        .fetch_one(&app.db_pool)
        .await
        .expect("query pg_stat_activity");
        if waiting > 0 {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "等超過 5 秒仍沒有偵測到停用交易卡在 FOR UPDATE 上——\
             測試前提不成立（要嘛鎖沒生效，要嘛停用提早失敗了）"
        );
        assert!(
            !deactivate_task.is_finished(),
            "停用在偵測到卡鎖之前就結束了，代表它根本沒被 holder 的 FOR SHARE 擋住"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // 真的呼叫指派——與 holder 的 FOR SHARE 相容，不受影響，應正常成功並 commit。
    let update_req = UpdateProtocolRequest {
        title: None,
        working_content: None,
        start_date: None,
        end_date: None,
        study_director_user_id: Some(sd),
        version: None,
        source_form_version: None,
    };
    let scope = scope_for(&app, secretary, p.id).await;
    ProtocolService::update(&app.db_pool, &actor, scope, &update_req)
        .await
        .expect("指派應成功（holder 的 FOR SHARE 不擋 FOR SHARE）");

    // 放掉人工鎖，讓卡住的停用取得 FOR UPDATE；此時它應該看到剛 commit 的
    // SD 指派，被 ensure_not_glp_study_director_tx 正確擋下。
    holder.rollback().await.expect("release holder lock");

    let result = tokio::time::timeout(Duration::from_secs(10), deactivate_task)
        .await
        .expect("停用不該逾時——代表放鎖後它仍然卡住")
        .expect("deactivate_self 所在的 task 不該 panic");

    let err = result
        .expect_err("指派已經 commit，卡住的停用取得鎖之後應該被 GLP SD 防護擋下，而不是成功");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應該是 GLP SD 防護的 BusinessRule 錯誤，實際：{err:?}"
    );

    let still_active: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
        .bind(sd)
        .fetch_one(&app.db_pool)
        .await
        .expect("read back");
    assert!(still_active, "被擋下的停用不該讓帳號真的變成 inactive");
}
