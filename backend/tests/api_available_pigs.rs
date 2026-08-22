//! Integration tests for R47 /api/v1/animals/available (可用豬隻快速查詢).

mod common;

use common::TestApp;
use erp_backend::services::AuthService;
use serial_test::serial;
use uuid::Uuid;

/// 開一個 PI 角色帳號（animal.animal.view_project + animal.record.view，但**非 view_all**）。
/// PI 為 AUP 角色，須以 user_roles(code='PI') 指派才會帶權限進 JWT。
async fn create_pi_user_and_login(app: &TestApp) -> String {
    let password = "TestPassword123!";
    let id = Uuid::new_v4();
    let email = format!("avp_pi_{}@test.local", &Uuid::new_v4().to_string()[..8]);
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, 'AVP PI Test', true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(&app.db_pool)
    .await
    .expect("insert PI user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    app.login(&email, password).await.expect("PI login failed")
}

/// code-review #386（IDOR 回歸）：只有 view_project（無 view_all）的 PI 不得透過
/// /animals/available 列舉未指派計畫（iacuc_no NULL）的全院庫存豬；admin（view_all）則看得到。
#[tokio::test]
#[serial]
async fn available_pigs_view_project_only_excludes_unassigned_inventory() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("需要一個既有 user");

    // 唯一標記放 remark（回應含 remark 欄、且不會被格式化），避免 ear_tag 格式化比對問題
    let unassigned_marker = format!("AVP-UNASSIGN-{}", &Uuid::new_v4().to_string()[..8]);
    let assigned_marker = format!("AVP-ASSIGN-{}", &Uuid::new_v4().to_string()[..8]);
    // 已指派（completed + iacuc_no 非空）vs 未指派（unassigned + iacuc_no NULL），各一隻、皆有新鮮體重。
    let seed_pig = |status: &'static str, iacuc: Option<String>, marker: String| {
        let pool = pool.clone();
        async move {
            let aid = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date, iacuc_no, remark, created_by)
                   VALUES ($1, $2, $3::animal_status, 'miniature', 'male', '2024-01-01', '2024-06-01', $4, $5, $6)"#,
            )
            .bind(aid)
            .bind(format!("AV{}", &aid.to_string()[..6]))
            .bind(status)
            .bind(iacuc)
            .bind(&marker)
            .bind(owner)
            .execute(&pool)
            .await
            .expect("insert animal");
            sqlx::query(
                "INSERT INTO animal_weights (animal_id, measure_date, weight) VALUES ($1, CURRENT_DATE, 50.0)",
            )
            .bind(aid)
            .execute(&pool)
            .await
            .expect("insert weight");
        }
    };
    seed_pig("unassigned", None, unassigned_marker.clone()).await;
    seed_pig(
        "completed",
        Some(format!("IACUC-AVP-{}", &Uuid::new_v4().to_string()[..6])),
        assigned_marker.clone(),
    )
    .await;

    let contains = |body: &serde_json::Value, marker: &str| -> bool {
        body["animals"]
            .as_array()
            .map(|arr| arr.iter().any(|a| a["remark"].as_str() == Some(marker)))
            .unwrap_or(false)
    };

    // admin（view_all）→ 兩隻都看得到
    let admin = app.login_as_admin().await;
    let admin_res = app
        .auth_get("/api/v1/animals/available?per_page=200", &admin)
        .await;
    assert_eq!(admin_res.status(), 200);
    let admin_body: serde_json::Value = admin_res.json().await.expect("parse admin");
    assert!(
        contains(&admin_body, &unassigned_marker),
        "admin 應看到未指派豬"
    );
    assert!(
        contains(&admin_body, &assigned_marker),
        "admin 應看到已指派豬"
    );

    // PI（view_project，非 view_all）→ 只看得到已指派豬，看不到未指派豬
    let pi = create_pi_user_and_login(&app).await;
    let pi_res = app
        .auth_get("/api/v1/animals/available?per_page=200", &pi)
        .await;
    assert_eq!(pi_res.status(), 200);
    let pi_body: serde_json::Value = pi_res.json().await.expect("parse pi");
    assert!(
        !contains(&pi_body, &unassigned_marker),
        "view_project-only 的 PI 不應看到未指派庫存豬（#386）"
    );
    assert!(
        contains(&pi_body, &assigned_marker),
        "view_project 的 PI 仍應看到已指派計畫豬（不可整列消失）"
    );
}

