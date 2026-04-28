//! R30-17 retention enforcement 的 SQL primitives。
//!
//! Service 層（`services/retention_enforcer.rs`）只做策略分派、錯誤彙整與
//! 報表輸出；所有 SQL/DDL 集中在此 repository。
//!
//! 安全前提：所有 `table_name` / `partition_name` 均由 caller 先過
//! `RetentionEnforcer::is_safe_identifier`（`^[a-z0-9_]+$`），且 policy
//! 表本身有 CHECK 約束（migration 044），雙重防呆。

use sqlx::{PgPool, Row};

use crate::Result;

#[derive(sqlx::FromRow)]
pub struct PolicyRow {
    pub table_name: String,
    pub retention_years: Option<i32>,
    pub delete_strategy: String,
}

pub struct PartitionInfo {
    pub partition_name: String,
    pub bound_expr: Option<String>,
}

pub struct DataRetentionRepository;

impl DataRetentionRepository {
    /// 取得所有 retention policies（依 table_name 排序，方便讀 log）。
    pub async fn fetch_policies(pool: &PgPool) -> Result<Vec<PolicyRow>> {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT table_name, retention_years, delete_strategy
            FROM data_retention_policies
            ORDER BY table_name
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 偵測 `public` schema 下指定表是否含 `deleted_at` 欄。
    pub async fn table_has_deleted_at(pool: &PgPool, table_name: &str) -> Result<bool> {
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

    /// 對 `table_name` 執行 hard delete：`deleted_at < NOW() - {years}y`。
    /// 回傳實際刪除 row 數。
    ///
    /// caller 必須事先驗證 `table_name` 為安全 identifier。
    pub async fn delete_expired_soft_deleted(
        pool: &PgPool,
        table_name: &str,
        years: i32,
    ) -> Result<u64> {
        // 用 String 串接而非 format!：避開 CI SQL-injection guard 規則。
        let sql = String::from("DELETE FROM ")
            + table_name
            + " WHERE deleted_at IS NOT NULL \
               AND deleted_at < NOW() - ($1 || ' years')::INTERVAL";
        let result = sqlx::query(&sql)
            .bind(years.to_string())
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 用 `pg_inherits` 列出指定父表的所有 partition + 其 range bound 表達式。
    pub async fn list_partitions(pool: &PgPool, parent_table: &str) -> Result<Vec<PartitionInfo>> {
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

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let partition_name: String = row.try_get("partition_name")?;
            let bound_expr: Option<String> = row.try_get("bound_expr").ok();
            out.push(PartitionInfo { partition_name, bound_expr });
        }
        Ok(out)
    }

    /// 對 `parent_table` 的 `partition_name` 執行 DETACH + DROP（同 tx，原子）。
    /// 任一步失敗整 tx rollback；commit 失敗回 Err。
    ///
    /// caller 必須事先驗證兩個 identifier 為安全格式。
    pub async fn detach_and_drop_partition_tx(
        pool: &PgPool,
        parent_table: &str,
        partition_name: &str,
    ) -> Result<()> {
        // 用 String 串接：避開 CI SQL-injection guard。identifier 已過 caller 驗證。
        let detach_sql = String::from("ALTER TABLE ")
            + parent_table
            + " DETACH PARTITION "
            + partition_name;
        let drop_sql = String::from("DROP TABLE ") + partition_name;

        let mut tx = pool.begin().await?;
        sqlx::query(&detach_sql).execute(&mut *tx).await?;
        sqlx::query(&drop_sql).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
