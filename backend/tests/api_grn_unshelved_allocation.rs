//! GRN 未上架 → 未分配庫存來源追溯 + 分配 FIFO 精確歸屬（migration 131）。
//!
//! 情境（對應 2026-07-16 倉儲 UX 決策，軟擋 GRN）：採購入庫核准時未落貨架 →
//! 倉庫總量上升但無儲位歸屬 → 產生「未分配庫存」。本測試驗證：
//!   1. get_unassigned_inventory 正確算出未分配量
//!   2. get_unassigned_sources 能追回是哪張 GRN 明細造成的
//!   3. assign_unassigned 分配時按 FIFO 攤回來源明細、寫 line_shelf_allocations 審計，
//!      且剩餘未上架量隨之遞減；分配滿後未分配歸零、來源清空
//!   4. 攤不回 GRN 來源的分配量（legacy）記 document_line_id = NULL

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{AssignUnassignedRequest, InventoryQuery, UnassignedSourceQuery};
use erp_backend::services::StockService;

#[path = "common/test_db.rs"]
mod test_db;

/// 分配走自核准 TR：測試以 System actor（actor_user_id() == SYSTEM_USER_ID）觸發，
/// 讓 documents.created_by / line_shelf_allocations.created_by 的 FK 指向 migration 033 SYSTEM user。
const TEST_ACTOR: ActorContext = ActorContext::System {
    reason: "test-assign",
};

/// migration 033 建立的固定 SYSTEM user；分配審計 created_by FK 指向它。
const SYSTEM_USER_ID: Uuid = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 種一個倉庫 + 產品 + 供應商 + 儲位，回傳 (warehouse, product, partner, shelf)。
async fn seed_base(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let wh = Uuid::new_v4();
    let prod = Uuid::new_v4();
    let partner = Uuid::new_v4();
    let shelf = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WH-{short}"))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, '支')")
        .bind(prod)
        .bind(format!("SKU-{short}"))
        .bind("測試採血管")
        .execute(pool)
        .await
        .expect("seed product");
    sqlx::query(
        "INSERT INTO partners (id, partner_type, code, name) VALUES ($1, 'supplier', $2, $3)",
    )
    .bind(partner)
    .bind(format!("SUP-{short}"))
    .bind("測試供應商")
    .execute(pool)
    .await
    .expect("seed partner");
    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(shelf)
    .bind(wh)
    .bind(format!("SL-{short}"))
    .execute(pool)
    .await
    .expect("seed shelf");

    (wh, prod, partner, shelf)
}

/// 種一張「已核准 GRN、明細未落貨架、已過帳」的來源；`created_offset_secs` 用來控制
/// FIFO 排序（越大 = 越舊）。回傳 GRN 明細 line id。
async fn seed_grn_source(
    pool: &PgPool,
    wh: Uuid,
    prod: Uuid,
    partner: Uuid,
    qty: Decimal,
    created_offset_secs: i64,
    batch_no: Option<&str>,
) -> Uuid {
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let grn = Uuid::new_v4();
    let line = Uuid::new_v4();
    let doc_no = format!("GRN-{short}");

    // 明確設定 created_at 以確定 FIFO 順序（NOW() - offset 秒）
    sqlx::query(
        "INSERT INTO documents \
         (id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date, created_by, created_at) \
         VALUES ($1, 'GRN', $2, 'approved', $3, $4, CURRENT_DATE, $5, NOW() - ($6 || ' seconds')::interval)",
    )
    .bind(grn)
    .bind(&doc_no)
    .bind(wh)
    .bind(partner)
    .bind(SYSTEM_USER_ID)
    .bind(created_offset_secs.to_string())
    .execute(pool)
    .await
    .expect("seed GRN doc");

    sqlx::query(
        "INSERT INTO document_lines \
         (id, document_id, line_no, product_id, qty, uom, batch_no, storage_location_id) \
         VALUES ($1, $2, 1, $3, $4, '支', $5, NULL)",
    )
    .bind(line)
    .bind(grn)
    .bind(prod)
    .bind(qty)
    .bind(batch_no)
    .execute(pool)
    .await
    .expect("seed GRN line (unshelved)");

    sqlx::query(
        "INSERT INTO stock_ledger \
         (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, line_id, direction, qty_base, batch_no, storage_location_id) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, $6, 'in', $7, $8, NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(wh)
    .bind(prod)
    .bind(grn)
    .bind(&doc_no)
    .bind(line)
    .bind(qty)
    .bind(batch_no)
    .execute(pool)
    .await
    .expect("seed ledger entry");

    // 分配走 TR → process_transfer::check_stock_available 讀 inventory_snapshots 快取；
    // 手動種 ledger 不會自動建快照，故在此同步 upsert 累加倉庫總量。
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (warehouse_id, product_id) \
         DO UPDATE SET on_hand_qty_base = inventory_snapshots.on_hand_qty_base + EXCLUDED.on_hand_qty_base, updated_at = NOW()",
    )
    .bind(wh)
    .bind(prod)
    .bind(qty)
    .execute(pool)
    .await
    .expect("seed inventory snapshot");

    line
}

