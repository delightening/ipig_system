//! R97-1：ERP 單據核准的職務分離（SoD）與 ADJ 二審門檻。
//!
//! 背景（2026-08-16 查 prod）：181 張已核准單據裡 171 張是「建立者自己核准」，
//! 且 69 張 ADJ 全部因為單價留空而算出金額 0 → 二審門檻自上線以來一次都沒生效過。
//!
//! 本檔鎖住四件事：
//! - T1/T2 受管制的 ADJ / GRN：建立者不得自核；換人核准則放行。
//! - T3 TR / STK **刻意不納入**——`assign_unassigned` 自動建的同倉自核准 TR
//!   是使用者 2026-07 的裁定，納入會直接打壞它。這個測試存在的目的是
//!   「未來有人想擴大 SoD 範圍時，會先看到這是刻意的」。
//! - T4/T5/T6 ADJ 門檻以 `unit_price` → `products.cost_price` 取價，
//!   **取價條件是 > 0 而非 IS NOT NULL**：`validate_line_qty_price` 對 ADJ
//!   只擋負數，明確填 0 若被當成有效價格，就是繞過門檻的第二扇門
//!   （T6；CodeAnt／Qodo 於 PR #139 各自獨立指出）。
//! - T7 兩階段核准必須是兩個人：第一階段的 WM 核准人不得再擔任最終核准人，
//!   否則大金額 ADJ 的兩道關卡會塌縮成一個人（Qodo 於 PR #139 指出）。
//! - T10/T11 **系統自動產生的單據豁免**（migration 152）：盤點差異調整單的
//!   `created_by` 記的是「核准盤點單的那個人」而非作者，套用自核守衛會讓剛盤完點
//!   的倉管核准不了系統為他產生的調整單。豁免依據必須是 `system_generated`
//!   而非 `source_doc_id`——後者可由 request 填入，等於留下繞過方式。

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
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 建單者（沿用 migration 033 已存在的 system user，免再 seed）。
fn creator() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "creator-r97-1@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-creator".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 另一位倉庫管理員——與建單者不同 id，符合 SoD。
///
/// 必須真的寫進 `users`：`documents.approved_by` 有 FK。
async fn other_manager(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, '另一位倉管', 'x', true)",
    )
    .bind(id)
    .bind(format!("wm2-r97-1-{}@example.com", &s[..8]))
    .execute(pool)
    .await
    .expect("seed second warehouse manager");
    ActorContext::User(CurrentUser {
        id,
        email: "wm2-r97-1@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-wm2".into(),
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
        .bind("SoD 測試倉")
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

/// `cost_price` 預設為 NULL——對應 prod 現況（177 個 ADJ 品項全部未維護成本）。
async fn seed_product(pool: &PgPool, cost_price: Option<Decimal>) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO products (id, sku, name, cost_price) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(format!("SKU-{}", &s[..10]))
        .bind("SoD 測試品")
        .bind(cost_price)
        .execute(pool)
        .await
        .expect("seed product");
    id
}

fn line(
    product: Uuid,
    qty: Decimal,
    shelf: Uuid,
    unit_price: Option<Decimal>,
) -> DocumentLineInput {
    DocumentLineInput {
        product_id: product,
        qty,
        uom: "pcs".into(),
        unit_price,
        batch_no: None,
        expiry_date: None,
        remark: None,
        storage_location_id: Some(shelf),
        storage_location_from_id: None,
        storage_location_to_id: None,
    }
}

fn req(doc_type: DocType, warehouse: Uuid, lines: Vec<DocumentLineInput>) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type,
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

/// 建立 + 送審一張 GRN，回傳 doc id。GRN 單價必填且 > 0。
async fn submitted_grn(pool: &PgPool, wh: Uuid, shelf: Uuid, product: Uuid) -> Uuid {
    let doc = DocumentService::create(
        pool,
        &creator(),
        &req(
            DocType::GRN,
            wh,
            vec![line(
                product,
                Decimal::from(10),
                shelf,
                Some(Decimal::from(10)),
            )],
        ),
    )
    .await
    .expect("create GRN");
    DocumentService::submit(pool, &creator(), doc.document.id)
        .await
        .expect("submit GRN");
    doc.document.id
}

// ── T1：受管制類型，建立者不得自核 ────────────────────────────────

#[tokio::test]
#[serial]
async fn grn_creator_cannot_approve_own_document() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;

    let doc_id = submitted_grn(&pool, wh, shelf, product).await;

    let err = DocumentService::approve(&pool, &creator(), doc_id)
        .await
        .expect_err("建立者自核 GRN 應被擋下");
    let msg = err.to_string();
    assert!(
        msg.contains("職務分離"),
        "錯誤訊息應點明職務分離，實際：{msg}"
    );
}

