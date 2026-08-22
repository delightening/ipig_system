//! R82-5 強制登出安全回歸測試。
//!
//! Bug：`AuditService::force_logout_session` 原本只設 `user_sessions.is_active=false`，
//! 不撤 token。但 auth 中介層**不查** `user_sessions.is_active`（只查 `users.is_active`
//! 與 `users.tokens_valid_after`）→ 被「強制登出」的使用者，既有 access token（~15 分）與
//! refresh token（最長 7 天）**仍然有效**，強制登出形同失效。
//!
//! 修復：連帶設 `users.tokens_valid_after=NOW()`（斷所有既發 access token）+
//! `AuthService::revoke_all_user_tokens_tx`（撤 refresh token），強制登出才真的斷線。

mod common;

use common::TestApp;
use erp_backend::services::{AuditService, AuthService};
use erp_backend::AppError;
use sqlx::PgPool;
use uuid::Uuid;

async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!("{label}-{}@test.local", &Uuid::new_v4().to_string()[..8]);
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(label)
    .execute(pool)
    .await
    .expect("insert user");
    id
}

async fn seed_session(pool: &PgPool, user_id: Uuid) -> Uuid {
    let sid = Uuid::new_v4();
    // 僅 user_id 必填無預設；is_active 預設 true、時間欄位預設 now()。
    sqlx::query("INSERT INTO user_sessions (id, user_id) VALUES ($1, $2)")
        .bind(sid)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("insert session");
    sid
}

async fn tokens_valid_after(pool: &PgPool, user_id: Uuid) -> Option<chrono::DateTime<chrono::Utc>> {
    sqlx::query_scalar("SELECT tokens_valid_after FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("query tokens_valid_after")
}

#[tokio::test]
async fn force_logout_revokes_user_tokens() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let admin_id = seed_user(pool, "flo-admin").await;
    let target_id = seed_user(pool, "flo-target").await;
    let session_id = seed_session(pool, target_id).await;

    // 前置：新帳號 tokens_valid_after 應為 NULL（未撤過 token）
    assert!(
        tokens_valid_after(pool, target_id).await.is_none(),
        "強制登出前 tokens_valid_after 應為 NULL"
    );

    AuditService::force_logout_session(pool, session_id, admin_id, Some("regression test"))
        .await
        .expect("force_logout_session 應成功");

    // 修復核心：token 全失效閘門必須被設（修復前完全不動 → 被登出者 token 仍可用最長 7 天）
    assert!(
        tokens_valid_after(pool, target_id).await.is_some(),
        "強制登出後必須設 users.tokens_valid_after（斷所有既發 access token）"
    );

    // session 也應標為 inactive
    let active: bool = sqlx::query_scalar("SELECT is_active FROM user_sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("query session is_active");
    assert!(!active, "強制登出後 session 應為 inactive");
}

#[tokio::test]
async fn force_logout_nonexistent_session_is_rejected() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let admin_id = seed_user(pool, "flo-admin2").await;

    // 不存在的 session_id → 應明確回 Forbidden（403）遮蔽存在性，不得靜默「成功」、不留誤導 audit。
    let result =
        AuditService::force_logout_session(pool, Uuid::new_v4(), admin_id, Some("no such session"))
            .await;

    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "強制登出不存在的 session 應回 Forbidden（403），實得：{result:?}"
    );
}
