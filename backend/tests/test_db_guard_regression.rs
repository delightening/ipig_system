//! 回歸測試：鎖住 `common/test_db.rs` 護欄本身的行為（R84-15 驗收條件 ③：
//! 「補回歸測試鎖住此行為（未設 / 指向 prod 兩種情境皆須 fail）」）。
//!
//! 在此之前，這道護欄的正確性只靠人工用拋棄式 scratch DB 逐次手動驗證
//! （2026-08-08/09 兩輪 review 皆如此）——每次改動都要重新手動確認，
//! 且下一個人接手時無從得知這些情境曾經被驗過。本檔把那些手動步驟固化。
//!
//! # 為什麼測公開 API 而不是內部函式
//!
//! `assert_disposable` / `marker_matches_this_database` 等對本檔皆為**私有**：
//! `#[path = "common/test_db.rs"]` 把該檔內容併成本檔的子模組，子模組的私有
//! 項目不會暴露給外層（Rust 的模組可見性是「同模組與其後代」，本檔是子模組的
//! 祖先，看不到子模組的私有項）。改測公開 API `connect_disposable` 剛好也是
//! 對的測法：測「觀察得到的行為」，不測實作細節，未來重構內部邏輯不必跟著改測試。

#[path = "common/test_db.rs"]
mod test_db;

use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// 從既有的 `TEST_DATABASE_URL` 衍生「同一台 server、不同資料庫名」的 DSN。
///
/// 這樣本檔建立的情境資料庫，帳號/密碼/host/port 永遠與呼叫端本來就通得過的
/// 那組一致，不必另外硬編一組本機才有效的連線資訊（CI 的 postgres service
/// container 帳密與本機不同）。
fn base_url() -> String {
    std::env::var("TEST_DATABASE_URL").expect(
        "本回歸測試仍需要 TEST_DATABASE_URL 指向一個可用的 postgres server\
         （用來建立/清除情境資料庫；該資料庫本身不會被護欄寫入，護欄只會寫入\
         下面各測試臨時建立的 scratch 資料庫）。",
    )
}

fn with_db_name(base: &str, db_name: &str) -> String {
    // 先切掉 query string 再找路徑分隔符：舊版對**整個** URL 找最後一個 `/`，
    // 若 query 含路徑型參數（例如 `?sslrootcert=/certs/root.pem`）就會找到
    // query 裡的斜線，而不是資料庫名前面那個，組出壞掉的 URL
    // （CodeRabbit 於本 PR 指出；本 repo 目前用的 DSN 皆無此類 query，
    // 屬潛伏未觸發但確實存在的 bug，已手動推演確認）。
    let (path, query) = base
        .split_once('?')
        .map_or((base, ""), |(p, _)| (p, &base[p.len()..]));
    let idx = path
        .rfind('/')
        .expect("TEST_DATABASE_URL 應含路徑分隔的資料庫名");
    let (prefix, _) = path.split_at(idx + 1);
    format!("{prefix}{db_name}{query}")
}

/// 純字串邏輯，不需要資料庫連線：鎖住 CodeRabbit 於本 PR 指出的邊界情況
/// ——query string 含路徑型參數（`/` 出現在 query 裡）時，資料庫名不能被
/// 誤植進 query 內容。
#[test]
fn with_db_name_ignores_slash_in_query_string() {
    let base = "postgres://user:pass@host:5432/mydb?sslrootcert=/certs/root.pem";
    let got = with_db_name(base, "scratch_db");
    assert_eq!(
        got, "postgres://user:pass@host:5432/scratch_db?sslrootcert=/certs/root.pem",
        "query string 裡的 `/` 不該被誤判成資料庫名前的路徑分隔符"
    );
}

#[test]
fn with_db_name_without_query_string() {
    let base = "postgres://user:pass@host:5432/mydb";
    let got = with_db_name(base, "scratch_db");
    assert_eq!(got, "postgres://user:pass@host:5432/scratch_db");
}

async fn admin_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&base_url())
        .await
        .expect("connect admin pool on base TEST_DATABASE_URL server")
}

