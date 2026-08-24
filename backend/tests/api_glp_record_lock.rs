//! C1 (GLP) record lock integration tests
//!
//! 驗證 21 CFR §11.10(e)(1)：簽章後的記錄不可修改 / 刪除。
//! 對應 docs/audit/system-review-2026-04-25.md C1。
//!
//! ## 測試範圍
//!
//! 1. **`signature_service_uuid_lock_round_trip`** — `lock_record_uuid` →
//!    `is_locked_uuid` round-trip 正確；`ensure_not_locked_uuid` 鎖定後回 Conflict。
//!
//! 2. **`update_locked_observation_returns_409`** — HTTP 端到端：建立 observation →
//!    DB 模擬簽章鎖定 → PUT 回 409。
//!
//! 3. **`delete_locked_observation_returns_409`** — 同上但 POST .../delete。
//!
//! 4. **`update_unlocked_observation_returns_200`** — 反例：未鎖定可正常修改。
//!
//! ## 測試設計
//!
//! 鎖定動作直接呼叫 `SignatureService::lock_record_uuid`（避開 sign_record 需要
//! 真實密碼 hash）。重點測 lock state → guard 行為，而非簽章本身的密碼驗證。

mod common;

use common::TestApp;
use erp_backend::services::SignatureService;
use erp_backend::AppError;
use serial_test::serial;
use uuid::Uuid;

/// 組出建立動物的請求 body（耳號由呼叫端配置後傳入）。
fn animal_body(ear_tag: &str) -> serde_json::Value {
    serde_json::json!({
        "ear_tag": ear_tag,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true
    })
}

/// 建立一隻動物，回傳 animal_id (UUID 字串)。
///
/// R109（2026-08-24）：原本用 `rand::random` 各自隨機挑 `ear_tag`（3 位數，
/// 100–999 共 900 個值）與 `entry_date`（月中 28 天），組合空間只有 28,000，
/// CI 上共用測試庫累積跑下來會生日碰撞——這個測試檔本身在 2026-08-24 就撞過一次
/// （`create animal failed: 409 Conflict … 耳號 100 已存在同出生日期的存活動物`）。
///
/// 改用 `common::create_animal_with_free_ear_tag`：配置空耳號、撞號就換一個重試。
/// 為什麼是重試而不是只做配置，見該函式的 doc comment（跨 process 的 TOCTOU）。
async fn create_test_animal(app: &TestApp, token: &str) -> String {
    let json = common::create_animal_with_free_ear_tag(app, token, animal_body).await;
    json["id"].as_str().expect("animal id missing").to_string()
}

