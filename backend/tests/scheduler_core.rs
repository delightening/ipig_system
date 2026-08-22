//! 整合測試：`SchedulerService`（services/scheduler.rs）核心觸發行為。
//!
//! 背景（R82-2）：scheduler.rs 1866 行、零測試，是排程核心；「該觸發沒觸發」與
//! 「不該觸發卻重複觸發」是全系統性故障（見 docs/reviews/2026-07-10-weakness-assessment.md）。
//! 本檔補回歸防護網，優先覆蓋這兩類最貴的失敗模式，其次覆蓋 leader election（多
//! instance 部署下防止重複註冊排程）與批次通知的當日 / per-entity dedup。
//!
//! **測試邊界**：`should_run_now`（daily/weekly/monthly 排程時窗判定，本次審查點名
//! 的最高風險函式）是 `SchedulerService` 的 private associated fn，且沒有任何 pub
//! 入口會呼叫它——`trigger_low_stock_check` / `trigger_expiry_check` 等 pub 手動觸發
//! 入口刻意繞過它（供 API 手動觸發用）。在不修改 src/ 的前提下，本檔無法直接對
//! `should_run_now` 的時窗比對邏輯做單元測試；改以兩件事補位：對
//! `notification_routing` 的 weekly Monday 08:00 設定值做狀態斷言（回歸保護
//! migration 127 的「週一 08:00」邊界不被意外改回 daily），以及對其餘 pub 觸發
//! 入口的「該建立通知 / 不該建立通知 / 重複呼叫不重複建立」做完整覆蓋。詳見任務
//! 回報中的「疑似 bug / testability gap」段落。

mod common;

use std::sync::Arc;

use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::config::Config;
use erp_backend::constants::SCHEDULER_LEADER_LOCK_ID;
use erp_backend::services::scheduler::SchedulerService;
use erp_backend::services::NotificationService;
use erp_backend::SYSTEM_USER_ID;

// ── 共用 setup：走既有 common::TestApp::spawn()（跑 migrations、設定測試用
//    JWT/AEAD 金鑰等 env——金鑰字面值只存在 common/mod.rs 一處，避免散落多檔
//    觸發 secret scanning）。scheduler 測試不打 HTTP，只取其 db_pool 並另建
//    Config；多起的 axum server 為既有 api_*.rs pattern 的固定成本。──

async fn setup() -> (PgPool, Arc<Config>) {
    let app = common::TestApp::spawn().await;
    // spawn() 已把 Config::from_env() 需要的測試 env 全數就位
    let config = Config::from_env().expect("build Config from env for scheduler_core tests");
    (app.db_pool.clone(), Arc::new(config))
}

// ── seed helpers ──────────────────────────────────────────────

async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("{}-{}@test.local", label, &id.to_string()[..8]))
    .bind(format!("scheduler test {label}"))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

async fn assign_role(pool: &PgPool, user_id: Uuid, role_code: &str) {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("assign role");
}

async fn count_notifications_by_related_type(
    pool: &PgPool,
    user_id: Uuid,
    related_entity_type: &str,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND related_entity_type = $2",
    )
    .bind(user_id)
    .bind(related_entity_type)
    .fetch_one(pool)
    .await
    .expect("count notifications by related type")
}

async fn count_notifications_by_entity(pool: &PgPool, user_id: Uuid, entity_id: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND related_entity_id = $2",
    )
    .bind(user_id)
    .bind(entity_id)
    .fetch_one(pool)
    .await
    .expect("count notifications by entity")
}

async fn seed_warehouse(pool: &PgPool, suffix: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-SC-{}", &suffix[..8]))
        .bind("scheduler_core 測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    id
}

/// 低庫存用品項（safety_stock 設定，走 inventory_snapshots）。
async fn seed_low_stock_product(pool: &PgPool, suffix: &str, safety: i64) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO products (id, sku, name, base_uom, safety_stock) \
         VALUES ($1, $2, '低庫存測試品', '個', $3)",
    )
    .bind(id)
    .bind(format!("SKU-SC-{}", &suffix[..8]))
    .bind(rust_decimal::Decimal::new(safety, 0))
    .execute(pool)
    .await
    .expect("seed low stock product");
    id
}

