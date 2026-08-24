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

/// 建立一隻動物，回傳 animal_id (UUID 字串)。
///
/// R109（2026-08-24）：原本用 `rand::random` 各自隨機挑 `ear_tag`（3 位數，
/// 100–999 共 900 個值）與 `entry_date`（月中 28 天），組合空間只有 28,000，
/// CI 上共用測試庫累積跑下來會生日碰撞——這個測試檔本身在 2026-08-24 就撞過一次
/// （`create animal failed: 409 Conflict … 耳號 100 已存在同出生日期的存活動物`）。
///
/// 改用 `common::free_ear_tag`：它用「配置」而非「抽籤」，查詢當下保證該耳號
/// **沒有任何非終態動物在用**，故不會撞。`entry_date` 改回固定值——保證不撞的是
/// 耳號本身（守衛的查詢條件不含 birth_date），固定日期不影響正確性，且跟
/// `api_animals.rs` / `api_vaccination_validation.rs` 等既有測試用同一套慣例。
async fn create_test_animal(app: &TestApp, token: &str) -> String {
    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let body = serde_json::json!({
        "ear_tag": ear_tag,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true
    });
    let res = app.auth_post("/api/v1/animals", &body, token).await;
    let status = res.status();
    let body_text = res.text().await.unwrap_or_default();
    assert!(
        status.is_success(),
        "create animal failed: {} body={}",
        status,
        body_text
    );
    let json: serde_json::Value = serde_json::from_str(&body_text).expect("parse animal json");
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