async fn line_alloc_qty(pool: &PgPool, line: Uuid) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(SUM(qty),0) FROM line_shelf_allocations WHERE document_line_id = $1",
    )
    .bind(line)
    .fetch_one(pool)
    .await
    .expect("sum line allocations")
}

async fn unassigned_qty(pool: &PgPool, wh: Uuid, prod: Uuid) -> Decimal {
    let q = InventoryQuery {
        warehouse_id: Some(wh),
        product_id: Some(prod),
        ..Default::default()
    };
    let rows = StockService::get_unassigned_inventory(pool, &q)
        .await
        .expect("get unassigned");
    rows.into_iter()
        .find(|r| r.product_id == prod)
        .map(|r| r.qty_unassigned)
        .unwrap_or(Decimal::ZERO)
}

#[tokio::test]
async fn grn_unshelved_source_traceable_and_assign_fifo_writes_audit() {
    let pool = setup_pool().await;
    let (wh, prod, partner, shelf) = seed_base(&pool).await;
    // 兩張來源 GRN，old 較舊（offset 大）qty 30、new 較新 qty 70，合計未分配 100。
    let line_old = seed_grn_source(&pool, wh, prod, partner, Decimal::from(30), 7200, None).await;
    let line_new = seed_grn_source(&pool, wh, prod, partner, Decimal::from(70), 3600, None).await;

    // 1. 未分配量 = 100
    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::from(100));

    // 2. 來源可追溯，兩筆、依 FIFO（舊在前），供應商帶出
    let sources = StockService::get_unassigned_sources(
        &pool,
        &UnassignedSourceQuery {
            warehouse_id: wh,
            product_id: prod,
        },
    )
    .await
    .expect("get sources");
    assert_eq!(sources.len(), 2, "應有 2 筆來源 GRN 明細");
    assert_eq!(
        sources[0].document_id,
        {
            // 第一筆（FIFO 最舊）對應 line_old 之單據
            let doc: Uuid =
                sqlx::query_scalar("SELECT document_id FROM document_lines WHERE id = $1")
                    .bind(line_old)
                    .fetch_one(&pool)
                    .await
                    .expect("test db op");
            doc
        },
        "FIFO 排序：最舊來源應在最前"
    );
    assert_eq!(sources[0].remaining_unshelved, Decimal::from(30));
    assert_eq!(sources[0].partner_name.as_deref(), Some("測試供應商"));

    // 3. 分配 50 → 跨越邊界：舊來源(30)先被扣光，新來源再吃剩下的 20
    StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(50),
            batch_no: None,
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await
    .expect("assign 50");

    assert_eq!(
        line_alloc_qty(&pool, line_old).await,
        Decimal::from(30),
        "FIFO：舊來源應被完全扣光"
    );
    assert_eq!(
        line_alloc_qty(&pool, line_new).await,
        Decimal::from(20),
        "FIFO：新來源只吃到跨越邊界的 20"
    );
    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::from(50));

    // 舊來源不再出現於未分配來源；新來源剩 50
    let sources = StockService::get_unassigned_sources(
        &pool,
        &UnassignedSourceQuery {
            warehouse_id: wh,
            product_id: prod,
        },
    )
    .await
    .expect("get sources 2");
    assert_eq!(sources.len(), 1, "舊來源扣光後應只剩 1 筆");
    assert_eq!(sources[0].remaining_unshelved, Decimal::from(50));

    // 4. 分配剩餘 50 → 未分配歸零、來源清空、審計總量 = 入庫總量 100
    StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(50),
            batch_no: None,
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await
    .expect("assign 50 (remainder)");

    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::ZERO);
    let sources = StockService::get_unassigned_sources(
        &pool,
        &UnassignedSourceQuery {
            warehouse_id: wh,
            product_id: prod,
        },
    )
    .await
    .expect("get sources 3");
    assert!(sources.is_empty(), "分配滿後不應再有未分配來源");
    assert_eq!(
        line_alloc_qty(&pool, line_old).await + line_alloc_qty(&pool, line_new).await,
        Decimal::from(100),
        "審計總量應等於入庫量"
    );
}