#[tokio::test]
#[serial]
async fn available_pigs_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/animals/available"))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn available_pigs_returns_expected_shape() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/animals/available", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body["animals"].is_array(), "animals 應為 array");
    // R47 備註欄：每筆應含 remark key（Option<String> → 可為 null，但 key 必存在）
    if let Some(first) = body["animals"].as_array().and_then(|a| a.first()) {
        assert!(
            first.get("remark").is_some(),
            "animals[].remark key 應存在（備註欄）"
        );
    }
    assert!(body["summary"].is_object(), "summary 應為 object");
    assert!(
        body["summary"]["total"].is_number(),
        "summary.total 應為 number"
    );
    assert!(
        body["summary"]["male"].is_number(),
        "summary.male 應為 number"
    );
    assert!(
        body["summary"]["female"].is_number(),
        "summary.female 應為 number"
    );
    assert!(
        body["summary"]["by_breed"].is_object(),
        "summary.by_breed 應為 object"
    );
    assert!(
        body["summary"]["excluded_weight_expired"].is_number(),
        "summary.excluded_weight_expired 應為 number"
    );
    assert!(body["page"].is_number(), "page 應為 number");
    assert!(body["per_page"].is_number(), "per_page 應為 number");
}

#[tokio::test]
#[serial]
async fn available_pigs_sex_filter_narrows_results() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 不帶 filter
    let all_res = app.auth_get("/api/v1/animals/available", &token).await;
    assert_eq!(all_res.status(), 200);
    let all_body: serde_json::Value = all_res.json().await.expect("parse all body");
    let all_total = all_body["summary"]["total"].as_i64().unwrap_or(0);
    let all_male = all_body["summary"]["male"].as_i64().unwrap_or(0);
    let all_female = all_body["summary"]["female"].as_i64().unwrap_or(0);
    assert_eq!(all_total, all_male + all_female, "total = male + female");

    // sex=male 過濾
    let male_res = app
        .auth_get("/api/v1/animals/available?sex=male", &token)
        .await;
    assert_eq!(male_res.status(), 200);
    let male_body: serde_json::Value = male_res.json().await.expect("parse male body");
    let male_total = male_body["summary"]["total"].as_i64().unwrap_or(0);
    let male_female_count = male_body["summary"]["female"].as_i64().unwrap_or(0);
    assert_eq!(male_total, all_male, "sex=male 的 total 應等於整體 male 數");
    assert_eq!(male_female_count, 0, "sex=male 結果中 female 應為 0");

    // 確認回傳列表所有 gender 都是 male
    if let Some(arr) = male_body["animals"].as_array() {
        for a in arr {
            assert_eq!(a["gender"], "male", "結果列中應只含 male");
        }
    }
}

#[tokio::test]
#[serial]
async fn available_pigs_include_breeding_toggle_changes_total() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 預設 include_breeding=false
    let excl_res = app.auth_get("/api/v1/animals/available", &token).await;
    assert_eq!(excl_res.status(), 200);
    let excl_body: serde_json::Value = excl_res.json().await.expect("parse excl body");
    let excl_total = excl_body["summary"]["total"].as_i64().unwrap_or(0);

    // include_breeding=true（飼養計畫 000 中的豬一起算）
    let inc_res = app
        .auth_get("/api/v1/animals/available?include_breeding=true", &token)
        .await;
    assert_eq!(inc_res.status(), 200);
    let inc_body: serde_json::Value = inc_res.json().await.expect("parse inc body");
    let inc_total = inc_body["summary"]["total"].as_i64().unwrap_or(0);

    // include_breeding=true 的 total 應 ≥ false 的 total
    assert!(
        inc_total >= excl_total,
        "include_breeding=true 的 total ({inc_total}) 應 ≥ false 的 total ({excl_total})"
    );
}
