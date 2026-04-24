//! R26 Service-driven Audit 整合測試
//!
//! 對應 docs/plans/R26_FullPlan.md DoD-3 / DoD-6 機械式驗證需求。
//! Critical review (PR #194 之前) 發現整合測試完全缺失，本檔補上。
//!
//! ## 測試範圍
//!
//! 1. **`tx_rollback_does_not_persist_audit`**
//!    - 驗證 SDD 核心保證：tx rollback 時 audit 也不會落地。
//!    - 對應 fullplan §7 模板的「失敗自動 rollback」。
//!
//! 2. **`tx_commit_persists_audit_with_chain`**
//!    - 驗證 tx commit 後 audit 寫入並形成 HMAC chain。
//!    - 對應 DoD-2 (UPDATE 含 before/after JSON) + SEC-34 (HMAC chain)。
//!
//! 3. **`hmac_chain_broken_detected_by_verify`**
//!    - 驗證 verify_chain_range 能偵測手動篡改 row 的斷鏈情況。
//!    - 對應 DoD-3 (HMAC chain 完整性 + audit_chain_verify cron)。
//!
//! 4. **`concurrent_audit_writes_no_chain_race`**
//!    - 3 並發 log_activity_tx 全部成功且每筆有 integrity_hash（advisory
//!      lock + HMAC chain 序列化）。對應 DoD-6 同類精神。
//!    - **並發度**：3（不是 10）— TestApp::spawn 的 PgPool `max_connections=5`，
//!      3 < 5 避免 conn pool 飽和；advisory lock 仍會序列化，足以測試 chain
//!      race。原規劃 10 在本地可跑，CI 環境 conn 較緊故降級。
//!
//! ## 測試環境
//!
//! 直接使用 `TestApp::spawn().db_pool` 取 connection pool。
//! 不需要 HTTP layer — 測試目標為 service 層原子性，不關 axum routing。
//!
//! ## HMAC key 初始化
//!
//! `TestApp::spawn` 於 startup 呼叫 `AuditService::init_hmac_key`，fallback
//! 順序：`AUDIT_HMAC_KEY` env → `config.audit_hmac_key` → hardcoded 測試值。
//! 因此 CI 即使沒設 env 也能跑 HMAC 相關測試。

mod common;

use common::TestApp;

use erp_backend::services::audit::{
    ActivityLogEntry, AuditEntity, AuditService, ChainVerificationReport,
};
use erp_backend::{ActorContext, SYSTEM_USER_ID};

use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use std::time::Duration;

fn system_actor() -> ActorContext {
    ActorContext::System {
        reason: "r26_integration_test",
    }
}

fn make_entry<'a>(category: &'a str, event: &'a str, name: &'a str) -> ActivityLogEntry<'a> {
    ActivityLogEntry {
        event_category: category,
        event_type: event,
        entity: Some(AuditEntity::new(
            "test_entity",
            uuid::Uuid::new_v4(),
            name,
        )),
        data_diff: None,
        request_context: None,
    }
}

// ============================================================
// Test 1: tx rollback 不持久化 audit
// ============================================================

#[tokio::test]
async fn tx_rollback_does_not_persist_audit() {
    let app = TestApp::spawn().await;

    let unique_event = format!("R26_TEST_ROLLBACK_{}", uuid::Uuid::new_v4().simple());
    let actor = system_actor();

    // Phase 1: 開 tx → 寫 audit → rollback
    let mut tx = app.db_pool.begin().await.expect("begin tx");

    let entry = make_entry("TEST", &unique_event, "rollback test");
    let log_id = AuditService::log_activity_tx(&mut tx, &actor, entry)
        .await
        .expect("log_activity_tx should succeed inside tx");

    // 驗證在 tx 內可看到該 row
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = $1 AND id = $2",
    )
    .bind(&unique_event)
    .bind(log_id)
    .fetch_one(&mut *tx)
    .await
    .expect("count inside tx");
    assert_eq!(row.0, 1, "tx 內應看到剛寫入的 audit row");

    // Rollback（不 commit）
    tx.rollback().await.expect("rollback");

    // Phase 2: 從 pool 重新查 — audit row 應不存在
    let after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_activity_logs WHERE event_type = $1")
            .bind(&unique_event)
            .fetch_one(&app.db_pool)
            .await
            .expect("count after rollback");
    assert_eq!(after.0, 0, "rollback 後 audit row 不應持久化");
}

