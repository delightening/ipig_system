//! Acceptance tests: 動物紀錄的計畫前置需求（Follow-up #1）
//!
//! 規格來源：docs/training/動物紀錄計畫前置需求分類規範.md §一、§四
//!
//! 需計畫（iacuc_no 非空才可建立）：手術 / 疼痛評估 / 病理報告 / 試驗性觀察
//! 免計畫：體重 / 疫苗驅蟲 / 血液檢查 / 觀察(異常/觀察性質) / 基本資料
//! 條件放行：犧牲紀錄——非計畫內死亡（因病安樂死、年限死亡、出售後視同場內犧牲）
//! 本來就沒有計畫，只讓 view_all 類身分登記；確認犧牲時須填犧牲日期。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

const LOGIN_PASSWORD: &str = "PlanPrereq$Pw1";

/// 建立可登入、帶指定角色的使用者，回傳登入 token。
async fn login_token_with_role(app: &TestApp, label: &str, role: &str) -> String {
    let id = Uuid::new_v4();
    let email = format!("prereq-{label}-{}@test.local", &id.to_string()[..6]);
    let hash = AuthService::hash_password(LOGIN_PASSWORD).expect("hash password");
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password) \
         VALUES ($1, $2, $3, $4, true, true, false)",
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("prereq {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    app.login(&email, LOGIN_PASSWORD)
        .await
        .expect("login as seeded role user")
}

async fn animal_status(app: &TestApp, animal_id: Uuid) -> String {
    sqlx::query_scalar("SELECT status::text FROM animals WHERE id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("read animal status")
}

async fn sacrifice_row_count(app: &TestApp, animal_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM animal_sacrifices WHERE animal_id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("count sacrifice rows")
}

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

// ── 犧牲紀錄：無計畫動物條件放行 ────────────────────────────────────────────────
//
// 非計畫內死亡（因病安樂死、年限死亡、出售後視同場內犧牲）沒有 IACUC，卻仍需登記犧牲
// 並把動物轉為 euthanized。放行條件：view_all 類身分（`has_protocol_view_all`）；
// 確認犧牲時犧牲日期必填。有計畫的動物行為不變。

async fn post_sacrifice(
    app: &TestApp,
    token: &str,
    animal_id: Uuid,
    body: &serde_json::Value,
) -> reqwest::Response {
    app.auth_post(
        &format!("/api/v1/animals/{animal_id}/sacrifice"),
        body,
        token,
    )
    .await
}

/// 無計畫動物、未確認犧牲：可登記，動物狀態不變。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_allowed_unconfirmed() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-unconf").await;
    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": false });
    let res = post_sacrifice(&app, &token, animal_id, &body).await;

    assert!(
        res.status().is_success(),
        "無計畫動物應可登記未確認的犧牲紀錄（status: {}）",
        res.status()
    );
    assert_eq!(sacrifice_row_count(&app, animal_id).await, 1);
    assert_eq!(animal_status(&app, animal_id).await, "in_experiment");
}

/// 無計畫動物、確認犧牲且有日期：動物轉 euthanized、離欄，並寫入稽核。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_confirmed_euthanizes_animal() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-conf").await;
    let animal_id = seed_animal(&app, None, user_id).await;
    sqlx::query("UPDATE animals SET pen_location = 'T-01' WHERE id = $1")
        .bind(animal_id)
        .execute(&app.db_pool)
        .await
        .expect("set pen_location");

    let body = serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": true });
    let res = post_sacrifice(&app, &token, animal_id, &body).await;
    assert!(
        res.status().is_success(),
        "無計畫動物確認犧牲應成功（status: {}）",
        res.status()
    );
    let created: serde_json::Value = TestApp::json(res).await;
    let sacrifice_id: Uuid = created["id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .expect("response carries sacrifice id");

    assert_eq!(animal_status(&app, animal_id).await, "euthanized");
    let pen: Option<String> = sqlx::query_scalar("SELECT pen_location FROM animals WHERE id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("read pen_location");
    assert!(pen.is_none(), "轉為安樂死後應離欄");

    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = 'SACRIFICE_UPSERT' AND entity_id = $1",
    )
    .bind(sacrifice_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count audit rows");
    assert_eq!(audits, 1, "應寫入一筆 SACRIFICE_UPSERT 稽核");
}

/// 無計畫動物、確認犧牲但沒填日期：422，動物狀態與犧牲紀錄皆不變。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_confirmed_requires_date() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-nodate").await;
    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({ "confirmed_sacrifice": true });
    let res = post_sacrifice(&app, &token, animal_id, &body).await;

    assert_eq!(res.status().as_u16(), 422, "無計畫動物確認犧牲須填日期");
    assert_eq!(animal_status(&app, animal_id).await, "in_experiment");
    assert_eq!(sacrifice_row_count(&app, animal_id).await, 0);
}

