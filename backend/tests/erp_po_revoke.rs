//! 撤銷「已核准、尚無有效下游單據」的採購單（PO）。
//!
//! PO 核准不寫庫存、不過帳，所以沒有東西需要紅字沖銷；`cancel` 卻一律拒絕 `approved`
//! 單據，使用者只能走二人核准的沖銷。本檔驗證補上的單人撤銷路徑
//! （`DocumentService::revoke_approved_po`）與它的邊界：
//!
//! - T1 成功：狀態 `cancelled`、稽核列帶原因、`can_revoke` 前真後假
//! - T2 缺原因／非倉管或管理員 → 拒絕
//! - T3 已有核准且未沖銷的入庫單 → 拒絕並提示改走沖銷
//! - T4 入庫單已被已核准的沖銷單沖銷 → 視為無有效入庫單，可撤銷
//! - T5 有草稿入庫單引用 → 拒絕並提示單號（不連帶作廢）
//! - T6 非 PO／非 Approved／沖銷單／重複撤銷 → 拒絕
//! - T7 列表的沖銷衍生狀態（`reversing` / `reversed`）
//! - T8 並發：撤銷 vs 從 PO 開入庫單，必為其一成功、另一回明確錯誤
//!
//! 整合測試只能在獨立丟棄 DB 跑（`TEST_DATABASE_URL`），見 `common/test_db.rs`。

use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{
    CreateDocumentRequest, DocStatus, DocType, DocumentLineInput, DocumentQuery,
};
use erp_backend::services::DocumentService;
use erp_backend::{ActorContext, AppError, SYSTEM_USER_ID};

#[path = "common/test_db.rs"]
mod test_db;

const REASON: &str = "下單品項有誤，改開新單";

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

/// 倉庫管理員：建單者兼撤銷／沖銷發起人。
fn wm_user() -> CurrentUser {
    user(SYSTEM_USER_ID, "wm-po-revoke", "WAREHOUSE_MANAGER")
}

fn wm_actor() -> ActorContext {
    ActorContext::User(wm_user())
}

/// 一般使用者：既非倉管也非管理員，不得撤銷。
fn plain_user() -> CurrentUser {
    user(Uuid::new_v4(), "plain-po-revoke", "viewer")
}

/// 第二位倉管——GRN 禁止自核，入庫需由非建單者核准。
async fn second_wm_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(user(
        test_db::seed_other_user(pool, "wm2-po-revoke").await,
        "wm2-po-revoke",
        "WAREHOUSE_MANAGER",
    ))
}

/// 管理員（沖銷核准人）——與發起人不同 id，符合 SoD。
async fn admin_actor(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, '撤銷測試管理員', 'x', true)",
    )
    .bind(id)
    .bind(format!("admin-po-revoke-{}@example.com", &s[..8]))
    .execute(pool)
    .await
    .expect("seed admin user");
    ActorContext::User(user(id, "admin-po-revoke", "admin"))
}

async fn seed_warehouse(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &s[..10]))
        .bind("撤銷測試倉")
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
        .bind("撤銷測試品")
        .execute(pool)
        .await
        .expect("seed product");
    id
}

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

/// 建立並核准一張採購單。PO 不在自核禁止清單內，建單者可自核。
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

/// 建立一張掛在 `po_id` 底下的草稿 GRN，回傳 (id, doc_no)。
async fn draft_grn_for_po(
    pool: &PgPool,
    warehouse: Uuid,
    shelf: Uuid,
    product: Uuid,
    po_id: Uuid,
    qty: i64,
) -> (Uuid, String) {
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
    (doc.document.id, doc.document.doc_no)
}

/// 建立並核准一張掛在 `po_id` 底下的 GRN，回傳 (id, doc_no)。
async fn approved_grn_for_po(
    pool: &PgPool,
    warehouse: Uuid,
    shelf: Uuid,
    product: Uuid,
    po_id: Uuid,
    qty: i64,
) -> (Uuid, String) {
    let (id, doc_no) = draft_grn_for_po(pool, warehouse, shelf, product, po_id, qty).await;
    DocumentService::submit(pool, &wm_actor(), id)
        .await
        .expect("submit GRN");
    let approver = second_wm_actor(pool).await;
    DocumentService::approve(pool, &approver, id)
        .await
        .expect("approve GRN");
    (id, doc_no)
}

