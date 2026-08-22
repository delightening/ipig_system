//! 修復回歸（2026-07-22）：`GET /inventory/on-hand?product_id=`（未指定倉庫/儲位）
//! 舊版落入全倉概覽路徑並**忽略 product_id**——回傳全品項彙總，SO 儲位挑選器與
//! 產品詳情庫存快照把「整倉總量」誤標為單品存量、貨架分佈全空。
//!
//! 修復後契約（`get_on_hand_product_across_warehouses`）：
//! - 僅回傳該產品的列（其他產品不得出現）。
//! - 貨架級列（跨倉、來自 storage_location_inventory）＋各倉未分配餘量列
//!   （帳面總量 − 貨架加總，`storage_location_id` 為 NULL）。
//! - 列彼此不重疊：每倉加總 = 帳面總量；全上架倉不出現餘量列。

mod common;

use common::TestApp;
use erp_backend::models::InventoryQuery;
use erp_backend::services::StockService;
use erp_backend::SYSTEM_USER_ID;
use rust_decimal::Decimal;
use serial_test::serial;
use uuid::Uuid;

async fn seed_warehouse(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, '存量測試倉')")
        .bind(id)
        .bind(format!("WH-{}", &id.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert warehouse");
    id
}

async fn seed_shelf(app: &TestApp, warehouse_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO storage_locations (id, warehouse_id, code) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(warehouse_id)
        .bind(format!("SL-{}", &id.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert storage location");
    id
}

async fn seed_product(app: &TestApp, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, '支')")
        .bind(id)
        .bind(format!("SKU-{}", &id.to_string()[..8]))
        .bind(name)
        .execute(&app.db_pool)
        .await
        .expect("insert product");
    id
}

/// 帳面入庫（stock_ledger only，未上架）；先建 GRN document 滿足 doc_id FK。
async fn seed_ledger_in(app: &TestApp, warehouse: Uuid, product: Uuid, qty: i64) {
    let doc_id = Uuid::new_v4();
    let doc_no = format!("GRN-T-{}", &doc_id.to_string()[..8]);
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(doc_id)
    .bind(&doc_no)
    .bind(warehouse)
    .bind(SYSTEM_USER_ID)
    .execute(&app.db_pool)
    .await
    .expect("seed GRN document");
    sqlx::query(
        "INSERT INTO stock_ledger (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', $6)",
    )
    .bind(Uuid::new_v4())
    .bind(warehouse)
    .bind(product)
    .bind(doc_id)
    .bind(&doc_no)
    .bind(Decimal::from(qty))
    .execute(&app.db_pool)
    .await
    .expect("insert stock_ledger");
}

/// 貨架上架量（storage_location_inventory）
async fn seed_sli(app: &TestApp, shelf: Uuid, product: Uuid, qty: i64) {
    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(shelf)
    .bind(product)
    .bind(Decimal::from(qty))
    .execute(&app.db_pool)
    .await
    .expect("insert storage_location_inventory");
}

#[tokio::test]
#[serial]
async fn on_hand_product_id_returns_only_that_product_with_shelf_and_unassigned_rows() {
    let app = TestApp::spawn().await;
    let wh1 = seed_warehouse(&app).await;
    let wh2 = seed_warehouse(&app).await;
    let shelf = seed_shelf(&app, wh1).await;

    // 目標產品：wh1 帳面 80（貨架 50 + 未分配 30）、wh2 帳面 20（全未分配）
    let target = seed_product(&app, "跨倉存量目標品").await;
    seed_ledger_in(&app, wh1, target, 80).await;
    seed_sli(&app, shelf, target, 50).await;
    seed_ledger_in(&app, wh2, target, 20).await;

    // 干擾產品：不得出現在結果（舊 bug：整倉全品項彙總）
    let noise = seed_product(&app, "干擾品").await;
    seed_ledger_in(&app, wh1, noise, 999).await;

    let rows = StockService::get_on_hand(
        &app.db_pool,
        &InventoryQuery {
            product_id: Some(target),
            ..Default::default()
        },
    )
    .await
    .expect("get_on_hand product-scoped");

    assert!(
        rows.iter().all(|r| r.product_id == target),
        "結果只能含目標產品（舊 bug 會混入全品項），實際：{:?}",
        rows.iter().map(|r| &r.product_name).collect::<Vec<_>>()
    );
    assert_eq!(rows.len(), 3, "應為 貨架列 + wh1 餘量列 + wh2 餘量列");

    let shelf_row = rows
        .iter()
        .find(|r| r.storage_location_id == Some(shelf))
        .expect("應有貨架級列");
    assert_eq!(shelf_row.qty_on_hand, Decimal::from(50));
    assert_eq!(shelf_row.warehouse_id, wh1);

    let wh1_unassigned = rows
        .iter()
        .find(|r| r.warehouse_id == wh1 && r.storage_location_id.is_none())
        .expect("wh1 應有未分配餘量列");
    assert_eq!(
        wh1_unassigned.qty_on_hand,
        Decimal::from(30),
        "80 帳面 − 50 貨架"
    );

    let wh2_unassigned = rows
        .iter()
        .find(|r| r.warehouse_id == wh2 && r.storage_location_id.is_none())
        .expect("wh2 應有未分配餘量列");
    assert_eq!(wh2_unassigned.qty_on_hand, Decimal::from(20));
}

#[tokio::test]
#[serial]
async fn on_hand_product_id_fully_shelved_has_no_remainder_row() {
    let app = TestApp::spawn().await;
    let wh = seed_warehouse(&app).await;
    let shelf = seed_shelf(&app, wh).await;
    let product = seed_product(&app, "全上架品").await;
    seed_ledger_in(&app, wh, product, 40).await;
    seed_sli(&app, shelf, product, 40).await;

    let rows = StockService::get_on_hand(
        &app.db_pool,
        &InventoryQuery {
            product_id: Some(product),
            ..Default::default()
        },
    )
    .await
    .expect("get_on_hand product-scoped");

    assert_eq!(rows.len(), 1, "全上架：只有貨架列、無未分配餘量列");
    assert_eq!(rows[0].storage_location_id, Some(shelf));
    assert_eq!(rows[0].qty_on_hand, Decimal::from(40));
}
