//! 整合測試：匯入體重存活防呆（`CreateWeightRequest.enforce_active`）。
//!
//! - 場內存活動物 + `enforce_active=true` → 成功
//! - 已安樂死動物 + `enforce_active=true`（匯入體重對話框路徑）→ 400 擋下
//! - 已安樂死動物 + 不送 `enforce_active`（動物詳情頁單筆登錄路徑）→ 成功放行
//!   （容許死亡當下補登最後體重）

mod common;

use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn weight_enforce_active_blocks_euthanized_but_detail_page_allows() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 建立一隻存活動物。耳號用「配置」而非亂數：三位數只有 900 個值，抽籤在共用
    // 測試 DB 上必然碰撞而回 409（force_create 跳不過重複守衛）。詳見
    // common::free_ear_tag。
    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let create_body = serde_json::json!({
        "ear_tag": ear_tag,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true
    });
    let created = app.auth_post("/api/v1/animals", &create_body, &token).await;
    assert!(
        created.status() == 200 || created.status() == 201,
        "建立動物失敗：{}",
        created.status()
    );
    let created: serde_json::Value = created.json().await.expect("parse create response");
    let animal_id = created["id"]
        .as_str()
        .expect("created animal id")
        .to_string();
    let path = format!("/api/v1/animals/{}/weights", animal_id);

    let body_enforce =
        serde_json::json!({ "measure_date": "2026-06-30", "weight": 30.0, "enforce_active": true });

    // 1) 存活動物 + enforce_active=true → 成功
    let r_alive = app.auth_post(&path, &body_enforce, &token).await;
    assert!(
        r_alive.status() == 200 || r_alive.status() == 201,
        "存活動物匯入體重應成功，得到 {}",
        r_alive.status()
    );

    // 直接改 DB 設為已安樂死（跳過犧牲單業務流程，聚焦驗證 guard）
    sqlx::query("UPDATE animals SET status = 'euthanized' WHERE id = $1")
        .bind(Uuid::parse_str(&animal_id).expect("valid uuid"))
        .execute(&app.db_pool)
        .await
        .expect("set animal euthanized");

    // 2) 已死亡 + enforce_active=true（匯入路徑）→ 400 擋下
    let r_block = app.auth_post(&path, &body_enforce, &token).await;
    assert_eq!(r_block.status(), 400, "匯入路徑應擋下已死亡動物的體重登錄");

    // 3) 已死亡 + 不送 enforce_active（詳情頁路徑）→ 成功放行
    let body_detail = serde_json::json!({ "measure_date": "2026-06-30", "weight": 31.0 });
    let r_allow = app.auth_post(&path, &body_detail, &token).await;
    assert!(
        r_allow.status() == 200 || r_allow.status() == 201,
        "詳情頁應放行死亡動物補登最後體重，得到 {}",
        r_allow.status()
    );
}
