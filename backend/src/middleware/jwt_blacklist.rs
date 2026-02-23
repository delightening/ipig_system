// JWT 黑名單（SEC-23 + SEC-33）
//
// 雙層架構：記憶體 HashMap（快取）+ PostgreSQL（持久化）
// - revoke: 同時寫入記憶體和 DB
// - is_revoked: 先查記憶體，miss 時查 DB 並回填快取
// - 啟動時從 DB 載入未過期的黑名單項目
// - 背景任務定期清理記憶體和 DB 的過期項目
//
// SEC-33: Mutex 中毒時改為 fail-closed（視為已撤銷，拒絕存取）

use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct JwtBlacklist {
    /// jti -> expires_at (Unix timestamp)
    revoked: Arc<Mutex<HashMap<String, i64>>>,
}

impl JwtBlacklist {
    pub fn new() -> Self {
        Self {
            revoked: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 從 DB 載入尚未過期的黑名單項目（啟動時呼叫）
    pub async fn load_from_db(&self, pool: &PgPool) {
        let now = Utc::now();
        match sqlx::query_as::<_, (String, chrono::DateTime<Utc>)>(
            "SELECT jti, expires_at FROM jwt_blacklist WHERE expires_at > $1",
        )
        .bind(now)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => {
                if let Ok(mut map) = self.revoked.lock() {
                    for (jti, exp) in &rows {
                        map.insert(jti.clone(), exp.timestamp());
                    }
                    tracing::info!(
                        "[JwtBlacklist] 從 DB 載入 {} 筆未過期的黑名單項目",
                        rows.len()
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "[JwtBlacklist] 載入 DB 黑名單失敗（首次啟動或尚未 migrate）: {}",
                    e
                );
            }
        }
    }

    /// 撤銷 JWT（同時寫入記憶體和 DB）
    pub async fn revoke(&self, jti: String, exp: i64, pool: &PgPool) {
        // 寫入記憶體快取
        if let Ok(mut map) = self.revoked.lock() {
            map.insert(jti.clone(), exp);
        }

        // 寫入 DB（非阻塞，失敗時僅記錄告警）
        let expires_at = chrono::DateTime::from_timestamp(exp, 0).unwrap_or_else(Utc::now);
        if let Err(e) = sqlx::query(
            "INSERT INTO jwt_blacklist (jti, expires_at) VALUES ($1, $2) ON CONFLICT (jti) DO NOTHING"
        )
        .bind(&jti)
        .bind(expires_at)
        .execute(pool)
        .await
        {
            tracing::warn!("[JwtBlacklist] 寫入 DB 失敗: {} (jti={})", e, jti);
        }
    }

    /// 撤銷 JWT（僅寫入記憶體，向後相容）
    pub fn revoke_memory_only(&self, jti: String, exp: i64) {
        if let Ok(mut map) = self.revoked.lock() {
            map.insert(jti, exp);
        }
    }

    /// 檢查 JWT 是否已撤銷
    /// SEC-33: Mutex 中毒時回傳 true（fail-closed，拒絕存取）
    pub fn is_revoked(&self, jti: &str) -> bool {
        match self.revoked.lock() {
            Ok(map) => map.contains_key(jti),
            Err(_) => {
                tracing::error!("[JwtBlacklist] Mutex 中毒，啟用 fail-closed 策略");
                true // 中毒時拒絕存取
            }
        }
    }

    /// 檢查 JWT 是否已撤銷（含 DB 回查）
    /// 記憶體 miss 時查 DB，並回填快取
    pub async fn is_revoked_with_db(&self, jti: &str, pool: &PgPool) -> bool {
        // 先查記憶體
        if self.is_revoked(jti) {
            return true;
        }

        // 記憶體 miss，查 DB
        match sqlx::query_as::<_, (String,)>(
            "SELECT jti FROM jwt_blacklist WHERE jti = $1 AND expires_at > NOW()",
        )
        .bind(jti)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(_)) => {
                tracing::debug!("[JwtBlacklist] DB 命中，回填快取 jti={}", jti);
                // 回填記憶體快取（不需要精確 exp，只要有就好）
                if let Ok(mut map) = self.revoked.lock() {
                    map.insert(jti.to_string(), Utc::now().timestamp() + 3600);
                }
                true
            }
            Ok(None) => false,
            Err(e) => {
                tracing::warn!("[JwtBlacklist] 查詢 DB 失敗: {} (jti={})", e, jti);
                false // DB 查詢失敗時，依記憶體結果
            }
        }
    }

    /// 清理已過期的記憶體項目
    pub fn cleanup(&self) {
        if let Ok(mut map) = self.revoked.lock() {
            let now = Utc::now().timestamp();
            map.retain(|_, exp| *exp > now);
        }
    }

    /// 清理 DB 中已過期的項目
    pub async fn cleanup_db(&self, pool: &PgPool) {
        if let Err(e) = sqlx::query("DELETE FROM jwt_blacklist WHERE expires_at < NOW()")
            .execute(pool)
            .await
        {
            tracing::warn!("[JwtBlacklist] 清理 DB 過期項目失敗: {}", e);
        }
    }

    /// 啟動背景清理任務（同時清理記憶體和 DB）
    pub fn start_cleanup_task(self, pool: PgPool) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                self.cleanup();
                self.cleanup_db(&pool).await;
            }
        });
    }
}
