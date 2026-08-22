//! 整合測試：盤點(STK)核准 → 依各貨架實盤 vs 系統現存量差異，自動產生待審核 ADJ。
//!
//! 對應 feat：盤點改貨架層級 + workflow::approve 的 create_stocktake_reconciliation。

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
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

/// 倉庫管理員 actor（核准用）。借用 migration 033 的 SYSTEM user 作 FK 目標。
fn wm_actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "test-wm@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

fn dec(n: i64) -> Decimal {
    Decimal::new(n, 0)
}

/// 建一張 submitted STK：某貨架系統現存量 `system_qty`、盤點實填 `counted_qty`（單行、貨架層級）。
/// 回傳 (stk_id, storage_location_id)。
async fn seed_submitted_stocktake(
    pool: &PgPool,
    system_qty: Decimal,
    counted_qty: Decimal,
) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh_id = Uuid::new_v4();
    let prod_id = Uuid::new_v4();
    let loc_id = Uuid::new_v4();
    let stk_id = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh_id)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");

    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, $3)")
        .bind(prod_id)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");

    sqlx::query(
        "INSERT INTO storage_locations (id, warehouse_id, code, name, is_active, current_count) \
         VALUES ($1, $2, $3, $4, true, 1)",
    )
    .bind(loc_id)
    .bind(wh_id)
    .bind(format!("L-{}", &suffix[..8]))
    .bind("測試貨架")
    .execute(pool)
    .await
    .expect("seed storage_location");

    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(loc_id)
    .bind(prod_id)
    .bind(system_qty)
    .execute(pool)
    .await
    .expect("seed shelf inventory");

    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'STK', $2, 'submitted', $3, CURRENT_DATE, $4)",
    )
    .bind(stk_id)
    .bind(format!("STK-T-{}", &suffix[..10]))
    .bind(wh_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed stocktake document");

    // 盤點行：綁貨架、qty = 實盤數
    sqlx::query(
        "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom, storage_location_id) \
         VALUES ($1, $2, 1, $3, $4, 'pcs', $5)",
    )
    .bind(Uuid::new_v4())
    .bind(stk_id)
    .bind(prod_id)
    .bind(counted_qty)
    .bind(loc_id)
    .execute(pool)
    .await
    .expect("seed stocktake line");

    (stk_id, loc_id)
}

#[tokio::test]
async fn stocktake_approve_generates_adjustment_for_diff() {
    let pool = setup_pool().await;
    // 系統 10、實盤 7 → 差異 -3
    let (stk_id, loc_id) = seed_submitted_stocktake(&pool, dec(10), dec(7)).await;

    DocumentService::approve(&pool, &wm_actor(), stk_id)
        .await
        .expect("盤點核准應成功");

    let stk_status: String = sqlx::query_scalar("SELECT status::text FROM documents WHERE id = $1")
        .bind(stk_id)
        .fetch_one(&pool)
        .await
        .expect("query stk status");
    assert_eq!(stk_status, "approved");

    // 應產生一張 source_doc_id = STK 的 submitted ADJ（待倉管核准）
    let adj: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, status::text FROM documents WHERE doc_type = 'ADJ' AND source_doc_id = $1",
    )
    .bind(stk_id)
    .fetch_optional(&pool)
    .await
    .expect("query adj");
    let (adj_id, adj_status) = adj.expect("盤點有差異應自動產生調整單");
    assert_eq!(adj_status, "submitted", "差異調整單應為待審核狀態");

    // ADJ 行：qty = 實盤(7) − 系統(10) = -3，綁原儲位
    let (qty, sloc): (Decimal, Option<Uuid>) = sqlx::query_as(
        "SELECT qty, storage_location_id FROM document_lines WHERE document_id = $1",
    )
    .bind(adj_id)
    .fetch_one(&pool)
    .await
    .expect("query adj line");
    assert_eq!(qty, dec(-3), "差異量應為 -3");
    assert_eq!(sloc, Some(loc_id), "調整行應綁原盤點貨架");
}

#[tokio::test]
async fn stocktake_approve_no_diff_creates_no_adjustment() {
    let pool = setup_pool().await;
    // 系統 10、實盤 10 → 無差異
    let (stk_id, _loc_id) = seed_submitted_stocktake(&pool, dec(10), dec(10)).await;

    DocumentService::approve(&pool, &wm_actor(), stk_id)
        .await
        .expect("盤點核准應成功");

    let adj_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documents WHERE doc_type = 'ADJ' AND source_doc_id = $1",
    )
    .bind(stk_id)
    .fetch_one(&pool)
    .await
    .expect("count adj");
    assert_eq!(adj_count, 0, "盤點無差異不應產生調整單");
}
