//! 回歸測試：IACUC 新送審通知（`scheduler.rs::check_iacuc_new_submissions`、
//! `handlers/system_settings.rs::send_iacuc_test_notification`）查詢 SUBMITTED 計畫書
//! 時曾寫 `u.name`，但 `users` 表沒有 `name` 欄位（只有 `display_name`），導致這條查詢
//! 無論有無資料都必定 42703（column does not exist）。兩處已改為 `u.display_name`，
//! 此測試直接驗證該共用 SQL 形狀本身是合法且能取回正確姓名。

use sqlx::PgPool;
use uuid::Uuid;

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

async fn seed_user(pool: &PgPool, display_name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("pi-join-{}@test.local", &id.to_string()[..8]))
    .bind(display_name)
    .execute(pool)
    .await
    .expect("seed user");
    id
}

async fn seed_submitted_protocol(pool: &PgPool, pi_user_id: Uuid, created_by: Uuid) -> String {
    let protocol_no = format!("TEST-JOIN-{}", &Uuid::new_v4().to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, 'PI display_name join regression', 'SUBMITTED', $3, $4)"#,
    )
    .bind(Uuid::new_v4())
    .bind(&protocol_no)
    .bind(pi_user_id)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("seed submitted protocol");
    protocol_no
}

#[tokio::test]
async fn submitted_protocol_pi_name_join_does_not_500() {
    let pool = setup_pool().await;
    let pi_id = seed_user(&pool, "測試 PI 顯示名稱").await;
    let protocol_no = seed_submitted_protocol(&pool, pi_id, pi_id).await;

    // 與 scheduler.rs::check_iacuc_new_submissions / system_settings.rs 的
    // send_iacuc_test_notification 修復後完全相同的 SQL 形狀。
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT p.protocol_no, p.title, u.display_name
        FROM protocols p
        LEFT JOIN users u ON u.id = p.pi_user_id
        WHERE p.status = 'SUBMITTED' AND p.protocol_no = $1
        "#,
    )
    .bind(&protocol_no)
    .fetch_all(&pool)
    .await
    .expect("u.display_name join regression (42703) should not occur");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2.as_deref(), Some("測試 PI 顯示名稱"));
}
