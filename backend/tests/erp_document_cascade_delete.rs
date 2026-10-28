//! Regression：刪除單據時連動處理下游引用單據（source_doc_id）。
//!
//! PO 被 GRN 以 `source_doc_id` 引用（FK `documents_source_doc_id_fkey` = NO ACTION）。
//! 使用者 2026-07-18 決策：
//!   - 草稿下游 → 隨主單一起連動刪除（各自寫 audit）。
//!   - 有任何非草稿（已核准等）下游 → 擋下刪除（AppError::BusinessRule）。
//!
//! 無此連動邏輯前，硬刪一張被 GRN 引用的 PO 會直接撞 FK 違反 →
//! 使用者只看到通用「資料庫操作失敗」，且無法清掉早期系統未完善時建立的殘單。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
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

/// admin + 倉管 actor。硬刪由 handler 的 is_admin 閘控制、service 直接收 is_hard，
/// delete 服務不檢查角色；WAREHOUSE_MANAGER 供 approve() 用（庫存 guard 測試）。
fn actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "test-admin@example.com".into(),
        roles: vec!["admin".into(), "WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// R97-1：GRN 禁止自核，而 `seed_doc` 一律以 `SYSTEM_USER_ID` 當建單者。
/// 前置入庫改由這位非建單者核准；本檔測的是刪除連動，誰核准與待測行為無關。
async fn approver_actor(pool: &PgPool) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: test_db::seed_other_user(pool, "approver-cascade").await,
        email: "test-approver@example.com".into(),
        roles: vec!["admin".into(), "WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test-approver".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 建一張已核准 PO + `n` 張以 `source_doc_id` 引用它的 GRN（指定狀態）。
/// 每張 GRN 含 1 行明細。回傳 `(po_id, grn_ids)`。`grn_status` 透過 `$::doc_status` cast 綁定。
async fn seed_po_with_grns(pool: &PgPool, grn_status: &str, n: usize) -> (Uuid, Vec<Uuid>) {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh_id = Uuid::new_v4();
    let prod_id = Uuid::new_v4();
    let po_id = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh_id)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");

    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, $3)")
        .bind(prod_id)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");

    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by) \
         VALUES ($1, 'PO', $2, 'approved', $3, CURRENT_DATE, $4)",
    )
    .bind(po_id)
    .bind(format!("PO-T-{}", &suffix[..10]))
    .bind(wh_id)
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed PO");

    let mut grn_ids = Vec::with_capacity(n);
    for i in 0..n {
        let gid = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO documents \
             (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, source_doc_id) \
             VALUES ($1, 'GRN', $2, $3::doc_status, $4, CURRENT_DATE, $5, $6)",
        )
        .bind(gid)
        .bind(format!("GRN-T-{}-{}", &suffix[..8], i))
        .bind(grn_status)
        .bind(wh_id)
        .bind(SYSTEM_USER_ID)
        .bind(po_id)
        .execute(pool)
        .await
        .expect("seed GRN");

        sqlx::query(
            "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom, unit_price) \
             VALUES ($1, $2, 1, $3, 1, 'pcs', NULL)",
        )
        .bind(Uuid::new_v4())
        .bind(gid)
        .bind(prod_id)
        .execute(pool)
        .await
        .expect("seed GRN line");

        grn_ids.push(gid);
    }
    (po_id, grn_ids)
}

async fn doc_exists(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("query document exists")
}

