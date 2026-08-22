//! R31-1 / R31-11: CSP violation report endpoint。
//!
//! 接收瀏覽器送來的 CSP 違規報告，正規化後交由 `services/csp_report.rs` 寫入
//! `security_alerts`。**不需認證、不需 CSRF** — 瀏覽器以匿名方式 POST。
//!
//! ## 兩種 payload format（dual support）
//!
//! 1. **Legacy `application/csp-report`**（CSP1/2，所有現有瀏覽器仍會送）：
//!    `{ "csp-report": { "blocked-uri": "...", "violated-directive": "...", ... } }`
//!    Report-Only header 觸發者由 nginx 加 query `?mode=ro` 標記。
//!
//! 2. **新版 `application/reports+json`**（CSP3 + Reporting API，Chrome / Edge
//!    96+ 已切換）：array of reports，每個包含 `body.disposition: "report" | "enforce"`
//!    可直接判斷 Report-Only，無需 query hack。
//!
//! Handler 依 `Content-Type` 分流到對應解析路徑，正規化為
//! [`services::csp_report::CspViolation`] 後共用寫入流程。
//!
//! ## 為何 handler 簽名仍是 `StatusCode`（非 `Result<impl IntoResponse, AppError>`）
//!
//! CSP 規範要求**一律回 204 No Content** — 即使 payload 解析失敗、DB INSERT 失敗，
//! 都不能讓瀏覽器以為要重試或當作錯誤。失敗時走 `tracing::error!` loud log，
//! 但 status 不變。這是合理的 endpoint pattern 例外。

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use crate::services::csp_report::{insert_csp_violation, CspViolation};
use crate::AppState;

#[derive(Deserialize)]
struct LegacyCspReportWrapper {
    #[serde(rename = "csp-report")]
    csp_report: LegacyCspBody,
}

#[derive(Deserialize)]
struct LegacyCspBody {
    #[serde(rename = "document-uri")]
    document_uri: Option<String>,
    #[serde(rename = "violated-directive")]
    violated_directive: Option<String>,
    #[serde(rename = "blocked-uri")]
    blocked_uri: Option<String>,
}

/// CSP3 Reporting API：每個 report 是一個 envelope。
#[derive(Deserialize)]
struct ReportingApiEnvelope {
    #[serde(rename = "type")]
    report_type: String,
    body: ReportingApiBody,
}

#[derive(Deserialize)]
struct ReportingApiBody {
    #[serde(rename = "documentURL")]
    document_url: Option<String>,
    /// CSP3 spec 用 effectiveDirective，但部分瀏覽器仍同時送 violatedDirective
    #[serde(rename = "effectiveDirective", alias = "violatedDirective")]
    effective_directive: Option<String>,
    #[serde(rename = "blockedURL")]
    blocked_url: Option<String>,
    /// "report" = Report-Only header；"enforce" = 強制執行
    #[serde(default)]
    disposition: Option<String>,
}

/// nginx 以 `?mode=ro` 標記 Report-Only header 的 report-uri（legacy payload 用）。
/// 新版 Reporting API 改看 body.disposition。
#[derive(Deserialize, Default)]
pub struct CspReportQuery {
    #[serde(default)]
    mode: Option<String>,
}

const CT_LEGACY: &str = "application/csp-report";
const CT_REPORTS_JSON: &str = "application/reports+json";

/// R33-3 pentest follow-up：CSP report payload 上限。
///
/// 此 endpoint 無認證、無 CSRF（瀏覽器規範要求匿名 POST），只受 api rate limiter 保護。
/// 沒有大小上限時，攻擊者可送 MB 級垃圾 payload 灌爆 audit log + DB row 體積。
/// 真實 CSP report 即使含長 URL + stack trace 也罕見 > 4KB；給 16KB 已寬鬆。
const CSP_REPORT_MAX_BYTES: usize = 16 * 1024;

/// 接收 CSP violation report。**一律回 204** — 解析失敗 / DB 失敗都走 tracing log，
/// 不影響 response status（CSP 規範 + 與既有 nginx 設定相容）。
pub async fn csp_report_handler(
    State(state): State<AppState>,
    Query(q): Query<CspReportQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // R33-3: payload 過大直接 drop（仍回 204 — CSP 規範不允許 4xx）
    if body.len() > CSP_REPORT_MAX_BYTES {
        tracing::warn!(
            len = body.len(),
            cap = CSP_REPORT_MAX_BYTES,
            "csp_report payload exceeds cap, dropping"
        );
        return StatusCode::NO_CONTENT;
    }

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Content-Type 可能含 charset / boundary，只比 prefix
    let violations = if content_type.starts_with(CT_REPORTS_JSON) {
        parse_reports_json(&body)
    } else if content_type.starts_with(CT_LEGACY) || content_type.is_empty() {
        // 部分老 browser 不送 Content-Type，預設走 legacy 解析
        parse_legacy(&body, q.mode.as_deref() == Some("ro"))
    } else {
        tracing::warn!(content_type, "csp_report unknown Content-Type, dropping");
        return StatusCode::NO_CONTENT;
    };

    for v in violations {
        tracing::warn!(
            event = "csp_violation",
            report_only = v.report_only,
            document_uri = ?v.document_uri,
            violated_directive = ?v.violated_directive,
            blocked_uri = ?v.blocked_uri,
        );
        // R31-13b: accepted-risk noise 僅在 Report-Only 路徑過濾。
        // enforce 出現 eval / wasm-eval = 真攻擊信號（瀏覽器實際擋下了 script），必升 alert。
        if v.report_only && v.is_accepted_noise() {
            continue;
        }
        if let Err(e) = insert_csp_violation(&state.db, &v).await {
            // 寫入失敗 loud log（R31-11 todo (c)；不能改 status — 瀏覽器期望 204）
            tracing::error!(error = %e, "csp_report insert into security_alerts failed");
        }
    }
    StatusCode::NO_CONTENT
}