/// 建立 observation，回傳 observation_id (UUID)。
async fn create_test_observation(app: &TestApp, token: &str, animal_id: &str) -> Uuid {
    let body = serde_json::json!({
        "event_date": "2026-04-25",
        "record_type": "observation",
        "content": "GLP lock test observation"
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/observations", animal_id),
            &body,
            token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "create observation failed: {}",
        res.status()
    );
    let json: serde_json::Value = res.json().await.expect("parse observation json");
    let id_str = json["id"].as_str().expect("observation id missing");
    Uuid::parse_str(id_str).expect("observation id parse")
}

/// 直接呼叫 service 鎖定（避開 sign_record 的密碼需求）。
async fn force_lock_observation(app: &TestApp, observation_id: Uuid) {
    // locked_by 用 admin 的 user_id（從 DB 抓）
    let admin_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(
            std::env::var("ADMIN_EMAIL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "admin@ipigsystem.asia".to_string()),
        )
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin id");

    SignatureService::lock_record_uuid(&app.db_pool, "observation", observation_id, admin_id.0)
        .await
        .expect("lock_record_uuid");
}

// ============================================================
// Test 1: SignatureService UUID lock round-trip
// ============================================================

#[tokio::test]
#[serial]
async fn signature_service_uuid_lock_round_trip() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let animal_id = create_test_animal(&app, &token).await;
    let observation_id = create_test_observation(&app, &token, &animal_id).await;

    // 初始：未鎖定
    let locked = SignatureService::is_locked_uuid(&app.db_pool, "observation", observation_id)
        .await
        .expect("is_locked_uuid");
    assert!(!locked, "新建 observation 應為未鎖定");

    // ensure_not_locked_uuid 應通過（未鎖定）
    SignatureService::ensure_not_locked_uuid(&app.db_pool, "observation", observation_id)
        .await
        .expect("ensure_not_locked_uuid before lock should succeed");

    // 鎖定
    force_lock_observation(&app, observation_id).await;

    // 鎖定後查詢
    let locked = SignatureService::is_locked_uuid(&app.db_pool, "observation", observation_id)
        .await
        .expect("is_locked_uuid after lock");
    assert!(locked, "lock_record_uuid 後 is_locked_uuid 應回 true");

    // ensure_not_locked_uuid 應回 Conflict
    let err = SignatureService::ensure_not_locked_uuid(&app.db_pool, "observation", observation_id)
        .await
        .expect_err("ensure_not_locked_uuid after lock should error");
    assert!(
        matches!(err, AppError::Conflict(_)),
        "鎖定後 ensure_not_locked_uuid 應回 Conflict，實得：{err:?}"
    );

    // DB 欄位也應被填寫
    let row: (bool, Option<chrono::DateTime<chrono::Utc>>, Option<Uuid>) = sqlx::query_as(
        "SELECT is_locked, locked_at, locked_by FROM animal_observations WHERE id = $1",
    )
    .bind(observation_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch observation lock cols");
    assert!(row.0, "is_locked 應為 true");
    assert!(row.1.is_some(), "locked_at 應已填寫");
    assert!(row.2.is_some(), "locked_by 應已填寫");
}

// ============================================================
// Test 2: PUT /observations/{id} 回 409 when locked
// ============================================================

#[tokio::test]
#[serial]
async fn update_locked_observation_returns_409() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let animal_id = create_test_animal(&app, &token).await;
    let observation_id = create_test_observation(&app, &token, &animal_id).await;

    force_lock_observation(&app, observation_id).await;

    let update_body = serde_json::json!({
        "content": "tampered content after lock"
    });
    let res = app
        .auth_put(
            &format!("/api/v1/observations/{}", observation_id),
            &update_body,
            &token,
        )
        .await;

    assert_eq!(
        res.status(),
        409,
        "已鎖定 observation 的 PUT 應回 409 Conflict，實得 {}",
        res.status()
    );
    let body: serde_json::Value = res.json().await.expect("parse error json");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("鎖定"), "錯誤訊息應提及『鎖定』，實得: {msg}");
}

// ============================================================
// Test 3: DELETE /observations/{id} 回 409 when locked
// ============================================================

#[tokio::test]
#[serial]
async fn delete_locked_observation_returns_409() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let animal_id = create_test_animal(&app, &token).await;
    let observation_id = create_test_observation(&app, &token, &animal_id).await;

    force_lock_observation(&app, observation_id).await;

    // 觀察刪除是 POST /observations/:id/delete（含原因）
    let delete_body = serde_json::json!({ "reason": "test" });
    let res = app
        .auth_post(
            &format!("/api/v1/observations/{}/delete", observation_id),
            &delete_body,
            &token,
        )
        .await;

    assert_eq!(
        res.status(),
        409,
        "已鎖定 observation 的刪除應回 409 Conflict，實得 {}",
        res.status()
    );
    let body: serde_json::Value = res.json().await.expect("parse error json");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("鎖定"), "錯誤訊息應提及『鎖定』，實得: {msg}");
}

// ============================================================
// Test 4: 反例：未鎖定可正常修改（確認 guard 不誤殺）
// ============================================================

#[tokio::test]
#[serial]
async fn update_unlocked_observation_returns_200() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let animal_id = create_test_animal(&app, &token).await;
    let observation_id = create_test_observation(&app, &token, &animal_id).await;

    let update_body = serde_json::json!({
        "content": "normal update before lock"
    });
    let res = app
        .auth_put(
            &format!("/api/v1/observations/{}", observation_id),
            &update_body,
            &token,
        )
        .await;

    assert!(
        res.status().is_success(),
        "未鎖定 observation 的 PUT 應成功，實得 {}",
        res.status()
    );
}

// ============================================================
// Test 5: 跨 process 耳號競態（2026-08-24 CodeRabbit 於 PR #17 指出）
// ============================================================

/// 「競爭者」子進程用的環境變數：要搶走的耳號。
const STEAL_ENV: &str = "IPIG_TEST_STEAL_EAR_TAG";

