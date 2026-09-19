//! R84-5 正式沖銷單（紅字沖銷）acceptance tests。
//!
//! 設計見 `docs/spec/modules/ERP流程.md` §6.3.1。核心不變式：
//! 沖銷鏡射的是原單**實際寫入**的東西，且**兩本帳都要動**——
//! `stock_ledger`（方向相反）、`storage_location_inventory`（增量維護、須顯式反向）、
//! `inventory_snapshots`（從 ledger 重算）。三者缺一即產生 storage drift，
//! 那正是 2026-05-20 migration 069 之前的老問題（R84-11 調查）。
//!
//! 涵蓋：
//! - T1 沖銷 GRN：ledger 反向列、SLI 退回、快照歸零，且會計傳票被鏡射
//! - T2 重複沖銷同一張單 → 被擋（應用層先擋，DB partial unique index 為最後防線）
//! - T3 沖銷單本身不可再被沖銷
//! - T4 SoD：發起人自己不能核准；不具 `erp.document.reverse_approve` 者不能核准
//! - T4b 具該權限但非管理員者**可以**核准（2026-08-26 起；見下方 `permitted_approver`）
//! - T5 庫存已被領用時沖銷入庫單 → 擋下（不可假裝把不存在的貨退回去）

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{CreateDocumentRequest, DocStatus, DocType, DocumentLineInput};
use erp_backend::services::DocumentService;
use erp_backend::{ActorContext, AppError, SYSTEM_USER_ID};

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

/// 倉庫管理員（沖銷發起人）。
fn wm_actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "wm-r84-5@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm".into(),
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
    .bind(format!("admin-r84-5-{}@example.com", &s[..8]))
    .execute(pool)
    .await
    .expect("seed admin user");
    ActorContext::User(CurrentUser {
        id,
        email: "admin-r84-5@example.com".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test-admin".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 具 `erp.document.reverse_approve`、但**不是**管理員的核准人（現實中即 DIRECTOR）。
///
/// 刻意不給任何管理員角色：若守衛又退回 `is_admin()`，這個 actor 就會被擋，
/// `permitted_non_admin_can_approve_reversal` 隨即轉紅。
async fn permitted_approver(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    let email = format!("director-r84-5-{}@example.com", &s[..8]);
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, '沖銷核准主管', 'x', true)",
    )
    .bind(id)
    .bind(&email)
    .execute(pool)
    .await
    .expect("seed director user");
    ActorContext::User(CurrentUser {
        id,
        email,
        roles: vec!["DIRECTOR".into()],
        permissions: vec!["erp.document.reverse_approve".into()],
        jti: "test-director".into(),
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
        .bind("沖銷測試倉")
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
        .bind("沖銷測試品")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

fn line(product: Uuid, qty: i64, shelf: Uuid) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: Some(Decimal::from(10)),
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: Some(shelf),
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

fn grn_req(warehouse: Uuid, lines: Vec<DocumentLineInput>) -> CreateDocumentRequest {
    CreateDocumentRequest {
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
        lines,
    }
}

/// 第二位倉管——R97-1 起 GRN 禁止自核，前置入庫需要一位非建單者來核准。
async fn second_wm_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "wm2-r84-5").await,
        email: "wm2-r84-5@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm2".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 建立並核准一張 GRN，回傳其 id。
async fn approved_grn(
    pool: &PgPool,
    warehouse: Uuid,
    shelf: Uuid,
    product: Uuid,
    qty: i64,
) -> Uuid {
    let doc = DocumentService::create(
        pool,
        &wm_actor(),
        &grn_req(warehouse, vec![line(product, qty, shelf)]),
    )
    .await
    .expect("create GRN");
    DocumentService::submit(pool, &wm_actor(), doc.document.id)
        .await
        .expect("submit GRN");
    // R97-1：GRN 的建立者不得自核，前置入庫改由第二位倉管核准。
    // 本檔測的是沖銷語意，誰核准這張前置 GRN 與待測行為無關。
    let approver = second_wm_actor(pool).await;
    DocumentService::approve(pool, &approver, doc.document.id)
        .await
        .expect("approve GRN");
    doc.document.id
}

async fn snapshot_qty(pool: &PgPool, warehouse: Uuid, product: Uuid) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(on_hand_qty_base, 0) FROM inventory_snapshots \
         WHERE warehouse_id = $1 AND product_id = $2",
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
        "SELECT COALESCE(SUM(on_hand_qty), 0) FROM storage_location_inventory \
         WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(shelf)
    .bind(product)
    .fetch_one(pool)
    .await
    .expect("query shelf inventory")
}

#[tokio::test]
#[serial]
async fn reversal_mirrors_ledger_shelf_and_snapshot() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 100).await;
    assert_eq!(snapshot_qty(&pool, wh, product).await, Decimal::from(100));
    assert_eq!(shelf_qty(&pool, shelf, product).await, Decimal::from(100));

    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    assert_eq!(reversal.document.status, DocStatus::Submitted);
    assert_eq!(reversal.document.reverses_doc_id, Some(grn_id));
    // 尚未核准 → 庫存不得變動
    assert_eq!(
        snapshot_qty(&pool, wh, product).await,
        Decimal::from(100),
        "沖銷單僅建立、未核准時不得影響庫存"
    );

    let approved = DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");
    assert_eq!(approved.document.status, DocStatus::Approved);

    // 三本帳都要回到 0——只動 ledger 不動 SLI 正是 storage drift 的成因
    assert_eq!(
        snapshot_qty(&pool, wh, product).await,
        Decimal::ZERO,
        "快照應歸零"
    );
    assert_eq!(
        shelf_qty(&pool, shelf, product).await,
        Decimal::ZERO,
        "儲位庫存應歸零（若只鏡射 ledger 這裡會殘留 100）"
    );

    let out_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM stock_ledger WHERE doc_id = $1 AND direction = 'out'",
    )
    .bind(approved.document.id)
    .fetch_one(&pool)
    .await
    .expect("query reversal ledger");
    assert_eq!(out_rows, 1, "沖銷單應寫入方向相反的 ledger 列");

    let entries: i64 =
        sqlx::query_scalar("SELECT count(*) FROM journal_entries WHERE source_entity_id = $1")
            .bind(approved.document.id)
            .fetch_one(&pool)
            .await
            .expect("query reversal journal");
    assert!(entries >= 1, "沖銷應鏡射會計傳票");
}

