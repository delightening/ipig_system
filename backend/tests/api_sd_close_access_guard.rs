//! SD 能不能通過 change-status handler 的物件層授權（R98-1 的第二道關卡）。
//!
//! # 為什麼需要這支測試
//!
//! `handlers/protocol/crud.rs` 的 change_status 有三道關卡：
//!
//! ```text
//!   1. require_permission!(close_own)              ← 本 PR 補的是這道
//!   2. access::require_protocol_related_access(..)  ← 本檔驗這道
//!   3. ProtocolService::change_status(..)           ← status.rs 的擁有人檢查
//! ```
//!
//! CodeRabbit 於 PR #23 指出：第 2 道的 `has_any_protocol_role` 查詢只看
//! PI／user_protocols／review_assignments／vet_review_assignments，
//! **沒有 `study_director_user_id`**，因此「只透過 SD 欄位關聯」的人會 403。
//!
//! 那個觀察就查詢本身而言是對的——但 `require_protocol_related_access` 的
//! 第一行是 `has_protocol_view_all` 短路，而 SD 必為 `EXPERIMENT_STAFF`，
//! 該角色實測**具有 `aup.protocol.view_all`**，所以實際上走不到那個查詢。
//!
//! 本檔把兩種情況都測出來，讓「到底會不會 403」有實據而不是推論。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;

async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("sdacc-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("sdacc-{role_code}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    id
}

/// 建一份已核准計畫，SD 與 PI 是不同人，且 SD **只**透過
/// `study_director_user_id` 關聯（不進 user_protocols、不是審查委員／獸醫）。
/// 這正是 CodeRabbit 描述的形狀，也是實測正式庫 34 份中 33 份的形狀。
async fn seed_approved_protocol(app: &TestApp, sd: Uuid) -> Uuid {
    let pi = seed_user(app, "PI").await;
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by, study_director_user_id)
           VALUES ($1, $2, $3, 'APPROVED'::protocol_status, $4, $4, $5)"#,
    )
    .bind(id)
    .bind(format!("SDACC-{}", &Uuid::new_v4().to_string()[..8]))
    .bind("SD 存取測試計畫")
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    id
}

fn cu(id: Uuid, roles: &[&str], permissions: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "sd@example.com".to_string(),
        roles: roles.iter().map(|r| r.to_string()).collect(),
        permissions: permissions.iter().map(|p| p.to_string()).collect(),
        jti: Uuid::new_v4().to_string(),
        exp: 0,
        impersonated_by: None,
    }
}

/// 🔴 **真實情境**：SD 帶著 EXPERIMENT_STAFF 實際擁有的權限，應該通過。
///
/// 實測正式庫：`EXPERIMENT_STAFF` 的權限含 `aup.protocol.view_all`，
/// 所以 `require_protocol_related_access` 的第一道短路就放行，
/// 走不到那個缺 SD 的 UNION 查詢。
#[tokio::test]
#[serial]
async fn sd_with_real_permissions_passes_access_guard() {
    let app = TestApp::spawn().await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_approved_protocol(&app, sd).await;

    let user = cu(
        sd,
        &["EXPERIMENT_STAFF"],
        &["aup.protocol.view_all", "aup.protocol.close_own"],
    );
    access::require_protocol_related_access(&app.db_pool, &user, protocol_id)
        .await
        .expect("SD 具 view_all，物件層授權應放行");
}

/// 🔴 **CodeRabbit 描述的情況**：拿掉 view_all 短路之後會怎樣。
///
/// 這支測試把「只透過 SD 欄位關聯」的人放在沒有 view_all 的狀態下，
/// 直接打在那個 UNION 查詢上。它的結果就是那條 Major 成不成立的實據。
///
/// ⚠️ 這不是假想情境：`aup.protocol.view_all` 是**角色權限配置**，
/// 隨時可能被調整。若哪天 EXPERIMENT_STAFF 的 view_all 被收回，
/// SD 就會立刻掉進這條路徑——而那時症狀是「SD 按結案得到 403」，
/// 很難聯想到是幾個月前改權限配置造成的。
#[tokio::test]
#[serial]
async fn sd_without_view_all_hits_the_membership_query() {
    let app = TestApp::spawn().await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_approved_protocol(&app, sd).await;

    // 刻意不給 view_all，只給結案權限
    let user = cu(sd, &[], &["aup.protocol.close_own"]);
    let result = access::require_protocol_related_access(&app.db_pool, &user, protocol_id).await;

    assert!(
        result.is_ok(),
        "只透過 study_director_user_id 關聯的 SD 應被視為與計畫有關聯，實得：{:?}",
        result.err()
    );
}

/// 對照組：與計畫完全無關的人，兩種情況都該被擋。
///
/// 有這支才能證明上面那支不是「因為守衛整個失效」而通過。
#[tokio::test]
#[serial]
async fn unrelated_user_is_still_blocked() {
    let app = TestApp::spawn().await;
    let sd = seed_user(&app, "EXPERIMENT_STAFF").await;
    let outsider = seed_user(&app, "EXPERIMENT_STAFF").await;
    let protocol_id = seed_approved_protocol(&app, sd).await;

    let user = cu(outsider, &[], &["aup.protocol.close_own"]);
    let result = access::require_protocol_related_access(&app.db_pool, &user, protocol_id).await;
    assert!(result.is_err(), "與計畫無關的人不該通過物件層授權");
}
