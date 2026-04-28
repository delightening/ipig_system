//! R30-9: SignatureService::invalidate audit-in-tx 整合測試
//!
//! 驗證：
//! 1. **成功 invalidate** — admin 用正確密碼作廢簽章 → signature.is_valid=false +
//!    user_activity_logs 寫一筆 `SignatureInvalidated` 事件（before/after diff）。
//! 2. **密碼錯** — `verify_password_by_id` 失敗 → Unauthorized；signature 不變、
//!    user_activity_logs 無 `SignatureInvalidated` 新事件。
//!
//! 測試直接呼叫 service 層（不需 HTTP route），驗證 D5/D6 設計：
//! - D5：作廢事件入 user_activity_logs（HMAC chain）
//! - D6：作廢需密碼驗證（不可逆操作）

mod common;

use common::TestApp;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::services::SignatureService;
use erp_backend::AppError;
use serial_test::serial;
use uuid::Uuid;

/// 取 admin 的 (id, email)，並使用測試環境固定密碼。
async fn admin_credentials(app: &TestApp) -> (Uuid, String, String) {
    let email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    let password = std::env::var("ADMIN_INITIAL_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "iPig$ecure1".to_string());
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin id");
    (row.0, email, password)
}

/// 直接 INSERT 一筆 electronic_signatures，避開 sign() 對密碼 hash 的依賴。
/// 回傳 signature_id。
async fn insert_test_signature(app: &TestApp, signer_id: Uuid) -> Uuid {
    let entity_id = Uuid::new_v4().to_string();
    // R30-10：meaning 欄為 NOT NULL（migration 043），測試 fixture 顯式帶
    // 'APPROVE'::signature_meaning 對齊 §11.50(a)(3) "approval"。
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO electronic_signatures (
            entity_type, entity_id, signer_id, signature_type,
            content_hash, signature_data, signature_method, meaning
        )
        VALUES ($1, $2, $3, 'APPROVE', $4, $5, 'password', 'APPROVE'::signature_meaning)
        RETURNING id
        "#,
    )
    .bind("test_entity")
    .bind(&entity_id)
    .bind(signer_id)
    .bind("a".repeat(64))
    .bind("b".repeat(64))
    .fetch_one(&app.db_pool)
    .await
    .expect("insert test signature");
    row.0
}

fn make_actor(user_id: Uuid, email: String) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: user_id,
        email,
        roles: vec!["ADMIN".into()],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    })
}

async fn count_invalidated_events(app: &TestApp, signature_id: Uuid) -> i64 {
    let (n,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_activity_logs
        WHERE event_category = 'AUDIT'
          AND event_type = 'SIGNATURE_INVALIDATED'
          AND entity_id = $1
        "#,
    )
    .bind(signature_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count audit events");
    n
}

#[tokio::test]
#[serial]
async fn invalidate_success_writes_audit_event() {
    let app = TestApp::spawn().await;
    let (admin_id, email, password) = admin_credentials(&app).await;
    let signature_id = insert_test_signature(&app, admin_id).await;

    let actor = make_actor(admin_id, email);
    SignatureService::invalidate(&app.db_pool, &actor, signature_id, "test reason", &password)
        .await
        .expect("invalidate should succeed");

    // 驗 signature 狀態
    let (is_valid, reason, by): (bool, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT is_valid, invalidated_reason, invalidated_by FROM electronic_signatures WHERE id = $1",
    )
    .bind(signature_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch signature");
    assert!(!is_valid, "signature should be invalid after invalidate");
    assert_eq!(reason.as_deref(), Some("test reason"));
    assert_eq!(by, Some(admin_id));

    // 驗 audit 事件存在（D5：HMAC chain 入鏈）
    let n = count_invalidated_events(&app, signature_id).await;
    assert_eq!(n, 1, "exactly one SignatureInvalidated audit event expected");
}

#[tokio::test]
#[serial]
async fn invalidate_with_wrong_password_returns_unauthorized_and_no_audit() {
    let app = TestApp::spawn().await;
    let (admin_id, email, _password) = admin_credentials(&app).await;
    let signature_id = insert_test_signature(&app, admin_id).await;

    let actor = make_actor(admin_id, email);
    let err = SignatureService::invalidate(
        &app.db_pool,
        &actor,
        signature_id,
        "should-fail",
        "definitely-wrong-password",
    )
    .await
    .expect_err("wrong password should fail");
    assert!(matches!(err, AppError::Unauthorized), "expected Unauthorized, got {err:?}");

    // 驗 signature 未變動
    let (is_valid,): (bool,) =
        sqlx::query_as("SELECT is_valid FROM electronic_signatures WHERE id = $1")
            .bind(signature_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch signature");
    assert!(is_valid, "signature should remain valid on auth failure");

    // 驗 audit 無新事件
    let n = count_invalidated_events(&app, signature_id).await;
    assert_eq!(n, 0, "no SignatureInvalidated audit event on auth failure");
}
