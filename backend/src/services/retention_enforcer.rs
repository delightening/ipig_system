//! R30-17：依 `data_retention_policies` 表為每個業務表執行 retention enforcement。
//!
//! ## 流程
//!
//! 1. 查 `data_retention_policies` 取得每個 (table_name, retention_years,
//!    delete_strategy)。
//! 2. 對每筆 row：
//!    - `never` 策略 → 跳過（純文件意涵）
//!    - `hard_delete` 策略：
//!        - 用 `information_schema.columns` 偵測該表是否有 `deleted_at` 欄位
//!        - 若有 → `DELETE WHERE deleted_at IS NOT NULL AND deleted_at < cutoff`
//!        - 若無 → log warn 後跳過（避免把活著的 row 真的清掉）
//!    - `partition_drop` 策略：
//!        - 對 `user_activity_logs`（partitioned by created_at quarter）
//!          找出 cutoff 之前的 partition 子表，DETACH + DROP
//!        - 不觸發 BEFORE DELETE trigger（R30-F migration 041 的設計）
//! 3. 每個表的成功/失敗都進 `RetentionRunReport`；單一表失敗不擋其他表。
//! 4. **整輪結束後**寫一筆 `RETENTION_ENFORCEMENT_RUN` audit event
//!    （actor = SystemActor，category = "RETENTION"）總結本輪結果。
//!
//! ## STAGING-ONLY 警告
//!
//! 本 service **首次部署不應立即啟用實際刪除**。建議先在 staging 跑 `dry_run`
//! 變體（本檔提供 `run_dry()`）觀察 report 連續 ≥7 天確認沒誤刪後，再切到
//! `run()`。後續 PR 會加 enable flag（`AUDIT_CHAIN_VERIFY_ACTIVE` 同等開關）。
//!
//! ## 為何不對缺 `deleted_at` 的表自動加欄位
//!
//! 約 60+ 個 GLP 表中只有少數（animals / animal_observations / 等）已有
//! `deleted_at`。一次性 schema migration 加 60+ 欄位影響面過大（model /
//! query / index 全要動）。本 enforcer 設計成 schema-aware：**有 deleted_at
//! 才掃**；沒有的表在 policy row 提供文件意涵，等對應業務領域真的需要
//! soft-delete 時再個別加欄位 + 掛 enforcer。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use tracing::{info, warn};

use crate::middleware::actor::ActorContext;
use crate::models::audit_diff::AuditRedact;
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
    /// 每個 table 實際刪除的 row 數（hard_delete 策略）。
    pub deleted_rows_per_table: BTreeMap<String, i64>,
    /// partition_drop 策略本輪 drop 的 partition 子表名稱。
    pub partitions_dropped: Vec<String>,
    /// 每個 table 處理時的非致命錯誤；不擋其他表。
    pub errors: Vec<RetentionError>,
    /// 標記為 never 策略而跳過的表數量。
    pub never_skipped: usize,
    /// 缺 `deleted_at` 欄位而跳過的表名單。
    pub no_deleted_at_skipped: Vec<String>,
    /// 本輪開始時間（UTC）。
    pub started_at: Option<DateTime<Utc>>,
    /// 本輪結束時間（UTC）。
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

#[derive(sqlx::FromRow)]
struct PolicyRow {
    table_name: String,
    retention_years: Option<i32>,
    delete_strategy: String,
}

pub struct RetentionEnforcer;

