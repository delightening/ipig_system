//! Acceptance（R84-2）：兩張庫存量表的非負 CHECK 約束（migration 137）。
//!
//! 驗證 0 與正值可寫入、負值（insert 與 update 兩路徑）被資料庫拒絕：
//!   - inventory_snapshots.on_hand_qty_base >= 0
//!   - storage_location_inventory.on_hand_qty >= 0

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

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

async fn seed_warehouse(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    id
}

async fn seed_product(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

async fn seed_storage_location(pool: &PgPool, wh: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(id)
    .bind(wh)
    .bind(format!("SL-{}", &suffix[..8]))
    .execute(pool)
    .await
    .expect("seed storage_location");
    id
}

async fn insert_snapshot(
    pool: &PgPool,
    wh: Uuid,
    prod: Uuid,
    qty: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base) \
         VALUES ($1, $2, $3)",
    )
    .bind(wh)
    .bind(prod)
    .bind(qty)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn insert_sli(pool: &PgPool, sl: Uuid, prod: Uuid, qty: Decimal) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(sl)
    .bind(prod)
    .bind(qty)
    .execute(pool)
    .await
    .map(|_| ())
}

#[tokio::test]
async fn inventory_snapshots_rejects_negative_on_hand() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;

    // 正值可寫入
    let p_pos = seed_product(&pool).await;
    insert_snapshot(&pool, wh, p_pos, Decimal::new(5, 0))
        .await
        .expect("正值 on_hand 應可寫入");

    // 0 可寫入
    let p_zero = seed_product(&pool).await;
    insert_snapshot(&pool, wh, p_zero, Decimal::ZERO)
        .await
        .expect("0 on_hand 應可寫入");

    // 負值 insert 被拒
    let p_neg = seed_product(&pool).await;
    let err = insert_snapshot(&pool, wh, p_neg, Decimal::new(-1, 0)).await;
    assert!(err.is_err(), "負值 insert 應被 CHECK 約束拒絕");

    // 正值 update 成負值被拒
    let update_err = sqlx::query(
        "UPDATE inventory_snapshots SET on_hand_qty_base = $1 \
         WHERE warehouse_id = $2 AND product_id = $3",
    )
    .bind(Decimal::new(-3, 0))
    .bind(wh)
    .bind(p_pos)
    .execute(&pool)
    .await;
    assert!(update_err.is_err(), "update 成負值應被 CHECK 約束拒絕");
}

#[tokio::test]
async fn storage_location_inventory_rejects_negative_on_hand() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let sl = seed_storage_location(&pool, wh).await;

    // 正值可寫入
    let p_pos = seed_product(&pool).await;
    insert_sli(&pool, sl, p_pos, Decimal::new(5, 0))
        .await
        .expect("正值 on_hand 應可寫入");

    // 0 可寫入
    let p_zero = seed_product(&pool).await;
    insert_sli(&pool, sl, p_zero, Decimal::ZERO)
        .await
        .expect("0 on_hand 應可寫入");

    // 負值 insert 被拒
    let p_neg = seed_product(&pool).await;
    let err = insert_sli(&pool, sl, p_neg, Decimal::new(-1, 0)).await;
    assert!(err.is_err(), "負值 insert 應被 CHECK 約束拒絕");

    // 正值 update 成負值被拒
    let update_err = sqlx::query(
        "UPDATE storage_location_inventory SET on_hand_qty = $1 \
         WHERE storage_location_id = $2 AND product_id = $3",
    )
    .bind(Decimal::new(-3, 0))
    .bind(sl)
    .bind(p_pos)
    .execute(&pool)
    .await;
    assert!(update_err.is_err(), "update 成負值應被 CHECK 約束拒絕");
}