#[tokio::test]
async fn delete_po_cascades_draft_grns() {
    let pool = setup_pool().await;
    let (po_id, grn_ids) = seed_po_with_grns(&pool, "draft", 2).await;

    // 修復前：此呼叫會撞 FK 違反 → AppError::Database（RED）
    DocumentService::delete(&pool, &actor(), po_id, true)
        .await
        .expect("硬刪 PO（下游皆草稿）應成功並連動刪除 GRN");

    assert!(!doc_exists(&pool, po_id).await, "PO 應已刪除");

    for gid in &grn_ids {
        assert!(!doc_exists(&pool, *gid).await, "草稿 GRN 應連動刪除");
        let lines: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM document_lines WHERE document_id = $1")
                .bind(gid)
                .fetch_one(&pool)
                .await
                .expect("count grn lines");
        assert_eq!(lines, 0, "連動刪的 GRN 明細應一併清除");
    }

    // 「要放紀錄」：PO 寫 DOC_HARD_DELETE、每張 GRN 寫 DOC_DELETE 到 user_activity_logs
    let po_audit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE entity_id::text = $1::text AND event_type = 'DOC_HARD_DELETE'",
    )
    .bind(po_id)
    .fetch_one(&pool)
    .await
    .expect("count po audit");
    assert_eq!(po_audit, 1, "PO 硬刪應寫入 1 筆 DOC_HARD_DELETE audit");

    for gid in &grn_ids {
        let g_audit: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_activity_logs \
             WHERE entity_id::text = $1::text AND event_type = 'DOC_DELETE'",
        )
        .bind(gid)
        .fetch_one(&pool)
        .await
        .expect("count grn audit");
        assert_eq!(g_audit, 1, "連動刪的 GRN 應寫入 1 筆 DOC_DELETE audit");
    }
}

#[tokio::test]
async fn delete_po_blocked_by_approved_grn() {
    let pool = setup_pool().await;
    let (po_id, _grn_ids) = seed_po_with_grns(&pool, "approved", 1).await;

    let err = DocumentService::delete(&pool, &actor(), po_id, true)
        .await
        .expect_err("有已核准下游 GRN 時應擋下刪除");
    assert!(
        err.to_string().contains("無法刪除"),
        "應回 BusinessRule『無法刪除』，實際：{err}"
    );

    assert!(doc_exists(&pool, po_id).await, "被擋下時 PO 不應被刪除");
}

/// 被沖銷過的單據硬刪時，沖銷單連帶刪除（使用者 2026-10-28 決策）。
///
/// 沖銷單以 `reverses_doc_id`（FK `documents_reverses_doc_id_fkey` = NO ACTION）指向原單，
/// 不帶 `source_doc_id`。收集下游若只沿 `source_doc_id` 走，沖銷單會被漏掉，最後
/// `DELETE FROM documents` 才撞 FK 違反：使用者只看到通用的「關聯資料不存在」，
/// 原單既刪不掉、也不知道為什麼。
///
/// 沖銷單任何狀態（待核准／已核准／已作廢）都連帶刪，且每張各寫一筆 `DOC_DELETE`、
/// 主單寫 `DOC_HARD_DELETE`。採購單的沖銷單沒有庫存／傳票效果，所以通過 `has_posted_effects`。
///
/// 這個案例每個狀態各用一張 PO；同一張 PO 掛兩張沖銷單的鏈見下一個案例。
#[tokio::test]
async fn hard_delete_cascades_reversal_documents_of_every_status() {
    let pool = setup_pool().await;

    for status in ["submitted", "approved", "cancelled"] {
        let (po_id, _) = seed_po_with_grns(&pool, "draft", 0).await;
        let rev_id = Uuid::new_v4();
        let rev_no = format!("PO-REV-{}", &Uuid::new_v4().simple().to_string()[..10]);
        sqlx::query(
            "INSERT INTO documents \
             (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, reverses_doc_id) \
             SELECT $1, 'PO', $2, $3::doc_status, warehouse_id, CURRENT_DATE, $4, id \
             FROM documents WHERE id = $5",
        )
        .bind(rev_id)
        .bind(&rev_no)
        .bind(status)
        .bind(SYSTEM_USER_ID)
        .bind(po_id)
        .execute(&pool)
        .await
        .expect("seed reversal document");

        DocumentService::delete(&pool, &actor(), po_id, true)
            .await
            .unwrap_or_else(|e| panic!("沖銷單狀態 {status}：硬刪應連帶刪除沖銷單，實際：{e}"));

        assert!(
            !doc_exists(&pool, po_id).await,
            "沖銷單狀態 {status}：原單應已刪除"
        );
        assert!(
            !doc_exists(&pool, rev_id).await,
            "沖銷單狀態 {status}：沖銷單應連帶刪除"
        );

        for (entity, event) in [(po_id, "DOC_HARD_DELETE"), (rev_id, "DOC_DELETE")] {
            let n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM user_activity_logs \
                 WHERE entity_id::text = $1::text AND event_type = $2",
            )
            .bind(entity)
            .bind(event)
            .fetch_one(&pool)
            .await
            .expect("count audit");
            assert_eq!(n, 1, "沖銷單狀態 {status}：{event} 應各寫入 1 筆 audit");
        }
    }
}

