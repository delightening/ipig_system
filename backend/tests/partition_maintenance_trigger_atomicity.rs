//! `PartitionMaintenanceJob` 建新分區時，分區表與它的 TRUNCATE 擋板必須原子地一起建立。
//!
//! # 為什麼「跑一次正常路徑看 trigger 在不在」不夠
//!
//! `ensure_partitions` 判斷一個分區「已存在」只看 `pg_tables`（見
//! `get_existing_partitions`），**不檢查它有沒有 trigger**。所以若
//! `CREATE TABLE ... PARTITION OF` 成功、緊接著建 trigger 那句失敗（連線中斷、
//! 資料庫重啟），分區會被後續每一次排程都判定為「已存在」而略過——trigger 永遠
//! 不會被補建，裸分區一直留到有人手動發現為止。
//!
//! 正常路徑測不到這件事：兩句都成功時，有沒有包在同一個 transaction 裡看起來一樣。
//! 要證明的是**失敗時的行為**，所以本檔刻意製造「第二句必定失敗」的情境。
//!
//! # 兩個踩過的坑，寫在這裡免得下一個人重踩
//!
//! **1. 不能用 `DROP FUNCTION ... CASCADE` 製造失敗。** 最直覺的做法是把
//! `check_user_activity_logs_no_truncate()` 砍掉，讓建 trigger 那句找不到函式。
//! 但那支函式被 **15 支既有 trigger 共用**（`user_activity_logs` parent + 12 個
//! 分區 + `electronic_signatures` + `animal_blood_test_items`），而整合測試是
//! **跨測試檔共用同一個測試資料庫**的——`CASCADE` 會把 `audit_chain_no_truncate.rs`
//! 依賴的擋板全部連帶刪掉，且不會自己長回來。
//! 這裡改用 `ALTER FUNCTION ... RENAME`：PostgreSQL 的 trigger 綁的是函式 OID
//! 不是名字，改名不影響任何既有 trigger，但會讓「用名字解析函式」的
//! `CREATE TRIGGER ... EXECUTE FUNCTION public.<name>()` 找不到目標而失敗——
//! 正好是需要的失敗點，而且可逆。
//!
//! **2. 還原步驟必須排在所有斷言之前。** 斷言一 panic 就不會往下跑，若把改名還原
//! 放在斷言後面，任何一次斷言失敗都會讓共用測試庫卡在「函式被改走」的狀態，
//! 波及同一輪其他測試檔。所以下面的順序是固定的：製造失敗 → 收集結果 → **先還原**
//! → 才開始斷言。
//!
//! **3. 得先騰出一個空位給它建。** `002_schema.sql` 已經預先建好 2026Q1–2028Q4 共
//! 12 個分區，而 `ensure_partitions` 只檢查「當年 + 未來 2 年」——在 2026 年跑，
//! 這 12 個全都已存在，`create_quarterly_partition` 一次都不會被呼叫。所以測試要
//! 先 drop 掉其中一個，才有東西可建。

use chrono::{Datelike, Utc};
use erp_backend::services::PartitionMaintenanceJob;
use serial_test::serial;
use sqlx::{PgPool, Row};

#[path = "common/test_db.rs"]
mod test_db;

/// 拿來當實驗品的分區：`ensure_partitions` 檢查範圍（當年 + 未來 2 年）的最後一季。
///
/// 挑最後一季是因為它離「現在」最遠，最不可能被其他測試寫進資料（活動紀錄的
/// `partition_date` 預設是 `CURRENT_DATE`）。
///
/// ⚠️ **年份必須從當年推導，不能寫死**（CodeRabbit 於 head `414ca06` 指出，成立）。
/// 寫死的話會變成定時炸彈：`ensure_partitions` 只看當年 +0/+1/+2，一旦寫死的年份
/// 掉出這個窗，`create_quarterly_partition` 就不會被呼叫，`result.failed` 是空的、
/// 下面的斷言失敗；更糟的是分區已經被 drop 而斷言先 panic，自我修復段跑不到，
/// 共用測試庫會少一個分區。用 `current_year + 2` 讓實驗品隨年份自動前移。
fn victim_partition() -> String {
    format!("user_activity_logs_{}_q4", Utc::now().year() + 2)
}

/// 擋板函式的真名，與暫時搬走時用的名字。
const GUARD_FN: &str = "check_user_activity_logs_no_truncate";
const GUARD_FN_PARKED: &str = "check_user_activity_logs_no_truncate__parked_by_test";

