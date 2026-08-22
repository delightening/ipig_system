//! R53-4 byproduct-samples handler 整合測試
//!
//! 重點：驗證路由 wiring + permission 守衛。
//! Service 層邏輯（DataDiff / 三層守衛 / Service-driven audit）已有 unit
//! tests (`services::animal::byproduct_sample::tests` 5 條) cover。
//!
//! 不做 full CRUD：完整流程需先建 animal → protocol → euthanasia_order
//! 三層 fixture（euthanasia_order 本身也有 deadline / PI 簽核流程），
//! 超出 service-driven audit 整合測試 scope。

mod common;

use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn list_by_euthanasia_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;
    let any_id = Uuid::new_v4();
    let res = app
        .client
        .get(app.url(&format!("/api/v1/euthanasia/{}/byproduct-samples", any_id)))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn list_by_euthanasia_returns_empty_for_unknown_id() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let unknown = Uuid::new_v4();

    let res = app
        .auth_get(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", unknown),
            &token,
        )
        .await;

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("Failed to parse");
    assert!(body.is_array());
    assert_eq!(body.as_array().expect("Expected array").len(), 0);
}

#[tokio::test]
#[serial]
async fn create_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;
    let any_id = Uuid::new_v4();
    // R53-4 review: body 不再帶 animal_id / source_protocol_id — service 從
    // path 的 euthanasia_id 推導（IDOR 守衛）。
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_org_name": "External Lab",
        "requester_contact_name": "Dr. External",
    });
    let res = app
        .client
        .post(app.url(&format!("/api/v1/euthanasia/{}/byproduct-samples", any_id)))
        .json(&body)
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn create_with_unknown_euthanasia_returns_404() {
    // admin 有 write 權限 → 過 permission gate；euthanasia 不存在 →
    // resolve_fks_from_euthanasia_tx 擋下回 404。
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let euth = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_org_name": "External Lab",
        "requester_contact_name": "Dr. External",
    });

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", euth),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
#[serial]
async fn create_with_neither_requester_returns_400() {
    // R53-14：requester_user_id / requester_org_name / requester_contact_name 全沒給
    // → validate_requester 擋下回 400。validate 在 FK 推導之前執行，FK 不存在的情境
    // 也不會觸發 — 直接 400。
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let euth = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        // 故意省略所有 requester 欄位
    });

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", euth),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn create_with_only_org_returns_400() {
    // R53-14：external requester 必須同時填機構 + 聯絡人，缺一不可。
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let euth = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_org_name": "國防醫學大學",
        // 故意省略 requester_contact_name
    });

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", euth),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn create_with_only_contact_returns_400() {
    // R53-14：同上 — 只填聯絡人沒填機構也擋下。
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let euth = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_contact_name": "王教授",
        // 故意省略 requester_org_name
    });

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", euth),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn create_with_inverted_work_time_returns_400() {
    // R53-14：work_ended_at < work_started_at → validate_work_time 擋下回 400。
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let euth = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_org_name": "External Lab",
        "requester_contact_name": "Dr. External",
        "work_started_at": "2026-05-17T11:00:00Z",
        "work_ended_at":   "2026-05-17T09:00:00Z",
    });

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", euth),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn get_unknown_id_returns_404() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let unknown = Uuid::new_v4();

    let res = app
        .auth_get(&format!("/api/v1/byproduct-samples/{}", unknown), &token)
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
#[serial]
async fn delete_unknown_id_returns_404() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let unknown = Uuid::new_v4();

    let res = app
        .auth_delete(&format!("/api/v1/byproduct-samples/{}", unknown), &token)
        .await;
    assert_eq!(res.status(), 404);
}

/// Helper: 用 admin 開一個 no-role 帳號（無 `animal.byproduct_sample.*` 權限）。
/// 用來驗證 RBAC gate 退化時 CI 會擋下。
async fn create_unprivileged_user_and_login(app: &common::TestApp) -> String {
    let admin_token = app.login_as_admin().await;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time error")
        .subsec_nanos();
    let email = format!("byproduct_noperm_{}@test.local", ts);
    let create_res = app
        .auth_post(
            "/api/v1/users",
            &serde_json::json!({
                "email": email,
                "display_name": "Byproduct NoPerm Test",
                "password": "TestPassword123!",
                "role_id": null,
                "is_internal": true,
            }),
            &admin_token,
        )
        .await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "Create unprivileged user returned {}",
        create_res.status()
    );
    app.login(&email, "TestPassword123!")
        .await
        .expect("Unprivileged user login failed")
}