/// ⚠️ 這不是一般測試，是上面那支測試 spawn 出來的**另一個 OS process** 的進入點。
/// 標 `#[ignore]` 讓它不會在正常跑測試時被執行；只有被明確用
/// `--ignored --exact` 點名時才跑。
///
/// 它做的事：把 `STEAL_ENV` 指定的耳號搶先建立成一隻存活動物，
/// 藉此**真的**在主測試的「配置耳號」與「送出建立請求」之間製造衝突。
#[tokio::test]
#[ignore = "由 create_animal_survives_cross_process_ear_tag_theft spawn，不單獨執行"]
async fn cross_process_ear_tag_thief() {
    let Ok(tag) = std::env::var(STEAL_ENV) else {
        panic!("{STEAL_ENV} 未設定——本測試只該由主測試 spawn，不該手動執行");
    };
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL 必須設定（子進程繼承自主測試）");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("競爭者連不上測試 DB");

    // 直接 INSERT，不走 HTTP——競爭者只需要「佔住這個耳號」，
    // 用最少的必填欄位（ear_tag / breed / gender / entry_date）即可。
    sqlx::query(
        r#"INSERT INTO animals (ear_tag, status, breed, gender, entry_date)
           VALUES ($1, 'in_experiment'::animal_status, 'white', 'female', '2026-01-15')"#,
    )
    .bind(&tag)
    .execute(&pool)
    .await
    .expect("競爭者搶耳號失敗");
}

/// 真正的跨 process 回歸測試：確認 `create_animal_with_free_ear_tag` 在
/// **另一個 OS process** 於空隙中搶走耳號時仍能完成建立。
///
/// # 為什麼要真的開一個 process，而不是用執行緒模擬
///
/// CodeRabbit 指出的競態就是「另一個 process」——`free_ear_tag` 的 `AtomicI64`
/// 只序列化同 process 內的呼叫者，對跨 process 無效。用執行緒模擬會被那個
/// atomic 擋掉，反而測不到要測的東西。
///
/// # 為什麼是確定性的，不是「跑很多次希望撞到」
///
/// 競爭注入點就在 `build_body` 這個 callback 裡：第一次拿到耳號時同步
/// spawn 競爭者並**等它跑完**，再回傳 body。所以衝突必然發生、每次都發生，
/// 不依賴時序運氣。
#[tokio::test]
#[serial]
async fn create_animal_survives_cross_process_ear_tag_theft() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let attempts = AtomicUsize::new(0);
    let stolen: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

    let animal = common::create_animal_with_free_ear_tag(&app, &token, |ear_tag| {
        // 只在第一次注入競爭：模擬「配置到耳號後、送出請求前被搶走」。
        if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
            *stolen.lock().expect("stolen lock") = Some(ear_tag.to_string());

            let exe = std::env::current_exe().expect("取不到目前測試 binary 路徑");
            let status = std::process::Command::new(exe)
                .args(["--exact", "cross_process_ear_tag_thief", "--ignored"])
                .env(STEAL_ENV, ear_tag)
                .status()
                .expect("spawn 競爭者 process 失敗");
            assert!(
                status.success(),
                "競爭者 process 應成功搶走耳號，實得 {status}"
            );
        }
        animal_body(ear_tag)
    })
    .await;

    // ① 必須至少試了兩次——第一次被搶走、第二次才成功。
    //    若這裡是 1，代表衝突根本沒發生，這個測試就沒有驗到任何東西。
    let n = attempts.load(Ordering::SeqCst);
    assert!(
        n >= 2,
        "應該至少重試過一次（第一次的耳號被另一個 process 搶走），實際只嘗試 {n} 次——\
         代表競爭沒有真的發生，本測試失去意義"
    );

    // ② 最終真的建立成功，且用的不是被搶走的那個耳號。
    let created_tag = animal["ear_tag"]
        .as_str()
        .expect("建立回應應含 ear_tag")
        .to_string();
    let stolen_tag = stolen
        .lock()
        .expect("stolen lock")
        .clone()
        .expect("應已記錄被搶的耳號");
    assert_ne!(
        created_tag, stolen_tag,
        "最終建立用的耳號不該是被搶走的那個"
    );
}

