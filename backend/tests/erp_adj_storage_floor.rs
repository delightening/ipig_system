//! Regression：ADJ 出庫調整不得把單一儲位庫存扣成負數（code-review H5）。
//!
//! Bug：process_adjustment 對 ADJ 行不論增減一律呼叫
//! upsert_storage_location_inventory(line.qty)，傳入原始帶號 qty。ADJ 出庫
//! 時 line.qty < 0，upsert 的 `on_hand_qty + EXCLUDED.on_hand_qty` 等於加負值，
//! 且該 upsert 分支無下限檢查 → 單一儲位可被扣成負數（warehouse-level
//! check_stock_available 只守倉庫總量，擋不住單一儲位 drift）。
//!
//! 修復：ADJ 出庫改走 decrement_storage_location_inventory（含 on_hand_qty >= qty
//! 下限檢查），出庫超過該儲位庫存時回 BusinessRule 拒絕。

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

/// R97-1：ADJ 禁止自核，而本檔的 fixture 一律以 `SYSTEM_USER_ID` 當建單者
/// （`seed_submitted_adj` 直接寫 DB）。核准改用這位第二倉管。
///
/// ⚠️ 這不只是為了讓測試變綠：`adj_out_exceeding_storage_location_is_rejected`
/// 只斷言 `is_err()`，若沿用建單者核准，它會因為 SoD 而失敗、看起來仍是綠的，
/// 但已經不再測「儲位不足要擋下」這件事。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "wm2-adj-floor").await,
        email: "test-wm2@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm2".into(),
        exp: 0,
        impersonated_by: None,
    })
}

struct AdjFixture {
    doc_id: Uuid,
    storage_location_id: Uuid,
    product_id: Uuid,
}

/// 種一張「已提交」ADJ 出庫單。
/// warehouse 總量 = `wh_total`（過 check_stock_available），
/// 該儲位庫存 = `sl_on_hand`，調整行 qty = `adj_qty`（負值=出庫）。
async fn seed_submitted_adj(
    pool: &PgPool,
    wh_total: Decimal,
    sl_on_hand: Decimal,
    adj_qty: Decimal,
) -> AdjFixture {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh_id = Uuid::new_v4();
    let prod_id = Uuid::new_v4();
    let sl_id = Uuid::new_v4();
    let doc_id = Uuid::new_v4();

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
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(sl_id)
    .bind(wh_id)
    .bind(format!("SL-{}", &suffix[..8]))
    .execute(pool)
    .await
    .expect("seed storage_location");

    // warehouse-level 快照（check_stock_available 讀此值）
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base) \
         VALUES ($1, $2, $3)",
    )
    .bind(wh_id)
    .bind(prod_id)
    .bind(wh_total)
    .execute(pool)
    .await
    .expect("seed inventory_snapshot");

    // 快照的 ledger 來源（GRN 入庫 = wh_total）。ADJ 核准後 update_inventory_snapshot
    // 會以 SUM(stock_ledger) 重算快照；若無此入庫來源，重算只會看到 adjust_out 而算成
    // 負值，觸發 migration 137 的非負約束（且 prod 的倉庫存量本就有 GRN ledger 來源，
    // 直接 seed 快照卻不 seed 來源是不真實的 fixture）。
    let grn_doc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(grn_doc_id)
    .bind(format!("GRN-T-{}", &suffix[..10]))
    .bind(wh_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed GRN document (ledger 來源)");
    sqlx::query(
        "INSERT INTO stock_ledger \
         (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', $6)",
    )
    .bind(Uuid::new_v4())
    .bind(wh_id)
    .bind(prod_id)
    .bind(grn_doc_id)
    .bind(format!("GRN-T-{}", &suffix[..10]))
    .bind(wh_total)
    .execute(pool)
    .await
    .expect("seed GRN in-ledger（快照來源）");

    // 該儲位庫存（少於倉庫總量，凸顯儲位 drift）
    sqlx::query(
        "INSERT INTO storage_location_inventory \
         (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(sl_id)
    .bind(prod_id)
    .bind(sl_on_hand)
    .execute(pool)
    .await
    .expect("seed storage_location_inventory");

    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'ADJ', $2, 'submitted', $3, CURRENT_DATE, $4)",
    )
    .bind(doc_id)
    .bind(format!("ADJ-T-{}", &suffix[..10]))
    .bind(wh_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed document");

    sqlx::query(
        "INSERT INTO document_lines \
         (id, document_id, line_no, product_id, qty, uom, unit_price, storage_location_id) \
         VALUES ($1, $2, 1, $3, $4, 'pcs', NULL, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(prod_id)
    .bind(adj_qty)
    .bind(sl_id)
    .execute(pool)
    .await
    .expect("seed document line");

    AdjFixture {
        doc_id,
        storage_location_id: sl_id,
        product_id: prod_id,
    }
}

async fn sl_on_hand(pool: &PgPool, sl_id: Uuid, prod_id: Uuid) -> Decimal {
    sqlx::query_scalar(
        "SELECT on_hand_qty FROM storage_location_inventory \
         WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(sl_id)
    .bind(prod_id)
    .fetch_one(pool)
    .await
    .expect("query storage_location_inventory")
}

// ── H5：倉庫足量但儲位不足，ADJ 出庫應被拒絕（不得扣成負數） ──
#[tokio::test]
async fn adj_out_exceeding_storage_location_is_rejected() {
    let pool = setup_pool().await;
    // 倉庫總量 10（check 通過），但此儲位只有 3，出庫 5
    let fx = seed_submitted_adj(
        &pool,
        Decimal::new(10, 0),
        Decimal::new(3, 0),
        Decimal::new(-5, 0),
    )
    .await;

    let approver = approver_actor(&pool).await;
    let result = DocumentService::approve(&pool, &approver, fx.doc_id).await;
    let err = result.expect_err("儲位只有 3、出庫 5 應被拒絕（修復前會 upsert 成負數而成功核准）");
    // 明確排除「錯在 SoD」——否則本測試會因為錯誤的理由而通過（R97-1）。
    assert!(
        !err.to_string().contains("職務分離"),
        "應因儲位庫存不足被拒，而不是職務分離，實際：{err}"
    );

    // tx 回滾 → 儲位庫存維持 3，未被扣成負數
    assert_eq!(
        sl_on_hand(&pool, fx.storage_location_id, fx.product_id).await,
        Decimal::new(3, 0),
        "被拒絕後儲位庫存應維持原值，不得變負"
    );

    let status: String = sqlx::query_scalar("SELECT status::text FROM documents WHERE id = $1")
        .bind(fx.doc_id)
        .fetch_one(&pool)
        .await
        .expect("query document status");
    assert_eq!(status, "submitted", "核准失敗後單據狀態不應變為 approved");
}

// ── 對照：儲位庫存足夠時 ADJ 出庫正常扣減 ──
#[tokio::test]
async fn adj_out_within_storage_location_succeeds() {
    let pool = setup_pool().await;
    // 倉庫總量 10、儲位 3、出庫 2 → 應成功，儲位剩 1
    let fx = seed_submitted_adj(
        &pool,
        Decimal::new(10, 0),
        Decimal::new(3, 0),
        Decimal::new(-2, 0),
    )
    .await;

    let approver = approver_actor(&pool).await;
    DocumentService::approve(&pool, &approver, fx.doc_id)
        .await
        .expect("儲位足量的 ADJ 出庫應成功");

    assert_eq!(
        sl_on_hand(&pool, fx.storage_location_id, fx.product_id).await,
        Decimal::new(1, 0),
        "出庫 2 後儲位庫存應為 1"
    );
}
