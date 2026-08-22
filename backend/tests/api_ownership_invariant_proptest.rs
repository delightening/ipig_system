//! R75-P3：物件擁有權不變式 property test（model-based proptest）。
//!
//! 背景：R75 整輪修補的是 object-level 授權（IDOR）——所有 `access::require_*`
//! 都是 handler/service body 內「要記得呼叫」的守衛。本測試在**隨機的
//! 使用者權限 × 角色 × 計畫成員身分**組合空間上，驗證授權原語本身的安全不變式：
//!
//!   **未授權的使用者對非自己的資源，`require_*` 必回 `Err`（Forbidden），永不 `Ok`。**
//!
//! 在 access 層，`Ok`/`Err(Forbidden)` 即對應 handler 的 `200` / `403`（handler 以
//! `?` 透傳 `AppError`），故「未授權者永不取得 `Ok`」⟺「永不 200 帶 R 內容」。
//!
//! 設計為 **model-based**：把「應放行」的條件寫成一份**刻意最小的 spec**
//! （只有 `view_all` 權限 / 4 個 view_all 角色 / 計畫成員），proptest 生成大量
//! profile（含 R75 攻擊者實際持有的 `view_own`/`create`/`PI`/`CLIENT` 等），斷言
//! 實作的放行集合**恰好等於** spec。任何「多放行一個權限/角色」的授權面擴張
//! （未來誤改 `VIEW_ALL_ROLES`、誤把 `view_own` 納入等）都會讓本測試失敗。
//!
//! ## 覆蓋的 resource type（object-ownership 不變式）
//! - **protocol**：`require_protocol_related_access`（亦為 copy〔R75-1〕、animal-stats
//!   〔R75-4〕、review-comment resolve〔R75-11〕、change_status 等的授權基礎）。
//! - **animal 讀取**：`require_animal_read_access`（子紀錄讀取 + 基礎紀錄寫入）。
//! - **animal 寫入 / 計畫綁定**：`require_animal_access`（手術/犧牲/病理/照護/獸醫單/
//!   轉讓寫入 + 動物編輯/刪除）。同時驗證**讀寫之別**：`animal.animal.view_all` 只放寬
//!   讀取、不放寬寫入（對應 `Scoped<AnimalRead/Write>` 型別保證）。
//!
//! ## 未涵蓋（不同不變式 / by-design 角色閘 / 另案）—— 不在本 property test 範圍
//! - **vet_patrol 報告**（`require_vet_patrol_view`）：報告無 protocol 欄、本質跨多動物，
//!   為**角色閘**而非 object-ownership（R75-5），不變式不同。
//! - **`require_protocol_view_access`（4-way UNION 變體）**：放行集合不同（user_protocols
//!   限 PI/CLIENT + review/vet assignment + pi_user_id），本檔測較簡單的
//!   `require_protocol_related_access`。
//! - **amendment 審查指派**（`is_amendment_reviewer`）、review/vet assignment 成員路徑。
//! - **byproduct sample / messaging / HR(請假·考勤) / equipment / ERP 文件**：by-design
//!   角色閘（R75-3/6/10），非 object-ownership IDOR。

mod common;

use std::sync::OnceLock;

use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use serial_test::serial;
use sqlx::PgPool;
use tokio::runtime::Runtime;
use uuid::Uuid;

use common::TestApp;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use erp_backend::AppError;

// ── 生成池：含「應放行」與「R75 攻擊者實際持有但不應放行」兩類 ──────────────────

/// 權限池。前 2 個會放行（各自層級），其餘為 R75 攻擊者可能持有但**不應**授予
/// 跨計畫存取的權限（`view_own`/`create` 正是 R75-1/4 的攻擊向量）。
const PERM_POOL: &[&str] = &[
    "aup.protocol.view_all",  // → protocol/animal 全部放行
    "animal.animal.view_all", // → 僅 animal 讀取放行（不放行 protocol、不放行 animal 寫入）
    "aup.protocol.view_own",  // R75-4 攻擊向量：不應放行
    "aup.protocol.create",    // R75-1 攻擊向量：不應放行
    "aup.protocol.edit",
    "animal.record.view",
    "audit.logs.view",
];

