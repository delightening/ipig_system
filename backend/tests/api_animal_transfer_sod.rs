//! 動物轉讓五段簽核的職責分離（P0-3，使用者 2026-09-05 裁定選項 B）。
//!
//! ## 改動前的狀態
//!
//! 五段流程有四段（發起 / 指定新計畫 / 完成 / 拒絕）共用 `animal.record.create`，
//! 只有第 4 段（PI 同意）另外經 `check_transfer_signing_authority` 分權——任何持有
//! `animal.record.create` 又能存取該動物的人可以獨力把流程從發起推到完成。
//! 見 `docs/reviews/2026-09-03-code-side-issues.md` §P0-3。
//!
//! ## 本次改動同時修掉兩個既有缺陷（不是設計選擇，是發錯權限）
//!
//! 1. **執秘做不到自己該做的事**：正常流程是執秘（`IACUC_STAFF`）發起與指定新計畫，
//!    但該角色從來沒有 `animal.record.create`（`startup/permissions.rs` 的
//!    IACUC_STAFF 清單），實際上一發起就 403。
//! 2. **純獸醫完全無法核准**：`approve_transfer` 把 `require_permission!("animal.record.create")`
//!    排在 `check_transfer_signing_authority` 之前，而 `VET` 角色同樣沒有那個權限碼——
//!    未兼任 `EXPERIMENT_STAFF` 的獸醫在走到簽署權責檢查之前就被擋掉，
//!    與該檢查自己的錯誤訊息「僅獸醫師或轉出 / 轉入計劃主持人可簽署此轉讓」直接矛盾。
//!    這條路徑從未被 VET-only 使用者真正通過過。
//!
//! ## 本檔守住的行為
//!
//! - 協調段（發起 / 指定 / 完成 / 拒絕）改為 `animal.transfer.manage`，只授予 `IACUC_STAFF`。
//! - `EXPERIMENT_STAFF` 保留 `animal.record.create`（別處的動物紀錄仍要用），但**不再**能
//!   推進轉讓——這是「取代而非疊加」的核心回歸。
//! - 純 `VET` 可核准轉讓。
//! - 既有的「發起人不得自核」與「非 VET / 非兩造 PI 不得核准」不變。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::constants::{ROLE_EXPERIMENT_STAFF, ROLE_IACUC_STAFF, ROLE_PI, ROLE_VET};
use erp_backend::services::AuthService;

const PASSWORD: &str = "TransferSoD$Pw1";

/// 建立一個可登入的使用者並指派角色（可多個），回傳 (id, email)。
async fn seed_login_user(app: &TestApp, label: &str, roles: &[&str]) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "trsod-{label}-{}@test.local",
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
    .bind(format!("transfer sod {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");

    for role in roles {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
        )
        .bind(id)
        .bind(role)
        .execute(&app.db_pool)
        .await
        .expect("assign role");
    }
    (id, email)
}

