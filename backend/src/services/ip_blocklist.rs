//! R24-1: IP 黑名單服務
//!
//! 負責：
//! - 快取 active blocklist（30s TTL，middleware 高頻查詢路徑）
//! - 自動封鎖（R22-6 IDOR / R22-1 auth ratelimit / R22-16 honeypot）
//! - 手動維護（admin GET/POST/PATCH）

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::PgPool;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{AppError, Result};

const CACHE_TTL: Duration = Duration::from_secs(30);

/// 全域 active blocklist cache — middleware 每 request 查一次，30s 過期後重讀 DB
static CACHE: std::sync::LazyLock<RwLock<Option<(HashSet<IpAddr>, Instant)>>> =
    std::sync::LazyLock::new(|| RwLock::new(None));

pub struct IpBlocklistService;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct IpBlocklistEntry {
    pub id: Uuid,
    /// 文字形式的 IP（sqlx 未開 ipnetwork feature，DB 端以 `ip_address::text` 取出）
    pub ip_address: String,
    pub reason: String,
    pub source: String,
    pub alert_id: Option<Uuid>,
    pub blocked_at: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,
    pub hit_count: i64,
    pub last_hit_at: Option<DateTime<Utc>>,
    pub unblocked_at: Option<DateTime<Utc>>,
    pub unblocked_by: Option<Uuid>,
    pub unblocked_reason: Option<String>,
}

impl IpBlocklistService {
    /// Middleware 高頻呼叫：該 IP 是否目前封鎖中？
    pub async fn is_blocked(pool: &PgPool, ip: IpAddr) -> bool {
        // Cache hit
        if let Ok(guard) = CACHE.read() {
            if let Some((set, ts)) = guard.as_ref() {
                if ts.elapsed() < CACHE_TTL {
                    return set.contains(&ip);
                }
            }
        }

        // Cache miss / expired — reload
        let fresh = match Self::load_active_ips(pool).await {
            Ok(set) => set,
            Err(e) => {
                tracing::error!("[R24-1] Failed to load active blocklist: {e}");
                // Fail-open：DB 掛了時不能全站 503
                return false;
            }
        };
        let in_set = fresh.contains(&ip);
        if let Ok(mut guard) = CACHE.write() {
            *guard = Some((fresh, Instant::now()));
        }
        in_set
    }

    /// 強制失效快取，下次 is_blocked 會重讀 DB。寫入操作後呼叫。
    pub fn invalidate_cache() {
        if let Ok(mut guard) = CACHE.write() {
            *guard = None;
        }
    }

    /// 非同步遞增 hit_count（fire-and-forget，不阻擋 request）
    pub fn spawn_record_hit(pool: PgPool, ip: IpAddr) {
        tokio::spawn(async move {
            let ip_str = ip.to_string();
            let _ = sqlx::query(
                "UPDATE ip_blocklist SET hit_count = hit_count + 1, last_hit_at = NOW()
                 WHERE ip_address = $1::inet AND unblocked_at IS NULL",
            )
            .bind(ip_str)
            .execute(&pool)
            .await;
        });
    }

    /// 自動封鎖：由 R22-6 / R22-1 / R22-16 觸發。
    /// - ttl_hours = Some(n)：封 n 小時後自動解除（middleware 查詢時會過濾）
    /// - ttl_hours = None：永久封鎖
    pub async fn auto_block(
        pool: &PgPool,
        ip: &str,
        source: &str,
        alert_id: Option<Uuid>,
        reason: &str,
        ttl_hours: Option<i64>,
    ) {
        if ip.parse::<IpAddr>().is_err() {
            tracing::warn!("[R24-1] auto_block: skip invalid IP {ip}");
            return;
        }
        let blocked_until = ttl_hours.map(|h| Utc::now() + ChronoDuration::hours(h));

        // 注意：Postgres 不支援 ON CONFLICT 搭配 partial unique index；改為 best-effort INSERT。
        // 若同 IP 已有 active entry，重複 insert 會因 unique index 失敗，吞 error 並更新原 entry 的 TTL。
        let insert = sqlx::query(
            r#"
            INSERT INTO ip_blocklist (ip_address, reason, source, alert_id, blocked_until)
            VALUES ($1::inet, $2, $3, $4, $5)
            "#,
        )
        .bind(ip)
        .bind(reason)
        .bind(source)
        .bind(alert_id)
        .bind(blocked_until)
        .execute(pool)
        .await;

        let result = if insert.is_ok() {
            insert
        } else {
            // 已存在 active entry → 延長 TTL、更新 reason/source
            sqlx::query(
                r#"
                UPDATE ip_blocklist
                SET blocked_until = $2,
                    reason = $3,
                    source = $4,
                    alert_id = COALESCE($5, alert_id)
                WHERE ip_address = $1::inet AND unblocked_at IS NULL
                "#,
            )
            .bind(ip)
            .bind(blocked_until)
            .bind(reason)
            .bind(source)
            .bind(alert_id)
            .execute(pool)
            .await
        };

        match result {
            Ok(_) => {
                tracing::warn!(
                    "[R24-1] IP {ip} blocked (source={source}, ttl_hours={ttl_hours:?})"
                );
                Self::invalidate_cache();
            }
            Err(e) => {
                tracing::error!("[R24-1] auto_block {ip} failed: {e}");
            }
        }
    }

