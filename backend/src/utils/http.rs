// HTTP 相關純函式工具

use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
};

/// 建構 RFC 5987 Content-Disposition header 值
/// 使用 percent-encode 避免 filename injection 攻擊
///
/// 產生格式：`attachment; filename*=UTF-8''{percent_encoded_filename}`
pub fn content_disposition_header(filename: &str) -> String {
    let encoded = urlencoding::encode(filename);
    format!("attachment; filename*=UTF-8''{encoded}")
}

/// L2 (2026-05-12)：建構檔案下載 response，附帶可選的 `X-PDF-Renderer` header。
///
/// 用途：pdf-service 回傳 PDF + renderer 標籤（`excel_daemon` / `word_daemon` /
/// `gotenberg_fallback` / `gotenberg_only`）時，handler 統一用此 helper 組裝
/// response，讓前端可依 header 在降級時 toast 提示使用者。
///
/// `content_type` 例：`application/pdf`、`application/vnd.openxmlformats-...`
/// `inline=true` → 瀏覽器內嵌預覽（PDF viewer 用 filename 當分頁標題）
/// `inline=false` → 觸發下載
pub fn file_response_with_renderer(
    bytes: Vec<u8>,
    content_type: &str,
    filename: &str,
    inline: bool,
    renderer: Option<String>,
) -> Result<Response<Body>, String> {
    let disposition = if inline {
        content_disposition_inline_header(filename)
    } else {
        content_disposition_header(filename)
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, disposition);
    if let Some(r) = renderer {
        if let Ok(v) = HeaderValue::from_str(&r) {
            builder = builder.header("X-PDF-Renderer", v);
        }
    }
    builder
        .body(Body::from(bytes))
        .map_err(|e| format!("Failed to build response: {}", e))
}

/// R35-4: 內嵌（inline）版 Content-Disposition — 用於瀏覽器內預覽（PDF viewer
/// 會以 filename 當分頁標題，取代 `blob:` URL）。同樣套 RFC 5987 percent-encode。
pub fn content_disposition_inline_header(filename: &str) -> String {
    let encoded = urlencoding::encode(filename);
    format!("inline; filename*=UTF-8''{encoded}")
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

    #[test]
    fn test_inline_header_ascii() {
        let result = content_disposition_inline_header("report.pdf");
        assert_eq!(result, "inline; filename*=UTF-8''report.pdf");
    }

    #[test]
    fn test_inline_header_unicode() {
        let result = content_disposition_inline_header("倉庫_現況.pdf");
        assert!(result.starts_with("inline; filename*=UTF-8''"));
        assert!(result.contains("%"));
    }
}
