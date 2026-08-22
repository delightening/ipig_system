//! SO 一段式多倉銷貨（migration 136）acceptance tests。
//!
//! 需求：SD 可在同一張銷貨單 SO 上銷不同倉庫來源的貨（不同商品分屬不同倉、或同商品跨倉湊數量）。
//! 設計：SO 核准即扣庫存（一段式，取代舊 SO→DO 兩段式）；每行倉庫取自該行儲位所屬倉
//! （documents.warehouse_id 對 SO 不再必填），ledger 逐行跟隨儲位 → 與 132/133 跨倉對帳不變式一致。
//!
//! 涵蓋：
//! - T1 同商品跨倉分批：一張 SO 兩行同品項、分屬兩倉，核准後各倉 ledger/快照/儲位庫存各自正確扣減。
//! - T2 超扣原子性：某行倉庫庫存不足 → 整張核准 rollback，無部分扣帳。
//! - T4 過帳硬失敗 / T5 DO 封鎖 / T6 無單價成本轉列（2026-07-21 裁定：內部銷貨不記金額）。

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

/// admin + 倉管：approve() 於 handler 層要求 WAREHOUSE_MANAGER；service 層測試沿用此 actor。
fn actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "test-sd@example.com".into(),
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
/// 本檔測的是 SO 多倉扣帳；SO 不在 SoD 管制範圍內，仍由 `actor()` 自核。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "approver-so-multi").await,
        email: "approver-so-multi@example.com".into(),
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