/// 建一張 APPROVED 計畫，回傳其 `iacuc_no`。
async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> String {
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc_no = format!("IACUC-TS-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, '轉讓職責分離測試計畫', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(format!("PR-TS-{unique}"))
    .bind(&iacuc_no)
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    iacuc_no
}

/// 建最小 facility→building→zone→pen 鏈（capacity 0 = 無限制），回傳 pen id。
async fn seed_pen(app: &TestApp) -> Uuid {
    let (fid, bid, zid, pid) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '轉讓 SoD 測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '轉讓 SoD 測試棟')",
    )
    .bind(bid)
    .bind(fid)
    .bind(format!("B{}", &s[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert building");
    sqlx::query("INSERT INTO zones (id, building_id, code) VALUES ($1, $2, $3)")
        .bind(zid)
        .bind(bid)
        .bind(format!("Z{}", &s[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert zone");
    sqlx::query(
        "INSERT INTO pens (id, zone_id, code, capacity, current_count) VALUES ($1, $2, $3, 0, 1)",
    )
    .bind(pid)
    .bind(zid)
    .bind(format!("P{}", &s[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert pen");
    pid
}

/// 建一隻「存活完成」、在欄、已掛 `iacuc_no` 的動物（＝可發起轉讓的前提）。
async fn seed_completed_animal(app: &TestApp, iacuc_no: &str, pen_id: Uuid, owner: Uuid) -> Uuid {
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date,
                                iacuc_no, pen_id, pen_location, created_by)
           VALUES ($1, $2, 'completed'::animal_status, 'miniature', 'male',
                   '2024-01-01', '2024-06-01', $3, $4, 'TSD-01', $5)"#,
    )
    .bind(aid)
    .bind(format!("TS{}", &aid.to_string()[..6]))
    .bind(iacuc_no)
    .bind(pen_id)
    .bind(owner)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

async fn roles_holding(app: &TestApp, permission_code: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"SELECT r.code
           FROM role_permissions rp
           JOIN roles r       ON r.id = rp.role_id
           JOIN permissions p ON p.id = rp.permission_id
           WHERE p.code = $1
           ORDER BY r.code"#,
    )
    .bind(permission_code)
    .fetch_all(&app.db_pool)
    .await
    .expect("query roles holding permission")
}

/// 一個場景所需的全部角色與資料。
struct Scene {
    /// 執行秘書（IACUC_STAFF）——協調段的執行者。
    staff_token: String,
    /// 純獸醫（僅 VET，未兼 EXPERIMENT_STAFF）。
    vet_token: String,
    /// 純獸醫的 user id（驗簽章落庫時的簽署者身分）。
    vet_id: Uuid,
    /// 試驗工作人員（僅 EXPERIMENT_STAFF，持 animal.record.create）。
    worker_token: String,
    animal_id: Uuid,
    /// 轉入計畫的 iacuc_no（與動物目前所屬計畫不同）。
    to_iacuc_no: String,
}

/// 備妥四個角色 + 兩張計畫 + 一隻可轉讓的動物，並回傳各自的登入 token。
async fn setup_scene(app: &TestApp) -> Scene {
    let (pi_id, _) = seed_login_user(app, "pi", &[ROLE_PI]).await;
    let (_staff_id, staff_email) = seed_login_user(app, "staff", &[ROLE_IACUC_STAFF]).await;
    let (vet_id, vet_email) = seed_login_user(app, "vet", &[ROLE_VET]).await;
    let (_worker_id, worker_email) = seed_login_user(app, "worker", &[ROLE_EXPERIMENT_STAFF]).await;

    let from_iacuc = seed_protocol(app, pi_id).await;
    let to_iacuc_no = seed_protocol(app, pi_id).await;
    let pen_id = seed_pen(app).await;
    let animal_id = seed_completed_animal(app, &from_iacuc, pen_id, pi_id).await;

    Scene {
        staff_token: app
            .login(&staff_email, PASSWORD)
            .await
            .expect("staff login"),
        vet_token: app.login(&vet_email, PASSWORD).await.expect("vet login"),
        vet_id,
        worker_token: app
            .login(&worker_email, PASSWORD)
            .await
            .expect("worker login"),
        animal_id,
        to_iacuc_no,
    }
}

fn initiate_body() -> serde_json::Value {
    serde_json::json!({
        "reason": "計畫結束後移轉至後續試驗",
        "remark": null,
        "transfer_type": "internal"
    })
}

/// 執秘發起 → 獸醫評估 → 執秘指定新計畫，回傳 transfer_id（狀態停在 `plan_assigned`）。
async fn advance_to_plan_assigned(app: &TestApp, scene: &Scene) -> Uuid {
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/transfers", scene.animal_id),
            &initiate_body(),
            &scene.staff_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "執秘持 animal.transfer.manage 應可發起轉讓，實得 {}",
        res.status()
    );
    let body: serde_json::Value = res.json().await.expect("parse transfer json");
    let transfer_id: Uuid = body["id"]
        .as_str()
        .expect("transfer id")
        .parse()
        .expect("parse uuid");

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/vet-evaluate"),
            &serde_json::json!({
                "health_status": "健康",
                "is_fit_for_transfer": true,
                "conditions": null
            }),
            &scene.vet_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "獸醫持 animal.vet.recommend 應可評估，實得 {}",
        res.status()
    );

    let res = app
        .auth_put(
            &format!("/api/v1/transfers/{transfer_id}/assign-plan"),
            &serde_json::json!({ "to_iacuc_no": scene.to_iacuc_no }),
            &scene.staff_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "執秘應可指定轉入計畫，實得 {}",
        res.status()
    );

    transfer_id
}

// ── 授權目錄層：權限碼發給誰 ──────────────────────────────────────────────────

/// 協調段的新權限碼只給執行秘書。admin 走 `is_admin` bypass，不需顯式授權。
#[tokio::test]
#[serial]
async fn transfer_manage_granted_only_to_iacuc_staff() {
    let app = TestApp::spawn().await;
    let holders = roles_holding(&app, "animal.transfer.manage").await;

    assert_eq!(
        holders,
        vec![ROLE_IACUC_STAFF.to_string()],
        "animal.transfer.manage 應僅授予執行秘書（協調者），實際持有者：{holders:?}"
    );
}

/// **取代而非疊加**的核心回歸：`EXPERIMENT_STAFF` / `INTERN` 保留
/// `animal.record.create`（別處的動物紀錄仍靠它），但不得拿到轉讓協調權。
///
/// 若日後有人「順手」把 animal.transfer.manage 也加進這兩個角色，等於把四段
/// 協調權還給同一批人，P0-3 的分離就失效了——本測試就是為此而寫。
#[tokio::test]
#[serial]
async fn record_create_holders_do_not_get_transfer_manage() {
    let app = TestApp::spawn().await;
    let record_create_holders = roles_holding(&app, "animal.record.create").await;
    let transfer_holders = roles_holding(&app, "animal.transfer.manage").await;

    for role in [ROLE_EXPERIMENT_STAFF, "INTERN"] {
        assert!(
            record_create_holders.iter().any(|r| r == role),
            "{role} 應保留 animal.record.create（動物紀錄登錄仍需要），實際持有者：{record_create_holders:?}"
        );
        assert!(
            !transfer_holders.iter().any(|r| r == role),
            "{role} 不得持有 animal.transfer.manage（否則四段協調權又回到同一批人），\
             實際持有者：{transfer_holders:?}"
        );
    }
}

// ── 正向：執秘可以跑完協調段 ──────────────────────────────────────────────────

/// 執秘能把整條流程從發起推到完成（獸醫評估與 PI 同意由各自權責的人做）。
/// 這同時證明第一個既有缺陷已修：改動前執秘連第一步都會 403。
#[tokio::test]
#[serial]
async fn iacuc_staff_drives_coordination_steps_end_to_end() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;

    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/approve"),
            &serde_json::json!({}),
            &scene.vet_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "獸醫應可核准（簽署權責），實得 {}",
        res.status()
    );

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/complete"),
            &serde_json::json!({}),
            &scene.staff_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "執秘應可完成轉讓，實得 {}",
        res.status()
    );

    let status: String =
        sqlx::query_scalar("SELECT status::text FROM animal_transfers WHERE id = $1")
            .bind(transfer_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query transfer status");
    assert_eq!(status, "completed", "五段跑完後轉讓應為 completed");
}

/// 執秘可以拒絕轉讓（拒絕同屬協調段）。
#[tokio::test]
#[serial]
async fn iacuc_staff_can_reject_transfer() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;

    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/transfers", scene.animal_id),
            &initiate_body(),
            &scene.staff_token,
        )
        .await;
    assert!(res.status().is_success(), "執秘應可發起轉讓");
    let body: serde_json::Value = res.json().await.expect("parse transfer json");
    let transfer_id = body["id"].as_str().expect("transfer id").to_string();

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/reject"),
            &serde_json::json!({ "reason": "健康狀況不適合" }),
            &scene.staff_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "執秘應可拒絕轉讓，實得 {}",
        res.status()
    );
}

