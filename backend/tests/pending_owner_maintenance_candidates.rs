//! 維修「待處理」關的候選名單必須涵蓋兩個權限，不是只有其中一個。
//!
//! 會把維修單推離 `pending` 的三條寫入路徑——`create_maintenance_record`、
//! `update_maintenance_record`、`delete_maintenance_record`——判準一致，
//! 都是 `equipment.maintenance.manage` **或** `equipment.manage`。
//!
//! 我原本只查前者（CodeRabbit 於 #30 指出）。只有 `equipment.manage` 的人
//! **處理得了這筆維修單，卻不會出現在候選名單裡**。
//!
//! 方向是**漏列**：比誤列輕（不會叫人去點一個必定 403 的按鈕），但一樣是
//! 名單與守衛分岔——使用者去催了名單上的人，而真正能動手的人沒被通知到。

mod common;
use common::TestApp;
use erp_backend::repositories::pending_owner as repo;
use erp_backend::services::pending_owner::resolve_for_maintenance;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

const NARROW: &str = "equipment.maintenance.manage";
const BOTH: &[&str] = &["equipment.maintenance.manage", "equipment.manage"];

async fn seed_user_with_role(pool: &PgPool, label: &str, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(id)
    .bind(format!("{label}-{}@example.com", &suffix[..8]))
    .bind(format!("{label}-{}", &suffix[..6]))
    .execute(pool)
    .await
    .expect("seed user");

    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("grant role");

    id
}

async fn seed_equipment(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO equipment (id, name, is_active, status) VALUES ($1, $2, true, 'active')",
    )
    .bind(id)
    .bind(format!(
        "維修測試設備-{}",
        &Uuid::new_v4().simple().to_string()[..6]
    ))
    .execute(pool)
    .await
    .expect("seed equipment");
    id
}

/// 建一筆「待處理」的維修單。`created_by` 用系統使用者——這一關**沒有職務分離**
/// （登錄者本來就常是動手修的人），所以候選名單不會因為建立者而少一人。
async fn seed_pending_maintenance(pool: &PgPool, equipment_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO equipment_maintenance_records \
         (id, equipment_id, maintenance_type, status, reported_at, created_by) \
         VALUES ($1, $2, 'repair', 'pending', CURRENT_DATE, $3)",
    )
    .bind(id)
    .bind(equipment_id)
    .bind(erp_backend::SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed maintenance record");
    id
}

#[tokio::test]
#[serial]
async fn pending_stage_lists_equipment_manage_holders_too() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;

    // 自己建出「只有 equipment.manage、沒有 equipment.maintenance.manage」的人，
    // 不依賴種子資料裡剛好存在這種人（CodeRabbit 於 #30 指出）。
    // ADMIN_STAFF 正是這個差：授予表給它 equipment.manage 但不給 maintenance.manage。
    let manage_only = seed_user_with_role(pool, "maint-manage-only", "ADMIN_STAFF").await;

    let narrow = repo::list_users_with_permission(pool, NARROW)
        .await
        .expect("list narrow");
    let both = repo::list_users_with_any_permission(pool, BOTH)
        .await
        .expect("list both");

    // 前提現在由上面那位使用者保證，但仍顯式驗一次——因為它依賴的是
    // **授予表的內容**（ADMIN_STAFF 的權限組合），那是資料不是程式碼，會變。
    assert!(
        both.iter().any(|(id, _)| *id == manage_only),
        "只有 `equipment.manage` 的人必須出現在兩權限來源裡"
    );
    assert!(
        !narrow.iter().any(|(id, _)| *id == manage_only),
        "前提消失：ADMIN_STAFF 現在也有 `{NARROW}` 了，本測試分辨不出\n\
         「只查一個」與「兩個都查」，請改用別的角色或刪除本測試。"
    );

    let equipment = seed_equipment(pool).await;
    let record = seed_pending_maintenance(pool, equipment).await;

    let resolved = resolve_for_maintenance(pool, &[record])
        .await
        .expect("resolve maintenance");
    let owner = resolved.get(&record).expect("待處理必須有 pending_owner");
    let total = owner.candidates.len() + owner.overflow as usize;

    assert_eq!(
        total,
        both.len(),
        "待處理關的候選名單應涵蓋 `{NARROW}` 與 `equipment.manage` 兩者。\n\
         只查前者的話總數會是 {}（本測試就是釘這個差）。\n\
         用總人數（candidates + overflow）比對，因為名單只列前 3 個。\n\
         實際 {total} 人",
        narrow.len()
    );
}