/// 無單價明細（內部銷貨常態，2026-07-21 裁定：SO 單價選填、不記收入）。
fn line_no_price(product: Uuid, qty: i64, shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: None,
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: Some(shelf),
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

fn line(product: Uuid, qty: i64, price: i64, shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: Some(Decimal::from(price)),
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: Some(shelf),
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

fn req(
    doc_type: DocType,
    warehouse_id: Option<Uuid>,
    lines: Vec<DocumentLineInput>,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type,
        warehouse_id,
        warehouse_from_id: None,
        warehouse_to_id: None,
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

/// 以 GRN 建 + 核准，把 `qty` 件庫存入到指定倉庫的指定儲位（寫 in ledger + 儲位庫存 + 快照）。
async fn seed_stock(pool: &PgPool, warehouse: Uuid, shelf: Uuid, product: Uuid, qty: i64) {
    let grn = DocumentService::create(
        pool,
        &actor(),
        &req(
            DocType::GRN,
            Some(warehouse),
            vec![line(product, qty, 10, shelf)],
        ),
    )
    .await
    .expect("create GRN");
    // 先提交才可核准
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

async fn shelf_qty(pool: &PgPool, shelf: Uuid, product: Uuid) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(SUM(on_hand_qty), 0) FROM storage_location_inventory WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(shelf)
    .bind(product)
    .fetch_one(pool)
    .await
    .expect("query shelf inventory")
}

#[tokio::test]
#[serial]
async fn so_same_product_across_two_warehouses_deducts_each_correctly() {
    let pool = setup_pool().await;
    let wh_a = seed_warehouse(&pool).await;
    let wh_b = seed_warehouse(&pool).await;
    let shelf_a = seed_shelf(&pool, wh_a).await;
    let shelf_b = seed_shelf(&pool, wh_b).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh_a, shelf_a, product, 100).await;
    seed_stock(&pool, wh_b, shelf_b, product, 100).await;

    // 一張 SO（表頭無倉庫）：同商品 60 出台北倉、40 出台中倉。
    let so = DocumentService::create(
        &pool,
        &actor(),
        &req(
            DocType::SO,
            None,
            vec![
                line_no_price(product, 60, shelf_a),
                line_no_price(product, 40, shelf_b),
            ],
        ),
    )
    .await
    .expect("create 跨倉 SO");

    // 每行倉庫應已從儲位反推回填。
    let line_whs: Vec<(Option<Uuid>, Uuid)> = sqlx::query_as(
        "SELECT warehouse_id, storage_location_id FROM document_lines WHERE document_id = $1 AND storage_location_id IS NOT NULL",
    )
    .bind(so.document.id)
    .fetch_all(&pool)
    .await
    .expect("query so lines");
    for (wh, shelf) in &line_whs {
        let expected = if *shelf == shelf_a { wh_a } else { wh_b };
        assert_eq!(*wh, Some(expected), "SO 明細倉庫應由儲位反推回填");
    }

    DocumentService::submit(&pool, &actor(), so.document.id)
        .await
        .expect("submit SO");
    DocumentService::approve(&pool, &actor(), so.document.id)
        .await
        .expect("核准跨倉 SO 應成功並逐行扣帳");

    // stock_ledger：兩筆 out，各倉各自數量。
    let out_rows: Vec<(Uuid, Decimal)> = sqlx::query_as(
        "SELECT warehouse_id, qty_base FROM stock_ledger WHERE doc_id = $1 AND direction = 'out' ORDER BY qty_base DESC",
    )
    .bind(so.document.id)
    .fetch_all(&pool)
    .await
    .expect("query ledger out rows");
    assert_eq!(out_rows.len(), 2, "跨倉 SO 應寫 2 筆 out 流水");
    assert_eq!(out_rows[0], (wh_a, Decimal::from(60)), "台北倉扣 60");
    assert_eq!(out_rows[1], (wh_b, Decimal::from(40)), "台中倉扣 40");

    // 快照：各倉各自遞減。
    assert_eq!(
        snapshot_qty(&pool, wh_a, product).await,
        Decimal::from(40),
        "wh_a 100-60"
    );
    assert_eq!(
        snapshot_qty(&pool, wh_b, product).await,
        Decimal::from(60),
        "wh_b 100-40"
    );

    // 儲位庫存：對應儲位各自遞減。
    assert_eq!(
        shelf_qty(&pool, shelf_a, product).await,
        Decimal::from(40),
        "shelf_a 100-60"
    );
    assert_eq!(
        shelf_qty(&pool, shelf_b, product).await,
        Decimal::from(60),
        "shelf_b 100-40"
    );

    // 快照平均成本只看入向流水：應維持進貨成本 10，不被銷貨售價 50 污染（#1004 follow-up L2）。
    let avg_cost: Option<Decimal> = sqlx::query_scalar(
        "SELECT avg_cost FROM inventory_snapshots WHERE warehouse_id = $1 AND product_id = $2",
    )
    .bind(wh_a)
    .bind(product)
    .fetch_one(&pool)
    .await
    .expect("query avg_cost");
    assert_eq!(
        avg_cost,
        Some(Decimal::from(10)),
        "avg_cost 應為進貨成本 10（僅入向流水）"
    );
}

#[tokio::test]
#[serial]
async fn so_rolls_back_entirely_when_one_warehouse_short() {
    let pool = setup_pool().await;
    let wh_a = seed_warehouse(&pool).await;
    let wh_b = seed_warehouse(&pool).await;
    let shelf_a = seed_shelf(&pool, wh_a).await;
    let shelf_b = seed_shelf(&pool, wh_b).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh_a, shelf_a, product, 100).await;
    seed_stock(&pool, wh_b, shelf_b, product, 30).await; // 不足以出 50

    let so = DocumentService::create(
        &pool,
        &actor(),
        &req(
            DocType::SO,
            None,
            vec![
                line_no_price(product, 60, shelf_a),
                line_no_price(product, 50, shelf_b),
            ],
        ),
    )
    .await
    .expect("create SO");
    DocumentService::submit(&pool, &actor(), so.document.id)
        .await
        .expect("submit SO");

    let err = DocumentService::approve(&pool, &actor(), so.document.id)
        .await
        .expect_err("某倉庫存不足時核准應失敗");
    assert!(
        err.to_string().contains("Insufficient") || err.to_string().contains("庫存"),
        "應回庫存不足錯誤，實際：{err}"
    );

    // 原子性：不得留下任何部分扣帳。
    let ledger: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(so.document.id)
        .fetch_one(&pool)
        .await
        .expect("count ledger");
    assert_eq!(ledger, 0, "核准失敗應完全 rollback，無任何 out 流水");
    assert_eq!(
        snapshot_qty(&pool, wh_a, product).await,
        Decimal::from(100),
        "wh_a 快照不變"
    );
    assert_eq!(
        snapshot_qty(&pool, wh_b, product).await,
        Decimal::from(30),
        "wh_b 快照不變"
    );
}

/// T4 過帳硬失敗（#1004 follow-up M1）：銷貨過帳失敗（會計科目缺漏）→ 整張核准 rollback。
/// 一段式銷貨「核准＝扣庫存＋記成本轉列」，不得出現「庫存已出、帳上沒動」的靜默漂移。
#[tokio::test]
#[serial]
async fn so_approval_fails_hard_when_posting_fails() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh, shelf, product, 100).await;

    let so = DocumentService::create(
        &pool,
        &actor(),
        &req(DocType::SO, None, vec![line_no_price(product, 10, shelf)]),
    )
    .await
    .expect("create SO");
    DocumentService::submit(&pool, &actor(), so.document.id)
        .await
        .expect("submit SO");

    // 弄壞銷貨成本科目（5200）→ post_sales 於 require_account_id 失敗（SO 只用 5200/1300）。
    sqlx::query("UPDATE chart_of_accounts SET code = '5200-BROKEN' WHERE code = '5200'")
        .execute(&pool)
        .await
        .expect("break 5200");

    let result = DocumentService::approve(&pool, &actor(), so.document.id).await;

    // 先復原科目再斷言：#[serial] 測試共用 DB，避免斷言失敗時汙染後續測試。
    sqlx::query("UPDATE chart_of_accounts SET code = '5200' WHERE code = '5200-BROKEN'")
        .execute(&pool)
        .await
        .expect("restore 5200");

    let err = result.expect_err("過帳失敗時銷貨核准應硬失敗");
    assert!(
        err.to_string().contains("5200"),
        "錯誤應指出缺漏科目，實際：{err}"
    );

    // 整張 rollback：無扣帳流水、快照不變、狀態仍 submitted 可重試。
    let ledger: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(so.document.id)
        .fetch_one(&pool)
        .await
        .expect("count ledger");
    assert_eq!(ledger, 0, "核准失敗不得留下扣帳流水");
    assert_eq!(
        snapshot_qty(&pool, wh, product).await,
        Decimal::from(100),
        "快照不變"
    );
    let status: String = sqlx::query_scalar("SELECT status::text FROM documents WHERE id = $1")
        .bind(so.document.id)
        .fetch_one(&pool)
        .await
        .expect("query status");
    assert_eq!(status, "submitted", "單據應維持送審狀態可重試");
}

