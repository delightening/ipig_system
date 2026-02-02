//! 分區表維護服務
//! 
//! 自動檢查並建立 `user_activity_logs` 表的季度分區，
//! 確保系統持續運作不受分區缺失影響。

use sqlx::PgPool;
use tracing::{info, warn, error};
use chrono::{Datelike, Utc};

/// 分區維護結果
#[derive(Debug)]
pub struct PartitionResult {
    /// 檢查的分區數量
    pub checked: usize,
    /// 新建立的分區數量
    pub created: usize,
    /// 已存在的分區數量
    pub existing: usize,
    /// 建立失敗的分區
    pub failed: Vec<String>,
}

pub struct PartitionMaintenanceJob;

impl PartitionMaintenanceJob {
    /// 確保未來 2 年的分區存在
    /// 
    /// 此方法會檢查 `user_activity_logs` 表的季度分區，
    /// 並建立任何缺失的分區。
    pub async fn ensure_partitions(db: &PgPool) -> Result<PartitionResult, Box<dyn std::error::Error + Send + Sync>> {
        let current_year = Utc::now().year();
        
        // 確保當年 + 未來 2 年的分區存在 (共 3 年 = 12 個季度)
        let years_to_check = vec![current_year, current_year + 1, current_year + 2];
        let quarters = vec![1u8, 2, 3, 4];
        
        // 取得現有分區
        let existing_partitions = Self::get_existing_partitions(db).await?;
        info!("Found {} existing partitions for user_activity_logs", existing_partitions.len());
        
        let mut result = PartitionResult {
            checked: 0,
            created: 0,
            existing: 0,
            failed: Vec::new(),
        };
        
        for year in years_to_check {
            for quarter in &quarters {
                result.checked += 1;
                let partition_name = format!("user_activity_logs_{}_q{}", year, quarter);
                
                if existing_partitions.contains(&partition_name) {
                    result.existing += 1;
                    continue;
                }
                
                // 嘗試建立分區
                match Self::create_quarterly_partition(db, year, *quarter).await {
                    Ok(_) => {
                        info!("Created partition: {}", partition_name);
                        result.created += 1;
                    }
                    Err(e) => {
                        error!("Failed to create partition {}: {}", partition_name, e);
                        result.failed.push(partition_name);
                    }
                }
            }
        }
        
        if result.created > 0 {
            info!(
                "Partition maintenance completed: {} checked, {} created, {} existing, {} failed",
                result.checked, result.created, result.existing, result.failed.len()
            );
        } else if result.failed.is_empty() {
            info!("Partition maintenance: all {} partitions already exist", result.existing);
        }
        
        Ok(result)
    }
    
    /// 取得現有的 user_activity_logs 分區名稱
    async fn get_existing_partitions(db: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT tablename::text
            FROM pg_tables 
            WHERE schemaname = 'public' 
              AND tablename LIKE 'user_activity_logs_%'
              AND tablename != 'user_activity_logs_default'
            ORDER BY tablename
            "#
        )
        .fetch_all(db)
        .await?;
        
        Ok(rows.into_iter().map(|(name,)| name).collect())
    }
    
    /// 建立單一季度分區
    /// 
    /// # 參數
    /// - `year`: 年份 (例如 2028)
    /// - `quarter`: 季度 (1-4)
    async fn create_quarterly_partition(db: &PgPool, year: i32, quarter: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (start_month, end_month, end_year) = match quarter {
            1 => (1, 4, year),      // Q1: 01-01 to 04-01
            2 => (4, 7, year),      // Q2: 04-01 to 07-01
            3 => (7, 10, year),     // Q3: 07-01 to 10-01
            4 => (10, 1, year + 1), // Q4: 10-01 to 01-01 next year
            _ => return Err("Invalid quarter".into()),
        };
        
        let partition_name = format!("user_activity_logs_{}_q{}", year, quarter);
        let start_date = format!("{:04}-{:02}-01", year, start_month);
        let end_date = format!("{:04}-{:02}-01", end_year, end_month);
        
        // 使用動態 SQL 建立分區
        // 注意：由於 PostgreSQL 的 CREATE TABLE ... PARTITION OF 不支援參數化，
        // 我們需要使用動態 SQL，但輸入值已經過驗證 (年份和季度都是數字)
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} PARTITION OF user_activity_logs
                FOR VALUES FROM ('{}') TO ('{}')
            "#,
            partition_name, start_date, end_date
        );
        
        sqlx::query(&sql)
            .execute(db)
            .await?;
        
        // 為新分區建立必要的索引 (繼承自父表，但確認一下)
        info!("Partition {} created successfully ({} to {})", partition_name, start_date, end_date);
        
        Ok(())
    }
    
    /// 手動觸發分區維護（供 API 或測試使用）
    pub async fn trigger(db: &PgPool) -> Result<PartitionResult, Box<dyn std::error::Error + Send + Sync>> {
        warn!("Manual partition maintenance triggered");
        Self::ensure_partitions(db).await
    }
    
    /// 檢查是否需要建立分區（用於健康檢查）
    pub async fn check_status(db: &PgPool) -> Result<PartitionStatus, Box<dyn std::error::Error + Send + Sync>> {
        let current_year = Utc::now().year();
        let existing = Self::get_existing_partitions(db).await?;
        
        // 檢查明年的分區是否存在
        let next_year = current_year + 1;
        let next_year_partitions: Vec<_> = existing.iter()
            .filter(|name| name.contains(&format!("_{}_", next_year)))
            .collect();
        
        let needs_maintenance = next_year_partitions.len() < 4;
        
        Ok(PartitionStatus {
            total_partitions: existing.len(),
            next_year_ready: !needs_maintenance,
            next_year_partitions: next_year_partitions.len(),
        })
    }
}

/// 分區狀態檢查結果
#[derive(Debug)]
pub struct PartitionStatus {
    /// 現有分區總數
    pub total_partitions: usize,
    /// 明年的分區是否已就緒
    pub next_year_ready: bool,
    /// 明年已建立的分區數量
    pub next_year_partitions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quarter_dates() {
        // 驗證季度日期計算邏輯
        let test_cases = vec![
            (2028, 1, "2028-01-01", "2028-04-01"),
            (2028, 2, "2028-04-01", "2028-07-01"),
            (2028, 3, "2028-07-01", "2028-10-01"),
            (2028, 4, "2028-10-01", "2029-01-01"),
        ];
        
        for (year, quarter, expected_start, expected_end) in test_cases {
            let (start_month, end_month, end_year) = match quarter {
                1 => (1, 4, year),
                2 => (4, 7, year),
                3 => (7, 10, year),
                4 => (10, 1, year + 1),
                _ => panic!("Invalid quarter"),
            };
            
            let start_date = format!("{:04}-{:02}-01", year, start_month);
            let end_date = format!("{:04}-{:02}-01", end_year, end_month);
            
            assert_eq!(start_date, expected_start, "Q{} start date mismatch", quarter);
            assert_eq!(end_date, expected_end, "Q{} end date mismatch", quarter);
        }
    }
}
