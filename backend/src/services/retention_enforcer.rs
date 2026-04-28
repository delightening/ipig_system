//! R30-17：依 `data_retention_policies` 表為每個業務表執行 retention enforcement。
//!
//! ## 職責邊界
//!
//! 本 service **只做策略分派 / 錯誤彙整 / 報表輸出 / audit 寫入**；
//! 所有 SQL/DDL primitives 集中在 `repositories::data_retention::DataRetentionRepository`。
//!
//! ## 流程
//!
//! 1. `repo.fetch_policies()` 取得每個 (table_name, retention_years, delete_strategy)。
//! 2. 對每筆 row：
//!    - `never` 策略 → 跳過（純文件意涵）
//!    - `hard_delete` 策略：先偵測 `deleted_at` 欄位，沒有 → skip + log；有 →
//!      `repo.delete_expired_soft_deleted`
//!    - `partition_drop` 策略：列 partition → 解析 upper bound → cutoff 之前
//!      的 partition 走 `repo.detach_and_drop_partition_tx`（DETACH+DROP 同 tx）
//! 3. 每個表的成功/失敗都進 `RetentionRunReport`；單一表失敗不擋其他表。
//! 4. **整輪結束後**寫一筆 `RETENTION_ENFORCEMENT_RUN` audit event。
//!
//! ## STAGING-ONLY / Feature Flag
//!
//! Scheduler 端有 `RETENTION_ENFORCER_ENABLED` env gate，預設 `false`。
//! 首次部署會 skip 整個 cron，連續觀察 dry-run 報表 ≥7 天確認沒誤刪後，
//! 再設為 `true` 啟用實刪。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::actor::ActorContext;
use crate::models::audit_diff::AuditRedact;
use crate::repositories::data_retention::{
    DataRetentionRepository, PartitionInfo, PolicyRow,
};
use crate::services::audit::{ActivityLogEntry, AuditService};
use crate::Result;

/// `RETENTION_ENFORCEMENT_RUN` 事件的 actor reason 標籤。
const ACTOR_REASON: &str = "retention_enforcer";

/// audit event_category（與既有規範 `ANIMAL` / `SECURITY` 等對齊）。
const AUDIT_CATEGORY: &str = "RETENTION";
const AUDIT_EVENT_TYPE: &str = "RETENTION_ENFORCEMENT_RUN";

/// 一次 enforcer 執行報表。
#[derive(Debug, Default)]
pub struct RetentionRunReport {
    pub deleted_rows_per_table: BTreeMap<String, i64>,
    pub partitions_dropped: Vec<String>,
    pub errors: Vec<RetentionError>,
    pub never_skipped: usize,
    pub no_deleted_at_skipped: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct RetentionError {
    pub table_name: String,
    pub message: String,
}

/// audit log 寫入用的可序列化摘要（DataDiff::create_only 需 AuditRedact + Serialize）。
#[derive(Debug, Serialize)]
struct RetentionRunSummary {
    total_deleted_rows: i64,
    deleted_rows_per_table: BTreeMap<String, i64>,
    partitions_dropped: Vec<String>,
    never_skipped: usize,
    no_deleted_at_skipped: Vec<String>,
    errors: Vec<RetentionErrorSummary>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
struct RetentionErrorSummary {
    table: String,
    message: String,
}

impl RetentionRunSummary {
    fn from_report(report: &RetentionRunReport) -> Self {
        Self {
            total_deleted_rows: report.deleted_rows_per_table.values().sum(),
            deleted_rows_per_table: report.deleted_rows_per_table.clone(),
            partitions_dropped: report.partitions_dropped.clone(),
            never_skipped: report.never_skipped,
            no_deleted_at_skipped: report.no_deleted_at_skipped.clone(),
            errors: report
                .errors
                .iter()
                .map(|e| RetentionErrorSummary {
                    table: e.table_name.clone(),
                    message: e.message.clone(),
                })
                .collect(),
            started_at: report.started_at,
            finished_at: report.finished_at,
        }
    }
}

impl AuditRedact for RetentionRunSummary {}

pub struct RetentionEnforcer;

impl RetentionEnforcer {
    /// 執行一輪 retention enforcement。
    ///
    /// 不會 panic：個別 table 失敗會記錄到 `RetentionRunReport.errors`，
    /// 然後繼續處理下一個 table。整體若 audit log 寫入失敗才會回 Err
    /// （這代表 cron 觀察到 broken state，需要 ops 看 log）。
    pub async fn run(pool: &sqlx::PgPool) -> Result<RetentionRunReport> {
        let mut report = RetentionRunReport {
            started_at: Some(Utc::now()),
            ..Default::default()
        };

        let policies = DataRetentionRepository::fetch_policies(pool).await?;
        for policy in &policies {
            Self::dispatch_policy(pool, policy, &mut report).await;
        }

        report.finished_at = Some(Utc::now());
        Self::write_audit_event(pool, &report).await?;
        Self::log_summary(&report);
        Ok(report)
    }

