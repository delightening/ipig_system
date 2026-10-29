//! 沖銷互抵：已核准的沖銷單（`REV`）在進銷貨報表裡依原單類型、以負值計入，與原單加總為 0。
//!
//! 沖銷單改為獨立類型 REV（migration 018/019）之前，沖銷單沿用原單類型，報表把它當成又一筆
//! 正向的入庫，金額被算兩次；改成 REV 之後若報表不處理，則是原單照算、沖銷不抵，仍然高估。
//! 這裡驗證：沖銷核准後，明細報表出現一列負值、彙總報表的金額回到沖銷前扣掉原單金額。
//!
//! 整合測試只能在獨立丟棄 DB 跑（`TEST_DATABASE_URL`），見 `common/test_db.rs`。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{CreateDocumentRequest, DocType, DocumentLineInput};
use erp_backend::services::report::{PurchaseSalesPartnerSummary, ReportQuery, ReportService};
use erp_backend::services::DocumentService;
use erp_backend::{ActorContext, SYSTEM_USER_ID};

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

fn user(id: Uuid, label: &str, role: &str) -> CurrentUser {
    CurrentUser {
        id,
        email: format!("{}@example.com", label),
        roles: vec![role.into()],
        permissions: vec![],
        jti: format!("test-{}", label),
        exp: 0,
        impersonated_by: None,
    }
}

fn wm_actor() -> ActorContext {
    ActorContext::User(user(SYSTEM_USER_ID, "wm-rev-report", "WAREHOUSE_MANAGER"))
}

/// 第二位倉管：入庫單禁止自核，需由非建單者核准。
async fn second_wm_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(user(
        test_db::seed_other_user(pool, "wm2-rev-report").await,
        "wm2-rev-report",
        "WAREHOUSE_MANAGER",
    ))
}

/// 管理員：沖銷核准人（與發起人不同 id，符合職務分離）。
async fn admin_actor(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, '沖銷報表測試管理員', 'x', true)",
    )
    .bind(id)
    .bind(format!("admin-rev-report-{}@example.com", &s[..8]))
    .execute(pool)
    .await
    .expect("seed admin user");
    ActorContext::User(user(id, "admin-rev-report", "admin"))
}

async fn seed_warehouse(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &s[..10]))
        .bind("沖銷報表測試倉")
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
        .bind("沖銷報表測試品")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

async fn seed_supplier(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO partners (id, code, name, partner_type) VALUES ($1, $2, $3, 'supplier')",
    )
    .bind(id)
    .bind(format!("SUP-{}", &s[..10]))
    .bind("沖銷報表測試供應商")
    .execute(pool)
    .await
    .expect("seed supplier");
    id
}

fn line(product: Uuid, qty: i64, shelf: Option<Uuid>) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty: Decimal::from(qty),
        uom: "pcs".into(),
        unit_price: Some(Decimal::from(10)),
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: shelf,
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

fn doc_req(
    doc_type: DocType,
    warehouse: Uuid,
    partner: Uuid,
    source_doc_id: Option<Uuid>,
    lines: Vec<DocumentLineInput>,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type,
        warehouse_id: Some(warehouse),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: Some(partner),
        source_doc_id,
        doc_date: chrono::Utc::now().date_naive(),
        remark: None,
        stocktake_scope: None,
        iacuc_no: None,
        protocol_id: None,
        lines,
    }
}

fn report_query(warehouse: Option<Uuid>) -> ReportQuery {
    ReportQuery {
        warehouse_id: warehouse,
        product_id: None,
        partner_id: None,
        date_from: None,
        date_to: None,
        category_id: None,
        iacuc_no: None,
        lab_name: None,
        customer_category: None,
        protocol_id: None,
    }
}

