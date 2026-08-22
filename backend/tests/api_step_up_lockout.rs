//! R66-B3: step-up 密碼重驗暴力破解防護回歸測試。
//!
//! 缺口：`confirm-password` / `2fa/disable` / 電子簽章等敏感操作的密碼重驗
//! （`AuthService::verify_password_by_id`）原本只受寫入限流（120/min）保護、
//! **不計入任何鎖定計數**。持有受害者 session（但不知密碼）的攻擊者可繞過
//! 登入鎖定，從這些 step-up 端點暴力猜密碼。
//!
//! 修復：在 `verify_password_by_id` 加「獨立計數器」鎖定——近 N 分鐘內
//! `reauth_failure` 達門檻即拒絕（即使密碼正確）。**刻意與登入鎖定分離**：
//! step-up 只計 `reauth_failure`、登入只計 `login_failure`，互不影響，
//! 避免攻擊者用 step-up 失敗把使用者鎖出登入（DoS）。
//!
//! 本測試直接在 service 層驗證三件事：
//! 1. 達門檻後 step-up 被鎖（即使密碼正確）。
//! 2. 未達門檻時 step-up 正常放行。
//! 3. `login_failure`（登入失敗）**不會**鎖 step-up（驗證計數器分離）。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::STEP_UP_LOCKOUT_MAX_ATTEMPTS;
use erp_backend::services::AuthService;
use erp_backend::AppError;

#[path = "common/test_db.rs"]
mod test_db;

const TEST_PASSWORD: &str = "iPig$ecure1";

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 建立啟用中的專屬測試使用者（密碼為 `TEST_PASSWORD`）。
/// 用獨立帳號避免污染 admin 的 reauth_failure 計數、干擾其他測試。
async fn seed_active_user(pool: &PgPool) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "stepup-lockout-{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password(TEST_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, 'step-up lockout user', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await
    .expect("insert user");
    (id, email)
}

/// 插入 `n` 筆指定 `event_type` 的 login_events（created_at = NOW()，落在鎖定視窗內）。
async fn insert_login_events(pool: &PgPool, user_id: Uuid, email: &str, event_type: &str, n: i64) {
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

/// 共用 bootstrap：建 pool + 專屬測試 user，回 `(pool, user_id, email)`。
async fn setup_case() -> (PgPool, Uuid, String) {
    let pool = setup_pool().await;
    let (user_id, email) = seed_active_user(&pool).await;
    (pool, user_id, email)
}

#[tokio::test]
async fn step_up_locks_after_reauth_failure_threshold() {
    let (pool, user_id, email) = setup_case().await;

    // 達門檻的 reauth_failure
    insert_login_events(
        &pool,
        user_id,
        &email,
        "reauth_failure",
        STEP_UP_LOCKOUT_MAX_ATTEMPTS,
    )
    .await;

    // 即使密碼正確，也應因鎖定被拒（鎖定先於密碼驗證）
    let result = AuthService::verify_password_by_id(&pool, user_id, TEST_PASSWORD).await;
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(
                msg.contains("次數過多"),
                "鎖定錯誤訊息應提示嘗試次數過多，實際: {msg}"
            );
        }
        other => panic!("達門檻後 step-up 應回 Validation 鎖定錯誤，實際: {other:?}"),
    }
}

#[tokio::test]
async fn step_up_allows_under_reauth_failure_threshold() {
    let (pool, user_id, email) = setup_case().await;

    // 少一次即未達門檻
    insert_login_events(
        &pool,
        user_id,
        &email,
        "reauth_failure",
        STEP_UP_LOCKOUT_MAX_ATTEMPTS - 1,
    )
    .await;

    // 未達門檻 + 密碼正確 → 應成功
    AuthService::verify_password_by_id(&pool, user_id, TEST_PASSWORD)
        .await
        .expect("未達 reauth_failure 門檻且密碼正確時 step-up 應放行");
}

#[tokio::test]
async fn login_failures_do_not_lock_step_up() {
    let (pool, user_id, email) = setup_case().await;

    // 大量 login_failure（遠超 step-up 門檻），但 0 筆 reauth_failure
    insert_login_events(
        &pool,
        user_id,
        &email,
        "login_failure",
        STEP_UP_LOCKOUT_MAX_ATTEMPTS + 3,
    )
    .await;

    // 計數器分離：login_failure 不應鎖 step-up → 密碼正確時放行
    AuthService::verify_password_by_id(&pool, user_id, TEST_PASSWORD)
        .await
        .expect("login_failure 不應鎖 step-up（計數器分離），密碼正確應放行");
}
