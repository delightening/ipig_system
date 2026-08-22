//! 回歸測試：`NotificationService::list_notifications` 對 `notification_type` 篩選
//! 曾把 native enum `notification_type` 綁成純文字（無 cast）比較 `type = $N`，
//! Postgres 對 `notification_type = text` 無對應運算子（42883）而 500。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::models::NotificationQuery;
use erp_backend::services::NotificationService;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'notification type filter test', true, false)"#,
    )
    .bind(id)
    .bind(format!("notif-type-filter-{}@test.local", &id.to_string()[..8]))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

async fn seed_notification(pool: &PgPool, user_id: Uuid, notification_type: &str) {
    sqlx::query(
        r#"INSERT INTO notifications (user_id, type, title)
           VALUES ($1, $2::notification_type, 'test notification')"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .execute(pool)
    .await
    .expect("seed notification");
}

#[tokio::test]
async fn list_notifications_with_type_filter_does_not_500() {
    let pool = setup_pool().await;
    let user_id = seed_user(&pool).await;
    seed_notification(&pool, user_id, "system_alert").await;
    seed_notification(&pool, user_id, "low_stock").await;

    let service = NotificationService::new(pool.clone());
    let query = NotificationQuery {
        is_read: None,
        notification_type: Some("system_alert".to_string()),
        entry: None,
    };

    let result = service
        .list_notifications(user_id, &query, 1, 20)
        .await
        .expect("status filter should not error (42883 regression)");

    assert!(
        result.data.iter().all(|n| n.r#type == "system_alert"),
        "notification_type filter leaked non-matching rows: {:?}",
        result.data
    );
    assert!(
        result.data.iter().any(|n| n.r#type == "system_alert"),
        "expected the seeded system_alert notification in filtered results"
    );
}
