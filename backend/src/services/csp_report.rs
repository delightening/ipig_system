//! R31-11: CSP violation 寫入 service。
//!
//! 把 `handlers/csp_report.rs` 的 SQL 集中到 service 層（對齊 CLAUDE.md
//! §「Backend 模組職責」— handler 禁止直接寫 SQL）。
//!
//! 同時提供統一的 [`CspViolation`] 結構，讓 handler 在解析新（Reporting API
//! `application/reports+json`）/ 舊（`application/csp-report`）兩種 payload
//! 後都正規化為這個型別，再交由本 service 寫入 `security_alerts`。
//!
//! ## R31-16（2026-08-24）：取證欄位 + 聚合
//!
//! 起因：prod 自 2026-07-31 起穩定產生 `script-src` / `blocked_uri=eval` 的
//! enforce 違規（約 3 筆/天，至 2026-08-23 共 83 筆），但當時只記 3 個欄位，
//! **無從判斷是誰呼叫 eval**——deployed bundle 內 `eval(` / `new Function(`
//! 皆 0 命中、乾淨 Chromium 打 prod `/login` 也 0 violation，指向瀏覽器端注入，
//! 卻沒有任何證據能定案。
//!
//! 兩項改動：
//! 1. **補記 `source_file` / `line_number` / `column_number` / `script_sample`
//!    / `user_agent`**——瀏覽器本來就送了這些欄位（eval 違規的 `script-sample`
//!    是被求值字串的前 40 字、`source-file` 若是擴充套件會是 `chrome-extension://…`），
//!    是我們自己丟掉的。
//! 2. **同指紋 24 小時聚合**（[`AGGREGATE_WINDOW`]）：重複發生累加
//!    `occurrence_count` 而**不是**新增 alert 列。這是聚合不是靜音——
//!    每一筆違規仍然 (a) 進 `tracing::warn!`（Loki 全留）(b) 累加 Prometheus
//!    counter `ipig_csp_violations_total` (c) 讓既有 alert 的 `updated_at` 前進。
//!    指紋不同（新的 directive / blocked_uri / source_file）一定開新 alert。

use serde_json::json;
use sqlx::PgPool;

use crate::Result;

pub const ALERT_TYPE_CSP_VIOLATION: &str = "CSP_VIOLATION";
pub const ALERT_TYPE_CSP_VIOLATION_REPORT_ONLY: &str = "CSP_VIOLATION_REPORT_ONLY";

/// R31-13b accepted-risk blocked_uri 字面值（CLAUDE.md「魔術字串必須定義為 const」）。
/// 集中於此 service 給 caller 比對與測試共用。
pub const BLOCKED_URI_EVAL: &str = "eval";
pub const BLOCKED_URI_WASM_EVAL: &str = "wasm-eval";

/// R31-16 取證欄位長度上限。此 endpoint 匿名、payload 上限 16KB（handler 端），
/// 但單一欄位仍可能是超長 data: URL 或整段被求值的腳本 —— 寫進 alert 與 log 前先夾。
const MAX_URI_LEN: usize = 512;
const MAX_SCRIPT_SAMPLE_LEN: usize = 200;
const MAX_USER_AGENT_LEN: usize = 256;

/// R31-16 聚合視窗（PostgreSQL interval 字面值，以 bind 參數傳入，不拼字串）。
const AGGREGATE_WINDOW: &str = "24 hours";

/// 尚未結案的 alert 狀態（聚合只打這三種；已 resolved 的不復活，開新列）。
const OPEN_STATUSES: [&str; 3] = ["open", "acknowledged", "investigating"];

/// Prometheus counter 名（每一筆違規都加，含被 noise filter 略過與被聚合的）。
pub const METRIC_CSP_VIOLATIONS: &str = "ipig_csp_violations_total";

/// directive label 允許清單 —— **這個 endpoint 無認證**，`violated_directive`
/// 是攻擊者可控字串，直接當 metric label 會炸 Prometheus 的 cardinality。
/// 只放行本站 CSP 實際會出現的 directive，其餘一律歸 [`DIRECTIVE_OTHER`]。
const KNOWN_DIRECTIVES: [&str; 19] = [
    "default-src",
    "script-src",
    "script-src-elem",
    "script-src-attr",
    "style-src",
    "style-src-elem",
    "style-src-attr",
    "img-src",
    "font-src",
    "connect-src",
    "frame-src",
    "child-src",
    "worker-src",
    "media-src",
    "object-src",
    "manifest-src",
    "frame-ancestors",
    "base-uri",
    "form-action",
];
const DIRECTIVE_OTHER: &str = "other";
const DIRECTIVE_NONE: &str = "none";