/// 角色池。前 4 個為 `VIEW_ALL_ROLES`（放行）；其餘（含外部 PI/CLIENT）非成員時不應放行。
const ROLE_POOL: &[&str] = &[
    "IACUC_CHAIR",
    "IACUC_STAFF",
    "VET",
    "REVIEWER",
    "PI",
    "CLIENT",
    "EXPERIMENT_STAFF",
    "INTERN",
    "QAU",
];

/// spec：哪些角色具 view_all（須與 `access.rs::VIEW_ALL_ROLES` 一致；漂移即測試失敗）。
const MODEL_VIEW_ALL_ROLES: &[&str] = &["IACUC_CHAIR", "IACUC_STAFF", "VET", "REVIEWER"];

// ── model（最小 spec）：實作放行集合須恰好等於下列判定 ────────────────────────

fn model_has_protocol_view_all(perms: &[&str], roles: &[&str]) -> bool {
    perms.contains(&"aup.protocol.view_all")
        || roles.iter().any(|r| MODEL_VIEW_ALL_ROLES.contains(r))
}

fn model_has_animal_view_all(perms: &[&str], roles: &[&str]) -> bool {
    model_has_protocol_view_all(perms, roles) || perms.contains(&"animal.animal.view_all")
}

// ── proptest 策略：權限/角色的隨機子集 ──────────────────────────────────────

fn perms_strategy() -> impl Strategy<Value = Vec<&'static str>> {
    proptest::sample::subsequence(PERM_POOL.to_vec(), 0..=PERM_POOL.len())
}

fn roles_strategy() -> impl Strategy<Value = Vec<&'static str>> {
    proptest::sample::subsequence(ROLE_POOL.to_vec(), 0..=ROLE_POOL.len())
}

// ── 共用 fixture（建一次）：一個 PI 擁有的 protocol、其下一隻動物、一個成員 ──────

struct Fixture {
    app: TestApp,
    protocol_id: Uuid,
    /// 對 `protocol_id` 有 `user_protocols` 列（CLIENT）的成員 id。
    member_id: Uuid,
    /// 隸屬 `protocol_id`（同 iacuc_no）的動物 id。
    animal_id: Uuid,
}

fn rt() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("build tokio runtime"))
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| rt().block_on(build_fixture()))
}

fn make_user(id: Uuid, roles: &[&str], perms: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "ownership-prop@test.local".into(),
        roles: roles.iter().map(|s| s.to_string()).collect(),
        permissions: perms.iter().map(|s| s.to_string()).collect(),
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("own-prop-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .bind("ownership prop user")
    .execute(pool)
    .await
    .expect("insert user");
    id
}

async fn build_fixture() -> Fixture {
    let app = TestApp::spawn().await;
    let pool = app.db_pool.clone();

    let pi = seed_user(&pool).await;

    let protocol_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    // animals.iacuc_no 為 varchar(20)：前綴須短（"IACUC-OP-" 9 + 8 = 17 ≤ 20）。
    let iacuc = format!("IACUC-OP-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'ownership invariant test', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(protocol_id)
    .bind(format!("PR-OP-{unique}"))
    .bind(&iacuc)
    .bind(pi)
    .execute(&pool)
    .await
    .expect("insert protocol");

    let animal_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-06-01', $3, $4)"#,
    )
    .bind(animal_id)
    .bind(format!("OP{}", &animal_id.to_string()[..6]))
    .bind(&iacuc)
    .bind(pi)
    .execute(&pool)
    .await
    .expect("insert animal");

    let member_id = seed_user(&pool).await;
    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(member_id)
    .bind(protocol_id)
    .execute(&pool)
    .await
    .expect("insert membership");

    Fixture {
        app,
        protocol_id,
        member_id,
        animal_id,
    }
}

/// 依 `is_member` 決定 user id：成員（DB 有 user_protocols 列）或全新非成員 uuid。
fn user_for(fx: &Fixture, is_member: bool, roles: &[&str], perms: &[&str]) -> CurrentUser {
    let id = if is_member {
        fx.member_id
    } else {
        Uuid::new_v4()
    };
    make_user(id, roles, perms)
}

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 96,
        ..ProptestConfig::default()
    }
}

