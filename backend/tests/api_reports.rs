//! Integration tests for report endpoints and notifications.

mod common;

use serde_json::Value;
use serial_test::serial;
use uuid::Uuid;

// ── Reports ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn stock_on_hand_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/stock-on-hand", &token).await;

    // 200 even if empty data set
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn stock_ledger_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/stock-ledger", &token).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn purchase_lines_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/reports/purchase-lines", &token).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
#[serial]
async fn protocol_consumption_report_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_get("/api/v1/reports/protocol-consumption", &token)
        .await;
    assert_eq!(res.status(), 200);
}

/// 🔴 這支報表的全部風險都在那段 SQL 裡，200 OK 什麼都證明不了。
///
/// 直接灌 `stock_ledger` 列（append-only 表允許 INSERT，只擋 UPDATE / DELETE），
/// 驗三件會算錯的事：
///   1. 沖銷（`SO/in` 鏡射列）要被減掉，不是照算
///   2. 整筆沖光的品項（淨額 0）不該佔一列
///   3. 非 `SO` 的異動（這裡用 `ADJ`）不算消耗
#[tokio::test]
#[serial]
async fn protocol_consumption_nets_reversals_and_excludes_non_so() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let db = &app.db_pool;

    let admin_id = admin_user_id(&app).await;
    let tag = Uuid::new_v4().simple().to_string();
    let short = &tag[..8];

    let warehouse_id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(warehouse_id)
        .bind(format!("W{short}"))
        .bind("消耗報表測試倉")
        .execute(db)
        .await
        .expect("insert warehouse");

    // 甲：領 10、再領 5、沖掉第一筆 10 → 淨 5，應出現
    // 乙：領 7、整筆沖掉 → 淨 0，不該出現
    // 丙：只有 ADJ 出庫 → 不是領用，不該出現
    let product_a = insert_product(db, &format!("SKU-A-{short}"), "甲耗材", "雙").await;
    let product_b = insert_product(db, &format!("SKU-B-{short}"), "乙耗材", "包").await;
    let product_c = insert_product(db, &format!("SKU-C-{short}"), "丙耗材", "支").await;

    let protocol_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, pi_user_id, created_by)
         VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(protocol_id)
    .bind(format!("P-{short}"))
    .bind(format!("IACUC-{short}"))
    .bind("消耗報表測試計畫")
    .bind(admin_id)
    .execute(db)
    .await
    .expect("insert protocol");

    // doc_no 帶亂數後綴，避免同一顆丟棄庫連跑多支測試時撞唯一鍵
    let doc_a = insert_so_doc(db, admin_id, protocol_id, &format!("SO-A-{short}")).await;
    let doc_b = insert_so_doc(db, admin_id, protocol_id, &format!("SO-B-{short}")).await;
    let doc_rev = insert_so_doc(db, admin_id, protocol_id, &format!("SO-R-{short}")).await;
    let doc_c = insert_doc(db, admin_id, protocol_id, &format!("ADJ-C-{short}"), "ADJ").await;

    for (doc_id, doc_no, doc_type, product_id, direction, qty) in [
        (doc_a, format!("SO-A-{short}"), "SO", product_a, "out", "10"),
        (doc_b, format!("SO-B-{short}"), "SO", product_a, "out", "5"),
        // 沖銷：沿用原 doc_type，只反轉 direction（同 reverse_document_stock 的行為）
        (
            doc_rev,
            format!("SO-R-{short}"),
            "SO",
            product_a,
            "in",
            "10",
        ),
        (doc_b, format!("SO-B-{short}"), "SO", product_b, "out", "7"),
        (doc_rev, format!("SO-R-{short}"), "SO", product_b, "in", "7"),
        (
            doc_c,
            format!("ADJ-C-{short}"),
            "ADJ",
            product_c,
            "adjust_out",
            "3",
        ),
    ] {
        sqlx::query(
            "INSERT INTO stock_ledger
                (id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
                 direction, qty_base, unit_cost)
             VALUES ($1, $2, $3, now(), $4::doc_type, $5, $6, $7::stock_direction, $8::numeric, 2)",
        )
        .bind(Uuid::new_v4())
        .bind(warehouse_id)
        .bind(product_id)
        .bind(doc_type)
        .bind(doc_id)
        .bind(doc_no)
        .bind(direction)
        .bind(qty)
        .execute(db)
        .await
        .expect("insert stock_ledger");
    }

    let rows: Vec<Value> = app
        .auth_get(
            &format!("/api/v1/reports/protocol-consumption?protocol_id={protocol_id}"),
            &token,
        )
        .await
        .json()
        .await
        .expect("parse report");

    assert_eq!(
        rows.len(),
        1,
        "只有甲耗材的淨消耗不為 0，乙（整筆沖掉）與丙（ADJ 非領用）都不該出現，實得 {rows:#?}"
    );

    let row = &rows[0];
    assert_eq!(
        row["product_id"].as_str().expect("product_id 應為字串"),
        product_a.to_string()
    );
    assert_eq!(
        json_num(&row["qty_base"]),
        5.0,
        "10 + 5 − 10（沖銷）= 5，沖銷沒被減掉的話會是 25"
    );
    assert_eq!(
        row["doc_count"].as_i64().expect("doc_count 應為整數"),
        3,
        "甲耗材由 3 張單構成（兩張領用 + 一張沖銷）"
    );
    assert_eq!(row["base_uom"].as_str().expect("base_uom 應為字串"), "雙");
    assert_eq!(
        row["iacuc_no"].as_str().expect("iacuc_no 應為字串"),
        format!("IACUC-{short}"),
        "iacuc_no 只作顯示，分組鍵是 protocol_id"
    );
    assert_eq!(json_num(&row["total_cost"]), 10.0, "5 × unit_cost 2");
}

