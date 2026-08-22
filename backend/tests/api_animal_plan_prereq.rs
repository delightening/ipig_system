//! Acceptance tests: 動物紀錄的計畫前置需求（Follow-up #1）
//!
//! 規格來源：docs/training/動物紀錄計畫前置需求分類規範.md §一、§四
//!
//! 需計畫（iacuc_no 非空才可建立）：手術 / 犧牲採樣 / 疼痛評估 / 病理報告 / 試驗性觀察
//! 免計畫：體重 / 疫苗驅蟲 / 血液檢查 / 觀察(異常/觀察性質) / 基本資料

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

// ── 輔助函式 ────────────────────────────────────────────────────────────────────────────

async fn create_test_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', $3, true, false)",
    )
    .bind(id)
    .bind(format!("prereq-{label}-{}@test.local", &id.to_string()[..6]))
    .bind(format!("prereq {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert test user");
    id
}

/// 建立動物；iacuc_no = None 代表未指派計畫。
async fn seed_animal(app: &TestApp, iacuc_no: Option<&str>, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by) \
         VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-01-01', $3, $4)",
    )
    .bind(id)
    .bind(format!("PP{}", &id.to_string()[..6]))
    .bind(iacuc_no)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    id
}

/// 建立 APPROVED 計畫，回傳 (protocol_id, iacuc_no)。
async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let iacuc = format!("IACUC-PP-{}", &id.to_string()[..8]);
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at) \
         VALUES ($1, $2, $3, 'plan prereq test', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(format!("PN-PP-{}", &id.to_string()[..8]))
    .bind(&iacuc)
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    (id, iacuc)
}

/// 直接以 SQL 植入手術紀錄（繞過 service guard），供疼痛評估測試的父記錄。
async fn seed_surgery_direct(app: &TestApp, animal_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animal_surgeries \
         (id, animal_id, is_first_experiment, surgery_date, surgery_site, created_by, created_at, updated_at) \
         VALUES ($1, $2, false, '2024-01-01', 'test site', $3, NOW(), NOW())",
    )
    .bind(id)
    .bind(animal_id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert surgery directly");
    id
}

// ── 拒絕測試：未指派計畫 → 422 ──────────────────────────────────────────────────────

/// 手術紀錄需計畫；動物 iacuc_no=NULL → 422
#[tokio::test]
#[serial]
async fn surgery_blocked_when_animal_has_no_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "surg-block").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({
        "surgery_date": "2026-06-15",
        "surgery_site": "腔部",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/surgeries"),
            &body,
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        422,
        "手術應被拒絕（動物尚未指派計畫）"
    );
}

/// 犧牲採樣需計畫；動物 iacuc_no=NULL → 422
#[tokio::test]
#[serial]
async fn sacrifice_blocked_when_animal_has_no_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-block").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({
        "sacrifice_date": "2026-06-15",
        "confirmed_sacrifice": false,
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/sacrifice"),
            &body,
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        422,
        "犧牲採樣應被拒絕（動物尚未指派計畫）"
    );
}

/// 病理組織報告需計畫；動物 iacuc_no=NULL → 422
#[tokio::test]
#[serial]
async fn pathology_blocked_when_animal_has_no_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "path-block").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/pathology"),
            &serde_json::json!({}),
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        422,
        "病理報告應被拒絕（動物尚未指派計畫）"
    );
}

/// 試驗性觀察需計畫；動物 iacuc_no=NULL → 422
#[tokio::test]
#[serial]
async fn experiment_observation_blocked_when_animal_has_no_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "obs-exp-block").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({
        "event_date": "2026-06-15",
        "record_type": "experiment",
        "content": "試驗性觀察",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/observations"),
            &body,
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        422,
        "試驗性觀察應被拒絕（動物尚未指派計畫）"
    );
}

/// 疼痛評估（care record PainAssessment）需計畫；動物 iacuc_no=NULL → 422
///
/// 父記錄（手術）以 SQL 直接植入，繞過 service 層守衛，以獨立測試 care_record 的守衛。
#[tokio::test]
#[serial]
async fn pain_assessment_blocked_when_animal_has_no_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "pain-block").await;

    let animal_id = seed_animal(&app, None, user_id).await;
    let surgery_id = seed_surgery_direct(&app, animal_id, user_id).await;

    let body = serde_json::json!({
        "record_type": "surgery",
        "record_id": surgery_id,
        "record_mode": "pain_assessment",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/care-records"),
            &body,
            &token,
        )
        .await;

    assert_eq!(
        res.status().as_u16(),
        422,
        "疼痛評估應被拒絕（動物尚未指派計畫）"
    );
}

// ── 放行測試：免計畫紀錄不受影響 ────────────────────────────────────────────────

/// 異常觀察免計畫；動物 iacuc_no=NULL → 成功
#[tokio::test]
#[serial]
async fn abnormal_observation_allowed_without_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "obs-abn-allow").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({
        "event_date": "2026-06-15",
        "record_type": "abnormal",
        "content": "異常觀察紀錄",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/observations"),
            &body,
            &token,
        )
        .await;

    assert!(
        res.status().is_success(),
        "異常觀察不需計畫，應成功建立（status: {}）",
        res.status()
    );
}

/// 一般觀察免計畫；動物 iacuc_no=NULL → 成功
#[tokio::test]
#[serial]
async fn general_observation_allowed_without_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "obs-gen-allow").await;

    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({
        "event_date": "2026-06-15",
        "record_type": "observation",
        "content": "一般觀察紀錄",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/observations"),
            &body,
            &token,
        )
        .await;

    assert!(
        res.status().is_success(),
        "一般觀察不需計畫，應成功建立（status: {}）",
        res.status()
    );
}

// ── 放行測試：已指派計畫的動物建立需計畫紀錄 → 成功 ─────────────────────────

/// 已指派計畫的動物可建立手術紀錄
#[tokio::test]
#[serial]
async fn surgery_allowed_for_animal_with_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "surg-allow").await;

    let (_proto_id, iacuc) = seed_protocol(&app, user_id).await;
    let animal_id = seed_animal(&app, Some(&iacuc), user_id).await;

    let body = serde_json::json!({
        "surgery_date": "2026-06-15",
        "surgery_site": "腔部",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/surgeries"),
            &body,
            &token,
        )
        .await;

    assert!(
        res.status().is_success(),
        "已指派計畫，手術應成功建立（status: {}）",
        res.status()
    );
}

/// 已指派計畫的動物可建立試驗性觀察
#[tokio::test]
#[serial]
async fn experiment_observation_allowed_for_animal_with_protocol() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "obs-exp-allow").await;

    let (_proto_id, iacuc) = seed_protocol(&app, user_id).await;
    let animal_id = seed_animal(&app, Some(&iacuc), user_id).await;

    let body = serde_json::json!({
        "event_date": "2026-06-15",
        "record_type": "experiment",
        "content": "試驗觀察",
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/observations"),
            &body,
            &token,
        )
        .await;

    assert!(
        res.status().is_success(),
        "已指派計畫，試驗觀察應成功建立（status: {}）",
        res.status()
    );
}
