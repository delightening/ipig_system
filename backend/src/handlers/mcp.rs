//! MCP（Model Context Protocol）HTTP Handler
//!
//! POST /api/v1/mcp
//!
//! 實作 JSON-RPC 2.0，支援 initialize / tools/list / tools/call。
//! 認證：Authorization: Bearer mcp_xxxx_... （查 user_mcp_keys 表）

use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::middleware::CurrentUser;
use crate::services::mcp;
use crate::{AppError, AppState, Result};

// ── JSON-RPC 型別 ──

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }
    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

// ── MCP Key 前綴 ──
const MCP_KEY_PREFIX: &str = "mcp_";

// ── POST /api/v1/mcp ──

pub async fn mcp_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>> {
    // 1. 認證（同時取得該 key 的 scopes；過期 key 已於查詢端排除）
    let (user, scopes) = match authenticate_mcp_key(&state, &headers).await {
        Ok(t) => t,
        Err(_) => {
            return Ok(Json(JsonRpcResponse::err(req.id, -32001, "Unauthorized")));
        }
    };

    // 2. 分派
    let resp = match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "tools/list" => handle_tools_list(req.id, &user),
        "tools/call" => handle_tool_call(&state, &user, &scopes, req.id, &req.params).await,
        _ => JsonRpcResponse::err(req.id, -32601, format!("Method not found: {}", req.method)),
    };

    Ok(Json(resp))
}

// ── initialize ──

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "ipig-review-server",
                "version": "1.0.0"
            }
        }),
    )
}

// ── tools/list（依角色過濾） ──

fn handle_tools_list(id: Option<Value>, user: &CurrentUser) -> JsonRpcResponse {
    let mut tools: Vec<Value> = vec![
        tool_def(
            "list_protocols",
            "列出 IACUC 計畫書清單，可依狀態篩選",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "計畫書狀態（留空=全部），e.g. PRE_REVIEW" },
                    "limit": { "type": "integer", "default": 20 }
                }
            }),
        ),
        tool_def(
            "read_protocol",
            "讀取 IACUC 計畫書完整內容（含各章節、PI 資訊、版本）",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "protocol_id": { "type": "string", "description": "計畫書 UUID 或編號（AUP-YYYY-NNN）" }
                },
                "required": ["protocol_id"]
            }),
        ),
    ];

    // STAFF / CHAIR / ADMIN：寫入工具 + 通知工具
    if is_write_role(user) {
        tools.push(tool_def("create_review_flag", "建立 Pre-Review 審查標註",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "protocol_id": { "type": "string" },
                    "flag_type": { "type": "string", "enum": ["needs_attention", "concern", "suggestion"] },
                    "section": { "type": "string" },
                    "message": { "type": "string" },
                    "suggestion": { "type": "string" }
                },
                "required": ["protocol_id", "flag_type", "section", "message", "suggestion"]
            }),
        ));
        tools.push(tool_def(
            "batch_return_to_pi",
            "退回計畫書給申請人補件（建立審查意見 + 變更狀態）",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "protocol_id": { "type": "string" },
                    "flags": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "section": { "type": "string" },
                                "message": { "type": "string" },
                                "suggestion": { "type": "string" }
                            },
                            "required": ["section", "message", "suggestion"]
                        }
                    },
                    "additional_note": { "type": "string" }
                },
                "required": ["protocol_id", "flags"]
            }),
        ));
        tools.push(tool_def(
            "get_review_history",
            "查詢申請人的歷史審查退件紀錄",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pi_user_id": { "type": "string" },
                    "limit": { "type": "integer", "default": 5 }
                },
                "required": ["pi_user_id"]
            }),
        ));
        tools.push(tool_def("notify_secretary", "透過 iPig SMTP 寄送通知信（供排程 agent 使用）",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "to_email": {
                        "type": "string",
                        "description": "收件人 Email，逗號分隔可多人。留空則自動讀取系統設定中的 IACUC 通知信箱"
                    },
                    "subject": { "type": "string", "description": "郵件主旨" },
                    "body_html": { "type": "string", "description": "HTML 郵件內容" },
                    "body_plain": { "type": "string", "description": "純文字備用內容（選填）" }
                },
                "required": ["subject", "body_html"]
            }),
        ));
    }

    // 只有 VET 有 submit_vet_review。管理員旁路已移除（2026-08-26 裁定 A，見下方說明）。
    if is_vet_role(user) {
        tools.push(tool_def(
            "submit_vet_review",
            "填寫並提交獸醫查檢表",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "protocol_id": { "type": "string" },
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "item_name": { "type": "string" },
                                "compliance": { "type": "string", "enum": ["V", "X", "-"] },
                                "comment": { "type": "string" }
                            },
                            "required": ["item_name", "compliance"]
                        }
                    },
                    "vet_signature": { "type": "string", "description": "獸醫師姓名（確認後填入）" }
                },
                "required": ["protocol_id", "items"]
            }),
        ));
    }

    JsonRpcResponse::ok(id, serde_json::json!({ "tools": tools }))
}

