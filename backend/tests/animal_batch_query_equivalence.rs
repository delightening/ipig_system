//! N+1 批次查詢與逐筆查詢的**等價性**回歸測試。
//!
//! 背景：`services/animal/{observation,weight,surgery}.rs` 新增了 `list_for_animals`，
//! 讓 `medical.rs` 的動物清單改為「一次批次查 + HashMap 分組」，取代原本每隻動物各查
//! 一次的 N+1 迴圈。這類改寫最容易出的錯不是編譯錯誤，而是**結果集語意悄悄變了**：
//! JOIN 型別選錯會多出或漏掉列、排序鍵不同會改變同一動物內的順序、沒有紀錄的動物
//! 可能整隻從結果中消失。
//!
//! 因此本檔不寫死預期輸出，而是直接斷言「批次版 == 逐筆版」——逐筆版是改寫前既有的
//! 行為基準，兩者對**任何**測試資料都必須一致。寫死預期值只能證明「在我想到的那幾筆
//! 資料上一致」，等價斷言才擋得住日後有人再改壞一次。
//!
//! 涵蓋（對應 CodeRabbit 於 PR #22 的三則 review）：
//! - 完全沒有紀錄的動物 → 兩版都必須回空 Vec，且該動物不可從結果中消失
//! - soft-deleted 紀錄（`deleted_at IS NOT NULL`）→ 兩版都必須排除
//! - 同一動物內的排序（event_date / measure_date / surgery_date DESC）
//! - 多隻動物混合時不會互相污染（A 的紀錄不會被分到 B）
//!
//! 比對方式：這些 model 未 derive `PartialEq`，改以 `serde_json::to_value` 比對完整
//! 結構與順序——不為了測試而去動 production model 的 derive。

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serial_test::serial;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use erp_backend::services::{AnimalObservationService, AnimalSurgeryService, AnimalWeightService};

#[path = "common/test_db.rs"]
mod test_db;

/// **fail-closed**：見 `tests/common/test_db.rs`。
///
/// 只讀 `TEST_DATABASE_URL`（不 fallback 到 `DATABASE_URL`，開發機那條指向 prod），
/// 並在 migration 與任何寫入之前驗證目標資料庫**本身**是丟棄用測試庫——不是靠
/// 名字判斷。本檔早期版本用「DSN 是否含 `test`」的字串啟發式，已由該護欄取代。
async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

/// 建立一隻測試動物，回傳 id。
///
/// `breed` / `gender` 皆為 DB enum（`animal_breed` = miniature/white/LYD/other、
/// `animal_gender` = male/female），不可填自由文字。
///
/// ⚠️ `ear_tag` 是 `VARCHAR(10)`（`migrations/006_animal_management.sql:39`），
/// 呼叫端組出來的標籤**含 prefix 不得超過 10 字元**，否則 insert 會以
/// `22001 value too long` 失敗。現用 3 碼 prefix + `-` 只剩 6 碼可用。
async fn insert_animal(pool: &PgPool, tag: &str) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO animals (ear_tag, breed, gender, entry_date)
           VALUES ($1, 'other', 'female', CURRENT_DATE)
           RETURNING id"#,
    )
    .bind(tag)
    .fetch_one(pool)
    .await
    .expect("insert animal")
}

/// 測試資料佈局，四種情境一次涵蓋：
/// - `rich`：多筆紀錄，且其中一筆為 soft-deleted（驗證排除 + 排序）
/// - `empty`：完全沒有任何紀錄（驗證不會整隻消失）
/// - `single`：僅一筆（驗證單筆情境與逐筆版一致）
/// - `outsider`：**有**紀錄但**不在** `ids()` 請求清單內
///
/// `outsider` 是刻意的反例：前三者只能證明「請求的動物拿到正確資料」，若
/// `list_for_animals` 漏掉 `WHERE animal_id = ANY($1)`，逐筆比對仍會全數通過
/// （每組資料都還是對的，只是多回了別人的）。加一隻不該出現的動物，才驗得到
/// 篩選條件本身還在。
struct Fixture {
    rich: Uuid,
    empty: Uuid,
    single: Uuid,
    outsider: Uuid,
}

