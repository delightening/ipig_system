//! 待處理人候選名單必須與授權判準同源。
//!
//! ## 為什麼要有這支測試
//!
//! 「待核准」徽章 hover 顯示「卡在王倉管」，使用者去催王倉管，王倉管點下去拿 403——
//! 這比不顯示更糟。候選名單與 handler 守衛只要分岔一次，整個功能就從幫忙變成誤導。
//!
//! 分岔最容易發生的地方是**授權判準裡不寫在授權表裡的那部分**：
//! `CurrentUser::has_permission`（`middleware/auth.rs:88-90`）對管理員一律回 true，
//! 不看 `role_permissions` 有沒有那一列。只比對授權表的查詢會安靜地少列管理員，
//! 而少列不會讓任何既有測試變紅。
//!
//! 本檔鎖住兩側：該列的必須列（含短路取得權限的管理員）、不該列的必須不列。

mod common;
use common::TestApp;
use erp_backend::repositories::pending_owner;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 刻意不存在於任何 `role_permissions` 的權限碼。
///
/// 用它才能區分「因為授權表有這一列而入選」與「因為管理員短路而入選」——
/// 拿真實權限碼查，管理員多半兩條路都成立，測不出短路有沒有被實作。
const UNGRANTED_PERMISSION: &str = "pending.owner.test.never.granted";

/// 管理員在 `roles` 表裡的真實代碼。
///
/// ⚠️ 是 `admin` 不是 `SYSTEM_ADMIN`。`constants.rs` 兩個都有定義、`is_admin()` 兩個都收，
/// 但**資料庫裡只有 `admin`**（2026-08-26 實查 15 個角色代碼）。
/// `RULES_BACKEND.md` §7.1 記載過同一個坑：fixture 用了不存在的 `SYSTEM_ADMIN` role code，
/// 連兩次紅 CI 才找到根因。
const ADMIN_ROLE: &str = "admin";

async fn seed_user_with_role(pool: &PgPool, label: &str, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(id)
    .bind(format!("{label}-{}@example.com", &suffix[..8]))
    .bind(format!("{label}-{}", &suffix[..4]))
    .execute(pool)
    .await
    .expect("seed user");

    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) \
         SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("grant role");

    id
}

async fn deactivate(pool: &PgPool, user_id: Uuid) {
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("deactivate user");
}

#[tokio::test]
#[serial]
async fn admins_are_listed_even_without_the_permission_row() {
    let app = TestApp::spawn().await;
    let admin = seed_user_with_role(&app.db_pool, "po-admin", ADMIN_ROLE).await;

    let listed = pending_owner::list_users_with_permission(&app.db_pool, UNGRANTED_PERMISSION)
        .await
        .expect("list candidates");

    assert!(
        listed.iter().any(|(id, _)| *id == admin),
        "管理員必須入選：`CurrentUser::has_permission`（middleware/auth.rs:88-90）\n\
         對管理員一律回 true，他實際上真的能核准。只掃 role_permissions 會漏掉他，\n\
         候選名單就少了一個真正能解卡的人。實際列出：{listed:?}"
    );
}

#[tokio::test]
#[serial]
async fn non_admins_without_the_permission_are_not_listed() {
    let app = TestApp::spawn().await;
    let staff = seed_user_with_role(&app.db_pool, "po-staff", "EXPERIMENT_STAFF").await;

    let listed = pending_owner::list_users_with_permission(&app.db_pool, UNGRANTED_PERMISSION)
        .await
        .expect("list candidates");

    assert!(
        !listed.iter().any(|(id, _)| *id == staff),
        "沒有該權限、也不是管理員的人不得入選——列出去等於叫使用者去催一個做不到的人"
    );
}

#[tokio::test]
#[serial]
async fn deactivated_users_never_appear() {
    let app = TestApp::spawn().await;
    let admin = seed_user_with_role(&app.db_pool, "po-gone", ADMIN_ROLE).await;
    deactivate(&app.db_pool, admin).await;

    let listed = pending_owner::list_users_with_permission(&app.db_pool, UNGRANTED_PERMISSION)
        .await
        .expect("list candidates");

    assert!(
        !listed.iter().any(|(id, _)| *id == admin),
        "停用帳號不得出現在「卡在誰」名單上——使用者會去催一個已經離職的人"
    );
}

/// 角色那一半**沒有**管理員短路。
///
/// `handlers/document.rs:213` 比對的是 `current_user.roles.contains(WAREHOUSE_MANAGER)`，
/// 那是純角色比對、不吃 `has_permission` 的短路。管理員若不具倉管角色，
/// 送出核准一樣會被擋，所以不能列進倉管關的候選人。
#[tokio::test]
#[serial]
async fn admin_without_the_required_role_is_excluded_from_two_condition_stages() {
    let app = TestApp::spawn().await;
    let admin = seed_user_with_role(&app.db_pool, "po-admin-norole", ADMIN_ROLE).await;

    let listed = pending_owner::list_users_with_permission_and_any_role(
        &app.db_pool,
        UNGRANTED_PERMISSION,
        &["WAREHOUSE_MANAGER".to_string()],
    )
    .await
    .expect("list candidates");

    assert!(
        !listed.iter().any(|(id, _)| *id == admin),
        "管理員的權限短路過得了權限那一關，但角色那一關擋的是 roles.contains()——\n\
         列出他等於叫使用者去催一個會拿 403 的人"
    );
}