/// 建一顆全新的空 scratch 資料庫（若同名已存在先丟棄——上次測試中斷的殘留）。
///
/// `DROP/CREATE DATABASE` 的資料庫名是識別字，PostgreSQL 不支援對識別字用 bind
/// 參數（`$1` 只能綁 VALUE，不能綁 IDENTIFIER）——動態組字串在這裡無法避免。
/// `sqlx::AssertSqlSafe` 是官方提供的逃生門，用於「已人工審查過不是注入」的情境
/// （sqlx 0.9 的編譯期 `SqlSafeStr` 檢查會擋下裸的 `format!` 字串）；這裡的 `name`
/// 只由固定字面前綴 + `std::process::id()` 組成，從不接受外部輸入，符合這個
/// 逃生門的正當使用情境。
async fn create_scratch_db(name: &str) {
    let admin = admin_pool().await;
    // `IF EXISTS` 已經讓「不存在」在 SQL 層變成非錯誤（成功但 0 列受影響）——
    // 所以這裡唯一還會回 Err 的情況是**真正的錯誤**（例如上次中斷的測試留下
    // 殘留連線導致無法 drop）。原本用 `.ok()` 吞掉，會讓下一行 CREATE DATABASE
    // 以「資料庫已存在」這個誤導性訊息失敗，蓋掉真正的根因（Qodo 於本 PR 指出）。
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS {name}"
    )))
    .execute(&admin)
    .await
    .expect("drop pre-existing scratch db (IF EXISTS 已讓「不存在」非錯誤，真正失敗於此代表殘留連線等問題)");
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
        .execute(&admin)
        .await
        .expect("create scratch db");
}

/// scratch 資料庫名帶當前 process id：降低多個 session 併發跑本檔時互撞的機率。
/// 不保證絕對唯一（沒有跨 session 的協調機制），但代價可接受——這些庫在測試
/// 開始時一律 `DROP ... IF EXISTS` 後重建，撞名頂多讓對方的殘留庫被清掉重來，
/// 不會有資料遺失（scratch 庫本來就不該有人依賴其內容存活）。
fn scratch_name(tag: &str) -> String {
    format!("ipig_guard_regression_{tag}_{}", std::process::id())
}

// 呼叫 `connect_disposable` 前都先 `std::env::set_var("TEST_DATABASE_URL", &url)`，
// 再包一層 `tokio::spawn(...).await`：panic 發生在 spawned task 內不會拖垮整個
// 測試 process，`JoinHandle` 回 `Err` 就是「護欄拒絕了」，回 `Ok` 就是「護欄放行」。
// 各測試直接展開這個模式（見下），不抽共用函式——回傳型別在「拒絕」與「放行」
// 兩種案例中一個要 `PgPool`、一個不需要，硬抽共用反而要多繞一層。
//
// `TEST_DATABASE_URL` 是 process 全域的環境變數，本檔每個測試都會覆寫它——
// 因此本檔全部測試都標 `#[serial]`，避免並行測試互相踩到對方設的值。

/// 暫時把 `TEST_DATABASE_URL` 清成空字串，離開作用域時無論成功或 panic
/// 都會還原成原值（Drop 在 panic unwind 過程中仍會執行）。
///
/// 兩個獨立問題疊在一起，CodeAnt / CodeRabbit / Qodo 於本 PR 各自指出：
///
/// 1. **順序**：`#[serial]` 只保證本檔測試不並行，不保證誰先跑。舊版用完
///    `remove_var` 就不管，若 `rejects_when_env_var_unset` 先跑，其他測試
///    呼叫 `base_url()` 時這個 process-global 變數已經不見了，會直接 panic；
///    若最後跑，則讓變數維持在「被清空」狀態離開這個測試 binary。
/// 2. **`remove_var` 不夠可靠**：`connect_disposable()` 內部第一步是
///    `require_test_database_url()`，它會呼叫 `dotenvy::dotenv().ok()`。
///    若工作目錄找得到含 `TEST_DATABASE_URL` 的 `.env`，`remove_var` 清空的
///    值會被 dotenvy **重新灌回**，測試結果就取決於本機是否剛好有這個檔案
///    ——不是護欄邏輯本身。改用 `set_var("", "")`：dotenvy 預設不覆蓋
///    process 中**已存在**的變數（即使是空字串），且 `require_test_database_url`
///    本來就把空字串等同未設定處理（`url.trim().is_empty()`），行為不變、
///    但不再看本機有沒有 `.env` 的臉色。
struct TestDatabaseUrlGuard {
    original: Option<String>,
}

