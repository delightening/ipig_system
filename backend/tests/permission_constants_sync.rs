//! CI 守衛：`frontend/src/lib/permissions.generated.ts` 必須與後端 `permissions` 表同步。
//!
//! 沒有這道守衛的話，新增後端權限而忘了重新產生 TS 常數 → 前端無法用常數表達那個權限
//! → 開發者退回手打字串 → 回到「兩邊靠人眼對齊」的原點。
//!
//! 見 `docs/audit/button-permission-gate-2026-08-07.md` §7-2。

use sqlx::PgPool;

use erp_backend::services::permission_codegen;

/// 取測試 DB 連線字串，**拒絕連上疑似正式環境的資料庫**。
///
/// 本測試雖然只讀 `permissions`，但會先跑 migration 與 `ensure_required_permissions`，
/// 兩者都會寫入資料庫 —— 連錯 DB 的代價是對正式 schema 動手。
///
/// 規則：優先 `TEST_DATABASE_URL`，未設時才看 `DATABASE_URL`；
/// **不論來自哪一個變數，都必須通過同一道「database 名稱含 `test`」檢查**。
///
/// 為什麼檢查要套用在兩個來源上：`TEST_DATABASE_URL` 這個名字本身不保證任何事，
/// 有人把它指向 prod 一樣會中。守衛該守的是「連到哪個 DB」，不是「用了哪個變數名」。
///
/// 為什麼保留 `DATABASE_URL` fallback：CI 只設 `DATABASE_URL`
/// （指向 runner 自己的丟棄庫 `ipig_db_test`），硬性只認 `TEST_DATABASE_URL`
/// 會讓 CI 全紅；要在 CI 補環境變數得改 `.github/workflows/*`，
/// 屬需使用者授權的項目。名稱檢查對兩個來源都生效後，fallback 不弱化防護。
fn test_database_url() -> String {
    let (url, source) = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => (url, "TEST_DATABASE_URL"),
        Err(_) => (
            std::env::var("DATABASE_URL").expect(
                "TEST_DATABASE_URL 與 DATABASE_URL 皆未設定。本測試會寫入資料庫，\
                 請指向獨立可丟棄的測試 DB。",
            ),
            "DATABASE_URL",
        ),
    };
    let db_name = url
        .rsplit('/')
        .next()
        .and_then(|tail| tail.split(['?', '#']).next())
        .unwrap_or_default();
    assert!(
        db_name.contains("test"),
        "{source} 指向的資料庫 `{db_name}` 不像測試庫（名稱不含 test）。\
         本測試會跑 migration，拒絕在可能是 prod 的連線上執行。\
         請指向獨立可丟棄的測試 DB。"
    );
    url
}

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let url = test_database_url();
    let pool = PgPool::connect(&url).await.expect("connect test db");
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
