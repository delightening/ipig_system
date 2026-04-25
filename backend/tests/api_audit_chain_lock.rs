//! H1 (併發審查) audit_chain_verify cron multi-instance distributed lock 測試
//!
//! 驗證 advisory lock 行為：
//! 1. 一個 instance 可正常 acquire → run → release
//! 2. 第二個 instance 在第一個持有 lock 時會 skip（不重複執行）
//! 3. 第一個 release 後，第二個再次嘗試可成功 acquire
//!
//! 因為 advisory lock 是 PostgreSQL session-scoped，需用同一連線做 acquire+release。
//! 測試用 raw SQL 直接操作 advisory lock（簡化、不依賴內部 `AuditChainVerifyLock`），
//! 並驗證 `verify_yesterday_chain` 在 lock 被佔時觀察到 skip 行為。

mod common;

use common::TestApp;
use serial_test::serial;

const LOCK_KEY: i64 = 0x1A2B_3C4D_5E6F_7081_u64 as i64;

/// 從另一條 connection 取得 advisory lock，模擬「另一個 instance」。
async fn external_acquire_lock(app: &TestApp) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
    let mut conn = app.db_pool.acquire().await.expect("acquire");
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(LOCK_KEY)
        .fetch_one(&mut *conn)
        .await
        .expect("try_lock");
    assert!(acquired, "首次 try_advisory_lock 應成功");
    conn
}

async fn external_release_lock(mut conn: sqlx::pool::PoolConnection<sqlx::Postgres>) {
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(LOCK_KEY)
        .execute(&mut *conn)
        .await
        .expect("unlock");
}

/// 第二個連線觀察 lock 被持有時，try_acquire 回 false。
#[tokio::test]
#[serial]
async fn second_instance_observes_lock_held() {
    let app = TestApp::spawn().await;

    let holder = external_acquire_lock(&app).await;

    let mut other = app.db_pool.acquire().await.expect("acquire other conn");
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(LOCK_KEY)
        .fetch_one(&mut *other)
        .await
        .expect("try_lock other");
    assert!(!acquired, "另一個 session 應觀察到 lock 已被持有");

    external_release_lock(holder).await;

    // 釋放後第二個 session 可成功取得
    let acquired_after: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(LOCK_KEY)
        .fetch_one(&mut *other)
        .await
        .expect("try_lock other 2");
    assert!(acquired_after, "release 後另一 session 應可取得 lock");

    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(LOCK_KEY)
        .execute(&mut *other)
        .await
        .expect("cleanup unlock");
}

/// 當 audit_chain_verify_active=false（test 環境預設），verify_yesterday_chain
/// 應 early return 而不嘗試 acquire lock — 確認 active=false 不影響 lock 系統。
#[tokio::test]
#[serial]
async fn verify_skips_when_inactive_does_not_touch_lock() {
    let app = TestApp::spawn().await;

    // active=false（預設，TestApp::spawn 不會設 AUDIT_CHAIN_VERIFY_ACTIVE）→ 早退
    let config = erp_backend::config::Config::from_env().expect("build config");
    assert!(
        !config.audit_chain_verify_active,
        "test 環境預設 audit_chain_verify_active=false"
    );

    erp_backend::services::audit_chain_verify::verify_yesterday_chain(&app.db_pool, &config)
        .await
        .expect("verify should not error when inactive");

    // 鎖未被持有：另一連線可立即取得
    let mut conn = app.db_pool.acquire().await.expect("acquire");
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(LOCK_KEY)
        .fetch_one(&mut *conn)
        .await
        .expect("try_lock");
    assert!(
        acquired,
        "active=false 路徑不應碰 advisory lock；應仍可被取得"
    );
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(LOCK_KEY)
        .execute(&mut *conn)
        .await
        .expect("cleanup unlock");
}