async fn seed_inventory_snapshot(pool: &PgPool, wh: Uuid, prod: Uuid, qty: i64) {
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base, updated_at) \
         VALUES ($1, $2, $3, NOW())",
    )
    .bind(wh)
    .bind(prod)
    .bind(rust_decimal::Decimal::new(qty, 0))
    .execute(pool)
    .await
    .expect("seed inventory snapshot");
}

/// 效期用品項（track_expiry=true，走 stock_ledger + v_expiry_alerts）。
async fn seed_expiry_product(pool: &PgPool, suffix: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO products (id, sku, name, base_uom, track_expiry) \
         VALUES ($1, $2, '效期測試品', '個', true)",
    )
    .bind(id)
    .bind(format!("SKU-EXP-{}", &suffix[..8]))
    .execute(pool)
    .await
    .expect("seed expiry product");
    id
}

async fn seed_grn_doc(pool: &PgPool, suffix: &str, warehouse_id: Uuid) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let doc_no = format!("GRN-SC-{}", &suffix[..10]);
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(id)
    .bind(&doc_no)
    .bind(warehouse_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed GRN document");
    (id, doc_no)
}

async fn seed_stock_ledger_in(
    pool: &PgPool,
    doc_id: Uuid,
    doc_no: &str,
    wh: Uuid,
    prod: Uuid,
    qty: i64,
    batch_no: &str,
    expiry_date: NaiveDate,
) {
    sqlx::query(
        "INSERT INTO stock_ledger \
            (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base, batch_no, expiry_date) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', $6, $7, $8)",
    )
    .bind(Uuid::new_v4())
    .bind(wh)
    .bind(prod)
    .bind(doc_id)
    .bind(doc_no)
    .bind(rust_decimal::Decimal::new(qty, 0))
    .bind(batch_no)
    .bind(expiry_date)
    .execute(pool)
    .await
    .expect("seed stock_ledger in (with batch/expiry)");
}

/// 已核准 PO（未入庫），回傳 doc_id。
async fn seed_approved_po(pool: &PgPool, suffix: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, doc_date, created_by) \
         VALUES ($1, 'PO', $2, 'approved', CURRENT_DATE, $3)",
    )
    .bind(id)
    .bind(format!("PO-SC-{}", &suffix[..10]))
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed approved PO");
    id
}

/// 為某 PO 建一張已核准 GRN（source_doc_id 指回 PO），代表已入庫。
async fn seed_approved_grn_for_po(pool: &PgPool, suffix: &str, po_id: Uuid, warehouse_id: Uuid) {
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, source_doc_id) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(format!("GRN-FOR-PO-{}", &suffix[..10]))
    .bind(warehouse_id)
    .bind(SYSTEM_USER_ID)
    .bind(po_id)
    .execute(pool)
    .await
    .expect("seed approved GRN for PO");
}

/// 建 protocol(SD=指定 user) + animal(同 iacuc) + 手術(days_ago 天前)。
/// with_sales=true 時加一張時間窗內已核准 DO。回傳 protocol_id。
async fn seed_protocol_with_surgery(
    pool: &PgPool,
    sd_id: Uuid,
    with_sales: bool,
    days_ago: i64,
) -> Uuid {
    let suffix = Uuid::new_v4().simple().to_string();
    let proto_id = Uuid::new_v4();
    let animal_id = Uuid::new_v4();
    let iacuc = format!("IACUC-SC-{}", &suffix[..8]);
    let surgery_date = (Utc::now() - Duration::days(days_ago)).date_naive();

    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, study_director_user_id, iacuc_no, created_by) \
         VALUES ($1, $2, '排程稽核測試計畫', $3, $5, $4, $3)",
    )
    .bind(proto_id)
    .bind(format!("P-SC-{}", &suffix[..10]))
    .bind(SYSTEM_USER_ID)
    .bind(&iacuc)
    .bind(sd_id)
    .execute(pool)
    .await
    .expect("seed protocol");

    sqlx::query(
        "INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no) \
         VALUES ($1, $2, 'other', 'male', CURRENT_DATE, $3)",
    )
    .bind(animal_id)
    .bind(format!("E{}", &suffix[..8]))
    .bind(&iacuc)
    .execute(pool)
    .await
    .expect("seed animal");

    sqlx::query(
        "INSERT INTO animal_surgeries (animal_id, surgery_date, surgery_site) \
         VALUES ($1, $2, '排程測試手術')",
    )
    .bind(animal_id)
    .bind(surgery_date)
    .execute(pool)
    .await
    .expect("seed surgery");

    if with_sales {
        sqlx::query(
            "INSERT INTO documents (id, doc_type, doc_no, status, protocol_id, doc_date, created_by) \
             VALUES ($1, 'SO', $2, 'approved', $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(format!("SO-SC-{}", &suffix[..10]))
        .bind(proto_id)
        .bind(surgery_date)
        .bind(SYSTEM_USER_ID)
        .execute(pool)
        .await
        .expect("seed sales SO");
    }

    proto_id
}

