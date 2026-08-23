//! R31-16（2026-08-24）整合測試：CSP report endpoint 的取證欄位與 24h 聚合。
//!
//! 背景：prod 自 2026-07-31 起每天約 3 筆 `script-src` / `blocked_uri=eval` 的
//! enforce 違規，但 handler 只記 3 個欄位，查不出呼叫端是誰；且每一筆都新開一列
//! `security_alerts`，admin 佇列被同一個成因灌爆（2026-08-23 實查 83 筆）。
//!
//! 本檔鎖住兩件事：
//! 1. `source-file` / `line-number` / `script-sample` / `user-agent` 必須落進
//!    `context_data`——這是定位呼叫端的唯一證據，不可再被丟掉。
//! 2. 同指紋 24h 內聚合成一列並累加 `occurrence_count`；**指紋不同就必須開新列**
//!    （聚合不等於靜音，新的成因一定要浮出來）。

mod common;

use common::TestApp;
use serial_test::serial;

const CT_LEGACY: &str = "application/csp-report";
const CT_REPORTS_JSON: &str = "application/reports+json";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/151.0.0.0 ipig-test";

/// 每個測試用獨立的 source_file 當隔離鍵——共用測試庫，別的測試也可能寫 alert。
fn unique_source(tag: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time error")
        .as_nanos();
    format!("chrome-extension://{tag}{nanos}/inject.js")
}

async fn post_csp(app: &TestApp, content_type: &str, body: String) -> reqwest::StatusCode {
    app.client
        .post(app.url("/api/v1/csp-report"))
        .header("content-type", content_type)
        .header("user-agent", UA)
        .body(body)
        .send()
        .await
        .expect("HTTP request failed")
        .status()
}

/// 取回該 source_file 對應的 (列數, occurrence_count 總和, 任一列的 context_data)
async fn alerts_for_source(
    app: &TestApp,
    source_file: &str,
) -> (i64, i64, Option<serde_json::Value>) {
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as(
        "SELECT context_data FROM security_alerts \
         WHERE alert_type LIKE 'CSP%' AND context_data->>'source_file' = $1 \
         ORDER BY created_at",
    )
    .bind(source_file)
    .fetch_all(&app.db_pool)
    .await
    .expect("query security_alerts failed");

    let count = rows.len() as i64;
    let occ_sum: i64 = rows
        .iter()
        .map(|(c,)| c["occurrence_count"].as_i64().unwrap_or(0))
        .sum();
    (count, occ_sum, rows.first().map(|(c,)| c.clone()))
}

#[tokio::test]
#[serial]
async fn legacy_report_records_forensic_fields() {
    let app = TestApp::spawn().await;
    let source = unique_source("forensic");
    let body = format!(
        r#"{{"csp-report":{{"document-uri":"https://ipigsystem.asia/login",
            "violated-directive":"script-src","blocked-uri":"eval",
            "source-file":"{source}","line-number":12,"column-number":7,
            "script-sample":"(function(){{return 1}})"}}}}"#
    );

    assert_eq!(post_csp(&app, CT_LEGACY, body).await, 204);

    let (count, occ, ctx) = alerts_for_source(&app, &source).await;
    assert_eq!(count, 1, "第一筆違規要開一列 alert");
    assert_eq!(occ, 1);
    let ctx = ctx.expect("context_data present");
    assert_eq!(ctx["violated_directive"], "script-src");
    assert_eq!(ctx["blocked_uri"], "eval");
    assert_eq!(ctx["line_number"], 12);
    assert_eq!(ctx["column_number"], 7);
    assert_eq!(ctx["script_sample"], "(function(){return 1})");
    assert_eq!(ctx["user_agent"], UA);
    assert_eq!(ctx["report_only"], false);
}

#[tokio::test]
#[serial]
async fn same_fingerprint_aggregates_instead_of_new_row() {
    let app = TestApp::spawn().await;
    let source = unique_source("aggregate");
    let body = format!(
        r#"{{"csp-report":{{"document-uri":"https://ipigsystem.asia/",
            "violated-directive":"script-src","blocked-uri":"eval","source-file":"{source}"}}}}"#
    );

    for _ in 0..3 {
        assert_eq!(post_csp(&app, CT_LEGACY, body.clone()).await, 204);
    }

    let (count, occ, _) = alerts_for_source(&app, &source).await;
    assert_eq!(count, 1, "同指紋 24h 內只留一列");
    assert_eq!(occ, 3, "但三次都必須被算進 occurrence_count，不可靜音");
}

#[tokio::test]
#[serial]
async fn concurrent_identical_reports_produce_one_alert() {
    // PR #14 CodeRabbit review：check-then-insert 不是原子操作——兩筆同指紋的
    // 「第一次」若同時進來，會各自 UPDATE 0 列然後各插一列。service 端已用
    // transaction-scoped advisory lock 序列化，本測試釘住該行為。
    let app = TestApp::spawn().await;
    let source = unique_source("concurrent");
    let body = format!(
        r#"{{"csp-report":{{"document-uri":"https://ipigsystem.asia/",
            "violated-directive":"script-src","blocked-uri":"eval","source-file":"{source}"}}}}"#
    );

    let (first, second) = tokio::join!(
        post_csp(&app, CT_LEGACY, body.clone()),
        post_csp(&app, CT_LEGACY, body.clone())
    );
    assert_eq!(first, 204);
    assert_eq!(second, 204);

    let (count, occ, _) = alerts_for_source(&app, &source).await;
    assert_eq!(count, 1, "並行的同指紋第一筆也只能開一列");
    assert_eq!(occ, 2, "兩次都要被算進 occurrence_count");
}

#[tokio::test]
#[serial]
async fn different_blocked_uri_opens_new_alert() {
    let app = TestApp::spawn().await;
    let source = unique_source("distinct");
    let eval_body = format!(
        r#"{{"csp-report":{{"violated-directive":"script-src","blocked-uri":"eval","source-file":"{source}"}}}}"#
    );
    let attack_body = format!(
        r#"{{"csp-report":{{"violated-directive":"script-src",
            "blocked-uri":"https://attacker.example/xss.js","source-file":"{source}"}}}}"#
    );

    assert_eq!(post_csp(&app, CT_LEGACY, eval_body).await, 204);
    assert_eq!(post_csp(&app, CT_LEGACY, attack_body).await, 204);

    let (count, _, _) = alerts_for_source(&app, &source).await;
    assert_eq!(count, 2, "blocked_uri 不同＝不同成因，必須各自開列");
}

#[tokio::test]
#[serial]
async fn reports_json_records_sample_and_source() {
    let app = TestApp::spawn().await;
    let source = unique_source("reportsjson");
    let body = format!(
        r#"[{{"type":"csp-violation","age":0,"url":"https://ipigsystem.asia/","user_agent":"UA",
            "body":{{"documentURL":"https://ipigsystem.asia/","blockedURL":"eval",
                    "effectiveDirective":"script-src","disposition":"enforce",
                    "sourceFile":"{source}","lineNumber":1,"columnNumber":2048,
                    "sample":"var _0x=function(){{"}}}}]"#
    );

    assert_eq!(post_csp(&app, CT_REPORTS_JSON, body).await, 204);

    let (count, _, ctx) = alerts_for_source(&app, &source).await;
    assert_eq!(count, 1);
    let ctx = ctx.expect("context_data present");
    assert_eq!(ctx["script_sample"], "var _0x=function(){");
    assert_eq!(ctx["line_number"], 1);
    assert_eq!(ctx["column_number"], 2048);
    assert_eq!(ctx["report_only"], false);
}
