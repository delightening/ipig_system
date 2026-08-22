//! 密碼變更後應解除「失敗計數」型鎖定的回歸測試。
//!
//! 缺口（使用者回報 2026-07-29）：帳號鎖定以「近 N 分鐘內 `login_failure` 筆數」
//! 判定（`validate_credentials`），而三條密碼變更路徑都不碰 `login_events`。
//! 使用者連續打錯密碼被鎖後，**即使立刻重設密碼也仍被鎖滿整個視窗**，
//! 且鎖定檢查排在密碼驗證之前 → 新密碼是對的也照樣被擋。
//!
//! 修復：密碼變更成功時於同一 tx 寫入一筆 `lockout_reset` 標記事件，
//! 計數端只計「最後一筆 `lockout_reset` 之後」的失敗。
//! 刻意不刪 `login_failure`：那是偵測暴力破解的稽核證據。
//!
//! 涵蓋範圍（三條寫入標記的密碼路徑各一）：
//! 1. 忘記密碼 token 重設 → 解除登入鎖定（使用者實際走的路徑）。
//! 2. 管理員重設 → 解除登入鎖定。
//! 3. 本人修改密碼 → 解除登入鎖定。
//! 4. 重設後的**新**失敗仍會重新累積並鎖定（標記不是永久免鎖金牌）。
//! 5. 密碼重設一併解除 step-up（`reauth_failure`）鎖定。
//!
//! 不涵蓋：`2fa_failure` 刻意**不**重置（TOTP secret 未隨密碼變更，
//! 重置等於削弱 2FA 暴力防護），故無對應解除測試。
//!
//! 環境（JWT 測試金鑰 / HMAC 稽核金鑰 / migration）一律借 `common::TestApp::spawn`
//! 就緒，不在本檔重複一份金鑰材料；`#[serial]` 因 spawn 會設 process 層級 env。

mod common;

use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use common::TestApp;
use erp_backend::config::Config;
use erp_backend::constants::STEP_UP_LOCKOUT_MAX_ATTEMPTS;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::LoginRequest;
use erp_backend::services::AuthService;
use erp_backend::AppError;

const OLD_PASSWORD: &str = "iPig$ecure1";
const NEW_PASSWORD: &str = "iPigN3wPass9";
const MAX_ATTEMPTS: i64 = 5;

async fn setup() -> (PgPool, Config, Uuid, String) {
    let app = TestApp::spawn().await;

    // 固定鎖定門檻，讓測試不受外部 .env / CI 環境影響
    std::env::set_var("DISABLE_ACCOUNT_LOCKOUT", "false");
    std::env::set_var("ACCOUNT_LOCKOUT_MAX_ATTEMPTS", MAX_ATTEMPTS.to_string());
    std::env::set_var("ACCOUNT_LOCKOUT_DURATION_MINUTES", "15");
    let config = Config::from_env().expect("build config from env");

    // 專屬帳號，避免污染其他測試的失敗計數
    let (id, email) = seed_user(&app.db_pool, "pw-reset-lock").await;
    (app.db_pool.clone(), config, id, email)
}

/// 建立啟用中的測試使用者（密碼為 `OLD_PASSWORD`），回 `(id, email)`
async fn seed_user(pool: &PgPool, prefix: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!("{prefix}-{}@test.local", &id.to_string()[..8]);
    let hash = AuthService::hash_password(OLD_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, 'password reset lockout user', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await
    .expect("insert user");
    (id, email)
}

/// 灌入 n 筆指定類型的失敗事件（created_at = NOW()，落在鎖定視窗內）
async fn seed_failures(pool: &PgPool, user_id: Uuid, email: &str, event_type: &str, n: i64) {
    for _ in 0..n {
        sqlx::query(
            r#"INSERT INTO login_events (id, user_id, email, event_type, created_at)
               VALUES ($1, $2, $3, $4, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(event_type)
        .execute(pool)
        .await
        .expect("insert login_event");
    }
}

fn login_req(email: &str, password: &str) -> LoginRequest {
    LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    }
}

/// 斷言目前處於帳號鎖定狀態（訊息含「鎖定」，且優先於密碼驗證）
async fn assert_locked(pool: &PgPool, config: &Config, email: &str, password: &str) {
    match AuthService::validate_credentials(pool, config, &login_req(email, password), None).await {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("鎖定"), "應回帳號鎖定錯誤，實際訊息: {msg}")
        }
        other => panic!("達門檻後應被鎖定，實際: {other:?}"),
    }
}

