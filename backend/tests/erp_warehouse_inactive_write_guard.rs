//! Regression：倉庫停用後仍可繼續往其寫入庫存（隱形庫存的另一半，#33）。
//!
//! Bug：#31 只堵住「停用當下已存在的結存」；`storage_location_inventory` 的 5 條寫入
//! 路徑（`upsert_storage_location_inventory` / `decrement_storage_location_inventory` /
//! `StorageLocationService::update_inventory_item` / `create_inventory_item` 的
//! INSERT、UPDATE 兩分支）沒有任何一處檢查 `warehouses.is_active`。讀取端
//! （`list_with_shelves` / `get_report_data`）全部過濾停用倉庫，寫入端完全不看
//! ——兩者相加即隱形庫存：新的 GRN/TR 核准、或倉庫停用與入庫交易並發交錯，都能
//! 讓已停用倉庫繼續累積帳上有、畫面上找不到的庫存。
//!
//! 修復：`WarehouseService::ensure_active_for_location_tx` 只守「增加」路徑
//! （decrement 不受限，讓已停用倉庫底下的殘留庫存仍可被領用/轉出清空），
//! 以 `FOR SHARE OF w` 鎖倉庫列，與 `update_tx` / `delete_tx` 停用時取的
//! `FOR UPDATE` 互斥，連併發交錯的競態窗口一併關掉。

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::ROLE_SYSTEM_ADMIN;
use erp_backend::middleware::CurrentUser;
use erp_backend::models::{
    CreateStorageLocationInventoryItemRequest, UpdateStorageLocationInventoryItemRequest,
};
use erp_backend::services::{DocumentService, StorageLocationService};
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

/// migration 033 的 SYSTEM user 當 actor，documents/audit 的 FK 都指向它，
/// 不必額外種 user row（與 `erp_doc_no_concurrency.rs` / `erp_adj_storage_floor.rs` 同作法）。
fn actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "inactive-wh-guard@test.local".into(),
        roles: vec![ROLE_SYSTEM_ADMIN.into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// R97-1：GRN / ADJ 禁止自核，而本檔的 fixture 以 `SYSTEM_USER_ID` 當建單者。
///
/// ⚠️ 核准人必須換人，否則 `grn_into_inactive_warehouse_is_rejected` 會因為
/// 職務分離而被擋——那與本檔要測的「停用倉庫閘門」是兩回事，
/// `assert_rejected_as_inactive` 也會因為錯誤型別不是 BusinessRule 而 panic。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "approver-inactive-wh").await,
        // 用 example.com 而非上方 actor() 的 test.local：`scripts/pii-scan.mjs` 的
        // 網域允許清單只含 example.com/org/net、test.com、localhost，
        // test.local 會被 pre-commit PII 掃描擋下。
        email: "inactive-wh-approver@example.com".into(),
        roles: vec![ROLE_SYSTEM_ADMIN.into()],
        permissions: vec![],
        jti: "test-approver".into(),
        exp: 0,
        impersonated_by: None,
    })
}

struct Fixture {
    warehouse_id: Uuid,
    storage_location_id: Uuid,
    product_id: Uuid,
}

/// 種一個倉庫（`active` 控制啟停）+ 一個儲位 + 一個品項。
async fn seed_base(pool: &PgPool, active: bool) -> Fixture {
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let wh = Uuid::new_v4();
    let sl = Uuid::new_v4();
    let prod = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name, is_active) VALUES ($1, $2, $3, $4)")
        .bind(wh)
        .bind(format!("WHIG-{short}"))
        .bind("停用寫入守衛測試倉")
        .bind(active)
        .execute(pool)
        .await
        .expect("seed warehouse");

    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, 'EA')")
        .bind(prod)
        .bind(format!("SKUIG-{short}"))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");

    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, name, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, '儲物架A', 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(sl)
    .bind(wh)
    .bind(format!("SLIG-{short}"))
    .execute(pool)
    .await
    .expect("seed storage_location");

    Fixture {
        warehouse_id: wh,
        storage_location_id: sl,
        product_id: prod,
    }
}