// ── T2：換一位倉管核准則放行 ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn grn_other_manager_can_approve() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;
    let approver = other_manager(&pool).await;

    let doc_id = submitted_grn(&pool, wh, shelf, product).await;

    let approved = DocumentService::approve(&pool, &approver, doc_id)
        .await
        .expect("換人核准應放行");
    assert_eq!(
        approved.document.status,
        erp_backend::models::DocStatus::Approved
    );
}

// ── T3：TR 刻意不納入 SoD（保護既有裁定） ────────────────────────

#[tokio::test]
#[serial]
async fn transfer_self_approval_stays_allowed_by_design() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf_from = seed_shelf(&pool, wh).await;
    let shelf_to = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;

    // 先入庫，TR 才有貨可調（GRN 由另一位倉管核准以符合 T1）。
    let approver = other_manager(&pool).await;
    let grn_id = submitted_grn(&pool, wh, shelf_from, product).await;
    DocumentService::approve(&pool, &approver, grn_id)
        .await
        .expect("approve GRN");

    let mut tr_line = line(product, Decimal::from(5), shelf_from, None);
    tr_line.storage_location_id = None;
    tr_line.storage_location_from_id = Some(shelf_from);
    tr_line.storage_location_to_id = Some(shelf_to);

    let mut tr_req = req(DocType::TR, wh, vec![tr_line]);
    tr_req.warehouse_from_id = Some(wh);
    tr_req.warehouse_to_id = Some(wh);

    let tr = DocumentService::create(&pool, &creator(), &tr_req)
        .await
        .expect("create TR");
    DocumentService::submit(&pool, &creator(), tr.document.id)
        .await
        .expect("submit TR");

    // 刻意由建立者自核——`assign_unassigned` 的自動 TR 正是走這條路。
    DocumentService::approve(&pool, &creator(), tr.document.id)
        .await
        .expect("TR 自核應維持放行（使用者 2026-07 裁定，勿改）");
}

// ── T4：ADJ 無法估價 → 一律送二審（原本是繞過門檻的方法） ────────

#[tokio::test]
#[serial]
async fn adj_without_any_price_requires_admin_approval() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    // 單價與 cost_price 都沒有 —— prod 上 541 筆 ADJ 明細的實際狀態。
    let product = seed_product(&pool, None).await;

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            vec![line(product, Decimal::from(-1), shelf, None)],
        ),
    )
    .await
    .expect("create ADJ");

    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");

    assert_eq!(
        submitted.document.requires_manager_approval,
        Some(true),
        "無法估價的 ADJ 必須升級為需 ADMIN 二審；退回金額 0 正是原本的漏洞"
    );
}

// ── T5：cost_price 可估價且低於門檻 → 不需二審 ──────────────────

#[tokio::test]
#[serial]
async fn adj_priced_below_threshold_skips_admin_approval() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    // 單價留空，但品項有標準成本 → 由 cost_price 估價（1 * 1 = 1，遠低於預設門檻 5000）。
    let product = seed_product(&pool, Some(Decimal::from(1))).await;

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            vec![line(product, Decimal::from(-1), shelf, None)],
        ),
    )
    .await
    .expect("create ADJ");

    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");

    assert_eq!(
        submitted.document.requires_manager_approval,
        Some(false),
        "有 cost_price 且金額低於門檻時不應要求二審——否則倉管日常會被門檻灌爆"
    );
}

// ── T6：明確填 unit_price = 0 不得繞過門檻（PR #139 bot 抓到的第二扇門） ──

#[tokio::test]
#[serial]
async fn adj_with_explicit_zero_price_still_requires_admin_approval() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    // 品項有高額標準成本；若 0 被當成有效價格，就不會 fallback 到它。
    let product = seed_product(&pool, Some(Decimal::from(999_999))).await;

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            // `validate_line_qty_price` 對 ADJ 只擋負數，0 是合法輸入。
            vec![line(product, Decimal::from(-1), shelf, Some(Decimal::ZERO))],
        ),
    )
    .await
    .expect("create ADJ");

    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");

    assert_eq!(
        submitted.document.requires_manager_approval,
        Some(true),
        "unit_price=0 必須視為未估價並 fallback 到 cost_price；\
         若把 0 當有效價格，填 0 就是繞過門檻的第二扇門"
    );
}