// ── 1) 低庫存週報批次觸發（trigger_low_stock_check）──────────────

/// 該觸發：admin 角色（low_stock_alert 路由角色之一）在有低庫存品項時應收到通知。
#[tokio::test]
async fn low_stock_trigger_notifies_routed_role_when_alert_exists() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let admin_user = seed_user(&pool, "lsadmin").await;
    assign_role(&pool, admin_user, "admin").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_low_stock_product(&pool, &suffix, 100).await;
    seed_inventory_snapshot(&pool, wh, prod, 10).await; // 10 < safety 100 → 低庫存

    SchedulerService::trigger_low_stock_check(&pool, &config)
        .await
        .expect("trigger_low_stock_check 應成功");

    let n = count_notifications_by_related_type(&pool, admin_user, "low_stock").await;
    assert_eq!(n, 1, "admin 角色應收到正好 1 則低庫存通知，實得 {n}");
}

/// 不該觸發：非路由角色（VET）即使系統內有低庫存品項也不應收到低庫存通知。
/// （比全域「無任何低庫存品項才不通知」更穩健——不受同進程其他測試殘留資料影響。）
#[tokio::test]
async fn low_stock_trigger_does_not_notify_unrelated_role() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let vet_user = seed_user(&pool, "lsvet").await;
    assign_role(&pool, vet_user, "VET").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_low_stock_product(&pool, &suffix, 100).await;
    seed_inventory_snapshot(&pool, wh, prod, 10).await;

    SchedulerService::trigger_low_stock_check(&pool, &config)
        .await
        .expect("trigger_low_stock_check 應成功");

    let n = count_notifications_by_related_type(&pool, vet_user, "low_stock").await;
    assert_eq!(n, 0, "VET 非低庫存路由角色，不應收到通知，實得 {n}");
}

/// 不該重複觸發：同一使用者當天重複呼叫 trigger_low_stock_check 兩次，
/// 只應建立 1 則通知（`has_today_notification` 當日 dedup）——這正是
/// 「排程重複觸發」在通知層最貴的失敗模式。
#[tokio::test]
async fn low_stock_trigger_repeated_call_same_day_does_not_duplicate() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let admin_user = seed_user(&pool, "lsdup").await;
    assign_role(&pool, admin_user, "admin").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_low_stock_product(&pool, &suffix, 100).await;
    seed_inventory_snapshot(&pool, wh, prod, 10).await;

    SchedulerService::trigger_low_stock_check(&pool, &config)
        .await
        .expect("第一次 trigger_low_stock_check 應成功");
    SchedulerService::trigger_low_stock_check(&pool, &config)
        .await
        .expect("第二次 trigger_low_stock_check 應成功（模擬排程重複觸發）");

    let n = count_notifications_by_related_type(&pool, admin_user, "low_stock").await;
    assert_eq!(n, 1, "重複觸發不應重複建立通知，應恰好 1 則，實得 {n}");
}