fn tool_def(name: &str, description: &str, input_schema: Value) -> Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

// ── tools/call ──

async fn handle_tool_call(
    state: &AppState,
    user: &CurrentUser,
    scopes: &[String],
    id: Option<Value>,
    params: &Value,
) -> JsonRpcResponse {
    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return JsonRpcResponse::err(id, -32602, "Missing tool name"),
    };
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    // 二次權限檢查（角色 + MCP key scope）
    if !check_tool_permission(user, scopes, name) {
        return JsonRpcResponse::err(id, -32003, format!("Permission denied for tool: {name}"));
    }

    let result = match name {
        "list_protocols" => mcp::list_protocols(state, user, &args).await,
        "read_protocol" => mcp::read_protocol(state, user, &args).await,
        "create_review_flag" => mcp::create_review_flag(state, user, &args).await,
        "batch_return_to_pi" => mcp::batch_return_to_pi(state, user, &args).await,
        "get_review_history" => mcp::get_review_history(state, &args).await,
        "submit_vet_review" => mcp::submit_vet_review(state, user, &args).await,
        "notify_secretary" => mcp::notify_secretary(state, user, &args).await,
        _ => Err(AppError::NotFound(format!("Unknown tool: {name}"))),
    };

    match result {
        Ok(val) => JsonRpcResponse::ok(
            id,
            serde_json::json!({
                "content": [{ "type": "text", "text": val.to_string() }]
            }),
        ),
        Err(e) => JsonRpcResponse::err(id, -32000, e.to_string()),
    }
}

// ── 認證 ──

