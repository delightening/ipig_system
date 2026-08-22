//! R53-9 豬隻病歷週報 handler 整合測試
//!
//! 驗證路由 wiring + permission + filter shape。Service 層 unit test 已 cover
//! filter 序列化往返。
//!
//! 不做 full-data 測試：需 fixture animals + observations 才能驗 timeline
//! 內容，本 PR scope 為 API contract 落地（MVP 只接 animal_observations，
//! 其他來源表待 R53-7 範本後追加）。

mod common;

use common::TestApp;
use erp_backend::services::{AnimalMedicalReportService, AuthService, WeeklyMedicalReportFilter};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn weekly_report_without_auth_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/api/v1/reports/animal-medical/weekly"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn weekly_report_with_empty_filter_returns_200_array() {
    // admin 有 animal.record.view → 200，回 array（可能空也可能有現有資料）
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let res = app
        .auth_post(
            "/api/v1/reports/animal-medical/weekly",
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("Failed to parse");
    assert!(body.is_array(), "回應應為 array，實際：{:?}", body);
}

#[tokio::test]
#[serial]
async fn weekly_report_filter_shape_accepted() {
    // 完整 filter shape 應被接受（empty arrays / None 都 OK）
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let body = serde_json::json!({
        "animal_ear_tags": ["nonexistent-001"],
        "protocol_ids": [],
        "start_date": "2026-05-10",
        "end_date": "2026-05-17",
    });
    let res = app
        .auth_post("/api/v1/reports/animal-medical/weekly", &body, &token)
        .await;
    assert_eq!(res.status(), 200);
    let arr: serde_json::Value = res.json().await.expect("Failed to parse");
    // 不存在的耳號 → 結果為空 array
    assert_eq!(arr.as_array().expect("array").len(), 0);
}

/// code-review #447（IDOR 回歸）：週報 service 端資料邊界。
///
/// Bug：`weekly_report` 不接受使用者範圍，無 view_all 的 PI / 委託人（只有
/// `animal.record.view` + view_project）送空 filter 即讀到全院跨計畫所有豬隻病歷。
/// 修復：handler 依 actor 計算可存取計畫集合，強制 AND 進查詢（`accessible_protocol_ids`）。
/// 本測試鎖定 service 層 boundary 契約：None=不限、Some(&[])=無權看空、Some(ids)=僅限該集合。
#[tokio::test]
#[serial]
async fn weekly_report_boundary_scopes_to_accessible_protocols() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    // 取一個既有 user 當 pi / created_by（admin seed）
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("需要一個既有 user");

    let pid = Uuid::new_v4();
    let iacuc = format!("IACUC-MR-{}", &pid.to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'medical report boundary', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-MR-{}", &pid.to_string()[..8]))
    .bind(&iacuc)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert protocol");

    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3, $4)"#,
    )
    .bind(aid)
    .bind(format!("MR{}", &aid.to_string()[..6]))
    .bind(&iacuc)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert animal");

    sqlx::query(
        r#"INSERT INTO animal_observations (animal_id, event_date, record_type, content, created_by)
           VALUES ($1, '2026-05-15', 'observation', 'boundary test obs', $2)"#,
    )
    .bind(aid)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert observation");

    let filter = WeeklyMedicalReportFilter {
        animal_ear_tags: None,
        protocol_ids: None,
        start_date: None,
        end_date: None,
    };

    // None = view_all → 看得到該筆
    let unrestricted = AnimalMedicalReportService::weekly_report(pool, &filter, None)
        .await
        .expect("unrestricted");
    assert!(
        unrestricted.iter().any(|e| e.animal_id == aid),
        "view_all（None boundary）應看到該筆病歷"
    );

    // Some(&[]) = 無可存取計畫 → 一律回空（核心 IDOR 防護）
    let empty_boundary = AnimalMedicalReportService::weekly_report(pool, &filter, Some(&[]))
        .await
        .expect("empty boundary");
    assert!(
        empty_boundary.is_empty(),
        "空 boundary 應回空，實得 {} 筆",
        empty_boundary.len()
    );

    // Some(&[unrelated]) = 不相關計畫 → 看不到該筆
    let unrelated_ids = vec![Uuid::new_v4()];
    let unrelated = AnimalMedicalReportService::weekly_report(pool, &filter, Some(&unrelated_ids))
        .await
        .expect("unrelated boundary");
    assert!(
        !unrelated.iter().any(|e| e.animal_id == aid),
        "不相關計畫 boundary 不應看到該筆"
    );

    // Some(&[pid]) = 相關計畫 → 看得到
    let related_ids = vec![pid];
    let scoped = AnimalMedicalReportService::weekly_report(pool, &filter, Some(&related_ids))
        .await
        .expect("related boundary");
    assert!(
        scoped.iter().any(|e| e.animal_id == aid),
        "相關計畫 boundary 應看到該筆"
    );
}