/// 兩種 payload 解析後的正規化 violation。
///
/// optional 欄位是刻意的：legacy `application/csp-report` 部分欄位可能
/// 被瀏覽器省略（Firefox / Safari 對某些 directive 的差異），新版 Reporting
/// API 也允許 `body.blockedURL` 為 null（例如 trusted-types 違規）。
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub document_uri: Option<String>,
    pub violated_directive: Option<String>,
    pub blocked_uri: Option<String>,
    /// R31-16 取證欄位：呼叫端的腳本 URL。擴充套件注入時為 `chrome-extension://<id>/…`。
    pub source_file: Option<String>,
    pub line_number: Option<i64>,
    pub column_number: Option<i64>,
    /// R31-16 取證欄位：被擋下的內容樣本（eval 違規＝被求值字串前 40 字）。
    pub script_sample: Option<String>,
    /// R31-16：送出 report 的 UA（nginx access log 有，但與 alert 對不起來）。
    pub user_agent: Option<String>,
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

    /// R31-16：所有字串欄位夾長度後回傳。handler 解析完立刻呼叫，
    /// 讓 log 與 DB 看到的是同一份已夾好的值。
    pub fn normalized(mut self) -> Self {
        self.document_uri = clamp(self.document_uri, MAX_URI_LEN);
        self.violated_directive = clamp(self.violated_directive, MAX_URI_LEN);
        self.blocked_uri = clamp(self.blocked_uri, MAX_URI_LEN);
        self.source_file = clamp(self.source_file, MAX_URI_LEN);
        self.script_sample = clamp(self.script_sample, MAX_SCRIPT_SAMPLE_LEN);
        self.user_agent = clamp(self.user_agent, MAX_USER_AGENT_LEN);
        self
    }

    /// metric label 用的 directive。取第一個 token（legacy `violated-directive`
    /// 舊瀏覽器會送整串 `script-src 'self' 'nonce-…'`），小寫後比對允許清單。
    pub fn directive_label(&self) -> &'static str {
        let Some(raw) = self.violated_directive.as_deref() else {
            return DIRECTIVE_NONE;
        };
        let head = raw
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        KNOWN_DIRECTIVES
            .iter()
            .find(|d| **d == head)
            .copied()
            .unwrap_or(DIRECTIVE_OTHER)
    }

    fn alert_type(&self) -> &'static str {
        if self.report_only {
            ALERT_TYPE_CSP_VIOLATION_REPORT_ONLY
        } else {
            ALERT_TYPE_CSP_VIOLATION
        }
    }

    /// admin 清單頁直接看得到的一行摘要（`security_alerts.description`）。
    fn description(&self) -> String {
        format!(
            "{} 違規 — directive={}, blocked={}, source={}:{}",
            if self.report_only {
                "Report-Only"
            } else {
                "Enforce"
            },
            self.violated_directive.as_deref().unwrap_or("(none)"),
            self.blocked_uri.as_deref().unwrap_or("(none)"),
            self.source_file.as_deref().unwrap_or("(none)"),
            self.line_number
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string()),
        )
    }
}

fn clamp(v: Option<String>, max: usize) -> Option<String> {
    v.map(|s| s.chars().take(max).collect())
}