impl TestDatabaseUrlGuard {
    fn clear() -> Self {
        let original = std::env::var("TEST_DATABASE_URL").ok();
        std::env::set_var("TEST_DATABASE_URL", "");
        Self { original }
    }
}

impl Drop for TestDatabaseUrlGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => std::env::set_var("TEST_DATABASE_URL", v),
            None => std::env::remove_var("TEST_DATABASE_URL"),
        }
    }
}

#[tokio::test]
#[serial]
async fn rejects_when_env_var_unset() {
    // 走 connect_disposable（公開 API）而非直接呼叫 require_test_database_url——
    // 後者對本檔私有（同前述模組可見性原因）。connect_disposable 內部第一步就是
    // 呼叫它，變數清空時會在真的嘗試連線之前就 panic，不需要任何資料庫存在。
    let _guard = TestDatabaseUrlGuard::clear();
    let outcome = tokio::spawn(test_db::connect_disposable(1)).await;
    assert!(
        outcome.is_err(),
        "R84-15 情境①：未設 TEST_DATABASE_URL 應該 panic 中止，卻沒有"
    );
    // _guard 在此離開作用域並還原原值——即使上面的 assert! 失敗，
    // Drop 仍會在 panic unwind 過程中執行。
}

#[tokio::test]
#[serial]
async fn rejects_populated_business_data() {
    let name = scratch_name("populated");
    create_scratch_db(&name).await;
    let url = with_db_name(&base_url(), &name);

    {
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("connect scratch db to seed business-like data");
        // 只需要 populated_probe_tables 讀得到的三張表存在且非空，
        // 不需要完整 schema——這就是護欄查詢本身要求的一切。
        sqlx::query("CREATE TABLE animals (id int)")
            .execute(&admin)
            .await
            .expect("seed: create animals table");
        sqlx::query("CREATE TABLE audit_logs (id int)")
            .execute(&admin)
            .await
            .expect("seed: create audit_logs table");
        sqlx::query("CREATE TABLE protocols (id int)")
            .execute(&admin)
            .await
            .expect("seed: create protocols table");
        sqlx::query("INSERT INTO animals VALUES (1)")
            .execute(&admin)
            .await
            .expect("seed: insert business-like row into animals");
    }

    std::env::set_var("TEST_DATABASE_URL", &url);
    let outcome = tokio::spawn(test_db::connect_disposable(3)).await;
    assert!(
        outcome.is_err(),
        "R84-15 情境②：核心業務表（animals）已有資料，看起來像正式資料庫，\
         護欄應該拒絕，卻放行了"
    );
}

#[tokio::test]
#[serial]
async fn rejects_empty_db_with_non_test_name() {
    // 刻意不含 "test" 這個子字串：模擬「完全空無一物、但根本不是測試庫」的情境
    // ——例如 prod cluster 自帶的 postgres 維護庫。這正是 2026-08-09 補上的
    // 名稱檢查第二層要擋的東西：內容判斷在這裡完全沒有訊號可用（探測表不存在、
    // 使用者資料表數為 0），名稱是唯一剩下的防線。
    let name = format!("ipig_guard_regression_prodlike_{}", std::process::id());
    assert!(
        !name.contains("test"),
        "測試前提錯誤：scratch 名稱不該含 test，否則測不到名稱檢查"
    );
    create_scratch_db(&name).await;
    let url = with_db_name(&base_url(), &name);

    std::env::set_var("TEST_DATABASE_URL", &url);
    let outcome = tokio::spawn(test_db::connect_disposable(3)).await;
    assert!(
        outcome.is_err(),
        "名稱檢查第二層：資料庫完全空無一物、但名稱不含 test，\
         護欄應該拒絕（這是唯一剩下的防線），卻放行了"
    );
}

#[tokio::test]
#[serial]
async fn accepts_empty_db_with_test_like_name() {
    // 正面案例之一：資料庫**連結構都沒有**（探測表不存在，`!probe_tables_all_exist`
    // 分支）。確保名稱檢查沒有誤傷「完全從零開始」的 bootstrap 路徑。
    let name = scratch_name("test_bootstrap_noschema");
    assert!(name.contains("test"), "測試前提錯誤：scratch 名稱應含 test");
    create_scratch_db(&name).await;
    let url = with_db_name(&base_url(), &name);

    std::env::set_var("TEST_DATABASE_URL", &url);
    let pool = tokio::spawn(test_db::connect_disposable(3))
        .await
        .expect("名稱檢查第二層：完全空、名稱含 test 的合法丟棄庫應該放行，卻被拒絕");
    drop(pool);
}