#[tokio::test]
#[serial]
async fn same_document_cannot_be_reversed_twice() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("第一張沖銷單");

    let err = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect_err("同一張單不得沖銷兩次");
    assert!(
        err.to_string().contains("已") && err.to_string().contains("沖銷"),
        "錯誤應說明已被沖銷過，實際：{err}"
    );
}

#[tokio::test]
#[serial]
async fn reversal_document_itself_cannot_be_reversed() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    let approved = DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");

    let err = DocumentService::create_reversal(&pool, &wm_actor(), approved.document.id)
        .await
        .expect_err("沖銷單不可再被沖銷");
    assert!(
        err.to_string().contains("不可再被沖銷"),
        "錯誤應說明沖銷單不可再沖銷，實際：{err}"
    );
}

#[tokio::test]
#[serial]
async fn reversal_enforces_separation_of_duties() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");

    // 不具 erp.document.reverse_approve 者不能核准。
    // 2026-08-26 前這裡擋的是「非管理員」，與 handler 的 require_permission! 不同源，
    // 導致唯一持有該權限的 DIRECTOR 反而過不了 service（見
    // tests/erp_approval_guard_consistency.rs）。現在兩邊都認同一個權限碼。
    let err = DocumentService::approve_reversal(&pool, &wm_actor(), reversal.document.id)
        .await
        .expect_err("倉庫管理員不得核准沖銷單");
    assert!(
        err.to_string().contains("erp.document.reverse_approve"),
        "錯誤應指名所需權限，實際：{err}"
    );

    // 發起人即使具 admin 角色也不能自己核准（SoD）
    let self_admin = ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID, // 與 wm_actor 同一 id ＝ 發起人本人
        email: "wm-r84-5@example.com".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test-self".into(),
        exp: 0,
        impersonated_by: None,
    });
    let err = DocumentService::approve_reversal(&pool, &self_admin, reversal.document.id)
        .await
        .expect_err("發起人不得自行核准");
    assert!(
        err.to_string().contains("職務分離"),
        "錯誤應說明職務分離，實際：{err}"
    );
}

/// T4b：具 `erp.document.reverse_approve` 但非管理員者，必須真的核准得了。
///
/// 這是 2026-08-26 修正的**正向**證明。上面那支只證明「沒權限的被擋」——
/// 若守衛退回 `if !user.is_admin()`，沒權限的**照樣**被擋，那支不會變紅。
/// 唯一能抓到退化的是這支：DIRECTOR 不具管理員角色，退回舊寫法就會 403。
#[tokio::test]
#[serial]
async fn permitted_non_admin_can_approve_reversal() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let director = permitted_approver(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");

    let approved = DocumentService::approve_reversal(&pool, &director, reversal.document.id)
        .await
        .expect("具 erp.document.reverse_approve 的非管理員應能核准沖銷單");
    assert_eq!(
        approved.document.status,
        DocStatus::Approved,
        "核准後狀態應為 Approved"
    );
}

