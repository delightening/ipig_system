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

    let narrow = repo::list_users_with_permission(pool, NARROW)
        .await
        .expect("list narrow");
    let both = repo::list_users_with_any_permission(pool, BOTH)
        .await
        .expect("list both");

    // 前提：兩個來源的人數必須不同，否則這支測試量不到任何東西。
    // 若哪天權限授予收斂成一致，這裡會要求重新檢視而不是靜默通過。
    assert!(
        both.len() > narrow.len(),
        "前提消失：`equipment.manage` 的持有者已全部涵蓋於 `{NARROW}`，\n\
         本測試分辨不出「只查一個」與「兩個都查」，請重新檢視或刪除。\n\
         narrow={} both={}",
        narrow.len(),
        both.len()
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