/// 走「忘記密碼」信件 token 重設密碼（使用者 2026-07-29 實際走的路徑）
async fn reset_via_token(pool: &PgPool, email: &str, new_password: &str) {
    let (_, token) = AuthService::forgot_password(pool, email)
        .await
        .expect("forgot_password 應成功")
        .expect("啟用中帳號應取得重設 token");
    AuthService::reset_password_with_token(pool, &token, new_password)
        .await
        .expect("token 重設密碼應成功");
}

#[tokio::test]
#[serial]
async fn token_reset_clears_login_lockout() {
    let (pool, config, user_id, email) = setup().await;
    seed_failures(&pool, user_id, &email, "login_failure", MAX_ATTEMPTS).await;
    assert_locked(&pool, &config, &email, OLD_PASSWORD).await;

    reset_via_token(&pool, &email, NEW_PASSWORD).await;

    AuthService::validate_credentials(&pool, &config, &login_req(&email, NEW_PASSWORD), None)
        .await
        .expect("重設密碼後應立即可用新密碼登入，不必等鎖定視窗過期");

    // 稽核證據不得被銷毀：原失敗紀錄仍在
    let kept: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM login_events WHERE email = $1 AND event_type = 'login_failure'",
    )
    .bind(&email)
    .fetch_one(&pool)
    .await
    .expect("count login_failure");
    assert_eq!(kept, MAX_ATTEMPTS, "解鎖不得刪除 login_failure 稽核紀錄");
}

#[tokio::test]
#[serial]
async fn admin_reset_clears_login_lockout() {
    let (pool, config, user_id, email) = setup().await;
    seed_failures(&pool, user_id, &email, "login_failure", MAX_ATTEMPTS).await;
    assert_locked(&pool, &config, &email, OLD_PASSWORD).await;

    // actor 需為真實使用者：稽核鏈的 actor_user_id 有 FK 指向 users
    let (admin_id, admin_email) = seed_user(&pool, "pw-reset-admin").await;
    let admin = ActorContext::User(CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec!["admin".to_string()],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    AuthService::reset_user_password(&pool, &admin, user_id, NEW_PASSWORD, None, None)
        .await
        .expect("管理員重設密碼應成功");

    AuthService::validate_credentials(&pool, &config, &login_req(&email, NEW_PASSWORD), None)
        .await
        .expect("管理員重設密碼後使用者應能立即登入");
}

#[tokio::test]
#[serial]
async fn self_change_password_clears_login_lockout() {
    let (pool, config, user_id, email) = setup().await;
    seed_failures(&pool, user_id, &email, "login_failure", MAX_ATTEMPTS).await;
    assert_locked(&pool, &config, &email, OLD_PASSWORD).await;

    // 本人改密碼走的是已登入狀態下的路徑（驗舊密碼、不經鎖定閘）
    let actor = ActorContext::User(CurrentUser {
        id: user_id,
        email: email.clone(),
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    AuthService::change_own_password(
        &pool,
        &config,
        &actor,
        user_id,
        OLD_PASSWORD,
        NEW_PASSWORD,
        None,
        None,
    )
    .await
    .expect("本人修改密碼應成功");

    AuthService::validate_credentials(&pool, &config, &login_req(&email, NEW_PASSWORD), None)
        .await
        .expect("本人改完密碼後應能立即登入，不必等鎖定視窗過期");
}

#[tokio::test]
#[serial]
async fn failures_after_reset_lock_again() {
    let (pool, config, user_id, email) = setup().await;
    seed_failures(&pool, user_id, &email, "login_failure", MAX_ATTEMPTS).await;
    reset_via_token(&pool, &email, NEW_PASSWORD).await;

    // 重設後的新失敗必須重新累積 → 鎖定機制不能被標記事件永久停用
    seed_failures(&pool, user_id, &email, "login_failure", MAX_ATTEMPTS).await;
    assert_locked(&pool, &config, &email, NEW_PASSWORD).await;
}

#[tokio::test]
#[serial]
async fn password_reset_clears_step_up_lockout() {
    let (pool, _config, user_id, email) = setup().await;
    seed_failures(
        &pool,
        user_id,
        &email,
        "reauth_failure",
        STEP_UP_LOCKOUT_MAX_ATTEMPTS,
    )
    .await;
    match AuthService::verify_password_by_id(&pool, user_id, OLD_PASSWORD).await {
        Err(AppError::Validation(msg)) => assert!(
            msg.contains("次數過多"),
            "step-up 應先被鎖定，實際訊息: {msg}"
        ),
        other => panic!("達門檻後 step-up 應被鎖定，實際: {other:?}"),
    }

    reset_via_token(&pool, &email, NEW_PASSWORD).await;

    AuthService::verify_password_by_id(&pool, user_id, NEW_PASSWORD)
        .await
        .expect("密碼重設後 step-up 重驗鎖定應一併解除");
}
