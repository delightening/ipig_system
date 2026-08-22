//! Regression：低庫存告警 view（v_low_stock_alerts）→ LowStockAlert FromRow。
//!
//! Bug：`LowStockAlert`（models/stock.rs）有 mandatory `warehouse_name: String`，但
//! `v_low_stock_alerts` view 原本只有 warehouse_id / warehouse_code、無 warehouse_name。
//! `services/notification/alert.rs` 的 `SELECT * FROM v_low_stock_alerts → Vec<LowStockAlert>`
//! 只要 view 有低庫存資料就觸發 sqlx FromRow `no column found for name: warehouse_name`
//! → GET /api/v1/notifications/alerts/low-stock 與低庫存告警 scheduler 500。
//!
//! 修法：migration 091 為 view 補 `w.name AS warehouse_name`。
//! 本測試 seed 一筆低於 safety_stock 的庫存，直接以 `LowStockAlert` 反序列化 view，
//! 驗證不報 FromRow 錯且 warehouse_name 正確（修前 view 缺欄會失敗）。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::models::LowStockAlert;

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

#[tokio::test]
async fn low_stock_alerts_view_deserializes_with_warehouse_name() {
    let pool = setup_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();
    let wh = Uuid::new_v4();
    let prod = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("低庫存測試倉")
        .execute(&pool)
        .await
        .expect("seed warehouse");

    // safety_stock = 10、on_hand = 5 → 落入 v_low_stock_alerts（below_safety）
    sqlx::query("INSERT INTO products (id, sku, name, base_uom, safety_stock) VALUES ($1, $2, $3, '個', 10)")
        .bind(prod)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("低庫存測試品")
        .execute(&pool)
        .await
        .expect("seed product");

    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base, updated_at) \
         VALUES ($1, $2, 5, NOW())",
    )
    .bind(wh)
    .bind(prod)
    .execute(&pool)
    .await
    .expect("seed inventory_snapshot");

    // 修前：view 缺 warehouse_name → FromRow 失敗。修後：正常反序列化。
    let alerts: Vec<LowStockAlert> = sqlx::query_as(
        "SELECT * FROM v_low_stock_alerts WHERE product_id = $1",
    )
    .bind(prod)
    .fetch_all(&pool)
    .await
    .expect("v_low_stock_alerts 應可反序列化為 LowStockAlert（修前因缺 warehouse_name 會 FromRow 失敗）");

    assert_eq!(alerts.len(), 1, "應有 1 筆低庫存告警");
    let a = &alerts[0];
    assert_eq!(
        a.warehouse_name, "低庫存測試倉",
        "warehouse_name 應取自 w.name"
    );
    assert_eq!(
        a.product_sku,
        format!("SKU-{}", &suffix[..8]),
        "product_sku 應取自 p.sku"
    );
    assert_eq!(
        a.qty_on_hand,
        rust_decimal::Decimal::new(5, 0),
        "qty_on_hand 應取自 on_hand_qty_base"
    );
    assert_eq!(a.safety_stock, Some(rust_decimal::Decimal::new(10, 0)));
    assert_eq!(a.stock_status, "below_safety");
}