/// 測試用：seed 一筆 protocol + animal + observation，回傳 animal_id（耳號嵌入唯一前綴）。
async fn seed_animal_with_observation(pool: &sqlx::PgPool, tag_prefix: &str) -> Uuid {
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("需要一個既有 user");
    let pid = Uuid::new_v4();
    let iacuc = format!("IACUC-{}-{}", tag_prefix, &pid.to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'medical report http boundary', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-{}-{}", tag_prefix, &pid.to_string()[..8]))
    .bind(&iacuc)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert protocol");

    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3, $4)"#,
    )
    .bind(aid)
    .bind(format!("{}{}", tag_prefix, &aid.to_string()[..5]))
    .bind(&iacuc)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert animal");

    sqlx::query(
        r#"INSERT INTO animal_observations (animal_id, event_date, record_type, content, created_by)
           VALUES ($1, '2026-05-15', 'observation', 'http boundary obs', $2)"#,
    )
    .bind(aid)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert observation");
    aid
}

/// 測試用：開一個 PI 角色帳號（持 `animal.record.view` + view_project，但**非 view_all**），回 token。
/// PI 為 AUP 角色，須以 `user_roles`（role code = 'PI'）指派才會帶權限進 JWT
/// （非 create-user 的 role_id），對齊既有 seed_login_user 模式。
async fn create_pi_user_and_login(app: &TestApp) -> String {
    let password = "TestPassword123!";
    let id = Uuid::new_v4();
    let email = format!("mr_pi_{}@test.local", &Uuid::new_v4().to_string()[..8]);
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, 'MR PI Test', true, true, false)"#,
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
    app.login(&email, password)
        .await
        .expect("PI user login failed")
}

/// code-review #447（IDOR 回歸，HTTP 端對端）：驗證 handler wiring（`report_protocol_boundary`
/// + 三個呼叫點）不會 regress。非 view_all 的 PI 送空 filter `{}` **不得**看到其無關計畫之病歷；
/// admin（view_all）則看得到（positive control）。對齊 coding guideline「IDOR 守衛屬 handler 層責任」。
#[tokio::test]
#[serial]
async fn weekly_report_endpoint_excludes_cross_protocol_for_non_view_all() {
    let app = TestApp::spawn().await;
    let aid = seed_animal_with_observation(&app.db_pool, "MRH").await;
    let aid_json = serde_json::json!(aid.to_string());

    // 非 view_all 的 PI 送空 filter → 不得看到該筆（無關計畫）
    let pi_token = create_pi_user_and_login(&app).await;
    let res = app
        .auth_post(
            "/api/v1/reports/animal-medical/weekly",
            &serde_json::json!({}),
            &pi_token,
        )
        .await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse");
    let rows = body.as_array().expect("array");
    assert!(
        !rows.iter().any(|e| e["animal_id"] == aid_json),
        "非 view_all 的 PI 送空 filter 不應看到跨計畫病歷"
    );

    // admin（view_all）送空 filter → 看得到（positive control，證明資料確實存在）
    let admin = app.login_as_admin().await;
    let res2 = app
        .auth_post(
            "/api/v1/reports/animal-medical/weekly",
            &serde_json::json!({}),
            &admin,
        )
        .await;
    assert_eq!(res2.status(), 200);
    let body2: serde_json::Value = res2.json().await.expect("parse");
    assert!(
        body2
            .as_array()
            .expect("array")
            .iter()
            .any(|e| e["animal_id"] == aid_json),
        "admin（view_all）應看到該筆病歷"
    );
}
