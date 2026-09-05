//! Regression：沖銷一張採購入庫單（GRN）後，來源採購單（PO）必須能重開更正單。
//!
//! Bug（2026-08-27 實查）：`approve_reversal` 完全不碰來源 PO，於是沖銷後
//!   1. PO 的 `receipt_status` 仍停在 `complete` → 前端「採購入庫」按鈕
//!      （要求 pending/partial）消失，使用者開不出更正單；
//!   2. `ensure_no_over_receipt` 的 `received` 仍把已沖銷的原單算進去
//!      （原單沖銷後**仍是 `approved`**）→ 就算硬開，核准時被
//!      「入庫數量超過採購量」擋下。
//!
//! 結果是「打錯 → 沖銷 → 重開正確的」這條唯一的補救路徑走不完，PO 永久卡死。
//!
//! 修復：`approve_reversal` 對 GRN 回呼 `update_po_receipt_status`；該函式與
//! `ensure_no_over_receipt` 的 `received` 算式一併排除已被沖銷的 GRN。
//!
//! 涵蓋：
//! - T1 沖銷後 `receipt_status` 回退，且可重開部分量的更正單（→ partial）
//! - T2 沖銷後可重開**全量**更正單而不被超收守衛擋下（→ complete）
//! - T3 沖銷後前端「採購入庫」按鈕的實際路徑（`create_additional_grn`）開得出更正單，
//!   且入庫進度查詢（`get_po_receipt_status`）不再把已沖銷的量算進 `received_qty`
//!
//! ⚠️ T3 是 2026-09-01 補的：同一個 `received` 語意在 `grn.rs` 共有**四處**算式，
//! 第一版修法只補了 `ensure_no_over_receipt` 與 `update_po_receipt_status` 兩處。
//! 另兩處（`create_additional_grn` / `get_po_receipt_status`）未同步的後果是
//! 「按鈕出現了、按下去卻回 All items have been received」——**補救路徑照樣斷**，
//! 而 T1/T2 因為直接呼叫 `DocumentService::create` 建 GRN，完全繞過了那條路徑，測不出來。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{CreateDocumentRequest, DocStatus, DocType, DocumentLineInput};
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

/// 倉庫管理員：建單者兼沖銷發起人。
fn wm_actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "wm-grn-reopen@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 第二位倉管——R97-1 起 GRN 禁止自核，入庫需由非建單者核准。
async fn second_wm_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "wm2-grn-reopen").await,
        email: "wm2-grn-reopen@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm2".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 管理員（沖銷核准人）——刻意與發起人不同 id，符合 SoD。
async fn admin_actor(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, '沖銷核准管理員', 'x', true)",
    )
    .bind(id)
    .bind(format!("admin-grn-reopen-{}@example.com", &s[..8]))
    .execute(pool)
    .await
    .expect("seed admin user");
    ActorContext::User(CurrentUser {
        id,
        email: "admin-grn-reopen@example.com".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test-admin".into(),
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
        .bind("沖銷重開測試倉")
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
        .bind("沖銷重開測試品")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

/// PO 明細：採購尚未入庫，`DocType::PO` 不需儲位（`requires_shelf` 排除）。
fn po_line(product: Uuid, qty: i64) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: Some(Decimal::from(10)),
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: None,
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

/// GRN 明細：單價必填且 > 0（進貨成本是平均成本法的源頭）。
fn grn_line(product: Uuid, qty: i64, shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        storage_location_id: Some(shelf),
        ..po_line(product, qty)
    }
}

fn doc_req(
    doc_type: DocType,
    warehouse: Uuid,
    source_doc_id: Option<Uuid>,
    lines: Vec<DocumentLineInput>,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type,
        warehouse_id: Some(warehouse),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        source_doc_id,
        doc_date: chrono::Utc::now().date_naive(),
        remark: None,
        stocktake_scope: None,
        iacuc_no: None,
        protocol_id: None,
        lines,
    }
}

/// 建立並核准一張採購單，回傳其 id。核准後 `receipt_status` 應為 `pending`。
///
/// PO 不在 `SELF_APPROVAL_FORBIDDEN_DOC_TYPES` 內（只含 ADJ / GRN），故可自核。
async fn approved_po(pool: &PgPool, warehouse: Uuid, product: Uuid, qty: i64) -> Uuid {
    let doc = DocumentService::create(
        pool,
        &wm_actor(),
        &doc_req(DocType::PO, warehouse, None, vec![po_line(product, qty)]),
    )
    .await
    .expect("create PO");
    DocumentService::submit(pool, &wm_actor(), doc.document.id)
        .await
        .expect("submit PO");
    DocumentService::approve(pool, &wm_actor(), doc.document.id)
        .await
        .expect("approve PO");
    doc.document.id
}

/// 建立並核准一張掛在 `po_id` 底下的 GRN，回傳其 id。
async fn approved_grn_for_po(
    pool: &PgPool,
    warehouse: Uuid,
    shelf: Uuid,
    product: Uuid,
    po_id: Uuid,
    qty: i64,
) -> Uuid {
    let doc = DocumentService::create(
        pool,
        &wm_actor(),
        &doc_req(
            DocType::GRN,
            warehouse,
            Some(po_id),
            vec![grn_line(product, qty, shelf)],
        ),
    )
    .await
    .expect("create GRN");
    DocumentService::submit(pool, &wm_actor(), doc.document.id)
        .await
        .expect("submit GRN");
    // R97-1：GRN 的建立者不得自核，改由第二位倉管核准。
    let approver = second_wm_actor(pool).await;
    DocumentService::approve(pool, &approver, doc.document.id)
        .await
        .expect("approve GRN");
    doc.document.id
}