/// 沖銷鏈：同一張原單先後被兩張沖銷單指向，第一張已作廢、第二張已核准。
///
/// migration 017 起只有未作廢的沖銷單佔名額，所以這種鏈是合法的真實形狀（沖銷發起後被作廢、
/// 重新發起再核准）。硬刪原單必須把兩張一起帶走，否則任何一張殘留都會撞
/// `documents_reverses_doc_id_fkey`。
#[tokio::test]
async fn hard_delete_cascades_a_cancelled_then_approved_reversal_chain() {
    let pool = setup_pool().await;
    let (po_id, _) = seed_po_with_grns(&pool, "draft", 0).await;

    let mut reversal_ids = Vec::new();
    for status in ["cancelled", "approved"] {
        let rev_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO documents \
             (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, reverses_doc_id) \
             SELECT $1, 'PO', $2, $3::doc_status, warehouse_id, CURRENT_DATE, $4, id \
             FROM documents WHERE id = $5",
        )
        .bind(rev_id)
        .bind(format!(
            "PO-REV-{}",
            &Uuid::new_v4().simple().to_string()[..10]
        ))
        .bind(status)
        .bind(SYSTEM_USER_ID)
        .bind(po_id)
        .execute(&pool)
        .await
        .expect("seed reversal document");
        reversal_ids.push(rev_id);
    }

    DocumentService::delete(&pool, &actor(), po_id, true)
        .await
        .unwrap_or_else(|e| panic!("沖銷鏈（已作廢＋已核准）硬刪應整條帶走，實際：{e}"));

    assert!(!doc_exists(&pool, po_id).await, "原單應已刪除");
    for rev_id in reversal_ids {
        assert!(!doc_exists(&pool, rev_id).await, "兩張沖銷單都應連帶刪除");
    }
}

// ── 以下為串鏈 / 庫存 guard 情境（共用可組合 helper）──────────────────

/// 共用 seed 情境：一個倉庫 + 一個品項。
struct SeedCtx {
    wh: Uuid,
    prod: Uuid,
}

async fn seed_ctx(pool: &PgPool) -> SeedCtx {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh = Uuid::new_v4();
    let prod = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試倉")
        .execute(pool)
        .await
        .expect("seed warehouse");
    sqlx::query("INSERT INTO products (id, sku, name) VALUES ($1, $2, $3)")
        .bind(prod)
        .bind(format!("SKU-{}", &suffix[..8]))
        .bind("測試品項")
        .execute(pool)
        .await
        .expect("seed product");
    SeedCtx { wh, prod }
}

