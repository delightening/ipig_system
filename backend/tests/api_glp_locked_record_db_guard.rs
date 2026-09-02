//! 已鎖定紀錄的**資料庫層級**寫入護欄回歸測試（GLP §11.10(e)(1)）
//!
//! 對照 `api_glp_record_lock.rs`：那支測的是 **service 層** guard
//! （`ensure_not_locked_uuid` → 409）。本支測的是它擋不到的那一半——
//! **繞過 service 層的寫入路徑**（raw SQL、維運工具、未來忘記呼叫 guard 的
//! 新 handler）是否仍被 DB 觸發器攔下。
//!
//! ## 為什麼要分成兩支測試檔
//!
//! service 層 guard 與 DB 觸發器擋的是不同的攻擊面，且**可以各自單獨失效**：
//! 拿掉 `ensure_not_locked` 呼叫，`api_glp_record_lock.rs` 會紅；
//! 拿掉 migration 的觸發器，那支**照樣全綠**——因為它每一條路徑都經過 service。
//! 本檔一律用 `app.db_pool` 直接下 SQL，確保紅燈只可能來自 DB 層。
//!
//! ## 測試範圍
//!
//! 1. `db_guard_blocks_raw_update_on_every_locked_table` — 5 張表逐一驗證核心欄位
//!    被擋，且錯誤訊息指得出是哪個鎖、哪個欄位。
//! 2. `db_guard_allows_vet_read_side_update_on_locked_records` — **合法旁側更新
//!    仍放行**：獸醫已讀標記（raw SQL + HTTP endpoint 兩種路徑都驗）。
//! 3. `db_guard_blocks_hard_delete_on_locked_record` — 鎖定後硬刪除被擋，
//!    未鎖定的同表紀錄照常可刪（確認不是無腦全擋）。
//! 4. `db_guard_ignores_unlocked_records` — 未鎖定紀錄完全不受影響。
//! 5. `locked_sacrifice_upsert_returns_409` — service 層補上的那道 guard：
//!    `upsert_sacrifice` 的 `ON CONFLICT DO UPDATE` 不得覆寫已鎖定的犧牲紀錄，
//!    且要回 409 而不是讓它撞 DB 觸發器變成 500。
//! 6. `repeat_lock_keeps_first_locker` — `lock_record_uuid` idempotent：
//!    多重簽章時第二簽不覆寫 `locked_by`（覆寫會被 DB 觸發器擋成 DB 例外）。

mod common;

use common::TestApp;
use erp_backend::services::SignatureService;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

// ── 輔助函式 ────────────────────────────────────────────────────────────────

async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', $3, true, false)",
    )
    .bind(id)
    .bind(format!("dbguard-{label}-{}@test.local", &id.to_string()[..6]))
    .bind(format!("dbguard {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert test user");
    id
}

/// 建立動物。`iacuc_no` 非空才過得了 `require_animal_has_protocol`
/// （犧牲採樣是需計畫的紀錄類型）。
async fn seed_animal(app: &TestApp, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by) \
         VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-01-01', $3, $4)",
    )
    .bind(id)
    .bind(format!("DG{}", &id.to_string()[..6]))
    .bind(format!("IACUC-DG-{}", &id.to_string()[..8]))
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert test animal");
    id
}

async fn seed_observation(pool: &PgPool, animal_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO animal_observations (animal_id, event_date, record_type, content) \
         VALUES ($1, '2026-04-25', 'observation'::record_type, 'original content') RETURNING id",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .expect("insert observation")
}

async fn seed_surgery(pool: &PgPool, animal_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO animal_surgeries (animal_id, surgery_date, surgery_site) \
         VALUES ($1, '2026-04-25', 'abdomen') RETURNING id",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .expect("insert surgery")
}

async fn seed_sacrifice(pool: &PgPool, animal_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO animal_sacrifices (animal_id, sacrifice_date) \
         VALUES ($1, '2026-04-25') RETURNING id",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .expect("insert sacrifice")
}

async fn seed_blood_test(pool: &PgPool, animal_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO animal_blood_tests (animal_id, test_date, remark) \
         VALUES ($1, '2026-04-25', 'original remark') RETURNING id",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .expect("insert blood test")
}

async fn seed_care_record(pool: &PgPool, observation_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO care_medication_records (record_type, record_id, record_mode, pain_score) \
         VALUES ('observation'::vet_record_type, $1, 'pain_assessment'::care_record_mode, 1) \
         RETURNING id",
    )
    .bind(observation_id)
    .fetch_one(pool)
    .await
    .expect("insert care medication record")
}

/// 直接把紀錄設為鎖定（模擬簽章後的狀態）。
/// 這一步本身是 `is_locked` false → true，必須放行，否則後面全部測不下去。
async fn force_lock(pool: &PgPool, table: &str, id: Uuid) {
    let sql = format!("UPDATE {table} SET is_locked = true, locked_at = NOW() WHERE id = $1");
    // 表名來自本檔寫死的清單，不是外部輸入
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("首次上鎖不該被擋（{table}）：{e}"));
}

// ============================================================
// Test 1: 5 張表的核心欄位在鎖定後都改不動
// ============================================================