/// 種一張「已提交」GRN 入庫單，明細指定儲位。
async fn seed_submitted_grn(pool: &PgPool, fx: &Fixture, qty: Decimal) -> Uuid {
    let suffix = Uuid::new_v4().simple().to_string();
    let doc_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'submitted', $3, CURRENT_DATE, $4)",
    )
    .bind(doc_id)
    .bind(format!("GRN-T-{}", &suffix[..10]))
    .bind(fx.warehouse_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed GRN document");

    sqlx::query(
        "INSERT INTO document_lines \
         (id, document_id, line_no, product_id, qty, uom, unit_price, storage_location_id) \
         VALUES ($1, $2, 1, $3, $4, 'EA', NULL, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(fx.product_id)
    .bind(qty)
    .bind(fx.storage_location_id)
    .execute(pool)
    .await
    .expect("seed GRN line");

    doc_id
}

async fn sli_qty(pool: &PgPool, sl_id: Uuid, prod_id: Uuid) -> Option<Decimal> {
    sqlx::query_scalar(
        "SELECT on_hand_qty FROM storage_location_inventory \
         WHERE storage_location_id = $1 AND product_id = $2",
    )
    .bind(sl_id)
    .bind(prod_id)
    .fetch_optional(pool)
    .await
    .expect("query storage_location_inventory")
}

fn assert_rejected_as_inactive(result: Result<impl std::fmt::Debug, AppError>, ctx: &str) {
    match result {
        Ok(v) => panic!("{ctx}：應被拒絕，實際成功：{v:?}"),
        Err(AppError::BusinessRule(msg)) => {
            assert!(
                msg.contains("已停用"),
                "{ctx}：錯誤訊息應說明倉庫已停用，實際：{msg}"
            );
        }
        Err(other) => panic!("{ctx}：應回 422 BusinessRule，實際：{other:?}"),
    }
}

// ── 寫入路徑 1：ledger 側 upsert（GRN/TR-in/SR/RTN/ADJ-in 共用） ──
#[tokio::test]
async fn grn_into_inactive_warehouse_is_rejected() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, false).await;
    let doc_id = seed_submitted_grn(&pool, &fx, Decimal::new(100, 0)).await;

    let approver = approver_actor(&pool).await;
    let result = DocumentService::approve(&pool, &approver, doc_id).await;
    assert_rejected_as_inactive(result, "停用倉庫的 GRN 入庫核准");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        None,
        "被拒絕後不應留下任何庫存列（隱形庫存）"
    );

    let status: String = sqlx::query_scalar("SELECT status::text FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("query document status");
    assert_eq!(status, "submitted", "核准失敗後單據狀態不應變為 approved");
}

// ── 對照：出庫（decrement）不受本閘門限制，殘留庫存仍可清空 ──
#[tokio::test]
async fn adj_out_from_inactive_warehouse_still_succeeds() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, false).await;

    // 直接種殘留庫存（模擬 #31 之前遺留的隱形庫存），不經服務層——
    // 本測試驗證的是「清空殘留」這條路徑本身不被本次修復擋住。
    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(fx.storage_location_id)
    .bind(fx.product_id)
    .bind(Decimal::new(10, 0))
    .execute(&pool)
    .await
    .expect("seed residual stock");
    sqlx::query(
        "INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base) VALUES ($1, $2, $3)",
    )
    .bind(fx.warehouse_id)
    .bind(fx.product_id)
    .bind(Decimal::new(10, 0))
    .execute(&pool)
    .await
    .expect("seed inventory_snapshot");

    // 快照的 ledger 來源：ADJ 核准後 update_inventory_snapshot 會以 SUM(stock_ledger)
    // 全量重算快照（冪等），若只有本筆 ADJ 出庫的 ledger 列，重算會算成 -10 而觸發
    // migration 137 的非負約束（同 erp_adj_storage_floor.rs 的教訓：必須也種一筆
    // 入庫來源，快照重算後才會落在 0 而不是負值）。
    let grn_doc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'GRN', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(grn_doc_id)
    .bind(format!(
        "GRN-T-{}",
        &Uuid::new_v4().simple().to_string()[..10]
    ))
    .bind(fx.warehouse_id)
    .bind(SYSTEM_USER_ID)
    .execute(&pool)
    .await
    .expect("seed GRN document (ledger 來源)");
    sqlx::query(
        "INSERT INTO stock_ledger \
         (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no, direction, qty_base) \
         VALUES ($1, $2, $3, NOW(), 'GRN', $4, $5, 'in', $6)",
    )
    .bind(Uuid::new_v4())
    .bind(fx.warehouse_id)
    .bind(fx.product_id)
    .bind(grn_doc_id)
    .bind(format!(
        "GRN-T-{}",
        &Uuid::new_v4().simple().to_string()[..10]
    ))
    .bind(Decimal::new(10, 0))
    .execute(&pool)
    .await
    .expect("seed GRN in-ledger（快照來源）");

    let doc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'ADJ', $2, 'submitted', $3, CURRENT_DATE, $4)",
    )
    .bind(doc_id)
    .bind(format!(
        "ADJ-T-{}",
        &Uuid::new_v4().simple().to_string()[..10]
    ))
    .bind(fx.warehouse_id)
    .bind(SYSTEM_USER_ID)
    .execute(&pool)
    .await
    .expect("seed ADJ document");
    sqlx::query(
        "INSERT INTO document_lines \
         (id, document_id, line_no, product_id, qty, uom, unit_price, storage_location_id) \
         VALUES ($1, $2, 1, $3, $4, 'EA', NULL, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(fx.product_id)
    .bind(Decimal::new(-10, 0))
    .bind(fx.storage_location_id)
    .execute(&pool)
    .await
    .expect("seed ADJ line (出庫 10)");

    let approver = approver_actor(&pool).await;
    DocumentService::approve(&pool, &approver, doc_id)
        .await
        .expect("停用倉庫的出庫調整不應被本閘門擋下");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        Some(Decimal::ZERO),
        "殘留庫存應可被清空至 0"
    );
}

