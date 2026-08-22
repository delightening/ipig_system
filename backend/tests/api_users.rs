//! Integration tests for user management (admin-only).

mod common;

use erp_backend::middleware::ActorContext;
use erp_backend::services::UserService;
use serial_test::serial;

const SYSTEM_TEST: ActorContext = ActorContext::System { reason: "test" };

#[tokio::test]
#[serial]
async fn list_users_returns_paginated_result() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/users", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    // list_users 回傳 Vec<UserResponse>，即直接陣列
    assert!(body.is_array(), "Expected array, got: {:?}", body);
    // At least the admin user should exist
    assert!(!body.as_array().expect("body should be array").is_empty());
}

#[tokio::test]
#[serial]
async fn create_user_and_fetch() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let new_user = serde_json::json!({
        "email": format!("testuser_{}@test.local", ts),
        "display_name": "Integration Test User",
        "password": "TestPassword123!",
        "role_id": null,
        // is_internal 為必填（無 serde default）——刻意如此，見 CreateUserRequest
        "is_internal": true
    });

    let create_res = app.auth_post("/api/v1/users", &new_user, &token).await;

    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned: {}",
        create_res.status()
    );

    let created: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create response");
    let user_id = created["id"].as_str().expect("User should have id");

    // Fetch individual user
    let get_res = app
        .auth_get(&format!("/api/v1/users/{}", user_id), &token)
        .await;
    assert_eq!(get_res.status(), 200);

    let fetched: serde_json::Value = get_res.json().await.expect("Failed to parse user response");
    assert_eq!(fetched["display_name"], "Integration Test User");
}

/// 停用（is_active=false）的使用者預設不出現在列表，但帶 include_inactive=true 時應出現。
/// 迴歸測試：先前 list_users 硬編碼 WHERE is_active=true，導致停用者永久從列表消失，
/// 前端「顯示停用帳號」開關形同虛設。
#[tokio::test]
#[serial]
async fn deactivated_user_hidden_by_default_visible_with_include_inactive() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // as_nanos（自 UNIX epoch 完整時間戳）確保 email 唯一，避免 subsec_nanos
    // 每秒重複造成的 flaky 衝突
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .as_nanos();
    let email = format!("deactivated_{}@test.local", ts);

    let new_user = serde_json::json!({
        "email": email,
        "display_name": "Soon Deactivated",
        "password": "TestPassword123!",
        "role_id": null,
        // is_internal 為必填（無 serde default）——刻意如此，見 CreateUserRequest
        "is_internal": true
    });
    let create_res = app.auth_post("/api/v1/users", &new_user, &token).await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned: {}",
        create_res.status()
    );
    let created: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create response");
    let user_id = created["id"].as_str().expect("User should have id");

    // 停用該帳號（等同管理頁「停用」按鈕：PUT is_active=false）
    let deactivate = serde_json::json!({ "is_active": false });
    let put_res = app
        .auth_put(&format!("/api/v1/users/{}", user_id), &deactivate, &token)
        .await;
    assert_eq!(put_res.status(), 200, "Deactivate should succeed");

    // 預設列表不應包含停用者
    let default_res = app.auth_get("/api/v1/users", &token).await;
    assert_eq!(default_res.status(), 200);
    let default_body: serde_json::Value = default_res.json().await.expect("parse default list");
    let in_default = default_body
        .as_array()
        .expect("list should be array")
        .iter()
        .any(|u| u["id"].as_str() == Some(user_id));
    assert!(!in_default, "停用帳號不應出現在預設列表");

    // include_inactive=true 應包含停用者，且 is_active=false
    let inc_res = app
        .auth_get("/api/v1/users?include_inactive=true", &token)
        .await;
    assert_eq!(inc_res.status(), 200);
    let inc_body: serde_json::Value = inc_res.json().await.expect("parse include_inactive list");
    let found = inc_body
        .as_array()
        .expect("list should be array")
        .iter()
        .find(|u| u["id"].as_str() == Some(user_id))
        .expect("include_inactive=true 應能找回停用帳號");
    assert_eq!(found["is_active"], false, "找回的帳號應標記為停用");
}