async fn setup_pool() -> PgPool {
    let pool = test_db::connect_disposable(5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn function_exists(pool: &PgPool, name: &str) -> bool {
    sqlx::query(
        "SELECT EXISTS (
            SELECT 1 FROM pg_proc
            WHERE proname = $1 AND pronamespace = 'public'::regnamespace
        ) AS present",
    )
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("probe function existence")
    .get::<bool, _>("present")
}

/// 改名，且對「已經是目標狀態」是安全的（回傳是否真的動了）。
///
/// 不用 `ALTER FUNCTION IF EXISTS`：先查 `pg_proc` 再決定要不要下 DDL，語意更明確，
/// 也讓這個 helper 在上一輪測試中途崩潰、函式還停在暫存名字時能自己收斂回來。
async fn rename_function(pool: &PgPool, from: &str, to: &str) -> bool {
    if !function_exists(pool, from).await {
        return false;
    }
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ALTER FUNCTION public.{from}() RENAME TO {to}"
    )))
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("rename function {from} -> {to}: {e}"));
    true
}

async fn partition_exists(pool: &PgPool, name: &str) -> bool {
    sqlx::query("SELECT to_regclass($1) IS NOT NULL AS present")
        .bind(format!("public.{name}"))
        .fetch_one(pool)
        .await
        .expect("probe partition existence")
        .get::<bool, _>("present")
}

async fn partition_has_guard_trigger(pool: &PgPool, name: &str) -> bool {
    sqlx::query(
        "SELECT EXISTS (
            SELECT 1 FROM pg_trigger
            WHERE tgrelid = $1::regclass
              AND tgname = 'check_user_activity_logs_no_truncate_trigger'
              AND NOT tgisinternal
        ) AS present",
    )
    .bind(format!("public.{name}"))
    .fetch_one(pool)
    .await
    .expect("probe trigger existence")
    .get::<bool, _>("present")
}

/// 建 trigger 那句失敗時，同一次呼叫裡的 `CREATE TABLE` 必須一起被 rollback——
/// 不能留下「表在、trigger 不在」的裸分區。
///
/// 本 case 走完整個循環：製造失敗 → 驗證沒有殘留 → 還原環境 → 驗證正常路徑會把
/// 分區連同擋板一起建回來。最後一段同時是正常路徑的回歸測試，也讓共用測試庫回到
/// 原本的狀態（12 個分區都在、都有擋板）。
#[tokio::test]
#[serial]
async fn failed_trigger_creation_rolls_back_the_partition_table() {
    let pool = setup_pool().await;
    let victim = victim_partition();

    // 上一輪若崩在中途，函式可能還停在暫存名字；先收斂回真名再開始。
    rename_function(&pool, GUARD_FN_PARKED, GUARD_FN).await;
    assert!(
        function_exists(&pool, GUARD_FN).await,
        "測試前提不成立：找不到擋板函式 {GUARD_FN}"
    );

    // 騰出空位（見檔頭坑 3）。DROP TABLE 會自動把分區從父表 detach。
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP TABLE IF EXISTS public.{victim}"
    )))
    .execute(&pool)
    .await
    .expect("drop victim partition to make room for ensure_partitions");

    // 製造失敗：把函式搬走，讓 CREATE TRIGGER ... EXECUTE FUNCTION 依名字解析不到。
    assert!(
        rename_function(&pool, GUARD_FN, GUARD_FN_PARKED).await,
        "改名應該要真的發生"
    );

    let result = PartitionMaintenanceJob::trigger(&pool).await;
    let victim_lingered = partition_exists(&pool, &victim).await;

    // 🔴 還原一定要在任何斷言之前（見檔頭坑 2）。
    rename_function(&pool, GUARD_FN_PARKED, GUARD_FN).await;

    let result = result.expect("ensure_partitions 本身應回 Ok：個別分區失敗會被收進 result.failed");
    assert!(
        result.failed.iter().any(|f| f == &victim),
        "擋板函式不存在時，{victim} 的建立應該要失敗並被記進 result.failed，實際：{:?}",
        result.failed
    );
    assert!(
        !victim_lingered,
        "{victim} 的 trigger 建立失敗了，但分區表還留在資料庫裡——\
         CREATE TABLE 與 CREATE TRIGGER 沒有包在同一個 transaction 內，\
         裸分區沒有被 rollback，而 ensure_partitions 之後只會把它當成「已存在」而永遠略過"
    );

    // 環境已還原，正常路徑應該把分區連同擋板一起建回來。
    let repaired = PartitionMaintenanceJob::trigger(&pool)
        .await
        .expect("guard function restored, ensure_partitions should succeed");
    assert!(
        repaired.failed.is_empty(),
        "還原後不應再有失敗的分區：{:?}",
        repaired.failed
    );
    assert!(
        partition_exists(&pool, &victim).await,
        "還原後 {victim} 應被重新建立"
    );
    assert!(
        partition_has_guard_trigger(&pool, &victim).await,
        "重新建立的 {victim} 必須帶 no_truncate 擋板，否則它就是個裸分區"
    );
}
