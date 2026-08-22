//! 通知／待辦分家（migration 143 + `kind` 欄位）的行為契約。
//!
//! 核心規則：**待辦只能由系統偵測動作完成後自動消失，使用者不可手動略過**
//! （使用者 2026-08-07 裁定）。
//!
//! 這條規則的實作重點不在前端不給按鈕 —— 那擋不住直接呼叫 API ——
//! 而在後端的 `mark_as_read` / `mark_all_as_read` 必須排除未完成待辦。
//! 本檔鎖住那個排除。
//!
//! 背景：2026-08-07 查 prod 發現既有置頂待辦的 `is_read` **全部是 true**，
//! 代表使用者早就按過「全部已讀」；當時前端用 `priority` 而非 `is_read` 決定顯示，
//! 才沒有被掩蓋掉。分家後若不擋，一按就把待處理清單清光。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::models::NotificationQuery;
use erp_backend::services::NotificationService;

#[path = "common/test_db.rs"]
mod test_db;

/// **fail-closed**：見 `tests/common/test_db.rs`。
///
/// 只讀 `TEST_DATABASE_URL`（不 fallback 到 `DATABASE_URL`，開發機那條指向 prod），
/// 並在 migration 與任何寫入之前驗證目標資料庫**本身**是丟棄用測試庫。
///
/// 本檔早期版本用「DSN 名稱是否含 test」的字串啟發式——那擋不住「名字剛好含 test 的
/// 正式庫」，只是當時 CI 只設 `DATABASE_URL`、不接受 fallback 就會紅，才用的折衷。
/// #37 把 `TEST_DATABASE_URL` 補進 ci.yml 並抽出這支共用護欄後，折衷不再需要。
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
           VALUES ($1, $2, 'fake', 'kind split test', true, false)"#,
    )
    .bind(id)
    .bind(format!("kind-split-{}@test.local", &id.to_string()[..8]))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

/// 直接插一列，指定 kind 與 priority。
async fn seed(pool: &PgPool, user: Uuid, kind: &str, priority: i16) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
             (id, user_id, type, title, priority, kind, is_read)
           VALUES ($1, $2, 'system_alert'::notification_type, 'fixture', $3, $4, false)"#,
    )
    .bind(id)
    .bind(user)
    .bind(priority)
    .bind(kind)
    .execute(pool)
    .await
    .expect("seed notification");
    id
}

async fn row_of(pool: &PgPool, id: Uuid) -> (bool, i16, String) {
    sqlx::query_as("SELECT is_read, priority, kind FROM notifications WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("read notification row")
}

/// 「全部已讀」不得清掉未完成待辦 —— 那等於開一條手動略過的後門。
#[tokio::test]
async fn mark_all_read_must_not_clear_pending_action() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    let info = seed(&pool, user, "info", 0).await;
    let action = seed(&pool, user, "action", 1).await;

    let svc = NotificationService::new(pool.clone());
    svc.mark_all_as_read(user).await.expect("mark all read");

    assert!(row_of(&pool, info).await.0, "一般通知應被標為已讀");
    assert!(
        !row_of(&pool, action).await.0,
        "未完成待辦不得被『全部已讀』標掉 —— 待辦只能由系統偵測完成後消失"
    );
}

/// 指定 id 標已讀同樣不得清掉待辦：前端不給按鈕擋不住直接呼叫 API。
#[tokio::test]
async fn mark_as_read_by_id_must_not_clear_pending_action() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    let action = seed(&pool, user, "action", 1).await;

    let svc = NotificationService::new(pool.clone());
    svc.mark_as_read(user, &[action])
        .await
        .expect("mark as read");

    assert!(
        !row_of(&pool, action).await.0,
        "直接指定 id 也不得把未完成待辦標為已讀"
    );
}

/// **已完成**的待辦（priority 已降 0）不再受保護 —— 它已離開待處理清單，
/// 只是留在鈴鐺歷史裡，理應能被標為已讀。
#[tokio::test]
async fn mark_all_read_does_clear_completed_action() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    let done = seed(&pool, user, "action", 0).await;

    let svc = NotificationService::new(pool.clone());
    svc.mark_all_as_read(user).await.expect("mark all read");

    assert!(
        row_of(&pool, done).await.0,
        "已完成的待辦（priority=0）屬一般歷史通知，應可標為已讀"
    );
}

/// 兩個計數互斥：同一件事不會在鈴鐺與驚嘆號各被數一次。
#[tokio::test]
async fn the_two_counters_are_disjoint() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    seed(&pool, user, "info", 0).await;
    seed(&pool, user, "info", 0).await;
    seed(&pool, user, "action", 1).await;

    let svc = NotificationService::new(pool.clone());
    let unread = svc.get_unread_count(user).await.expect("unread count");
    let pending = svc
        .get_action_required_count(user)
        .await
        .expect("action count");

    assert_eq!(unread, 2, "鈴鐺只數未讀『通知』，不含待辦");
    assert_eq!(pending, 1, "驚嘆號只數未完成待辦");
}

