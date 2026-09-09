//! P1-6：`GET /api/v1/animals/stats` 的授權行為與同權限判準的鄰居一致。
//!
//! 2026-09-10 之前該端點無權限時回「全 0 的統計」+ 200，而 `list_animals`
//! 與 `list_available_pigs` 用**完全相同**的兩個權限碼判斷、兩者都回 403。
//! 三者不一致會讓人誤判——看到 0 以為自己有權限而場內確實沒豬；
//! 稽核上也看不出「這次查詢被降級過」。
//!
//! 本檔把三個端點放在同一支測試裡一起斷言，理由是**不一致本身才是缺陷**：
//! 只驗 stats 回 403 的話，日後有人把 `list_animals` 改成降級，這裡照樣全綠。

mod common;

use common::TestApp;
use erp_backend::services::AuthService;
use serial_test::serial;
use uuid::Uuid;

const PASSWORD: &str = "TestPassword123!";

/// 開一個**完全沒有任何 `animal.*` 權限**的帳號。
///
/// 用 `WAREHOUSE_MANAGER`：依 `06_PERMISSIONS_RBAC.md` §4.1 的矩陣，該角色在
/// `animal.*`（43 碼）那一格是「—」＝一碼都沒有，因此同時缺
/// `animal.animal.view_all` 與 `animal.animal.view_project`——正是本測試要的條件。
///
/// ⚠️ 斷言 `rows_affected() == 1`：`INSERT … SELECT … WHERE code = $2` 在角色碼
/// 不存在時會插 0 列並回 `Ok`，測試會靜默失去鑑別力（同 MR !10 踩過的坑）。
async fn create_user_without_animal_perms(app: &TestApp) -> String {
    let id = Uuid::new_v4();
    let email = format!(
        "stats_noperm_{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password(PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, 'Stats NoPerm Test', true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(&app.db_pool)
    .await
    .expect("insert user");

    let res = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind("WAREHOUSE_MANAGER")
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    assert_eq!(
        res.rows_affected(),
        1,
        "WAREHOUSE_MANAGER 未被指派——roles 表沒有這個代碼（seed 漏了或被改名）。\
         少了這道斷言，使用者會變成「零角色」而測試依然通過，等於沒驗到東西。"
    );

    app.login(&email, PASSWORD).await.expect("login failed")
}

/// 三個端點的權限判準相同（`animal.animal.view_all` 或 `animal.animal.view_project`），
/// 無權限時**都**必須回 403。
///
/// # 鑑別力
///
/// 對 2026-09-10 之前的舊碼會紅：`/animals/stats` 那條斷言會拿到 200
/// （回應是全 0 的 `AnimalStatsResponse`）而非 403。
/// 另兩條在改動前後都是 403——它們在這裡是**對照組**，用來證明
/// 「403 才是這組端點的既有慣例」，而不是我新發明的行為。
#[tokio::test]
#[serial]
async fn animal_read_endpoints_all_reject_without_permission() {
    let app = TestApp::spawn().await;
    let token = create_user_without_animal_perms(&app).await;

    // 主張：本次修正的目標端點
    let stats = app.auth_get("/api/v1/animals/stats", &token).await;
    assert_eq!(
        stats.status(),
        403,
        "無 animal 檢視權時 /animals/stats 應回 403；200 代表它又退回「回全 0 統計」的降級行為"
    );

    // 對照組一：同權限判準，本來就回 403
    let list = app.auth_get("/api/v1/animals", &token).await;
    assert_eq!(list.status(), 403, "/animals 的既有行為就是 403");

    // 對照組二：同權限判準，本來就回 403
    let available = app.auth_get("/api/v1/animals/available", &token).await;
    assert_eq!(
        available.status(),
        403,
        "/animals/available 的既有行為就是 403"
    );
}

/// 有權限者不受影響：admin 仍拿得到統計，且回應結構完整。
///
/// 這支的用途是擋住「把 403 加得太寬」——只驗無權限回 403 的話，
/// 把整個端點改成永遠 403 也會全綠。
#[tokio::test]
#[serial]
async fn animal_stats_still_works_for_permitted_user() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/animals/stats", &token).await;
    assert_eq!(res.status(), 200, "admin 應仍能取得統計");

    let body: serde_json::Value = res.json().await.expect("parse stats body");
    // 不斷言具體數字（測試庫內容會變），只斷言結構完整——
    // 舊碼的降級路徑回的也是這四個欄位，所以這裡不是鑑別點，是防迴歸。
    for key in [
        "status_counts",
        "pen_animals_count",
        "pen_counts_by_building",
        "total",
    ] {
        assert!(
            body.get(key).is_some(),
            "回應缺少欄位 `{key}`，實際：{body}"
        );
    }
}