/// 建一張單據（含 1 行明細，qty 5、無單價），可指定 `source_doc_id` 串成鏈。
async fn seed_doc(
    pool: &PgPool,
    ctx: &SeedCtx,
    doc_type: &str,
    status: &str,
    source: Option<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    let doc_no = format!(
        "{}-{}",
        doc_type,
        &Uuid::new_v4().simple().to_string()[..12]
    );
    sqlx::query(
        "INSERT INTO documents \
         (id, doc_type, doc_no, status, warehouse_id, doc_date, created_by, source_doc_id) \
         VALUES ($1, $2::doc_type, $3, $4::doc_status, $5, CURRENT_DATE, $6, $7)",
    )
    .bind(id)
    .bind(doc_type)
    .bind(doc_no)
    .bind(status)
    .bind(ctx.wh)
    .bind(SYSTEM_USER_ID)
    .bind(source)
    .execute(pool)
    .await
    .expect("seed doc");
    sqlx::query(
        "INSERT INTO document_lines (id, document_id, line_no, product_id, qty, uom, unit_price) \
         VALUES ($1, $2, 1, $3, 5, 'pcs', NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(ctx.prod)
    .execute(pool)
    .await
    .expect("seed doc line");
    id
}

#[tokio::test]
async fn delete_blocked_when_doc_has_stock_ledger() {
    let pool = setup_pool().await;
    let ctx = seed_ctx(&pool).await;

    // 已提交 GRN → 核准 → 寫入 stock_ledger（已入庫）
    let grn_id = seed_doc(&pool, &ctx, "GRN", "submitted", None).await;
    let approver = approver_actor(&pool).await;
    DocumentService::approve(&pool, &approver, grn_id)
        .await
        .expect("核准 GRN 應成功並寫入庫存流水");
    let ledger: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stock_ledger WHERE doc_id = $1")
        .bind(grn_id)
        .fetch_one(&pool)
        .await
        .expect("count ledger");
    assert!(ledger >= 1, "核准後應有庫存流水");

    // 硬刪應被擋：已寫庫存，刪除會孤兒化 stock_ledger、破壞庫存帳
    let err = DocumentService::delete(&pool, &actor(), grn_id, true)
        .await
        .expect_err("已寫庫存的已核准 GRN 不應可刪");
    assert!(
        err.to_string().contains("庫存"),
        "應回庫存/帳務 guard 訊息，實際：{err}"
    );
    assert!(doc_exists(&pool, grn_id).await, "被擋下時 GRN 不應被刪");
}

#[tokio::test]
async fn delete_blocked_by_nondraft_grandchild() {
    let pool = setup_pool().await;
    let ctx = seed_ctx(&pool).await;

    // A(PO,approved) → B(GRN,draft) → C(GRN,approved)：孫代非草稿
    let a = seed_doc(&pool, &ctx, "PO", "approved", None).await;
    let b = seed_doc(&pool, &ctx, "GRN", "draft", Some(a)).await;
    let c = seed_doc(&pool, &ctx, "GRN", "approved", Some(b)).await;

    // 只看直接子單會漏掉 C；刪 B 仍會撞 C 的 FK。整條鏈掃描後應乾淨擋下。
    let err = DocumentService::delete(&pool, &actor(), a, true)
        .await
        .expect_err("下游鏈含非草稿孫代時應擋下（非 DB 錯誤）");
    assert!(
        err.to_string().contains("無法刪除"),
        "應回連動擋刪訊息，實際：{err}"
    );
    assert!(
        doc_exists(&pool, a).await && doc_exists(&pool, b).await && doc_exists(&pool, c).await,
        "被擋下時整條鏈都應保留"
    );
}

#[tokio::test]
async fn delete_cascades_multilevel_draft_chain() {
    let pool = setup_pool().await;
    let ctx = seed_ctx(&pool).await;

    // A(PO,approved) → B(GRN,draft) → C(GRN,draft)：整條草稿鏈連動刪
    let a = seed_doc(&pool, &ctx, "PO", "approved", None).await;
    let b = seed_doc(&pool, &ctx, "GRN", "draft", Some(a)).await;
    let c = seed_doc(&pool, &ctx, "GRN", "draft", Some(b)).await;

    DocumentService::delete(&pool, &actor(), a, true)
        .await
        .expect("整條草稿下游鏈應連動刪除");
    assert!(!doc_exists(&pool, a).await, "A 應刪除");
    assert!(!doc_exists(&pool, b).await, "B 應連動刪除");
    assert!(!doc_exists(&pool, c).await, "C（孫代草稿）應連動刪除");
}