// ── 寫入路徑 2：StorageLocationService::create_inventory_item（INSERT 分支） ──
#[tokio::test]
async fn create_inventory_item_into_inactive_warehouse_is_rejected() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, false).await;

    let req = CreateStorageLocationInventoryItemRequest {
        product_id: fx.product_id,
        on_hand_qty: Decimal::new(5, 0),
        batch_no: None,
        expiry_date: None,
    };
    let result = StorageLocationService::create_inventory_item(
        &pool,
        &actor(),
        fx.storage_location_id,
        &req,
    )
    .await;
    assert_rejected_as_inactive(result, "停用倉庫手動新增儲位庫存（INSERT）");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        None,
        "被拒絕後不應留下任何庫存列"
    );
}

// ── 寫入路徑 2b：create_inventory_item 的 UPDATE 分支（既有 row 再疊加） ──
#[tokio::test]
async fn create_inventory_item_update_branch_on_inactive_warehouse_is_rejected() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, false).await;

    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(fx.storage_location_id)
    .bind(fx.product_id)
    .bind(Decimal::new(3, 0))
    .execute(&pool)
    .await
    .expect("seed existing inventory row");

    let req = CreateStorageLocationInventoryItemRequest {
        product_id: fx.product_id,
        on_hand_qty: Decimal::new(5, 0),
        batch_no: None,
        expiry_date: None,
    };
    let result = StorageLocationService::create_inventory_item(
        &pool,
        &actor(),
        fx.storage_location_id,
        &req,
    )
    .await;
    assert_rejected_as_inactive(result, "停用倉庫手動疊加儲位庫存（UPDATE 分支）");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        Some(Decimal::new(3, 0)),
        "被拒絕後既有庫存不應被疊加"
    );
}

// ── 寫入路徑 3：StorageLocationService::update_inventory_item（直接設定數量） ──
#[tokio::test]
async fn update_inventory_item_on_inactive_warehouse_is_rejected() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, false).await;

    let item_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO storage_location_inventory (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(item_id)
    .bind(fx.storage_location_id)
    .bind(fx.product_id)
    .bind(Decimal::new(3, 0))
    .execute(&pool)
    .await
    .expect("seed existing inventory row");

    let req = UpdateStorageLocationInventoryItemRequest {
        on_hand_qty: Decimal::new(999, 0),
    };
    let result =
        StorageLocationService::update_inventory_item(&pool, &actor(), item_id, &req).await;
    assert_rejected_as_inactive(result, "停用倉庫手動設定儲位庫存數量");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        Some(Decimal::new(3, 0)),
        "被拒絕後數量不應被覆蓋"
    );
}

// ── 併發情境：停用交易與入庫交易交錯，入庫必須等到停用 commit 後才讀到最新狀態 ──
//
// 模擬 #33 描述的失效情境 1：`delete_tx` 對 warehouse 列取 `FOR UPDATE` 但入庫路徑
// 過去從不 SELECT `warehouses`，兩者互不相干、可同時進行。本測試手動持有等同
// `delete_tx` 會取的鎖（`SELECT ... FOR UPDATE`），驗證 GRN 核准會被同一把鎖擋住，
// 待「停用」commit 後才繼續，且看到的必然是停用後的狀態（拒絕），不會有交錯窗口
// 讓入庫在停用完成前就把庫存寫進去。
#[tokio::test]
async fn grn_blocked_by_concurrent_deactivate_sees_post_commit_state() {
    let pool = setup_pool().await;
    let fx = seed_base(&pool, true).await; // 一開始是啟用中
    let doc_id = seed_submitted_grn(&pool, &fx, Decimal::new(50, 0)).await;

    // 持有等同 delete_tx / update_tx 停用時會取的鎖，先不 commit。
    let mut deactivate_tx = pool.begin().await.expect("begin deactivate tx");
    sqlx::query("SELECT * FROM warehouses WHERE id = $1 FOR UPDATE")
        .bind(fx.warehouse_id)
        .fetch_one(&mut *deactivate_tx)
        .await
        .expect("lock warehouse row");

    let approve_pool = pool.clone();
    let approver = approver_actor(&pool).await;
    let approve_handle =
        tokio::spawn(
            async move { DocumentService::approve(&approve_pool, &approver, doc_id).await },
        );

    // 讓 approve 的 SELECT ... FOR SHARE OF w 有機會先發出、卡在鎖等待上，
    // 再讓「停用」commit——確保入庫確實跨過了停用交易的邊界，而非搶在它之前完成。
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    sqlx::query("UPDATE warehouses SET is_active = false WHERE id = $1")
        .bind(fx.warehouse_id)
        .execute(&mut *deactivate_tx)
        .await
        .expect("deactivate warehouse");
    deactivate_tx.commit().await.expect("commit deactivate");

    let result = approve_handle.await.expect("join approve task");
    assert_rejected_as_inactive(result, "與停用交易並發交錯的 GRN 入庫核准");

    assert_eq!(
        sli_qty(&pool, fx.storage_location_id, fx.product_id).await,
        None,
        "並發交錯下也不應留下任何隱形庫存"
    );
}
