//! 資安稽核 M-4：文件 list 端點跨建立者財務摘要洩漏回歸測試（service 層）。
//!
//! 修復前 `DocumentService::list` 無 created_by 收斂 → 任何持 `erp.document.view` 的
//! 非 WM/admin 角色（如 PURCHASING）可列出全部單據的財務摘要（金額/夥伴/計畫號）。
//! 修復後 list 接受 `created_by_scope`：`Some(uid)` 只回該使用者建立的單據，
//! `None`（WM/admin 全域監督）回全部。handler 依 `has_role(WM) || is_admin()` 決定傳哪個。

mod common;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::models::{DocumentQuery, UpdateDocumentRequest};
use erp_backend::services::DocumentService;
use erp_backend::{ActorContext, SYSTEM_USER_ID};
use serial_test::serial;
use uuid::Uuid;

fn empty_query() -> DocumentQuery {
    DocumentQuery {
        doc_type: None,
        doc_types: None,
        status: None,
        warehouse_id: None,
        partner_id: None,
        date_from: None,
        date_to: None,
        keyword: None,
        iacuc_no: None,
        product_id: None,
    }
}

async fn seed_user(app: &TestApp, tag: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("{}-{}@test.local", tag, &Uuid::new_v4().to_string()[..8]))
    .bind(format!("doc-user-{tag}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_document(app: &TestApp, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO documents (id, doc_type, doc_no, doc_date, created_by)
           VALUES ($1, 'PO', $2, CURRENT_DATE, $3)"#,
    )
    .bind(id)
    .bind(format!("PO-{}", &id.to_string()[..8]))
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert document");
    id
}

#[tokio::test]
#[serial]
async fn list_scoped_to_creator_hides_other_users_documents() {
    let app = TestApp::spawn().await;
    let user_a = seed_user(&app, "a").await;
    let user_b = seed_user(&app, "b").await;
    let a_doc1 = seed_document(&app, user_a).await;
    let a_doc2 = seed_document(&app, user_a).await;
    let b_doc = seed_document(&app, user_b).await;

    // Some(user_a)：非 WM/admin → 只看得到 A 自己建立的單據
    let scoped = DocumentService::list(&app.db_pool, &empty_query(), Some(user_a))
        .await
        .expect("scoped list");
    let scoped_ids: Vec<Uuid> = scoped.iter().map(|d| d.id).collect();
    assert!(
        scoped_ids.contains(&a_doc1) && scoped_ids.contains(&a_doc2),
        "A 應看到自己建立的兩張單"
    );
    assert!(
        !scoped_ids.contains(&b_doc),
        "A 不應看到 B 建立的單（M-4 跨建立者洩漏）"
    );

    // None（WM/admin）：全域可見，含 B 的單
    let all = DocumentService::list(&app.db_pool, &empty_query(), None)
        .await
        .expect("unscoped list");
    let all_ids: Vec<Uuid> = all.iter().map(|d| d.id).collect();
    assert!(
        all_ids.contains(&a_doc1) && all_ids.contains(&b_doc),
        "None scope（WM/admin）應看到全部含 B 的單"
    );
}

async fn seed_product(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, '支')")
        .bind(id)
        .bind(format!("SKU-{}", &id.to_string()[..8]))
        .bind("product-filter-test")
        .execute(&app.db_pool)
        .await
        .expect("insert product");
    id
}

async fn seed_document_with_line(app: &TestApp, created_by: Uuid, product_id: Uuid) -> Uuid {
    let doc_id = seed_document(app, created_by).await;
    sqlx::query(
        "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom) \
         VALUES ($1, $2, 1, $3, 1, '支')",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(product_id)
    .execute(&app.db_pool)
    .await
    .expect("insert document line");
    doc_id
}

/// 產品詳情頁「相關單據」：`DocumentQuery::product_id` 只回含該產品明細的單據。
/// 修復前此 tab 為寫死空狀態、後端 list 也無 product 過濾。
#[tokio::test]
#[serial]
async fn list_filtered_by_product_id_returns_only_docs_with_that_product() {
    let app = TestApp::spawn().await;
    let user = seed_user(&app, "pf").await;
    let prod_a = seed_product(&app).await;
    let prod_b = seed_product(&app).await;
    let doc_with_a = seed_document_with_line(&app, user, prod_a).await;
    let doc_with_b = seed_document_with_line(&app, user, prod_b).await;

    let mut q = empty_query();
    q.product_id = Some(prod_a);
    let docs = DocumentService::list(&app.db_pool, &q, None)
        .await
        .expect("list by product_id");
    let ids: Vec<Uuid> = docs.iter().map(|d| d.id).collect();
    assert!(ids.contains(&doc_with_a), "應含引用 A 產品的單");
    assert!(
        !ids.contains(&doc_with_b),
        "不應含只引用 B 產品的單（product_id 過濾失效）"
    );
}

async fn seed_warehouse(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &id.to_string()[..8]))
        .bind("測試倉")
        .execute(&app.db_pool)
        .await
        .expect("insert warehouse");
    id
}

async fn seed_shelf(app: &TestApp, warehouse_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO storage_locations (id, warehouse_id, code) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(warehouse_id)
        .bind(format!("SL-{}", &id.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert storage location");
    id
}

fn admin_actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "xw-admin@test.local".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// CodeRabbit critical 回歸：只改 `warehouse_id`、不帶 `lines` 的部分更新，仍須以「既有明細」
/// 做跨倉錯配驗證。修復前驗證藏在 `if let Some(lines)` 內 → 換倉庫但既有明細儲位仍屬舊倉庫，
/// 靜默重現跨倉錯配（正是本 PR 要修的東西）。
#[tokio::test]
#[serial]
async fn update_warehouse_only_still_validates_cross_warehouse() {
    let app = TestApp::spawn().await;
    let user = seed_user(&app, "xw").await;
    let wh_a = seed_warehouse(&app).await;
    let wh_b = seed_warehouse(&app).await;
    let shelf_a = seed_shelf(&app, wh_a).await;
    let prod = seed_product(&app).await;

    // 草稿 PO 在 WH_A，明細儲位在 WH_A（一致）
    let doc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'PO', $2, 'draft', $3, CURRENT_DATE, $4)",
    )
    .bind(doc_id)
    .bind(format!("PO-{}", &doc_id.to_string()[..8]))
    .bind(wh_a)
    .bind(user)
    .execute(&app.db_pool)
    .await
    .expect("insert doc");
    sqlx::query(
        "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom, storage_location_id) \
         VALUES ($1, $2, 1, $3, 1, '支', $4)",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(prod)
    .bind(shelf_a)
    .execute(&app.db_pool)
    .await
    .expect("insert line");

    // 只改倉庫到 WH_B、不帶 lines → 既有明細儲位仍屬 WH_A → 應擋下（修復前會繞過）
    let req = UpdateDocumentRequest {
        warehouse_id: Some(wh_b),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        protocol_id: None,
        source_doc_id: None,
        doc_date: None,
        remark: None,
        lines: None,
    };
    let err = DocumentService::update(&app.db_pool, &admin_actor(), doc_id, &req)
        .await
        .expect_err("只改倉庫不帶明細的部分更新應被跨倉驗證擋下");
    assert!(
        err.to_string().contains("儲位") || err.to_string().contains("倉庫"),
        "應回跨倉錯配 BusinessRule，實際：{err}"
    );

    // 倉庫未變更（update 整體回滾）
    let wh: Uuid = sqlx::query_scalar("SELECT warehouse_id FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query warehouse");
    assert_eq!(wh, wh_a, "被擋下時倉庫不應變更");
}
