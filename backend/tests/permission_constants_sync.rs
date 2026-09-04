//! CI 守衛：`frontend/src/lib/permissions.generated.ts` 必須與後端 `permissions` 表同步。
//!
//! 沒有這道守衛的話，新增後端權限而忘了重新產生 TS 常數 → 前端無法用常數表達那個權限
//! → 開發者退回手打字串 → 回到「兩邊靠人眼對齊」的原點。
//!
//! 見 `docs/audit/button-permission-gate-2026-08-07.md` §7-2。

use sqlx::PgPool;

use erp_backend::services::permission_codegen;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    // migration seed 之外，還有一批權限是啟動時才補的 —— 兩者聯集才是真相源。
    erp_backend::startup::ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    pool
}

#[tokio::test]
async fn generated_permission_constants_match_backend() {
    let pool = setup_pool().await;

    let codes = permission_codegen::fetch_permission_codes(&pool)
        .await
        .expect("fetch permission codes");

    // 先擋撞名：兩個不同權限碼映到同一個 TS 常數名時，物件字面值會靜默覆蓋，
    // 產出的常數會指向錯誤的權限字串。
    let mut keys: Vec<String> = codes
        .iter()
        .map(|c| c.to_uppercase().replace(['.', '-'], "_"))
        .collect();
    let before = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(
        keys.len(),
        before,
        "有兩個以上的權限碼會映射到同一個 TS 常數名（. 與 _ 都會轉成 _）。\
         請改掉其中一個權限碼，或調整 to_const_key 的規則。"
    );

    let expected = permission_codegen::render_ts(&codes);
    let path = permission_codegen::generated_ts_path();
    let actual = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", path.display()));

    // 行尾差異（CRLF/LF）不該讓這道守衛紅 —— 它要守的是內容，不是 checkout 設定。
    let norm = |s: &str| s.replace("\r\n", "\n");
    assert_eq!(
        norm(&actual),
        norm(&expected),
        "\n\n{} 與後端 permissions 表不同步（後端目前有 {} 個權限碼）。\n\
         重新產生：cd backend && cargo run --bin gen_permission_constants\n",
        path.display(),
        codes.len()
    );
}
