//! 計畫列表的待處理人查詢必須真的跑得起來。
//!
//! ## 這支測試防的 bug
//!
//! 初版 `resolve_protocol_intake` 寫了 `AND deleted_at IS NULL`，但 **`protocols` 表
//! 根本沒有 `deleted_at` 欄位**——軟刪除是把 `status` 設成 `'DELETED'`
//! （2026-08-26 實查 16 個 `protocol_status` 值）。
//!
//! 後果不是「少過濾幾筆」而是**整支查詢炸掉**（`42703 column "deleted_at" does not
//! exist`），`GET /protocols` 與 `GET /my-projects` 兩支 API 一起 500。
//!
//! `cargo check` 與 `cargo test --lib` 都抓不到：SQL 字串在編譯期不驗，
//! 而單元測試不連資料庫。只有真的對資料庫跑一次才看得到。

mod common;
use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::pending_owner;
use serial_test::serial;
use uuid::Uuid;

/// 無委員身分檢視權的檢視者。本檔測的是「受理關的候選名單」與軟刪除排除，
/// 與委員會姓名可見性無關，給空權限即可。
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
async fn protocol_intake_query_runs_against_the_real_schema() {
    let app = TestApp::spawn().await;

    let suffix = Uuid::new_v4().simple().to_string();
    let pi = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(pi)
    .bind(format!("intake-{}@example.com", &suffix[..8]))
    .bind(format!("送審人-{}", &suffix[..4]))
    .execute(&app.db_pool)
    .await
    .expect("seed PI");

    let protocol = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by, status, submitted_at) \
         VALUES ($1, $2, $3, $4, $4, 'SUBMITTED'::protocol_status, NOW())",
    )
    .bind(protocol)
    .bind(format!("INTAKE-{}", &suffix[..8]))
    .bind("行政受理測試計畫")
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("seed protocol");

    let owners = pending_owner::resolve_for_protocols(
        &app.db_pool,
        &[protocol],
        &viewer_without_committee_access(),
    )
    .await
    .expect(
        "解析器必須跑得起來。失敗多半是 SQL 引用了實際不存在的欄位——\
             protocols 的軟刪除靠 status='DELETED'，沒有 deleted_at",
    );

    let owner = owners
        .get(&protocol)
        .expect("SUBMITTED 的計畫在等行政受理，必須有待處理人");
    assert_eq!(owner.stage, "aup_protocol_intake");
}

/// 已軟刪除（`status='DELETED'`）的計畫不在等任何人。
///
/// 與上一項成對：拿掉 `deleted_at` 條件之後，排除軟刪除的責任落在狀態白名單上，
/// 這條確認白名單真的擋得住。
#[tokio::test]
#[serial]
async fn soft_deleted_protocols_are_excluded_by_the_status_whitelist() {
    let app = TestApp::spawn().await;

    let suffix = Uuid::new_v4().simple().to_string();
    let pi = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active, is_internal) \
         VALUES ($1, $2, $3, 'x', true, true)",
    )
    .bind(pi)
    .bind(format!("del-{}@example.com", &suffix[..8]))
    .bind(format!("已刪除計畫的 PI-{}", &suffix[..4]))
    .execute(&app.db_pool)
    .await
    .expect("seed PI");

    let protocol = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, pi_user_id, created_by, status, submitted_at) \
         VALUES ($1, $2, $3, $4, $4, 'DELETED'::protocol_status, NOW())",
    )
    .bind(protocol)
    .bind(format!("DEL-{}", &suffix[..8]))
    .bind("已刪除的計畫")
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("seed deleted protocol");

    let owners = pending_owner::resolve_for_protocols(
        &app.db_pool,
        &[protocol],
        &viewer_without_committee_access(),
    )
    .await
    .expect("resolve");
    assert!(
        !owners.contains_key(&protocol),
        "已刪除的計畫不該出現在待處理清單裡"
    );
}
