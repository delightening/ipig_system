//! 整合測試：低庫存彙總改「全公司總量 vs 公司預設安全庫存」（一品項一筆 + 各倉分布）。
//!
//! 對應變更：`StockService::get_low_stock_totals`（services/stock/inventory.rs）。
//! 核心 regression：舊版逐倉庫比較會把「A倉60 + B倉60、safety=100」誤判成兩列低庫存
//! （單倉 60 < 100），但全公司總量 120 ≥ 100 其實健康。新版以總量判斷，不應出現。

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::StockService;
use erp_backend::SYSTEM_USER_ID;

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

async fn seed_warehouse(pool: &PgPool, suffix: &str, tag: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{tag}-{}", &suffix[..6]))
        .bind(format!("低庫存測試倉-{tag}"))
        .execute(pool)
        .await
        .expect("seed warehouse");
    id
}

/// 建一個有安全庫存的品項，回傳 product_id。
async fn seed_product(pool: &PgPool, suffix: &str, tag: &str, safety: i64) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO products (id, sku, name, base_uom, safety_stock) VALUES ($1, $2, $3, '個', $4)")
        .bind(id)
        .bind(format!("SKU-{tag}-{}", &suffix[..6]))
        .bind(format!("低庫存測試品-{tag}"))
        .bind(Decimal::new(safety, 0))
        .execute(pool)
        .await
        .expect("seed product");
    id
}

/// 為 ledger FK 建一張 GRN document，回傳 doc_id。
async fn seed_grn_doc(pool: &PgPool, suffix: &str, warehouse_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(id)
    .bind(format!("GRN-T-{}", &suffix[..10]))
    .bind(warehouse_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed GRN document");
    id
}

/// 對某倉某品項寫一筆 'in' 流水（qty_base = qty）。
async fn seed_ledger_in(pool: &PgPool, doc_id: Uuid, doc_no: &str, wh: Uuid, prod: Uuid, qty: i64) {
    sqlx::query(
        "INSERT INTO stock_ledger (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', $6)",
    )
    .bind(Uuid::new_v4())
    .bind(wh)
    .bind(prod)
    .bind(doc_id)
    .bind(doc_no)
    .bind(Decimal::new(qty, 0))
    .execute(pool)
    .await
    .expect("seed stock_ledger in");
}

#[tokio::test]
async fn low_stock_totals_use_company_wide_sum_not_per_warehouse() {
    let pool = setup_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();

    let wh_a = seed_warehouse(&pool, &suffix, "A").await;
    let wh_b = seed_warehouse(&pool, &suffix, "B").await;
    let doc_no = format!("GRN-T-{}", &suffix[..10]);
    let doc_id = seed_grn_doc(&pool, &suffix, wh_a).await;

    // 健康品項：A 60 + B 60 = 120 ≥ safety 100 → 不應出現（舊版逐倉會誤報兩列）
    let healthy = seed_product(&pool, &suffix, "HEALTHY", 100).await;
    seed_ledger_in(&pool, doc_id, &doc_no, wh_a, healthy, 60).await;
    seed_ledger_in(&pool, doc_id, &doc_no, wh_b, healthy, 60).await;

    // 低庫存品項：A 30 + B 20 = 50 < safety 100 → 一筆，total=50，breakdown 兩倉
    let low = seed_product(&pool, &suffix, "LOW", 100).await;
    seed_ledger_in(&pool, doc_id, &doc_no, wh_a, low, 30).await;
    seed_ledger_in(&pool, doc_id, &doc_no, wh_b, low, 20).await;

    // 缺貨品項：無任何流水 → total=0 < safety 5 → out_of_stock，breakdown 空
    let oos = seed_product(&pool, &suffix, "OOS", 5).await;

    let alerts = StockService::get_low_stock_totals(&pool)
        .await
        .expect("get_low_stock_totals 應成功");

    // 健康品項不應出現（核心 regression：全公司總量判斷，非逐倉）
    assert!(
        !alerts.iter().any(|a| a.product_id == healthy),
        "A倉60+B倉60=120 ≥ safety 100 的健康品項不應出現在低庫存清單"
    );

    // 低庫存品項：恰一筆，total=50，狀態 low，breakdown 兩倉合計 50
    let low_alerts: Vec<_> = alerts.iter().filter(|a| a.product_id == low).collect();
    assert_eq!(low_alerts.len(), 1, "低庫存品項應恰好一筆（非逐倉多列）");
    let la = low_alerts[0];
    assert_eq!(
        la.total_on_hand.normalize(),
        Decimal::from(50),
        "全公司總量應為 50"
    );
    assert_eq!(la.stock_status, "low");
    assert_eq!(la.warehouse_breakdown.len(), 2, "breakdown 應含 A、B 兩倉");
    let breakdown_sum: Decimal = la.warehouse_breakdown.iter().map(|w| w.qty_on_hand).sum();
    assert_eq!(
        breakdown_sum.normalize(),
        Decimal::from(50),
        "各倉分布合計應等於總量"
    );

    // 缺貨品項：total=0、out_of_stock、breakdown 空（qty=0 倉庫不列入）
    let oos_alerts: Vec<_> = alerts.iter().filter(|a| a.product_id == oos).collect();
    assert_eq!(oos_alerts.len(), 1, "缺貨品項應出現一筆");
    let oa = oos_alerts[0];
    assert_eq!(
        oa.total_on_hand.normalize(),
        Decimal::from(0),
        "缺貨總量為 0"
    );
    assert_eq!(oa.stock_status, "out_of_stock");
    assert!(
        oa.warehouse_breakdown.is_empty(),
        "缺貨品項各倉皆 0，breakdown 應為空"
    );
}