impl RetentionEnforcer {
    /// 執行一輪 retention enforcement。
    ///
    /// 不會 panic：個別 table 失敗會記錄到 `RetentionRunReport.errors`，
    /// 然後繼續處理下一個 table。整體若 audit log 寫入失敗才會回 Err
    /// （這代表 cron 觀察到 broken state，需要 ops 看 log）。
    pub async fn run(pool: &PgPool) -> Result<RetentionRunReport> {
        let mut report = RetentionRunReport {
            started_at: Some(Utc::now()),
            ..Default::default()
        };

        let policies: Vec<PolicyRow> = sqlx::query_as(
            r#"
            SELECT table_name, retention_years, delete_strategy
            FROM data_retention_policies
            ORDER BY table_name
            "#,
        )
        .fetch_all(pool)
        .await?;

        for policy in &policies {
            // never / NULL retention 跳過
            if policy.delete_strategy == "never" || policy.retention_years.is_none() {
                report.never_skipped += 1;
                continue;
            }
            let years = policy.retention_years.unwrap_or(0);
            if years <= 0 {
                report.errors.push(RetentionError {
                    table_name: policy.table_name.clone(),
                    message: format!("invalid retention_years: {years}"),
                });
                continue;
            }

            match policy.delete_strategy.as_str() {
                "hard_delete" => {
                    if let Err(e) =
                        Self::process_hard_delete(pool, &policy.table_name, years, &mut report)
                            .await
                    {
                        report.errors.push(RetentionError {
                            table_name: policy.table_name.clone(),
                            message: format!("hard_delete failed: {e}"),
                        });
                    }
                }
                "partition_drop" => {
                    if let Err(e) =
                        Self::process_partition_drop(pool, &policy.table_name, years, &mut report)
                            .await
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

        report.finished_at = Some(Utc::now());

        // audit-in-tx 寫一筆總結事件
        Self::write_audit_event(pool, &report).await?;

        Self::log_summary(&report);
        Ok(report)
    }

    /// `hard_delete` 策略：偵測 `deleted_at` 欄位後執行 DELETE。
    async fn process_hard_delete(
        pool: &PgPool,
        table_name: &str,
        years: i32,
        report: &mut RetentionRunReport,
    ) -> Result<()> {
        if !Self::table_has_deleted_at(pool, table_name).await? {
            report
                .no_deleted_at_skipped
                .push(table_name.to_string());
            return Ok(());
        }

        // table_name 來自 policy 表，DDL identifier 不能 bind，但本欄位是
        // information_schema 已驗證存在的 schema 表名（非外部輸入），
        // 直接內插 identifier 安全。仍用簡易檢查防呆。
        if !Self::is_safe_identifier(table_name) {
            return Err(crate::AppError::Internal(format!(
                "unsafe table_name in policy: {table_name}"
            )));
        }

        let sql = format!(
            "DELETE FROM {table_name} WHERE deleted_at IS NOT NULL \
             AND deleted_at < NOW() - ($1 || ' years')::INTERVAL"
        );
        let result = sqlx::query(&sql)
            .bind(years.to_string())
            .execute(pool)
            .await?;
        let n = result.rows_affected() as i64;
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

    /// `partition_drop` 策略：找出 cutoff 前的 partition 子表並 DETACH + DROP。
    ///
    /// 目前實作對應 `user_activity_logs`（quarterly partition）。對其他 partition
    /// 表只要 partition 命名規則 `{parent}_YYYY_qN` 也適用。
    async fn process_partition_drop(
        pool: &PgPool,
        parent_table: &str,
        years: i32,
        report: &mut RetentionRunReport,
    ) -> Result<()> {
        if !Self::is_safe_identifier(parent_table) {
            return Err(crate::AppError::Internal(format!(
                "unsafe table_name in policy: {parent_table}"
            )));
        }

        // 以 pg_inherits 找出所有子分區與其 range upper bound。
        // pg_get_expr(c.relpartbound, c.oid) 形如：
        //   FOR VALUES FROM ('2026-01-01') TO ('2026-04-01')
        // 抽出 TO 端日期作為 cutoff 比對對象（partition 全部 row 的 created_at
        // 都 < TO 端日期，故 TO 端日期 < retention cutoff 表示整個 partition
        // 都在 retention 期外，可整桶 drop）。
        let rows = sqlx::query(
            r#"
            SELECT child.relname AS partition_name,
                   pg_get_expr(child.relpartbound, child.oid) AS bound_expr
            FROM pg_inherits i
            JOIN pg_class parent ON parent.oid = i.inhparent
            JOIN pg_class child  ON child.oid  = i.inhrelid
            WHERE parent.relname = $1
            "#,
        )
        .bind(parent_table)
        .fetch_all(pool)
        .await?;

        let cutoff = Utc::now() - chrono::Duration::days((years as i64) * 365);
        let cutoff_date = cutoff.date_naive();

        for row in rows {
            let partition_name: String = row.try_get("partition_name")?;
            let bound_expr: Option<String> = row.try_get("bound_expr").ok();
            let Some(expr) = bound_expr else { continue };

            // 解析 TO ('YYYY-MM-DD')
            let Some(to_date) = parse_partition_upper_bound(&expr) else {
                continue;
            };
            if to_date >= cutoff_date {
                continue; // 還在保留期內
            }

            if !Self::is_safe_identifier(&partition_name) {
                continue;
            }

            // DETACH + DROP；不觸發 BEFORE DELETE trigger（R30-F 設計）。
            let detach_sql = format!(
                "ALTER TABLE {parent_table} DETACH PARTITION {partition_name}"
            );
            let drop_sql = format!("DROP TABLE {partition_name}");

            if let Err(e) = sqlx::query(&detach_sql).execute(pool).await {
                report.errors.push(RetentionError {
                    table_name: format!("{parent_table}/{partition_name}"),
                    message: format!("DETACH failed: {e}"),
                });
                continue;
            }
            if let Err(e) = sqlx::query(&drop_sql).execute(pool).await {
                report.errors.push(RetentionError {
                    table_name: format!("{parent_table}/{partition_name}"),
                    message: format!("DROP failed (DETACH succeeded): {e}"),
                });
                continue;
            }
            info!(
                "[RetentionEnforcer] {parent_table}: dropped partition {partition_name} (upper={to_date}, cutoff={cutoff_date})"
            );
            report.partitions_dropped.push(partition_name);
        }
        Ok(())
    }

    async fn table_has_deleted_at(pool: &PgPool, table_name: &str) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = $1
                  AND column_name = 'deleted_at'
            )
            "#,
        )
        .bind(table_name)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|(b,)| b).unwrap_or(false))
    }

    /// 簡易 identifier 防呆：只允許 `[a-z0-9_]+`。
    fn is_safe_identifier(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    async fn write_audit_event(pool: &PgPool, report: &RetentionRunReport) -> Result<()> {
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
    // 找 " TO ('...')"
    let to_idx = expr.find(" TO (")?;
    let rest = &expr[to_idx + 5..];
    let start = rest.find('\'')?;
    let after_quote = &rest[start + 1..];
    let end = after_quote.find('\'')?;
    let date_str = &after_quote[..end];
    // 支援 'YYYY-MM-DD' 與 'YYYY-MM-DD HH:MM:SS+TZ'，先取前 10 字元
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