// ── T7：兩階段核准必須是兩個人（WM 核准人不得再當最終核准人） ──────

#[tokio::test]
#[serial]
async fn wm_approver_cannot_also_be_final_admin_approver() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await; // 無法估價 → 必定需二審

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            vec![line(product, Decimal::from(-1), shelf, None)],
        ),
    )
    .await
    .expect("create ADJ");
    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");
    assert_eq!(submitted.document.requires_manager_approval, Some(true));

    // 第一階段：由非建單者的 B 完成 WM 核准。
    let both_roles = other_manager(&pool).await;
    DocumentService::approve(&pool, &both_roles, adj.document.id)
        .await
        .expect("WM 核准應通過（B 不是建單者）");

    // 第二階段：同一個 B 再以 ADMIN 身分做最終核准 —— 必須被擋，
    // 否則大金額 ADJ 的兩道關卡塌縮成一個人。
    let err = DocumentService::admin_approve(&pool, &both_roles, adj.document.id)
        .await
        .expect_err("WM 核准人不得再擔任最終核准人");
    assert!(
        err.to_string().contains("職務分離"),
        "應因職務分離被擋，實際：{err}"
    );
}

// ── T8/T9：ADJ 自己的兩條核准路徑（CodeRabbit 於 PR #139 指出的覆蓋缺口） ──
//
// T1–T7 對 `SELF_APPROVAL_FORBIDDEN_DOC_TYPES` 的驗證其實只走 GRN：
// T4–T6 測的是送審階段的門檻計算、T7 觸發的是第二道（WM≠ADMIN）守衛。
// 也就是說**把 ADJ 從清單移除，整份測試仍會全綠**。以下兩項補上這個缺口。

/// ADJ 建立者不得走一般核准（`approve`）核准自己的單。
#[tokio::test]
#[serial]
async fn adj_creator_cannot_approve_own_document() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    // 有低額 cost_price → 不會因為門檻而轉去 ADMIN 流程，確保打到的是一般核准路徑。
    let product = seed_product(&pool, Some(Decimal::from(1))).await;

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            vec![line(product, Decimal::from(-1), shelf, None)],
        ),
    )
    .await
    .expect("create ADJ");
    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");
    assert_eq!(
        submitted.document.requires_manager_approval,
        Some(false),
        "本案要測一般核准路徑，不應落入 ADMIN 二審流程"
    );

    let err = DocumentService::approve(&pool, &creator(), adj.document.id)
        .await
        .expect_err("建立者自核 ADJ 應被擋下");
    assert!(
        err.to_string().contains("職務分離"),
        "應因職務分離被擋，實際：{err}"
    );
}

/// ADJ 建立者不得以 ADMIN 身分做最終核准（第一道守衛在 `admin_approve` 也要生效）。
#[tokio::test]
#[serial]
async fn adj_creator_cannot_final_approve_own_document() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await; // 無法估價 → 必定需二審

    let adj = DocumentService::create(
        &pool,
        &creator(),
        &req(
            DocType::ADJ,
            wh,
            vec![line(product, Decimal::from(-1), shelf, None)],
        ),
    )
    .await
    .expect("create ADJ");
    let submitted = DocumentService::submit(&pool, &creator(), adj.document.id)
        .await
        .expect("submit ADJ");
    assert_eq!(submitted.document.requires_manager_approval, Some(true));

    // 第一階段由非建單者完成，讓單據合法進入 wm_approved。
    let wm = other_manager(&pool).await;
    DocumentService::approve(&pool, &wm, adj.document.id)
        .await
        .expect("WM 核准應通過");

    // 建單者自己來做最終核准 —— 第一道守衛必須擋下。
    let err = DocumentService::admin_approve(&pool, &creator(), adj.document.id)
        .await
        .expect_err("建立者不得對自己的單做最終核准");
    assert!(
        err.to_string().contains("職務分離"),
        "應因職務分離被擋，實際：{err}"
    );
}

// ── T10/T11：系統自動產生的單據豁免（migration 152） ──────────────
//
// 盤點單核准後，`create_stocktake_reconciliation` 會自動產生一張待審核 ADJ，
// 並把 `created_by` 記成「核准盤點單的那個人」。那張單沒有真正的作者，
// 套用「建立者不得自核」防的是一個不存在的人，還會讓剛盤完點的倉管
// 核准不了系統為他產生的調整單（該函式設計本就是「倉管單一核准即套用」）。
//
// 2026-08-16 查 prod：過去 28 張盤點自動 ADJ **全部**是同一人建立又核准。

