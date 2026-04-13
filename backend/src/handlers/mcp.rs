//! MCP（Model Context Protocol）HTTP Handler
//!
//! POST /api/v1/mcp
//!
//! 實作 JSON-RPC 2.0，支援 initialize / tools/list / tools/call。
//! 認證：Authorization: Bearer mcp_xxxx_... （查 user_mcp_keys 表）

use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{AppError, AppState, Result};
use crate::middleware::CurrentUser;
use crate::services::mcp;

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
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into() }),
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
    // 1. 認證
    let user = match authenticate_mcp_key(&state, &headers).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Json(JsonRpcResponse::err(req.id, -32001, "Unauthorized")));
        }
    };

    // 2. 分派
    let resp = match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "tools/list" => handle_tools_list(req.id, &user),
        "tools/call" => handle_tool_call(&state, &user, req.id, &req.params).await,
        _ => JsonRpcResponse::err(req.id, -32601, format!("Method not found: {}", req.method)),
    };

    Ok(Json(resp))
}

// ── initialize ──

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(id, serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "ipig-review-server",
            "version": "1.0.0"
        }
    }))
}

// ── tools/list（依角色過濾） ──

fn handle_tools_list(id: Option<Value>, user: &CurrentUser) -> JsonRpcResponse {
    let mut tools: Vec<Value> = vec![
        tool_def("list_protocols", "列出 IACUC 計畫書清單，可依狀態篩選",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "計畫書狀態（留空=全部），e.g. PRE_REVIEW" },
                    "limit": { "type": "integer", "default": 20 }
                }
            }),
        ),
        tool_def("read_protocol", "讀取 IACUC 計畫書完整內容（含各章節、PI 資訊、版本）",
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
        tools.push(tool_def("batch_return_to_pi", "退回計畫書給申請人補件（建立審查意見 + 變更狀態）",
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
        tools.push(tool_def("get_review_history", "查詢申請人的歷史審查退件紀錄",
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

    // VET / ADMIN 有 submit_vet_review
    if is_vet_role(user) || is_admin_role(user) {
        tools.push(tool_def("submit_vet_review", "填寫並提交獸醫查檢表",
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
    id: Option<Value>,
    params: &Value,
) -> JsonRpcResponse {
    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return JsonRpcResponse::err(id, -32602, "Missing tool name"),
    };
    let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

    // 二次權限檢查
    if !check_tool_permission(user, name) {
        return JsonRpcResponse::err(id, -32003, format!("Permission denied for tool: {name}"));
    }

    let result = match name {
        "list_protocols"     => mcp::list_protocols(state, user, &args).await,
        "read_protocol"      => mcp::read_protocol(state, user, &args).await,
        "create_review_flag" => mcp::create_review_flag(state, user, &args).await,
        "batch_return_to_pi" => mcp::batch_return_to_pi(state, user, &args).await,
        "get_review_history" => mcp::get_review_history(state, &args).await,
        "submit_vet_review"  => mcp::submit_vet_review(state, user, &args).await,
        "notify_secretary"   => mcp::notify_secretary(state, user, &args).await,
        _ => Err(AppError::NotFound(format!("Unknown tool: {name}"))),
    };

    match result {
        Ok(val) => JsonRpcResponse::ok(id, serde_json::json!({
            "content": [{ "type": "text", "text": val.to_string() }]
        })),
        Err(e) => JsonRpcResponse::err(id, -32000, e.to_string()),
    }
}

// ── 認證 ──

async fn authenticate_mcp_key(state: &AppState, headers: &HeaderMap) -> Result<CurrentUser> {
    let token = extract_bearer(headers).ok_or(AppError::Unauthorized)?;
    if !token.starts_with(MCP_KEY_PREFIX) {
        return Err(AppError::Unauthorized);
    }

    let hash = hash_mcp_key(&token);

    // 查 user_mcp_keys → 取得 user_id
    let user_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT user_id FROM user_mcp_keys
        WHERE key_hash = $1 AND revoked_at IS NULL
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
        let _ = sqlx::query(
            "UPDATE user_mcp_keys SET last_used_at = NOW() WHERE key_hash = $1",
        )
        .bind(h)
        .execute(&db)
        .await;
    });

    // 載入 CurrentUser（借用現有的 user 載入邏輯）
    load_current_user(&state.db, user_id).await
}

async fn load_current_user(db: &sqlx::PgPool, user_id: Uuid) -> Result<CurrentUser> {
    let email: String = sqlx::query_scalar(
        "SELECT email FROM users WHERE id = $1 AND is_active = true",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT role_code FROM user_roles WHERE user_id = $1",
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

fn check_tool_permission(user: &CurrentUser, tool: &str) -> bool {
    match tool {
        "list_protocols" | "read_protocol" => true,
        "create_review_flag" | "batch_return_to_pi" | "get_review_history" | "notify_secretary" => is_write_role(user),
        "submit_vet_review" => is_vet_role(user) || is_admin_role(user),
        _ => false,
    }
}

fn is_write_role(user: &CurrentUser) -> bool {
    user.roles.iter().any(|r| {
        [crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_SYSTEM_ADMIN]
            .contains(&r.as_str())
    })
}

fn is_vet_role(user: &CurrentUser) -> bool {
    user.roles.iter().any(|r| r == crate::constants::ROLE_VET)
}

fn is_admin_role(user: &CurrentUser) -> bool {
    user.roles.iter().any(|r| r == crate::constants::ROLE_SYSTEM_ADMIN)
}

fn hash_mcp_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(String::from))
}