async fn receipt_status(pool: &PgPool, po_id: Uuid) -> String {
    sqlx::query_scalar::<_, Option<String>>("SELECT receipt_status FROM documents WHERE id = $1")
        .bind(po_id)
        .fetch_one(pool)
        .await
        .expect("query receipt_status")
        .unwrap_or_default()
}

/// 沖銷一張已核准的 GRN（發起 + ADMIN 核准）。
async fn reverse_grn(pool: &PgPool, grn_id: Uuid, admin: &ActorContext) {
    let reversal = DocumentService::create_reversal(pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    let approved = DocumentService::approve_reversal(pool, admin, reversal.document.id)
        .await
        .expect("核准沖銷單");
    assert_eq!(approved.document.status, DocStatus::Approved);
}

#[tokio::test]
#[serial]
async fn reversing_grn_rolls_back_po_receipt_status_and_allows_partial_reopen() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let po_id = approved_po(&pool, wh, product, 100).await;
    assert_eq!(
        receipt_status(&pool, po_id).await,
        "pending",
        "PO 核准後應標記為待入庫"
    );

    // 誤打全量入庫
    let grn_id = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;
    assert_eq!(
        receipt_status(&pool, po_id).await,
        "complete",
        "全量入庫後應為已入庫"
    );

    reverse_grn(&pool, grn_id, &admin).await;

    // 修復前這裡是 "complete"——PO 卡死，前端「採購入庫」按鈕消失。
    assert_eq!(
        receipt_status(&pool, po_id).await,
        "pending",
        "沖銷後入庫進度必須回退，否則使用者開不出更正單"
    );

    // 重開正確數量的更正單。修復前會被 ensure_no_over_receipt 擋下
    //（received 仍含已沖銷的 100，100 + 80 > 100）。
    approved_grn_for_po(&pool, wh, shelf, product, po_id, 80).await;
    assert_eq!(
        receipt_status(&pool, po_id).await,
        "partial",
        "更正單核准後應反映實收數量"
    );
}

#[tokio::test]
#[serial]
async fn reversing_grn_allows_full_quantity_reopen_without_over_receipt_error() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let po_id = approved_po(&pool, wh, product, 100).await;
    let grn_id = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;
    reverse_grn(&pool, grn_id, &admin).await;

    // 沖銷後重開同樣的全量：received 若未排除已沖銷的原單，會算成 200 > 100 而回
    // Conflict「入庫數量超過採購量」。這是本次修復的第二半。
    approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;
    assert_eq!(
        receipt_status(&pool, po_id).await,
        "complete",
        "全量重開後應回到已入庫"
    );
}

/// T3：走前端「採購入庫」按鈕的實際路徑。
///
/// `DocumentDetailPage.tsx` 的按鈕打 `POST /documents/{po}/create-grn`
/// → `DocumentService::create_additional_grn`，該函式自己算一次 `received` 決定
/// 「還剩多少可入庫」。這一處若未排除已沖銷的 GRN，remaining 會全為 0，
/// 直接回 `BusinessRule("All items have been received")`——按鈕雖然因 T1 而重新出現，
/// 使用者按下去仍舊開不出更正單。
#[tokio::test]
#[serial]
async fn reversing_grn_allows_reopen_through_create_additional_grn() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let po_id = approved_po(&pool, wh, product, 100).await;
    let grn_id = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;
    reverse_grn(&pool, grn_id, &admin).await;

    // 入庫進度查詢（GRN 表單靠它顯示 ordered / received / remaining）
    let status = DocumentService::get_po_receipt_status(&pool, po_id)
        .await
        .expect("查詢 PO 入庫進度");
    assert_eq!(status.status, "pending", "沖銷後整張 PO 應回到待入庫");
    let item = status
        .items
        .iter()
        .find(|i| i.product_id == product)
        .expect("入庫進度應含該品項");
    assert_eq!(
        item.received_qty,
        Decimal::ZERO,
        "已沖銷的量不得再算進 received_qty"
    );
    assert_eq!(
        item.remaining_qty,
        Decimal::from(100),
        "沖銷後剩餘可入庫量應回到全量"
    );

    // 前端按鈕的實際路徑：修復前這裡是 BusinessRule("All items have been received")
    let draft = DocumentService::create_additional_grn(&pool, po_id, SYSTEM_USER_ID)
        .await
        .expect("沖銷後必須能從 PO 開出更正入庫單");
    assert_eq!(draft.document.doc_type, DocType::GRN);
    assert_eq!(draft.document.source_doc_id, Some(po_id));
    assert_eq!(draft.lines.len(), 1, "更正單應只含該品項一列");
    assert_eq!(
        draft.lines[0].qty,
        Decimal::from(100),
        "預設帶入的數量應為沖銷後的剩餘全量"
    );
}
