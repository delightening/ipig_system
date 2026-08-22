//! 計畫書草稿可見性與編輯/送出授權收緊 回歸測試。
//!
//! 對齊原始 spec §4.1 與使用者裁定：
//! - 草稿只對其 PI / SD / 成員，以及監督角色（執秘 IACUC_STAFF、主席）可見；
//!   持 `aup.protocol.view_all` 但非監督角色者（如 QAU / EXPERIMENT_STAFF）看不到他人草稿。
//! - 草稿編輯 / 送出收緊為 admin / PI / SD；執秘（IACUC_STAFF）與 CLIENT 唯讀（PUT/submit → 403）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

/// 建立可登入使用者 + 指定角色，回傳 (id, email)。
async fn seed_login_user(
    app: &TestApp,
    label: &str,
    role_code: &str,
    password: &str,
) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "draftvis-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password(password).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("draftvis {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    (id, email)
}

/// 建立一位不可登入的占位使用者（供他人計畫的 pi_user_id）。
async fn seed_placeholder_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("draftvis-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("draftvis {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert placeholder user");
    id
}

/// 建立指定狀態的計畫，回傳 id。`sd` 為 study_director_user_id（None 則 NULL）。
async fn seed_protocol(app: &TestApp, status: &str, pi: Uuid, sd: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, study_director_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'draft-visibility test', ($4::text)::protocol_status, $5, $6, $5, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("DV-{unique}"))
    .bind(format!("DVIACUC-{unique}"))
    .bind(status)
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

/// 將 user 以指定 role 加為計畫成員。
async fn add_member(app: &TestApp, protocol_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
           VALUES ($1, $2, ($3::text)::protocol_role, NOW(), $1)
           ON CONFLICT (user_id, protocol_id) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .bind(role)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols membership");
}

/// 從 `/api/v1/protocols` 回應抽出 id 清單。
fn id_list(body: &serde_json::Value) -> Vec<String> {
    body.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// 取某計畫項目的 `can_edit` 旗標。
fn can_edit_of(body: &serde_json::Value, id: Uuid) -> Option<bool> {
    body.as_array()?.iter().find_map(|p| {
        if p.get("id").and_then(|v| v.as_str()) == Some(&id.to_string()) {
            p.get("can_edit").and_then(|v| v.as_bool())
        } else {
            None
        }
    })
}

/// QAU（持 view_all 但非監督角色）：看得到他人非草稿，但**看不到他人草稿**；自己為 SD 的草稿可見。
#[tokio::test]
#[serial]
async fn qau_view_all_excludes_others_drafts() {
    let app = TestApp::spawn().await;
    let pw = "DraftVis$Pw1";
    let (qau_id, email) = seed_login_user(&app, "qau", "QAU", pw).await;
    let other_pi = seed_placeholder_user(&app, "otherpi").await;

    let others_draft = seed_protocol(&app, "DRAFT", other_pi, None).await;
    let others_approved = seed_protocol(&app, "APPROVED", other_pi, None).await;
    let my_draft_as_sd = seed_protocol(&app, "DRAFT", other_pi, Some(qau_id)).await;

    let token = app.login(&email, pw).await.expect("qau login");
    let res = app.auth_get("/api/v1/protocols", &token).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse protocols");
    let ids = id_list(&body);

    assert!(
        ids.contains(&others_approved.to_string()),
        "view_all 應看得到他人非草稿"
    );
    assert!(
        !ids.contains(&others_draft.to_string()),
        "非監督的 view_all 不應看到他人草稿"
    );
    assert!(
        ids.contains(&my_draft_as_sd.to_string()),
        "應看得到自己為 SD 的草稿"
    );
    // can_edit 與可見性是各自獨立投影；SD 對自己負責的草稿應 can_edit=true（防 SQL 回歸）。
    assert_eq!(
        can_edit_of(&body, my_draft_as_sd),
        Some(true),
        "SD 對自己負責的草稿 can_edit 應為 true"
    );
}

/// 執秘（IACUC_STAFF，監督角色）：看得到他人草稿（唯讀）；can_edit=false；PUT/submit → 403。
#[tokio::test]
#[serial]
async fn iacuc_staff_sees_drafts_but_readonly() {
    let app = TestApp::spawn().await;
    let pw = "DraftVis$Pw2";
    let (_staff_id, email) = seed_login_user(&app, "staff", "IACUC_STAFF", pw).await;
    let other_pi = seed_placeholder_user(&app, "otherpi2").await;
    let others_draft = seed_protocol(&app, "DRAFT", other_pi, None).await;

    let token = app.login(&email, pw).await.expect("staff login");

    // 可見且 can_edit=false
    let res = app.auth_get("/api/v1/protocols", &token).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse protocols");
    assert!(
        id_list(&body).contains(&others_draft.to_string()),
        "執秘應看得到他人草稿（監督）"
    );
    assert_eq!(
        can_edit_of(&body, others_draft),
        Some(false),
        "執秘對草稿 can_edit 應為 false"
    );

    // 編輯內容（含 title → touches_content）→ 403（執秘內容唯讀）
    let put = app
        .auth_put(
            &format!("/api/v1/protocols/{others_draft}"),
            &serde_json::json!({"title": "unauthorized edit"}),
            &token,
        )
        .await;
    assert_eq!(put.status(), 403, "執秘編輯內容應 403");

    // 送出 → 403
    let submit = app
        .auth_post(
            &format!("/api/v1/protocols/{others_draft}/submit"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(submit.status(), 403, "執秘送出草稿應 403");
}

/// CLIENT 成員：只看到被指定的計畫；can_edit=false；PUT → 403。
#[tokio::test]
#[serial]
async fn client_member_sees_only_assigned_and_readonly() {
    let app = TestApp::spawn().await;
    let pw = "DraftVis$Pw3";
    let (client_id, email) = seed_login_user(&app, "client", "CLIENT", pw).await;
    let other_pi = seed_placeholder_user(&app, "otherpi3").await;

    let assigned = seed_protocol(&app, "DRAFT", other_pi, None).await;
    add_member(&app, assigned, client_id, "CLIENT").await;
    let not_assigned = seed_protocol(&app, "DRAFT", other_pi, None).await;

    let token = app.login(&email, pw).await.expect("client login");
    let res = app.auth_get("/api/v1/protocols", &token).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse protocols");
    let ids = id_list(&body);

    assert!(
        ids.contains(&assigned.to_string()),
        "CLIENT 應看到被指定的計畫"
    );
    assert!(
        !ids.contains(&not_assigned.to_string()),
        "CLIENT 不應看到未被指定的計畫"
    );
    assert_eq!(
        can_edit_of(&body, assigned),
        Some(false),
        "CLIENT 對草稿 can_edit 應為 false"
    );

    // CLIENT 連純 SD 指派都不允許（require_protocol_sd_assign：非 admin/執秘/can_edit）→ 403
    let put = app
        .auth_put(
            &format!("/api/v1/protocols/{assigned}"),
            &serde_json::json!({"title": "unauthorized edit"}),
            &token,
        )
        .await;
    assert_eq!(put.status(), 403, "CLIENT 編輯內容應 403");
}

/// PI：看得到自己草稿且 can_edit=true；PUT 通過授權（非 403）。
#[tokio::test]
#[serial]
async fn pi_can_edit_own_draft() {
    let app = TestApp::spawn().await;
    let pw = "DraftVis$Pw4";
    let (pi_id, email) = seed_login_user(&app, "pi", "PI", pw).await;
    let my_draft = seed_protocol(&app, "DRAFT", pi_id, None).await;
    // 建立流程通常會自動加 PI 成員；測試直接 seed，故補上成員列以對齊真實資料。
    add_member(&app, my_draft, pi_id, "PI").await;

    let token = app.login(&email, pw).await.expect("pi login");
    let res = app.auth_get("/api/v1/protocols", &token).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse protocols");
    assert!(
        id_list(&body).contains(&my_draft.to_string()),
        "PI 應看到自己的草稿"
    );
    assert_eq!(
        can_edit_of(&body, my_draft),
        Some(true),
        "PI 對自己草稿 can_edit 應為 true"
    );

    // 內容變更（含 title → touches_content）走 content-edit 授權分支；PI 應通過並更新成功（200）。
    let put = app
        .auth_put(
            &format!("/api/v1/protocols/{my_draft}"),
            &serde_json::json!({"title": "PI content edit"}),
            &token,
        )
        .await;
    assert_eq!(put.status(), 200, "PI 編輯自己草稿內容應成功（200）");
}
