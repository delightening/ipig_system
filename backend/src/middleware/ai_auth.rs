//! AI API Key 認證 middleware
//!
//! AI 請求使用 `Authorization: Bearer ipig_ai_xxx` 格式，
//! 與一般使用者的 JWT 認證分離，走獨立的認證流程。

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

use crate::{AppError, AppState, Result};

/// AI 請求的已驗證身份資訊（插入 request extensions）
#[derive(Debug, Clone)]
pub struct AiCaller {
    pub api_key_id: uuid::Uuid,
    pub name: String,
    pub scopes: Vec<String>,
}

impl AiCaller {
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
            || self.scopes.contains(&"*".to_string())
            || self.scopes.contains(&"read".to_string()) && scope.ends_with(".read")
    }
}

/// AI API key 前綴
const AI_KEY_PREFIX: &str = "ipig_ai_";

pub async fn ai_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_bearer(&request).ok_or(AppError::Unauthorized)?;

    if !token.starts_with(AI_KEY_PREFIX) {
        return Err(AppError::Unauthorized);
    }

    let key_hash = hash_api_key(&token);

    let row = sqlx::query_as::<_, (uuid::Uuid, String, serde_json::Value, bool, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT id, name, scopes, is_active, expires_at FROM ai_api_keys WHERE key_hash = $1"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("[AI Auth] DB error: {}", e);
        AppError::Internal("AI auth lookup failed".to_string())
    })?
    .ok_or(AppError::Unauthorized)?;

    let (id, name, scopes_json, is_active, expires_at) = row;

    if !is_active {
        tracing::warn!("[AI Auth] key {} is deactivated", id);
        return Err(AppError::Forbidden("API key is deactivated".to_string()));
    }

    if let Some(exp) = expires_at {
        if exp < chrono::Utc::now() {
            tracing::warn!("[AI Auth] key {} is expired", id);
            return Err(AppError::Forbidden("API key has expired".to_string()));
        }
    }

    // 更新使用統計（fire-and-forget）
    let db = state.db.clone();
    let key_id = id;
    tokio::spawn(async move {
        let _ = sqlx::query(
            "UPDATE ai_api_keys SET last_used_at = now(), usage_count = usage_count + 1 WHERE id = $1"
        )
        .bind(key_id)
        .execute(&db)
        .await;
    });

    let scopes: Vec<String> = serde_json::from_value(scopes_json).unwrap_or_default();

    let caller = AiCaller {
        api_key_id: id,
        name,
        scopes,
    };

    request.extensions_mut().insert(caller);

    Ok(next.run(request).await)
}

/// 計算 API key 的 SHA-256 hash
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_bearer(request: &Request) -> Option<String> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    auth_header.strip_prefix("Bearer ").map(|s| s.to_string())
}

/// 生成新的 AI API key
pub fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &random_bytes,
    );
    format!("ipig_ai_{}", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_has_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("ipig_ai_"));
        assert!(key.len() > 20);
    }

    #[test]
    fn test_hash_api_key_deterministic() {
        let key = "ipig_ai_test_key_123";
        let h1 = hash_api_key(key);
        let h2 = hash_api_key(key);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_ai_caller_scope_check() {
        let caller = AiCaller {
            api_key_id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            scopes: vec!["read".to_string()],
        };
        assert!(caller.has_scope("animal.read"));
        assert!(caller.has_scope("protocol.read"));
        assert!(!caller.has_scope("animal.write"));
    }

    #[test]
    fn test_ai_caller_wildcard_scope() {
        let caller = AiCaller {
            api_key_id: uuid::Uuid::new_v4(),
            name: "admin".to_string(),
            scopes: vec!["*".to_string()],
        };
        assert!(caller.has_scope("anything"));
    }
}
