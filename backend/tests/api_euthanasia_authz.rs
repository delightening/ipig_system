//! 安樂死開單／執行／仲裁裁決三個 handler 的授權邊界回歸測試。
//!
//! 背景：2026-08-09「11 處 role 硬判改 permission code」把
//! `handlers/euthanasia.rs` 的 create_order / execute_order / decide_appeal
//! 從 `has_role()` 硬判改為 `has_permission()`。這三個 handler 在改動前完全沒有
//! 任何整合測試覆蓋（改動前就是如此），而改動本身讓 admin 從「完全不能通過」
//! 變成「一律可繞過」（`has_permission` 對 admin 一律回 true）——這是安全關鍵
//! 路徑上刻意核可的行為變更，須有測試釘住，避免日後被誤改回 / 誤改壞。
//!
//! 涵蓋：無權限角色 403、合法權限持有者成功、admin 繞過成功。不涵蓋完整臨床
//! 流程（pi_approve 的 PI 簽章鏈、appeal 的 email/notification），那些超出本次
//! 授權改動的範圍。

mod common;

use chrono::{Duration, Utc};
use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

const PASSWORD: &str = "Euthanasia$Pw1";

/// 建立一個「可登入」使用者並指派指定角色，回傳 (id, email)。
async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "euth-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password(PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("euthanasia {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    (id, email)
}

async fn admin_user_id(app: &TestApp) -> Uuid {
    let email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch admin user id")
}

