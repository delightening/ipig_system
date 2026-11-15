//! 欄位地板面積：依動物體重與同欄隻數判斷欄位是否超標（只警示，不擋操作）。
//!
//! 涵蓋：尺寸的建立／驗證、欄位清單的 `floor_area`、放入預覽、套用尺寸到整個區、
//! 有面積的欄位不再被整數容量擋、沒面積的欄位仍用整數容量、完全沒體重的動物不可放入。
//!
//! 欄位代碼一律用隨機的 `T` 開頭代碼；所有資料都是合成的。

mod common;

use std::str::FromStr;

use common::TestApp;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

/// 在既有的區底下建一個啟用中的測試欄位（可帶長寬），回傳 (pen_id, code)。
async fn make_pen(
    app: &TestApp,
    zone_id: Uuid,
    capacity: i32,
    dimensions: Option<(&str, &str)>,
) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let code = format!("T{}", id.simple().to_string()[..6].to_uppercase());
    let length = dimensions.map(|(l, _)| Decimal::from_str(l).expect("length"));
    let width = dimensions.map(|(_, w)| Decimal::from_str(w).expect("width"));
    sqlx::query(
        "INSERT INTO pens (id, zone_id, code, capacity, current_count, length_m, width_m) \
         VALUES ($1, $2, $3, $4, 0, $5, $6)",
    )
    .bind(id)
    .bind(zone_id)
    .bind(&code)
    .bind(capacity)
    .bind(length)
    .bind(width)
    .execute(&app.db_pool)
    .await
    .expect("insert pen");
    (id, code)
}

/// 建立一個專屬的區（避免與共用的固定區裡其他測試的欄位混在一起）。
async fn make_zone(app: &TestApp) -> Uuid {
    let (_, building) = common::ensure_fixture_pen(&app.db_pool).await;
    let zone = Uuid::new_v4();
    sqlx::query("INSERT INTO zones (id, building_id, code) VALUES ($1, $2, $3)")
        .bind(zone)
        .bind(building)
        .bind(format!("ZA{}", &zone.simple().to_string()[..6]))
        .execute(&app.db_pool)
        .await
        .expect("insert zone");
    zone
}

fn animal_body(ear_tag: &str, pen_code: &str, weight: f64) -> Value {
    json!({
        "ear_tag": ear_tag,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": weight,
        "pen_location": pen_code,
        "force_create": true
    })
}

async fn create_in(app: &TestApp, token: &str, pen_code: &str, weight: f64) -> Value {
    common::create_animal_with_free_ear_tag(app, token, |ear_tag| {
        animal_body(ear_tag, pen_code, weight)
    })
    .await
}

/// 十進位欄位可能以字串或數字出現，統一轉成 Decimal 比對。
fn dec(value: &Value) -> Decimal {
    let text = match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    Decimal::from_str(&text).unwrap_or_else(|_| panic!("不是十進位數字：{value}"))
}

fn d(text: &str) -> Decimal {
    Decimal::from_str(text).expect("測試用的十進位數字")
}

async fn floor_area_of(app: &TestApp, token: &str, zone_id: Uuid, pen_id: Uuid) -> Value {
    let res = app
        .auth_get(&format!("/api/v1/facilities/pens?zone_id={zone_id}"), token)
        .await;
    assert_eq!(res.status().as_u16(), 200);
    let pens: Vec<Value> = res.json().await.expect("pens json");
    let pen = pens
        .into_iter()
        .find(|p| p["id"] == pen_id.to_string())
        .expect("欄位應在清單內");
    pen["floor_area"].clone()
}

// ---------- 尺寸的建立與驗證 ----------

