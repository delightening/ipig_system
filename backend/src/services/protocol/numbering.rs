use chrono::Utc;
use sqlx::PgPool;

use super::ProtocolService;
use crate::{AppError, Result};

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
    /// 生成 APIG 編號
    /// 格式：APIG-{ROC}{03}
    /// {ROC} 為民國年（西元年 - 1911）
    /// {03} 為流水號（3位數，補零）
    ///
    /// 注意：需要避免重複使用已經轉換為 PIG 的編號
    /// 例如：如果 APIG-115001 已經變成 PIG-115001，則流水號 001 不應再被使用
    pub(super) async fn generate_apig_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        use chrono::Datelike;
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;

        // 查詢該民國年的所有 APIG 編號
        // 格式：APIG-{ROC年}{3位數流水號}，例如：APIG-114001, APIG-115001
        let apig_prefix = format!("APIG-{}", roc_year);
        let apig_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", apig_prefix))
        .fetch_all(pool)
        .await?;

        // 查詢該民國年的所有 PIG 編號（因為 PIG 編號可能曾經是 APIG 編號）
        // 格式：PIG-{ROC年}{3位數流水號}，例如：PIG-115001
        // 這些流水號不應再被用於新的 APIG 編號
        let iacuc_prefix = format!("PIG-{}", roc_year);
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", iacuc_prefix))
        .fetch_all(pool)
        .await?;

        // 解析 APIG 編號的流水號
        let apig_seqs: Vec<i32> = apig_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &apig_prefix))
            .collect();

        // 解析 PIG 編號的流水號（這些流水號曾經是 APIG 編號，不應重複使用）
        let iacuc_seqs: Vec<i32> = iacuc_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &iacuc_prefix))
            .collect();

        // 合併所有已使用的流水號（包括當前的 APIG 和曾經是 APIG 的 PIG）
        let mut all_used_seqs: Vec<i32> = apig_seqs;
        all_used_seqs.extend(iacuc_seqs);

        // 找出最大值
        let max_seq = all_used_seqs.iter().max().copied();

        // 下一個流水號（從1開始，如果沒有現有編號）
        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        // 確保流水號不超過999
        if seq > 999 {
            return Err(AppError::Internal(
                "APIG 編號流水號已達上限（999），無法生成新編號".to_string(),
            ));
        }

        Ok(format_protocol_no(&apig_prefix, seq))
    }

    /// 生成 IACUC 編號
    /// 格式：PIG-{ROC}{03}
    /// {ROC} 為民國年（西元年 - 1911）
    /// {03} 為流水號（3位數，補零）
    pub(super) async fn generate_iacuc_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        use chrono::Datelike;
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;

        // 查詢該民國年的所有 IACUC 編號
        // 格式：PIG-{ROC年}{3位數流水號}，例如：PIG-114017, PIG-115001
        let prefix = format!("PIG-{}", roc_year);
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
        )
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

        // 解析流水號並找出最大值
        // IACUC 編號格式：PIG-{ROC年}{3位數流水號}
        // 例如：PIG-114017 → ROC年=114, 流水號=017
        let max_seq = iacuc_nos
            .iter()
            .filter_map(|no| parse_no_sequence(no, &prefix))
            .max();

        // 下一個流水號（從1開始，如果沒有現有編號）
        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        // 確保流水號不超過999
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