/// 指定批號分配：只從相符批號的來源攤扣、上架庫存沿用該批號、超量報錯。
#[tokio::test]
async fn assign_with_specified_batch_matches_source_and_shelf() {
    let pool = setup_pool().await;
    let (wh, prod, partner, shelf) = seed_base(&pool).await;
    // 兩批：BATCH-A 40、BATCH-B 60，合計未分配 100。
    let line_a = seed_grn_source(
        &pool,
        wh,
        prod,
        partner,
        Decimal::from(40),
        7200,
        Some("BATCH-A"),
    )
    .await;
    let line_b = seed_grn_source(
        &pool,
        wh,
        prod,
        partner,
        Decimal::from(60),
        3600,
        Some("BATCH-B"),
    )
    .await;

    // 指定 BATCH-A 分配 30 → 只扣 line_a，上架庫存批號為 BATCH-A，line_b 不動
    StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(30),
            batch_no: Some("BATCH-A".to_string()),
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await
    .expect("assign batch A 30");

    assert_eq!(
        line_alloc_qty(&pool, line_a).await,
        Decimal::from(30),
        "只從 BATCH-A 來源攤扣"
    );
    assert_eq!(
        line_alloc_qty(&pool, line_b).await,
        Decimal::ZERO,
        "BATCH-B 來源不應被動到"
    );

    // 上架庫存必須落在 BATCH-A 批號列（批號忠實）
    let shelf_a_qty: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(on_hand_qty),0) FROM storage_location_inventory \
         WHERE storage_location_id = $1 AND product_id = $2 AND batch_no = 'BATCH-A'",
    )
    .bind(shelf)
    .bind(prod)
    .fetch_one(&pool)
    .await
    .expect("shelf BATCH-A qty");
    assert_eq!(
        shelf_a_qty,
        Decimal::from(30),
        "上架庫存應記在 BATCH-A 批號"
    );

    // 指定 BATCH-A 再分配 20（剩 10）→ 超過該批可用量，報錯
    let over = StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(20),
            batch_no: Some("BATCH-A".to_string()),
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await;
    assert!(
        over.is_err(),
        "指定批號超過該批可用量應報錯，不得 fallback 到其他批號"
    );
}

#[tokio::test]
async fn assign_beyond_grn_source_records_null_line_allocation() {
    // legacy 情境：未分配量存在但無對應 GRN 未上架明細（如 070 baseline backfill）→
    // 分配時攤不回任何明細，應記 document_line_id = NULL 的審計，仍成功扣減未分配。
    let pool = setup_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let wh = Uuid::new_v4();
    let prod = Uuid::new_v4();
    let shelf = Uuid::new_v4();
    let grn = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, '倉')")
        .bind(wh)
        .bind(format!("WH-{short}"))
        .execute(&pool)
        .await
        .expect("test db op");
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, '品', '支')")
        .bind(prod)
        .bind(format!("SKU-{short}"))
        .execute(&pool)
        .await
        .expect("test db op");
    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(shelf)
    .bind(wh)
    .bind(format!("SL-{short}"))
    .execute(&pool)
    .await
    .expect("test db op");
    // 一張 ADJ 入庫（非 GRN）造成未分配，v_grn_line_unshelved 不涵蓋 → 來源不明。
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'ADJ', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(grn)
    .bind(format!("ADJ-{short}"))
    .bind(wh)
    .bind(SYSTEM_USER_ID)
    .execute(&pool)
    .await
    .expect("test db op");
    sqlx::query(
        "INSERT INTO stock_ledger \
         (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base, storage_location_id) \
         VALUES ($1, $2, $3, NOW(), 'ADJ', $4, $5, 'adjust_in', 50, NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(wh)
    .bind(prod)
    .bind(grn)
    .bind(format!("ADJ-{short}"))
    .execute(&pool)
    .await
    .expect("test db op");
    // 分配走 TR → check_stock_available 讀 inventory_snapshots；種快照 = ADJ 入庫量 50。
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base) VALUES ($1, $2, 50)",
    )
    .bind(wh)
    .bind(prod)
    .execute(&pool)
    .await
    .expect("test db op");

    StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(50),
            batch_no: None,
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await
    .expect("assign legacy 50");

    let null_line_qty: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(qty),0) FROM line_shelf_allocations \
         WHERE document_line_id IS NULL AND storage_location_id = $1 AND product_id = $2",
    )
    .bind(shelf)
    .bind(prod)
    .fetch_one(&pool)
    .await
    .expect("test db op");
    assert_eq!(
        null_line_qty,
        Decimal::from(50),
        "來源不明應記 NULL line 審計"
    );
    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::ZERO);
}