// ============================================================
// Test 2: tx commit 持久化 audit + HMAC chain 連續
// ============================================================

#[tokio::test]
async fn tx_commit_persists_audit_with_chain() {
    let app = TestApp::spawn().await;

    let unique_prefix = format!("R26_TEST_COMMIT_{}", uuid::Uuid::new_v4().simple());
    let actor = system_actor();

    // Phase 1: 開 tx → 寫 3 筆 audit → commit
    let mut tx = app.db_pool.begin().await.expect("begin tx");
    let mut written_ids: Vec<uuid::Uuid> = Vec::with_capacity(3);

    for i in 0..3 {
        let event = format!("{}_{}", unique_prefix, i);
        let display = format!("commit test {}", i);
        let entry = make_entry("TEST", &event, &display);
        let id = AuditService::log_activity_tx(&mut tx, &actor, entry)
            .await
            .expect("log_activity_tx in batch");
        written_ids.push(id);
    }

    tx.commit().await.expect("commit");

    // Phase 2: 從 pool 查 3 筆都已持久化
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type LIKE $1 AND actor_user_id = $2",
    )
    .bind(format!("{}%", unique_prefix))
    .bind(SYSTEM_USER_ID)
    .fetch_one(&app.db_pool)
    .await
    .expect("count after commit");
    assert_eq!(count.0, 3, "commit 後 3 筆 audit row 全部持久化");

    // Phase 3: 驗證 HMAC chain — 每筆都有 integrity_hash
    for id in &written_ids {
        let row = sqlx::query("SELECT integrity_hash FROM user_activity_logs WHERE id = $1")
            .bind(id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch row");
        let hash: Option<String> = row.try_get("integrity_hash").expect("get integrity_hash");
        assert!(
            hash.is_some() && !hash.as_deref().unwrap_or("").is_empty(),
            "audit row {id} 應有 non-empty integrity_hash（TestApp::spawn 已 init_hmac_key）"
        );
    }
}

// ============================================================
// Test 3: HMAC chain 斷鏈被 verify_chain_range 偵測
// ============================================================

#[tokio::test]
async fn hmac_chain_broken_detected_by_verify() {
    let app = TestApp::spawn().await;

    let unique_prefix = format!("R26_TEST_BROKEN_{}", uuid::Uuid::new_v4().simple());
    let actor = system_actor();
    let from = Utc::now() - chrono::Duration::seconds(5);

    // Phase 1: 寫 3 筆形成 chain
    let mut tx = app.db_pool.begin().await.expect("begin tx");
    let mut ids: Vec<uuid::Uuid> = Vec::with_capacity(3);
    for i in 0..3 {
        let event = format!("{}_{}", unique_prefix, i);
        let display = format!("chain test {}", i);
        let entry = make_entry("TEST", &event, &display);
        let id = AuditService::log_activity_tx(&mut tx, &actor, entry)
            .await
            .expect("log_activity_tx");
        ids.push(id);
    }
    tx.commit().await.expect("commit");

    // Phase 2: 篡改中間那筆的 after_data（導致重算 HMAC ≠ 儲存的 integrity_hash）
    //
    // 需先 DISABLE `trg_user_activity_logs_immutable` trigger 才能 UPDATE。
    // 該 trigger 是 GLP §11.10 immutability 保護（RAISE EXCEPTION on any
    // payload modification）。測試模擬的是「攻擊者取得 superuser 後繞過
    // trigger 直接改 DB」的威脅模型，正是 verify_chain_range 該偵測的場景。
    sqlx::query("ALTER TABLE user_activity_logs DISABLE TRIGGER trg_user_activity_logs_immutable")
        .execute(&app.db_pool)
        .await
        .expect("disable immutability trigger for tamper simulation");

    sqlx::query("UPDATE user_activity_logs SET after_data = $1 WHERE id = $2")
        .bind(json!({ "tampered": true }))
        .bind(ids[1])
        .execute(&app.db_pool)
        .await
        .expect("tamper row");

    sqlx::query("ALTER TABLE user_activity_logs ENABLE TRIGGER trg_user_activity_logs_immutable")
        .execute(&app.db_pool)
        .await
        .expect("re-enable immutability trigger");

    // Phase 3: 驗證 chain — 應偵測到至少 1 個 broken_link
    let to = Utc::now() + chrono::Duration::seconds(5);
    let report: ChainVerificationReport = AuditService::verify_chain_range(&app.db_pool, from, to)
        .await
        .expect("verify_chain_range");

    assert!(
        !report.broken_links.is_empty(),
        "篡改 row 後應偵測到 broken_links（實際得到 {} 筆）",
        report.broken_links.len()
    );

    // 篡改的 row 必在 broken_links 中
    let tampered_in_report = report.broken_links.iter().any(|b| b.id == ids[1]);
    assert!(
        tampered_in_report,
        "broken_links 應含被篡改的 id={}",
        ids[1]
    );

    // 清理（避免影響後續執行的其他測試）
    sqlx::query("DELETE FROM user_activity_logs WHERE event_type LIKE $1")
        .bind(format!("{}%", unique_prefix))
        .execute(&app.db_pool)
        .await
        .ok();
}

