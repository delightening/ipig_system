//! R53-6: PI audit log entity_type blacklist 整合測試
//!
//! 驗證：`byproduct_sample` entity_type 整類不出現在 `AuditService::list_activities`
//! 與 `export_activities` 的結果中（即使 admin 查也看不到 — 全 viewer 一致過濾，
//! 避免「忘記檢查 viewer role」的 bug）。
//!
//! Admin 需稽核時請走 `/euthanasia/:id/byproduct-samples` API（其自有
//! `animal.byproduct_sample.view` permission gate）。

mod common;

use chrono::{Duration, Utc};
use common::TestApp;
use erp_backend::models::ActivityLogQuery;
use erp_backend::services::audit::{
    ActivityLogEntry, AuditEntity, AuditService, AUDIT_ENTITY_BLACKLIST,
};
use erp_backend::{ActorContext, SYSTEM_USER_ID};
use serial_test::serial;
use uuid::Uuid;

fn system_actor() -> ActorContext {
    ActorContext::System {
        reason: "r53_6_blacklist_test",
    }
}

/// 取 blacklist 第一項作為「被過濾的 entity_type」測試樣本。確保測試與
/// 實作同步，未來 blacklist 內容變更不需手動改測試（Gemini review）。
fn blacklisted_entity_type() -> &'static str {
    AUDIT_ENTITY_BLACKLIST
        .first()
        .copied()
        .expect("AUDIT_ENTITY_BLACKLIST 不可為空")
}

#[tokio::test]
#[serial]
async fn list_activities_filters_byproduct_sample_entries() {
    let app = TestApp::spawn().await;
    let blacklisted = blacklisted_entity_type();

    // 寫 1 筆 blacklist entity_type 事件（應被過濾）
    let bp_id = Uuid::new_v4();
    let mut tx = app.db_pool.begin().await.expect("tx begin");
    AuditService::log_activity_tx(
        &mut tx,
        &system_actor(),
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "BYPRODUCT_SAMPLE_CREATE",
            entity: Some(AuditEntity::new(blacklisted, bp_id, "test sample")),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    .expect("write byproduct event");
    tx.commit().await.expect("commit");

    // 寫 1 筆非黑名單事件（animal）作對照
    let animal_id = Uuid::new_v4();
    let mut tx = app.db_pool.begin().await.expect("tx begin");
    AuditService::log_activity_tx(
        &mut tx,
        &system_actor(),
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "ANIMAL_CREATE",
            entity: Some(AuditEntity::new("animal", animal_id, "test animal")),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    .expect("write animal event");
    tx.commit().await.expect("commit");

    // List — actor 過濾為 SYSTEM_USER_ID 只看本次測試寫的兩筆。
    // 用昨日為 from 避免午夜邊界 flaky（Gemini review）。
    let query = ActivityLogQuery {
        user_id: Some(SYSTEM_USER_ID),
        event_category: None,
        event_type: None,
        entity_type: None,
        entity_id: None,
        is_suspicious: None,
        from: Some((Utc::now() - Duration::days(1)).date_naive()),
        to: None,
        query: None,
        page: None,
        per_page: Some(100),
    };
    let result = AuditService::list_activities(&app.db_pool, &query)
        .await
        .expect("list_activities");

    let entity_types: Vec<&str> = result
        .data
        .iter()
        .filter_map(|r| r.entity_type.as_deref())
        .collect();
    assert!(
        !entity_types.contains(&blacklisted),
        "{} 應被 blacklist 過濾，實際出現：{:?}",
        blacklisted,
        entity_types,
    );
    assert!(
        entity_types.contains(&"animal"),
        "對照組 animal 應出現，實際結果：{:?}",
        entity_types,
    );
}

#[tokio::test]
#[serial]
async fn export_activities_filters_byproduct_sample_entries() {
    let app = TestApp::spawn().await;
    let blacklisted = blacklisted_entity_type();

    let bp_id = Uuid::new_v4();
    let mut tx = app.db_pool.begin().await.expect("tx begin");
    AuditService::log_activity_tx(
        &mut tx,
        &system_actor(),
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "BYPRODUCT_SAMPLE_UPDATE",
            entity: Some(AuditEntity::new(blacklisted, bp_id, "export-test")),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    .expect("write byproduct event");
    tx.commit().await.expect("commit");

    let query = ActivityLogQuery {
        user_id: Some(SYSTEM_USER_ID),
        event_category: None,
        event_type: None,
        entity_type: Some(blacklisted.to_string()), // 明確指定也應過濾掉
        entity_id: None,
        is_suspicious: None,
        from: Some((Utc::now() - Duration::days(1)).date_naive()),
        to: None,
        query: None,
        page: None,
        per_page: None,
    };
    let result = AuditService::export_activities(&app.db_pool, &query)
        .await
        .expect("export_activities");
    let entity_types: Vec<&str> = result
        .iter()
        .filter_map(|r| r.entity_type.as_deref())
        .collect();
    assert!(
        !entity_types.contains(&blacklisted),
        "{} 應被 blacklist 過濾（即使 caller 明確 query 它）",
        blacklisted,
    );
}