// ── 反向：EXPERIMENT_STAFF 不再能推進任何協調段 ────────────────────────────────

/// 核心回歸：僅持 `animal.record.create` 的試驗工作人員不能再發起轉讓。
///
/// 注意 403 必須來自權限碼那一關而非動物存取：`EXPERIMENT_STAFF` 持有
/// `aup.protocol.view_all`，`Scoped::<AnimalWrite>::authorize` 對它是短路放行的，
/// 所以擋下來的只可能是 `require_permission!`。錯誤訊息一併斷言以釘住這一點。
#[tokio::test]
#[serial]
async fn experiment_staff_can_no_longer_initiate_transfer() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;

    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/transfers", scene.animal_id),
            &initiate_body(),
            &scene.worker_token,
        )
        .await;
    assert_eq!(
        res.status(),
        403,
        "僅持 animal.record.create 者不得再發起轉讓"
    );
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("animal.transfer.manage"),
        "403 應來自 animal.transfer.manage 這道閘（而非動物存取檢查），實得 body：{text}"
    );
}

/// 另外三個協調段（指定新計畫 / 完成 / 拒絕）同樣擋住 `EXPERIMENT_STAFF`。
/// 流程先由執秘與獸醫推到 `plan_assigned`，確保 403 不是被狀態檢查順帶擋下。
#[tokio::test]
#[serial]
async fn experiment_staff_can_no_longer_assign_complete_or_reject() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;
    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    let res = app
        .auth_put(
            &format!("/api/v1/transfers/{transfer_id}/assign-plan"),
            &serde_json::json!({ "to_iacuc_no": scene.to_iacuc_no }),
            &scene.worker_token,
        )
        .await;
    assert_eq!(res.status(), 403, "試驗工作人員不得指定轉入計畫");

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/reject"),
            &serde_json::json!({ "reason": "測試" }),
            &scene.worker_token,
        )
        .await;
    assert_eq!(res.status(), 403, "試驗工作人員不得拒絕轉讓");

    // 完成需先經 PI 同意；先讓獸醫核准，再確認 complete 仍擋住。
    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/approve"),
            &serde_json::json!({}),
            &scene.vet_token,
        )
        .await;
    assert!(res.status().is_success(), "獸醫核准應成功");

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/complete"),
            &serde_json::json!({}),
            &scene.worker_token,
        )
        .await;
    assert_eq!(
        res.status(),
        403,
        "試驗工作人員不得完成轉讓（external 完成即離場終態）"
    );
}

