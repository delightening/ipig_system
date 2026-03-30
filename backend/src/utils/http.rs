// HTTP 相關純函式工具

/// 建構 RFC 5987 Content-Disposition header 值
/// 使用 percent-encode 避免 filename injection 攻擊
///
/// 產生格式：`attachment; filename*=UTF-8''{percent_encoded_filename}`
pub fn content_disposition_header(filename: &str) -> String {
    let encoded = urlencoding::encode(filename);
    format!("attachment; filename*=UTF-8''{encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_filename() {
        let result = content_disposition_header("report.pdf");
        assert_eq!(result, "attachment; filename*=UTF-8''report.pdf");
    }

    #[test]
    fn test_unicode_filename() {
        let result = content_disposition_header("報表_2024.pdf");
        assert!(result.starts_with("attachment; filename*=UTF-8''"));
        assert!(result.contains("%"));
    }

    #[test]
    fn test_filename_with_special_chars() {
        let result = content_disposition_header("file\"name.pdf");
        assert!(!result.contains('"'));
    }
}
