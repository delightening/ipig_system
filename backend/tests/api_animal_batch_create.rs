//! 批次新增動物（`POST /api/v1/animals/batch`，批量進豬）。
//!
//! 要釘住的契約：
//! 1. 全部合法 → 全部建立，且每隻都有自己的 `ANIMAL_CREATE` 稽核、整批另有一筆
//!    `ANIMAL_BATCH_CREATE` 摘要。
//! 2. 部分成功：一隻不合法不影響其他隻；結果以 `index` 對回請求順序。
//! 3. 耳號重複沿用單隻規則：僅耳號相同 → `duplicate_warning`（帶 `force_create` 可建立）；
//!    同耳號且同出生日期 → `failed`，`force_create` 也跳不過。
//! 4. 欄位容量沿用單隻規則：放不下的那幾隻 `failed`，放得下的照建。
//! 5. 請求本身不合法（空陣列、超過單次上限）回 400；未登入回 401。

mod common;

use serde_json::{json, Value};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use common::TestApp;

/// 與 `services/animal/core/batch_create.rs` 的 `BATCH_CREATE_MAX_ANIMALS` 同值。
const BATCH_MAX: usize = 50;

fn animal(ear_tag: &str) -> Value {
    json!({
        "ear_tag": ear_tag,
        "gender": "male",
        "breed": "minipig",
        "entry_date": "2026-07-13",
        "entry_weight": 25.5,
        "pen_location": "A-01",
    })
}

/// 一次配置 n 個彼此不同、且目前沒有非終態動物在用的三位數耳號。
///
/// 查詢條件與 `guard_duplicate_ear_tag` 一致；各測試皆 `#[serial]`，不會互搶。
async fn free_ear_tags(pool: &PgPool, n: i64) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT lpad(g::text, 3, '0')
        FROM generate_series($1::int, $2::int) g
        WHERE NOT EXISTS (
            SELECT 1 FROM animals a
            WHERE a.ear_tag = lpad(g::text, 3, '0')
              AND a.deleted_at IS NULL
              AND a.status NOT IN ('euthanized', 'sudden_death')
        )
        ORDER BY g
        LIMIT $3
        "#,
    )
    .bind(common::EAR_TAG_MIN as i32)
    .bind(common::EAR_TAG_MAX as i32)
    .bind(n)
    .fetch_all(pool)
    .await
    .expect("query free ear tags")
}

async fn post_batch(app: &TestApp, token: &str, animals: Vec<Value>) -> (u16, Value) {
    let res = app
        .auth_post(
            "/api/v1/animals/batch",
            &json!({ "animals": animals }),
            token,
        )
        .await;
    let status = res.status().as_u16();
    let body: Value = res.json().await.unwrap_or(Value::Null);
    (status, body)
}

async fn count_audit(pool: &PgPool, event_type: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM user_activity_logs WHERE event_type = $1")
        .bind(event_type)
        .fetch_one(pool)
        .await
        .expect("count audit rows")
}

/// 建最小 facility→building→zone→pen 鏈，回傳指定容量的 pen id。
async fn seed_pen_with_capacity(pool: &PgPool, capacity: i32) -> Uuid {
    let (fid, bid, zid, pid) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '批次新增測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '批次新增測試棟')",
    )
    .bind(bid)
    .bind(fid)
    .bind(format!("B{}", &s[..8]))
    .execute(pool)
    .await
    .expect("insert building");
    sqlx::query("INSERT INTO zones (id, building_id, code) VALUES ($1, $2, $3)")
        .bind(zid)
        .bind(bid)
        .bind(format!("Z{}", &s[..8]))
        .execute(pool)
        .await
        .expect("insert zone");
    sqlx::query("INSERT INTO pens (id, zone_id, code, capacity) VALUES ($1, $2, $3, $4)")
        .bind(pid)
        .bind(zid)
        .bind(format!("P{}", &s[..8]))
        .bind(capacity)
        .execute(pool)
        .await
        .expect("insert pen");
    pid
}

#[tokio::test]
#[serial]
async fn batch_create_creates_every_animal_with_audit() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let tags = free_ear_tags(&app.db_pool, 3).await;
    let summary_before = count_audit(&app.db_pool, "ANIMAL_BATCH_CREATE").await;

    let (status, body) = post_batch(&app, &token, tags.iter().map(|t| animal(t)).collect()).await;

    assert_eq!(status, 200, "實際回應 {body}");
    assert_eq!(body["created_count"], 3, "{body}");
    assert_eq!(body["not_created_count"], 0, "{body}");
    let results = body["results"].as_array().expect("results 應為陣列");
    assert_eq!(results.len(), 3);
    let mut ids = Vec::new();
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r["index"], i, "結果須依請求順序");
        assert_eq!(r["ear_tag"], tags[i]);
        assert_eq!(r["outcome"], "created", "{r}");
        ids.push(Uuid::parse_str(r["animal_id"].as_str().expect("animal_id")).expect("uuid"));
    }

    let in_db: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE id = ANY($1) AND ear_tag = ANY($2)")
            .bind(&ids)
            .bind(&tags)
            .fetch_one(&app.db_pool)
            .await
            .expect("count animals");
    assert_eq!(in_db, 3, "三隻都應真的寫進資料庫");

    let per_row: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = 'ANIMAL_CREATE' AND entity_id = ANY($1)",
    )
    .bind(&ids)
    .fetch_one(&app.db_pool)
    .await
    .expect("count per-row audit");
    assert_eq!(per_row, 3, "每隻動物都應有自己的 ANIMAL_CREATE 稽核");

    let summary_after = count_audit(&app.db_pool, "ANIMAL_BATCH_CREATE").await;
    assert_eq!(summary_after - summary_before, 1, "整批只記一筆摘要稽核");
}