async fn status_of(pool: &PgPool, id: Uuid) -> DocStatus {
    sqlx::query_scalar::<_, DocStatus>("SELECT status FROM documents WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("query status")
}

/// 取業務規則錯誤的訊息；其他錯誤型別直接讓測試失敗。
fn business_rule_message(err: AppError) -> String {
    match err {
        AppError::BusinessRule(msg) => msg,
        other => panic!("預期 BusinessRule，實得 {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn revoking_approved_unreceived_po_cancels_it_and_audits_reason() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let product = seed_product(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;

    // can_revoke 與 revoke 共用判斷：撤銷前為真
    let before = DocumentService::get_by_id(&pool, po_id).await.expect("get");
    assert!(
        DocumentService::can_revoke(&pool, &wm_user(), &before.document)
            .await
            .expect("can_revoke"),
        "已核准且無下游單據的 PO，倉管應可撤銷"
    );

    let revoked = DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
        .await
        .expect("撤銷應成功");
    assert_eq!(revoked.document.status, DocStatus::Cancelled);
    assert_eq!(status_of(&pool, po_id).await, DocStatus::Cancelled);

    // 稽核：事件類型與一般作廢區分，after 快照帶原因，且與狀態更新同一個 tx 落地。
    let (event_count, reason): (i64, Option<String>) = sqlx::query_as(
        "SELECT COUNT(*), MAX(after_data->>'revoke_reason') FROM user_activity_logs \
         WHERE event_type = 'DOC_REVOKE_APPROVED' AND entity_type = 'document' AND entity_id = $1",
    )
    .bind(po_id)
    .fetch_one(&pool)
    .await
    .expect("query audit");
    assert_eq!(event_count, 1, "撤銷必須寫一筆稽核");
    assert_eq!(reason.as_deref(), Some(REASON), "原因必須進稽核快照");

    // 撤銷後不再可撤銷
    let after = DocumentService::get_by_id(&pool, po_id).await.expect("get");
    assert!(
        !DocumentService::can_revoke(&pool, &wm_user(), &after.document)
            .await
            .expect("can_revoke"),
        "已撤銷的 PO 不應再可撤銷"
    );
}

#[tokio::test]
#[serial]
async fn revoking_requires_reason_and_warehouse_manager_or_admin() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let product = seed_product(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;

    for bad in ["", "   ", "太短"] {
        let err = DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, bad)
            .await
            .expect_err("缺原因或原因過短必須拒絕");
        assert!(
            matches!(err, AppError::Validation(_)),
            "原因 {:?} 應回 Validation，實得 {:?}",
            bad,
            err
        );
    }

    let outsider = ActorContext::User(plain_user());
    let err = DocumentService::revoke_approved_po(&pool, &outsider, po_id, REASON)
        .await
        .expect_err("非倉管／管理員必須拒絕");
    assert!(matches!(err, AppError::Forbidden(_)), "實得 {:?}", err);

    // 管理員可撤銷
    let admin = admin_actor(&pool).await;
    DocumentService::revoke_approved_po(&pool, &admin, po_id, REASON)
        .await
        .expect("管理員可撤銷");

    // 以上拒絕都不得留下狀態變更或稽核列的副作用——最終只有管理員那一次
    let events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'DOC_REVOKE_APPROVED' AND entity_id = $1",
    )
    .bind(po_id)
    .fetch_one(&pool)
    .await
    .expect("query audit");
    assert_eq!(events, 1);
}

#[tokio::test]
#[serial]
async fn po_with_approved_grn_must_go_through_reversal() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (_, grn_no) = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;

    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
            .await
            .expect_err("已有核准入庫單的 PO 不得撤銷"),
    );
    assert!(msg.contains(&grn_no), "訊息應帶入庫單號：{}", msg);
    assert!(msg.contains("沖銷"), "訊息應引導改走沖銷：{}", msg);
    assert_eq!(status_of(&pool, po_id).await, DocStatus::Approved);

    // can_revoke 與 revoke 同源：此時為假
    let doc = DocumentService::get_by_id(&pool, po_id).await.expect("get");
    assert!(
        !DocumentService::can_revoke(&pool, &wm_user(), &doc.document)
            .await
            .expect("can_revoke")
    );
}

#[tokio::test]
#[serial]
async fn po_whose_grn_was_reversed_can_be_revoked() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (grn_id, _) = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;

    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");

    // 沖銷單還在待核准：原入庫單仍算有效，PO 不可撤銷
    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
            .await
            .expect_err("沖銷尚未核准，入庫單仍有效"),
    );
    assert!(msg.contains("沖銷"), "{}", msg);

    DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");

    // 沖銷核准後，入庫單不算數（與 exclude_reversed_grn! 同源）→ 可撤銷
    let revoked = DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
        .await
        .expect("入庫單已被沖銷，PO 應可撤銷");
    assert_eq!(revoked.document.status, DocStatus::Cancelled);
}

