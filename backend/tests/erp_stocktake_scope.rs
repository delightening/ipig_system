//! 整合測試：盤點單的「盤點範圍」(`stocktake_scope`) 是否真的過濾底稿。
//!
//! 情境來源：準備室同時放藥品與耗材，但只有藥品需要逐項盤點。
//! 後端早就支援 `StocktakeScope.category_codes`，前端在本次改動才把欄位接上，
//! 因此這條路徑過去從未被實際走過——本檔補上端到端的證據（走公開的
//! `DocumentService::create`，與前端送出的 JSON 形狀一致）。
//!
//! 同時鎖住一個刻意的行為改變：範圍 JSON 形狀錯誤時**必須報錯**，
//! 不可沿用舊的 `.ok()` 靜默降級成全盤（使用者會盤完才發現篩選被丟掉）。

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::models::{CreateDocumentRequest, DocType};
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

/// 倉庫管理員 actor。借用 migration 033 的 SYSTEM user 作 FK 目標（同
/// `erp_stocktake_reconciliation.rs` 的既有做法）。
fn wm_actor() -> ActorContext {
    ActorContext::User(CurrentUser {
        id: SYSTEM_USER_ID,
        email: "test-wm@example.com".into(),
        roles: vec!["WAREHOUSE_MANAGER".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 一個倉、一個貨架，上面放兩支不同品類的品項（各有現存量）。
/// 回傳 (warehouse_id, 藥品 sku, 耗材 sku)。
async fn seed_two_category_shelf(pool: &PgPool) -> (Uuid, String, String) {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh_id = Uuid::new_v4();
    let loc_id = Uuid::new_v4();
    let drug_id = Uuid::new_v4();
    let consumable_id = Uuid::new_v4();
    let drug_sku = format!("DRG-{}", &suffix[..8]);
    let consumable_sku = format!("CON-{}", &suffix[..8]);

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh_id)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("測試準備室")
        .execute(pool)
        .await
        .expect("seed warehouse");

    sqlx::query(
        "INSERT INTO storage_locations (id, warehouse_id, code, name, is_active, current_count) \
         VALUES ($1, $2, $3, $4, true, 2)",
    )
    .bind(loc_id)
    .bind(wh_id)
    .bind(format!("L-{}", &suffix[..8]))
    .bind("測試貨架")
    .execute(pool)
    .await
    .expect("seed storage_location");

    // category_code 用真值 DRG / CON（見 sku_categories 表），不自造代碼。
    for (id, sku, name, category) in [
        (drug_id, &drug_sku, "測試藥品", "DRG"),
        (consumable_id, &consumable_sku, "測試耗材", "CON"),
    ] {
        sqlx::query("INSERT INTO products (id, sku, name, category_code) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(sku)
            .bind(name)
            .bind(category)
            .execute(pool)
            .await
            .expect("seed product");

        sqlx::query(
            "INSERT INTO storage_location_inventory \
             (id, storage_location_id, product_id, on_hand_qty, updated_at) \
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(loc_id)
        .bind(id)
        .bind(Decimal::new(10, 0))
        .execute(pool)
        .await
        .expect("seed shelf inventory");
    }

    (wh_id, drug_sku, consumable_sku)
}

fn stk_request(wh_id: Uuid, scope: Option<serde_json::Value>) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type: DocType::STK,
        warehouse_id: Some(wh_id),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        source_doc_id: None,
        doc_date: NaiveDate::from_ymd_opt(2026, 8, 27).expect("valid date"),
        remark: None,
        stocktake_scope: scope,
        iacuc_no: None,
        protocol_id: None,
        lines: vec![],
    }
}

/// 建一張 STK 並回傳底稿裡出現的 SKU（排序後）。
async fn draft_skus(pool: &PgPool, wh_id: Uuid, scope: Option<serde_json::Value>) -> Vec<String> {
    let doc = DocumentService::create(pool, &wm_actor(), &stk_request(wh_id, scope))
        .await
        .expect("建立盤點單應成功");
    let mut skus: Vec<String> = doc.lines.iter().map(|l| l.product_sku.clone()).collect();
    skus.sort();
    skus
}

#[tokio::test]
async fn scope_with_category_filter_lists_only_that_category() {
    let pool = setup_pool().await;
    let (wh_id, drug_sku, consumable_sku) = seed_two_category_shelf(&pool).await;

    let scope = serde_json::json!({
        "scope_type": "partial",
        "category_codes": ["DRG"],
    });
    let skus = draft_skus(&pool, wh_id, Some(scope)).await;

    assert_eq!(
        skus,
        vec![drug_sku],
        "限定 DRG 時底稿只應出現藥品，耗材 {consumable_sku} 不該被列入"
    );
}

#[tokio::test]
async fn scope_without_category_filter_lists_everything() {
    let pool = setup_pool().await;
    let (wh_id, drug_sku, consumable_sku) = seed_two_category_shelf(&pool).await;

    // 前端「不選任何品類」送出的形狀：scope_type=full、category_codes 為空陣列
    let scope = serde_json::json!({
        "scope_type": "full",
        "category_codes": [],
    });
    let skus = draft_skus(&pool, wh_id, Some(scope)).await;

    let mut expected = vec![drug_sku, consumable_sku];
    expected.sort();
    assert_eq!(skus, expected, "空的品類清單等同全盤");
}

#[tokio::test]
async fn absent_scope_lists_everything() {
    let pool = setup_pool().await;
    let (wh_id, drug_sku, consumable_sku) = seed_two_category_shelf(&pool).await;

    let skus = draft_skus(&pool, wh_id, None).await;

    let mut expected = vec![drug_sku, consumable_sku];
    expected.sort();
    assert_eq!(skus, expected, "完全沒帶範圍時維持既有的全盤行為");
}

#[tokio::test]
async fn null_scope_lists_everything() {
    let pool = setup_pool().await;
    let (wh_id, drug_sku, consumable_sku) = seed_two_category_shelf(&pool).await;

    // JSON null 是明確的「不限範圍」，與形狀錯誤不同，不該報錯
    let skus = draft_skus(&pool, wh_id, Some(serde_json::Value::Null)).await;

    let mut expected = vec![drug_sku, consumable_sku];
    expected.sort();
    assert_eq!(skus, expected, "null 範圍應視為全盤而非錯誤");
}

#[tokio::test]
async fn malformed_scope_is_rejected_not_silently_full_counted() {
    let pool = setup_pool().await;
    let (wh_id, _, _) = seed_two_category_shelf(&pool).await;

    // 缺必填的 scope_type；舊版 `.ok()` 會把它吞掉並改成全盤
    let scope = serde_json::json!({ "category_codes": ["DRG"] });

    let err = DocumentService::create(&pool, &wm_actor(), &stk_request(wh_id, Some(scope)))
        .await
        .expect_err("形狀錯誤的盤點範圍必須報錯，不可靜默全盤");

    let msg = err.to_string();
    assert!(
        msg.contains("盤點範圍格式錯誤"),
        "錯誤訊息應指出是盤點範圍的問題，實際為：{msg}"
    );
}

/// 以下三案鎖住 `validate_scope`——形狀合法但**自相矛盾**的範圍。
///
/// 與上面那案的差別：那案是 JSON 反序列化就失敗（缺必填欄位），這三案 serde 完全解得開，
/// 錯的是意思。`generate_stocktake_lines` 從不讀 `scope_type`，所以在加上驗證之前，
/// 這三種 payload 都會安靜地產生一份與請求不符的底稿。
/// （CodeRabbit 於 PR #37 指出第一案，Minor；後兩案是查證時一併發現的同族問題。）
#[tokio::test]
async fn full_scope_with_category_filter_is_rejected() {
    let pool = setup_pool().await;
    let (wh_id, _, _) = seed_two_category_shelf(&pool).await;

    // 宣告全盤卻又給篩選：驗證前會產生**部分**底稿，使用者以為自己全盤了
    let scope = serde_json::json!({
        "scope_type": "full",
        "category_codes": ["DRG"],
    });

    let err = DocumentService::create(&pool, &wm_actor(), &stk_request(wh_id, Some(scope)))
        .await
        .expect_err("full + 非空篩選是矛盾的範圍，必須報錯");

    let msg = err.to_string();
    assert!(
        msg.contains("不可同時指定"),
        "錯誤訊息應點出 full 與篩選並存的矛盾，實際為：{msg}"
    );
}

#[tokio::test]
async fn unknown_scope_type_is_rejected() {
    let pool = setup_pool().await;
    let (wh_id, _, _) = seed_two_category_shelf(&pool).await;

    // `fulll` 是 typo。若只檢查「等不等於 full」，它會被當成 partial 而套用篩選——
    // 使用者本意全盤，拿到的卻是只有藥品的底稿。
    let scope = serde_json::json!({
        "scope_type": "fulll",
        "category_codes": ["DRG"],
    });

    let err = DocumentService::create(&pool, &wm_actor(), &stk_request(wh_id, Some(scope)))
        .await
        .expect_err("未知的 scope_type 必須報錯，不可當成 partial 放行");

    let msg = err.to_string();
    assert!(
        msg.contains("scope_type"),
        "錯誤訊息應點出是 scope_type 的值有問題，實際為：{msg}"
    );
}

#[tokio::test]
async fn warehouse_ids_in_scope_is_rejected_because_unimplemented() {
    let pool = setup_pool().await;
    let (wh_id, _, _) = seed_two_category_shelf(&pool).await;

    // `warehouse_ids` 從未被實作：底稿只依單據本身的 warehouse_id 產生。
    // 帶了卻被靜默忽略 = 呼叫端以為指定了跨倉範圍，實際拿到單倉底稿。
    let scope = serde_json::json!({
        "scope_type": "partial",
        "warehouse_ids": [Uuid::new_v4()],
    });

    let err = DocumentService::create(&pool, &wm_actor(), &stk_request(wh_id, Some(scope)))
        .await
        .expect_err("尚未支援的 warehouse_ids 必須報錯，不可靜默忽略");

    let msg = err.to_string();
    assert!(
        msg.contains("warehouse_ids"),
        "錯誤訊息應點名 warehouse_ids，實際為：{msg}"
    );
}