#[tokio::test]
#[serial]
async fn batch_create_is_partial_success_when_one_row_is_invalid() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let tags = free_ear_tags(&app.db_pool, 2).await;

    // 中間那隻耳號不是數字 → 驗證不過；前後兩隻照建
    let (status, body) = post_batch(
        &app,
        &token,
        vec![animal(&tags[0]), animal("abc"), animal(&tags[1])],
    )
    .await;

    assert_eq!(status, 200, "部分失敗仍應回 200，實際 {body}");
    assert_eq!(body["created_count"], 2, "{body}");
    assert_eq!(body["not_created_count"], 1, "{body}");
    let results = body["results"].as_array().expect("results");
    assert_eq!(results[0]["outcome"], "created");
    assert_eq!(results[1]["outcome"], "failed");
    assert_eq!(results[1]["index"], 1);
    assert_eq!(results[1]["ear_tag"], "abc");
    assert!(
        results[1]["message"]
            .as_str()
            .is_some_and(|m| m.contains("耳號必須為三位數")),
        "失敗原因應可讀，實際 {}",
        results[1]
    );
    assert_eq!(results[2]["outcome"], "created");

    let created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM animals WHERE ear_tag = ANY($1) AND deleted_at IS NULL",
    )
    .bind(&tags)
    .fetch_one(&app.db_pool)
    .await
    .expect("count animals");
    assert_eq!(created, 2, "失敗的那隻不應影響其他兩隻");
}

#[tokio::test]
#[serial]
async fn batch_create_warns_on_same_ear_tag_different_birth_date_and_accepts_force() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let tag = free_ear_tags(&app.db_pool, 1).await.remove(0);

    // 先建一隻有出生日期的
    let mut first = animal(&tag);
    first["birth_date"] = json!("2025-01-01");
    let (status, body) = post_batch(&app, &token, vec![first]).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["created_count"], 1, "{body}");

    // 同耳號、不同出生日期、沒帶 force_create → 警告、不建立
    let mut second = animal(&tag);
    second["birth_date"] = json!("2025-02-02");
    let (_, body) = post_batch(&app, &token, vec![second.clone()]).await;
    let r = &body["results"][0];
    assert_eq!(r["outcome"], "duplicate_warning", "{body}");
    assert!(r["animal_id"].is_null());
    assert!(
        r["existing_animals"]
            .as_array()
            .is_some_and(|a| !a.is_empty()),
        "應附上與之重複的既有動物，實際 {r}"
    );
    assert_eq!(body["created_count"], 0);
    assert_eq!(body["not_created_count"], 1);

    // 勾選確認（force_create）後才建立
    second["force_create"] = json!(true);
    let (_, body) = post_batch(&app, &token, vec![second]).await;
    assert_eq!(body["results"][0]["outcome"], "created", "{body}");
}

#[tokio::test]
#[serial]
async fn batch_create_blocks_same_ear_tag_and_same_birth_date_even_with_force() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let tag = free_ear_tags(&app.db_pool, 1).await.remove(0);

    let mut first = animal(&tag);
    first["birth_date"] = json!("2025-03-03");
    let (_, body) = post_batch(&app, &token, vec![first.clone()]).await;
    assert_eq!(body["created_count"], 1, "{body}");

    first["force_create"] = json!(true);
    let (status, body) = post_batch(&app, &token, vec![first]).await;
    assert_eq!(status, 200);
    let r = &body["results"][0];
    assert_eq!(r["outcome"], "failed", "force_create 也跳不過：{body}");
    assert!(
        r["message"]
            .as_str()
            .is_some_and(|m| m.contains("已存在同出生日期")),
        "實際 {r}"
    );
}

#[tokio::test]
#[serial]
async fn batch_create_respects_pen_capacity_per_animal() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let tags = free_ear_tags(&app.db_pool, 3).await;
    let pen_id = seed_pen_with_capacity(&app.db_pool, 2).await;

    let rows: Vec<Value> = tags
        .iter()
        .map(|t| {
            let mut a = animal(t);
            a["pen_id"] = json!(pen_id);
            a
        })
        .collect();
    let (status, body) = post_batch(&app, &token, rows).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(body["created_count"], 2, "容量 2 只放得下前兩隻：{body}");
    assert_eq!(body["not_created_count"], 1, "{body}");
    let results = body["results"].as_array().expect("results");
    assert_eq!(results[0]["outcome"], "created");
    assert_eq!(results[1]["outcome"], "created");
    assert_eq!(results[2]["outcome"], "failed");
    assert!(
        results[2]["message"]
            .as_str()
            .is_some_and(|m| m.contains("已滿")),
        "應說明欄位已滿，實際 {}",
        results[2]
    );
}

#[tokio::test]
#[serial]
async fn batch_create_rejects_empty_and_oversized_requests() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (status, _) = post_batch(&app, &token, vec![]).await;
    assert_eq!(status, 400, "空陣列應回 400");

    let too_many: Vec<Value> = (0..=BATCH_MAX).map(|_| animal("100")).collect();
    let (status, _) = post_batch(&app, &token, too_many).await;
    assert_eq!(status, 400, "超過單次上限應回 400，且不得寫入任何動物");
}

#[tokio::test]
#[serial]
async fn batch_create_requires_authentication() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/animals/batch"))
        .json(&json!({ "animals": [animal("100")] }))
        .send()
        .await
        .expect("send request");
    assert_eq!(res.status().as_u16(), 401);
}