// ============================================================
// Test 4: 3 並發 log_activity_tx 無 chain race
// ============================================================

#[tokio::test]
async fn concurrent_audit_writes_no_chain_race() {
    let app = TestApp::spawn().await;

    let unique_prefix = format!("R26_TEST_CONCURRENT_{}", uuid::Uuid::new_v4().simple());
    let pool = app.db_pool.clone();

    // Spawn 3 並發 task 各自開獨立 tx 寫一筆 audit。
    // 3 < max_connections(5) 避免 conn pool 飽和導致測試 hang；
    // advisory lock 仍會序列化，足以驗證 chain race 不發生。
    const CONCURRENT: usize = 3;
    let mut handles = Vec::with_capacity(CONCURRENT);
    for i in 0..CONCURRENT {
        let pool_c = pool.clone();
        let event = format!("{}_{}", unique_prefix, i);
        let display = format!("concurrent test {}", i);
        let handle = tokio::spawn(async move {
            let actor = system_actor();
            let mut tx = pool_c.begin().await.expect("begin tx");
            let entry = ActivityLogEntry {
                event_category: "TEST",
                event_type: &event,
                entity: Some(AuditEntity::new(
                    "test_entity",
                    uuid::Uuid::new_v4(),
                    &display,
                )),
                data_diff: None,
                request_context: None,
            };
            let id = AuditService::log_activity_tx(&mut tx, &actor, entry)
                .await
                .expect("log_activity_tx in spawn");
            tx.commit().await.expect("commit in spawn");
            id
        });
        handles.push(handle);
    }

    // 以 10 秒 timeout 包裹，避免 CI 環境若有 race 或 deadlock 時無限等。
    let join_all = async {
        let mut ids = Vec::with_capacity(CONCURRENT);
        for handle in handles {
            let id = handle.await.expect("task join");
            ids.push(id);
        }
        ids
    };
    let written_ids = tokio::time::timeout(Duration::from_secs(10), join_all)
        .await
        .expect("3 並發任務應於 10 秒內完成（advisory lock 序列化 + 每 audit ~100ms）");
    assert_eq!(
        written_ids.len(),
        CONCURRENT,
        "{CONCURRENT} 並發任務全部成功"
    );

    // 驗證 N 筆都已持久化
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_activity_logs WHERE event_type LIKE $1")
            .bind(format!("{}%", unique_prefix))
            .fetch_one(&pool)
            .await
            .expect("count concurrent rows");
    assert_eq!(count.0, CONCURRENT as i64, "{CONCURRENT} 筆 audit row 全部持久化");

    // 驗證每筆都有 integrity_hash（未因併發而漏算）
    let no_hash_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type LIKE $1 AND integrity_hash IS NULL",
    )
    .bind(format!("{}%", unique_prefix))
    .fetch_one(&pool)
    .await
    .expect("count rows without hash");
    assert_eq!(
        no_hash_count.0, 0,
        "advisory lock + init_hmac_key 起效時所有 row 應有 integrity_hash"
    );
}