/// 端到端 smoke：連續多次配置都拿到相異耳號、中途不 panic。
///
/// ⚠️ **這支測試不是 OFFSET 那個 bug 的回歸測試**，別把它當成那個用
/// （2026-08-24 CodeRabbit 於 PR #17 指出，我原本的註解就是這樣寫錯的）。
///
/// 理由：舊版的 `OFFSET` 在第 N 次配置用偏移量 `N-1`，而候選集合當時還剩
/// `900-(N-1)` 個。兩者要相遇得等到**第 451 次**。只跑 12 次的話，舊版用的是
/// 偏移量 0~11、集合 900~889，**12 次全部會成功**——把有 bug 的實作放回來，
/// 這支測試照樣綠。
///
/// 真正鎖住那個 bug 的是 `offset_allocation_breaks_at_midpoint`（純函式單元測試，
/// 毫秒級就能涵蓋第 451 次），本測試只負責驗證「端到端跑得動」。
#[tokio::test]
#[serial]
async fn repeated_allocation_end_to_end_smoke() {
    use std::collections::HashSet;

    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let mut seen: HashSet<String> = HashSet::new();
    for i in 1..=12 {
        let animal = common::create_animal_with_free_ear_tag(&app, &token, animal_body).await;
        let tag = animal["ear_tag"]
            .as_str()
            .unwrap_or_else(|| panic!("第 {i} 次建立的回應缺少 ear_tag"))
            .to_string();
        assert!(
            seen.insert(tag.clone()),
            "第 {i} 次配到重複的耳號 {tag}——配置機制失效"
        );
    }

    assert_eq!(seen.len(), 12, "12 次建立應得到 12 個相異耳號");
}

/// 真正的回歸測試：把舊版 `OFFSET` 的配置邏輯當純函式重現，證明它會在
/// 池子還有一半空號時就誤報用盡；同時證明現行做法（不帶 OFFSET）不會。
///
/// 為什麼不用真的建 451 隻動物：那要跑幾分鐘、還會把共用測試庫的耳號池
/// 用掉一半，影響同一顆 DB 上的其他測試。這個 bug 的成因純粹是
/// 「單調遞增的偏移量」對上「遞減的候選集合」，不依賴任何 DB 行為，
/// 用純函式重現能完整涵蓋而且是毫秒級。
#[test]
fn offset_allocation_breaks_at_midpoint() {
    /// 重現舊版：第 n 次配置取 `ORDER BY g OFFSET (n-1) LIMIT 1`。
    /// 回傳第一次取不到值（＝會 panic 說「耳號用盡」）的次數，None 表示都沒事。
    fn first_failure_with_offset(pool_size: usize, attempts: usize) -> Option<usize> {
        let mut remaining = pool_size;
        for n in 0..attempts {
            // OFFSET n 打進只剩 remaining 列的集合：n >= remaining 就回 None
            if n >= remaining {
                return Some(n + 1);
            }
            remaining -= 1; // 該次配置成功建立一隻，池子少一個
        }
        None
    }

    /// 現行做法：永遠取第一個空號，沒有偏移量。
    fn first_failure_without_offset(pool_size: usize, attempts: usize) -> Option<usize> {
        let mut remaining = pool_size;
        for n in 0..attempts {
            if remaining == 0 {
                return Some(n + 1);
            }
            remaining -= 1;
        }
        None
    }

    // ① 舊版在 900 個空號的池子上，第 451 次就誤報用盡——當下還有 450 個可用。
    assert_eq!(
        first_failure_with_offset(900, 900),
        Some(451),
        "舊版 OFFSET 實作應在第 451 次誤報耳號用盡"
    );

    // ② 現行做法要真的用完 900 個才會回報用盡，這時是誠實的。
    assert_eq!(
        first_failure_without_offset(900, 900),
        None,
        "現行做法在池子還有空號時不該回報用盡"
    );
    assert_eq!(
        first_failure_without_offset(900, 901),
        Some(901),
        "現行做法只在真的用完 900 個之後才回報用盡"
    );

    // ③ 這正是為什麼上面那支 12 次的端到端測試抓不到這個 bug——
    //    舊版在 12 次之內完全正常。留這個斷言，避免有人日後又把它當回歸測試用。
    assert_eq!(
        first_failure_with_offset(900, 12),
        None,
        "舊版在只跑 12 次時不會失敗——證明 12 次的端到端測試無法涵蓋此 bug"
    );
}