/// 多收件者各自 notification_settings 分流：同一批觸發中，
/// email_low_stock=false 的使用者應被跳過，缺 notification_settings 列的使用者
/// 應維持「預設視為開啟」的行為，email_low_stock=true 的使用者正常收到。
/// 鎖住 per-recipient 設定判斷的分流行為，供後續查詢批次化（N+1 → 批次）重構的回歸網。
#[tokio::test]
async fn low_stock_trigger_respects_per_recipient_settings_including_missing_row() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let user_disabled = seed_user(&pool, "lsdisabled").await;
    assign_role(&pool, user_disabled, "admin").await;
    sqlx::query("UPDATE notification_settings SET email_low_stock = false WHERE user_id = $1")
        .bind(user_disabled)
        .execute(&pool)
        .await
        .expect("disable email_low_stock for user_disabled");

    let user_missing_settings = seed_user(&pool, "lsmissing").await;
    assign_role(&pool, user_missing_settings, "admin").await;
    sqlx::query("DELETE FROM notification_settings WHERE user_id = $1")
        .bind(user_missing_settings)
        .execute(&pool)
        .await
        .expect("delete notification_settings row for user_missing_settings");

    let user_enabled = seed_user(&pool, "lsenabled").await;
    assign_role(&pool, user_enabled, "admin").await;
    // 預設 email_low_stock = true，不需額外更新。

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_low_stock_product(&pool, &suffix, 100).await;
    seed_inventory_snapshot(&pool, wh, prod, 10).await;

    SchedulerService::trigger_low_stock_check(&pool, &config)
        .await
        .expect("trigger_low_stock_check 應成功");

    let n_disabled = count_notifications_by_related_type(&pool, user_disabled, "low_stock").await;
    assert_eq!(
        n_disabled, 0,
        "email_low_stock=false 應被跳過，實得 {n_disabled}"
    );

    let n_missing =
        count_notifications_by_related_type(&pool, user_missing_settings, "low_stock").await;
    assert_eq!(
        n_missing, 1,
        "缺 notification_settings 列應視為預設開啟，應收到 1 則，實得 {n_missing}"
    );

    let n_enabled = count_notifications_by_related_type(&pool, user_enabled, "low_stock").await;
    assert_eq!(
        n_enabled, 1,
        "email_low_stock=true 應正常收到 1 則，實得 {n_enabled}"
    );
}

// ── 2) 效期預警週報批次觸發（trigger_expiry_check）────────────────

/// 該觸發：admin 角色在有效期預警品項時應收到通知（expiry_date 落在 v_expiry_alerts
/// 的 [今日-90, 今日+60] 窗內、track_expiry=true、on_hand>0）。
#[tokio::test]
async fn expiry_trigger_notifies_routed_role_when_alert_exists() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let admin_user = seed_user(&pool, "expadmin").await;
    assign_role(&pool, admin_user, "admin").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_expiry_product(&pool, &suffix).await;
    let (doc_id, doc_no) = seed_grn_doc(&pool, &suffix, wh).await;
    let near_expiry = Utc::now().date_naive() + Duration::days(10);
    seed_stock_ledger_in(
        &pool,
        doc_id,
        &doc_no,
        wh,
        prod,
        20,
        "BATCH-EXP",
        near_expiry,
    )
    .await;

    SchedulerService::trigger_expiry_check(&pool, &config)
        .await
        .expect("trigger_expiry_check 應成功");

    let n = count_notifications_by_related_type(&pool, admin_user, "expiry_warning").await;
    assert_eq!(n, 1, "admin 角色應收到正好 1 則效期預警通知，實得 {n}");
}

/// 不該觸發：非路由角色（VET，expiry_alert 路由只有 admin/WAREHOUSE_MANAGER）
/// 即使系統內有效期預警品項也不應收到通知。
#[tokio::test]
async fn expiry_trigger_does_not_notify_unrelated_role() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let vet_user = seed_user(&pool, "expvet").await;
    assign_role(&pool, vet_user, "VET").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_expiry_product(&pool, &suffix).await;
    let (doc_id, doc_no) = seed_grn_doc(&pool, &suffix, wh).await;
    let near_expiry = Utc::now().date_naive() + Duration::days(10);
    seed_stock_ledger_in(
        &pool,
        doc_id,
        &doc_no,
        wh,
        prod,
        20,
        "BATCH-EXP2",
        near_expiry,
    )
    .await;

    SchedulerService::trigger_expiry_check(&pool, &config)
        .await
        .expect("trigger_expiry_check 應成功");

    let n = count_notifications_by_related_type(&pool, vet_user, "expiry_warning").await;
    assert_eq!(n, 0, "VET 非效期預警路由角色，不應收到通知，實得 {n}");
}