#[tokio::test]
#[serial]
async fn po_with_draft_grn_is_blocked_with_doc_no() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (draft_id, draft_no) = draft_grn_for_po(&pool, wh, shelf, product, po_id, 40).await;

    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
            .await
            .expect_err("有草稿入庫單引用，必須先處理"),
    );
    assert!(msg.contains("請先處理單據"), "{}", msg);
    assert!(msg.contains(&draft_no), "訊息應帶草稿單號：{}", msg);

    // 擋下就是擋下：不連帶作廢草稿，PO 維持 approved
    assert_eq!(status_of(&pool, po_id).await, DocStatus::Approved);
    assert_eq!(status_of(&pool, draft_id).await, DocStatus::Draft);
}

#[tokio::test]
#[serial]
async fn only_approved_non_reversal_purchase_orders_can_be_revoked() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;

    // 草稿 PO：非 Approved
    let draft = DocumentService::create(
        &pool,
        &wm_actor(),
        &doc_req(DocType::PO, wh, None, vec![po_line(product, 10)]),
    )
    .await
    .expect("create draft PO");
    business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), draft.document.id, REASON)
            .await
            .expect_err("草稿 PO 不走撤銷（走既有作廢／刪除）"),
    );

    // 非 PO：已核准的 GRN
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (grn_id, _) = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;
    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), grn_id, REASON)
            .await
            .expect_err("非 PO 不可撤銷"),
    );
    assert!(msg.contains("採購單"), "{}", msg);

    // 沖銷單：已核准的 PO 沖銷單（PO 要有有效入庫單才能發起沖銷，見 create_reversal 的守衛）
    let other_po = approved_po(&pool, wh, product, 5).await;
    approved_grn_for_po(&pool, wh, shelf, product, other_po, 5).await;
    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), other_po)
        .await
        .expect("建立 PO 沖銷單");
    DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准 PO 沖銷單");

    // 被沖銷的原單：沖銷單以 reverses_doc_id 指向它、不帶 source_doc_id，
    // 若只查下游單據會漏掉，原單就能被撤銷成 cancelled 卻留著有效的沖銷單。
    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), other_po, REASON)
            .await
            .expect_err("已被沖銷的 PO 不可撤銷"),
    );
    assert!(msg.contains("沖銷單"), "{}", msg);
    assert_eq!(status_of(&pool, other_po).await, DocStatus::Approved);

    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), reversal.document.id, REASON)
            .await
            .expect_err("沖銷單不可撤銷"),
    );
    assert!(msg.contains("沖銷單"), "{}", msg);

    // 重複撤銷：第二次時 PO 已是 cancelled
    let third = approved_po(&pool, wh, product, 7).await;
    DocumentService::revoke_approved_po(&pool, &wm_actor(), third, REASON)
        .await
        .expect("第一次撤銷");
    business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), third, REASON)
            .await
            .expect_err("不可重複撤銷"),
    );
}

#[tokio::test]
#[serial]
async fn list_exposes_derived_reversal_state_without_new_db_status() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (grn_id, _) = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;

    let query = DocumentQuery {
        doc_type: Some(DocType::GRN),
        doc_types: None,
        status: None,
        warehouse_id: None,
        partner_id: None,
        date_from: None,
        date_to: None,
        keyword: None,
        iacuc_no: None,
        product_id: None,
    };
    let state_of = |items: Vec<erp_backend::models::DocumentListItem>| {
        items
            .into_iter()
            .find(|d| d.id == grn_id)
            .expect("列表應含該 GRN")
            .reversal_state
    };

    assert_eq!(
        state_of(
            DocumentService::list(&pool, &query, None)
                .await
                .expect("list")
        ),
        None,
        "未沖銷時沒有衍生狀態"
    );

    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    assert_eq!(
        state_of(
            DocumentService::list(&pool, &query, None)
                .await
                .expect("list")
        )
        .as_deref(),
        Some("reversing"),
        "沖銷單待核准 → 沖銷中"
    );

    DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");
    assert_eq!(
        state_of(
            DocumentService::list(&pool, &query, None)
                .await
                .expect("list")
        )
        .as_deref(),
        Some("reversed"),
        "沖銷單已核准 → 已沖銷"
    );
    // 原單的 DB 狀態仍是 approved：「已沖銷」是衍生顯示，不是新狀態
    assert_eq!(status_of(&pool, grn_id).await, DocStatus::Approved);
}

