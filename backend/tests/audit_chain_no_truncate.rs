//! Migration 013：稽核鏈三張表的 STATEMENT 層 TRUNCATE 擋板。
//!
//! 背景與 `erp_ledger_immutability.rs` 的 TRUNCATE 那一組相同：PostgreSQL 的
//! TRUNCATE **不觸發** BEFORE UPDATE / BEFORE DELETE 的 row-level trigger，所以
//! 只有 immutable / no_delete 的表，實際上可以被一句 TRUNCATE 清空。
//! `user_activity_logs`（稽核紀錄）、`electronic_signatures`（電子簽章）、
//! `animal_blood_test_items`（血檢結果）三張表在 013 之前正是這個狀態。
//!
//! 這個缺口可被利用、不是理論風險：app 的 DB 連線帳號與這三張表的 owner 相同，
//! 而 owner 擁有不受 GRANT / REVOKE 限制的 TRUNCATE 權限。
//!
//! 本檔鎖住三件事，少任何一件擋板都是形式上的：
//!
//! 1. 三張表各自拒絕 TRUNCATE，且被擋下的操作不得清掉任何列。
//! 2. **`user_activity_logs` 的每一個分割區也各自拒絕**。這條最容易漏：
//!    row-level trigger 建在 parent 上會自動套用到所有分割區，但
//!    statement-level TRUNCATE trigger 不會——只建在 parent 上的話，
//!    `TRUNCATE user_activity_logs_2026_q3` 直接繞過擋板。
//! 3. `animal_blood_test_items` 的 bypass GUC 確實可用。它不是可有可無的裝飾：
//!    IDXF 全庫匯入的 `TRUNCATE ... CASCADE` 會遞移波及該表，escape hatch 壞掉
//!    等於匯入功能壞掉，而那種壞法只會在跑匯入時才炸。

use serial_test::serial;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[path = "common/test_db.rs"]
mod test_db;

/// IDXF 全庫匯入前的清理語句，與
/// `services/data_import.rs::cleanup_partial_unique_tables` 逐字相同。
///
/// 寫死在測試裡是刻意的：這條 SQL 與血檢表擋板之間的耦合（CASCADE 沿
/// `pens ← animals ← animal_blood_tests ← animal_blood_test_items` 遞移）
/// 完全不在型別系統的視野內，只能靠測試釘住。
const IDXF_CLEANUP_SQL: &str =
    r#"TRUNCATE TABLE "pens", "zones", "buildings", "facilities" RESTART IDENTITY CASCADE"#;

