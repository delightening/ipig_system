//! 計畫詳情頁的 PI 顯示資訊（`ProtocolService::get_by_id`）要以
//! `working_content.basic.pi` 為準，fallback 至 FK 使用者——跟 `list()` /
//! `my_protocols` 等其餘「顯示計畫 PI」查詢一致（`utils::pi_sql`）。
//!
//! 🔴 **回歸測試**：`get_by_id` 先前直接
//! `SELECT display_name, email, organization FROM users WHERE id = pi_user_id`，
//! 沒有走 `pi_sql` helper。外部 PI（無系統帳號）時 `pi_user_id` 只是匯入者的
//! 佔位值——計畫詳情頁因此顯示**匯入者本人**的姓名/email/單位，而不是
//! `working_content.basic.pi` 記的真正外部 PI 資訊。本檔守住修復後的行為。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::models::ImportApprovedProtocolRequest;
use erp_backend::services::ProtocolService;
use erp_backend::ActorContext;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "pi_display_detail_regression",
};

async fn seed_sd(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'display sd', true, false)"#,
    )
    .bind(id)
    .bind(format!(
        "display-sd-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    ))
    .execute(&app.db_pool)
    .await
    .expect("insert sd user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign EXPERIMENT_STAFF role");
    id
}

/// 🔴 核心：外部 PI 匯入的計畫，詳情頁應顯示 `working_content.basic.pi` 的
/// 姓名/email/單位，不是匯入者（佔位值）的帳號資訊。
#[tokio::test]
#[serial]
async fn get_by_id_shows_external_pi_not_importer() {
    let app = TestApp::spawn().await;
    let importer = seed_sd(&app).await;
    let sd = seed_sd(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    let req = ImportApprovedProtocolRequest {
        title: "PI 顯示回歸測試計劃".to_string(),
        pi_user_id: None, // 外部 PI，pi_user_id 佔位成 importer
        study_director_user_id: sd,
        iacuc_no: format!("PIG-DISP-{unique}"),
        application_no: None,
        working_content: Some(serde_json::json!({
            "basic": {
                "pi": { "name": "外部王教授", "email": "wang@external.example" },
                "sponsor": { "name": "王教授實驗室" }
            }
        })),
        start_date: None,
        end_date: None,
        submitted_at: None,
        pre_review_at: None,
        vet_review_at: None,
        committee_first_review_at: None,
        revision_required_at: None,
        committee_second_review_at: None,
        approved_at: None,
        remark: None,
        notice_version_label: None,
        notice_attachment_id: None,
        notice_acknowledged_at: None,
        source_form_version: None,
    };
    let p = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, importer)
        .await
        .expect("外部 PI 匯入應成功");
    assert_eq!(p.pi_user_id, importer, "pi_user_id 應為匯入者的佔位值");

    let view = ProtocolService::get_by_id(&app.db_pool, p.id)
        .await
        .expect("get_by_id");

    assert_eq!(
        view.pi_name.as_deref(),
        Some("外部王教授"),
        "詳情頁 PI 姓名應顯示 working_content.basic.pi.name，不是匯入者本人"
    );
    assert_eq!(
        view.pi_email.as_deref(),
        Some("wang@external.example"),
        "詳情頁 PI email 應顯示 working_content.basic.pi.email，不是匯入者本人"
    );
    assert_eq!(
        view.pi_organization.as_deref(),
        Some("王教授實驗室"),
        "詳情頁委託單位應顯示 working_content.basic.sponsor.name"
    );
}

/// 對照組：系統內真正的 PI（有帳號、`working_content` 沒填 `basic.pi`）
/// 應該 fallback 回 FK 使用者的姓名/email/單位——不要因為修復外部 PI
/// 而連真 PI 的正常顯示都跟著壞掉。
#[tokio::test]
#[serial]
async fn get_by_id_falls_back_to_fk_user_when_no_basic_pi() {
    let app = TestApp::spawn().await;
    let pi = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, organization, is_active, must_change_password)
           VALUES ($1, $2, 'fake', '系統內林教授', '林教授實驗室', true, false)"#,
    )
    .bind(pi)
    .bind(format!(
        "real-pi-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    ))
    .execute(&app.db_pool)
    .await
    .expect("insert pi user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");

    let req = erp_backend::models::CreateProtocolRequest {
        title: "真 PI 對照組".to_string(),
        pi_user_id: Some(pi),
        working_content: Some(serde_json::json!({ "basic": { "is_glp": false } })),
        start_date: None,
        end_date: None,
        study_director_user_id: None,
    };
    let actor = ActorContext::System {
        reason: "pi_display_detail_regression",
    };
    let p = ProtocolService::create(&app.db_pool, &actor, &req, pi)
        .await
        .expect("create");

    let view = ProtocolService::get_by_id(&app.db_pool, p.id)
        .await
        .expect("get_by_id");

    assert_eq!(view.pi_name.as_deref(), Some("系統內林教授"));
    assert_eq!(
        view.pi_organization.as_deref(),
        Some("林教授實驗室"),
        "沒有 basic.sponsor 時應 fallback 回 FK 使用者的 organization"
    );
}
