//! 回歸測試：`access::require_animal_access` 經 `iacuc_no` 解析動物所屬計畫。
//!
//! 修復前 `access::get_animal_protocol_id` 執行 `SELECT protocol_id FROM animals`，但
//! `animals` 表並無 `protocol_id` 欄位 → 任何非 view_all 使用者（PI / co-editor）存取
//! 動物子資源時，存取檢查會在 SQL 層報「column does not exist」而 500（而非預期的
//! 403/404）。view_all 角色因前置短路而無感，故缺陷在 prod 長期潛伏。
//!
//! 修復後改以 `animals.iacuc_no = protocols.iacuc_no` 解析 `protocols.id`。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::constants::ROLE_VET;
use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use erp_backend::AppError;

/// 無任何 view_all 角色 / 權限的使用者（roles/permissions 皆空）→ 走 per-protocol 成員檢查
fn non_view_all_user(id: Uuid) -> CurrentUser {
    CurrentUser {
        id,
        email: "animal-access@test.local".into(),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

/// 具 view_all 角色（VET）的使用者 → 走 has_protocol_view_all 短路分支
fn view_all_user(id: Uuid) -> CurrentUser {
    CurrentUser {
        id,
        email: "animal-access-vet@test.local".into(),
        roles: vec![ROLE_VET.to_string()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("aa-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("animal access {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建立一個 APPROVED 計畫，pi_user_id = `pi_id`（使其成為 has_any_protocol_role 命中者）。
/// 回傳 (protocol_id, iacuc_no)。
async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc = format!("IACUC-AA-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'animal access test', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR-AA-{unique}"))
    .bind(&iacuc)
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    (id, iacuc)
}

/// 建立一隻動物，`iacuc_no` 可為 None（未指派計畫）。回傳 animal_id。
async fn seed_animal(app: &TestApp, iacuc_no: Option<&str>, created_by: Uuid) -> Uuid {
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-06-01', $3, $4)"#,
    )
    .bind(aid)
    .bind(format!("AA{}", &aid.to_string()[..6]))
    .bind(iacuc_no)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

// ── 正向：計畫 PI（非 view_all）存取自己計畫的動物 → Ok（同時證明不再 500） ──────────
#[tokio::test]
#[serial]
async fn require_animal_access_grants_protocol_member() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "pi").await;
    let (_protocol_id, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, Some(&iacuc), pi).await;

    access::require_animal_access(&app.db_pool, &non_view_all_user(pi), animal_id)
        .await
        .expect("計畫 PI 應可存取自己計畫的動物（且不應因 protocol_id 欄缺失而 500）");
}

// ── 關鍵 repro：非成員（非 view_all）存取他人計畫的動物 → Forbidden（修復前為 500） ───
#[tokio::test]
#[serial]
async fn require_animal_access_rejects_non_member_without_500() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "owner-pi").await;
    let (_protocol_id, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, Some(&iacuc), pi).await;

    // outsider：存在但與該計畫無任何關聯、且無 view_all
    let outsider = seed_user(&app, "outsider").await;
    let err = access::require_animal_access(&app.db_pool, &non_view_all_user(outsider), animal_id)
        .await
        .expect_err("非成員不應取得存取權");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（修復前因 animals.protocol_id 欄不存在而為 Database/500），實得：{err:?}"
    );
}

// ── 邊界：未指派計畫（iacuc_no NULL）的動物，非 view_all → NotFound（fail-closed） ─────
#[tokio::test]
#[serial]
async fn require_animal_access_unassigned_animal_fails_closed() {
    let app = TestApp::spawn().await;
    let owner = seed_user(&app, "creator").await;
    let animal_id = seed_animal(&app, None, owner).await;

    let outsider = seed_user(&app, "viewer").await;
    let err = access::require_animal_access(&app.db_pool, &non_view_all_user(outsider), animal_id)
        .await
        .expect_err("未指派計畫的動物對非 view_all 使用者應 fail-closed");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應為 NotFound（未指派 → 查無 protocol），實得：{err:?}"
    );
}

// ── 一致性 repro：view_all 角色存取「不存在」的動物 → NotFound（修復前因短路放行回空集合） ──
//
// 修復前 require_animal_access / require_animal_access_basic 對 view_all 角色直接 Ok(())，
// 不檢查動物是否存在 → 子資源端點對假 animal_id 回 200 空集合，而非 view_all 角色卻回 404，
// 同一 URL 因角色不同得到不同 status code。修復後 view_all 也須先確認動物存在。
#[tokio::test]
#[serial]
async fn require_animal_access_view_all_missing_animal_is_not_found() {
    let app = TestApp::spawn().await;
    let vet = seed_user(&app, "vet").await;
    let missing_animal = Uuid::new_v4(); // 從未 insert，必不存在

    let err = access::require_animal_access(&app.db_pool, &view_all_user(vet), missing_animal)
        .await
        .expect_err("view_all 角色對不存在的動物應得 NotFound（與非 view_all 一致）");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應為 NotFound（不因 view_all 短路而放行不存在的動物），實得：{err:?}"
    );
}

// ── 回歸：view_all 角色存取「存在」的動物（即使未指派計畫） → Ok ──────────────────────
#[tokio::test]
#[serial]
async fn require_animal_access_view_all_existing_unassigned_animal_grants() {
    let app = TestApp::spawn().await;
    let owner = seed_user(&app, "creator2").await;
    let animal_id = seed_animal(&app, None, owner).await; // 存在但 iacuc_no NULL

    let vet = seed_user(&app, "vet2").await;
    access::require_animal_access(&app.db_pool, &view_all_user(vet), animal_id)
        .await
        .expect("view_all 角色對既存動物（含未指派計畫）仍應放行");
}

// ── 免計畫讀取（require_animal_read_access）：不存在 → NotFound、存在（含未指派） → Ok ──
// #717 原以 require_animal_access_basic 驗證；本 PR（#716）移除 _basic、改由統一讀取守衛
// require_animal_read_access 處理，view_all / animal.view_all 角色一律先驗動物存在。
#[tokio::test]
#[serial]
async fn require_animal_read_access_missing_animal_is_not_found() {
    let app = TestApp::spawn().await;
    let missing_animal = Uuid::new_v4();

    let err = access::require_animal_read_access(
        &app.db_pool,
        &view_all_user(Uuid::new_v4()),
        missing_animal,
    )
    .await
    .expect_err("不存在的動物在讀取路徑亦應回 NotFound");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應為 NotFound，實得：{err:?}"
    );
}

#[tokio::test]
#[serial]
async fn require_animal_read_access_existing_unassigned_animal_grants() {
    let app = TestApp::spawn().await;
    let owner = seed_user(&app, "creator3").await;
    let animal_id = seed_animal(&app, None, owner).await;

    access::require_animal_read_access(&app.db_pool, &view_all_user(Uuid::new_v4()), animal_id)
        .await
        .expect("既存動物（含未指派計畫）在讀取路徑應放行");
}
