//! AI API Key 認證 middleware
//!
//! AI 請求使用 `Authorization: Bearer ipig_ai_xxx` 格式，
//! 與一般使用者的 JWT 認證分離，走獨立的認證流程。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

use crate::constants::{SEC_EVENT_AI_KEY_DEACTIVATED, SEC_EVENT_AI_KEY_EXPIRED, SEC_EVENT_RATE_LIMIT_AI_KEY};
use crate::services::AuditService;
use crate::{AppError, AppState, Result};

/// Per-key sliding window rate limiter (in-memory)
#[derive(Debug, Clone)]
pub struct AiRateLimiter {
    windows: Arc<Mutex<HashMap<uuid::Uuid, Vec<std::time::Instant>>>>,
}

impl AiRateLimiter {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check(&self, key_id: uuid::Uuid, limit: i32) -> bool {
        if limit <= 0 {
            return true;
        }
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(60);
        let mut map = self.windows.lock().await;
        let entries = map.entry(key_id).or_default();
        entries.retain(|t| now.duration_since(*t) < window);
        if entries.len() >= limit as usize {
            return false;
        }
        entries.push(now);
        true
    }
}

impl Default for AiRateLimiter {
    fn default() -> Self {
        Self::new()
    }

    }

/// Global AI rate limiter instance
static AI_RATE_LIMITER: std::sync::LazyLock<AiRateLimiter> =
    std::sync::LazyLock::new(AiRateLimiter::new);

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

    let row = sqlx::query_as::<_, (uuid::Uuid, String, serde_json::Value, bool, Option<chrono::DateTime<chrono::Utc>>, i32)>(
        "SELECT id, name, scopes, is_active, expires_at, rate_limit_per_minute FROM ai_api_keys WHERE key_hash = $1"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("[AI Auth] DB error: {}", e);
        AppError::Internal("AI auth lookup failed".to_string())
    })?
    .ok_or(AppError::Unauthorized)?;

    let (id, name, scopes_json, is_active, expires_at, rate_limit) = row;

    // R22-2: 提取 request 資訊供安全事件記錄用
    let req_path = request.uri().path().to_string();
    let req_method = request.method().to_string();

    if !is_active {
        tracing::warn!("[AI Auth] key {} is deactivated", id);
        let db = state.db.clone();
        let name_c = name.clone();
        let path_c = req_path.clone();
        let method_c = req_method.clone();
        tokio::spawn(async move {
            let _ = AuditService::log_security_event(
                &db, SEC_EVENT_AI_KEY_DEACTIVATED, None, None,
                Some(&path_c), Some(&method_c),
                serde_json::json!({ "key_id": id, "key_name": name_c, "reason": "deactivated" }),
            ).await;
        });
        return Err(AppError::Forbidden("API key is deactivated".to_string()));
    }

    if let Some(exp) = expires_at {
        if exp < chrono::Utc::now() {
            tracing::warn!("[AI Auth] key {} is expired", id);
            let db = state.db.clone();
            let name_c = name.clone();
            let path_c = req_path.clone();
            let method_c = req_method.clone();
            tokio::spawn(async move {
                let _ = AuditService::log_security_event(
                    &db, SEC_EVENT_AI_KEY_EXPIRED, None, None,
                    Some(&path_c), Some(&method_c),
                    serde_json::json!({ "key_id": id, "key_name": name_c, "reason": "expired", "expired_at": exp }),
                ).await;
            });
            return Err(AppError::Forbidden("API key has expired".to_string()));
        }
    }

    // Per-key rate limit enforcement
    if !AI_RATE_LIMITER.check(id, rate_limit).await {
        tracing::warn!("[AI Auth] key {} rate limited ({}/min)", id, rate_limit);
        let db = state.db.clone();
        let name_c = name.clone();
        let path_c = req_path;
        let method_c = req_method;
        tokio::spawn(async move {
            let _ = AuditService::log_security_event(
                &db, SEC_EVENT_RATE_LIMIT_AI_KEY, None, None,
                Some(&path_c), Some(&method_c),
                serde_json::json!({ "key_id": id, "key_name": name_c, "reason": "rate_limited", "limit": rate_limit }),
            ).await;
        });
        return Err(AppError::TooManyRequests(format!(
            "Rate limit exceeded: {} requests per minute",
            rate_limit
        )));
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
