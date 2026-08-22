//! signature bridge GC 整合測試（SignatureBridgeService::cleanup_stale_sessions）
//!
//! 驗證 GC 的刪除/保留邊界，重點是縮短明文 payload at-rest 殘留：
//! - CONSUMED / EXPIRED → 刪
//! - PENDING 過 TTL（expires_at < now）→ 刪；PENDING 未過期 → 留（使用中）
//! - COMPLETED 超過 1h grace（submitted_at 久遠）→ 刪（棄單，連 payload 一起清）
//! - COMPLETED 仍在 grace 內（剛 submit）→ 留（桌機正要取走）

mod common;

use common::TestApp;
use erp_backend::services::SignatureBridgeService;
use serial_test::serial;
use uuid::Uuid;

/// 取 admin id 當 session 的 user_id（FK NOT NULL REFERENCES users）。
async fn admin_id(app: &TestApp) -> Uuid {
    let email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin id");
    row.0
}

/// 直接 INSERT 一筆 bridge session，明確控制 status / expires_at / submitted_at。
/// 回傳 session id。
#[allow(clippy::too_many_arguments)]
async fn insert_session(
    app: &TestApp,
    user_id: Uuid,
    status: &str,
    expires_offset_min: i64,
    submitted_offset_min: Option<i64>,
    with_payload: bool,
) -> Uuid {
    let id = Uuid::new_v4();
    // R66-C6：payload 欄位現為 TEXT（AEAD 信封字串）。GC 只在意 payload 有無，內容用任意信封字串。
    let payload: Option<String> = with_payload.then(|| "1:dummy-encrypted-envelope".to_string());
    sqlx::query(
        r#"INSERT INTO signature_bridge_sessions
             (id, user_id, mobile_token_hash, purpose, payload, status, expires_at, submitted_at)
           VALUES ($1, $2, $3, 'role.update', $4, $5,
                   NOW() + ($6 || ' minutes')::interval,
                   CASE WHEN $7::bigint IS NULL THEN NULL
                        ELSE NOW() + ($7 || ' minutes')::interval END)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(format!("hash-{id}"))
    .bind(payload)
    .bind(status)
    .bind(expires_offset_min.to_string())
    .bind(submitted_offset_min.map(|m| m.to_string()))
    .execute(&app.db_pool)
    .await
    .expect("insert bridge session");
    id
}

async fn session_exists(app: &TestApp, id: Uuid) -> bool {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM signature_bridge_sessions WHERE id = $1")
            .bind(id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count session");
    n > 0
}

#[tokio::test]
#[serial]
async fn cleanup_deletes_stale_keeps_active() {
    let app = TestApp::spawn().await;
    let uid = admin_id(&app).await;

    // 刪除目標：
    let consumed = insert_session(&app, uid, "CONSUMED", -10, Some(-30), false).await;
    let expired = insert_session(&app, uid, "EXPIRED", -10, None, false).await;
    let pending_expired = insert_session(&app, uid, "PENDING", -1, None, false).await;
    let completed_abandoned = insert_session(&app, uid, "COMPLETED", -120, Some(-90), true).await;

    // 保留目標：
    let pending_active = insert_session(&app, uid, "PENDING", 4, None, false).await;
    let completed_fresh = insert_session(&app, uid, "COMPLETED", -1, Some(-1), true).await;

    let deleted = SignatureBridgeService::cleanup_stale_sessions(&app.db_pool)
        .await
        .expect("cleanup");

    // 至少刪掉我們造的 4 筆 stale（可能含其他測試殘留，故用 >=）
    assert!(deleted >= 4, "expected >=4 deletions, got {deleted}");

    assert!(!session_exists(&app, consumed).await, "CONSUMED 應被刪");
    assert!(!session_exists(&app, expired).await, "EXPIRED 應被刪");
    assert!(
        !session_exists(&app, pending_expired).await,
        "過期 PENDING 應被刪"
    );
    assert!(
        !session_exists(&app, completed_abandoned).await,
        "超過 grace 的 COMPLETED 棄單應被刪（明文 payload 清除）"
    );

    assert!(
        session_exists(&app, pending_active).await,
        "未過期 PENDING（使用中）應保留"
    );
    assert!(
        session_exists(&app, completed_fresh).await,
        "grace 內 COMPLETED（桌機正要取走）應保留"
    );

    // 清理本測試保留的 row，避免污染後續測試計數
    sqlx::query("DELETE FROM signature_bridge_sessions WHERE id = ANY($1)")
        .bind(vec![pending_active, completed_fresh])
        .execute(&app.db_pool)
        .await
        .expect("cleanup leftovers");
}
