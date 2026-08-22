//! Regression：單據編號取號在並發下不得撞號，且序號跨過 99 後仍須遞增。
//!
//! 舊實作 `DocumentService::generate_doc_no` 是**無鎖的 read-then-write**：
//!
//! ```text
//! SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1
//!   → parse + 1 → INSERT
//! ```
//!
//! 雖包在 transaction 內，但 PostgreSQL 預設 READ COMMITTED 下純 SELECT **不鎖任何列**，
//! 兩個並發交易會讀到同一個最大序號、算出同一個新號、雙雙 INSERT，其中一個撞上唯一索引
//! `documents_doc_no_key`。同型缺陷在 protocol 取號已於 CRIT-01 修過（advisory lock），
//! 當時漏了 documents 這條路徑。
//!
//! 這個缺陷長期以 `api_grn_unshelved_allocation.rs` 平行執行時的間歇紅燈呈現
//! （症狀：`Key (doc_no)=(TR-260731-01) already exists`），一度被歸因為「那 4 支測試
//! 沒加 `#[serial]`」。實際上它們是忠實重現了正式路徑的並發缺陷——加 `#[serial]`
//! 等於關掉唯一的警報器，所以本 PR 選擇修正式路徑而不動那支測試。
//!
//! 第二個缺陷（同一函數）：`ORDER BY doc_no DESC` 取「最後一筆」是**字典序**，
//! 到第 100 張時 `"…-99"` 會排在 `"…-100"` 之前，於是重新算出 100 並再次撞號。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::constants::ROLE_SYSTEM_ADMIN;
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

/// 以 migration 033 的 SYSTEM user 當 actor：`documents.created_by` 的 FK 指向它。
fn actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "doc-no-race@test.local".into(),
        roles: vec![ROLE_SYSTEM_ADMIN.into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 一個倉庫 + 一個品項——PO 建單的最小前置（不動庫存、不需批號效期與儲位）。
async fn seed_ctx(pool: &PgPool) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let wh = Uuid::new_v4();
    let prod = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WHDN-{short}"))
        .bind("取號並發測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, 'EA')")
        .bind(prod)
        .bind(format!("SKUDN-{short}"))
        .bind("取號並發測試品")
        .execute(pool)
        .await
        .expect("seed product");

    (wh, prod)
}

fn po_request(wh: Uuid, prod: Uuid) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type: DocType::PO,
        warehouse_id: Some(wh),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        source_doc_id: None,
        doc_date: chrono::Utc::now().date_naive(),
        remark: Some("取號並發測試".into()),
        stocktake_scope: None,
        iacuc_no: None,
        protocol_id: None,
        lines: vec![DocumentLineInput {
            product_id: prod,
            qty: Decimal::from(1),
            uom: "EA".to_string(),
            unit_price: Some(Decimal::from(10)),
            batch_no: None,
            expiry_date: None,
            remark: None,
            storage_location_id: None,
            storage_location_from_id: None,
            storage_location_to_id: None,
        }],
    }
}

/// 核心回歸：N 個並發建單全部成功，且 `doc_no` 兩兩相異。
///
/// 舊實作在此必紅——多數請求會拿到
/// `duplicate key value violates unique constraint "documents_doc_no_key"`。
#[tokio::test]
#[serial]
async fn concurrent_creation_yields_unique_doc_no() {
    let pool = setup_pool().await;
    let (wh, prod) = seed_ctx(&pool).await;

    const N: usize = 8;
    let mut handles = Vec::with_capacity(N);
    for _ in 0..N {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            DocumentService::create(&pool, &actor(), &po_request(wh, prod)).await
        }));
    }

    let mut doc_nos = Vec::with_capacity(N);
    let mut failures = Vec::new();
    for h in handles {
        match h.await.expect("join task") {
            Ok(doc) => doc_nos.push(doc.document.doc_no),
            Err(e) => failures.push(e.to_string()),
        }
    }

    assert!(
        failures.is_empty(),
        "並發建單不應失敗，取號須為原子操作。失敗 {}/{N} 筆：{failures:?}",
        failures.len()
    );

    let mut unique = doc_nos.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        N,
        "並發取得的單據編號必須兩兩相異，實際：{doc_nos:?}"
    );
}

/// 序號跨過 99 之後仍須嚴格遞增（舊實作的 `ORDER BY doc_no DESC` 是字典序，
/// `"-99"` 會排在 `"-100"` 之前 → 重複算出 100）。
///
/// 先建一張取得當天前綴，再把 01..=99 佔滿，然後驗證下一張拿到 100 而非重複號。
#[tokio::test]
#[serial]
async fn sequence_continues_past_ninety_nine() {
    let pool = setup_pool().await;
    let (wh, prod) = seed_ctx(&pool).await;

    let first = DocumentService::create(&pool, &actor(), &po_request(wh, prod))
        .await
        .expect("建立首張單據");
    let prefix = first
        .document
        .doc_no
        .rsplit_once('-')
        .expect("doc_no 應為 {PREFIX}-{SEQ} 格式")
        .0
        .to_string();

    // 佔滿至 99。`ON CONFLICT DO NOTHING` 讓本測試可重複執行，也容忍同一天
    // 已被其他測試用掉的號。
    for seq in 1..=99u32 {
        sqlx::query(
            "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
             VALUES ($1, 'PO'::doc_type, $2, 'draft'::doc_status, $3, CURRENT_DATE, $4) \
             ON CONFLICT (doc_no) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(format!("{prefix}-{seq:02}"))
        .bind(wh)
        .bind(SYSTEM_USER_ID)
        .execute(&pool)
        .await
        .expect("佔位序號");
    }

    let next = DocumentService::create(&pool, &actor(), &po_request(wh, prod))
        .await
        .expect("99 之後仍應能建單");
    let seq: u32 = next
        .document
        .doc_no
        .rsplit_once('-')
        .expect("doc_no 格式")
        .1
        .parse()
        .expect("序號應為數字");

    assert!(
        seq >= 100,
        "序號跨過 99 後須繼續遞增，實際取得 {seq}（doc_no={}）",
        next.document.doc_no
    );
}
