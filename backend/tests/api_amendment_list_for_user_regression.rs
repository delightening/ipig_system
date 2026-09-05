//! 迴歸：`GET /amendments` 對**非** staff 使用者不得整支炸掉。
//!
//! ## 這支測試防的 bug
//!
//! `AmendmentService::list_for_user` 是執行期 `query_as::<_, T>`，卻沿用了
//! `sqlx::query_as!` 巨集才有的型別標註語法：
//!
//! ```sql
//! SELECT a.status as "status: AmendmentStatus"
//! ```
//!
//! 在**巨集**裡那是型別標註；在**執行期**它就只是一個欄位別名。實測
//! （2026-08-26，測試庫）那段 SQL 產生的欄位名真的叫 `status: AmendmentStatus`，
//! 於是 `FromRow` 去找 `status` / `amendment_type` 一律落空、整個請求失敗。
//!
//! 影響面刻意很窄，所以沒被發現：`handlers/amendment.rs:118-122` 只在
//! **沒有 `aup.protocol.view_all`** 時才走這條路。內部人員（IACUC 執行秘書、admin）
//! 走 `list()`，一切正常；計畫方（PI 本人、試驗工作人員）才會撞到。
//!
//! ## 為什麼是這個形狀
//!
//! 斷言「回得出資料」而不只是「不 panic」——空結果集**不會**觸發 `FromRow`，
//! 拿空清單當通過等於什麼都沒測。所以先塞一筆 amendment 進去。

mod common;
use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::models::{AmendmentQuery, AmendmentStatus};
use erp_backend::services::AmendmentService;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 建一位使用者、一個計畫、一筆已送審的變更申請，並把使用者掛進該計畫成員。
///
/// 回傳 (user_id, amendment_id)。
async fn seed_amendment_visible_to_member(pool: &PgPool) -> (Uuid, Uuid) {
    let suffix = Uuid::new_v4().simple().to_string();
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(user_id)
    .bind(format!("amd-{}@example.com", &suffix[..8]))
    .bind(format!("變更申請成員-{}", &suffix[..4]))
    .execute(pool)
    .await
    .expect("seed user");

    let protocol_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by) \
         VALUES ($1, $2, $3, $4, $4)",
    )
    .bind(protocol_id)
    .bind(format!("AMD-{}", &suffix[..8]))
    .bind("變更申請列表迴歸用計畫")
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed protocol");

    // role_in_protocol 為 NOT NULL；'PI' 是既有測試一致採用的值
    // （notification_dispatch_routing.rs:184 等）。
    sqlx::query(
        "INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol) VALUES ($1, $2, 'PI')",
    )
    .bind(user_id)
    .bind(protocol_id)
    .execute(pool)
    .await
    .expect("link member to protocol");

    let amendment_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO amendments (id, protocol_id, amendment_no, title, status, created_by, submitted_at) \
         VALUES ($1, $2, $3, $4, 'SUBMITTED', $5, NOW())",
    )
    .bind(amendment_id)
    .bind(protocol_id)
    .bind(format!("AMD-{}-01", &suffix[..8]))
    .bind("迴歸用變更申請")
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed amendment");

    (user_id, amendment_id)
}

/// 無任何權限的檢視者。
///
/// 本檔測的是「PI / 非成員看變更申請列表」，那正是**沒有** `aup.protocol.change_status`
/// 的情境——委員會審查中的那幾筆對他們不該有 `pending_owner`。
/// 用有權限的檢視者會讓這支迴歸測試偏離它原本要守的東西。
fn viewer_without_committee_access() -> CurrentUser {
    CurrentUser {
        id: Uuid::new_v4(),
        email: "no-perm@example.com".into(),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

#[tokio::test]
#[serial]
async fn list_for_user_decodes_status_and_type() {
    let app = TestApp::spawn().await;
    let (user_id, amendment_id) = seed_amendment_visible_to_member(&app.db_pool).await;

    let listed = AmendmentService::list_for_user(
        &app.db_pool,
        &AmendmentQuery {
            protocol_id: None,
            status: None,
            amendment_type: None,
        },
        user_id,
        &viewer_without_committee_access(),
    )
    .await
    .expect(
        "list_for_user 必須解得出列。失敗多半是 SELECT 又用了 `as \"col: Type\"`——\
         那是 query_as! 巨集語法，在執行期查詢裡會變成欄位別名，FromRow 就找不到欄位了",
    );

    let found = listed
        .iter()
        .find(|a| a.id == amendment_id)
        .expect("剛建立的變更申請必須出現在成員可見清單裡");

    assert_eq!(
        found.status,
        AmendmentStatus::Submitted,
        "status 必須解成列舉值，不是落空或退化成預設"
    );
}

/// 沒有 staff 權限的使用者不得看到別人計畫的變更申請。
///
/// 與上一項一起看才完整：修 FromRow 的同時不能順手把可見範圍放寬。
#[tokio::test]
#[serial]
async fn list_for_user_still_scopes_to_member_protocols() {
    let app = TestApp::spawn().await;
    let (_owner, amendment_id) = seed_amendment_visible_to_member(&app.db_pool).await;

    let outsider = Uuid::new_v4();
    let suffix = Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(outsider)
    .bind(format!("amd-out-{}@example.com", &suffix[..8]))
    .bind(format!("非成員-{}", &suffix[..4]))
    .execute(&app.db_pool)
    .await
    .expect("seed outsider");

    let listed = AmendmentService::list_for_user(
        &app.db_pool,
        &AmendmentQuery {
            protocol_id: None,
            status: None,
            amendment_type: None,
        },
        outsider,
        &viewer_without_committee_access(),
    )
    .await
    .expect("list_for_user");

    assert!(
        !listed.iter().any(|a| a.id == amendment_id),
        "非計畫成員不得看到該計畫的變更申請"
    );
}