/// T8：撤銷 PO 與「從 PO 開入庫單」並發，結果必為其一成功、另一回明確錯誤。
///
/// 兩邊都先 `FOR UPDATE` 鎖 PO（`revoke_approved_po` 與 `create_additional_grn`），
/// 所以後到者必定看得到先到者已提交的結果。修復前 `create_additional_grn` 不鎖 PO，
/// 兩邊可能同時成功，留下「指向已作廢 PO 的入庫單」。
#[tokio::test]
#[serial]
async fn revoke_and_create_grn_never_both_succeed() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let product = seed_product(&pool).await;

    for round in 0..6 {
        let po_id = approved_po(&pool, wh, product, 100).await;

        let revoke = {
            let pool = pool.clone();
            tokio::spawn(async move {
                DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON).await
            })
        };
        let grn = {
            let pool = pool.clone();
            tokio::spawn(async move {
                DocumentService::create_additional_grn(&pool, po_id, SYSTEM_USER_ID).await
            })
        };
        let revoke = revoke.await.expect("join revoke");
        let grn = grn.await.expect("join grn");

        assert_ne!(
            revoke.is_ok(),
            grn.is_ok(),
            "第 {} 輪：撤銷與開入庫單不得同時成功或同時失敗（revoke_err={:?}, grn_ok={}）",
            round,
            revoke.as_ref().err(),
            grn.is_ok()
        );

        // 不變量：已作廢的 PO 底下不得有未作廢的入庫單
        let live_children: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM documents g \
             JOIN documents po ON po.id = g.source_doc_id \
             WHERE po.id = $1 AND po.status = 'cancelled' AND g.status <> 'cancelled'",
        )
        .bind(po_id)
        .fetch_one(&pool)
        .await
        .expect("query invariant");
        assert_eq!(
            live_children, 0,
            "第 {} 輪：留下了指向已作廢 PO 的入庫單",
            round
        );
    }
}

/// 詳情回應的分流欄位 `(can_revoke, revoke_blocked_by, has_effective_downstream)`。
/// `RevokeView` 型別本身不對外公開，所以攤成 tuple 方便比對。
async fn view_of(
    pool: &PgPool,
    user: &CurrentUser,
    po_id: Uuid,
) -> (bool, Option<(String, DocType, DocStatus)>, bool) {
    let doc = DocumentService::get_by_id(pool, po_id).await.expect("get");
    let view = DocumentService::revoke_view(pool, user, &doc.document)
        .await
        .expect("revoke_view");
    (
        view.can_revoke,
        view.revoke_blocked_by
            .map(|b| (b.doc_no, b.doc_type, b.status)),
        view.has_effective_downstream,
    )
}

/// QA 退件（只有草稿／待核准入庫單的 PO）：撤銷要被點名擋下（單號與狀態），
/// 而且不能因此放行沖銷——`has_effective_downstream` 必須是假。
#[tokio::test]
#[serial]
async fn revoke_view_names_draft_and_submitted_grn_as_blocker() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    // 沒有任何下游單據：可撤銷，沒有擋下者
    let clean_po = approved_po(&pool, wh, product, 10).await;
    assert_eq!(
        view_of(&pool, &wm_user(), clean_po).await,
        (true, None, false)
    );

    // 只有草稿入庫單
    let draft_po = approved_po(&pool, wh, product, 10).await;
    let (draft_id, draft_no) = draft_grn_for_po(&pool, wh, shelf, product, draft_po, 4).await;
    assert_eq!(
        view_of(&pool, &wm_user(), draft_po).await,
        (
            false,
            Some((draft_no.clone(), DocType::GRN, DocStatus::Draft)),
            false
        )
    );
    // 角色不符者：can_revoke 為假，而且**不得**讀到擋下者——那是別人建立的下游單據的單號與
    // 狀態，詳情的存取檢查只看建單者（CodeRabbit #186）
    let (can, blocked, _) = view_of(&pool, &plain_user(), draft_po).await;
    assert!(!can);
    assert!(blocked.is_none());

    // 同一張入庫單提交後（待核准）：仍是擋下者，狀態跟著變
    DocumentService::submit(&pool, &wm_actor(), draft_id)
        .await
        .expect("submit GRN");
    assert_eq!(
        view_of(&pool, &wm_user(), draft_po).await,
        (
            false,
            Some((draft_no, DocType::GRN, DocStatus::Submitted)),
            false
        )
    );
}