/// 建立並核准一張採購單與其入庫單（30 個 × 單價 10 ＝ 300），回傳 (grn_id, 供應商 id)。
async fn approved_po_and_grn(pool: &PgPool, wh: Uuid, shelf: Uuid, product: Uuid) -> (Uuid, Uuid) {
    let supplier = seed_supplier(pool).await;
    let po = DocumentService::create(
        pool,
        &wm_actor(),
        &doc_req(
            DocType::PO,
            wh,
            supplier,
            None,
            vec![line(product, 30, None)],
        ),
    )
    .await
    .expect("create PO");
    DocumentService::submit(pool, &wm_actor(), po.document.id)
        .await
        .expect("submit PO");
    DocumentService::approve(pool, &wm_actor(), po.document.id)
        .await
        .expect("approve PO");

    let grn = DocumentService::create(
        pool,
        &wm_actor(),
        &doc_req(
            DocType::GRN,
            wh,
            supplier,
            Some(po.document.id),
            vec![line(product, 30, Some(shelf))],
        ),
    )
    .await
    .expect("create GRN");
    DocumentService::submit(pool, &wm_actor(), grn.document.id)
        .await
        .expect("submit GRN");
    DocumentService::approve(pool, &second_wm_actor(pool).await, grn.document.id)
        .await
        .expect("approve GRN");
    (grn.document.id, supplier)
}

async fn total_purchase_monthly(pool: &PgPool) -> Decimal {
    ReportService::purchase_sales_monthly(pool, &report_query(None))
        .await
        .expect("monthly")
        .iter()
        .map(|r| r.net_purchase)
        .sum()
}

async fn total_purchase_by_category(pool: &PgPool) -> Decimal {
    ReportService::purchase_sales_by_category(pool, &report_query(None))
        .await
        .expect("by category")
        .iter()
        .map(|r| r.purchase_amount)
        .sum()
}

async fn partner_row(pool: &PgPool, supplier: Uuid) -> PurchaseSalesPartnerSummary {
    ReportService::purchase_sales_by_partner(pool, &report_query(None))
        .await
        .expect("by partner")
        .into_iter()
        .find(|r| r.partner_id == supplier)
        .expect("供應商應在彙總內")
}

#[tokio::test]
#[serial]
async fn approved_reversal_nets_out_original_in_purchase_reports() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    let (grn_id, supplier) = approved_po_and_grn(&pool, wh, shelf, product).await;

    // 沖銷前：入庫單 300 算進彙總
    let monthly_before = total_purchase_monthly(&pool).await;
    let category_before = total_purchase_by_category(&pool).await;
    let before = partner_row(&pool, supplier).await;
    assert_eq!(before.total_amount, Decimal::from(300));
    assert_eq!(before.net_amount, Decimal::from(300));
    assert_eq!(before.doc_count, 1);

    // 發起但尚未核准的沖銷單不影響彙總（只認已核准）
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    assert_eq!(total_purchase_monthly(&pool).await, monthly_before);
    assert_eq!(total_purchase_by_category(&pool).await, category_before);

    DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");

    // 彙總：各報表都恰好少 300
    assert_eq!(
        total_purchase_monthly(&pool).await,
        monthly_before - Decimal::from(300),
        "進銷貨月彙總應扣掉被沖銷的入庫金額"
    );
    assert_eq!(
        total_purchase_by_category(&pool).await,
        category_before - Decimal::from(300),
        "產品類別彙總應扣掉被沖銷的入庫金額"
    );
    let after = partner_row(&pool, supplier).await;
    assert_eq!(after.total_amount, Decimal::ZERO);
    assert_eq!(after.net_amount, Decimal::ZERO);
    assert_eq!(after.doc_count, 1, "筆數只算原始單據，沖銷單不另計");

    // 明細：入庫單一列正值，沖銷單一列負值，兩列加總為 0
    let rows = ReportService::purchase_lines(&pool, &report_query(Some(wh)))
        .await
        .expect("purchase lines");
    let grn_row = rows
        .iter()
        .find(|r| r.doc_id == grn_id)
        .expect("入庫單明細");
    let rev_row = rows
        .iter()
        .find(|r| r.doc_id == reversal.document.id)
        .expect("沖銷單應出現在採購明細");
    assert_eq!(grn_row.qty, Decimal::from(30));
    assert_eq!(rev_row.qty, Decimal::from(-30));
    assert_eq!(grn_row.line_total, Some(Decimal::from(300)));
    assert_eq!(rev_row.line_total, Some(Decimal::from(-300)));
    assert!(
        rev_row.doc_no.starts_with("REV-"),
        "沖銷單號：{}",
        rev_row.doc_no
    );
}
