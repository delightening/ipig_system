//! 回歸測試：動物預約與試驗規劃的「檢視 / 操作」權限拆分（migration 144）。
//!
//! 背景：本頁原本讀寫共用 `animal.info.assign`，造成兩個問題 ——
//! 1. 連唯讀查詢也擋在該權限後，研究主持人（SD）與試驗工作人員**連頁面都進不來**。
//! 2. `animal.info.assign` 同時被 `batch_assign_animals` 使用，且由 EXPERIMENT_STAFF /
//!    VET 持有，因此沿用它就做不到使用者要求的「僅執行秘書可操作」。
//!
//! 本測試鎖住使用者 2026-08-07 的裁定，避免日後被無意改回去。
//! 見 `docs/audit/button-permission-gate-2026-08-07.md` §6。

use sqlx::PgPool;

/// 只認 `TEST_DATABASE_URL`，未設時才收明顯是測試庫的 `DATABASE_URL`（名稱含 `test`）。
/// 本測試會跑 migration；連錯 DB 的代價是對正式 schema 動手。
fn test_database_url() -> String {
    let (url, source) = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => (url, "TEST_DATABASE_URL"),
        Err(_) => (
            std::env::var("DATABASE_URL").expect(
                "TEST_DATABASE_URL 與 DATABASE_URL 皆未設定。本測試會跑 migration，\
                 請指向獨立可丟棄的測試 DB。",
            ),
            "DATABASE_URL",
        ),
    };
    let db_name = url
        .rsplit('/')
        .next()
        .and_then(|tail| tail.split(['?', '#']).next())
        .unwrap_or_default();
    assert!(
        db_name.contains("test"),
        "{source} 指向的資料庫 `{db_name}` 不像測試庫（名稱不含 test）。\
         本測試會跑 migration，拒絕在可能是 prod 的連線上執行。"
    );
    url
}

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let pool = PgPool::connect(&test_database_url())
        .await
        .expect("connect test db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn roles_holding(pool: &PgPool, permission_code: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"SELECT r.code
           FROM role_permissions rp
           JOIN roles r       ON r.id = rp.role_id
           JOIN permissions p ON p.id = rp.permission_id
           WHERE p.code = $1
           ORDER BY r.code"#,
    )
    .bind(permission_code)
    .fetch_all(pool)
    .await
    .expect("query roles holding permission")
}

/// 檢視權：執秘 + 全體試驗工作人員 + 負責人。
/// admin 不需顯式授權（middleware `is_admin` 一律放行）。
///
/// 原本還有第四個角色 `STUDY_DIRECTOR`，用意是涵蓋「已被指派為 SD 但帳號未帶
/// `EXPERIMENT_STAFF`」的歷史帳號。該全域角色已於 2026-08-17 移除
/// （SD 只存在於 `protocols.study_director_user_id`），而
/// `validate_and_authorize_sd` 本來就強制 SD 必須具 `EXPERIMENT_STAFF`，
/// 新指派的 SD 一律被第二項涵蓋。
#[tokio::test]
async fn planning_view_granted_to_exactly_the_three_intended_roles() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "animal.planning.view").await;

    for expected in ["IACUC_STAFF", "EXPERIMENT_STAFF", "DIRECTOR"] {
        assert!(
            holders.iter().any(|r| r == expected),
            "{expected} 應有 animal.planning.view（實際持有者：{holders:?}）"
        );
    }
}

/// 使用者明確裁定：獸醫與計畫主持人**連檢視都不給**。
///
/// 這一例特別重要——初版 migration 曾用「既有持有 animal.info.assign 者一併補檢視權」
/// 的回填語句，結果 VET 因持有該權限而意外拿到檢視權，違反裁定。
#[tokio::test]
async fn planning_view_not_granted_to_vet_or_pi() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "animal.planning.view").await;

    for forbidden in ["VET", "PI"] {
        assert!(
            !holders.iter().any(|r| r == forbidden),
            "{forbidden} 不得有 animal.planning.view（實際持有者：{holders:?}）"
        );
    }
}

/// 操作權：僅執行秘書。admin 走 bypass，不列在此。
#[tokio::test]
async fn planning_manage_granted_only_to_iacuc_staff() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "animal.planning.manage").await;

    assert_eq!(
        holders,
        vec!["IACUC_STAFF".to_string()],
        "animal.planning.manage 應僅授予執行秘書；admin 由 is_admin bypass，不需顯式授權"
    );
}

/// `animal.info.assign` 不得被本次拆分波及 —— 它同時是
/// `batch_assign_animals`（動物列表的批次分配到計畫）的閘。
/// 若為了「僅執秘可操作規劃頁」而從 EXPERIMENT_STAFF 拔掉它，
/// 會連帶砍掉他們在別處的批次分配功能。
#[tokio::test]
async fn info_assign_still_held_by_experiment_staff() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "animal.info.assign").await;

    assert!(
        holders.iter().any(|r| r == "EXPERIMENT_STAFF"),
        "EXPERIMENT_STAFF 必須保留 animal.info.assign（batch_assign_animals 用），\
         實際持有者：{holders:?}"
    );
}