#[tokio::test]
#[serial]
async fn list_by_euthanasia_without_permission_returns_403() {
    // 補 RBAC 防退化測試：無 `animal.byproduct_sample.view` 權限的登入用戶
    // 觸 GET 應被 require_permission! 擋下回 403，避免權限 gate 移除 / 寫錯
    // 後測試無法 catch。
    let app = common::TestApp::spawn().await;
    let token = create_unprivileged_user_and_login(&app).await;
    let any_id = Uuid::new_v4();
    let res = app
        .auth_get(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", any_id),
            &token,
        )
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
#[serial]
async fn create_without_permission_returns_403() {
    // 同上，但對應 write 權限（`animal.byproduct_sample.write`）。
    let app = common::TestApp::spawn().await;
    let token = create_unprivileged_user_and_login(&app).await;
    let any_id = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test sample",
        "requester_org_name": "External Lab",
        "requester_contact_name": "Dr. External",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/{}/byproduct-samples", any_id),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 403);
}

// ── #445：計劃內犧牲路徑建立（animal-path，不需安樂死單）────────────────

/// seed 一隻指定 status 的動物（含 iacuc_no + 對應 protocol），回 animal_id。
async fn seed_animal_with_protocol(app: &common::TestApp, status: &str) -> Uuid {
    let pool = &app.db_pool;
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("need a user");
    let pid = Uuid::new_v4();
    let iacuc = format!("IACUC-BP-{}", &pid.to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'byproduct animal-path', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-BP-{}", &pid.to_string()[..8]))
    .bind(&iacuc)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert protocol");
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, $3::animal_status, 'miniature', 'male', '2024-01-01', $4, $5)"#,
    )
    .bind(aid)
    .bind(format!("BP{}", &aid.to_string()[..6]))
    .bind(status)
    .bind(&iacuc)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert animal");
    aid
}

#[tokio::test]
#[serial]
async fn create_for_animal_without_auth_returns_401() {
    let app = common::TestApp::spawn().await;
    let any_id = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test",
        "requester_org_name": "Lab",
        "requester_contact_name": "Dr",
    });
    let res = app
        .client
        .post(app.url(&format!("/api/v1/animals/{}/byproduct-samples", any_id)))
        .json(&body)
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn create_for_animal_not_sacrificed_returns_400() {
    // 動物未犧牲（unassigned）→ service 閘門擋下回 400
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let aid = seed_animal_with_protocol(&app, "unassigned").await;
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "肝臟組織 5g",
        "requester_org_name": "國防醫學大學",
        "requester_contact_name": "王教授",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/byproduct-samples", aid),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
#[serial]
async fn create_for_animal_euthanized_succeeds_with_null_euthanasia_id() {
    // 已犧牲（euthanized）動物 → 200 + 建立成功 + euthanasia_id 為 null（計劃內犧牲路徑）
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let aid = seed_animal_with_protocol(&app, "euthanized").await;
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "心臟血 10mL",
        "requester_org_name": "國防醫學大學",
        "requester_contact_name": "王教授",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/byproduct-samples", aid),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 200);
    let row: serde_json::Value = res.json().await.expect("parse");
    assert_eq!(row["animal_id"].as_str(), Some(aid.to_string().as_str()));
    assert!(
        row["euthanasia_id"].is_null(),
        "計劃內犧牲路徑 euthanasia_id 應為 null，實得 {row:?}"
    );
}

#[tokio::test]
#[serial]
async fn create_for_animal_without_permission_returns_403() {
    let app = common::TestApp::spawn().await;
    let token = create_unprivileged_user_and_login(&app).await;
    let any_id = Uuid::new_v4();
    let body = serde_json::json!({
        "sampled_at": "2026-05-17T10:00:00Z",
        "sample_content": "test",
        "requester_org_name": "Lab",
        "requester_contact_name": "Dr",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/byproduct-samples", any_id),
            &body,
            &token,
        )
        .await;
    assert_eq!(res.status(), 403);
}
