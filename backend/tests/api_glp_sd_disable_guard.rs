//! GLP 案 SD 不得停用帳號（裁定 13）。
//!
//! ⚠️ 這是裁定 12（「SD 離職前必須先把 GLP 案結案」）的**強制機制**。
//! 沒有這道閘，那句話只是一個沒有人執行的約定：帳號一停用，該 GLP 案就同時
//! 失去「能換 SD」（GLP 鎖）與「能結案」（結案要 SD 簽章）兩條路，變成永久死鎖。
//!
//! 本檔一半的測試守的是**不要過度阻擋**——這道閘擋的是人事作業，
//! 過度阻擋會讓離職流程卡住，而卡住的代價由不知情的人承擔。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::UpdateUserRequest;
use erp_backend::services::UserService;
use erp_backend::AppError;

async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("glpdis-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("glpdis-{role_code}"))
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

/// 直接建計畫（繞過 service，本檔受測的是停用閘門不是建立流程）。
///
/// 回傳實際使用的 `protocol_no`，供斷言比對。
///
/// ⚠️ 編號帶隨機後綴，**測試必須可重複執行**。第一版把編號寫死成 GLPDIS-001，
/// 一次逾時中斷後資料留在測試庫，重跑就撞 `protocols_protocol_no_key` 唯一約束
/// 而失敗——症狀看起來像「程式壞了」，實際是測試資料沒有隔離。
/// TestApp::spawn() 不清資料，同一顆測試庫會跨次累積。
async fn seed_protocol(app: &TestApp, sd: Uuid, is_glp: bool, status: &str) -> String {
    let pi = seed_user(app, "PI").await;
    let id = Uuid::new_v4();
    let protocol_no = format!("GLPDIS-{}", &Uuid::new_v4().to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by, study_director_user_id, is_glp)
           VALUES ($1, $2, $3, $4::protocol_status, $5, $5, $6, $7)"#,
    )
    .bind(id)
    .bind(&protocol_no)
    .bind(format!("停用閘門測試 {protocol_no}"))
    .bind(status)
    .bind(pi)
    .bind(sd)
    .bind(is_glp)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    protocol_no
}

fn admin_actor(id: Uuid) -> ActorContext {
    ActorContext::User(CurrentUser {
        id,
        email: "admin@example.com".to_string(),
        roles: vec!["admin".to_string()],
        permissions: vec!["admin.user.edit".to_string()],
        jti: Uuid::new_v4().to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 全 None 的更新請求。
///
/// `UpdateUserRequest` 沒有 derive `Default`——刻意不在本 PR 幫它加，
/// 那會改到 production model 且與受測點無關。測試自己建一份就好。
fn blank_update() -> UpdateUserRequest {
    UpdateUserRequest {
        email: None,
        display_name: None,
        phone: None,
        phone_ext: None,
        organization: None,
        entry_date: None,
        position: None,
        aup_roles: None,
        years_experience: None,
        trainings: None,
        is_internal: None,
        is_active: None,
        role_ids: None,
        expires_at: None,
        version: None,
        force_role_change: None,
    }
}

fn deactivate() -> UpdateUserRequest {
    UpdateUserRequest {
        is_active: Some(false),
        ..blank_update()
    }
}

/// 🔴 核心：有未結案 GLP 案在身 → 擋，且訊息要列出計畫編號。
#[tokio::test]
#[serial]
async fn cannot_disable_sd_with_open_glp_protocol() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_no = seed_protocol(&app, sd, true, "APPROVED").await;

    let err = UserService::update(&app.db_pool, &admin_actor(admin), sd, &deactivate())
        .await
        .expect_err("有未結案 GLP 案的 SD 不得停用");

    let msg = format!("{err:?}");
    assert!(
        matches!(&err, AppError::BusinessRule(_)),
        "應為 BusinessRule，實得：{err:?}"
    );
    // 裁定 13 明訂訊息要能執行——只說「無法停用」的話人事不知道要找誰結案
    assert!(
        msg.contains(&protocol_no),
        "錯誤訊息必須列出擋住它的計畫編號 {protocol_no}，實得：{msg}"
    );
}

/// 🔴 被擋之後帳號必須仍是啟用的——檢查要在寫入之前，不是之後。
///
/// 對照 `handlers/user.rs` 的停用偵測：那個在 UserService::update **之後**才跑，
/// 若把閘門放在那裡，帳號早就停用了，回錯也救不回來。
#[tokio::test]
#[serial]
async fn rejected_disable_leaves_account_active() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    seed_protocol(&app, sd, true, "APPROVED").await;

    let _ = UserService::update(&app.db_pool, &admin_actor(admin), sd, &deactivate()).await;

    let still_active: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
        .bind(sd)
        .fetch_one(&app.db_pool)
        .await
        .expect("read back");
    assert!(still_active, "被拒之後帳號必須維持啟用");
}

/// 多份計畫時要全部列出，不是只列第一份。
///
/// 只列一份的話，人事結掉它、再停用、又被擋——每次只知道一個，
/// 要來回試 N 次才能結束。
#[tokio::test]
#[serial]
async fn message_lists_all_blocking_protocols() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let first = seed_protocol(&app, sd, true, "APPROVED").await;
    let second = seed_protocol(&app, sd, true, "DRAFT").await;

    let err = UserService::update(&app.db_pool, &admin_actor(admin), sd, &deactivate())
        .await
        .expect_err("應被擋");
    let msg = format!("{err:?}");
    assert!(msg.contains(&first), "缺 {first}：{msg}");
    assert!(msg.contains(&second), "缺 {second}：{msg}");
}

