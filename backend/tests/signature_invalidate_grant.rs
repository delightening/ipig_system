//! 回歸測試：`signature.invalidate` 授予 QAU 與 DIRECTOR（2026-09-06 使用者裁定，P0-1）。
//!
//! 背景：這個權限碼定義在 `003_seed.sql:329`，但在本次之前**授予零角色**——
//! 等於只有 admin 靠 `CurrentUser::has_permission` 的短路做得到
//! （`middleware/auth.rs`：`is_admin()` 一律放行）。而「簽章作廢的執行者＝系統管理員」
//! 在 GLP 稽核上站不住：作廢是品保判斷（簽錯人、離職撤回、signer key compromise），
//! 不是有 root 權限的人該決定的事。
//!
//! # 為什麼這支測試不能只跑 migration
//!
//! 授予寫在 **Rust 的 `ensure_all_role_permissions`**（`startup/permissions.rs`），
//! 不在任何 migration 裡。鄰檔 `animal_planning_permission_split.rs` 那種
//! 「只跑 `sqlx::migrate!` 再查 `role_permissions`」的形狀對本項**必然查無**，
//! 會讓人誤判成授予失敗。本檔因此在斷言前顯式呼叫那兩支 startup 函式。
//!
//! # 鑑別力
//!
//! `grant_comes_from_startup_sync_not_migrations` 先在**只跑完 migration** 的狀態下
//! 斷言兩個角色**還沒有**這個權限，再跑 startup 同步、斷言變成有。
//! 少了前半段，這支測試在「授予其實來自 seed」時也會綠——那就證明不了
//! `permissions.rs` 那兩行有在做事，日後有人刪掉它也不會紅。

use sqlx::PgPool;

use erp_backend::startup::{ensure_all_role_permissions, ensure_required_permissions};

#[path = "common/test_db.rs"]
mod test_db;

const CODE: &str = "signature.invalidate";
/// 使用者 2026-09-06 裁定的授予對象。admin／SYSTEM_ADMIN 不需顯式授權（短路放行）。
const EXPECTED_ROLES: [&str; 2] = ["QAU", "DIRECTOR"];

async fn migrated_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，比照鄰檔明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
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

/// 跑完 startup 同步後，QAU 與 DIRECTOR 都持有 `signature.invalidate`。
#[tokio::test]
async fn qau_and_director_hold_signature_invalidate() {
    let pool = migrated_pool().await;
    ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    ensure_all_role_permissions(&pool)
        .await
        .expect("ensure_all_role_permissions");

    let holders = roles_holding(&pool, CODE).await;
    for expected in EXPECTED_ROLES {
        assert!(
            holders.iter().any(|r| r == expected),
            "{expected} 應持有 {CODE}（實際持有者：{holders:?}）"
        );
    }
}

/// 鑑別力測試：證明授予真的來自 `permissions.rs` 的 startup 同步，而不是 migration/seed。
///
/// 這支同時也是「刪掉那兩行就會紅」的保險——若日後有人把授予從
/// `ensure_all_role_permissions` 拿掉，後半段的斷言會失敗。
#[tokio::test]
async fn grant_comes_from_startup_sync_not_migrations() {
    let pool = migrated_pool().await;

    let before = roles_holding(&pool, CODE).await;
    for expected in EXPECTED_ROLES {
        assert!(
            !before.iter().any(|r| r == expected),
            "測試前提不成立：只跑 migration 時 {expected} 就已持有 {CODE}，\
             代表授予來源已改成 migration/seed，本檔的鑑別力假設要重寫（實際：{before:?}）"
        );
    }

    ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    ensure_all_role_permissions(&pool)
        .await
        .expect("ensure_all_role_permissions");

    let after = roles_holding(&pool, CODE).await;
    for expected in EXPECTED_ROLES {
        assert!(
            after.iter().any(|r| r == expected),
            "{expected} 應在 startup 同步後取得 {CODE}（實際：{after:?}）"
        );
    }
}

/// 不是「發給大家」——除了裁定的兩個角色與 admin 類角色之外不應擴散。
///
/// ⚠️ 這裡刻意**不**斷言 holders 完全等於某個固定集合：`003_seed.sql` 可能已經
/// 授予 admin／SYSTEM_ADMIN，且日後使用者可能再加人。本測試守的是
/// 「一般作業角色不該拿到簽章作廢權」這條線。
#[tokio::test]
async fn signature_invalidate_not_granted_to_operational_roles() {
    let pool = migrated_pool().await;
    ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    ensure_all_role_permissions(&pool)
        .await
        .expect("ensure_all_role_permissions");

    let holders = roles_holding(&pool, CODE).await;
    for forbidden in [
        "EXPERIMENT_STAFF",
        "INTERN",
        "VET",
        "IACUC_STAFF",
        "PURCHASING",
    ] {
        assert!(
            !holders.iter().any(|r| r == forbidden),
            "{forbidden} 不得持有 {CODE}——簽章作廢是品保／負責人層級的稀有操作，\
             不是日常作業權限（實際持有者：{holders:?}）"
        );
    }
}
