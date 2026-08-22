//! R84-1：同單同品項透支漏洞修復 acceptance tests。
//!
//! 背景：`process_document`（`backend/src/services/stock/ledger.rs`）舊版在所有明細行
//! 都跑完 `process_single_line` 後，才統一對每個 (warehouse, product) 重算一次
//! `inventory_snapshots`。若同一張單裡兩行命中同一個 (warehouse, product)，第二行的
//! `check_stock_available` 讀到的仍是「本單處理前」的舊快照 —— 兩行各自檢查都可能通過，
//! 但加總已經超賣，核准完後快照重算才會變成負值。
//!
//! 這個漏洞在 SO／ADJ 等**有指定儲位**的單據上，會被 `decrement_storage_location_inventory`
//! 的「單一儲位原子扣帳＋`on_hand_qty >= qty` 下限」攔住第二行（同儲位或不同儲位皆然，
//! 因為兩個不同儲位若各自都夠扣，加總本來就不會超賣）。真正攔不住的情境是
//! **warehouse-only（無指定儲位）**：`storage_location_from_id = None` 時
//! `decrement_storage_location_inventory` 根本不會被呼叫（見 `process_transfer` 的
//! `if let Some(from_loc) = ...`），此時唯一的防線只剩 `check_stock_available`，
//! 這正是本檔測試的情境（調撥單 TR，`requires_shelf()` 排除 TR，故不強制填儲位）。
//!
//! 涵蓋：
//! - T1 同單兩行同倉同品項、無指定儲位，加總超賣 → 核准應失敗，且完全 rollback（無部分扣帳、
//!   快照不變），而非核准成功後留下負庫存。
//! - T2 同單兩行同倉同品項、無指定儲位，加總未超賣 → 核准應成功，快照正確反映兩行加總扣減。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{CreateDocumentRequest, DocType, DocumentLineInput};
use erp_backend::services::DocumentService;
use erp_backend::{ActorContext, SYSTEM_USER_ID};

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

fn actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "test-r84-1@example.com".into(),
        // 授權實際靠 `admin`（`has_permission` / `is_admin` 短路）。
        // 原本還帶一個 "STUDY_DIRECTOR" 字串，是裝飾性的——`authorize_sales_document`
        // 查的是 permissions 不是 roles。該全域角色已移除，字串一併清掉避免誤導。
        roles: vec!["admin".into(), "WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// R97-1：GRN 禁止自核，前置入庫改由這位非建單者核准。
/// 本檔測的是同品項多明細的超領防護（TR），前置 GRN 由誰核准不影響待測行為。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "approver-r84-1").await,
        email: "approver-r84-1@example.com".into(),
        roles: vec!["admin".into(), "WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-approver".into(),
        exp: 0,
        impersonated_by: None,
    })
}

async fn seed_warehouse(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &s[..10]))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    id
}

async fn seed_shelf(pool: &PgPool, warehouse_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO storage_locations (id, warehouse_id, code) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(warehouse_id)
        .bind(format!("A-{}", &s[..10]))
        .execute(pool)
        .await
        .expect("seed shelf");
    id
}

async fn seed_product(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("SKU-{}", &s[..10]))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

fn grn_line(product: Uuid, qty: i64, shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: Some(Decimal::from(10)),
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: Some(shelf),
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

/// 調撥出庫明細：**故意不填** `storage_location_from_id`（warehouse-only 顆粒度），
/// 這樣才會落到「唯一防線是 check_stock_available」的情境，見檔頭說明。
fn tr_line(product: Uuid, qty: i64, to_shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: None,
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: None,
        storage_location_from_id: None,
        storage_location_to_id: Some(to_shelf),
    }
}

fn tr_req(
    warehouse_from: Uuid,
    warehouse_to: Uuid,
    lines: Vec<DocumentLineInput>,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type: DocType::TR,
        warehouse_id: None,
        warehouse_from_id: Some(warehouse_from),
        warehouse_to_id: Some(warehouse_to),
        partner_id: None,
        source_doc_id: None,
        doc_date: chrono::Utc::now().date_naive(),
        remark: None,
        stocktake_scope: None,
        iacuc_no: None,
        protocol_id: None,
        lines,
    }
}

async fn seed_stock(pool: &PgPool, warehouse: Uuid, shelf: Uuid, product: Uuid, qty: i64) {
    let grn = DocumentService::create(
        pool,
        &actor(),
        &CreateDocumentRequest {
            doc_type: DocType::GRN,
            warehouse_id: Some(warehouse),
            warehouse_from_id: None,
            warehouse_to_id: None,
            partner_id: None,
            source_doc_id: None,
            doc_date: chrono::Utc::now().date_naive(),
            remark: None,
            stocktake_scope: None,
            iacuc_no: None,
            protocol_id: None,
            lines: vec![grn_line(product, qty, shelf)],
        },
    )
    .await
    .expect("create GRN");
    DocumentService::submit(pool, &actor(), grn.document.id)
        .await
        .expect("submit GRN");
    let approver = approver_actor(pool).await;
    DocumentService::approve(pool, &approver, grn.document.id)
        .await
        .expect("approve GRN → 入庫");
}

