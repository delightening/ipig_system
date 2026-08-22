//! 通用 HTML escape 工具 — outbox email channel / 任何把使用者輸入插入 HTML 模板的 caller 都應呼叫。
//!
//! 提供最小實作（5 個結構字元: `& < > " '`），不引入新 crate。如需更完整的
//! HTML 處理（屬性、URL、事件 handler 等），改評估 `html-escape` crate。
//!
//! Design ref: `docs/dev/notification-and-outbox.md` §「常見坑」。

/// 把字串中的 5 個 HTML 結構字元轉成 entity，防 XSS。
///
/// 適用於把純文字插入 HTML **文字節點**（如 `<p>{}</p>`）。
/// **不適用**於插入屬性值（需額外處理 quote）、URL、JavaScript 字串。
pub fn html_escape_minimal(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::html_escape_minimal;

    #[test]
    fn escapes_all_five_html_structural_chars() {
        let input = r#"<script>alert("xss" & 'attack')</script>"#;
        let out = html_escape_minimal(input);
        assert!(!out.contains('<'), "< not escaped: {out}");
        assert!(!out.contains('>'), "> not escaped: {out}");
        assert!(!out.contains('"'), "\" not escaped: {out}");
        assert!(!out.contains('\''), "' not escaped: {out}");
        assert_eq!(
            out,
            "&lt;script&gt;alert(&quot;xss&quot; &amp; &#x27;attack&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn passes_through_safe_text() {
        assert_eq!(html_escape_minimal("hello world 123"), "hello world 123");
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(html_escape_minimal(""), "");
    }

    #[test]
    fn handles_unicode() {
        assert_eq!(html_escape_minimal("動物 < 通知"), "動物 &lt; 通知");
    }
}
