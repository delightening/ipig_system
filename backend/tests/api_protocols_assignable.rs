//! `GET /api/v1/protocols?assignable=true` 過濾回歸測試（效能 P1）。
//!
//! 「可指派計畫」= 已核准（APPROVED / APPROVED_WITH_CONDITIONS）且具 iacuc_no，
//! 供動物 IACUC No. 下拉用。後端於 view_all 與「我的計劃」兩條授權路徑皆套用過濾，
//! 各自可見範圍不變（過濾在範圍內取交集）。

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
        "assignable-{label}-{}@test.local",
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
    .bind(format!("assignable {label}"))
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

/// 建立計畫，可指定 status 與 iacuc_no（None → NULL）。回傳 id。
async fn seed_protocol(app: &TestApp, pi: Uuid, status: &str, iacuc: Option<String>) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'assignable filter test', $4::protocol_status, $5, $5, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("AS-{unique}"))
    .bind(iacuc)
    .bind(status)
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

fn ids_of(body: &serde_json::Value) -> Vec<String> {
    body.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// view_all（admin）：assignable=true 僅回已核准 + 有 iacuc_no 的計畫。
#[tokio::test]
#[serial]
async fn assignable_view_all_filters_to_approved_with_iacuc() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 任意 PI（不需可登入，僅供 pi_user_id）
    let (pi, _email) = seed_login_user(&app, "pi-va", "PI", "Assignable$Pw1").await;

    let approved = seed_protocol(
        &app,
        pi,
        "APPROVED",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;
    let awc = seed_protocol(
        &app,
        pi,
        "APPROVED_WITH_CONDITIONS",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;
    let approved_no_iacuc = seed_protocol(&app, pi, "APPROVED", None).await;
    let draft = seed_protocol(
        &app,
        pi,
        "DRAFT",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;

    let res = app
        .auth_get("/api/v1/protocols?assignable=true", &token)
        .await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse assignable list");
    let ids = ids_of(&body);

    assert!(
        ids.contains(&approved.to_string()),
        "APPROVED + iacuc 應包含"
    );
    assert!(
        ids.contains(&awc.to_string()),
        "APPROVED_WITH_CONDITIONS + iacuc 應包含"
    );
    assert!(
        !ids.contains(&approved_no_iacuc.to_string()),
        "APPROVED 但無 iacuc 應排除"
    );
    assert!(!ids.contains(&draft.to_string()), "DRAFT 應排除");
}

/// 我的計劃路徑（PI，無 view_all）：assignable 在成員制範圍內取交集，
/// 只回自己且已核准 + 有 iacuc_no 的計畫；他人計畫與自己的草稿皆排除。
#[tokio::test]
#[serial]
async fn assignable_my_protocols_path_intersects_membership() {
    let app = TestApp::spawn().await;

    let pw = "Assignable$Pw2";
    let (pi, email) = seed_login_user(&app, "pi-mine", "PI", pw).await;
    let (other_pi, _e2) = seed_login_user(&app, "pi-other", "PI", "Assignable$Pw3").await;

    let mine_approved = seed_protocol(
        &app,
        pi,
        "APPROVED",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;
    let mine_draft = seed_protocol(
        &app,
        pi,
        "DRAFT",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;
    let other_approved = seed_protocol(
        &app,
        other_pi,
        "APPROVED",
        Some(format!("APIG-{}", &Uuid::new_v4().to_string()[..6])),
    )
    .await;

    let token = app.login(&email, pw).await.expect("pi login");
    let res = app
        .auth_get("/api/v1/protocols?assignable=true", &token)
        .await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.expect("parse assignable my list");
    let ids = ids_of(&body);

    assert!(
        ids.contains(&mine_approved.to_string()),
        "自己 APPROVED + iacuc 應包含"
    );
    assert!(
        !ids.contains(&mine_draft.to_string()),
        "自己的 DRAFT 應排除"
    );
    assert!(
        !ids.contains(&other_approved.to_string()),
        "他人計畫應排除（成員制）"
    );
}
