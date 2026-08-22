//! Regression：儲位庫存查詢回傳 product_spec（GET /storage-locations/{id}/inventory）。
//!
//! Bug：PR #347 為 `StorageLocationInventoryItem` 加 `product_spec` 欄位、補了
//! `services/warehouse.rs` 的批次查詢，但漏掉 `services/storage_location.rs` 的
//! 7 個查詢 → SELECT 不含 `product_spec`，sqlx `FromRow` 報
//! "no column found for name: product_spec" → 儲位庫存頁 500。
//! （這台 prod 一直跑舊映像、rebuild 後才浮現。）
//!
//! 修復：7 個查詢補 `p.spec AS product_spec`。本測試覆蓋主檢視 `get_inventory`。

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::StorageLocationService;

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

/// get_inventory 回傳的項目須含 product_spec（取自 products.spec），不得 500。
#[tokio::test]
async fn get_inventory_returns_product_spec() {
    let pool = setup_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();
    let wh = Uuid::new_v4();
    let sl = Uuid::new_v4();
    let prod = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試倉")
        .execute(&pool)
        .await
        .expect("seed warehouse");

    sqlx::query("INSERT INTO products (id, sku, name, spec) VALUES ($1, $2, $3, $4)")
        .bind(prod)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("測試品項")
        .bind("180 cm")
        .execute(&pool)
        .await
        .expect("seed product");

    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(sl)
    .bind(wh)
    .bind(format!("SL-{}", &suffix[..8]))
    .execute(&pool)
    .await
    .expect("seed storage_location");

    sqlx::query(
        "INSERT INTO storage_location_inventory \
         (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(sl)
    .bind(prod)
    .bind(Decimal::new(5, 0))
    .execute(&pool)
    .await
    .expect("seed storage_location_inventory");

    // 修復前：FromRow 報 "no column found for name: product_spec" → Err
    let items = StorageLocationService::get_inventory(&pool, sl)
        .await
        .expect("get_inventory 不應 500（修復前因缺 product_spec 欄會 FromRow 失敗）");

    assert_eq!(items.len(), 1, "應回傳 1 筆庫存");
    assert_eq!(
        items[0].product_spec.as_deref(),
        Some("180 cm"),
        "product_spec 應取自 products.spec"
    );
}
