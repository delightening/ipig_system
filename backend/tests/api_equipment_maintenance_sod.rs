//! SEC-SoD（資安稽核 L-2）設備維護保養驗收職權分離回歸測試。
//!
//! 修復前：`review_maintenance_record` 只檢 `equipment.maintenance.review` /
//! `equipment.manage` 權限，未比對「登錄者（created_by）≠ 驗收者」，同時持權者
//! 可自簽驗收自己登錄的維護紀錄（與 approve_disposal 的自核守衛不一致）。
//! 本測試斷言登錄者驗收自己的紀錄回 403，且紀錄維持待驗收、設備狀態不變。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::{AuthService, MAINTENANCE_RESIGN_SUPERSEDE_REASON};

/// 建立可登入的內部使用者，回傳 id（供「登錄者 ≠ 驗收者」情境用）。
async fn seed_internal_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!(
        "maint-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..6]
    );
    let hash = AuthService::hash_password("iPig$ecure1").expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("maintenance {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert internal user");
    id
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

async fn seed_equipment(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO equipment (id, name, status) VALUES ($1, $2, 'inactive')")
        .bind(id)
        .bind(format!("高壓滅菌釜-{}", &id.to_string()[..6]))
        .execute(&app.db_pool)
        .await
        .expect("insert equipment");
    id
}

/// 種一筆待驗收（pending_review）的維護紀錄，回傳紀錄 id。
async fn seed_pending_review_record(app: &TestApp, equipment_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO equipment_maintenance_records
               (id, equipment_id, maintenance_type, status, reported_at, created_by)
           VALUES ($1, $2, 'repair', 'pending_review', CURRENT_DATE, $3)"#,
    )
    .bind(id)
    .bind(equipment_id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert maintenance record");
    id
}

// ── SEC-SoD（L-2）：登錄者不得驗收自己的維護保養紀錄 ──
#[tokio::test]
#[serial]
async fn review_maintenance_rejects_self_signoff() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    // 登錄者 = 驗收者（admin 本人），觸發職權分離守衛
    let admin_id = admin_user_id(&app).await;
    let equipment_id = seed_equipment(&app).await;
    let record_id = seed_pending_review_record(&app, equipment_id, admin_id).await;

    let res = app
        .auth_post(
            &format!("/api/v1/equipment-maintenance/{record_id}/review"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "登錄者驗收自己的維護紀錄必須被職權分離守衛擋下（403）"
    );

    // 自簽被擋後：紀錄維持 pending_review、設備維持 inactive（未被自動恢復）
    let rec_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment_maintenance_records WHERE id = $1")
            .bind(record_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch record status");
    assert_eq!(
        rec_status, "pending_review",
        "自簽被擋後紀錄應維持 pending_review"
    );

    let equip_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch equipment status");
    assert_eq!(equip_status, "inactive", "自簽被擋後設備不應被自動恢復啟用");
}

// ── SEC-SoD（L-2）：自簽在「簽章」步驟就被擋，且不留孤兒簽章 ──
//
// 驗收在前端是「先簽章、後 review」兩支獨立請求，SoD 若只擋在 review，登錄者自簽時
// 第一步（簽章）會先把 reviewer_signature_id 寫入（上鎖），第二步才被擋，導致紀錄卡在
// 「已簽章 + 仍待驗收」、之後每次重試都倒在「已簽章，不得覆寫」。本測試斷言 SoD 已提前到
// 簽章步驟：自簽直接 403，reviewer_signature_id 維持 NULL，且不產生任何 electronic_signatures。
#[tokio::test]
#[serial]
async fn sign_maintenance_reviewer_rejects_self_signoff_without_orphan() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let admin_id = admin_user_id(&app).await;
    let equipment_id = seed_equipment(&app).await;
    let record_id = seed_pending_review_record(&app, equipment_id, admin_id).await;

    let res = app
        .auth_post(
            &format!("/api/v1/signatures/maintenance/{record_id}/reviewer"),
            &serde_json::json!({ "password": "iPig$ecure1", "handwriting_svg": "<svg/>" }),
            &token,
        )
        .await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "登錄者自簽驗收自己的維護紀錄必須在簽章步驟就被職權分離守衛擋下（403）"
    );

    // 未產生孤兒簽章：reviewer_signature_id 維持 NULL、紀錄未上鎖
    let sig_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT reviewer_signature_id FROM equipment_maintenance_records WHERE id = $1",
    )
    .bind(record_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch reviewer_signature_id");
    assert!(
        sig_id.is_none(),
        "自簽被擋後不應留下孤兒簽章（reviewer_signature_id 應維持 NULL）"
    );

    // electronic_signatures 也不應有任何 maintenance_reviewer 簽章
    let sig_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM electronic_signatures WHERE entity_type = 'maintenance_reviewer' AND entity_id = $1",
    )
    .bind(record_id.to_string())
    .fetch_one(&app.db_pool)
    .await
    .expect("count signatures");
    assert_eq!(sig_count, 0, "自簽被擋後不應寫入任何 electronic_signatures");
}