/// 直接建一張 ADJ 並指定 `system_generated`，模擬自動產生路徑。
///
/// 不走 `DocumentService::create`：`system_generated` 刻意不在任何 request DTO 內
/// （否則具 create+approve 權限者可自行標記以繞過守衛），只有 service 內部
/// 的自動產生路徑會寫入，故測試也直接寫 DB。
async fn seed_adj(pool: &PgPool, wh: Uuid, product: Uuid, shelf: Uuid, system: bool) -> Uuid {
    let id = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, \
                                requires_manager_approval, system_generated, created_by) \
         VALUES ($1, 'ADJ', $2, 'submitted', $3, CURRENT_DATE, false, $4, $5)",
    )
    .bind(id)
    .bind(format!("ADJ-SG-{}", &s[..8]))
    .bind(wh)
    .bind(system)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed ADJ");
    sqlx::query(
        "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom, storage_location_id) \
         VALUES ($1, $2, 1, $3, 1, 'pcs', $4)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(product)
    .bind(shelf)
    .execute(pool)
    .await
    .expect("seed ADJ line");
    id
}

/// 系統自動產生的 ADJ：建立者可自核（恢復盤點流程原本的設計）。
#[tokio::test]
#[serial]
async fn system_generated_adj_allows_self_approval() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;

    let adj_id = seed_adj(&pool, wh, product, shelf, true).await;

    // creator() 的 id 即 seed 時的 created_by —— 正是自核情境。
    DocumentService::approve(&pool, &creator(), adj_id)
        .await
        .expect("系統自動產生的 ADJ 應可由建立者核准（盤點流程仰賴此行為）");
}

/// 對照組：同樣條件但非系統產生 → 守衛仍必須擋下。
///
/// 這條是本組的重點——證明豁免只對 `system_generated` 生效，
/// 不是把 ADJ 的守衛整個關掉。
#[tokio::test]
#[serial]
async fn human_created_adj_still_blocks_self_approval() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;

    let adj_id = seed_adj(&pool, wh, product, shelf, false).await;

    let err = DocumentService::approve(&pool, &creator(), adj_id)
        .await
        .expect_err("人工開立的 ADJ 仍不得自核");
    assert!(
        err.to_string().contains("職務分離"),
        "應因職務分離被擋，實際：{err}"
    );
}

/// `system_generated` 一經寫入即不可變更（migration 152 的 DB trigger）。
///
/// 這條鎖住的是豁免機制本身的完整性：旗標若能事後翻轉，具 create+approve
/// 權限者只要把自己的單標成系統產生就能自核，整道守衛形同虛設
/// （Qodo 於 PR #144 指出「若有任何路徑能設定或事後改動它」）。
/// 設計上它不在任何 request DTO 內，但那只擋得住現在——DB trigger 才是強制。
#[tokio::test]
#[serial]
async fn system_generated_flag_cannot_be_flipped_after_insert() {
    let pool = setup_pool().await;
    let wh = seed_warehouse(&pool).await;
    let shelf = seed_shelf(&pool, wh).await;
    let product = seed_product(&pool, None).await;

    let human = seed_adj(&pool, wh, product, shelf, false).await;
    let system = seed_adj(&pool, wh, product, shelf, true).await;

    // 人工單想翻成系統單 → 擋（這是實際的攻擊方向：自己給自己豁免）。
    let up = sqlx::query("UPDATE documents SET system_generated = true WHERE id = $1")
        .bind(human)
        .execute(&pool)
        .await
        .expect_err("不得把人工單翻成系統產生");
    assert!(
        up.to_string().contains("不可變更"),
        "應被 trigger 擋下，實際：{up}"
    );

    // 反方向同樣不可，避免用「先標再翻回」規避稽核。
    let down = sqlx::query("UPDATE documents SET system_generated = false WHERE id = $1")
        .bind(system)
        .execute(&pool)
        .await
        .expect_err("不得把系統單翻回人工");
    assert!(
        down.to_string().contains("不可變更"),
        "應被 trigger 擋下，實際：{down}"
    );

    // 不碰該欄位的一般 UPDATE 必須照常——否則核准流程會被這道 trigger 打壞。
    sqlx::query("UPDATE documents SET remark = 'immutability smoke' WHERE id = $1")
        .bind(human)
        .execute(&pool)
        .await
        .expect("未變更 system_generated 的 UPDATE 應通過");
}