impl Fixture {
    /// 送進批次查詢的動物清單——**刻意不含 `outsider`**。
    fn ids(&self) -> Vec<Uuid> {
        vec![self.rich, self.empty, self.single]
    }
}

/// 對同一隻動物、同一時點各寫一筆 observation / weight / surgery。
///
/// 三張表一起寫，三個等價性測試才能共用同一份 fixture；`deleted_at` 傳
/// `Some(_)` 即為 soft-deleted 列。
async fn insert_record_triplet(
    pool: &PgPool,
    animal: Uuid,
    at: DateTime<Utc>,
    weight: Decimal,
    deleted_at: Option<DateTime<Utc>>,
) {
    sqlx::query(
        r#"INSERT INTO animal_observations
               (animal_id, event_date, record_type, content, deleted_at)
           VALUES ($1, $2, 'observation', 'batch-equivalence', $3)"#,
    )
    .bind(animal)
    .bind(at)
    .bind(deleted_at)
    .execute(pool)
    .await
    .expect("insert observation");

    sqlx::query(
        r#"INSERT INTO animal_weights
               (animal_id, measure_date, weight, deleted_at)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(animal)
    .bind(at)
    .bind(weight)
    .bind(deleted_at)
    .execute(pool)
    .await
    .expect("insert weight");

    sqlx::query(
        r#"INSERT INTO animal_surgeries
               (animal_id, surgery_date, surgery_site, deleted_at)
           VALUES ($1, $2, 'test-site', $3)"#,
    )
    .bind(animal)
    .bind(at)
    .bind(deleted_at)
    .execute(pool)
    .await
    .expect("insert surgery");
}

async fn seed(pool: &PgPool, prefix: &str) -> Fixture {
    let rich = insert_animal(pool, &format!("{prefix}-RICH")).await;
    let empty = insert_animal(pool, &format!("{prefix}-EMPTY")).await;
    let single = insert_animal(pool, &format!("{prefix}-SINGLE")).await;
    // `-OUT` 而非 `-OUTSIDER`：ear_tag 只有 VARCHAR(10)，3 碼 prefix 後放不下
    let outsider = insert_animal(pool, &format!("{prefix}-OUT")).await;

    let now = Utc::now();
    for (animal, days, deleted) in [
        (rich, 1_i64, false),
        (rich, 3, false),
        (rich, 2, true), // soft-deleted：兩版都必須排除
        (single, 5, false),
        (outsider, 4, false), // 不在請求清單內，批次結果不該出現
    ] {
        let deleted_at = if deleted { Some(now) } else { None };
        insert_record_triplet(
            pool,
            animal,
            now - Duration::days(days),
            Decimal::new(1000 + days, 1),
            deleted_at,
        )
        .await;
    }

    Fixture {
        rich,
        empty,
        single,
        outsider,
    }
}

/// 把批次結果依 animal_id 分組（保持原順序），供與逐筆結果比對。
fn group_by<T, F>(rows: Vec<T>, key: F) -> HashMap<Uuid, Vec<T>>
where
    F: Fn(&T) -> Uuid,
{
    let mut m: HashMap<Uuid, Vec<T>> = HashMap::new();
    for r in rows {
        m.entry(key(&r)).or_default().push(r);
    }
    m
}

/// 批次結果不得含 `ids()` 以外的動物。
///
/// 逐筆等價比對只看得到「請求的動物拿到的資料對不對」，`list_for_animals`
/// 若漏掉 `WHERE animal_id = ANY($1)`，每一組資料仍然正確、比對照樣全過，
/// 只是順帶把整張表都撈回來了。`fx.outsider` 是保證存在的反例：它有紀錄、
/// 但不在請求清單內，篩選一旦失效就會在這裡現形。
fn assert_outsider_absent<T>(batched: &HashMap<Uuid, Vec<T>>, fx: &Fixture, label: &str) {
    let requested: HashSet<Uuid> = fx.ids().into_iter().collect();
    let unexpected: Vec<Uuid> = batched
        .keys()
        .copied()
        .filter(|id| !requested.contains(id))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{label} 批次結果含未請求的 animal_id {unexpected:?}（fixture outsider={}）：\
         list_for_animals 的 `animal_id = ANY($1)` 篩選可能已遺失",
        fx.outsider
    );
}

