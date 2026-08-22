//! MCP 認證回歸測試（code-review C1）。
//!
//! Bug：`load_current_user` 以 `SELECT role_code FROM user_roles` 載入角色，
//! 但 user_roles 表只有 role_id（角色碼在 roles.code）。該查詢執行期必拋
//! 「column does not exist」→ `?` 傳成 AppError → authenticate_mcp_key 吞為
//! Unauthorized，導致所有 MCP key 認證崩潰、roles 永遠載不到（write/vet scope
//! 守衛形同失效）。修正改走 `JOIN roles r ON ur.role_id = r.id` 取 r.code。
//!
//! 既有 mcp 測試只測純函式 check_tool_permission，完全不打 DB，故 CI 全綠仍掩蓋此 bug。
//! 本測試走真實 HTTP /api/v1/mcp 端到端，DB 層驗證角色確實載入。

mod common;

use common::TestApp;
use serde_json::Value;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::handlers::mcp::hash_mcp_key;
use erp_backend::services::AuthService;

/// 種一個具指定角色碼的內部使用者，回傳 user_id。
async fn seed_user_with_role(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!("mcp-auth-{}@test.local", &Uuid::new_v4().to_string()[..8]);
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind("mcp auth test")
    .execute(&app.db_pool)
    .await
    .expect("insert user");

    let affected = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role")
    .rows_affected();
    assert_eq!(affected, 1, "角色 {role_code} 必須存在於 roles 表");
    id
}

/// 為 user 建立一把有效 MCP key（明文 + 寫入雜湊），回傳明文 token。
async fn seed_mcp_key(app: &TestApp, user_id: Uuid, scopes: &[&str]) -> String {
    let token = format!("mcp_{}", Uuid::new_v4().simple());
    let key_hash = hash_mcp_key(&token);
    let scope_vec: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
    sqlx::query(
        r#"INSERT INTO user_mcp_keys (user_id, key_hash, key_prefix, name, scopes)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(user_id)
    .bind(&key_hash)
    .bind(&token[..12])
    .bind("test key")
    .bind(&scope_vec)
    .execute(&app.db_pool)
    .await
    .expect("insert mcp key");
    token
}

fn tool_names(result: &Value) -> Vec<String> {
    result
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

// ── C1：MCP key 認證成功且角色正確載入（write role 看到 write 工具） ──
#[tokio::test]
#[serial]
async fn mcp_auth_loads_roles_and_gates_write_tools() {
    let app = TestApp::spawn().await;
    let user_id = seed_user_with_role(&app, "IACUC_STAFF").await;
    let token = seed_mcp_key(&app, user_id, &["read", "write"]).await;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    let res = app.auth_post("/api/v1/mcp", &body, &token).await;
    assert_eq!(
        res.status().as_u16(),
        200,
        "HTTP 應 200（JSON-RPC 包在 body）"
    );

    let json: Value = res.json().await.expect("parse json-rpc response");

    // 回歸核心：認證不得因 role 查詢崩潰而回 -32001 Unauthorized
    assert!(
        json.get("error").is_none(),
        "MCP 認證不應失敗（舊 bug 會因 role_code 欄位不存在而回 -32001），實得：{json}"
    );

    let result = json.get("result").expect("應有 result");
    let names = tool_names(result);
    // 唯讀工具：所有有效 key 皆有
    assert!(
        names.contains(&"list_protocols".to_string()),
        "應含 list_protocols"
    );
    // write 工具：證明 IACUC_STAFF 角色確實被載入（is_write_role 命中）
    assert!(
        names.contains(&"create_review_flag".to_string()),
        "IACUC_STAFF 應看到 write 工具 create_review_flag（角色未正確載入則不會出現），實得：{names:?}"
    );
}

// ── 對照：唯讀角色（PI）認證成功但看不到 write 工具 ──
#[tokio::test]
#[serial]
async fn mcp_auth_readonly_role_no_write_tools() {
    let app = TestApp::spawn().await;
    let user_id = seed_user_with_role(&app, "PI").await;
    let token = seed_mcp_key(&app, user_id, &["read"]).await;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    let res = app.auth_post("/api/v1/mcp", &body, &token).await;
    let json: Value = res.json().await.expect("parse json-rpc response");

    assert!(json.get("error").is_none(), "PI 認證應成功，實得：{json}");
    let names = tool_names(&json["result"]);
    assert!(
        names.contains(&"list_protocols".to_string()),
        "PI 應有唯讀工具"
    );
    assert!(
        !names.contains(&"create_review_flag".to_string()),
        "PI（非 write role）不應看到 write 工具，實得：{names:?}"
    );
}