    /// 手動新增（admin 後台）
    pub async fn manual_add(
        pool: &PgPool,
        ip: IpAddr,
        reason: &str,
        ttl_hours: Option<i64>,
        by_user: Uuid,
    ) -> Result<Uuid> {
        let ip_str = ip.to_string();
        let blocked_until = ttl_hours.map(|h| Utc::now() + ChronoDuration::hours(h));
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO ip_blocklist (ip_address, reason, source, blocked_until, blocked_by)
            VALUES ($1::inet, $2, 'manual', $3, $4)
            RETURNING id
            "#,
        )
        .bind(ip_str)
        .bind(reason)
        .bind(blocked_until)
        .bind(by_user)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("manual_add: {e}")))?;
        Self::invalidate_cache();
        Ok(id)
    }

    /// 解除封鎖（admin 後台）— soft：設 unblocked_at
    pub async fn unblock(
        pool: &PgPool,
        id: Uuid,
        by_user: Uuid,
        reason: &str,
    ) -> Result<()> {
        let rows = sqlx::query(
            r#"
            UPDATE ip_blocklist
            SET unblocked_at = NOW(), unblocked_by = $2, unblocked_reason = $3
            WHERE id = $1 AND unblocked_at IS NULL
            "#,
        )
        .bind(id)
        .bind(by_user)
        .bind(reason)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("unblock: {e}")))?
        .rows_affected();
        if rows == 0 {
            return Err(AppError::NotFound("IP blocklist 項目不存在或已解除".into()));
        }
        Self::invalidate_cache();
        Ok(())
    }

    /// 列表（admin 後台）— active / history 切換
    pub async fn list(
        pool: &PgPool,
        only_active: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<IpBlocklistEntry>> {
        let rows = if only_active {
            sqlx::query_as::<_, IpBlocklistEntry>(
                r#"
                SELECT id, host(ip_address) AS ip_address, reason, source, alert_id,
                       blocked_at, blocked_until, blocked_by, hit_count, last_hit_at,
                       unblocked_at, unblocked_by, unblocked_reason
                FROM ip_blocklist
                WHERE unblocked_at IS NULL
                  AND (blocked_until IS NULL OR blocked_until > NOW())
                ORDER BY blocked_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, IpBlocklistEntry>(
                r#"
                SELECT id, host(ip_address) AS ip_address, reason, source, alert_id,
                       blocked_at, blocked_until, blocked_by, hit_count, last_hit_at,
                       unblocked_at, unblocked_by, unblocked_reason
                FROM ip_blocklist
                ORDER BY blocked_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        };
        rows.map_err(|e| AppError::Internal(format!("list blocklist: {e}")))
    }

    /// Internal：載入所有 active IP（middleware cache reload）
    /// 注意：INET::text 會回 CIDR 格式（127.0.0.1/32），IpAddr 無法解析；改用 host() 取純 IP
    async fn load_active_ips(pool: &PgPool) -> std::result::Result<HashSet<IpAddr>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT host(ip_address) FROM ip_blocklist
            WHERE unblocked_at IS NULL
              AND (blocked_until IS NULL OR blocked_until > NOW())
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|(s,)| s.parse::<IpAddr>().ok())
            .collect())
    }
}