async fn setup_pool() -> PgPool {
    let pool = test_db::connect_disposable(5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 擋板必須是 trigger 丟出的 P0001，**且必須是 `table` 自己那支擋板**。
///
/// # 為什麼一定要比對表名，只驗 P0001 + "append-only" 不夠
///
/// 這條判準比 `erp_ledger_immutability.rs` 的版本嚴格，理由是實測踩到的：
/// `TRUNCATE electronic_signatures CASCADE` 的 cascade 會遞移展開到 **49 張表**，
/// 其中包含 `stock_ledger`——那張表在 002_schema.sql 就有自己的 no_truncate 擋板。
/// 於是就算把本次新增的 `electronic_signatures` 擋板整支拿掉，該語句**照樣**
/// 拋出 P0001 + "append-only"，只是換成帳簿表擋的。
///
/// 用寬鬆判準寫成的測試因此是假綠：mutation 驗證時（移除 013 全部 trigger）
/// 其餘四支測試都如預期轉紅，只有它照樣通過——它從頭到尾沒測到自己該測的東西。
fn assert_blocked_by(err: &sqlx::Error, table: &str, ctx: &str) {
    let db_err = err
        .as_database_error()
        .unwrap_or_else(|| panic!("{ctx}：應為資料庫錯誤，實際：{err}"));
    assert_eq!(
        db_err.code().as_deref(),
        Some("P0001"),
        "{ctx}：應為 trigger 擋下的 P0001，實際 code={:?} msg={}",
        db_err.code(),
        db_err.message()
    );
    let expected = format!("{table} is append-only");
    assert!(
        db_err.message().contains(&expected),
        "{ctx}：應由 {table} 自己的 no_truncate 擋板擋下（訊息需含「{expected}」），\
         實際訊息：{}",
        db_err.message()
    );
}

async fn seed_activity_log(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_activity_logs (id, event_category, event_type) \
         VALUES ($1, 'security', 'NO_TRUNCATE_FIXTURE')",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("seed activity log");
    id
}

async fn seed_signature(pool: &PgPool) -> Uuid {
    let signer = test_db::seed_other_user(pool, "no-truncate-sig").await;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO electronic_signatures \
             (id, entity_type, entity_id, signer_id, signature_type, content_hash, \
              signature_data, meaning) \
         VALUES ($1, 'protocol', $2, $3, 'electronic', 'x', 'x', 'AUTHOR')",
    )
    .bind(id)
    .bind(Uuid::new_v4().to_string())
    .bind(signer)
    .execute(pool)
    .await
    .expect("seed electronic signature");
    id
}

/// 造一筆血檢明細（連同其依賴的動物與血檢單），回傳 item id。
///
/// `animals.pen_id` 留 NULL：本 helper 只為了讓表裡有列可供「被擋下的 TRUNCATE
/// 不得清空任何列」比對，不需要接上 pens。IDXF 的 cascade 路徑由
/// [`blood_test_items_bypass_lets_idxf_cleanup_through`] 直接跑真正的清理語句驗證。
async fn seed_blood_test_item(pool: &PgPool) -> Uuid {
    let s = Uuid::new_v4().simple().to_string();
    let animal = Uuid::new_v4();
    let test = Uuid::new_v4();
    let item = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO animals (id, ear_tag, breed, gender, entry_date) \
         VALUES ($1, $2, 'LYD', 'male', CURRENT_DATE)",
    )
    .bind(animal)
    // animals.ear_tag 是 varchar(10)，前綴 + 8 碼 hex 剛好用滿，不能再長。
    .bind(format!("NT{}", &s[..8]))
    .execute(pool)
    .await
    .expect("seed animal");

    sqlx::query(
        "INSERT INTO animal_blood_tests (id, animal_id, test_date, status) \
         VALUES ($1, $2, CURRENT_DATE, 'completed')",
    )
    .bind(test)
    .bind(animal)
    .execute(pool)
    .await
    .expect("seed blood test");

    sqlx::query(
        "INSERT INTO animal_blood_test_items (id, blood_test_id, item_name, result_value) \
         VALUES ($1, $2, '不可竄改測試項', '1.0')",
    )
    .bind(item)
    .bind(test)
    .execute(pool)
    .await
    .expect("seed blood test item");

    item
}

async fn count_by_id(pool: &PgPool, table: &str, id: Uuid) -> i64 {
    let sql = format!("SELECT count(*) AS n FROM {table} WHERE id = $1");
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("count surviving row")
        .get("n")
}

// ── user_activity_logs ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn user_activity_logs_rejects_truncate() {
    let pool = setup_pool().await;
    let id = seed_activity_log(&pool).await;

    let err = sqlx::query("TRUNCATE user_activity_logs")
        .execute(&pool)
        .await
        .expect_err("user_activity_logs 應拒絕 TRUNCATE");
    assert_blocked_by(&err, "user_activity_logs", "user_activity_logs TRUNCATE");

    assert_eq!(
        count_by_id(&pool, "user_activity_logs", id).await,
        1,
        "被擋下的 TRUNCATE 不得清空任何列"
    );
}

