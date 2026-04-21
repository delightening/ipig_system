use chrono::Utc;
use sqlx::{Postgres, Transaction};

use super::ProtocolService;
use crate::{AppError, Result};

/// 所有 protocol 編號產生函式共用的 advisory lock key。
///
/// 三個 generator（`generate_apig_no` / `generate_iacuc_no` /
/// `generate_apig_nos_batch`）**共用同一個 lock**，因 APIG 與 PIG 編號的
/// 流水號空間彼此交疊（APIG-115001 approved 後會變成 PIG-115001），避免
/// 不同 generator 在同一 tx 之外併發時互相讀到過時的 max_seq。
///
/// 搭配 `pg_advisory_xact_lock`：tx commit/rollback 時自動釋放，不會卡死。
const IACUC_LOCK_KEY: &str = "protocol_iacuc_number_gen";

/// 於 transaction 內取得 protocol-numbering advisory lock。
/// 並發執行時後到的 tx 會在此阻塞，直到先到的 tx 結束。
async fn acquire_numbering_lock(tx: &mut Transaction<'_, Postgres>) -> Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(IACUC_LOCK_KEY)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// 從編號字串解析流水號。
/// 例如：`parse_no_sequence("APIG-115003", "APIG-115")` → `Some(3)`
pub(super) fn parse_no_sequence(no: &str, prefix: &str) -> Option<i32> {
    if !no.starts_with(prefix) || no.len() <= prefix.len() {
        return None;
    }
    no[prefix.len()..].parse::<i32>().ok()
}

/// 將前綴與流水號格式化為 3 位補零編號。
/// 例如：`format_protocol_no("APIG-115", 7)` → `"APIG-115007"`
pub(super) fn format_protocol_no(prefix: &str, seq: i32) -> String {
    format!("{}{:03}", prefix, seq)
}

impl ProtocolService {
    /// Pool-based wrapper：在獨立 mini-tx 內呼叫 `generate_apig_no`。
    ///
    /// **注意**：這是 **partial** race protection。advisory lock 序列化 gen 階段，
    /// 但 gen commit 後到呼叫者執行 UPDATE 前仍有短暫視窗。完整 race 修復需要
    /// 呼叫者本身是 transaction-aware（見 R26-7 `change_status` 完整重構）。
    /// `submit()` 已是 tx-aware 不走此 wrapper。
    pub(super) async fn generate_apig_no_pool(pool: &sqlx::PgPool) -> Result<String> {
        let mut tx = pool.begin().await?;
        let no = Self::generate_apig_no(&mut tx).await?;
        tx.commit().await?;
        Ok(no)
    }

    /// Pool-based wrapper for `generate_iacuc_no`（見 `generate_apig_no_pool` 說明）。
    pub(super) async fn generate_iacuc_no_pool(pool: &sqlx::PgPool) -> Result<String> {
        let mut tx = pool.begin().await?;
        let no = Self::generate_iacuc_no(&mut tx).await?;
        tx.commit().await?;
        Ok(no)
    }

    /// Pool-based wrapper for `generate_apig_nos_batch`。
    pub(super) async fn generate_apig_nos_batch_pool(
        pool: &sqlx::PgPool,
        count: usize,
    ) -> Result<Vec<String>> {
        let mut tx = pool.begin().await?;
        let nos = Self::generate_apig_nos_batch(&mut tx, count).await?;
        tx.commit().await?;
        Ok(nos)
    }

    /// 生成 APIG 編號（`APIG-{ROC}{03}`，ROC = 西元年 - 1911）。
    ///
    /// 需 transaction 參數以支援 advisory lock —— lock 讓兩個同時核准計畫
    /// 的 request 序列化執行，避免 `max + 1` read-modify-write 產生重複
    /// 編號（對應 CRIT-01）。
    ///
    /// 注意：查詢時**同時考慮 APIG-* 與 PIG-*** 的流水號，避免重複使用
    /// 已經轉換為 PIG 的編號（例如 APIG-115001 approved 後成為 PIG-115001，
    /// 流水號 001 就不應再被新的 APIG 使用）。
    pub(super) async fn generate_apig_no(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<String> {
        acquire_numbering_lock(tx).await?;

        let now = Utc::now();
        use chrono::Datelike;
        let roc_year = now.year() - 1911;

        let apig_prefix = format!("APIG-{}", roc_year);
        let apig_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", apig_prefix))
        .fetch_all(&mut **tx)
        .await?;

        let iacuc_prefix = format!("PIG-{}", roc_year);
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", iacuc_prefix))
        .fetch_all(&mut **tx)
        .await?;

        // 合併 APIG + PIG 的流水號找最大值
        let max_seq = apig_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &apig_prefix))
            .chain(iacuc_nos.iter().filter_map(|no| parse_no_sequence(no, &iacuc_prefix)))
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        if seq > 999 {
            return Err(AppError::Internal(
                "APIG 編號流水號已達上限（999），無法生成新編號".to_string(),
            ));
        }

        Ok(format_protocol_no(&apig_prefix, seq))
    }

    /// 批量生成 N 個 APIG 編號（單一 tx + 單次 lock）。
    ///
    /// 注意：批量場景已序列化成 tx 內操作，之前的 `tokio::try_join!` 不適用
    /// （單一 `&mut Transaction` 不可同時被兩個 future 借用），改為循序等候
    /// 兩個 SELECT。效能影響極小（批量核准很罕見）。
    pub(super) async fn generate_apig_nos_batch(
        tx: &mut Transaction<'_, Postgres>,
        count: usize,
    ) -> Result<Vec<String>> {
        if count == 0 {
            return Ok(Vec::new());
        }

        acquire_numbering_lock(tx).await?;

        let now = Utc::now();
        use chrono::Datelike;
        let roc_year = now.year() - 1911;

        let apig_prefix = format!("APIG-{}", roc_year);
        let iacuc_prefix = format!("PIG-{}", roc_year);

        // 序列化執行（tx 不可被兩個 future 同時借用）
        let apig_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", apig_prefix))
        .fetch_all(&mut **tx)
        .await?;
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", iacuc_prefix))
        .fetch_all(&mut **tx)
        .await?;

        let mut all_used: Vec<i32> = apig_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &apig_prefix))
            .chain(iacuc_nos.iter().filter_map(|no| parse_no_sequence(no, &iacuc_prefix)))
            .collect();
        all_used.sort_unstable();

        let max_seq = all_used.last().copied().unwrap_or(0);
        let start = max_seq + 1;

        if start + count as i32 - 1 > 999 {
            return Err(AppError::Internal(
                "APIG 編號流水號將超過上限（999）".to_string(),
            ));
        }

        Ok((start..start + count as i32)
            .map(|seq| format_protocol_no(&apig_prefix, seq))
            .collect())
    }

    /// 生成 IACUC 編號（`PIG-{ROC}{03}`，approved 計畫使用）。
    ///
    /// 需 transaction 參數以支援 advisory lock（對應 CRIT-01，同 `generate_apig_no`）。
    pub(super) async fn generate_iacuc_no(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<String> {
        acquire_numbering_lock(tx).await?;

        let now = Utc::now();
        use chrono::Datelike;
        let roc_year = now.year() - 1911;

        let prefix = format!("PIG-{}", roc_year);
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", prefix))
        .fetch_all(&mut **tx)
        .await?;

        let max_seq = iacuc_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &prefix))
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        if seq > 999 {
            return Err(AppError::Internal(
                "IACUC 編號流水號已達上限（999），無法生成新編號".to_string(),
            ));
        }

        Ok(format_protocol_no(&prefix, seq))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_protocol_no, parse_no_sequence};

    // --- parse_no_sequence ---

    #[test]
    fn test_parse_no_sequence_normal() {
        assert_eq!(parse_no_sequence("APIG-115001", "APIG-115"), Some(1));
        assert_eq!(parse_no_sequence("APIG-115042", "APIG-115"), Some(42));
        assert_eq!(parse_no_sequence("PIG-114017", "PIG-114"), Some(17));
    }

    #[test]
    fn test_parse_no_sequence_wrong_prefix() {
        assert_eq!(parse_no_sequence("APIG-115001", "PIG-115"), None);
        assert_eq!(parse_no_sequence("PIG-115001", "APIG-115"), None);
    }

    #[test]
    fn test_parse_no_sequence_non_numeric_suffix() {
        assert_eq!(parse_no_sequence("APIG-115abc", "APIG-115"), None);
        assert_eq!(parse_no_sequence("APIG-115", "APIG-115"), None);
    }

    #[test]
    fn test_parse_no_sequence_exact_prefix() {
        // 前綴長度等於字串長度（無後綴）應回 None
        assert_eq!(parse_no_sequence("PIG-115", "PIG-115"), None);
    }

    // --- format_protocol_no ---

    #[test]
    fn test_format_protocol_no_zero_pad() {
        assert_eq!(format_protocol_no("APIG-115", 1), "APIG-115001");
        assert_eq!(format_protocol_no("PIG-114", 17), "PIG-114017");
        assert_eq!(format_protocol_no("APIG-115", 999), "APIG-115999");
    }

    #[test]
    fn test_format_protocol_no_over_three_digits() {
        // 超過 999 時不補零（溢位由呼叫端阻擋）
        assert_eq!(format_protocol_no("PIG-115", 1000), "PIG-1151000");
    }
}