// ── 修復第二個既有缺陷：純 VET 可以核准 ────────────────────────────────────────

/// 未兼任 `EXPERIMENT_STAFF` 的獸醫可核准轉讓。
///
/// 改動前這裡必然 403：`approve_transfer` 先驗 `animal.record.create`，而 VET
/// 沒有該權限碼，於是永遠走不到 `check_transfer_signing_authority`——那個檢查
/// 第一件事就是 `has_role(VET) → Ok`，等於整條 VET 路徑是死的。
#[tokio::test]
#[serial]
async fn vet_only_user_can_approve_transfer() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;
    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/approve"),
            &serde_json::json!({}),
            &scene.vet_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "純 VET（無 animal.record.create）應可核准轉讓，實得 {}",
        res.status()
    );

    let status: String =
        sqlx::query_scalar("SELECT status::text FROM animal_transfers WHERE id = $1")
            .bind(transfer_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query transfer status");
    assert_eq!(status, "pi_approved", "核准後狀態應為 pi_approved");
}

// ── 既有 SoD 行為不得因本次改動而鬆動 ─────────────────────────────────────────

/// 發起人不得自核：讓同一人身兼 `IACUC_STAFF`（可發起）與 `VET`（可簽署），
/// 通過簽署權責檢查後仍應被 service 層的自核擋板攔下。
#[tokio::test]
#[serial]
async fn initiator_still_cannot_self_approve() {
    let app = TestApp::spawn().await;
    let (pi_id, _) = seed_login_user(&app, "pi-self", &[ROLE_PI]).await;
    let (_dual_id, dual_email) = seed_login_user(&app, "dual", &[ROLE_IACUC_STAFF, ROLE_VET]).await;
    let from_iacuc = seed_protocol(&app, pi_id).await;
    let to_iacuc = seed_protocol(&app, pi_id).await;
    let pen_id = seed_pen(&app).await;
    let animal_id = seed_completed_animal(&app, &from_iacuc, pen_id, pi_id).await;
    let token = app.login(&dual_email, PASSWORD).await.expect("dual login");

    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal_id}/transfers"),
            &initiate_body(),
            &token,
        )
        .await;
    assert!(res.status().is_success(), "兼任執秘者應可發起");
    let body: serde_json::Value = res.json().await.expect("parse transfer json");
    let transfer_id = body["id"].as_str().expect("transfer id").to_string();

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/vet-evaluate"),
            &serde_json::json!({
                "health_status": "健康",
                "is_fit_for_transfer": true,
                "conditions": null
            }),
            &token,
        )
        .await;
    assert!(res.status().is_success(), "兼任獸醫者應可評估");

    let res = app
        .auth_put(
            &format!("/api/v1/transfers/{transfer_id}/assign-plan"),
            &serde_json::json!({ "to_iacuc_no": to_iacuc }),
            &token,
        )
        .await;
    assert!(res.status().is_success(), "兼任執秘者應可指定計畫");

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/approve"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(res.status(), 403, "發起人自核必須被擋");
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("發起轉讓者不得自行核准"),
        "應由 service 層的自核擋板攔下（證明它已通過簽署權責檢查），實得 body：{text}"
    );
}