/// 分割區必須各自有擋板——statement-level trigger 不會從 parent 繼承下來。
///
/// 動態列舉而不是寫死 12 個名字：分割區集合會隨 `PartitionMaintenanceJob`
/// 每季增加，寫死的清單只會愈來愈不完整，而「漏掉的那個」正是缺口所在。
#[tokio::test]
#[serial]
async fn every_user_activity_logs_partition_rejects_truncate() {
    let pool = setup_pool().await;
    let id = seed_activity_log(&pool).await;

    let partitions: Vec<String> = sqlx::query_scalar(
        "SELECT inhrelid::regclass::text FROM pg_inherits \
         WHERE inhparent = 'public.user_activity_logs'::regclass ORDER BY 1",
    )
    .fetch_all(&pool)
    .await
    .expect("list partitions");

    assert!(
        !partitions.is_empty(),
        "user_activity_logs 應為分割表且至少有一個分割區，實際列舉不到任何分割區"
    );

    for part in &partitions {
        let sql = format!("TRUNCATE {part}");
        let err = sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&pool)
            .await
            .expect_err(&format!("分割區 {part} 應拒絕 TRUNCATE，實際卻成功了"));
        // 分割區的 trigger 共用 parent 的函式，訊息因此仍點名 user_activity_logs。
        assert_blocked_by(&err, "user_activity_logs", &format!("{part} TRUNCATE"));
    }

    assert_eq!(
        count_by_id(&pool, "user_activity_logs", id).await,
        1,
        "被擋下的分割區 TRUNCATE 不得清空任何列"
    );
}

// ── electronic_signatures ─────────────────────────────────────────

#[tokio::test]
#[serial]
async fn electronic_signatures_rejects_truncate() {
    let pool = setup_pool().await;
    let id = seed_signature(&pool).await;

    // 不帶 CASCADE 會先撞上 FK（amendments 參照本表），那是 FK 擋的不是擋板擋的；
    // 要驗擋板本身就必須用 CASCADE 走到 trigger 那一步。
    let err = sqlx::query("TRUNCATE electronic_signatures CASCADE")
        .execute(&pool)
        .await
        .expect_err("electronic_signatures 應拒絕 TRUNCATE CASCADE");
    assert_blocked_by(
        &err,
        "electronic_signatures",
        "electronic_signatures TRUNCATE CASCADE",
    );

    assert_eq!(
        count_by_id(&pool, "electronic_signatures", id).await,
        1,
        "被擋下的 TRUNCATE 不得清空任何列"
    );
}

// ── animal_blood_test_items ───────────────────────────────────────

#[tokio::test]
#[serial]
async fn blood_test_items_rejects_truncate() {
    let pool = setup_pool().await;
    let id = seed_blood_test_item(&pool).await;

    let err = sqlx::query("TRUNCATE animal_blood_test_items")
        .execute(&pool)
        .await
        .expect_err("animal_blood_test_items 應拒絕 TRUNCATE");
    assert_blocked_by(
        &err,
        "animal_blood_test_items",
        "animal_blood_test_items TRUNCATE",
    );

    assert_eq!(
        count_by_id(&pool, "animal_blood_test_items", id).await,
        1,
        "被擋下的 TRUNCATE 不得清空任何列"
    );
}

/// escape hatch 必須真的能開——否則 IDXF 全庫匯入會壞在一個只有跑匯入才會炸的地方。
///
/// 兩段對照都在同一個 tx 內跑完後 rollback：TRUNCATE 在 PostgreSQL 是
/// transactional，rollback 後測試庫的參考資料原封不動，不影響其他測試檔。
#[tokio::test]
#[serial]
async fn blood_test_items_bypass_lets_idxf_cleanup_through() {
    let pool = setup_pool().await;

    // (a) 沒開 bypass：IDXF 的清理語句應該被血檢表的擋板攔下。
    //     這一段同時證明「data_import 那邊的 SET LOCAL 不是多餘的」。
    let mut tx = pool.begin().await.expect("begin tx");
    let err = sqlx::query(IDXF_CLEANUP_SQL)
        .execute(&mut *tx)
        .await
        .expect_err("未開 bypass 時，IDXF 清理語句應被血檢表擋板攔下");
    assert_blocked_by(
        &err,
        "animal_blood_test_items",
        "IDXF cleanup without bypass",
    );
    tx.rollback().await.expect("rollback tx");

    // (b) 開了 bypass：同一句應該跑得過。
    let mut tx = pool.begin().await.expect("begin tx");
    sqlx::query("SET LOCAL app.bypass_blood_test_items_truncate = 'true'")
        .execute(&mut *tx)
        .await
        .expect("set bypass GUC");
    sqlx::query(IDXF_CLEANUP_SQL)
        .execute(&mut *tx)
        .await
        .expect("開了 bypass 之後，IDXF 清理語句應可執行");
    tx.rollback().await.expect("rollback tx");
}
