//! R84-6：批號對帳分級 acceptance tests。
//!
//! 背景：`get_lot_movements` 原本只比對「本批號的推導量 vs 儲位實際量」，不一致就標成
//! 帳實不符。但 2026-05-20 (migration 069) 之前的異動只寫 `stock_ledger` 不寫
//! `storage_location_inventory`，之後的 R62-2 補帳與 PHANTOMFIX 幻影庫存清理雖然把
//! **品項總量**補平了，沖銷列卻沒有帶批號，差額因此掛在批號之外。實測 prod：195 個
//! 有批號的 lot 裡 157 個被標紅，其中 150 個是這種歷史歸屬差異，不是數量出錯——
//! 對帳功能 8 成在喊狼來了，紅字就失去意義。
//!
//! 修正：先看品項總量再判定嚴重度，分三級（`LotReconciliationStatus`）。
//!
//! 涵蓋：
//! - T1 批號本身帳實相符 → `Balanced`
//! - T2 批號不符，但品項總量相符（歷史 ledger-only 補帳的形狀）→ `AttributionOnly`
//! - T3 批號不符，品項總量也對不上 → `Unbalanced`

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{
    CreateDocumentRequest, DocType, DocumentLineInput, LotMovementsQuery, LotReconciliationStatus,
};
use erp_backend::services::{DocumentService, StockService};
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
        email: "test-r84-6@example.com".into(),
        // 授權實際靠 `admin`；原本的 "STUDY_DIRECTOR" 字串為裝飾性，
        // 該全域角色已移除，一併清掉避免誤導。
        roles: vec!["admin".into(), "WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// R97-1：GRN 禁止自核，前置入庫改由這位非建單者核准。
/// 本檔測的是批號對帳狀態判定，前置 GRN 由誰核准不影響待測行為。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "approver-r84-6").await,
        email: "approver-r84-6@example.com".into(),
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

/// 用 GRN 入庫建立一個乾淨的批號：ledger 與 storage_location_inventory 兩邊都寫。
/// 回傳該 GRN 的 (doc_id, doc_no)，供後續模擬歷史列時滿足 `stock_ledger.doc_id` 外鍵。
async fn seed_lot_via_grn(
    pool: &PgPool,
    warehouse: Uuid,
    shelf: Uuid,
    product: Uuid,
    batch_no: &str,
    qty: i64,
) -> (Uuid, String) {
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
            lines: vec![DocumentLineInput {
                product_id: product,
                qty: Decimal::from(qty),
                uom: "pcs".into(),
                unit_price: Some(Decimal::from(10)),
                batch_no: Some(batch_no.to_string()),
                expiry_date: None,
                remark: None,
                storage_location_id: Some(shelf),
                storage_location_from_id: None,
                storage_location_to_id: None,
            }],
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
    (grn.document.id, grn.document.doc_no)
}

/// 直接塞一列 `stock_ledger` 而**不動** `storage_location_inventory`——這是
/// migration 069 之前的記帳形狀，現行任何 code path 都產不出來，只能手動重建。
#[allow(clippy::too_many_arguments)]
async fn insert_legacy_ledger_row(
    pool: &PgPool,
    warehouse: Uuid,
    product: Uuid,
    doc_id: Uuid,
    doc_no: &str,
    direction: &str,
    qty: i64,
    batch_no: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO stock_ledger
            (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
             direction, qty_base, batch_no)
        VALUES ($1, $2, $3, NOW(), 'ADJ', $4, $5, $6::text::stock_direction, $7, $8)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(warehouse)
    .bind(product)
    .bind(doc_id)
    .bind(doc_no)
    .bind(direction)
    .bind(Decimal::from(qty))
    .bind(batch_no)
    .execute(pool)
    .await
    .expect("insert legacy ledger row");
}

fn query_for(product: Uuid, batch_no: &str) -> LotMovementsQuery {
    LotMovementsQuery {
        product_id: product,
        batch_no: batch_no.to_string(),
        expiry_date: None,
    }
}

#[tokio::test]
#[serial]
async fn lot_matching_shelf_inventory_is_balanced() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    seed_lot_via_grn(&pool, wh, shelf, product, "LOT-A", 100).await;

    let res = StockService::get_lot_movements(&pool, &query_for(product, "LOT-A"))
        .await
        .expect("get_lot_movements");
    let r = res.reconciliation;

    assert_eq!(r.remaining, Decimal::from(100));
    assert_eq!(r.derived_remaining, Decimal::from(100));
    assert!(r.balanced);
    assert_eq!(r.status, LotReconciliationStatus::Balanced);
}

/// 批號對不上但品項總量相符 → 只是歸屬問題，不該標成帳實不符。
/// 形狀取自 prod 的 CON-CYR-005/24G10：批號上多記了一筆調增，沖銷它的調減卻沒帶批號。
#[tokio::test]
#[serial]
async fn lot_mismatch_with_balanced_product_total_is_attribution_only() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    let (doc_id, doc_no) = seed_lot_via_grn(&pool, wh, shelf, product, "LOT-A", 100).await;

    // 批號上多出 +50（ledger only），另有 -50 掛在「無批號」桶子裡（ledger only）。
    // 批號層級：100 + 50 = 150 vs 實際 100 → 不平。
    // 品項層級：100 + 50 - 50 = 100 vs 實際 100 → 相符。
    insert_legacy_ledger_row(
        &pool,
        wh,
        product,
        doc_id,
        &doc_no,
        "adjust_in",
        50,
        Some("LOT-A"),
    )
    .await;
    insert_legacy_ledger_row(&pool, wh, product, doc_id, &doc_no, "adjust_out", 50, None).await;

    let res = StockService::get_lot_movements(&pool, &query_for(product, "LOT-A"))
        .await
        .expect("get_lot_movements");
    let r = res.reconciliation;

    assert_eq!(r.derived_remaining, Decimal::from(150));
    assert_eq!(r.remaining, Decimal::from(100));
    assert!(!r.balanced, "批號層級確實不平");
    assert_eq!(
        r.product_derived_total, r.product_remaining_total,
        "品項總量應相符"
    );
    assert_eq!(r.unattributed_adjust_net, Decimal::from(-50));
    assert_eq!(
        r.status,
        LotReconciliationStatus::AttributionOnly,
        "品項總量相符時不得標成帳實不符"
    );
}

/// 品項總量也對不上 → 這才是真的帳實不符，必須標紅。
/// 形狀取自 prod 的 CON-NEE-009/5N08B：儲位有量，但完全沒有對應的入庫或調整。
#[tokio::test]
#[serial]
async fn lot_mismatch_with_unbalanced_product_total_is_unbalanced() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    seed_lot_via_grn(&pool, wh, shelf, product, "LOT-A", 100).await;

    // 憑空多出的儲位庫存：有實體、無帳。
    sqlx::query(
        r#"
        INSERT INTO storage_location_inventory
            (id, storage_location_id, product_id, on_hand_qty, batch_no)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(shelf)
    .bind(product)
    .bind(Decimal::from(30))
    .bind("LOT-B")
    .execute(&pool)
    .await
    .expect("insert phantom shelf inventory");

    let res = StockService::get_lot_movements(&pool, &query_for(product, "LOT-B"))
        .await
        .expect("get_lot_movements");
    let r = res.reconciliation;

    assert_eq!(r.derived_remaining, Decimal::ZERO, "LOT-B 完全沒有帳");
    assert_eq!(r.remaining, Decimal::from(30));
    assert_ne!(
        r.product_derived_total, r.product_remaining_total,
        "品項總量應該也對不上"
    );
    assert_eq!(r.status, LotReconciliationStatus::Unbalanced);
}