/// 建一個已核准計畫 + 一隻掛在該計畫下的動物，回傳 animal_id。
async fn seed_protocol_and_animal(app: &TestApp, pi_user_id: Uuid) -> Uuid {
    let pid = Uuid::new_v4();
    let iacuc = format!("IACUC-EU-{}", &pid.to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, 'euthanasia authz test', 'APPROVED', $4, $4)"#,
    )
    .bind(pid)
    .bind(format!("P-EU-{}", &pid.to_string()[..8]))
    .bind(&iacuc)
    .bind(pi_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");

    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no, status, created_by)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3, 'in_experiment', $4)"#,
    )
    .bind(aid)
    .bind(format!("EU{}", &aid.to_string()[..6]))
    .bind(&iacuc)
    .bind(pi_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

/// 直接種一筆安樂死單據（略過 create_order/pi_approve 鏈，取得指定初始狀態）。
async fn seed_order(
    app: &TestApp,
    animal_id: Uuid,
    vet_user_id: Uuid,
    pi_user_id: Uuid,
    status: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let deadline = Utc::now() + Duration::hours(24);
    sqlx::query(
        r#"INSERT INTO euthanasia_orders (id, animal_id, vet_user_id, pi_user_id, status, reason, deadline_at)
           VALUES ($1, $2, $3, $4, $5::euthanasia_order_status, 'authz test order', $6)"#,
    )
    .bind(id)
    .bind(animal_id)
    .bind(vet_user_id)
    .bind(pi_user_id)
    .bind(status)
    .bind(deadline)
    .execute(&app.db_pool)
    .await
    .expect("insert euthanasia order");
    id
}

/// 直接種一筆暫緩申請，指定 chair_user_id（decide_appeal 的 service 層以
/// `chair_user_id` 綁定裁決者，非任意 CHAIR 皆可裁決）。
async fn seed_appeal(app: &TestApp, order_id: Uuid, pi_user_id: Uuid, chair_user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO euthanasia_appeals (id, order_id, pi_user_id, reason, chair_user_id)
           VALUES ($1, $2, $3, 'authz test appeal', $4)"#,
    )
    .bind(id)
    .bind(order_id)
    .bind(pi_user_id)
    .bind(chair_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert euthanasia appeal");
    id
}

// ── create_order ─────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn create_order_forbidden_without_permission() {
    let app = TestApp::spawn().await;
    let (pi_id, pi_email) = seed_login_user(&app, "pi-noperm", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let token = app.login(&pi_email, PASSWORD).await.expect("pi login");

    let res = app
        .auth_post(
            "/api/v1/euthanasia/orders",
            &serde_json::json!({ "animal_id": animal_id, "reason": "test" }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "PI 角色不具 animal.euthanasia.create，應被擋下"
    );
}

#[tokio::test]
#[serial]
async fn create_order_vet_succeeds() {
    let app = TestApp::spawn().await;
    let (pi_id, _) = seed_login_user(&app, "pi-owner", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let (vet_id, vet_email) = seed_login_user(&app, "vet", "VET").await;
    let token = app.login(&vet_email, PASSWORD).await.expect("vet login");

    let res = app
        .auth_post(
            "/api/v1/euthanasia/orders",
            &serde_json::json!({ "animal_id": animal_id, "reason": "test" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "VET 持有 animal.euthanasia.create，應可開單，實得 {}",
        res.status()
    );

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM euthanasia_orders WHERE animal_id = $1 AND vet_user_id = $2",
    )
    .bind(animal_id)
    .bind(vet_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count orders");
    assert_eq!(count, 1, "應寫入一筆以該獸醫為開單人的安樂死單");
}

#[tokio::test]
#[serial]
async fn create_order_admin_bypasses_without_vet_role() {
    let app = TestApp::spawn().await;
    let admin_id = admin_user_id(&app).await;
    let (pi_id, _) = seed_login_user(&app, "pi-owner2", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_post(
            "/api/v1/euthanasia/orders",
            &serde_json::json!({ "animal_id": animal_id, "reason": "admin bypass test" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "admin 不具 VET 角色，但 has_permission 對 admin 一律回 true，應可開單，實得 {}",
        res.status()
    );

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM euthanasia_orders WHERE animal_id = $1 AND vet_user_id = $2",
    )
    .bind(animal_id)
    .bind(admin_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count orders");
    assert_eq!(count, 1, "admin 繞過後應成功寫入一筆單據");
}

// ── execute_order ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn execute_order_forbidden_without_permission() {
    let app = TestApp::spawn().await;
    let (pi_id, pi_email) = seed_login_user(&app, "pi-exec-noperm", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let order_id = seed_order(&app, animal_id, pi_id, pi_id, "approved").await;
    let token = app.login(&pi_email, PASSWORD).await.expect("pi login");

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/orders/{order_id}/execute"),
            &serde_json::json!({ "password": PASSWORD }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "PI 角色不具 animal.euthanasia.execute，應被擋下"
    );

    let status: String =
        sqlx::query_scalar("SELECT status::text FROM euthanasia_orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch order status");
    assert_eq!(status, "approved", "被擋下後單據狀態應維持 approved");
}

#[tokio::test]
#[serial]
async fn execute_order_vet_succeeds() {
    let app = TestApp::spawn().await;
    let (pi_id, _) = seed_login_user(&app, "pi-exec-owner", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let (vet_id, vet_email) = seed_login_user(&app, "vet-exec", "VET").await;
    let order_id = seed_order(&app, animal_id, vet_id, pi_id, "approved").await;
    let token = app.login(&vet_email, PASSWORD).await.expect("vet login");

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/orders/{order_id}/execute"),
            &serde_json::json!({ "password": PASSWORD, "handwriting_svg": "<svg/>" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "VET 持有 animal.euthanasia.execute，應可執行，實得 {}",
        res.status()
    );

    let (status, executed_by): (String, Option<Uuid>) =
        sqlx::query_as("SELECT status::text, executed_by FROM euthanasia_orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch order after execute");
    assert_eq!(status, "executed", "執行成功後單據狀態應轉 executed");
    assert_eq!(executed_by, Some(vet_id), "執行者應記為該獸醫");

    let animal_status: String =
        sqlx::query_scalar("SELECT status::text FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch animal status");
    assert_eq!(animal_status, "euthanized", "動物狀態應轉為已安樂死");
}

#[tokio::test]
#[serial]
async fn execute_order_admin_bypasses_without_vet_or_staff_role() {
    let app = TestApp::spawn().await;
    let admin_id = admin_user_id(&app).await;
    let (pi_id, _) = seed_login_user(&app, "pi-exec-owner2", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let order_id = seed_order(&app, animal_id, admin_id, pi_id, "approved").await;
    let token = app.login_as_admin().await;
    let admin_password = std::env::var("ADMIN_INITIAL_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "iPig$ecure1".to_string());

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/orders/{order_id}/execute"),
            &serde_json::json!({ "password": admin_password, "handwriting_svg": "<svg/>" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "admin 不具 VET/EXPERIMENT_STAFF 角色，但 has_permission 對 admin 一律回 true，應可執行，實得 {}",
        res.status()
    );

    let status: String =
        sqlx::query_scalar("SELECT status::text FROM euthanasia_orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch order status");
    assert_eq!(status, "executed", "admin 繞過後應執行成功");
}

// ── decide_appeal ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn decide_appeal_forbidden_without_permission() {
    let app = TestApp::spawn().await;
    let (pi_id, pi_email) = seed_login_user(&app, "pi-decide-noperm", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let order_id = seed_order(&app, animal_id, pi_id, pi_id, "chair_arbitration").await;
    let (chair_id, _) = seed_login_user(&app, "chair-other", "IACUC_CHAIR").await;
    let appeal_id = seed_appeal(&app, order_id, pi_id, chair_id).await;
    let token = app.login(&pi_email, PASSWORD).await.expect("pi login");

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/appeals/{appeal_id}/decide"),
            &serde_json::json!({ "decision": "reject_appeal", "password": PASSWORD }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "PI 角色不具 animal.euthanasia.arbitrate，應被擋下"
    );
}

#[tokio::test]
#[serial]
async fn decide_appeal_chair_succeeds() {
    let app = TestApp::spawn().await;
    let (pi_id, _) = seed_login_user(&app, "pi-decide-owner", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let order_id = seed_order(&app, animal_id, pi_id, pi_id, "chair_arbitration").await;
    let (chair_id, chair_email) = seed_login_user(&app, "chair", "IACUC_CHAIR").await;
    let appeal_id = seed_appeal(&app, order_id, pi_id, chair_id).await;
    let token = app
        .login(&chair_email, PASSWORD)
        .await
        .expect("chair login");

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/appeals/{appeal_id}/decide"),
            &serde_json::json!({
                "decision": "reject_appeal",
                "password": PASSWORD,
                "handwriting_svg": "<svg/>"
            }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "被指定的 IACUC_CHAIR 持有 animal.euthanasia.arbitrate，應可裁決，實得 {}",
        res.status()
    );

    let chair_decision: Option<String> =
        sqlx::query_scalar("SELECT chair_decision FROM euthanasia_appeals WHERE id = $1")
            .bind(appeal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch appeal decision");
    assert_eq!(
        chair_decision.as_deref(),
        Some("reject_appeal"),
        "裁決結果應寫入 chair_decision"
    );

    let order_status: String =
        sqlx::query_scalar("SELECT status::text FROM euthanasia_orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch order status");
    assert_eq!(
        order_status, "approved",
        "駁回暫緩後單據應轉為 approved（可執行安樂死）"
    );
}

#[tokio::test]
#[serial]
async fn decide_appeal_admin_bypasses_without_chair_role() {
    let app = TestApp::spawn().await;
    let admin_id = admin_user_id(&app).await;
    let (pi_id, _) = seed_login_user(&app, "pi-decide-owner2", "PI").await;
    let animal_id = seed_protocol_and_animal(&app, pi_id).await;
    let order_id = seed_order(&app, animal_id, pi_id, pi_id, "chair_arbitration").await;
    // service 層以 chair_user_id 綁定裁決者，故繞過測試須把 appeal 指給 admin 自己。
    let appeal_id = seed_appeal(&app, order_id, pi_id, admin_id).await;
    let token = app.login_as_admin().await;
    let admin_password = std::env::var("ADMIN_INITIAL_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "iPig$ecure1".to_string());

    let res = app
        .auth_post(
            &format!("/api/v1/euthanasia/appeals/{appeal_id}/decide"),
            &serde_json::json!({
                "decision": "approve_appeal",
                "password": admin_password,
                "handwriting_svg": "<svg/>"
            }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "admin 不具 IACUC_CHAIR 角色，但 has_permission 對 admin 一律回 true，應可裁決，實得 {}",
        res.status()
    );

    let order_status: String =
        sqlx::query_scalar("SELECT status::text FROM euthanasia_orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch order status");
    assert_eq!(
        order_status, "cancelled",
        "admin 繞過並核准暫緩後單據應轉為 cancelled"
    );
}

// ── 待處理清單要看得見生效中的 PI 代理人（migration 010）───────────
//
// `lock_order_for_pi` 允許代理人核准/暫緩，但清單若仍只查 `eo.pi_user_id`，
// 代理人**永遠發現不了那張單**——而它有 24 小時期限、逾時由 `check_expired_orders`
// 自動核准。權利給了卻沒有行使的管道，等於沒給。
//
// 這支測試同時釘住三件事，少任何一件都可能悄悄壞掉：看得到（核准後）、
// 沒有把借位的 PI 帳號擠掉、撤銷後就看不到。

/// 外部 PI 計畫 + 指定 SD + 掛在該計畫下的動物，回傳 (protocol_id, animal_id)。
async fn seed_external_pi_protocol_and_animal(
    app: &TestApp,
    borrowed_pi_user_id: Uuid,
    sd_user_id: Uuid,
) -> (Uuid, Uuid) {
    let pid = Uuid::new_v4();
    let iacuc = format!("IACUC-EUD-{}", &pid.to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by,
              study_director_user_id, pi_is_external)
           VALUES ($1, $2, $3, 'euthanasia delegate visibility', 'APPROVED', $4, $4, $5, true)"#,
    )
    .bind(pid)
    .bind(format!("P-EUD-{}", &pid.to_string()[..8]))
    .bind(&iacuc)
    .bind(borrowed_pi_user_id)
    .bind(sd_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert external-PI protocol");

    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no, status, created_by)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3, 'in_experiment', $4)"#,
    )
    .bind(aid)
    .bind(format!("EUD{}", &aid.to_string()[..5]))
    .bind(&iacuc)
    .bind(borrowed_pi_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    (pid, aid)
}

#[tokio::test]
#[serial]
async fn pending_orders_visible_to_active_pi_delegate() {
    use erp_backend::middleware::{ActorContext, CurrentUser};
    use erp_backend::services::{EuthanasiaService, ProtocolService};

    let app = TestApp::spawn().await;
    let (borrowed_pi, _) = seed_login_user(&app, "borrowed-pi", "PI").await;
    let (sd, _) = seed_login_user(&app, "sd", "EXPERIMENT_STAFF").await;
    let (delegate, _) = seed_login_user(&app, "delegate", "PI").await;
    let (vet, _) = seed_login_user(&app, "vet", "VET").await;
    let (protocol_id, animal_id) =
        seed_external_pi_protocol_and_animal(&app, borrowed_pi, sd).await;
    let order_id = seed_order(&app, animal_id, vet, borrowed_pi, "pending_pi").await;

    let sd_actor = ActorContext::User(CurrentUser {
        id: sd,
        email: format!("{sd}@test.local"),
        roles: vec!["EXPERIMENT_STAFF".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    });

    async fn sees(app: &TestApp, user: Uuid, order_id: Uuid) -> bool {
        EuthanasiaService::get_pending_orders_for_pi(&app.db_pool, user)
            .await
            .expect("list pending orders")
            .iter()
            .any(|o| o.id == order_id)
    }

    assert!(
        !sees(&app, delegate, order_id).await,
        "尚未核准代理授權前，這個人不該看得到別人的待處理單"
    );

    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &sd_actor,
        protocol_id,
        delegate,
        None,
        None,
    )
    .await
    .expect("SD 核准代理人");

    assert!(
        sees(&app, delegate, order_id).await,
        "生效中代理人必須看得到待處理單——否則 24 小時期限只能眼睜睜等它逾時自動核准"
    );
    assert!(
        sees(&app, borrowed_pi, order_id).await,
        "代理人加進來不該把原本的 pi_user_id 擠掉"
    );

    ProtocolService::revoke_pi_delegate(&app.db_pool, &sd_actor, protocol_id, None)
        .await
        .expect("SD 撤銷代理人");

    assert!(
        !sees(&app, delegate, order_id).await,
        "撤銷後就不該再看得到——可見範圍必須跟 lock_order_for_pi 的授權判準一致"
    );
}

// ── 暫緩申請要留下代簽證據（migration 010）─────────────────────────
//
// `lock_order_for_pi` 開放代理人之後，`euthanasia_appeals.pi_user_id` 已不再保證
// 等於計畫 PI。而暫緩申請**不建立簽章**，借不到 `electronic_signatures.delegation_id`
// 那條證據鏈——少了 `euthanasia_appeals.delegation_id`，事後只能靠時間窗回推
// 「當時他是不是代理人」，而授權可撤銷可重發，那種回推不是可靠證據。
//
// 三支一組：代理人申請要標、本人申請不得亂標、授權撤銷後不得再寫進去。

async fn appeal_delegation_id(app: &TestApp, appeal_id: Uuid) -> Option<Uuid> {
    sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT delegation_id FROM euthanasia_appeals WHERE id = $1",
    )
    .bind(appeal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("read appeal")
}

fn user_actor_for(id: Uuid, role: &str) -> erp_backend::middleware::ActorContext {
    use erp_backend::middleware::{ActorContext, CurrentUser};
    ActorContext::User(CurrentUser {
        id,
        email: format!("{id}@test.local"),
        roles: vec![role.to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

#[tokio::test]
#[serial]
async fn appeal_by_delegate_records_delegation_evidence() {
    use erp_backend::models::CreateEuthanasiaAppealRequest;
    use erp_backend::services::{EuthanasiaService, ProtocolService};

    let app = TestApp::spawn().await;
    let (borrowed_pi, _) = seed_login_user(&app, "borrowed-pi", "PI").await;
    let (sd, _) = seed_login_user(&app, "sd", "EXPERIMENT_STAFF").await;
    let (delegate, _) = seed_login_user(&app, "delegate", "PI").await;
    let (vet, _) = seed_login_user(&app, "vet", "VET").await;
    let (protocol_id, animal_id) =
        seed_external_pi_protocol_and_animal(&app, borrowed_pi, sd).await;
    let order_id = seed_order(&app, animal_id, vet, borrowed_pi, "pending_pi").await;

    let delegation_id = ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &user_actor_for(sd, "EXPERIMENT_STAFF"),
        protocol_id,
        delegate,
        None,
        None,
    )
    .await
    .expect("SD 核准代理人")
    .id;

    let appeal = EuthanasiaService::pi_appeal(
        &app.db_pool,
        &user_actor_for(delegate, "PI"),
        order_id,
        &CreateEuthanasiaAppealRequest {
            reason: "代理人代為申請暫緩".to_string(),
            attachment_path: None,
            version: None,
        },
    )
    .await
    .expect("代理人應可申請暫緩");

    assert_eq!(
        appeal.pi_user_id, delegate,
        "pi_user_id 必須是實際送出申請的代理人本人"
    );
    assert_eq!(
        appeal_delegation_id(&app, appeal.id).await,
        Some(delegation_id),
        "代理人提出的暫緩必須綁上那筆授權，否則稽核上看起來就是 PI 本人申請的"
    );
}

#[tokio::test]
#[serial]
async fn appeal_in_person_leaves_delegation_null() {
    use erp_backend::models::CreateEuthanasiaAppealRequest;
    use erp_backend::services::{EuthanasiaService, ProtocolService};

    let app = TestApp::spawn().await;
    let (pi, _) = seed_login_user(&app, "pi", "PI").await;
    let (sd, _) = seed_login_user(&app, "sd", "EXPERIMENT_STAFF").await;
    let (vet, _) = seed_login_user(&app, "vet", "VET").await;
    let (protocol_id, animal_id) = seed_external_pi_protocol_and_animal(&app, pi, sd).await;
    let order_id = seed_order(&app, animal_id, vet, pi, "pending_pi").await;

    // ⚠️ 這一筆授權是本測試的重點，不是佈景（CodeRabbit #53 第六輪）：
    // 少了它，`pi_appeal` 走本人路徑時 `delegation_id` 本來就只會是 NULL——
    // 優先序邏輯整個寫反也照樣綠。要驗「本人身分優先於代理身分」，
    // 場景就必須是「兩種身分同時成立」。
    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &user_actor_for(sd, "EXPERIMENT_STAFF"),
        protocol_id,
        pi,
        None,
        None,
    )
    .await
    .expect("SD 核准代理人（本例中恰好就是 PI 本人）");

    let appeal = EuthanasiaService::pi_appeal(
        &app.db_pool,
        &user_actor_for(pi, "PI"),
        order_id,
        &CreateEuthanasiaAppealRequest {
            reason: "PI 本人申請暫緩".to_string(),
            attachment_path: None,
            version: None,
        },
    )
    .await
    .expect("PI 本人應可申請暫緩");

    assert_eq!(
        appeal_delegation_id(&app, appeal.id).await,
        None,
        "本人申請不得標成代簽——假的可歸責資訊比沒有更糟"
    );
}

#[tokio::test]
#[serial]
async fn appeal_rejected_when_delegation_revoked() {
    use erp_backend::models::CreateEuthanasiaAppealRequest;
    use erp_backend::services::{EuthanasiaService, ProtocolService};
    use erp_backend::AppError;

    let app = TestApp::spawn().await;
    let (borrowed_pi, _) = seed_login_user(&app, "borrowed-pi", "PI").await;
    let (sd, _) = seed_login_user(&app, "sd", "EXPERIMENT_STAFF").await;
    let (delegate, _) = seed_login_user(&app, "delegate", "PI").await;
    let (vet, _) = seed_login_user(&app, "vet", "VET").await;
    let (protocol_id, animal_id) =
        seed_external_pi_protocol_and_animal(&app, borrowed_pi, sd).await;
    let order_id = seed_order(&app, animal_id, vet, borrowed_pi, "pending_pi").await;

    ProtocolService::authorize_pi_delegate(
        &app.db_pool,
        &user_actor_for(sd, "EXPERIMENT_STAFF"),
        protocol_id,
        delegate,
        None,
        None,
    )
    .await
    .expect("SD 核准代理人");
    ProtocolService::revoke_pi_delegate(
        &app.db_pool,
        &user_actor_for(sd, "EXPERIMENT_STAFF"),
        protocol_id,
        Some("測試：申請前撤銷"),
    )
    .await
    .expect("SD 撤銷代理人");

    let err = EuthanasiaService::pi_appeal(
        &app.db_pool,
        &user_actor_for(delegate, "PI"),
        order_id,
        &CreateEuthanasiaAppealRequest {
            reason: "授權已撤銷仍嘗試申請".to_string(),
            attachment_path: None,
            version: None,
        },
    )
    .await
    .expect_err("授權已撤銷不得再提出暫緩");
    assert!(
        matches!(err, AppError::NotFound(_) | AppError::Forbidden(_)),
        "實得：{err:?}"
    );

    let appeal_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM euthanasia_appeals WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count appeals");
    assert_eq!(appeal_count, 0, "被擋下時不得留下任何暫緩申請紀錄");
}
