//! Audit log 相關 SQL primitives。
//!
//! 與 `services/audit.rs`（HMAC chain 寫入 / 驗證主邏輯）職責分工：
//! 此 repository 只放純查詢類 SQL（report / monitor 用途），不涉及 HMAC 計算。

use sqlx::PgPool;

use crate::Result;

pub struct AuditLogRepository;

impl AuditLogRepository {
    /// R28-5：尚未 backfill `hmac_version` 的 row 數量。
    ///
    /// 條件：`hmac_version IS NULL AND integrity_hash IS NOT NULL`
    /// （SECURITY 類別 row `integrity_hash` 為 NULL，合法非鏈內成員，排除）。
    ///
    /// 用於 scheduler `register_hmac_legacy_gauge_job` 每 10 分鐘更新
    /// `ipig_audit_hmac_legacy_rows{version="null"}` gauge。目標 → 0。
    /// 配合 partial index `idx_audit_hmac_null` (migration 045) 走 index-only scan。
    pub async fn count_legacy_hmac_rows(pool: &PgPool) -> Result<i64> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_activity_logs \
             WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL",
        )
        .fetch_one(pool)
        .await?;
        Ok(n)
    }
}