    /// 單筆 policy 分派 + 錯誤收斂。
    async fn dispatch_policy(
        pool: &sqlx::PgPool,
        policy: &PolicyRow,
        report: &mut RetentionRunReport,
    ) {
        // never / NULL retention 跳過
        if policy.delete_strategy == "never" || policy.retention_years.is_none() {
            report.never_skipped += 1;
            return;
        }
        let years = policy.retention_years.unwrap_or(0);
        if years <= 0 {
            report.errors.push(RetentionError {
                table_name: policy.table_name.clone(),
                message: format!("invalid retention_years: {years}"),
            });
            return;
        }

        match policy.delete_strategy.as_str() {
            "hard_delete" => {
                if let Err(e) =
                    Self::process_hard_delete(pool, &policy.table_name, years, report).await
                {
                    report.errors.push(RetentionError {
                        table_name: policy.table_name.clone(),
                        message: format!("hard_delete failed: {e}"),
                    });
                }
            }
            "partition_drop" => {
                if let Err(e) =
                    Self::process_partition_drop(pool, &policy.table_name, years, report).await
                {
                    report.errors.push(RetentionError {
                        table_name: policy.table_name.clone(),
                        message: format!("partition_drop failed: {e}"),
                    });
                }
            }
            other => {
                report.errors.push(RetentionError {
                    table_name: policy.table_name.clone(),
                    message: format!("unknown delete_strategy: {other}"),
                });
            }
        }
    }

    /// `hard_delete` 策略：偵測 `deleted_at` 欄位後委派 repository 執行。
    async fn process_hard_delete(
        pool: &sqlx::PgPool,
        table_name: &str,
        years: i32,
        report: &mut RetentionRunReport,
    ) -> Result<()> {
        if !DataRetentionRepository::table_has_deleted_at(pool, table_name).await? {
            report.no_deleted_at_skipped.push(table_name.to_string());
            return Ok(());
        }
        if !Self::is_safe_identifier(table_name) {
            return Err(crate::AppError::Internal(format!(
                "unsafe table_name in policy: {table_name}"
            )));
        }
        let n =
            DataRetentionRepository::delete_expired_soft_deleted(pool, table_name, years).await?
                as i64;
        if n > 0 {
            info!(
                "[RetentionEnforcer] {table_name}: hard-deleted {n} row(s) older than {years} years"
            );
        }
        report
            .deleted_rows_per_table
            .insert(table_name.to_string(), n);
        Ok(())
    }

    /// `partition_drop` 策略：列 partition → cutoff 比對 → DETACH+DROP（tx 由 repo 包）。
    async fn process_partition_drop(
        pool: &sqlx::PgPool,
        parent_table: &str,
        years: i32,
        report: &mut RetentionRunReport,
    ) -> Result<()> {
        if !Self::is_safe_identifier(parent_table) {
            return Err(crate::AppError::Internal(format!(
                "unsafe table_name in policy: {parent_table}"
            )));
        }

        let partitions = DataRetentionRepository::list_partitions(pool, parent_table).await?;
        let cutoff = Utc::now() - chrono::Duration::days((years as i64) * 365);
        let cutoff_date = cutoff.date_naive();

        for PartitionInfo { partition_name, bound_expr } in partitions {
            let Some(expr) = bound_expr else { continue };
            let Some(to_date) = parse_partition_upper_bound(&expr) else {
                continue;
            };
            if to_date >= cutoff_date {
                continue;
            }
            if !Self::is_safe_identifier(&partition_name) {
                continue;
            }
            match DataRetentionRepository::detach_and_drop_partition_tx(
                pool,
                parent_table,
                &partition_name,
            )
            .await
            {
                Ok(()) => {
                    info!(
                        "[RetentionEnforcer] {parent_table}: dropped partition {partition_name} \
                         (upper={to_date}, cutoff={cutoff_date})"
                    );
                    report.partitions_dropped.push(partition_name);
                }
                Err(e) => {
                    report.errors.push(RetentionError {
                        table_name: format!("{parent_table}/{partition_name}"),
                        message: format!("detach+drop failed: {e}"),
                    });
                }
            }
        }
        Ok(())
    }