#[tokio::test]
#[serial]
async fn create_pen_accepts_dimensions_and_rejects_bad_ones() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;

    let code = format!(
        "T{}",
        Uuid::new_v4().simple().to_string()[..6].to_uppercase()
    );
    let res = app
        .auth_post(
            "/api/v1/facilities/pens",
            &json!({ "zone_id": zone, "code": code, "length_m": "1.20", "width_m": "2.40" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 201, "body={:?}", res.text().await);

    // 前端也可能送 JSON 數字（不是字串）：照樣收，並存成兩位小數
    let res = app
        .auth_post(
            "/api/v1/facilities/pens",
            &json!({ "zone_id": zone, "code": format!("{code}N"), "length_m": 1.2, "width_m": 2.4 }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 201, "body={:?}", res.text().await);
    let created: Value = res.json().await.expect("pen json");
    assert_eq!(dec(&created["length_m"]), d("1.20"));
    assert_eq!(dec(&created["width_m"]), d("2.40"));

    // 只給其中一邊 → 400
    let res = app
        .auth_post(
            "/api/v1/facilities/pens",
            &json!({ "zone_id": zone, "code": format!("{code}X"), "length_m": "1.20" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "只給長度應回 400");

    // 超過兩位小數 → 400
    let res = app
        .auth_post(
            "/api/v1/facilities/pens",
            &json!({ "zone_id": zone, "code": format!("{code}Y"), "length_m": "1.234", "width_m": "2.4" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "三位小數應回 400");

    // 非正數 → 400
    let res = app
        .auth_post(
            "/api/v1/facilities/pens",
            &json!({ "zone_id": zone, "code": format!("{code}Z"), "length_m": "0", "width_m": "2.4" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400, "0 應回 400");
}

#[tokio::test]
#[serial]
async fn update_pen_sets_dimensions_and_leaves_them_alone_when_omitted() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (pen, _) = make_pen(&app, zone, 4, None).await;

    let res = app
        .auth_put(
            &format!("/api/v1/facilities/pens/{pen}"),
            &json!({ "length_m": "1.20", "width_m": "2.40" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);
    let updated: Value = res.json().await.expect("pen json");
    assert_eq!(dec(&updated["length_m"]), d("1.20"));

    // 只改別的欄位：尺寸維持
    let res = app
        .auth_put(
            &format!("/api/v1/facilities/pens/{pen}"),
            &json!({ "name": "改名" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200);
    let after: Value = res.json().await.expect("pen json");
    assert_eq!(dec(&after["width_m"]), d("2.40"));

    // 只給一邊 → 400，且不動資料
    let res = app
        .auth_put(
            &format!("/api/v1/facilities/pens/{pen}"),
            &json!({ "width_m": "3.00" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400);
}

// ---------- 清單的 floor_area ----------

#[tokio::test]
#[serial]
async fn list_pens_reports_floor_area_and_integer_capacity_no_longer_blocks_a_pen_with_area() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    // 整數容量故意設成 1：有面積的欄位不該再被它擋
    let (pen, code) = make_pen(&app, zone, 1, Some(("1.20", "2.40"))).await;

    for _ in 0..3 {
        create_in(&app, &token, &code, 30.0).await;
    }
    // 3 隻 30 kg：3 × 0.90 = 2.70 ≤ 2.88
    let area = floor_area_of(&app, &token, zone, pen).await;
    assert_eq!(area["status"], "ok");
    assert_eq!(area["animal_count"], 3);
    assert_eq!(area["animals_missing_weight"], 0);
    assert_eq!(dec(&area["required_area_m2"]), d("2.70"));
    assert_eq!(dec(&area["area_m2"]), d("2.88"));

    // 第 4 隻：不被擋（只警示），欄位變成超標
    create_in(&app, &token, &code, 30.0).await;
    let area = floor_area_of(&app, &token, zone, pen).await;
    assert_eq!(area["status"], "exceeded");
    assert_eq!(area["animal_count"], 4);
}

#[tokio::test]
#[serial]
async fn floor_area_uses_the_latest_weight_record_over_entry_weight() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (pen, code) = make_pen(&app, zone, 4, Some(("1.20", "2.40"))).await;

    let animal = create_in(&app, &token, &code, 20.0).await;
    let animal_id: Uuid = animal["id"].as_str().expect("id").parse().expect("uuid");
    // 進場 20 kg 單獨需要 1.08；之後長到 120 kg 的體重紀錄，單獨需要 4.32 > 2.88 → 超標
    sqlx::query(
        "INSERT INTO animal_weights (animal_id, measure_date, weight) VALUES ($1, '2026-02-01', 120.0)",
    )
    .bind(animal_id)
    .execute(&app.db_pool)
    .await
    .expect("insert weight");

    let area = floor_area_of(&app, &token, zone, pen).await;
    assert_eq!(area["status"], "exceeded");
    assert_eq!(dec(&area["required_area_m2"]), d("4.32"));

    // 軟刪除最新那筆 → 退回進場體重
    sqlx::query("UPDATE animal_weights SET deleted_at = now() WHERE animal_id = $1")
        .bind(animal_id)
        .execute(&app.db_pool)
        .await
        .expect("soft delete weight");
    let area = floor_area_of(&app, &token, zone, pen).await;
    assert_eq!(area["status"], "ok");
    assert_eq!(dec(&area["required_area_m2"]), d("1.08"));
}

#[tokio::test]
#[serial]
async fn pen_without_dimensions_reports_no_area_and_keeps_the_integer_capacity_limit() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (pen, code) = make_pen(&app, zone, 1, None).await;

    create_in(&app, &token, &code, 30.0).await;
    let area = floor_area_of(&app, &token, zone, pen).await;
    assert_eq!(area["status"], "no_area");
    assert!(area["area_m2"].is_null());

    // 沒填長寬的欄位：整數容量仍是硬限制
    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let res = app
        .auth_post(
            "/api/v1/animals",
            &animal_body(&ear_tag, &code, 30.0),
            &token,
        )
        .await;
    assert!(
        res.status().is_client_error(),
        "沒有面積的滿欄應被整數容量擋下，實際 {}",
        res.status()
    );
}

// ---------- 放入預覽 ----------

#[tokio::test]
#[serial]
async fn area_preview_answers_what_if_without_writing_anything() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (pen, _) = make_pen(&app, zone, 4, Some(("1.20", "2.40"))).await;

    let preview = |query: String| {
        let app = &app;
        let token = &token;
        async move {
            let res = app
                .auth_get(
                    &format!("/api/v1/facilities/pens/{pen}/area-preview{query}"),
                    token,
                )
                .await;
            assert_eq!(res.status().as_u16(), 200);
            res.json::<Value>().await.expect("preview json")
        }
    };

    // 不給體重：空欄位的目前狀態
    let now = preview(String::new()).await;
    assert_eq!(now["status"], "ok");
    assert_eq!(now["animal_count"], 0);

    // 再放一隻 60 kg：單獨 2.16 ≤ 2.88
    let fits = preview("?add_weight_kg=60".to_string()).await;
    assert_eq!(fits["status"], "ok");
    assert_eq!(fits["animal_count"], 1);

    // 再放一隻 120 kg：單獨 4.32 > 2.88 → 預計超標
    let over = preview("?add_weight_kg=120".to_string()).await;
    assert_eq!(over["status"], "exceeded");

    // 預覽不寫入：欄位仍是空的
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE pen_id = $1")
        .bind(pen)
        .fetch_one(&app.db_pool)
        .await
        .expect("count");
    assert_eq!(count, 0);

    // 不存在的欄位 → 404
    let res = app
        .auth_get(
            &format!("/api/v1/facilities/pens/{}/area-preview", Uuid::new_v4()),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 404);
}

// ---------- 套用尺寸到整個區 ----------

#[tokio::test]
#[serial]
async fn apply_pen_size_updates_only_active_pens_of_that_zone_and_writes_one_audit_event() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let other_zone = make_zone(&app).await;
    let (active_a, _) = make_pen(&app, zone, 4, None).await;
    let (active_b, _) = make_pen(&app, zone, 4, Some(("9.00", "9.00"))).await;
    let (inactive, _) = make_pen(&app, zone, 4, None).await;
    sqlx::query("UPDATE pens SET is_active = false WHERE id = $1")
        .bind(inactive)
        .execute(&app.db_pool)
        .await
        .expect("deactivate");
    let (other_zone_pen, _) = make_pen(&app, other_zone, 4, None).await;

    let res = app
        .auth_post(
            &format!("/api/v1/facilities/zones/{zone}/apply-pen-size"),
            &json!({ "length_m": "1.20", "width_m": "2.40" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);
    let result: Value = res.json().await.expect("result json");
    assert_eq!(result["updated_count"], 2, "只更新該區啟用中的欄位");

    let dims = |pen: Uuid| {
        let pool = app.db_pool.clone();
        async move {
            sqlx::query_as::<_, (Option<Decimal>, Option<Decimal>)>(
                "SELECT length_m, width_m FROM pens WHERE id = $1",
            )
            .bind(pen)
            .fetch_one(&pool)
            .await
            .expect("dims")
        }
    };
    let expected = (Some(d("1.20")), Some(d("2.40")));
    assert_eq!(dims(active_a).await, expected);
    assert_eq!(dims(active_b).await, expected, "原本有別的尺寸也會被覆蓋");
    assert_eq!(dims(inactive).await, (None, None), "停用的欄位不動");
    assert_eq!(dims(other_zone_pen).await, (None, None), "別的區不動");

    let events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'FACILITY_PEN_APPLY_SIZE' AND entity_id = $1",
    )
    .bind(zone)
    .fetch_one(&app.db_pool)
    .await
    .expect("audit count");
    assert_eq!(events, 1, "一次套用只寫一筆稽核事件");

    // 不合理的尺寸 → 400；不存在的區 → 404
    let res = app
        .auth_post(
            &format!("/api/v1/facilities/zones/{zone}/apply-pen-size"),
            &json!({ "length_m": "0", "width_m": "2.40" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 400);
    let res = app
        .auth_post(
            &format!("/api/v1/facilities/zones/{}/apply-pen-size", Uuid::new_v4()),
            &json!({ "length_m": "1.20", "width_m": "2.40" }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 404);
}

// ---------- 完全沒體重的動物不可放入 ----------

#[tokio::test]
#[serial]
async fn creating_an_animal_with_a_non_positive_entry_weight_is_rejected() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (pen, code) = make_pen(&app, zone, 4, Some(("1.20", "2.40"))).await;

    for weight in [0.0, -5.0] {
        let ear_tag = common::free_ear_tag(&app.db_pool).await;
        let res = app
            .auth_post(
                "/api/v1/animals",
                &animal_body(&ear_tag, &code, weight),
                &token,
            )
            .await;
        assert_eq!(
            res.status().as_u16(),
            400,
            "進場體重 {weight} 應回 400，實際 {}",
            res.status()
        );
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE pen_id = $1")
        .bind(pen)
        .fetch_one(&app.db_pool)
        .await
        .expect("count");
    assert_eq!(count, 0, "被拒絕的建立不可留下動物");
}

#[tokio::test]
#[serial]
async fn moving_an_animal_without_any_weight_is_rejected_but_echoing_its_pen_is_not() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let zone = make_zone(&app).await;
    let (_, from_code) = make_pen(&app, zone, 4, Some(("1.20", "2.40"))).await;
    let (to_pen, to_code) = make_pen(&app, zone, 4, Some(("1.20", "2.40"))).await;

    let animal = create_in(&app, &token, &from_code, 30.0).await;
    let id = animal["id"].as_str().expect("animal id");
    // 模擬舊資料：沒有進場體重、也沒有任何體重紀錄
    sqlx::query("UPDATE animals SET entry_weight = NULL WHERE id = $1::uuid")
        .bind(id)
        .execute(&app.db_pool)
        .await
        .expect("clear entry weight");
    // 建立動物時系統會自動寫一筆體重紀錄（來源是進場體重），也要清掉才算「完全沒有體重」
    sqlx::query("DELETE FROM animal_weights WHERE animal_id = $1::uuid")
        .bind(id)
        .execute(&app.db_pool)
        .await
        .expect("clear weight records");

    // 搬到別的欄位 → 被擋
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": to_code }),
            &token,
        )
        .await;
    assert!(
        res.status().is_client_error(),
        "沒有任何體重的動物不可換欄位，實際 {}",
        res.status()
    );

    // 原欄位原樣回填（沒換欄位）→ 不受影響
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": from_code }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);

    // 補上一筆體重紀錄後就可以搬
    sqlx::query(
        "INSERT INTO animal_weights (animal_id, measure_date, weight) VALUES ($1::uuid, '2026-02-01', 31.0)",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("insert weight");
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{id}"),
            &json!({ "pen_location": to_code }),
            &token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "body={:?}", res.text().await);
    let after: Value = app
        .auth_get(&format!("/api/v1/animals/{id}"), &token)
        .await
        .json()
        .await
        .expect("animal json");
    assert_eq!(after["pen_id"], to_pen.to_string());
}
