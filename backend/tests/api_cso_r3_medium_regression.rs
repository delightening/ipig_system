//! CSO 第三次 comprehensive sweep — MEDIUM #3/#4 回歸測試。
//!
//! #3：改密碼 / 重設只撤 refresh token，未過期 access token 仍可用 → 加
//!     `users.tokens_valid_after`，auth_middleware 比對 JWT iat 後拒絕舊 token。
//! #4：JWT 內 roles 快照在角色變更後殘留 → 角色實際變動時一併設 tokens_valid_after。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::UpdateUserRequest;
use erp_backend::services::{AiReviewService, AuthService, UserService};
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "cso_r3_medium_regression",
};

async fn seed_login_user(app: &TestApp, label: &str, password: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "cso-r3m-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("cso r3m {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    (id, email)
}

fn role_update(role_ids: Vec<Uuid>) -> UpdateUserRequest {
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
        role_ids: Some(role_ids),
        expires_at: None,
        version: None,
        force_role_change: None,
    }
}

async fn tokens_valid_after(app: &TestApp, user_id: Uuid) -> Option<chrono::DateTime<chrono::Utc>> {
    sqlx::query_scalar("SELECT tokens_valid_after FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query tokens_valid_after")
}

// ── #3：tokens_valid_after 之前簽發的 access token 應被 auth_middleware 拒絕 ──
#[tokio::test]
#[serial]
async fn access_token_rejected_after_tokens_valid_after() {
    let app = TestApp::spawn().await;
    let pw = "iPig$ecure1";
    let (user_id, email) = seed_login_user(&app, "enforce", pw).await;

    let token = app.login(&email, pw).await.expect("login 應成功");

    // 初始 tokens_valid_after = NULL → token 有效
    let res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(res.status().as_u16(), 200, "tva NULL 時 token 應有效");

    // 設 tokens_valid_after 為未來（晚於此 token 的 iat）→ 舊 token 失效
    sqlx::query(
        "UPDATE users SET tokens_valid_after = NOW() + interval '60 seconds' WHERE id = $1",
    )
    .bind(user_id)
    .execute(&app.db_pool)
    .await
    .expect("set tva");

    let res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(
        res.status().as_u16(),
        401,
        "iat 早於 tokens_valid_after 的 token 必須被拒絕"
    );
}

// ── #3：admin 重設密碼會設定 tokens_valid_after ──
#[tokio::test]
#[serial]
async fn admin_password_reset_sets_tokens_valid_after() {
    let app = TestApp::spawn().await;
    let (user_id, _email) = seed_login_user(&app, "reset", "iPig$ecure1").await;

    assert!(
        tokens_valid_after(&app, user_id).await.is_none(),
        "初始應為 NULL"
    );

    // admin actor 必須是「真實存在的使用者」（audit log FK → users.id）
    let (admin_id, _admin_email) = seed_login_user(&app, "admin-actor", "iPig$ecure9").await;
    let admin = ActorContext::User(CurrentUser {
        id: admin_id,
        email: "admin@test.local".into(),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    });
    AuthService::reset_user_password(&app.db_pool, &admin, user_id, "iPig$ecure2", None, None)
        .await
        .expect("admin 重設密碼應成功");

    assert!(
        tokens_valid_after(&app, user_id).await.is_some(),
        "重設密碼後 tokens_valid_after 應被設定（撤銷舊 access token）"
    );
}

// ── #4：角色實際變動會設定 tokens_valid_after ──
#[tokio::test]
#[serial]
async fn role_change_sets_tokens_valid_after() {
    let app = TestApp::spawn().await;
    let (user_id, _email) = seed_login_user(&app, "rolechg", "iPig$ecure1").await;

    assert!(
        tokens_valid_after(&app, user_id).await.is_none(),
        "初始應為 NULL"
    );

    // 從 PI 改為 EXPERIMENT_STAFF（before != after）→ 應設 tokens_valid_after
    let new_role: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'EXPERIMENT_STAFF'")
        .fetch_one(&app.db_pool)
        .await
        .expect("EXPERIMENT_STAFF role 必須存在");

    UserService::update(
        &app.db_pool,
        &SYSTEM_TEST,
        user_id,
        &role_update(vec![new_role]),
    )
    .await
    .expect("角色變更應成功");

    assert!(
        tokens_valid_after(&app, user_id).await.is_some(),
        "角色變動後 tokens_valid_after 應被設定（撤銷舊 roles 快照）"
    );
}

// ── L-3：自助停用帳號會設定 tokens_valid_after（撤銷其他裝置未過期 access token）──
#[tokio::test]
#[serial]
async fn self_deactivate_sets_tokens_valid_after() {
    let app = TestApp::spawn().await;
    let (user_id, email) = seed_login_user(&app, "l3selfdel", "iPig$ecure1").await;

    assert!(
        tokens_valid_after(&app, user_id).await.is_none(),
        "初始應為 NULL"
    );

    // 以本人身分自助停用帳號（deactivate_self 守門：actor.id 必須 == id）
    let actor = ActorContext::User(CurrentUser {
        id: user_id,
        email,
        roles: vec!["PI".into()],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    UserService::deactivate_self(&app.db_pool, &actor, user_id)
        .await
        .expect("自助停用應成功");

    assert!(
        tokens_valid_after(&app, user_id).await.is_some(),
        "自助停用後 tokens_valid_after 應被設定（L-3：撤銷其他裝置未過期 access token）"
    );
    let is_active: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query is_active");
    assert!(!is_active, "帳號應已停用");
}

// ── #6：全域每日 LLM 呼叫上限 hard-stop ──
#[tokio::test]
#[serial]
async fn global_llm_budget_hard_stops_at_limit() {
    let app = TestApp::spawn().await;

    // limit 0：今日呼叫數（>= 0）必達上限 → BusinessRule hard-stop
    let blocked = AiReviewService::check_global_llm_budget(&app.db_pool, 0).await;
    assert!(
        matches!(blocked, Err(AppError::BusinessRule(_))),
        "limit 0 應 hard-stop，實得：{blocked:?}"
    );

    // limit 極大：永遠在預算內 → 放行
    AiReviewService::check_global_llm_budget(&app.db_pool, i64::MAX)
        .await
        .expect("極大上限應放行");
}
