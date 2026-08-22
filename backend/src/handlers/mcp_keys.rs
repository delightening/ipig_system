//! MCP API Key 管理 Handler
//! GET/POST/DELETE /api/v1/user/mcp-keys

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{middleware::CurrentUser, AppError, AppState, Result};

// ── Response / Request ──

#[derive(Debug, Serialize)]
pub struct McpKeyResponse {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// CSO-r3 #5：權限範圍（read / write）
    pub scopes: Vec<String>,
    /// CSO-r3 #5：過期時間（NULL=不過期，grandfather 既有 key）
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMcpKeyRequest {
    /// 使用者自訂名稱，例如「我的 claude.ai」
    pub name: String,
    /// CSO-r3 #5：是否授予 write scope（可呼叫 mutation 工具）。
    /// 預設 false = 唯讀（僅 list/read 工具）。
    #[serde(default)]
    pub write: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateMcpKeyResponse {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    /// CSO-r3 #5：此 key 的權限範圍（read / write）
    pub scopes: Vec<String>,
    /// CSO-r3 #5：過期時間（新 key 預設 +1 年）
    pub expires_at: Option<DateTime<Utc>>,
    /// 完整 key，僅回傳一次
    pub full_key: String,
}

// ── GET /api/v1/user/mcp-keys ──

pub async fn list_mcp_keys(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<McpKeyResponse>>> {
    type McpKeyRow = (
        Uuid,
        String,
        String,
        Option<DateTime<Utc>>,
        DateTime<Utc>,
        Vec<String>,
        Option<DateTime<Utc>>,
    );
    let rows: Vec<McpKeyRow> = sqlx::query_as(
        r#"
            SELECT id, key_prefix, name, last_used_at, created_at, scopes, expires_at
            FROM user_mcp_keys
            WHERE user_id = $1 AND revoked_at IS NULL
            ORDER BY created_at DESC
            "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    let keys = rows
        .into_iter()
        .map(
            |(id, key_prefix, name, last_used_at, created_at, scopes, expires_at)| McpKeyResponse {
                id,
                key_prefix,
                name,
                last_used_at,
                created_at,
                scopes,
                expires_at,
            },
        )
        .collect();

    Ok(Json(keys))
}

// ── POST /api/v1/user/mcp-keys ──

pub async fn create_mcp_key(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateMcpKeyRequest>,
) -> Result<Json<CreateMcpKeyResponse>> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("金鑰名稱不得為空".to_string()));
    }

    // SEC: MCP 工具為內部 IACUC 工作流程用途；外部帳號（外部 PI / 委託單位等
    // is_internal=false）不得建立 MCP 金鑰（縱深防禦，搭配工具層的可見範圍收斂）。
    // Option<bool>：欄位若為 NULL 不致 decode panic，且 NULL 視同非內部（fail closed）。
    let is_internal: Option<bool> =
        sqlx::query_scalar("SELECT is_internal FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&state.db)
            .await?;
    if !is_internal.unwrap_or(false) {
        return Err(AppError::Forbidden(
            "僅內部人員可建立 MCP 連線金鑰".to_string(),
        ));
    }

    // 限制每人最多 5 個有效 key
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_mcp_keys WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    if count >= 5 {
        return Err(AppError::BusinessRule(
            "每人最多可建立 5 個 MCP 金鑰，請先撤銷舊金鑰".to_string(),
        ));
    }

    // 產生 key：mcp_<8 hex>_<32 hex>
    // 用 block scope 確保 ThreadRng（!Send）在任何 .await 前被 drop
    let (full_key, key_prefix, key_hash) = {
        let mut rng = rand::rng();
        let prefix_bytes: [u8; 4] = rng.random();
        let secret_bytes: [u8; 16] = rng.random();
        let prefix_hex = hex::encode(prefix_bytes);
        let secret_hex = hex::encode(secret_bytes);
        let full_key = format!("mcp_{prefix_hex}_{secret_hex}");
        let key_prefix = format!("mcp_{prefix_hex}");
        let mut hasher = Sha256::new();
        hasher.update(full_key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());
        (full_key, key_prefix, key_hash)
    };

    let id = Uuid::new_v4();
    let now: DateTime<Utc> = Utc::now();
    // CSO-r3 #5：預設唯讀；write=true 才授予 mutation 權。新 key 預設 +1 年過期。
    let scopes: Vec<String> = if req.write {
        vec!["read".to_string(), "write".to_string()]
    } else {
        vec!["read".to_string()]
    };
    let expires_at = now + chrono::Duration::days(365);

    sqlx::query(
        r#"
        INSERT INTO user_mcp_keys (id, user_id, key_hash, key_prefix, name, scopes, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(&key_hash)
    .bind(&key_prefix)
    .bind(&name)
    .bind(&scopes)
    .bind(expires_at)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(Json(CreateMcpKeyResponse {
        id,
        key_prefix,
        name,
        created_at: now,
        scopes,
        expires_at: Some(expires_at),
        full_key,
    }))
}

// ── POST /api/v1/user/mcp-keys/:id/revoke ──

pub async fn revoke_mcp_key(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let affected = sqlx::query(
        r#"
        UPDATE user_mcp_keys
        SET revoked_at = NOW()
        WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(key_id)
    .bind(user.id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("金鑰不存在或已撤銷".to_string()));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