#[tokio::test]
#[serial]
async fn db_guard_blocks_raw_update_on_every_locked_table() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let user_id = seed_user(&app, "blocks").await;
    let animal_id = seed_animal(&app, user_id).await;

    let observation_id = seed_observation(pool, animal_id).await;
    let surgery_id = seed_surgery(pool, animal_id).await;
    let sacrifice_id = seed_sacrifice(pool, animal_id).await;
    let blood_test_id = seed_blood_test(pool, animal_id).await;
    let care_id = seed_care_record(pool, observation_id).await;

    // (表名, 主鍵, 改核心欄位的 SQL, 訊息裡應出現的欄位名)
    let cases: Vec<(&str, Uuid, &str, &str)> = vec![
        (
            "animal_observations",
            observation_id,
            "UPDATE animal_observations SET content = 'tampered' WHERE id = $1",
            "content",
        ),
        (
            "animal_surgeries",
            surgery_id,
            "UPDATE animal_surgeries SET surgery_site = 'tampered' WHERE id = $1",
            "surgery_site",
        ),
        (
            "animal_sacrifices",
            sacrifice_id,
            "UPDATE animal_sacrifices SET sacrifice_date = '2026-12-31' WHERE id = $1",
            "sacrifice_date",
        ),
        (
            "animal_blood_tests",
            blood_test_id,
            "UPDATE animal_blood_tests SET remark = 'tampered' WHERE id = $1",
            "remark",
        ),
        (
            "care_medication_records",
            care_id,
            "UPDATE care_medication_records SET pain_score = 3 WHERE id = $1",
            "pain_score",
        ),
    ];

    for (table, id, sql, column) in cases {
        force_lock(pool, table, id).await;

        // 不用 expect_err(&format!(..))：clippy::expect_fun_call 會擋（-D warnings）
        let msg = match sqlx::query(sql).bind(id).execute(pool).await {
            Ok(_) => panic!("{table}：已鎖定紀錄的 raw UPDATE 竟然成功——DB 層護欄不存在或失效"),
            Err(e) => e.to_string(),
        };

        assert!(
            msg.contains("GLP 簽章鎖"),
            "{table}：錯誤訊息應說明是 GLP 簽章鎖擋的，實得：{msg}"
        );
        assert!(
            msg.contains(column),
            "{table}：錯誤訊息應指出被擋的欄位 {column}，實得：{msg}"
        );
    }

    // 逐張確認值真的沒被改到（錯誤訊息對了、資料被改掉也是失敗）
    let content: String =
        sqlx::query_scalar("SELECT content FROM animal_observations WHERE id = $1")
            .bind(observation_id)
            .fetch_one(pool)
            .await
            .expect("read back observation");
    assert_eq!(content, "original content", "鎖定紀錄的內容不該被改動");
}

// ============================================================
// Test 2: 合法旁側更新（獸醫已讀）仍放行
// ============================================================

#[tokio::test]
#[serial]
async fn db_guard_allows_vet_read_side_update_on_locked_records() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let token = app.login_as_admin().await;
    let user_id = seed_user(&app, "vetread").await;
    let animal_id = seed_animal(&app, user_id).await;

    let observation_id = seed_observation(pool, animal_id).await;
    let surgery_id = seed_surgery(pool, animal_id).await;
    let blood_test_id = seed_blood_test(pool, animal_id).await;
    let care_id = seed_care_record(pool, observation_id).await;

    force_lock(pool, "animal_observations", observation_id).await;
    force_lock(pool, "animal_surgeries", surgery_id).await;
    force_lock(pool, "animal_blood_tests", blood_test_id).await;
    force_lock(pool, "care_medication_records", care_id).await;

    // ① raw SQL：三張有 vet_read_at / updated_at 的表
    for (table, id) in [
        ("animal_observations", observation_id),
        ("animal_surgeries", surgery_id),
        ("animal_blood_tests", blood_test_id),
    ] {
        let sql = format!(
            "UPDATE {table} SET vet_read = true, vet_read_at = NOW(), updated_at = NOW() WHERE id = $1"
        );
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("{table}：鎖定後標記獸醫已讀是合法旁側更新，不該被擋：{e}"));
    }

    // ② care_medication_records 只有 vet_read 一欄（無 vet_read_at / updated_at）
    sqlx::query("UPDATE care_medication_records SET vet_read = true WHERE id = $1")
        .bind(care_id)
        .execute(pool)
        .await
        .expect("care_medication_records：鎖定後標記獸醫已讀不該被擋");

    // ③ HTTP 端到端：mark_vet_read 這條 service 路徑刻意不呼叫 ensure_not_locked，
    //    DB 護欄若把它一起擋掉，這裡就會是非 2xx。
    let res = app
        .auth_post(
            &format!("/api/v1/observations/{observation_id}/vet-read"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "已鎖定 observation 的獸醫已讀 endpoint 應仍可用，實得 {}",
        res.status()
    );

    let vet_read: bool =
        sqlx::query_scalar("SELECT vet_read FROM animal_observations WHERE id = $1")
            .bind(observation_id)
            .fetch_one(pool)
            .await
            .expect("read back vet_read");
    assert!(vet_read, "vet_read 應已被標記");
}