// ── 不變式 1：protocol object-ownership ───────────────────────────────────────
#[test]
#[serial]
fn protocol_related_access_matches_minimal_spec() {
    let fx = fixture();
    let strategy = (perms_strategy(), roles_strategy(), any::<bool>());
    TestRunner::new(config())
        .run(&strategy, |(perms, roles, is_member)| {
            let user = user_for(fx, is_member, &roles, &perms);
            let expected_ok = model_has_protocol_view_all(&perms, &roles) || is_member;
            let result = rt().block_on(access::require_protocol_related_access(
                &fx.app.db_pool,
                &user,
                fx.protocol_id,
            ));
            prop_assert_eq!(
                result.is_ok(),
                expected_ok,
                "protocol: perms={:?} roles={:?} member={} → {:?}",
                perms,
                roles,
                is_member,
                result
            );
            if !expected_ok {
                prop_assert!(
                    matches!(result, Err(AppError::Forbidden(_))),
                    "未授權者必為 Forbidden（IDOR 防護），實得 {:?}",
                    result
                );
            }
            Ok(())
        })
        .expect("protocol ownership 不變式須對所有生成 profile 成立");
}

// ── 不變式 2：animal 讀取（含 animal.animal.view_all 放寬）─────────────────────
#[test]
#[serial]
fn animal_read_access_matches_minimal_spec() {
    let fx = fixture();
    let strategy = (perms_strategy(), roles_strategy(), any::<bool>());
    TestRunner::new(config())
        .run(&strategy, |(perms, roles, is_member)| {
            let user = user_for(fx, is_member, &roles, &perms);
            let expected_ok = model_has_animal_view_all(&perms, &roles) || is_member;
            let result = rt().block_on(access::require_animal_read_access(
                &fx.app.db_pool,
                &user,
                fx.animal_id,
            ));
            prop_assert_eq!(
                result.is_ok(),
                expected_ok,
                "animal-read: perms={:?} roles={:?} member={} → {:?}",
                perms,
                roles,
                is_member,
                result
            );
            if !expected_ok {
                prop_assert!(
                    matches!(result, Err(AppError::Forbidden(_))),
                    "未授權者必為 Forbidden（動物讀取 IDOR 防護），實得 {:?}",
                    result
                );
            }
            Ok(())
        })
        .expect("animal-read ownership 不變式須對所有生成 profile 成立");
}

// ── 不變式 3：animal 寫入（嚴格：animal.animal.view_all 不放寬寫入）──────────────
#[test]
#[serial]
fn animal_write_access_matches_minimal_spec() {
    let fx = fixture();
    let strategy = (perms_strategy(), roles_strategy(), any::<bool>());
    TestRunner::new(config())
        .run(&strategy, |(perms, roles, is_member)| {
            let user = user_for(fx, is_member, &roles, &perms);
            // 寫入只認 protocol view_all（不含 animal.animal.view_all）或成員 → 讀寫之別。
            let expected_ok = model_has_protocol_view_all(&perms, &roles) || is_member;
            let result = rt().block_on(access::require_animal_access(
                &fx.app.db_pool,
                &user,
                fx.animal_id,
            ));
            prop_assert_eq!(
                result.is_ok(),
                expected_ok,
                "animal-write: perms={:?} roles={:?} member={} → {:?}",
                perms,
                roles,
                is_member,
                result
            );
            if !expected_ok {
                prop_assert!(
                    matches!(result, Err(AppError::Forbidden(_))),
                    "未授權者必為 Forbidden（動物寫入 IDOR 防護），實得 {:?}",
                    result
                );
            }
            Ok(())
        })
        .expect("animal-write ownership 不變式須對所有生成 profile 成立");
}