/// 有效入庫單：只能沖銷（has_effective_downstream 為真、不是擋下者）；
/// 該入庫單被核准的沖銷單沖銷後，回到「可撤銷」。
#[tokio::test]
#[serial]
async fn revoke_view_flags_effective_grn_until_it_is_reversed() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;
    let admin = admin_actor(&pool).await;
    let po_id = approved_po(&pool, wh, product, 100).await;
    let (grn_id, _) = approved_grn_for_po(&pool, wh, shelf, product, po_id, 100).await;

    assert_eq!(view_of(&pool, &wm_user(), po_id).await, (false, None, true));

    let reversal = DocumentService::create_reversal(&pool, &wm_actor(), grn_id)
        .await
        .expect("建立沖銷單");
    // 沖銷單待核准：入庫單仍有效
    assert_eq!(view_of(&pool, &wm_user(), po_id).await, (false, None, true));

    DocumentService::approve_reversal(&pool, &admin, reversal.document.id)
        .await
        .expect("核准沖銷單");
    assert_eq!(view_of(&pool, &wm_user(), po_id).await, (true, None, false));
}

/// 沖銷與撤銷互補：PO 沒有有效入庫單時，直接呼叫沖銷也要被擋（UI 藏按鈕不夠，API 仍可直呼），
/// 並且不得留下沖銷單。
#[tokio::test]
#[serial]
async fn reversal_of_po_requires_an_effective_grn() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool).await;

    let reversal_count = |po_id: Uuid| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM documents WHERE reverses_doc_id = $1",
            )
            .bind(po_id)
            .fetch_one(&pool)
            .await
            .expect("count reversals")
        }
    };

    // 完全沒有入庫單
    let bare_po = approved_po(&pool, wh, product, 10).await;
    let msg = business_rule_message(
        DocumentService::create_reversal(&pool, &wm_actor(), bare_po)
            .await
            .expect_err("沒有有效入庫單的 PO 不該沖銷"),
    );
    assert!(msg.contains("撤銷"), "訊息應引導改用撤銷：{}", msg);
    assert_eq!(reversal_count(bare_po).await, 0);

    // 只有草稿入庫單：草稿不算有效，同樣擋下
    let draft_po = approved_po(&pool, wh, product, 10).await;
    draft_grn_for_po(&pool, wh, shelf, product, draft_po, 4).await;
    business_rule_message(
        DocumentService::create_reversal(&pool, &wm_actor(), draft_po)
            .await
            .expect_err("只有草稿入庫單的 PO 不該沖銷"),
    );
    assert_eq!(reversal_count(draft_po).await, 0);

    // 有已核准且未沖銷的入庫單：既有沖銷路徑照常
    let received_po = approved_po(&pool, wh, product, 10).await;
    approved_grn_for_po(&pool, wh, shelf, product, received_po, 10).await;
    DocumentService::create_reversal(&pool, &wm_actor(), received_po)
        .await
        .expect("有有效入庫單的 PO 可以發起沖銷");
    assert_eq!(reversal_count(received_po).await, 1);
}

/// CodeRabbit #186：已核准的**非入庫單**下游單據仍擋撤銷（撤銷不會回沖它），但不算「有效入庫單」——
/// 不能因此放行 PO 的沖銷（否則沒有入庫單的 PO 也能走二人核准的沖銷）。
#[tokio::test]
#[serial]
async fn approved_non_grn_downstream_blocks_revoke_but_does_not_allow_po_reversal() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let product = seed_product(&pool).await;
    let po_id = approved_po(&pool, wh, product, 10).await;

    // 直接插一張已核准、指向該 PO 的採購退貨（業務流程不一定能走到，這裡只驗判準）
    let pr_no = format!("PR-T-{}", &Uuid::new_v4().simple().to_string()[..10]);
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, source_doc_id) \
         VALUES ($1, 'PR', $2, 'approved', $3, CURRENT_DATE, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(&pr_no)
    .bind(wh)
    .bind(SYSTEM_USER_ID)
    .bind(po_id)
    .execute(&pool)
    .await
    .expect("seed approved PR");

    // 撤銷：被擋，訊息點名該單據
    let msg = business_rule_message(
        DocumentService::revoke_approved_po(&pool, &wm_actor(), po_id, REASON)
            .await
            .expect_err("已核准的下游單據擋撤銷"),
    );
    assert!(msg.contains(&pr_no), "{}", msg);

    // 但不算有效入庫單：沖銷按鈕不放行，直接呼叫沖銷也被拒
    assert_eq!(
        view_of(&pool, &wm_user(), po_id).await,
        (false, None, false)
    );
    business_rule_message(
        DocumentService::create_reversal(&pool, &wm_actor(), po_id)
            .await
            .expect_err("沒有有效入庫單，不得沖銷"),
    );
}