/// T6（2026-07-21 裁定）：內部銷貨無單價——SO 單價選填，核准照樣逐行扣庫存，
/// 會計僅記成本轉列（借銷貨成本 5200 / 貸存貨 1300，用進貨平均成本），
/// 不認列營收（零金額收入分錄由 retain_postable_lines 剔除）。
#[tokio::test]
#[serial]
async fn so_without_price_approves_and_posts_cost_only() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    seed_stock(&pool, wh, shelf, product, 100).await; // 進貨單價 10 → 平均成本 10

    let so = DocumentService::create(
        &pool,
        &actor(),
        &req(DocType::SO, None, vec![line_no_price(product, 10, shelf)]),
    )
    .await
    .expect("無單價 SO 應可建立（內部銷貨常態）");
    DocumentService::submit(&pool, &actor(), so.document.id)
        .await
        .expect("submit SO");
    DocumentService::approve(&pool, &actor(), so.document.id)
        .await
        .expect("無單價 SO 核准應成功（扣庫存＋成本轉列）");

    assert_eq!(
        snapshot_qty(&pool, wh, product).await,
        Decimal::from(90),
        "庫存照扣 100-10"
    );

    // 不認列營收：4100 貸方總額必為 0（零金額分錄根本不寫入）。
    let revenue: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.credit_amount), 0) \
         FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.journal_entry_id \
         JOIN chart_of_accounts coa ON coa.id = jel.account_id \
         WHERE je.source_entity_id = $1 AND coa.code = '4100'",
    )
    .bind(so.document.id)
    .fetch_one(&pool)
    .await
    .expect("sum revenue credit");
    assert_eq!(revenue, Decimal::ZERO, "內部銷貨不認列收入");

    // 成本轉列：借 5200 = 10 件 × 平均成本 10 = 100。
    let cogs: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.debit_amount), 0) \
         FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.journal_entry_id \
         JOIN chart_of_accounts coa ON coa.id = jel.account_id \
         WHERE je.source_entity_id = $1 AND coa.code = '5200'",
    )
    .bind(so.document.id)
    .fetch_one(&pool)
    .await
    .expect("sum cogs debit");
    assert_eq!(cogs, Decimal::from(100), "成本轉列 = 10 × 平均成本 10");
}

// T5（#1004 follow-up L1）「DO 新建應被封鎖」的測試已於 R84-9（2026-07-23）移除：
// `DocType::DO` 連同 service 層的封鎖特例一併從 Rust 清除後，此處已無法建構該值，
// 測試本身無法編譯。它守護的不變式（防止 SO 一段式與 DO 並用對同筆銷貨雙扣庫存、
// 雙認營收）沒有消失，而是移到更早的一層——帶 `"doc_type":"DO"` 的請求在**反序列化
// 階段**就會被拒。對應測試改放 `models/document.rs` 的
// `deprecated_doc_types_are_rejected_at_deserialization`。