/// 已軟刪除（匿名化）帳號即使帶 include_inactive=true 也不應出現在列表。
/// 迴歸測試：刪除者（email `…@deleted.local`）與單純停用帳號同為 is_active=false，
/// 但「顯示停用帳號」只應列出後者；刪除者已匿名化，不再具管理價值，應被排除。
#[tokio::test]
#[serial]
async fn soft_deleted_user_hidden_even_with_include_inactive() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .as_nanos();
    let email = format!("tobedeleted_{}@test.local", ts);

    let new_user = serde_json::json!({
        "email": email,
        "display_name": "Soon Deleted",
        "password": "TestPassword123!",
        "role_id": null,
        // is_internal 為必填（無 serde default）——刻意如此，見 CreateUserRequest
        "is_internal": true
    });
    let create_res = app.auth_post("/api/v1/users", &new_user, &token).await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned: {}",
        create_res.status()
    );
    let created: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create response");
    let user_id = created["id"].as_str().expect("User should have id");

    // 直接複製 UserService::delete 的匿名化狀態（email 改寫為 …@deleted.local），
    // 避免走需 X-Reauth-Token 的刪除端點；本測試驗證的是 list 的排除條件。
    // email 網域後綴綁定 DELETED_USER_EMAIL_DOMAIN，與 delete/list 共用同一常數。
    // R63-B6 收窄：delete 不再抹 display_name（記錄者名字保留），排除靠 email 網域。
    sqlx::query(
        r#"UPDATE users SET
            is_active = false,
            email = 'deleted_' || id::text || $2
        WHERE id = $1"#,
    )
    .bind(uuid::Uuid::parse_str(user_id).expect("valid uuid"))
    .bind(erp_backend::constants::DELETED_USER_EMAIL_DOMAIN)
    .execute(&app.db_pool)
    .await
    .expect("soft delete user");

    // include_inactive=true 會列出停用帳號，但已刪除（匿名化）帳號應被排除
    let inc_res = app
        .auth_get("/api/v1/users?include_inactive=true", &token)
        .await;
    assert_eq!(inc_res.status(), 200);
    let inc_body: serde_json::Value = inc_res.json().await.expect("parse include_inactive list");
    let found = inc_body
        .as_array()
        .expect("list should be array")
        .iter()
        .any(|u| u["id"].as_str() == Some(user_id));
    assert!(!found, "已軟刪除帳號不應出現在 include_inactive=true 列表");
}

/// R63-B6 收窄驗收：delete 保留 display_name（記錄者歸屬），但匿名化其餘 PII、停用帳號。
/// 直接呼叫 service（避開刪除端點需 X-Reauth-Token），驗證 DB 落地狀態。
#[tokio::test]
#[serial]
async fn delete_preserves_display_name_but_anonymizes_pii() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .as_nanos();
    let email = format!("preserve_name_{}@test.local", ts);

    let new_user = serde_json::json!({
        "email": email,
        "display_name": "王小明",
        "password": "TestPassword123!",
        "role_id": null,
        // is_internal 為必填（無 serde default）——刻意如此，見 CreateUserRequest
        "is_internal": true
    });
    let create_res = app.auth_post("/api/v1/users", &new_user, &token).await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create user returned: {}",
        create_res.status()
    );
    let created: serde_json::Value = create_res.json().await.expect("parse create response");
    let user_id = uuid::Uuid::parse_str(created["id"].as_str().expect("id")).expect("valid uuid");

    UserService::delete(&app.db_pool, &SYSTEM_TEST, user_id)
        .await
        .expect("delete user");

    let (display_name, email_after, is_active): (String, String, bool) =
        sqlx::query_as("SELECT display_name, email, is_active FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch deleted user");

    assert_eq!(display_name, "王小明", "display_name 應保留（記錄者歸屬）");
    assert!(
        email_after.ends_with("@deleted.local"),
        "email 應匿名化，實際: {email_after}"
    );
    assert!(!is_active, "帳號應停用");
}

#[tokio::test]
#[serial]
async fn list_roles_returns_array() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/roles", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body.is_array());
}

#[tokio::test]
#[serial]
async fn list_permissions_returns_array() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/permissions", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body.is_array());
    assert!(!body
        .as_array()
        .expect("permissions should be array")
        .is_empty());
}