// ── 簽章端點：同型缺陷的修復與其把關 ──────────────────────────────────────────

/// `POST /api/v1/signatures/transfer/{id}` 與 `approve_transfer` 是同一個形狀的缺陷：
/// 權限碼閘排在 `check_transfer_signing_authority` 之前，而 VET 沒有那個碼。
/// 改動前純 VET 必然 403，改動後應可簽署。
#[tokio::test]
#[serial]
async fn vet_only_user_can_sign_transfer_record() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;
    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    let res = app
        .auth_post(
            &format!("/api/v1/signatures/transfer/{transfer_id}"),
            &serde_json::json!({ "password": PASSWORD }),
            &scene.vet_token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "純 VET（無 animal.record.create）應可簽署轉讓記錄，實得 {}",
        res.status()
    );

    // 表名 / 欄位取自 `services/signature/mod.rs:942-945` 的 INSERT，非憑印象。
    // `entity_id` 是 text（service 端以 `&transfer_id.to_string()` 傳入）。
    let signer: Option<Uuid> = sqlx::query_scalar(
        "SELECT signer_id FROM electronic_signatures \
         WHERE entity_type = 'transfer' AND entity_id = $1",
    )
    .bind(transfer_id.to_string())
    .fetch_optional(&app.db_pool)
    .await
    .expect("query electronic_signatures");
    assert_eq!(
        signer,
        Some(scene.vet_id),
        "簽章應落庫，且簽署者須是那位純 VET"
    );
}

/// 刪掉權限碼閘之後，簽章端點**不得**因此變寬：
/// 既非獸醫、也非兩造計畫 PI 的人仍應被 `check_transfer_signing_authority` 擋下。
#[tokio::test]
#[serial]
async fn non_vet_non_party_pi_still_cannot_sign_transfer_record() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;
    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    let res = app
        .auth_post(
            &format!("/api/v1/signatures/transfer/{transfer_id}"),
            &serde_json::json!({ "password": PASSWORD }),
            &scene.worker_token,
        )
        .await;
    assert_eq!(
        res.status(),
        403,
        "試驗工作人員既非獸醫也非兩造 PI，不得簽署轉讓記錄"
    );
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("僅獸醫師或轉出 / 轉入計劃主持人可簽署此轉讓"),
        "403 應來自 check_transfer_signing_authority（證明刪掉權限碼閘沒有讓端點失守），實得 body：{text}"
    );
}

/// 簽署權責不因本次改動而放寬：既非獸醫、也非兩造計畫 PI 的執秘不得核准。
#[tokio::test]
#[serial]
async fn non_vet_non_party_pi_still_cannot_approve() {
    let app = TestApp::spawn().await;
    let scene = setup_scene(&app).await;
    let transfer_id = advance_to_plan_assigned(&app, &scene).await;

    // 另一位執秘：持 animal.transfer.manage，但不是 VET、也不是兩造計畫的 PI。
    let (_other_id, other_email) = seed_login_user(&app, "staff2", &[ROLE_IACUC_STAFF]).await;
    let other_token = app
        .login(&other_email, PASSWORD)
        .await
        .expect("staff2 login");

    let res = app
        .auth_post(
            &format!("/api/v1/transfers/{transfer_id}/approve"),
            &serde_json::json!({}),
            &other_token,
        )
        .await;
    assert_eq!(
        res.status(),
        403,
        "持協調權不等於持簽署權責，執秘不得代為核准"
    );
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("僅獸醫師或轉出 / 轉入計劃主持人可簽署此轉讓"),
        "403 應來自 check_transfer_signing_authority，實得 body：{text}"
    );
}