// ============================================================
// Test 3: 鎖定後硬刪除被擋；未鎖定的同表紀錄照常可刪
// ============================================================

#[tokio::test]
#[serial]
async fn db_guard_blocks_hard_delete_on_locked_record() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let user_id = seed_user(&app, "delete").await;
    let animal_id = seed_animal(&app, user_id).await;

    let locked_id = seed_observation(pool, animal_id).await;
    let unlocked_id = seed_observation(pool, animal_id).await;
    force_lock(pool, "animal_observations", locked_id).await;

    let err = sqlx::query("DELETE FROM animal_observations WHERE id = $1")
        .bind(locked_id)
        .execute(pool)
        .await
        .expect_err("已鎖定紀錄的 DELETE 竟然成功");
    let msg = err.to_string();
    assert!(
        msg.contains("GLP 簽章鎖") && msg.contains("不可 DELETE"),
        "DELETE 錯誤訊息應說明是 GLP 簽章鎖擋的，實得：{msg}"
    );

    let rows = sqlx::query("DELETE FROM animal_observations WHERE id = $1")
        .bind(unlocked_id)
        .execute(pool)
        .await
        .expect("未鎖定紀錄應可刪除——護欄不該無腦全擋")
        .rows_affected();
    assert_eq!(rows, 1, "未鎖定紀錄應剛好刪掉 1 列");
}

// ============================================================
// Test 4: 未鎖定紀錄完全不受影響
// ============================================================

#[tokio::test]
#[serial]
async fn db_guard_ignores_unlocked_records() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let user_id = seed_user(&app, "unlocked").await;
    let animal_id = seed_animal(&app, user_id).await;
    let observation_id = seed_observation(pool, animal_id).await;

    sqlx::query("UPDATE animal_observations SET content = 'edited while unlocked' WHERE id = $1")
        .bind(observation_id)
        .execute(pool)
        .await
        .expect("未鎖定紀錄的核心欄位應可修改");

    let content: String =
        sqlx::query_scalar("SELECT content FROM animal_observations WHERE id = $1")
            .bind(observation_id)
            .fetch_one(pool)
            .await
            .expect("read back content");
    assert_eq!(content, "edited while unlocked");
}

// ============================================================
// Test 5: upsert_sacrifice 不得覆寫已鎖定的犧牲紀錄（service 層補的 guard）
// ============================================================

#[tokio::test]
#[serial]
async fn locked_sacrifice_upsert_returns_409() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let token = app.login_as_admin().await;
    let user_id = seed_user(&app, "sacrupsert").await;
    let animal_id = seed_animal(&app, user_id).await;

    let sacrifice_id = seed_sacrifice(pool, animal_id).await;
    force_lock(pool, "animal_sacrifices", sacrifice_id).await;

    // upsert 走 ON CONFLICT DO UPDATE：同一隻動物再 POST 一次就是覆寫既有那筆
    let body = serde_json::json!({
        "sacrifice_date": "2026-12-31",
        "confirmed_sacrifice": true,
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/sacrifice"),
            &body,
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        409,
        "已鎖定的犧牲紀錄被 upsert 覆寫時應回 409（而不是成功、也不是撞 DB 觸發器變 500），實得 {}",
        res.status()
    );

    let date: chrono::NaiveDate =
        sqlx::query_scalar("SELECT sacrifice_date FROM animal_sacrifices WHERE id = $1")
            .bind(sacrifice_id)
            .fetch_one(pool)
            .await
            .expect("read back sacrifice_date");
    assert_eq!(
        date.to_string(),
        "2026-04-25",
        "已鎖定的犧牲紀錄內容不該被 upsert 改動"
    );
}

// ============================================================
// Test 6: lock_record_uuid idempotent — 多重簽章不覆寫鎖定人
// ============================================================

#[tokio::test]
#[serial]
async fn repeat_lock_keeps_first_locker() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let first_signer = seed_user(&app, "signer1").await;
    let second_signer = seed_user(&app, "signer2").await;
    let animal_id = seed_animal(&app, first_signer).await;
    let observation_id = seed_observation(pool, animal_id).await;

    SignatureService::lock_record_uuid(pool, "observation", observation_id, first_signer)
        .await
        .expect("第一次上鎖應成功");

    // 第二簽（WITNESS 等）會再呼叫一次 lock_record_uuid。
    // 舊實作會覆寫 locked_at / locked_by，而鎖定欄位在 DB 層不可變更 → 會炸。
    SignatureService::lock_record_uuid(pool, "observation", observation_id, second_signer)
        .await
        .expect("已鎖定紀錄再次呼叫上鎖應為 no-op，不該報錯");

    let locked_by: Option<Uuid> =
        sqlx::query_scalar("SELECT locked_by FROM animal_observations WHERE id = $1")
            .bind(observation_id)
            .fetch_one(pool)
            .await
            .expect("read back locked_by");
    assert_eq!(
        locked_by,
        Some(first_signer),
        "鎖定人應維持第一個簽章者，不該被第二簽覆寫"
    );
}
