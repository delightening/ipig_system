//! R31-11: CSP violation 寫入 service。
//!
//! 把 `handlers/csp_report.rs` 的 SQL 集中到 service 層（對齊 CLAUDE.md
//! §「Backend 模組職責」— handler 禁止直接寫 SQL）。
//!
//! 同時提供統一的 [`CspViolation`] 結構，讓 handler 在解析新（Reporting API
//! `application/reports+json`）/ 舊（`application/csp-report`）兩種 payload
//! 後都正規化為這個型別，再交由本 service 寫入 `security_alerts`。

use serde_json::json;
use sqlx::PgPool;

use crate::Result;

pub const ALERT_TYPE_CSP_VIOLATION: &str = "CSP_VIOLATION";
pub const ALERT_TYPE_CSP_VIOLATION_REPORT_ONLY: &str = "CSP_VIOLATION_REPORT_ONLY";

/// R31-13b accepted-risk blocked_uri 字面值（CLAUDE.md「魔術字串必須定義為 const」）。
/// 集中於此 service 給 caller 比對與測試共用。
pub const BLOCKED_URI_EVAL: &str = "eval";
pub const BLOCKED_URI_WASM_EVAL: &str = "wasm-eval";

/// 兩種 payload 解析後的正規化 violation。
///
/// 三個欄位都 optional 是刻意的：legacy `application/csp-report` 部分欄位可能
/// 被瀏覽器省略（Firefox / Safari 對某些 directive 的差異），新版 Reporting
/// API 也允許 `body.blockedURL` 為 null（例如 trusted-types 違規）。
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub document_uri: Option<String>,
    pub violated_directive: Option<String>,
    pub blocked_uri: Option<String>,
    /// `true` = Report-Only header 觸發；`false` = enforce header 觸發。
    /// 來源：legacy 走 `?mode=ro` query；新版走 body.disposition == "report"。
    pub report_only: bool,
}

impl CspViolation {
    /// R31-13b / R31-15: 已記錄為 accepted-risk 的 violation pattern。
    /// **接受清單**（每加一條都需 `csp-baseline-2026-04.md` 文件對應 + R31 follow-up 評估）：
    /// - `eval` / `wasm-eval`：Cloudflare Insights beacon + transitive deps，
    ///   frontend src grep 0 處（R31-13b 永久接受）。
    pub fn is_accepted_noise(&self) -> bool {
        matches!(
            self.blocked_uri.as_deref(),
            Some(BLOCKED_URI_EVAL) | Some(BLOCKED_URI_WASM_EVAL)
        )
    }
}

/// 把 violation 寫進 `security_alerts` 表。失敗回 `Err`（caller 決定如何處理 —
/// CSP report endpoint 要求一律回 204 給瀏覽器，但 caller 應 `tracing::error!`
/// loud log，不可靜默吞）。
pub async fn insert_csp_violation(pool: &PgPool, v: &CspViolation) -> Result<()> {
    let context = json!({
        "document_uri": v.document_uri,
        "violated_directive": v.violated_directive,
        "blocked_uri": v.blocked_uri,
        "report_only": v.report_only,
    });
    let alert_type = if v.report_only {
        ALERT_TYPE_CSP_VIOLATION_REPORT_ONLY
    } else {
        ALERT_TYPE_CSP_VIOLATION
    };
    sqlx::query(
        "INSERT INTO security_alerts (alert_type, severity, title, context_data) \
         VALUES ($1, 'info', 'CSP violation reported', $2)",
    )
    .bind(alert_type)
    .bind(context)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::CspViolation;

    fn v(blocked_uri: Option<&str>) -> CspViolation {
        CspViolation {
            document_uri: None,
            violated_directive: None,
            blocked_uri: blocked_uri.map(String::from),
            report_only: true,
        }
    }

    #[test]
    fn eval_and_wasm_eval_are_accepted_noise() {
        assert!(v(Some("eval")).is_accepted_noise());
        assert!(v(Some("wasm-eval")).is_accepted_noise());
    }

    #[test]
    fn other_blocked_uris_are_not_noise() {
        assert!(!v(Some("inline")).is_accepted_noise());
        assert!(!v(Some("https://attacker.example/xss.js")).is_accepted_noise());
        assert!(!v(Some("https://google-analytics.com/g/collect")).is_accepted_noise());
        assert!(!v(None).is_accepted_noise());
        // case-sensitive — 變體不算 noise
        assert!(!v(Some("EVAL")).is_accepted_noise());
    }
}