/// 待辦計數**不看 `is_read`**：待辦的存否由業務狀態決定，不由使用者看過與否決定。
#[tokio::test]
async fn action_count_ignores_is_read() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    let action = seed(&pool, user, "action", 1).await;
    sqlx::query("UPDATE notifications SET is_read = true, read_at = NOW() WHERE id = $1")
        .bind(action)
        .execute(&pool)
        .await
        .expect("force is_read");

    let svc = NotificationService::new(pool.clone());
    assert_eq!(
        svc.get_action_required_count(user)
            .await
            .expect("action count"),
        1,
        "即使被標為已讀（例如舊資料），未完成待辦仍須計入待處理"
    );
}

/// `?entry=` 篩選讓兩個入口各取所需，且**清單與紅點必須算出同一批**。
///
/// 特別鎖住：鈴鐺入口**不等於** `kind='info'` —— 已完成的待辦
/// （`kind='action'` 但 `priority=0`）也要留在鈴鐺歷史裡，
/// 那正是「我當初處理過哪些事」。
#[tokio::test]
async fn entry_filter_separates_the_two_entries() {
    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    seed(&pool, user, "info", 0).await; // 一般通知
    seed(&pool, user, "action", 0).await; // 已完成的待辦
    seed(&pool, user, "action", 1).await; // 未完成的待辦

    let svc = NotificationService::new(pool.clone());
    let query = |entry: Option<&str>| NotificationQuery {
        is_read: None,
        notification_type: None,
        entry: entry.map(str::to_string),
    };

    let todo = svc
        .list_notifications(user, &query(Some("todo")), 1, 20)
        .await
        .expect("list todo");
    assert_eq!(todo.total, 1, "待處理只收未完成待辦");
    assert_eq!(todo.data[0].kind, "action");
    assert_eq!(todo.data[0].priority, 1);

    let bell = svc
        .list_notifications(user, &query(Some("bell")), 1, 20)
        .await
        .expect("list bell");
    assert_eq!(bell.total, 2, "鈴鐺收一般通知 + 已完成的待辦");
    assert!(
        bell.data.iter().any(|n| n.kind == "action"),
        "已完成的待辦必須留在鈴鐺歷史裡；鈴鐺入口不可實作成 kind='info'"
    );
    assert!(
        bell.data.iter().all(|n| n.priority == 0),
        "鈴鐺不得混入未完成待辦，否則同一件事兩個入口都出現"
    );

    // 清單筆數與紅點計數必須一致（分家後最容易漂移的地方）
    assert_eq!(
        todo.total,
        svc.get_action_required_count(user)
            .await
            .expect("action count"),
        "待處理清單筆數與驚嘆號紅點必須算出同一批"
    );

    // 未帶 entry ＝ 不篩選，維持舊前端相容（部署期間新舊前端並存）
    let all = svc
        .list_notifications(user, &query(None), 1, 20)
        .await
        .expect("list all");
    assert_eq!(all.total, 3, "未帶 entry 應回全部，不可預設篩選");

    // 打錯字視同未帶，不回空清單（空清單會讓使用者以為待辦不見了）
    let typo = svc
        .list_notifications(user, &query(Some("bel")), 1, 20)
        .await
        .expect("list typo");
    assert_eq!(typo.total, 3, "無法辨識的 entry 應退回不篩選，不可回空");
}

/// 建立端：`kind` 由 `priority` 決定，集中在唯一的 INSERT 點綁定。
/// 若兩者脫鉤（例如 priority=1 卻標成 info），該列會同時不出現在兩個入口。
#[tokio::test]
async fn create_binds_kind_to_priority() {
    use erp_backend::models::{CreateNotificationRequest, NotificationType};

    let pool = setup_pool().await;
    let user = seed_user(&pool).await;
    let svc = NotificationService::new(pool.clone());

    let plain = svc
        .create_notification(CreateNotificationRequest {
            user_id: user,
            notification_type: NotificationType::SystemAlert,
            title: "一般".into(),
            content: None,
            related_entity_type: None,
            related_entity_id: None,
        })
        .await
        .expect("create plain");
    assert_eq!(plain.kind, "info");
    assert_eq!(plain.priority, 0);

    let pinned = svc
        .create_pinned_notification(CreateNotificationRequest {
            user_id: user,
            notification_type: NotificationType::SystemAlert,
            title: "待辦".into(),
            content: None,
            related_entity_type: None,
            related_entity_id: None,
        })
        .await
        .expect("create pinned");
    assert_eq!(pinned.kind, "action");
    assert_eq!(pinned.priority, 1);
}