/// 分配上架會開一張「同倉、已核准」TR 移轉單並記帳（transfer_out/in 淨零、目標儲位 sli +=），
/// 且不雙重計數（sli 剛好等於分配量，非兩倍）。
#[tokio::test]
async fn assign_creates_approved_tr_document_and_posts_ledger() {
    let pool = setup_pool().await;
    let (wh, prod, partner, shelf) = seed_base(&pool).await;
    let _line = seed_grn_source(
        &pool,
        wh,
        prod,
        partner,
        Decimal::from(30),
        3600,
        Some("B1"),
    )
    .await;
    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::from(30));

    StockService::assign_unassigned(
        &pool,
        &AssignUnassignedRequest {
            warehouse_id: wh,
            product_id: prod,
            storage_location_id: shelf,
            qty: Decimal::from(30),
            batch_no: None,
            expiry_date: None,
        },
        &TEST_ACTOR,
    )
    .await
    .expect("assign 30");

    // 1. 建立同倉、已核准 TR 移轉單
    let (tr_id, tr_status, wfrom, wto): (Uuid, String, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as(
            "SELECT id, status::text, warehouse_from_id, warehouse_to_id FROM documents \
             WHERE doc_type = 'TR' AND doc_no LIKE 'TR-%' AND warehouse_from_id = $1 \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(wh)
        .fetch_one(&pool)
        .await
        .expect("find TR doc");
    assert_eq!(tr_status, "approved", "TR 應為已核准");
    assert_eq!(wfrom, Some(wh));
    assert_eq!(wto, Some(wh), "同倉：from == to");

    // 2. TR 明細：from_loc = NULL（未分配池）、to_loc = 目標儲位、批號忠實
    let (l_qty, l_from, l_to, l_batch): (Decimal, Option<Uuid>, Option<Uuid>, Option<String>) =
        sqlx::query_as(
            "SELECT qty, storage_location_from_id, storage_location_to_id, batch_no \
             FROM document_lines WHERE document_id = $1 ORDER BY line_no LIMIT 1",
        )
        .bind(tr_id)
        .fetch_one(&pool)
        .await
        .expect("TR line");
    assert_eq!(l_qty, Decimal::from(30));
    assert_eq!(l_from, None, "來源為未分配池 → from_loc NULL");
    assert_eq!(l_to, Some(shelf));
    assert_eq!(l_batch.as_deref(), Some("B1"), "批號忠實上架");

    // 3. ledger 成對 transfer_out / transfer_in（同倉淨零）
    let (tout, tin): (Decimal, Decimal) = sqlx::query_as(
        "SELECT \
           COALESCE(SUM(qty_base) FILTER (WHERE direction = 'transfer_out'), 0), \
           COALESCE(SUM(qty_base) FILTER (WHERE direction = 'transfer_in'), 0) \
         FROM stock_ledger WHERE doc_id = $1",
    )
    .bind(tr_id)
    .fetch_one(&pool)
    .await
    .expect("ledger sums");
    assert_eq!(tout, Decimal::from(30), "transfer_out 30");
    assert_eq!(tin, Decimal::from(30), "transfer_in 30");

    // 4. 不雙重計數：目標儲位 sli 剛好 30（非 60）；未分配歸零
    let shelf_qty: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(on_hand_qty), 0) FROM storage_location_inventory \
         WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(shelf)
    .bind(prod)
    .fetch_one(&pool)
    .await
    .expect("shelf qty");
    assert_eq!(shelf_qty, Decimal::from(30), "sli 應剛好 30，無雙重計數");
    assert_eq!(unassigned_qty(&pool, wh, prod).await, Decimal::ZERO);
}