async fn authenticate_mcp_key(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(CurrentUser, Vec<String>)> {
    let token = extract_bearer(headers).ok_or(AppError::Unauthorized)?;
    if !token.starts_with(MCP_KEY_PREFIX) {
        return Err(AppError::Unauthorized);
    }

    let hash = hash_mcp_key(&token);

    // 查 user_mcp_keys → user_id + scopes。
    // CSO-r3 #5：撤銷 / 已過期（expires_at < NOW）的 key 不匹配 → Unauthorized。
    let (user_id, scopes): (Uuid, Vec<String>) = sqlx::query_as(
        r#"
        SELECT user_id, scopes FROM user_mcp_keys
        WHERE key_hash = $1 AND revoked_at IS NULL
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(&hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    // 更新 last_used_at（fire-and-forget）
    let db = state.db.clone();
    let h = hash.clone();
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE user_mcp_keys SET last_used_at = NOW() WHERE key_hash = $1")
            .bind(h)
            .execute(&db)
            .await;
    });

    // 載入 CurrentUser（借用現有的 user 載入邏輯）
    let user = load_current_user(&state.db, user_id).await?;
    Ok((user, scopes))
}

async fn load_current_user(db: &sqlx::PgPool, user_id: Uuid) -> Result<CurrentUser> {
    let email: String =
        sqlx::query_scalar("SELECT email FROM users WHERE id = $1 AND is_active = true")
            .bind(user_id)
            .fetch_optional(db)
            .await?
            .ok_or(AppError::Unauthorized)?;

    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.code FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = $1 AND r.is_active = true
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let permissions: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.code FROM permissions p
        JOIN role_permissions rp ON p.id = rp.permission_id
        JOIN user_roles ur ON rp.role_id = ur.role_id
        WHERE ur.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(CurrentUser {
        id: user_id,
        email,
        roles,
        permissions,
        jti: String::new(),
        exp: i64::MAX,
        impersonated_by: None,
    })
}

// ── 權限輔助 ──

fn check_tool_permission(user: &CurrentUser, scopes: &[String], tool: &str) -> bool {
    match tool {
        // 唯讀工具：read scope（所有 key 皆具）即可
        "list_protocols" | "read_protocol" => true,
        // 特權「讀取」：需 write 角色，但唯讀 scope 亦可（非 mutation）
        "get_review_history" => is_write_role(user),
        // mutation 工具：需 write 角色 + key 具 write scope（CSO-r3 #5）
        "create_review_flag" | "batch_return_to_pi" | "notify_secretary" => {
            has_write_scope(scopes) && is_write_role(user)
        }
        "submit_vet_review" => has_write_scope(scopes) && is_vet_role(user),
        _ => false,
    }
}

/// MCP key 是否具 write scope（CSO-r3 #5）。
fn has_write_scope(scopes: &[String]) -> bool {
    scopes.iter().any(|s| s == "write")
}

fn is_write_role(user: &CurrentUser) -> bool {
    crate::services::access::is_iacuc_staff_or_chair(user)
}

fn is_vet_role(user: &CurrentUser) -> bool {
    user.roles.iter().any(|r| r == crate::constants::ROLE_VET)
}

// ⚠️ 這裡原本有一個 `is_admin_role`，只比對 `ROLE_SYSTEM_ADMIN`。
// 那個角色代碼在 `roles` 表裡不存在（實查），所以那個閘**對所有人回 false**
// ——不是「少了 legacy fallback」，是整個閘從未放行過任何人。
//
// 本 PR 原本把它改成 `user.is_admin()`。那看似只是修好比對，實際上是
// **啟用一條從未執行過的程式路徑**，而那條路徑是壞的：
// `submit_vet_review` 的 UPDATE 綁 `WHERE vet_id = <呼叫者>`，未被指派的管理員
// 會更新到 0 列，然後照樣回 `success: true`——整份獸醫審查靜默消失。
//
// 使用者 2026-08-26 裁定（選項 A）：**移除管理員旁路**，只有被指派的獸醫能送出。
// 資料模型只有 `vet_review_assignments`（以 `vet_id` 為鍵）這一個位置可放，
// 沒有非指派者的容身處；GLP 簽章歸屬也不該讓非獸醫簽獸醫查檢表
// （送出內容帶 `vet_signature` 與 `signed_by`）。
//
// 這條旁路從未生效過，移除不會奪走任何人現有的能力。

/// SHA-256 雜湊 MCP key（明文只在建立時回傳一次，DB 僅存雜湊）。
/// `pub` 供整合測試重用同一雜湊邏輯播種測試用 key。
pub fn hash_mcp_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staff_user() -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            email: "staff@test.local".into(),
            roles: vec![crate::constants::ROLE_IACUC_STAFF.to_string()],
            permissions: vec![],
            jti: String::new(),
            exp: i64::MAX,
            impersonated_by: None,
        }
    }

    // CSO-r3 #5：唯讀 scope 的 MCP key 不可呼叫 mutation 工具；write scope 才可。
    #[test]
    fn read_only_scope_blocks_mutation_tools() {
        let u = staff_user();
        let read = vec!["read".to_string()];
        let read_write = vec!["read".to_string(), "write".to_string()];

        // 唯讀工具：兩種 scope 皆可
        assert!(check_tool_permission(&u, &read, "list_protocols"));
        assert!(check_tool_permission(&u, &read, "read_protocol"));
        // 特權讀取：write 角色即可，不需 write scope
        assert!(check_tool_permission(&u, &read, "get_review_history"));

        // mutation 工具：唯讀 scope 必須被擋
        assert!(!check_tool_permission(&u, &read, "create_review_flag"));
        assert!(!check_tool_permission(&u, &read, "batch_return_to_pi"));
        assert!(!check_tool_permission(&u, &read, "notify_secretary"));

        // write scope + write 角色 → 放行
        assert!(check_tool_permission(&u, &read_write, "create_review_flag"));
        assert!(check_tool_permission(&u, &read_write, "batch_return_to_pi"));
        assert!(check_tool_permission(&u, &read_write, "notify_secretary"));
    }
}
