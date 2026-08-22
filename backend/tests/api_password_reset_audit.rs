//! 忘記密碼 token 重設稽核回歸測試（code-review H4）。
//!
//! Bug：reset_password_with_token 改了密碼、撤了 refresh token、設了
//! tokens_valid_after，卻完全不寫任何 audit row。攻擊者取得重設 token
//! （信箱被盜）接管帳號時，HMAC chain 與 user_activity_logs 無任何痕跡。
//!
//! 修復：補一筆 SECURITY 事件 PASSWORD_TOKEN_RESET（Anonymous actor，
//! 目標帳號記於 after_data.target_user_id）。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::AuthService;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn seed_active_user(pool: &PgPool) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "reset-audit-{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, 'reset audit user', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await
    .expect("insert user");
    (id, email)
}

async fn token_reset_audit_count(pool: &PgPool, user_id: Uuid) -> i64 {
    // entity 綁定 target user（entity_type/entity_id），確保可依 entity_id 查詢（gemini fix）
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'PASSWORD_TOKEN_RESET' \
           AND entity_type = 'user' AND entity_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("count PASSWORD_TOKEN_RESET audit")
}

#[tokio::test]
async fn token_reset_writes_security_audit() {
    let pool = setup_pool().await;
    let (user_id, email) = seed_active_user(&pool).await;

    assert_eq!(
        token_reset_audit_count(&pool, user_id).await,
        0,
        "重設前不應有 PASSWORD_TOKEN_RESET 稽核"
    );

    let (_uid, token) = AuthService::forgot_password(&pool, &email)
        .await
        .expect("forgot_password 應成功")
        .expect("active 帳號應回傳 reset token");

    AuthService::reset_password_with_token(&pool, &token, "iPig$ecure9")
        .await
        .expect("token 重設密碼應成功");

    assert_eq!(
        token_reset_audit_count(&pool, user_id).await,
        1,
        "token 重設後必須留下一筆 PASSWORD_TOKEN_RESET 稽核（修復前完全沒有）"
    );
}
