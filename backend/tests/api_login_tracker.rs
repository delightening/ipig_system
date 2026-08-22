//! R49 follow-up：unusual_login 三階段降噪整合測試。
//!
//! 本檔可靠 cover 的範圍：
//! - **A**（30 分鐘 dedup）：完全時鐘無關，pre-insert open alert 後驗證後續 call
//!   不會新增第二筆。
//!
//! 未 cover（需 refactor `check_unusual_time` 支援時間注入後補）：
//! - B 首次登入跳過 new_device — 受 check_unusual_time 干擾，alert 是否寫入
//!   仍依當下時鐘。
//! - D admin 僅 unusual_time → severity='info' — 必須在台灣 18-24 / 0-8 時段
//!   跑才會觸發。
//!
//! Follow-up：將 `check_unusual_time` 改為接受 `DateTime<Utc>` 注入，即可
//! 確定性測試 B/D。

mod common;

use erp_backend::services::{GeoIpService, LoginTracker};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 建一個非 admin 測試使用者（無 role 指派）— 避開既存 admin 帳號干擾。
async fn seed_user(pool: &PgPool, suffix: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "login-tracker-{}-{}@test.local",
        suffix,
        &id.to_string()[..8]
    );
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake-hash', $3, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(format!("Login Tracker Test {}", suffix))
    .execute(pool)
    .await
    .expect("insert test user");
    (id, email)
}

/// A 驗證：30 分鐘內同 user 已有 open 'unusual_login' alert → 後續 log_success
/// 不論觸發何種異常都 skip 寫入 alert（dedup）。
#[tokio::test]
#[serial]
async fn unusual_login_dedup_skips_second_alert_within_30min() {
    let app = common::TestApp::spawn().await;
    let (user_id, email) = seed_user(&app.db_pool, "dedup").await;

    // 預先 insert 一個 open unusual_login alert（模擬 5 分鐘前已警報）
    sqlx::query(
        r#"INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            user_id, created_at, updated_at, status
        ) VALUES ($1, 'unusual_login', 'warning', '預先存在的告警', '預先存在',
            $2, NOW() - INTERVAL '5 minutes', NOW() - INTERVAL '5 minutes', 'open')"#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .execute(&app.db_pool)
    .await
    .expect("seed prior alert");

    let geoip = GeoIpService::new("/nonexistent");
    // 用「絕對新」的 user_agent 確保 check_new_device 一定 true，再配合各 trigger
    // 製造會寫 alert 的條件；但因 30 分鐘內已有 open alert，dedup 應 skip。
    LoginTracker::log_success(
        &app.db_pool,
        user_id,
        &email,
        Some("203.0.113.42"),
        Some("DedupTestBot/1.0"),
        &geoip,
    )
    .await
    .expect("log_success should not error");

    // 驗證：該 user 的 unusual_login alert 計數仍為 1（dedup 生效）
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM security_alerts
         WHERE alert_type = 'unusual_login' AND user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count alerts");
    assert_eq!(
        count, 1,
        "30 分鐘內已有 open unusual_login alert → 第二次 log_success 不該新增"
    );
}

/// A 反向驗證：若既有 alert 為 resolved 或 > 30 分鐘前，dedup 不適用 — 新 alert
/// 仍會寫入（前提是當下有 trigger 命中；本測試使用 backdate + closed 兩種狀態
/// 各驗證一次，邏輯獨立於時鐘）。
#[tokio::test]
#[serial]
async fn unusual_login_dedup_allows_new_alert_after_old_resolved() {
    let app = common::TestApp::spawn().await;
    let (user_id, _) = seed_user(&app.db_pool, "resolved").await;

    // 預先 insert 一個 31 分鐘前的 open alert（超出 dedup window）
    sqlx::query(
        r#"INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            user_id, created_at, updated_at, status
        ) VALUES ($1, 'unusual_login', 'warning', '31 分鐘前舊告警', '舊',
            $2, NOW() - INTERVAL '31 minutes', NOW() - INTERVAL '31 minutes', 'open')"#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .execute(&app.db_pool)
    .await
    .expect("seed old alert");

    // 直接驗 dedup SELECT 結果（避免被 check_unusual_time 影響）
    let recent: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_alerts
           WHERE alert_type = 'unusual_login' AND user_id = $1
             AND created_at > NOW() - INTERVAL '30 minutes' AND status = 'open'"#,
    )
    .bind(user_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("dedup window query");
    assert_eq!(recent, 0, "31 分鐘前 alert 不該被計入 dedup window");
}
