//! Pentest F1（2026-07-05）：功能級授權缺失回歸測試。
//!
//! 修補前以下四端點無任何 permission gate，任何已認證者（含外部 CLIENT，
//! is_internal=false）皆可讀內部營運資料：
//!   - `GET /api/v1/alerts/low-stock` / `/alerts/expiry` → 全院庫存/效期告警
//!   - `GET /api/v1/treatment-drugs`                    → 醫療用藥處方參考
//!   - `GET /api/v1/animal-sources`                     → 供應商 address/contact/phone（PII）
//!
//! 修補：
//!   - alerts   → `erp.stock.view`（對齊前端 Dashboard 只在 hasErpPermission 時才呼叫）
//!   - drugs / sources → `animal.animal.view_all`（涵蓋內部動物營運角色 EXPERIMENT_STAFF/VET，
//!     擋外部 CLIENT / PI 的 view_project；不用 source.manage 以免誤殺 EXPERIMENT_STAFF 下拉）
//!
//! 本測試以四種身分交叉驗證，避免 gate 被移除（→ 外洩）或寫過嚴（→ 誤殺內部角色）：
//!   noperm/CLIENT 全 403、EXPERIMENT_STAFF/admin 全 200。

mod common;

use common::TestApp;
use serial_test::serial;

/// F1 受管的四個端點。
const F1_ENDPOINTS: &[&str] = &[
    "/api/v1/alerts/low-stock",
    "/api/v1/alerts/expiry",
    "/api/v1/treatment-drugs",
    "/api/v1/animal-sources",
];

/// 依 role code 取 role_id（None = 零權限使用者）。
async fn role_id_by_code(app: &TestApp, code: &str) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM roles WHERE code = $1 AND is_active = true LIMIT 1",
    )
    .bind(code)
    .fetch_one(&app.db_pool)
    .await
    .unwrap_or_else(|_| panic!("role {code} must exist after startup migrations"))
}

/// 以指定 role_ids（空 = 零權限）建立使用者並登入，回傳 access token。
/// 注意：CreateUserRequest 的欄位是 `role_ids: Vec<Uuid>`（陣列），非 `role_id`；
/// 傳錯欄位會被 serde default 成空陣列（靜默建成零權限帳號）。
///
/// `is_internal` 由呼叫端明確帶——**不給預設值**。這裡曾硬編 `true`，使得
/// `external_client_blocked_on_all_f1_endpoints` 明明宣稱在測「外部委託人
/// （is_internal=false）」，實際建的卻是內部 CLIENT，註解與實作相反。
async fn create_user_and_login(
    app: &TestApp,
    admin_token: &str,
    role_ids: &[uuid::Uuid],
    tag: &str,
    is_internal: bool,
) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time error")
        .subsec_nanos();
    let email = format!("f1_{tag}_{ts}@test.local");
    let res = app
        .auth_post(
            "/api/v1/users",
            &serde_json::json!({
                "email": email,
                "display_name": format!("F1 {tag} Test"),
                "password": "TestPassword123!",
                "role_ids": role_ids,
                // is_internal 為必填（無 serde default）——刻意如此，見 CreateUserRequest
                "is_internal": is_internal,
            }),
            admin_token,
        )
        .await;
    assert!(
        res.status() == 201 || res.status() == 200,
        "Create {tag} user returned {}",
        res.status()
    );
    app.login(&email, "TestPassword123!")
        .await
        .unwrap_or_else(|| panic!("{tag} user login failed"))
}

/// 斷言 token 對四端點皆回指定 status。
async fn assert_all(app: &TestApp, token: &str, expected: u16, who: &str) {
    for path in F1_ENDPOINTS {
        let status = app.auth_get(path, token).await.status();
        assert_eq!(
            status, expected,
            "{who} 對 {path} 應回 {expected}（實際 {status}）"
        );
    }
}

#[tokio::test]
#[serial]
async fn noperm_user_blocked_on_all_f1_endpoints() {
    let app = TestApp::spawn().await;
    let admin = app.login_as_admin().await;
    let token = create_user_and_login(&app, &admin, &[], "noperm", true).await;
    assert_all(&app, &token, 403, "零權限使用者").await;
}

#[tokio::test]
#[serial]
async fn external_client_blocked_on_all_f1_endpoints() {
    // 外部委託人（is_internal=false）：F1 的核心威脅主體 — 應全 403。
    let app = TestApp::spawn().await;
    let admin = app.login_as_admin().await;
    let client_role = role_id_by_code(&app, "CLIENT").await;
    let token = create_user_and_login(&app, &admin, &[client_role], "client", false).await;
    assert_all(&app, &token, 403, "外部 CLIENT").await;
}

#[tokio::test]
#[serial]
async fn internal_experiment_staff_allowed_on_all_f1_endpoints() {
    // EXPERIMENT_STAFF 具 erp.stock.view + animal.animal.view_all → 四端點皆應 200。
    // 此案專門守住「gate 不得過嚴誤殺內部建檔角色」（曾誤用 source.manage 破其下拉）。
    let app = TestApp::spawn().await;
    let admin = app.login_as_admin().await;
    let staff_role = role_id_by_code(&app, "EXPERIMENT_STAFF").await;
    let token = create_user_and_login(&app, &admin, &[staff_role], "staff", true).await;
    assert_all(&app, &token, 200, "內部 EXPERIMENT_STAFF").await;
}

#[tokio::test]
#[serial]
async fn admin_allowed_on_all_f1_endpoints() {
    let app = TestApp::spawn().await;
    let admin = app.login_as_admin().await;
    assert_all(&app, &admin, 200, "admin").await;
}