async fn admin_user_id(app: &common::TestApp) -> Uuid {
    let email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin user id")
}

async fn insert_product(db: &sqlx::PgPool, sku: &str, name: &str, base_uom: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(sku)
        .bind(name)
        .bind(base_uom)
        .execute(db)
        .await
        .expect("insert product");
    id
}

async fn insert_so_doc(db: &sqlx::PgPool, user: Uuid, protocol: Uuid, doc_no: &str) -> Uuid {
    insert_doc(db, user, protocol, doc_no, "SO").await
}

async fn insert_doc(
    db: &sqlx::PgPool,
    user: Uuid,
    protocol: Uuid,
    doc_no: &str,
    doc_type: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, doc_type, doc_no, status, doc_date, created_by, protocol_id)
         VALUES ($1, $2::doc_type, $3, 'approved'::doc_status, CURRENT_DATE, $4, $5)",
    )
    .bind(id)
    .bind(doc_type)
    .bind(doc_no)
    .bind(user)
    .bind(protocol)
    .execute(db)
    .await
    .expect("insert document");
    id
}

/// `Decimal` 依序列化設定可能落在 JSON 字串或數字，兩種都收。
fn json_num(v: &Value) -> f64 {
    match v {
        Value::String(s) => s.parse().expect("numeric string"),
        Value::Number(n) => n.as_f64().expect("f64"),
        other => panic!("not numeric: {other}"),
    }
}

#[tokio::test]
#[serial]
async fn reports_without_auth_return_401() {
    let app = common::TestApp::spawn().await;

    let endpoints = [
        "/api/v1/reports/stock-on-hand",
        "/api/v1/reports/stock-ledger",
        "/api/v1/reports/purchase-lines",
        "/api/v1/reports/protocol-consumption",
    ];

    for endpoint in endpoints {
        let res = app
            .client
            .get(app.url(endpoint))
            .send()
            .await
            .expect("HTTP request failed");
        assert_eq!(
            res.status(),
            401,
            "Endpoint {} should require auth",
            endpoint
        );
    }
}

// ── Notifications ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn list_notifications_returns_200() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/notifications", &token).await;
    assert_eq!(res.status(), 200);
}