/// 不該重複觸發：同一使用者當天重複呼叫 trigger_expiry_check 兩次，只應建立 1 則通知。
#[tokio::test]
async fn expiry_trigger_repeated_call_same_day_does_not_duplicate() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let admin_user = seed_user(&pool, "expdup").await;
    assign_role(&pool, admin_user, "admin").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_expiry_product(&pool, &suffix).await;
    let (doc_id, doc_no) = seed_grn_doc(&pool, &suffix, wh).await;
    let near_expiry = Utc::now().date_naive() + Duration::days(10);
    seed_stock_ledger_in(
        &pool,
        doc_id,
        &doc_no,
        wh,
        prod,
        20,
        "BATCH-EXP3",
        near_expiry,
    )
    .await;

    SchedulerService::trigger_expiry_check(&pool, &config)
        .await
        .expect("第一次 trigger_expiry_check 應成功");
    SchedulerService::trigger_expiry_check(&pool, &config)
        .await
        .expect("第二次 trigger_expiry_check 應成功（模擬排程重複觸發）");

    let n = count_notifications_by_related_type(&pool, admin_user, "expiry_warning").await;
    assert_eq!(n, 1, "重複觸發不應重複建立通知，應恰好 1 則，實得 {n}");
}

/// 多收件者各自 notification_settings 分流：同一批觸發中，
/// email_expiry_warning=false 的使用者應被跳過，缺 notification_settings 列的使用者
/// 應維持「預設視為開啟」的行為，email_expiry_warning=true 的使用者正常收到。
/// 鎖住 per-recipient 設定判斷的分流行為，供後續查詢批次化（N+1 → 批次）重構的回歸網。
#[tokio::test]
async fn expiry_trigger_respects_per_recipient_settings_including_missing_row() {
    let (pool, config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let user_disabled = seed_user(&pool, "expdisabled").await;
    assign_role(&pool, user_disabled, "admin").await;
    sqlx::query("UPDATE notification_settings SET email_expiry_warning = false WHERE user_id = $1")
        .bind(user_disabled)
        .execute(&pool)
        .await
        .expect("disable email_expiry_warning for user_disabled");

    let user_missing_settings = seed_user(&pool, "expmissing").await;
    assign_role(&pool, user_missing_settings, "admin").await;
    sqlx::query("DELETE FROM notification_settings WHERE user_id = $1")
        .bind(user_missing_settings)
        .execute(&pool)
        .await
        .expect("delete notification_settings row for user_missing_settings");

    let user_enabled = seed_user(&pool, "expenabled").await;
    assign_role(&pool, user_enabled, "admin").await;
    // 預設 email_expiry_warning = true，不需額外更新。

    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_expiry_product(&pool, &suffix).await;
    let (doc_id, doc_no) = seed_grn_doc(&pool, &suffix, wh).await;
    let near_expiry = Utc::now().date_naive() + Duration::days(10);
    seed_stock_ledger_in(
        &pool,
        doc_id,
        &doc_no,
        wh,
        prod,
        20,
        "BATCH-EXP-MULTI",
        near_expiry,
    )
    .await;

    SchedulerService::trigger_expiry_check(&pool, &config)
        .await
        .expect("trigger_expiry_check 應成功");

    let n_disabled =
        count_notifications_by_related_type(&pool, user_disabled, "expiry_warning").await;
    assert_eq!(
        n_disabled, 0,
        "email_expiry_warning=false 應被跳過，實得 {n_disabled}"
    );

    let n_missing =
        count_notifications_by_related_type(&pool, user_missing_settings, "expiry_warning").await;
    assert_eq!(
        n_missing, 1,
        "缺 notification_settings 列應視為預設開啟，應收到 1 則，實得 {n_missing}"
    );

    let n_enabled =
        count_notifications_by_related_type(&pool, user_enabled, "expiry_warning").await;
    assert_eq!(
        n_enabled, 1,
        "email_expiry_warning=true 應正常收到 1 則，實得 {n_enabled}"
    );
}

/// R82-11 迴歸：`send_expiry_notifications` 只依「呼叫端傳入的 alerts」建立通知，
/// 不再獨立重查寫死視窗的 `v_expiry_alerts`。這確保 in-app 通知與 scheduler 依 admin
/// 設定（`fn_expiry_alerts(warn_days, cutoff_days)`）算出的 config-aware 清單同源，
/// 管理員調整的預警視窗對兩個通道皆生效。
///
/// 傳「空」alerts：即使系統內確有一筆在 `v_expiry_alerts` 視窗內的效期品項，也不應
/// 建立任何通知。舊碼（內部 `SELECT * FROM v_expiry_alerts`）會據此誤發——red-on-old。
#[tokio::test]
async fn expiry_notifications_use_passed_alerts_not_hardcoded_view() {
    let (pool, _config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let admin_user = seed_user(&pool, "expcfg").await;
    assign_role(&pool, admin_user, "admin").await;

    // 系統內確有一筆效期品項（落在 v_expiry_alerts 的 [今日-90, 今日+60] 窗內）：
    // 舊碼會重查此 view 而對 admin 誤發通知。
    let wh = seed_warehouse(&pool, &suffix).await;
    let prod = seed_expiry_product(&pool, &suffix).await;
    let (doc_id, doc_no) = seed_grn_doc(&pool, &suffix, wh).await;
    seed_stock_ledger_in(
        &pool,
        doc_id,
        &doc_no,
        wh,
        prod,
        20,
        "BATCH-CFG",
        Utc::now().date_naive() + Duration::days(10),
    )
    .await;

    // 傳入空 alerts → 新碼直接回 0、不重查 v_expiry_alerts。
    let svc = NotificationService::new(pool.clone());
    let sent = svc
        .send_expiry_notifications(&[])
        .await
        .expect("send_expiry_notifications 應成功");
    assert_eq!(sent, 0, "傳空 alerts 應 0 收件者，實得 {sent}");

    let got = count_notifications_by_related_type(&pool, admin_user, "expiry_warning").await;
    assert_eq!(
        got, 0,
        "傳空 alerts 不應建立任何通知（舊碼會重查寫死的 v_expiry_alerts 而誤發）；實得 {got}"
    );
}

// ── 3) 採購單未入庫提醒（trigger_po_pending_receipt_check）────────

/// 該觸發：已核准但未入庫的 PO → 倉管收到置頂提醒。
#[tokio::test]
async fn po_pending_receipt_notifies_warehouse_manager_for_unreceived_po() {
    let (pool, _config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let wm_user = seed_user(&pool, "pownnotif").await;
    assign_role(&pool, wm_user, "WAREHOUSE_MANAGER").await;

    let po_id = seed_approved_po(&pool, &suffix).await;

    SchedulerService::trigger_po_pending_receipt_check(&pool)
        .await
        .expect("trigger_po_pending_receipt_check 應成功");

    let n = count_notifications_by_entity(&pool, wm_user, po_id).await;
    assert_eq!(n, 1, "未入庫 PO 應通知倉管 1 則，實得 {n}");
}

/// 不該觸發：已入庫（有對應已核准 GRN）的 PO 不應產生未入庫提醒。
#[tokio::test]
async fn po_pending_receipt_no_notification_when_grn_approved() {
    let (pool, _config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let wm_user = seed_user(&pool, "powreceived").await;
    assign_role(&pool, wm_user, "WAREHOUSE_MANAGER").await;

    let wh = seed_warehouse(&pool, &suffix).await;
    let po_id = seed_approved_po(&pool, &suffix).await;
    seed_approved_grn_for_po(&pool, &suffix, po_id, wh).await;

    SchedulerService::trigger_po_pending_receipt_check(&pool)
        .await
        .expect("trigger_po_pending_receipt_check 應成功");

    let n = count_notifications_by_entity(&pool, wm_user, po_id).await;
    assert_eq!(n, 0, "已入庫 PO 不應產生未入庫提醒，實得 {n}");
}

/// 不該重複觸發：同一張未入庫 PO 重複觸發兩次，該倉管只應收到 1 則提醒
/// （`notify_po_pending_receipt` 以 notifications 表 per-PO dedup）。
#[tokio::test]
async fn po_pending_receipt_repeated_trigger_does_not_duplicate() {
    let (pool, _config) = setup().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let wm_user = seed_user(&pool, "powdup").await;
    assign_role(&pool, wm_user, "WAREHOUSE_MANAGER").await;

    let po_id = seed_approved_po(&pool, &suffix).await;

    SchedulerService::trigger_po_pending_receipt_check(&pool)
        .await
        .expect("第一次 trigger_po_pending_receipt_check 應成功");
    SchedulerService::trigger_po_pending_receipt_check(&pool)
        .await
        .expect("第二次 trigger_po_pending_receipt_check 應成功（模擬排程重複觸發）");

    let n = count_notifications_by_entity(&pool, wm_user, po_id).await;
    assert_eq!(
        n, 1,
        "重複觸發同一張未入庫 PO 不應重複提醒，應恰好 1 則，實得 {n}"
    );
}

// ── 4) 手術缺銷貨單據稽核（trigger_surgery_sales_audit）──────────

/// 該觸發：計畫有手術但缺對應銷貨單據 → 該計畫 SD 收到通知。
#[tokio::test]
async fn surgery_sales_audit_notifies_sd_for_missing_sales() {
    let (pool, _config) = setup().await;

    let sd_user = seed_user(&pool, "surgnotif").await;
    sqlx::query("UPDATE users SET is_active = true WHERE id = $1")
        .bind(sd_user)
        .execute(&pool)
        .await
        .expect("ensure sd active");

    let proto_id = seed_protocol_with_surgery(&pool, sd_user, false, 30).await;

    SchedulerService::trigger_surgery_sales_audit(&pool)
        .await
        .expect("trigger_surgery_sales_audit 應成功");

    let n = count_notifications_by_entity(&pool, sd_user, proto_id).await;
    assert_eq!(n, 1, "缺銷貨單據的計畫應通知 SD 1 則，實得 {n}");
}

/// 不該觸發：手術時間窗內已有核准銷貨單據 → 不應產生稽核通知。
#[tokio::test]
async fn surgery_sales_audit_no_notification_when_sales_exists() {
    let (pool, _config) = setup().await;

    let sd_user = seed_user(&pool, "surgok").await;
    let proto_id = seed_protocol_with_surgery(&pool, sd_user, true, 30).await;

    SchedulerService::trigger_surgery_sales_audit(&pool)
        .await
        .expect("trigger_surgery_sales_audit 應成功");

    let n = count_notifications_by_entity(&pool, sd_user, proto_id).await;
    assert_eq!(n, 0, "有對應銷貨單據不應產生稽核通知，實得 {n}");
}

/// 不該重複觸發：同一計畫重複稽核兩次，SD 只應收到 1 則通知
/// （`notify_surgery_missing_sales` 以 notifications 表 per-protocol-per-user dedup）。
#[tokio::test]
async fn surgery_sales_audit_repeated_trigger_does_not_duplicate() {
    let (pool, _config) = setup().await;

    let sd_user = seed_user(&pool, "surgdup").await;
    let proto_id = seed_protocol_with_surgery(&pool, sd_user, false, 30).await;

    SchedulerService::trigger_surgery_sales_audit(&pool)
        .await
        .expect("第一次 trigger_surgery_sales_audit 應成功");
    SchedulerService::trigger_surgery_sales_audit(&pool)
        .await
        .expect("第二次 trigger_surgery_sales_audit 應成功（模擬排程重複觸發）");

    let n = count_notifications_by_entity(&pool, sd_user, proto_id).await;
    assert_eq!(
        n, 1,
        "重複觸發同一計畫稽核不應重複通知，應恰好 1 則，實得 {n}"
    );
}

// ── 5) Leader election（多 instance 部署防重複註冊排程）──────────

/// 該有的保護：另一個 instance 已持有 leader advisory lock 時，本 instance
/// `SchedulerService::start` 應偵測到並跳過 job 註冊、正常回傳 Ok（不 hang、不 error）。
/// 這是「多 instance 部署下重複觸發」的第一道防線——驗證此路徑不會崩潰或卡死。
#[tokio::test]
#[serial_test::serial]
async fn scheduler_start_returns_ok_when_leader_lock_already_held_elsewhere() {
    let (pool, config) = setup().await;

    // 模擬「另一個 instance」：用獨立連線持有 session-level advisory lock。
    let mut holder_conn = pool.acquire().await.expect("acquire holder connection");
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(SCHEDULER_LEADER_LOCK_ID)
        .fetch_one(&mut *holder_conn)
        .await
        .expect("holder acquire lock");
    assert!(acquired, "測試前置：holder 連線應能取得 leader lock");

    let token = tokio_util::sync::CancellationToken::new();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        SchedulerService::start(pool.clone(), config.clone(), token.clone()),
    )
    .await
    .expect("start() 在鎖被佔用時不應 hang（15s timeout）");

    assert!(
        result.is_ok(),
        "lock 被其他 instance 持有時，start() 仍應正常回傳 Ok（跳過註冊），實得 {:?}",
        result.err().map(|e| e.to_string())
    );

    // 清理：釋放測試持有的 lock（顯式 unlock，避免 pooled connection 不關閉造成殘留）。
    let unlocked: bool = sqlx::query_scalar("SELECT pg_advisory_unlock($1)")
        .bind(SCHEDULER_LEADER_LOCK_ID)
        .fetch_one(&mut *holder_conn)
        .await
        .expect("holder release lock");
    assert!(unlocked, "應成功釋放測試持有的 leader lock");
    drop(holder_conn);
    token.cancel();
}

/// 該觸發的路徑：lock 未被佔用時，本 instance 應成為 leader 並成功完成啟動
/// （所有 job 註冊不 error）。事後 cancel token 並顯式釋放鎖，避免鎖殘留影響其他測試。
#[tokio::test]
#[serial_test::serial]
async fn scheduler_start_becomes_leader_and_starts_when_lock_free() {
    let (pool, config) = setup().await;

    let token = tokio_util::sync::CancellationToken::new();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        SchedulerService::start(pool.clone(), config.clone(), token.clone()),
    )
    .await
    .expect("start() 在鎖可取得時不應 hang（15s timeout）");

    let mut sched = result.expect("lock 可取得時 start() 應成功並註冊所有 job");

    // 收尾：cancel shutdown token（觸發背景 lock 釋放 task）+ 停止 scheduler，
    // 避免遺留背景排程任務／advisory lock 影響後續測試。
    token.cancel();
    let _ = sched.shutdown().await;

    // 確認 lock 確實已釋放（否則下一個 leader 測試會被誤判為「鎖仍被佔用」）。
    // 背景 unlock task 與本測試同 runtime 並發，CI 高負載下完成時機不定——
    // 用 5s 上限的 retry loop 取代固定 sleep，避免 timing-based flaky。
    let mut check_conn = pool.acquire().await.expect("acquire check connection");
    let reacquired = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let ok: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
                .bind(SCHEDULER_LEADER_LOCK_ID)
                .fetch_one(&mut *check_conn)
                .await
                .expect("re-acquire lock after shutdown");
            if ok {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("leader lock 未在 5s 內釋放");
    assert!(
        reacquired,
        "shutdown token cancel 後，leader lock 應已釋放供後續 instance 取得"
    );
    let _: bool = sqlx::query_scalar("SELECT pg_advisory_unlock($1)")
        .bind(SCHEDULER_LEADER_LOCK_ID)
        .fetch_one(&mut *check_conn)
        .await
        .expect("release re-acquired lock");
}

// ── 6) 排程時窗設定值回歸保護（週一 08:00 weekly 邊界）────────────

/// 回歸保護：migration 127 已將 low_stock_alert / expiry_alert 由 daily 改為
/// weekly（週一 08:00）。`should_run_now` 的判定邏輯是 private 且無 pub 入口可直接
/// 呼叫（見檔頭說明），故改以斷言驅動判定的 DB 設定值本身，防止未來 migration
/// 不慎把這兩個事件的 routing 改回 daily 或改到其他 hour/day 而不自知。
#[tokio::test]
async fn low_stock_and_expiry_routing_is_weekly_monday_8am() {
    let (pool, _config) = setup().await;

    let rows: Vec<(String, String, i16, Option<i16>)> = sqlx::query_as(
        "SELECT event_type, frequency, hour_of_day, day_of_week \
         FROM notification_routing \
         WHERE event_type IN ('low_stock_alert', 'expiry_alert') AND is_active = true",
    )
    .fetch_all(&pool)
    .await
    .expect("fetch low_stock_alert / expiry_alert routing rows");

    assert!(
        !rows.is_empty(),
        "low_stock_alert / expiry_alert 應至少各有一列 active routing"
    );

    for (event_type, frequency, hour_of_day, day_of_week) in rows {
        assert_eq!(
            frequency, "weekly",
            "{event_type} 的 frequency 應為 weekly（migration 127），實得 {frequency}"
        );
        assert_eq!(
            hour_of_day, 8,
            "{event_type} 的 hour_of_day 應為 8（週一 08:00），實得 {hour_of_day}"
        );
        assert_eq!(
            day_of_week,
            Some(1),
            "{event_type} 的 day_of_week 應為 1（週一，0=週日起算），實得 {day_of_week:?}"
        );
    }
}