    /// 簡易 identifier 防呆：只允許 `[a-z0-9_]+`。
    pub fn is_safe_identifier(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    async fn write_audit_event(pool: &sqlx::PgPool, report: &RetentionRunReport) -> Result<()> {
        let actor = ActorContext::System {
            reason: ACTOR_REASON,
        };
        let summary = RetentionRunSummary::from_report(report);

        let mut tx = pool.begin().await?;
        AuditService::log_activity_tx(
            &mut tx,
            &actor,
            ActivityLogEntry {
                event_category: AUDIT_CATEGORY,
                event_type: AUDIT_EVENT_TYPE,
                entity: None,
                data_diff: Some(crate::models::audit_diff::DataDiff::create_only(&summary)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    fn log_summary(report: &RetentionRunReport) {
        let total_deleted: i64 = report.deleted_rows_per_table.values().sum();
        let elapsed = match (report.started_at, report.finished_at) {
            (Some(s), Some(e)) => (e - s).num_milliseconds(),
            _ => -1,
        };
        info!(
            "[RetentionEnforcer] run finished: deleted={} rows across {} table(s), \
             {} partitions dropped, {} table(s) skipped (never), {} table(s) skipped (no deleted_at), \
             {} error(s), elapsed={}ms",
            total_deleted,
            report.deleted_rows_per_table.iter().filter(|(_, n)| **n > 0).count(),
            report.partitions_dropped.len(),
            report.never_skipped,
            report.no_deleted_at_skipped.len(),
            report.errors.len(),
            elapsed,
        );
        for e in &report.errors {
            warn!("[RetentionEnforcer] error in {}: {}", e.table_name, e.message);
        }
    }
}

/// 從 PostgreSQL `pg_get_expr(relpartbound)` 結果抽出 TO 端日期。
///
/// 預期形式（單欄分區，date / timestamptz）：
///   `FOR VALUES FROM ('2026-01-01') TO ('2026-04-01')`
///
/// 找不到就回 None（呼叫端應視為「不是時間 range partition、不可推斷
/// retention」並跳過）。
fn parse_partition_upper_bound(expr: &str) -> Option<chrono::NaiveDate> {
    let to_idx = expr.find(" TO (")?;
    let rest = &expr[to_idx + 5..];
    let start = rest.find('\'')?;
    let after_quote = &rest[start + 1..];
    let end = after_quote.find('\'')?;
    let date_str = &after_quote[..end];
    let head = if date_str.len() >= 10 { &date_str[..10] } else { date_str };
    chrono::NaiveDate::parse_from_str(head, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_upper_bound_simple_date() {
        let s = "FOR VALUES FROM ('2026-01-01') TO ('2026-04-01')";
        assert_eq!(
            parse_partition_upper_bound(s),
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
        );
    }

    #[test]
    fn parse_upper_bound_timestamptz() {
        let s = "FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-04-01 00:00:00+00')";
        assert_eq!(
            parse_partition_upper_bound(s),
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
        );
    }

    #[test]
    fn parse_upper_bound_missing_returns_none() {
        assert!(parse_partition_upper_bound("FOR VALUES IN ('a', 'b')").is_none());
        assert!(parse_partition_upper_bound("FOR VALUES FROM ('2026-01-01')").is_none());
    }

    #[test]
    fn safe_identifier_accepts_typical_table_names() {
        assert!(RetentionEnforcer::is_safe_identifier("animal_observations"));
        assert!(RetentionEnforcer::is_safe_identifier("user_activity_logs_2026_q1"));
    }

    #[test]
    fn safe_identifier_rejects_injection_attempts() {
        assert!(!RetentionEnforcer::is_safe_identifier("animals; DROP TABLE"));
        assert!(!RetentionEnforcer::is_safe_identifier(""));
        assert!(!RetentionEnforcer::is_safe_identifier("Animals"));
        assert!(!RetentionEnforcer::is_safe_identifier("a\"b"));
    }
}