async fn snapshot_qty(pool: &PgPool, warehouse: Uuid, product: Uuid) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(on_hand_qty_base, 0) FROM inventory_snapshots WHERE warehouse_id = $1 AND product_id = $2",
    )
    .bind(warehouse)
    .bind(product)
    .fetch_optional(pool)
    .await
    .expect("query snapshot")
    .unwrap_or(Decimal::ZERO)
}

#[tokio::test]
#[serial]
async fn tr_two_lines_same_product_overdraft_is_rejected_not_left_negative() {
    let pool = setup_pool().await;
    let wh_from = seed_warehouse(&pool).await;
    let wh_to = seed_warehouse(&pool).await;
    let shelf_from = seed_shelf(&pool, wh_from).await;
    let shelf_to = seed_shelf(&pool, wh_to).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh_from, shelf_from, product, 100).await;

    // 一張 TR：同倉同品項兩行、皆無指定來源儲位（warehouse-only），各扣 60，加總 120 > 100。
    let tr = DocumentService::create(
        &pool,
        &actor(),
        &tr_req(
            wh_from,
            wh_to,
            vec![
                tr_line(product, 60, shelf_to),
                tr_line(product, 60, shelf_to),
            ],
        ),
    )
    .await
    .expect("create TR");
    DocumentService::submit(&pool, &actor(), tr.document.id)
        .await
        .expect("submit TR");

    let err = DocumentService::approve(&pool, &actor(), tr.document.id)
        .await
        .expect_err("同單兩行同品項加總超賣，核准應失敗，不得留下負庫存");
    assert!(
        err.to_string().contains("Insufficient") || err.to_string().contains("庫存"),
        "應回庫存不足錯誤，實際：{err}"
    );

    // 原子性：核准失敗應完全 rollback，不留任何 ledger 紀錄，快照維持原值（不是負數）。
    let ledger_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM stock_ledger WHERE doc_id = $1")
            .bind(tr.document.id)
            .fetch_one(&pool)
            .await
            .expect("count ledger");
    assert_eq!(ledger_count, 0, "核准失敗應完全 rollback，無任何流水紀錄");

    let final_qty = snapshot_qty(&pool, wh_from, product).await;
    assert_eq!(
        final_qty,
        Decimal::from(100),
        "快照應維持原值 100，不得因第二行的過期檢查而變成負數（-20）"
    );
    assert!(final_qty >= Decimal::ZERO, "庫存快照絕不應該是負值");
}

#[tokio::test]
#[serial]
async fn tr_two_lines_same_product_within_stock_both_succeed() {
    let pool = setup_pool().await;
    let wh_from = seed_warehouse(&pool).await;
    let wh_to = seed_warehouse(&pool).await;
    let shelf_from = seed_shelf(&pool, wh_from).await;
    let shelf_to = seed_shelf(&pool, wh_to).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh_from, shelf_from, product, 100).await;

    // 同倉同品項兩行、各扣 40，加總 80 <= 100，應正常核准成功。
    let tr = DocumentService::create(
        &pool,
        &actor(),
        &tr_req(
            wh_from,
            wh_to,
            vec![
                tr_line(product, 40, shelf_to),
                tr_line(product, 40, shelf_to),
            ],
        ),
    )
    .await
    .expect("create TR");
    DocumentService::submit(&pool, &actor(), tr.document.id)
        .await
        .expect("submit TR");
    DocumentService::approve(&pool, &actor(), tr.document.id)
        .await
        .expect("加總未超賣，核准應成功");

    let out_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stock_ledger WHERE doc_id = $1 AND direction = 'transfer_out'",
    )
    .bind(tr.document.id)
    .fetch_one(&pool)
    .await
    .expect("count transfer_out rows");
    assert_eq!(out_count, 2, "兩行都應各自寫入一筆 transfer_out 流水");

    assert_eq!(
        snapshot_qty(&pool, wh_from, product).await,
        Decimal::from(20),
        "來源倉快照應正確反映兩行加總扣減：100-40-40=20"
    );
    assert_eq!(
        snapshot_qty(&pool, wh_to, product).await,
        Decimal::from(80),
        "目標倉快照應正確反映兩行加總增加：0+40+40=80"
    );
}