/// 沖銷單的 `requires_manager_approval` + `wm_approved` 會落入一般 `admin_approve` 的前置條件，
/// 但那條路會呼叫 `process_document`——用「當下庫存狀態」**重跑業務邏輯**而非鏡射原單，
/// 且不會反向扣減儲位庫存。必須擋下，否則會算出與原單不同的結果並製造 storage drift。
#[tokio::test]
#[serial]
async fn reversal_cannot_go_through_normal_admin_approve() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");

    let err = DocumentService::admin_approve(&pool, &admin, reversal.document.id)
        .await
        .expect_err("沖銷單不得走一般最終核准");
    assert!(
        err.to_string().contains("沖銷核准流程"),
        "錯誤應導引至沖銷核准流程，實際：{err}"
    );

    // 確認沒有被誤走一般核准而寫入任何庫存流水
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(reversal.document.id)
        .fetch_one(&pool)
        .await
        .expect("query ledger rows");
    assert_eq!(rows, 0, "被擋下的核准不得留下任何 ledger 列");
}

/// 沖銷單也不得走**一般核准**（`/documents/{id}/approve`）。
///
/// 這條比 admin_approve 更危險：沖銷單的 `manager_approval_status` 是 `wm_approved`，
/// 而 `approve` 的 `needs_admin` 判定要求 `pending`——對沖銷單為 false，於是直接落到
/// 一般核准分支呼叫 `process_document`（用當下庫存重跑業務邏輯，GRN 沖銷會再寫一筆 `in`），
/// 且該路徑只要 WAREHOUSE_MANAGER 即可執行，等同繞過 ADMIN 最終核准（SoD）。
#[tokio::test]
#[serial]
async fn reversal_cannot_go_through_normal_approve() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 10).await;
    let before = snapshot_qty(&pool, wh, product).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");

    // 發起人（倉管）直接呼叫一般核准 —— 必須被擋
    let err = DocumentService::approve(&pool, &wm_actor(), reversal.document.id)
        .await
        .expect_err("沖銷單不得走一般核准");
    assert!(
        err.to_string().contains("沖銷核准流程"),
        "錯誤應導引至沖銷核准流程，實際：{err}"
    );

    // 不得留下任何庫存副作用
    assert_eq!(
        snapshot_qty(&pool, wh, product).await,
        before,
        "被擋下的核准不得改動庫存"
    );
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(reversal.document.id)
        .fetch_one(&pool)
        .await
        .expect("query ledger rows");
    assert_eq!(rows, 0, "被擋下的核准不得寫入任何 ledger 列");
}

#[tokio::test]
#[serial]
async fn reversal_blocked_when_goods_already_consumed() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let grn_id = approved_grn(&pool, wh, shelf, product, 50).await;

    // 貨已被領走（直接扣儲位庫存模擬後續出庫）
    sqlx::query(
        "UPDATE storage_location_inventory SET on_hand_qty = 0 \
         WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(shelf)
    .bind(product)
    .execute(&pool)
    .await
    .expect("模擬領用");

    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    let err = DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect_err("貨已不在，沖銷應被擋下");
    // 斷言改為比對錯誤變體與其欄位，不再比對訊息字串（2026-09-10）。
    //
    // 原本斷言 `err.to_string().contains("儲位庫存不足")`。那五個字在 `26cead0`
    //（倉庫政策旗標與領用卡關訊息）被改掉了——新訊息指出「這多半是帳面與實體不符、
    // 不是缺貨」並給出下一步（先開盤點單校正），語意比舊的準確。
    // 所以該改的是斷言，不是訊息。
    //
    // 但不是換一組新字串繼續比對：**那則訊息是寫給現場看的，本來就會被改文案**，
    // 用字串釘住它，等於每次改善措辭都要回頭修測試，而且哪天有人把它改壞了
    // （例如刪掉引導語）測試照樣綠——它驗的是「有沒有這幾個字」不是「語意對不對」。
    //
    // 改用 `AppError::InsufficientStock` 這個結構化變體，順帶驗兩個數字：
    // 貨已被領光所以帳面是 0，而沖銷要退回的是原入庫的 50。這比原斷言強——
    // 原本只要訊息裡有那五個字就過，數量錯了也看不出來。
    match err {
        AppError::InsufficientStock {
            on_hand, required, ..
        } => {
            assert_eq!(on_hand, Decimal::ZERO, "貨已被領光，儲位帳面應為 0");
            assert_eq!(
                required,
                Decimal::from(50),
                "沖銷要退回的是原入庫量 50，不是別的數字"
            );
        }
        other => panic!("應為 InsufficientStock（庫存不足擋下沖銷），實際：{other}"),
    }

    // 整筆 rollback：沖銷單仍為未核准，且沒有留下半套的 ledger 列
    let status: String = sqlx::query_scalar("SELECT status::text FROM documents WHERE id = $1")
        .bind(reversal.document.id)
        .fetch_one(&pool)
        .await
        .expect("query reversal status");
    assert_eq!(status, "submitted", "失敗後沖銷單不應被標記為已核准");

    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(reversal.document.id)
        .fetch_one(&pool)
        .await
        .expect("query ledger rows");
    assert_eq!(rows, 0, "失敗應整筆 rollback，不得留下部分鏡射的 ledger 列");
}