#[tokio::test]
#[serial]
async fn accepts_freshly_migrated_empty_db_with_test_like_name() {
    // 正面案例之二，也是**實際上最重要的那條路徑**：探測表齊全但全空
    // （`probe_tables_all_exist() && populated_probe_tables().is_empty()` 分支）。
    //
    // 這才是 CI 每一次都會走的流程——`backend-test` job 先跑
    // `sqlx migrate run` 建好全部結構，才執行 `cargo test`，此時核心業務表
    // 已存在但仍是空的、也還沒蓋章。上一個測試 `accepts_empty_db_with_test_like_name`
    // 覆蓋的是「連結構都沒有」，兩者是 assert_disposable 裡兩個不同的分支，
    // 沒跑過 migration 就無法真的驗到這條路徑（CodeRabbit 於本 PR 指出：
    // 原本只有前者，實際上最常踩到的這條反而零覆蓋）。
    let name = scratch_name("test_bootstrap_migrated");
    assert!(name.contains("test"), "測試前提錯誤：scratch 名稱應含 test");
    create_scratch_db(&name).await;
    let url = with_db_name(&base_url(), &name);

    {
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("connect scratch db to run migrations");
        sqlx::migrate!("./migrations")
            .run(&admin)
            .await
            .expect("run real migrations on scratch db");
    }

    std::env::set_var("TEST_DATABASE_URL", &url);
    let pool = tokio::spawn(test_db::connect_disposable(3)).await.expect(
        "名稱檢查第二層：剛 migrate 完、核心業務表齊全但全空、名稱含 test 的\
             合法丟棄庫應該放行（這是 CI 實際走的路徑），卻被拒絕",
    );
    drop(pool);
}

#[tokio::test]
#[serial]
async fn rejects_marker_forged_from_different_database() {
    // Critical 1（codeant-ai 於 #37 指出）的回歸鎖：標記表若被 pg_dump / pg_restore
    // 帶到別顆庫，或被人手動偽造，身分（db_name/db_oid）比對不上就不該被信任。
    //
    // 不實際跑 pg_dump/pg_restore（測試環境不保證有該執行檔），改用等價的手段：
    // 直接在一顆「內容上看起來像 prod（名稱不含 test 且空無一物）」的 scratch 庫裡
    // 手動塞一列指向別的資料庫身分的標記——這正是「標記從別處搬過來」在資料庫
    // 層面唯一會留下的痕跡。護欄應該判定身分對不上、當作沒有標記，
    // 落回完整檢查（此案例會落到名稱檢查那一關被拒絕）。
    let name = format!("ipig_guard_regression_forged_{}", std::process::id());
    assert!(!name.contains("test"));
    create_scratch_db(&name).await;
    let url = with_db_name(&base_url(), &name);

    {
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("connect scratch db to plant forged marker");
        sqlx::query(
            "CREATE TABLE public.__ipig_disposable_test_db_v2 (\
                 db_name    text        NOT NULL,\
                 db_oid     bigint      NOT NULL,\
                 cluster_id text,\
                 stamped_at timestamptz NOT NULL DEFAULT now()\
             )",
        )
        .execute(&admin)
        .await
        .expect("seed: create forged marker table");
        // 故意填錯的身分：不是這顆庫真正的 db_name/db_oid。
        sqlx::query(
            "INSERT INTO public.__ipig_disposable_test_db_v2 (db_name, db_oid, cluster_id) \
             VALUES ('some_other_database_entirely', 999999999, NULL)",
        )
        .execute(&admin)
        .await
        .expect("seed: insert forged marker row");
    }

    std::env::set_var("TEST_DATABASE_URL", &url);
    let outcome = tokio::spawn(test_db::connect_disposable(3)).await;
    assert!(
        outcome.is_err(),
        "Critical 1 回歸：標記表存在但身分（db_name/db_oid）對不上目前這顆資料庫，\
         護欄應視同沒有標記、落回完整檢查（並在此案例因名稱不含 test 而拒絕），\
         但它信任了偽造的標記"
    );
}
