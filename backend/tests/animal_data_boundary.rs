//! 動物「資料邊界」（`AnimalTransferService::get_data_boundary`）。
//!
//! 動物被轉移後，部分角色只能看到轉移完成之後建立的紀錄；不受限的角色回傳 `None`
//! （看得到全部），受限的角色回傳該動物最近一筆已完成轉移的 `completed_at`。
//!
//! 這裡驗證：
//! - 管理員（正式資料庫的代碼是小寫 `admin`）、試驗工作人員、獸醫、IACUC 工作人員／主席
//!   不受限（回傳 `None`）；
//! - PI、委託人（CLIENT）等其他角色，在有已完成轉移時拿到邊界時間；
//! - 邊界取「最近一筆已完成」的轉移，未完成（申請中、已駁回）的轉移不算。
//!
//! 整合測試只能在獨立丟棄 DB 跑（`TEST_DATABASE_URL`），見 `common/test_db.rs`。

use chrono::{DateTime, TimeZone, Utc};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::AnimalTransferService;
use erp_backend::SYSTEM_USER_ID;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    let pool = test_db::connect_disposable(5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

fn roles(codes: &[&str]) -> Vec<String> {
    codes.iter().map(|c| c.to_string()).collect()
}

async fn seed_animal(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-06-01', $3, $4)"#,
    )
    .bind(id)
    .bind(format!("DB{}", &id.simple().to_string()[..6]))
    .bind("ITEST-BOUNDARY")
    .bind(SYSTEM_USER_ID)
    .execute(pool)
    .await
    .expect("seed animal");
    id
}

/// 寫入一筆轉移。`completed_at` 只有 `completed` 狀態才該有值。
async fn seed_transfer(
    pool: &PgPool,
    animal_id: Uuid,
    status: &str,
    completed_at: Option<DateTime<Utc>>,
) {
    sqlx::query(
        r#"INSERT INTO animal_transfers
               (animal_id, from_iacuc_no, to_iacuc_no, status, initiated_by, reason, completed_at)
           VALUES ($1, 'ITEST-FROM', 'ITEST-TO', $2::animal_transfer_status, $3, '資料邊界測試', $4)"#,
    )
    .bind(animal_id)
    .bind(status)
    .bind(SYSTEM_USER_ID)
    .bind(completed_at)
    .execute(pool)
    .await
    .expect("seed transfer");
}

async fn boundary_for(pool: &PgPool, animal_id: Uuid, codes: &[&str]) -> Option<DateTime<Utc>> {
    AnimalTransferService::get_data_boundary(pool, animal_id, SYSTEM_USER_ID, &roles(codes))
        .await
        .expect("get_data_boundary")
        .boundary
}

fn t(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, 0, 0)
        .single()
        .expect("valid timestamp")
}

/// 有已完成轉移時：不受限角色回傳 `None`，其餘角色回傳邊界時間。
#[tokio::test]
#[serial]
async fn exempt_roles_see_everything_and_others_get_the_boundary() {
    let pool = setup_pool().await;
    let animal = seed_animal(&pool).await;
    let completed = t(2026, 10, 15, 8);
    seed_transfer(&pool, animal, "completed", Some(completed)).await;

    // 不受限：看得到轉移前的紀錄
    for code in [
        "admin", // 正式資料庫的管理員代碼（小寫）
        "ADMIN",
        "SYSTEM_ADMIN",
        "EXPERIMENT_STAFF",
        "VET",
        "IACUC_STAFF",
        "IACUC_CHAIR",
    ] {
        assert_eq!(
            boundary_for(&pool, animal, &[code]).await,
            None,
            "{} 不應受資料邊界限制",
            code
        );
    }

    // 受限：只看得到轉移完成之後的紀錄
    for code in ["PI", "CLIENT", "DIRECTOR", "REVIEWER"] {
        assert_eq!(
            boundary_for(&pool, animal, &[code]).await,
            Some(completed),
            "{} 應受資料邊界限制",
            code
        );
    }
}

/// 同時持有受限與不受限角色：不受限優先。
#[tokio::test]
#[serial]
async fn user_with_any_exempt_role_is_not_restricted() {
    let pool = setup_pool().await;
    let animal = seed_animal(&pool).await;
    seed_transfer(&pool, animal, "completed", Some(t(2026, 10, 15, 8))).await;

    assert_eq!(
        boundary_for(&pool, animal, &["PI", "EXPERIMENT_STAFF"]).await,
        None
    );
}

/// 邊界取最近一筆已完成的轉移；未完成（申請中、已駁回）的不算。
#[tokio::test]
#[serial]
async fn boundary_is_the_latest_completed_transfer_only() {
    let pool = setup_pool().await;
    let animal = seed_animal(&pool).await;
    seed_transfer(&pool, animal, "completed", Some(t(2026, 9, 1, 8))).await;
    let latest = t(2026, 10, 15, 8);
    seed_transfer(&pool, animal, "completed", Some(latest)).await;
    seed_transfer(&pool, animal, "pending", None).await;
    seed_transfer(&pool, animal, "rejected", None).await;

    assert_eq!(boundary_for(&pool, animal, &["PI"]).await, Some(latest));
    assert_eq!(boundary_for(&pool, animal, &["CLIENT"]).await, Some(latest));
}

/// 從未轉移過的動物沒有邊界，任何角色都看得到全部。
#[tokio::test]
#[serial]
async fn animal_without_completed_transfer_has_no_boundary() {
    let pool = setup_pool().await;
    let animal = seed_animal(&pool).await;
    seed_transfer(&pool, animal, "pending", None).await;

    assert_eq!(boundary_for(&pool, animal, &["PI"]).await, None);
    assert_eq!(boundary_for(&pool, animal, &["CLIENT"]).await, None);
}