// ── 以下守「不要過度阻擋」──────────────────────────────────────

/// 非 GLP 計畫不擋——那種案子的 SD 是可以更換的，沒有死鎖風險。
#[tokio::test]
#[serial]
async fn non_glp_protocol_does_not_block() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    seed_protocol(&app, sd, false, "APPROVED").await;

    UserService::update(&app.db_pool, &admin_actor(admin), sd, &deactivate())
        .await
        .expect("非 GLP 計畫不該擋住停用");
}

/// 已結案／已刪除／已駁回的 GLP 案不擋——那些計畫不再需要 SD 負責。
#[tokio::test]
#[serial]
async fn closed_states_do_not_block() {
    for status in ["CLOSED", "DELETED", "REJECTED"] {
        let app = TestApp::spawn().await;
        let admin = seed_user(&app, "admin").await;
        let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
        seed_protocol(&app, sd, true, status).await;

        UserService::update(&app.db_pool, &admin_actor(admin), sd, &deactivate())
            .await
            .unwrap_or_else(|e| panic!("{status} 的 GLP 案不該擋住停用：{e:?}"));
    }
}

/// 別人的 GLP 案不擋——只看「這個帳號是不是那些計畫的 SD」。
#[tokio::test]
#[serial]
async fn other_persons_glp_protocol_does_not_block() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd_other = seed_user(&app, "EXPERIMENT_STAFF").await;
    let target = seed_user(&app, "EXPERIMENT_STAFF").await;
    seed_protocol(&app, sd_other, true, "APPROVED").await;

    UserService::update(&app.db_pool, &admin_actor(admin), target, &deactivate())
        .await
        .expect("別人的 GLP 案不該擋住這個帳號");
}

/// 只有「停用」這個動作被擋，改別的欄位不受影響。
///
/// 若寫成「只要有 GLP 案就什麼都不能改」，人事連改電話都不行。
#[tokio::test]
#[serial]
async fn other_field_updates_are_unaffected() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    seed_protocol(&app, sd, true, "APPROVED").await;

    let req = UpdateUserRequest {
        display_name: Some("改個名字".to_string()),
        ..blank_update()
    };
    let updated = UserService::update(&app.db_pool, &admin_actor(admin), sd, &req)
        .await
        .expect("非停用的更新不該被擋");
    assert_eq!(updated.display_name, "改個名字");
}

/// 重新啟用（false → true）不受影響——閘門只擋「啟用 → 停用」。
#[tokio::test]
#[serial]
async fn re_enabling_is_not_blocked() {
    let app = TestApp::spawn().await;
    let admin = seed_user(&app, "admin").await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    seed_protocol(&app, sd, true, "APPROVED").await;
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(sd)
        .execute(&app.db_pool)
        .await
        .expect("先停用");

    let req = UpdateUserRequest {
        is_active: Some(true),
        ..blank_update()
    };
    UserService::update(&app.db_pool, &admin_actor(admin), sd, &req)
        .await
        .expect("重新啟用不該被擋");
}
