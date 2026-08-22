//! 管理員手動解除帳號鎖定的驗收測試。
//!
//! 帳號鎖定沒有持久旗標，是「視窗內 login_failure 筆數 >= 門檻」的滑動視窗判定
//! （`AuthService::validate_credentials`）。解鎖因此不是清 flag，而是在 login_events
//! 寫一筆 `lockout_admin_clear` 標記，把計數起點推到那一刻——沿用 #1086 為密碼變更
//! 引入的同一機制（`lockout_reset`），只是換一個型別以便稽核時分辨來源。
//!
//! 本測試釘住五件事：
//! 1. 達門檻後登入被鎖（前提條件成立）。
//! 2. 解鎖後**同一組正確密碼**可以登入。
//! 3. 解鎖**不刪** login_events 的失敗紀錄——那是稽核來源，只是不再計入。
//! 4. 解鎖會留下 `lockout_admin_clear` 標記（軌跡可查）。
//! 5. 解鎖後若又累積到門檻，會重新鎖定（解鎖不是永久豁免）。

mod common;

use common::TestApp;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::config::Config;
use erp_backend::constants::LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR;
use erp_backend::models::LoginRequest;
use erp_backend::services::AuthService;
use erp_backend::AppError;

const TEST_PASSWORD: &str = "iPig$ecure1";

/// 建立啟用中的專屬測試使用者，避免污染其他測試帳號的失敗計數。
async fn seed_active_user(pool: &PgPool) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "lockout-clear-{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password(TEST_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, 'lockout clear user', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await
    .expect("insert user");
    (id, email)
}

/// 插入 `n` 筆落在鎖定視窗內的 login_failure。
async fn insert_failures(pool: &PgPool, user_id: Uuid, email: &str, n: i64) {
    for _ in 0..n {
        sqlx::query(
            r#"INSERT INTO login_events (id, user_id, email, event_type, created_at)
               VALUES ($1, $2, $3, 'login_failure', NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .execute(pool)
        .await
        .expect("insert login_event");
    }
}

async fn count_events(pool: &PgPool, email: &str, event_type: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM login_events WHERE email = $1 AND event_type = $2",
    )
    .bind(email)
    .bind(event_type)
    .fetch_one(pool)
    .await
    .expect("count login_events")
}

fn login_request(email: &str) -> LoginRequest {
    LoginRequest {
        email: email.to_string(),
        password: TEST_PASSWORD.to_string(),
    }
}

/// 本測試專用的 Config。
///
/// **必須顯式打開鎖定**：CI 的 cargo test / coverage job 設了
/// `DISABLE_ACCOUNT_LOCKOUT=true`（避免平行測試累積 login_events 把 admin 帳號鎖掉，
/// 見 .github/workflows/ci.yml），`Config::from_env()` 會照單全收，
/// 於是 `validate_credentials` 整段跳過鎖定檢查——而鎖定正是本測試的主題。
/// 不從環境繼承這個旗標，測試才會在本機與 CI 得到同一個結果。
fn lockout_enabled_config() -> Config {
    let mut config = Config::from_env().expect("build config");
    config.disable_account_lockout = false;
    config
}

/// 斷言目前處於鎖定狀態（鎖定先於密碼驗證，故正確密碼也會被擋）。
async fn assert_locked(pool: &PgPool, config: &Config, email: &str) {
    match AuthService::validate_credentials(pool, config, &login_request(email), None).await {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("鎖定"), "應回帳號鎖定訊息，實際: {msg}");
        }
        other => panic!("達門檻後應鎖定，實際: {other:?}"),
    }
}

#[tokio::test]
#[serial]
async fn admin_unlock_lets_the_user_log_in_again() {
    let app = TestApp::spawn().await;
    let config = lockout_enabled_config();
    let (user_id, email) = seed_active_user(&app.db_pool).await;

    insert_failures(
        &app.db_pool,
        user_id,
        &email,
        config.account_lockout_max_attempts,
    )
    .await;
    assert_locked(&app.db_pool, &config, &email).await;

    let unlocked_id = AuthService::clear_account_lockout(&app.db_pool, &email)
        .await
        .expect("admin 解鎖應成功");
    assert_eq!(unlocked_id, user_id, "解鎖應回傳該帳號的 user id");

    AuthService::validate_credentials(&app.db_pool, &config, &login_request(&email), None)
        .await
        .expect("解鎖後同一組正確密碼應可登入");
}

#[tokio::test]
#[serial]
async fn unlock_keeps_failure_events_and_leaves_a_marker() {
    let app = TestApp::spawn().await;
    let config = lockout_enabled_config();
    let (user_id, email) = seed_active_user(&app.db_pool).await;

    let attempts = config.account_lockout_max_attempts;
    insert_failures(&app.db_pool, user_id, &email, attempts).await;
    AuthService::clear_account_lockout(&app.db_pool, &email)
        .await
        .expect("admin 解鎖應成功");

    assert_eq!(
        count_events(&app.db_pool, &email, "login_failure").await,
        attempts,
        "解鎖只推移計數起點，不得刪除 login_events 的失敗紀錄（暴力破解的稽核證據）"
    );
    assert_eq!(
        count_events(&app.db_pool, &email, LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR).await,
        1,
        "解鎖應留下 lockout_admin_clear 標記，日後查得出是管理員放行"
    );
}

#[tokio::test]
#[serial]
async fn failures_after_unlock_lock_the_account_again() {
    let app = TestApp::spawn().await;
    let config = lockout_enabled_config();
    let (user_id, email) = seed_active_user(&app.db_pool).await;

    insert_failures(
        &app.db_pool,
        user_id,
        &email,
        config.account_lockout_max_attempts,
    )
    .await;
    AuthService::clear_account_lockout(&app.db_pool, &email)
        .await
        .expect("admin 解鎖應成功");

    // 解鎖不是永久豁免：解鎖後新累積的失敗照樣計數
    insert_failures(
        &app.db_pool,
        user_id,
        &email,
        config.account_lockout_max_attempts,
    )
    .await;
    assert_locked(&app.db_pool, &config, &email).await;
}

#[tokio::test]
#[serial]
async fn unlocking_an_unknown_email_is_not_found() {
    let app = TestApp::spawn().await;

    // 帳號枚舉嘗試也會產生 brute_force 警報，但那種警報沒有帳號可解
    let result = AuthService::clear_account_lockout(&app.db_pool, "no-such-user@test.local").await;
    match result {
        Err(AppError::NotFound(_)) => {}
        other => panic!("不存在的 email 應回 NotFound，實際: {other:?}"),
    }
}