fn parse_legacy(body: &[u8], report_only: bool) -> Vec<CspViolation> {
    match serde_json::from_slice::<LegacyCspReportWrapper>(body) {
        Ok(wrapper) => {
            let r = wrapper.csp_report;
            vec![CspViolation {
                document_uri: r.document_uri,
                violated_directive: r.violated_directive,
                blocked_uri: r.blocked_uri,
                report_only,
            }]
        }
        Err(e) => {
            tracing::warn!(error = %e, "csp_report legacy payload parse failed");
            Vec::new()
        }
    }
}

fn parse_reports_json(body: &[u8]) -> Vec<CspViolation> {
    let envelopes: Vec<ReportingApiEnvelope> = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "csp_report reports+json payload parse failed");
            return Vec::new();
        }
    };
    envelopes
        .into_iter()
        .filter(|env| env.report_type == "csp-violation")
        .map(|env| CspViolation {
            document_uri: env.body.document_url,
            violated_directive: env.body.effective_directive,
            blocked_uri: env.body.blocked_url,
            // disposition 缺省（部分瀏覽器舊版實作）保守視為 **enforce**（report_only=false）：
            // 若預設 true 而瀏覽器其實是 enforce + blocked_uri=eval/wasm-eval → 會被
            // noise filter 吞掉，漏掉真實攻擊信號（CodeRabbit + Gemini PR #312 review
            // 同時抓到原邏輯反了）。寧可多寫幾筆 enforce alert（admin 可重新分類），
            // 也不漏真攻擊。實際情況：Chrome 96+ 送 reports+json 時必帶 disposition。
            report_only: env
                .body
                .disposition
                .as_deref()
                .map(|d| d == "report")
                .unwrap_or(false),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_payload_extracts_fields() {
        let body = br#"{"csp-report":{"document-uri":"https://x/","violated-directive":"script-src","blocked-uri":"eval"}}"#;
        let vs = parse_legacy(body, true);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].blocked_uri.as_deref(), Some("eval"));
        assert_eq!(vs[0].violated_directive.as_deref(), Some("script-src"));
        assert!(vs[0].report_only);
    }

    #[test]
    fn parse_legacy_payload_invalid_returns_empty() {
        let vs = parse_legacy(b"not json", false);
        assert!(vs.is_empty());
    }

    #[test]
    fn parse_reports_json_extracts_csp_violation_only() {
        let body = br#"[
            {"type":"csp-violation","age":0,"url":"https://x/","user_agent":"UA",
             "body":{"documentURL":"https://x/","blockedURL":"eval",
                     "effectiveDirective":"script-src","disposition":"report"}},
            {"type":"deprecation","age":0,"url":"https://x/","user_agent":"UA",
             "body":{"id":"foo","message":"bar"}}
        ]"#;
        let vs = parse_reports_json(body);
        assert_eq!(vs.len(), 1, "deprecation report should be filtered out");
        assert_eq!(vs[0].blocked_uri.as_deref(), Some("eval"));
        assert!(vs[0].report_only);
    }

    #[test]
    fn parse_reports_json_disposition_enforce_means_not_report_only() {
        let body = br#"[
            {"type":"csp-violation","age":0,"url":"https://x/","user_agent":"UA",
             "body":{"documentURL":"https://x/","blockedURL":"https://attacker/x.js",
                     "effectiveDirective":"script-src","disposition":"enforce"}}
        ]"#;
        let vs = parse_reports_json(body);
        assert_eq!(vs.len(), 1);
        assert!(
            !vs[0].report_only,
            "disposition=enforce → report_only=false"
        );
    }

    #[test]
    fn reports_json_missing_disposition_defaults_to_enforce() {
        // 防回歸（PR #312 review）：disposition 缺省必須視為 enforce
        // （report_only=false），否則 enforce + eval 違規會被 noise filter 吞。
        let body = br#"[
            {"type":"csp-violation","age":0,"url":"https://x/","user_agent":"UA",
             "body":{"documentURL":"https://x/","blockedURL":"eval",
                     "effectiveDirective":"script-src"}}
        ]"#;
        let vs = parse_reports_json(body);
        assert_eq!(vs.len(), 1);
        assert!(
            !vs[0].report_only,
            "缺 disposition 必須保守視為 enforce，避免漏掉真實攻擊"
        );
    }

    #[test]
    fn enforce_eval_violation_is_not_filtered_as_noise() {
        // 防回歸：enforce 路徑出現 eval 必須視為真攻擊，不可被 noise filter 吞。
        let v = CspViolation {
            document_uri: Some("https://x/".into()),
            violated_directive: Some("script-src".into()),
            blocked_uri: Some("eval".into()),
            report_only: false,
        };
        assert!(v.is_accepted_noise(), "blocked_uri='eval' 屬 noise 清單");
        // handler 邏輯：noise 過濾須加 `report_only && is_accepted_noise()` 守門
        assert!(
            !(v.report_only && v.is_accepted_noise()),
            "enforce 路徑必須 alert"
        );
    }

    #[test]
    fn parse_reports_json_violated_directive_alias_works() {
        // 部分瀏覽器仍送舊欄名 violatedDirective
        let body = br#"[
            {"type":"csp-violation","age":0,"url":"https://x/","user_agent":"UA",
             "body":{"documentURL":"https://x/","blockedURL":"inline",
                     "violatedDirective":"style-src","disposition":"enforce"}}
        ]"#;
        let vs = parse_reports_json(body);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].violated_directive.as_deref(), Some("style-src"));
    }
}