// ── 自我復原：待驗收紀錄殘留的簽章可被重新簽章取代並完成驗收 ──
//
// 模擬「簽章成功但 review 未完成」造成的卡住狀態：合法驗收者（≠ 登錄者）先簽一次
// （寫入 reviewer_signature_id），再簽一次——舊碼會回 409「已簽章，不得覆寫」而永久卡住，
// 新碼允許重新簽章取代（舊簽章 row 保留維持稽核鏈），最後 review 順利完成、狀態轉 completed。
#[tokio::test]
#[serial]
async fn sign_maintenance_reviewer_supersedes_leftover_and_completes() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    // 登錄者為另一名內部使用者，admin 為合法驗收者（SoD 通過）
    let creator_id = seed_internal_user(&app, "creator").await;
    let equipment_id = seed_equipment(&app).await;
    let record_id = seed_pending_review_record(&app, equipment_id, creator_id).await;

    let sign_body = serde_json::json!({ "password": "iPig$ecure1", "handwriting_svg": "<svg/>" });
    let sign_path = format!("/api/v1/signatures/maintenance/{record_id}/reviewer");

    // 第一次簽章成功 → reviewer_signature_id 寫入（模擬卡住前的第一步）
    let res1 = app.auth_post(&sign_path, &sign_body, &token).await;
    assert!(
        res1.status().is_success(),
        "第一次簽章應成功，實得 {}",
        res1.status()
    );
    let sig1: Option<Uuid> = sqlx::query_scalar(
        "SELECT reviewer_signature_id FROM equipment_maintenance_records WHERE id = $1",
    )
    .bind(record_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch sig1");
    assert!(
        sig1.is_some(),
        "第一次簽章後 reviewer_signature_id 應被寫入"
    );

    // 第二次簽章（模擬 review 失敗後重試）→ 新碼允許取代，不再回 409
    let res2 = app.auth_post(&sign_path, &sign_body, &token).await;
    assert!(
        res2.status().is_success(),
        "待驗收紀錄殘留簽章時，重新簽章應被允許（自我復原），實得 {}",
        res2.status()
    );
    let sig2: Option<Uuid> = sqlx::query_scalar(
        "SELECT reviewer_signature_id FROM equipment_maintenance_records WHERE id = $1",
    )
    .bind(record_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch sig2");
    assert!(sig2.is_some(), "重新簽章後 reviewer_signature_id 應仍存在");
    assert_ne!(
        sig1, sig2,
        "重新簽章應指向新的簽章（舊 row 保留於 electronic_signatures）"
    );

    // 舊簽章 row 應保留（不刪），維持 HMAC 稽核鏈不斷鏈 → 同一實體共 2 筆簽章 row。
    let total_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM electronic_signatures WHERE entity_type = 'maintenance_reviewer' AND entity_id = $1",
    )
    .bind(record_id.to_string())
    .fetch_one(&app.db_pool)
    .await
    .expect("count all signatures");
    assert_eq!(
        total_count, 2,
        "重新簽章後應保留舊簽章 row（新舊共 2 筆），維持稽核鏈不斷鏈"
    );

    // 21 CFR Part 11 簽章唯一性：舊孤兒簽章 row 保留（維持稽核鏈）但須標記為無效，
    // 且作廢原因對齊單一事實來源常數，避免同一實體同時存在多個有效簽章。
    let (old_valid, old_reason): (bool, Option<String>) = sqlx::query_as(
        "SELECT is_valid, invalidated_reason FROM electronic_signatures WHERE id = $1",
    )
    .bind(sig1.expect("sig1 應存在"))
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch old signature status");
    assert!(!old_valid, "被取代的舊簽章應標記為 is_valid=false");
    assert_eq!(
        old_reason.as_deref(),
        Some(MAINTENANCE_RESIGN_SUPERSEDE_REASON),
        "舊簽章作廢原因應對齊 MAINTENANCE_RESIGN_SUPERSEDE_REASON 常數"
    );
    // 新簽章應為唯一有效簽章
    let valid_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM electronic_signatures WHERE entity_type = 'maintenance_reviewer' AND entity_id = $1 AND is_valid = true",
    )
    .bind(record_id.to_string())
    .fetch_one(&app.db_pool)
    .await
    .expect("count valid signatures");
    assert_eq!(valid_count, 1, "重簽後同一實體應僅有一個有效簽章");

    // review 順利完成 → 狀態轉 completed、設備自動恢復啟用
    let res3 = app
        .auth_post(
            &format!("/api/v1/equipment-maintenance/{record_id}/review"),
            &serde_json::json!({ "approved": true }),
            &token,
        )
        .await;
    assert!(
        res3.status().is_success(),
        "驗收 review 應完成，實得 {}",
        res3.status()
    );
    let rec_status: String =
        sqlx::query_scalar("SELECT status::text FROM equipment_maintenance_records WHERE id = $1")
            .bind(record_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch record status");
    assert_eq!(rec_status, "completed", "驗收完成後紀錄應轉為 completed");
}
