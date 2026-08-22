//! Regression：倉庫底下仍有結存時不得停用（2026-08-05 prod 事件）。
//!
//! Bug：`WarehouseService::delete` 只做 `UPDATE warehouses SET is_active = false`，
//! 不看底下儲位還有沒有庫存。倉庫一停用，庫存查詢樹（`list_with_shelves`）與
//! 現況報表（`get_report_data`）都只撈 `is_active = true` 的倉庫 → 底下的結存從
//! 所有畫面消失，但 `stock_ledger` 帳上仍在（隱形庫存）。
//! prod 實例：倉庫「儲藏室 (2)」被停用時，底下 4 個儲位尚有 4,424 單位。
//!
//! 修復：停用前檢查結存，有就回 422 BusinessRule 並列出儲位。
//! 「編輯 → 啟用狀態關掉」與「刪除」是同一條 is_active 路徑，兩邊都受閘門管。

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::UpdateWarehouseRequest;
use erp_backend::services::audit::RequestContext;
use erp_backend::services::WarehouseService;
use erp_backend::AppError;

#[path = "common/test_db.rs"]
mod test_db;

const TEST_IP: &str = "203.0.113.9";
const TEST_UA: &str = "warehouse-guard-test/1.0";

/// **fail-closed**：見 `tests/common/test_db.rs`。
///
/// 本測試會對目標庫寫入 users / warehouses / products / storage_locations /
/// storage_location_inventory 與稽核列，且**沒有 teardown**——誤指向 prod 的代價很高。
/// 護欄只讀 `TEST_DATABASE_URL`（不 fallback，開發機那條指向 prod），並在 migration
/// 與任何寫入之前驗證目標資料庫**本身**是丟棄用測試庫，不是靠名字判斷。
/// 本檔早期版本用「DSN 是否含 `test`」的字串啟發式，已由該護欄取代。
async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 建一筆真實 user row 供 audit `actor_user_id` FK。
async fn seed_actor(pool: &PgPool) -> ActorContext {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', 'wh guard actor', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("wh-guard-{}@test.local", &id.to_string()[..8]))
    .execute(pool)
    .await
    .expect("seed actor user");

    ActorContext::User(CurrentUser {
        id,
        email: format!("wh-guard-{}@test.local", &id.to_string()[..8]),
        roles: vec!["ADMIN".to_string()],
        permissions: vec!["erp.warehouse.delete".to_string()],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 建倉庫 + 一個儲位 + 一筆結存，回傳 (warehouse_id, storage_location_id)。
async fn seed_warehouse_with_stock(pool: &PgPool, qty: Decimal) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let wh = Uuid::new_v4();
    let sl = Uuid::new_v4();
    let prod = Uuid::new_v4();

    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(wh)
        .bind(format!("WH-{}", &suffix[..8]))
        .bind("守衛測試倉")
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

    sqlx::query(
        "INSERT INTO storage_locations \
         (id, warehouse_id, code, name, location_type, row_index, col_index, width, height, is_active) \
         VALUES ($1, $2, $3, '儲物架A', 'shelf', 0, 0, 1, 1, true)",
    )
    .bind(sl)
    .bind(wh)
    .bind(format!("SL-{}", &suffix[..8]))
    .execute(pool)
    .await
    .expect("seed storage_location");

    sqlx::query(
        "INSERT INTO storage_location_inventory \
         (id, storage_location_id, product_id, on_hand_qty, updated_at) \
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(sl)
    .bind(prod)
    .bind(qty)
    .execute(pool)
    .await
    .expect("seed storage_location_inventory");

    (wh, sl)
}

async fn is_active(pool: &PgPool, wh: Uuid) -> bool {
    sqlx::query_scalar("SELECT is_active FROM warehouses WHERE id = $1")
        .bind(wh)
        .fetch_one(pool)
        .await
        .expect("read warehouse is_active")
}

fn request_ctx() -> RequestContext<'static> {
    RequestContext {
        ip_address: Some(TEST_IP),
        user_agent: Some(TEST_UA),
    }
}

/// 有結存 → 刪除（軟刪）被擋，倉庫維持啟用。
#[tokio::test]
async fn delete_warehouse_blocked_when_stock_remains() {
    let pool = setup_pool().await;
    let actor = seed_actor(&pool).await;
    let (wh, _sl) = seed_warehouse_with_stock(&pool, Decimal::new(4424, 0)).await;

    // 修復前：這裡會 Ok(())，倉庫連同 4,424 單位一起從畫面消失
    let err = WarehouseService::delete(&pool, &actor, wh, Some(request_ctx()))
        .await
        .expect_err("底下仍有結存時不應允許停用倉庫");

    match err {
        AppError::BusinessRule(msg) => {
            assert!(
                msg.contains("無法停用"),
                "錯誤訊息應說明無法停用，實際：{msg}"
            );
            assert!(
                msg.contains("儲物架A"),
                "錯誤訊息應列出仍有結存的儲位，實際：{msg}"
            );
            assert!(msg.contains("4424"), "錯誤訊息應帶出結存數量，實際：{msg}");
        }
        other => panic!("應回 422 BusinessRule，實際：{other:?}"),
    }

    assert!(is_active(&pool, wh).await, "被擋下後倉庫仍應維持啟用");
}

/// 「編輯 → 啟用狀態關掉」與刪除是同一條路徑，同樣要被擋。
#[tokio::test]
async fn update_to_inactive_blocked_when_stock_remains() {
    let pool = setup_pool().await;
    let actor = seed_actor(&pool).await;
    let (wh, _sl) = seed_warehouse_with_stock(&pool, Decimal::new(60, 0)).await;

    let req = UpdateWarehouseRequest {
        name: None,
        address: None,
        is_active: Some(false),
    };
    let err = WarehouseService::update(&pool, &actor, wh, &req, Some(request_ctx()))
        .await
        .expect_err("編輯把啟用狀態關掉，等同停用，應同樣被擋");

    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應回 422 BusinessRule，實際：{err:?}"
    );
    assert!(is_active(&pool, wh).await, "被擋下後倉庫仍應維持啟用");
}

/// 結存清空後可正常停用，且稽核紀錄帶得到 IP / User-Agent。
#[tokio::test]
async fn delete_succeeds_when_empty_and_records_request_context() {
    let pool = setup_pool().await;
    let actor = seed_actor(&pool).await;
    let (wh, sl) = seed_warehouse_with_stock(&pool, Decimal::new(5, 0)).await;

    sqlx::query("DELETE FROM storage_location_inventory WHERE storage_location_id = $1")
        .bind(sl)
        .execute(&pool)
        .await
        .expect("清空結存");

    WarehouseService::delete(&pool, &actor, wh, Some(request_ctx()))
        .await
        .expect("無結存時應可停用");

    assert!(!is_active(&pool, wh).await, "停用後 is_active 應為 false");

    // 修復前 delete_tx 傳 request_context: None → 稽核只記得誰做的，記不到從哪裡做的。
    // 欄位是 inet，`::text` 會帶出 /32 遮罩，故用 host() 取純位址。
    let (ip, ua): (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT host(ip_address), user_agent FROM user_activity_logs \
         WHERE event_type = 'WAREHOUSE_DELETE' AND entity_id = $1",
    )
    .bind(wh)
    .fetch_one(&pool)
    .await
    .expect("應有一筆 WAREHOUSE_DELETE 稽核");

    assert_eq!(ip.as_deref(), Some(TEST_IP), "稽核應記下來源 IP");
    assert_eq!(ua.as_deref(), Some(TEST_UA), "稽核應記下 User-Agent");
}
