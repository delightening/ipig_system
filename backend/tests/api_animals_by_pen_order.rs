//! 欄位頁（`GET /api/v1/animals/by-pen`）：欄位內的動物依耳號由小到大排列。
//!
//! 修復前 `ORDER BY p.pen_location, p.id`——`id` 是 UUID，欄位內的順序是任意的。
//! 耳號在資料庫有「必為三位數字」的限制，所以字串序＝數字序（099 排在 100 前面）。
//!
//! 測試刻意讓 **UUID 順序與耳號順序相反**（耳號最大的動物 UUID 最小），
//! 舊的排序一定會把耳號倒過來，所以這支測試在舊程式碼下必為紅燈，不靠運氣。

mod common;

use common::TestApp;
use serde_json::Value;
use serial_test::serial;
use uuid::Uuid;

/// 取 n 個目前沒有非終態動物在用的三位數耳號（由小到大）。
async fn free_ear_tags(app: &TestApp, n: i64) -> Vec<String> {
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
    .fetch_all(&app.db_pool)
    .await
    .expect("query free ear tags")
}

#[tokio::test]
#[serial]
async fn animals_in_a_pen_are_listed_by_ear_tag_ascending() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let created_by: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("any user");

    // 非 A–G 開頭的欄位代碼不會被正規化，回應裡的 pen_location 與寫入值相同
    let pen = format!("S{:02}", (Uuid::new_v4().as_u128() % 90) as u8 + 10);
    let tags = free_ear_tags(&app, 3).await; // 由小到大：低、中、高

    // UUID 與耳號順序相反：base 相同、最低位元組 1/2/3 分別給「高、中、低」耳號
    let base = Uuid::new_v4().as_u128() << 8;
    let rows = [
        (base | 1, &tags[2]),
        (base | 2, &tags[1]),
        (base | 3, &tags[0]),
    ];
    for (id, ear_tag) in rows {
        sqlx::query(
            r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, created_by, pen_location)
               VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3, $4)"#,
        )
        .bind(Uuid::from_u128(id))
        .bind(ear_tag)
        .bind(created_by)
        .bind(&pen)
        .execute(&app.db_pool)
        .await
        .expect("insert animal");
    }

    let res = app.auth_get("/api/v1/animals/by-pen", &token).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.expect("by-pen json");

    let group = body
        .as_array()
        .expect("by-pen 應為陣列")
        .iter()
        .find(|g| g["pen_location"] == pen.as_str())
        .unwrap_or_else(|| panic!("回應中找不到欄位 {pen}"));
    let listed: Vec<&str> = group["animals"]
        .as_array()
        .expect("animals 應為陣列")
        .iter()
        .filter_map(|a| a["ear_tag"].as_str())
        .filter(|t| tags.iter().any(|mine| mine == t))
        .collect();

    let expected: Vec<&str> = tags.iter().map(String::as_str).collect();
    assert_eq!(listed, expected, "欄位內的動物應依耳號由小到大排列");
}