/// 把 violation 寫進 `security_alerts` 表。失敗回 `Err`（caller 決定如何處理 —
/// CSP report endpoint 要求一律回 204 給瀏覽器，但 caller 應 `tracing::error!`
/// loud log，不可靜默吞）。
///
/// R31-16：同指紋（alert_type + violated_directive + blocked_uri + source_file）
/// 在 [`AGGREGATE_WINDOW`] 內已有未結案 alert → 累加 `occurrence_count` + 推進
/// `updated_at`；否則開新列。回傳 `true` = 開了新 alert，`false` = 併進既有 alert。
pub async fn insert_csp_violation(pool: &PgPool, v: &CspViolation) -> Result<bool> {
    let alert_type = v.alert_type();
    // 指紋只取「同一個成因」的三個欄位；document_uri / user_agent 不進指紋，
    // 否則同一個擴充套件在 10 個頁面就變成 10 筆 alert。
    let fingerprint = json!({
        "violated_directive": v.violated_directive,
        "blocked_uri": v.blocked_uri,
        "source_file": v.source_file,
    });

    let aggregated = sqlx::query(
        r#"
        UPDATE security_alerts
        SET context_data = jsonb_set(
                context_data,
                '{occurrence_count}',
                to_jsonb(COALESCE((context_data->>'occurrence_count')::int, 1) + 1)
            ),
            updated_at = NOW()
        WHERE id = (
            SELECT id
            FROM security_alerts
            WHERE alert_type = $1
              AND status = ANY($2)
              AND created_at > NOW() - $3::interval
              AND context_data @> $4::jsonb
            ORDER BY created_at DESC
            LIMIT 1
        )
        "#,
    )
    .bind(alert_type)
    .bind(&OPEN_STATUSES[..])
    .bind(AGGREGATE_WINDOW)
    .bind(&fingerprint)
    .execute(pool)
    .await?;

    if aggregated.rows_affected() > 0 {
        return Ok(false);
    }

    let context = json!({
        "document_uri": v.document_uri,
        "violated_directive": v.violated_directive,
        "blocked_uri": v.blocked_uri,
        "source_file": v.source_file,
        "line_number": v.line_number,
        "column_number": v.column_number,
        "script_sample": v.script_sample,
        "user_agent": v.user_agent,
        "report_only": v.report_only,
        "occurrence_count": 1,
    });
    sqlx::query(
        "INSERT INTO security_alerts (alert_type, severity, title, description, context_data) \
         VALUES ($1, 'info', 'CSP violation reported', $2, $3)",
    )
    .bind(alert_type)
    .bind(v.description())
    .bind(context)
    .execute(pool)
    .await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(blocked_uri: Option<&str>) -> CspViolation {
        CspViolation {
            document_uri: None,
            violated_directive: None,
            blocked_uri: blocked_uri.map(String::from),
            source_file: None,
            line_number: None,
            column_number: None,
            script_sample: None,
            user_agent: None,
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

    #[test]
    fn directive_label_maps_known_directives() {
        let mut x = v(None);
        x.violated_directive = Some("script-src".into());
        assert_eq!(x.directive_label(), "script-src");
        // legacy 瀏覽器送整串 directive 值 → 只取第一個 token
        x.violated_directive = Some("script-src 'self' 'nonce-abc'".into());
        assert_eq!(x.directive_label(), "script-src");
        x.violated_directive = Some("SCRIPT-SRC-ELEM".into());
        assert_eq!(x.directive_label(), "script-src-elem");
    }

    #[test]
    fn directive_label_rejects_attacker_controlled_values() {
        // 匿名 endpoint：任意字串不得變成 metric label（cardinality 攻擊）
        let mut x = v(None);
        x.violated_directive = Some("../../etc/passwd".into());
        assert_eq!(x.directive_label(), DIRECTIVE_OTHER);
        x.violated_directive = None;
        assert_eq!(x.directive_label(), DIRECTIVE_NONE);
    }

    #[test]
    fn normalized_clamps_long_fields() {
        let mut x = v(Some("eval"));
        x.source_file = Some("x".repeat(MAX_URI_LEN + 100));
        x.script_sample = Some("y".repeat(MAX_SCRIPT_SAMPLE_LEN + 100));
        x.user_agent = Some("z".repeat(MAX_USER_AGENT_LEN + 100));
        let x = x.normalized();
        assert_eq!(x.source_file.as_deref().map(str::len), Some(MAX_URI_LEN));
        assert_eq!(
            x.script_sample.as_deref().map(str::len),
            Some(MAX_SCRIPT_SAMPLE_LEN)
        );
        assert_eq!(
            x.user_agent.as_deref().map(str::len),
            Some(MAX_USER_AGENT_LEN)
        );
    }

    #[test]
    fn normalized_clamps_by_chars_not_bytes() {
        // 多位元組字元不可被切在半路（會產生無效 UTF-8 panic 或亂碼）
        let mut x = v(None);
        x.script_sample = Some("測".repeat(MAX_SCRIPT_SAMPLE_LEN + 10));
        let out = x.normalized().script_sample.expect("sample kept");
        assert_eq!(out.chars().count(), MAX_SCRIPT_SAMPLE_LEN);
    }

    #[test]
    fn description_mentions_disposition_and_source() {
        let mut x = v(Some("eval"));
        x.report_only = false;
        x.violated_directive = Some("script-src".into());
        x.source_file = Some("chrome-extension://abcd/inject.js".into());
        x.line_number = Some(42);
        let d = x.description();
        assert!(d.contains("Enforce"), "{d}");
        assert!(d.contains("script-src"), "{d}");
        assert!(d.contains("chrome-extension://abcd/inject.js:42"), "{d}");
    }
}
