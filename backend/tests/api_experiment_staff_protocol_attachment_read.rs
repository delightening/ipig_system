//! 回歸測試：EXPERIMENT_STAFF/INTERN 持 `aup.protocol.view_all` 時應能讀取
//! （list/download）**任一**計畫的附件，不限自己相關的計畫。
//!
//! 背景（2026-08-14，Qodo 於 PR #136 review 抓到）：`check_attachment_permission`
//! 的 protocol 分支原本硬性要求角色權限碼 `aup.protocol.edit`。同一支 PR 把這個
//! 權限碼從 EXPERIMENT_STAFF/INTERN 收回（R76-2 拆除 CO_EDITOR 後這個碼對編輯
//! 已無意義），會讓這兩個角色連自己的計畫附件都讀不到——因為 `require_permission!`
//! 在最前面無條件擋下，根本進不到後面真正判斷「你是不是這個計畫的相關人」的
//! `require_protocol_related_access`（那裡其實認得 `aup.protocol.view_all`）。
//!
//! 修法：讀取路徑改成只靠 `require_protocol_related_access`（涵蓋 view_all 與計畫
//! 關聯），不再前置檢查 `aup.protocol.edit`——見 `handlers/upload.rs` 的
//! `check_attachment_permission`。本測試鎖住修好後的行為。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "expstaff-attach-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("expstaff attach {label}"))
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

/// 建立一個 APPROVED 計畫，pi_user_id 隨機（與受測角色無任何關聯）。
async fn seed_unrelated_protocol(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    let pi_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'unrelated pi', true, false)"#,
    )
    .bind(pi_id)
    .bind(format!("expstaff-attach-pi-{unique}@test.local"))
    .execute(&app.db_pool)
    .await
    .expect("insert pi");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'unrelated protocol', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PRAT-{unique}"))
    .bind(format!("IACUCAT-{unique}"))
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

async fn seed_protocol_attachment(app: &TestApp, protocol_id: Uuid, uploaded_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO attachments (id, category, entity_type, entity_id, file_name, file_path, file_size, mime_type, uploaded_by)
           VALUES ($1, 'protocol_attachment', 'protocol', $2, 'doc.pdf', '/tmp/expstaff-attach-none.pdf', 10, 'application/pdf', $3)"#,
    )
    .bind(id)
    .bind(protocol_id)
    .bind(uploaded_by)
    .execute(&app.db_pool)
    .await
    .expect("insert attachment");
    id
}

async fn assert_role_can_list_and_download_unrelated_protocol_attachment(role_code: &str) {
    let app = TestApp::spawn().await;
    let pw = "iPig$ecure1";

    let (user_id, email) = seed_login_user(&app, role_code, role_code).await;
    let protocol_id = seed_unrelated_protocol(&app).await;
    let att_id = seed_protocol_attachment(&app, protocol_id, user_id).await;

    let token = app.login(&email, pw).await.expect("login 應成功");

    let res = app
        .auth_get(
            &format!("/api/v1/attachments?entity_type=protocol&entity_id={protocol_id}"),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        200,
        "{role_code}（持 aup.protocol.view_all）應能列出非自己相關計畫的附件，實得 {}",
        res.status()
    );
    let body: Vec<serde_json::Value> = TestApp::json(res).await;
    assert!(
        body.iter()
            .any(|a| a["id"].as_str() == Some(att_id.to_string().as_str())),
        "列表應包含該附件"
    );

    // 下載端點的授權檢查（check_attachment_permission）跑在實際讀檔之前，但
    // 這裡的附件是純 DB fixture、磁碟上沒有對應檔案，讀檔那步必定失敗——
    // 故只斷言「不是 403」（授權通過），不斷言 200（那要看檔案系統，不是本測試範圍）。
    let res = app
        .auth_get(&format!("/api/v1/attachments/{att_id}"), &token)
        .await;
    assert_ne!(
        res.status().as_u16(),
        403,
        "{role_code}（持 aup.protocol.view_all）下載非自己相關計畫的附件不應被授權擋下（403），實得 {}",
        res.status()
    );
}

#[tokio::test]
#[serial]
async fn experiment_staff_can_read_unrelated_protocol_attachment() {
    assert_role_can_list_and_download_unrelated_protocol_attachment("EXPERIMENT_STAFF").await;
}

#[tokio::test]
#[serial]
async fn intern_can_read_unrelated_protocol_attachment() {
    assert_role_can_list_and_download_unrelated_protocol_attachment("INTERN").await;
}
