// JWT 黑名單模組（SEC-23）
//
// 使用記憶體 HashMap 儲存已撤銷的 JWT jti，
// 背景定時清理過期項目以避免記憶體洩漏。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// JWT 黑名單：儲存已撤銷的 token jti 及其過期時間戳
#[derive(Clone)]
pub struct JwtBlacklist {
    /// key = jti, value = JWT 過期時間（Unix timestamp）
    revoked: Arc<Mutex<HashMap<String, i64>>>,
}

impl JwtBlacklist {
    /// 建立新的 JWT 黑名單並啟動背景清理任務
    pub fn new() -> Self {
        let blacklist = Self {
            revoked: Arc::new(Mutex::new(HashMap::new())),
        };

        // 背景清理任務：每 5 分鐘清除已過期的 jti
        let cleanup = blacklist.revoked.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                let now = chrono::Utc::now().timestamp();
                if let Ok(mut map) = cleanup.lock() {
                    map.retain(|_jti, exp| *exp > now);
                    tracing::debug!("[JwtBlacklist] 清理完成，剩餘 {} 筆", map.len());
                }
            }
        });

        blacklist
    }

    /// 將 JWT 加入黑名單（撤銷）
    pub fn revoke(&self, jti: &str, exp: i64) {
        if let Ok(mut map) = self.revoked.lock() {
            map.insert(jti.to_string(), exp);
            tracing::info!("[JwtBlacklist] 已撤銷 token jti={}", jti);
        }
    }

    /// 檢查 JWT 是否已被撤銷
    pub fn is_revoked(&self, jti: &str) -> bool {
        if let Ok(map) = self.revoked.lock() {
            map.contains_key(jti)
        } else {
            false // mutex 中毒時放行
        }
    }
}