#[tokio::test]
#[serial]
async fn observation_batch_equals_per_animal() {
    let pool = setup_pool().await;
    let fx = seed(&pool, "OBS").await;

    let batched = group_by(
        AnimalObservationService::list_for_animals(&pool, &fx.ids())
            .await
            .expect("batch query"),
        |o| o.animal_id,
    );

    for id in fx.ids() {
        let per_item = AnimalObservationService::list(&pool, id, None)
            .await
            .expect("per-animal query");
        let from_batch = batched.get(&id).cloned().unwrap_or_default();

        assert_eq!(
            serde_json::to_value(&per_item).expect("serialize per-item result"),
            serde_json::to_value(&from_batch).expect("serialize batched result"),
            "observation 批次與逐筆結果不一致（animal_id={id}）"
        );
    }

    // 空紀錄動物必須回空 Vec，而不是從結果中消失
    assert!(
        batched.get(&fx.empty).map(|v| v.is_empty()).unwrap_or(true),
        "沒有紀錄的動物不應出現在批次結果中"
    );
    // rich 有 3 筆但其中 1 筆 soft-deleted → 應只剩 2 筆
    assert_eq!(
        batched.get(&fx.rich).map(|v| v.len()).unwrap_or(0),
        2,
        "soft-deleted 的 observation 未被排除"
    );
    assert_outsider_absent(&batched, &fx, "observation");
}

#[tokio::test]
#[serial]
async fn weight_batch_equals_per_animal() {
    let pool = setup_pool().await;
    let fx = seed(&pool, "WGT").await;

    let batched = group_by(
        AnimalWeightService::list_for_animals(&pool, &fx.ids())
            .await
            .expect("batch query"),
        |w| w.animal_id,
    );

    for id in fx.ids() {
        let per_item = AnimalWeightService::list(&pool, id, None)
            .await
            .expect("per-animal query");
        let from_batch = batched.get(&id).cloned().unwrap_or_default();

        assert_eq!(
            serde_json::to_value(&per_item).expect("serialize per-item result"),
            serde_json::to_value(&from_batch).expect("serialize batched result"),
            "weight 批次與逐筆結果不一致（animal_id={id}）"
        );
    }

    assert_eq!(
        batched.get(&fx.rich).map(|v| v.len()).unwrap_or(0),
        2,
        "soft-deleted 的 weight 未被排除"
    );
    assert_outsider_absent(&batched, &fx, "weight");
}

#[tokio::test]
#[serial]
async fn surgery_batch_equals_per_animal() {
    let pool = setup_pool().await;
    let fx = seed(&pool, "SRG").await;

    let batched = group_by(
        AnimalSurgeryService::list_for_animals(&pool, &fx.ids())
            .await
            .expect("batch query"),
        |s| s.animal_id,
    );

    for id in fx.ids() {
        let per_item = AnimalSurgeryService::list(&pool, id, None)
            .await
            .expect("per-animal query");
        let from_batch = batched.get(&id).cloned().unwrap_or_default();

        assert_eq!(
            serde_json::to_value(&per_item).expect("serialize per-item result"),
            serde_json::to_value(&from_batch).expect("serialize batched result"),
            "surgery 批次與逐筆結果不一致（animal_id={id}）"
        );
    }

    assert_eq!(
        batched.get(&fx.rich).map(|v| v.len()).unwrap_or(0),
        2,
        "soft-deleted 的 surgery 未被排除"
    );
    assert_outsider_absent(&batched, &fx, "surgery");
}