#[tokio::test]
#[serial]
async fn two_condition_stage_lists_users_satisfying_both_halves() {
    let app = TestApp::spawn().await;
    // 同時是管理員（滿足權限半邊的短路）與倉管（滿足角色半邊）。
    let wm_admin = seed_user_with_role(&app.db_pool, "po-wm-admin", ADMIN_ROLE).await;
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) \
         SELECT $1, id FROM roles WHERE code = 'WAREHOUSE_MANAGER'",
    )
    .bind(wm_admin)
    .execute(&app.db_pool)
    .await
    .expect("grant warehouse manager role");

    let listed = pending_owner::list_users_with_permission_and_any_role(
        &app.db_pool,
        UNGRANTED_PERMISSION,
        &["WAREHOUSE_MANAGER".to_string()],
    )
    .await
    .expect("list candidates");

    assert!(
        listed.iter().any(|(id, _)| *id == wm_admin),
        "兩個條件都滿足的人必須入選。實際列出：{listed:?}"
    );
}

#[tokio::test]
#[serial]
async fn any_role_lookup_matches_either_role() {
    let app = TestApp::spawn().await;
    let admin_staff = seed_user_with_role(&app.db_pool, "po-adminstaff", "ADMIN_STAFF").await;
    let sysadmin = seed_user_with_role(&app.db_pool, "po-sysadmin", ADMIN_ROLE).await;

    // 加班第一關（pending_admin_staff）的判準：is_admin || ADMIN_STAFF
    // （`services/hr/overtime.rs:248`）。
    let listed = pending_owner::list_users_with_any_role(
        &app.db_pool,
        &[ADMIN_ROLE.to_string(), "ADMIN_STAFF".to_string()],
    )
    .await
    .expect("list candidates");

    assert!(
        listed.iter().any(|(id, _)| *id == admin_staff),
        "ADMIN_STAFF 應入選"
    );
    assert!(
        listed.iter().any(|(id, _)| *id == sysadmin),
        "SYSTEM_ADMIN 應入選"
    );
}

#[tokio::test]
#[serial]
async fn empty_role_list_returns_nobody_instead_of_everybody() {
    let app = TestApp::spawn().await;

    // 空清單若被翻成 `code = ANY('{}')` 以外的東西（例如省略條件），
    // 會變成「全體使用者」——那是把所有人都列成候選人的災難性預設。
    let listed = pending_owner::list_users_with_any_role(&app.db_pool, &[])
        .await
        .expect("list candidates");
    assert!(listed.is_empty(), "空角色清單必須回空，不得退化成全體");

    let listed = pending_owner::list_users_with_permission_and_any_role(
        &app.db_pool,
        UNGRANTED_PERMISSION,
        &[],
    )
    .await
    .expect("list candidates");
    assert!(listed.is_empty(), "空角色清單必須回空，不得退化成只看權限");
}

/// 管理員短路所依賴的角色代碼必須真的存在於 `roles` 表。
///
/// 這條防的是**靜默失效**：`repositories/pending_owner.rs` 的 OR 分支比對
/// `constants.rs` 的 `ROLE_SYSTEM_ADMIN` / `ROLE_ADMIN_LEGACY`，兩者若都對不上
/// 實際資料，查詢照樣執行、照樣回結果，只是永遠少列管理員——沒有任何錯誤訊息，
/// 上面那些測試也不會紅（它們各自 seed 自己的使用者）。
///
/// 2026-08-26 實查：`roles` 只有 `admin`，**沒有** `SYSTEM_ADMIN`。
#[tokio::test]
#[serial]
async fn admin_short_circuit_role_codes_exist_in_the_database() {
    let app = TestApp::spawn().await;

    let existing = sqlx::query_scalar::<_, String>(
        "SELECT code FROM roles WHERE code = ANY($1) ORDER BY code",
    )
    .bind(vec![
        erp_backend::constants::ROLE_SYSTEM_ADMIN.to_string(),
        erp_backend::constants::ROLE_ADMIN_LEGACY.to_string(),
    ])
    .fetch_all(&app.db_pool)
    .await
    .expect("query admin roles");

    assert!(
        !existing.is_empty(),
        "`constants.rs` 的管理員角色代碼沒有一個存在於 roles 表。\n\
         管理員短路會靜默失效：候選名單永遠少列管理員，卻不會有任何錯誤。\n\
         實際存在的代碼請以 roles 表為準，並同步 constants.rs 與本測試。"
    );
}