/// 試驗工作人員與實習生（具 `animal.record.create` 且具 view_all）可補登無計畫動物的犧牲。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_allowed_for_internal_view_all_roles() {
    let app = TestApp::spawn().await;
    let user_id = create_test_user(&app, "sacr-roles").await;

    for role in ["EXPERIMENT_STAFF", "INTERN"] {
        let token = login_token_with_role(&app, role, role).await;
        let animal_id = seed_animal(&app, None, user_id).await;
        let body =
            serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": true });
        let res = post_sacrifice(&app, &token, animal_id, &body).await;

        assert!(
            res.status().is_success(),
            "{role} 應可補登無計畫動物的犧牲（status: {}）",
            res.status()
        );
        assert_eq!(animal_status(&app, animal_id).await, "euthanized");
    }
}

/// 沒有 `animal.record.create` 的身分（獸醫、IACUC 工作人員、PI、委託人）仍在權限關卡被擋 403，
/// 不會走到放行邏輯——鎖住「本次修正沒有放寬誰能寫犧牲紀錄」。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_rejected_for_roles_without_record_create() {
    let app = TestApp::spawn().await;
    let user_id = create_test_user(&app, "sacr-denied").await;

    for role in ["VET", "IACUC_STAFF", "PI", "CLIENT"] {
        let token = login_token_with_role(&app, role, role).await;
        let animal_id = seed_animal(&app, None, user_id).await;
        let body =
            serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": true });
        let res = post_sacrifice(&app, &token, animal_id, &body).await;

        assert_eq!(res.status().as_u16(), 403, "{role} 不應能寫犧牲紀錄");
        assert_eq!(animal_status(&app, animal_id).await, "in_experiment");
        assert_eq!(sacrifice_row_count(&app, animal_id).await, 0);
    }
}

/// 有計畫的動物：行為不變——確認犧牲不需要日期也能成功（日期必填只套用無計畫動物）。
#[tokio::test]
#[serial]
async fn sacrifice_with_protocol_unchanged_date_not_required() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-plan").await;
    let (_proto_id, iacuc) = seed_protocol(&app, user_id).await;
    let animal_id = seed_animal(&app, Some(&iacuc), user_id).await;

    let body = serde_json::json!({ "confirmed_sacrifice": true });
    let res = post_sacrifice(&app, &token, animal_id, &body).await;

    assert!(
        res.status().is_success(),
        "有計畫動物確認犧牲（無日期）行為不變，應成功（status: {}）",
        res.status()
    );
    assert_eq!(animal_status(&app, animal_id).await, "euthanized");
}

/// 無計畫動物已是終態（已安樂死）：再確認犧牲仍被狀態轉換規則擋下（400）。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_terminal_status_rejected() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-terminal").await;
    let animal_id = seed_animal(&app, None, user_id).await;
    sqlx::query("UPDATE animals SET status = 'euthanized'::animal_status WHERE id = $1")
        .bind(animal_id)
        .execute(&app.db_pool)
        .await
        .expect("set terminal status");

    let body = serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": true });
    let res = post_sacrifice(&app, &token, animal_id, &body).await;

    assert_eq!(res.status().as_u16(), 400, "終態動物不可再確認犧牲");
}

/// 同一請求連送兩次：第二次只更新，不重複建立（`animal_sacrifices` 以 animal_id 唯一）。
#[tokio::test]
#[serial]
async fn sacrifice_without_protocol_upsert_is_idempotent() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let user_id = create_test_user(&app, "sacr-idem").await;
    let animal_id = seed_animal(&app, None, user_id).await;

    let body = serde_json::json!({ "sacrifice_date": "2026-06-15", "confirmed_sacrifice": false });
    for _ in 0..2 {
        let res = post_sacrifice(&app, &token, animal_id, &body).await;
        assert!(res.status().is_success(), "status: {}", res.status());
    }
    assert_eq!(sacrifice_row_count(&app, animal_id).await, 1);
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
