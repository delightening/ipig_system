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

use serial_test::serial;
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
///
/// `#[serial]`：與下面那支破壞性測試共用同一顆測試庫，不能與它並行
/// （它會短暫清空本碼的授予列）。
#[tokio::test]
#[serial]
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
/// # 初版是錯的，錯在哪裡（2026-09-06，CI 抓到）
///
/// 初版先斷言「只跑完 migration 時兩個角色**還沒有**這個權限」，再跑同步、斷言變成有。
/// **那個前提在本專案不可能成立**：`common/test_db.rs::connect_disposable` 連的是
/// `TEST_DATABASE_URL` 指向的**同一顆共用測試庫**（它做的是「確認這顆庫可安全丟棄」，
/// 不是每次建一顆新的），而任何跑過 `TestApp::spawn()` 的測試都會執行完整 app 啟動、
/// 連帶跑掉 `ensure_all_role_permissions`。輪到本檔時授予**早就存在**，
/// 初版必然紅在前提斷言上——紅的是測試的假設，不是被測的程式。
///
/// # 改法：不假設初始狀態，自己造出來
///
/// 先把這個權限碼的所有 `role_permissions` 列清掉，再跑 startup 同步。
/// 若同步後兩個角色又持有它，那就只可能來自 `permissions.rs`——migration 不會重跑。
/// 這比「假設它一開始沒有」更強：它直接證明那兩行在做事，刪掉就會紅。
///
/// ⚠️ **破壞性操作 + 共用測試庫**：故標 `#[serial]`，且**清除與還原之間不放任何斷言**
/// （斷言在中間 panic 會讓共用庫留在「少了這個授予」的狀態，害到後面的測試）。
/// 這是本專案 2026-09-04b 記取過的教訓：mutation 驗證會留下殘骸，要主動保證還原。
#[tokio::test]
#[serial]
async fn grant_comes_from_startup_sync_not_migrations() {
    let pool = migrated_pool().await;

    // ── 破壞 ──（此後到還原完成之前，不得有任何 assert）
    sqlx::query(
        "DELETE FROM role_permissions \
         WHERE permission_id = (SELECT id FROM permissions WHERE code = $1)",
    )
    .bind(CODE)
    .execute(&pool)
    .await
    .expect("clear existing grants for the code under test");

    // ── 還原 ──
    ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    ensure_all_role_permissions(&pool)
        .await
        .expect("ensure_all_role_permissions");

    // ── 還原完成，才開始斷言 ──
    let after = roles_holding(&pool, CODE).await;
    for expected in EXPECTED_ROLES {
        assert!(
            after.iter().any(|r| r == expected),
            "把 {CODE} 的授予全部清掉後再跑 startup 同步，{expected} 應該要被重新授予。\
             沒有被授予＝`ensure_all_role_permissions` 裡那一行不見了或失效了（實際：{after:?}）"
        );
    }
}

/// 不是「發給大家」——除了裁定的兩個角色與 admin 類角色之外不應擴散。
///
/// ⚠️ 這裡刻意**不**斷言 holders 完全等於某個固定集合：`003_seed.sql` 可能已經
/// 授予 admin／SYSTEM_ADMIN，且日後使用者可能再加人。本測試守的是
/// 「一般作業角色不該拿到簽章作廢權」這條線。
///
/// `#[serial]`：理由同上，與破壞性測試共用同一顆測試庫。
#[tokio::test]
#[serial]
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
