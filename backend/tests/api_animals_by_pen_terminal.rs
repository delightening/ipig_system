//! 回歸測試：已安樂死（或猝死/已轉讓）但 pen_location 殘留的動物，
//! 不得出現在欄位視圖 `list_by_pen`，也不得計入 `stats.pen_animals_count`。
//!
//! 背景：正常安樂死/犧牲/猝死流程都會清空 pen_location，但歷史或匯入資料
//! 可能在清除邏輯加入前就已終態，殘留 pen_location。欄位格線與計數須以
//! 「在欄活躍」（status NOT IN euthanized/sudden_death/transferred）為準，
//! 與 pen.current_count 重算定義一致。

mod common;

use common::TestApp;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{AnimalStatus, UpdateAnimalRequest};
use erp_backend::services::AnimalService;
use serial_test::serial;
use uuid::Uuid;

/// 插入一隻帶 pen_location 的動物（給定 status），回傳其 id。
async fn seed_penned_animal(app: &TestApp, status: AnimalStatus, pen_location: &str) -> Uuid {
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date, pen_location, created_by)
           VALUES ($1, $2, $3::animal_status, 'miniature', 'male', '2024-01-01', '2024-06-01', $4, $5)"#,
    )
    .bind(aid)
    .bind(format!("PT{}", &aid.to_string()[..6]))
    .bind(status)
    .bind(pen_location)
    .bind(owner)
    .execute(&app.db_pool)
    .await
    .expect("insert penned animal");
    aid
}

#[tokio::test]
#[serial]
async fn by_pen_excludes_terminal_animals_with_stale_pen_location() {
    let app = TestApp::spawn().await;

    // 唯一欄位名避免與既有測試資料相撞。
    let pen = format!("ZZ{}", Uuid::new_v4().to_string()[..4].to_uppercase());

    let active_id = seed_penned_animal(&app, AnimalStatus::Unassigned, &pen).await;
    let _euth_id = seed_penned_animal(&app, AnimalStatus::Euthanized, &pen).await;
    let _dead_id = seed_penned_animal(&app, AnimalStatus::SuddenDeath, &pen).await;

    let by_pen = AnimalService::list_by_pen(&app.db_pool)
        .await
        .expect("list_by_pen");

    let group = by_pen
        .iter()
        .find(|g| g.pen_location == pen)
        .expect("該欄位應存在（至少有一隻活躍動物）");

    let ids: Vec<Uuid> = group.animals.iter().map(|a| a.id).collect();
    assert_eq!(
        ids,
        vec![active_id],
        "欄位視圖只應含活躍動物，排除已安樂死/猝死者"
    );
}

#[tokio::test]
#[serial]
async fn stats_pen_count_excludes_terminal_animals() {
    let app = TestApp::spawn().await;

    let before = AnimalService::stats(&app.db_pool)
        .await
        .expect("stats before")
        .pen_animals_count;

    let pen = format!("YY{}", Uuid::new_v4().to_string()[..4].to_uppercase());
    seed_penned_animal(&app, AnimalStatus::Unassigned, &pen).await; // +1（活躍）
    seed_penned_animal(&app, AnimalStatus::Euthanized, &pen).await; // 不計
    seed_penned_animal(&app, AnimalStatus::Transferred, &pen).await; // 不計

    let after = AnimalService::stats(&app.db_pool)
        .await
        .expect("stats after")
        .pen_animals_count;

    assert_eq!(
        after - before,
        1,
        "欄位計數只應 +1（活躍動物），終態動物即使殘留 pen_location 也不計"
    );
}

/// 回歸（#999 follow-up High）：透過**通用更新**把動物轉入終態（如編輯頁把狀態改成
/// 安樂死）時，pen_location 必須被清空。舊 SQL `CASE WHEN status = 'euthanized'`
/// 讀到的是更新前的舊值，轉入終態那一次反而保留舊欄位 → 殘留資料持續生成
/// （migration 135 只清過一次歷史）。
#[tokio::test]
#[serial]
async fn generic_update_to_terminal_status_clears_pen_location() {
    let app = TestApp::spawn().await;

    let pen = format!("XX{}", Uuid::new_v4().to_string()[..4].to_uppercase());
    let animal_id = seed_penned_animal(&app, AnimalStatus::Unassigned, &pen).await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let actor = ActorContext::User(CurrentUser {
        id: user_id,
        email: "terminal-test@test.local".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    });

    // 只改狀態、不帶 pen_location（模擬編輯頁把狀態切到安樂死、欄位留原值不動）。
    let req = UpdateAnimalRequest {
        status: Some(AnimalStatus::Euthanized),
        pen_location: None,
        pen_id: None,
        species_id: None,
        iacuc_no: None,
        experiment_date: None,
        remark: None,
        version: None,
    };
    let (updated, _) = AnimalService::update(&app.db_pool, &actor, animal_id, &req)
        .await
        .expect("通用更新轉安樂死應成功");

    assert_eq!(updated.status, AnimalStatus::Euthanized);
    assert_eq!(
        updated.pen_location, None,
        "轉入終態時 pen_location 必須被清空（不得殘留 {pen}）"
    );

    // DB 落地值再驗一次（防 RETURNING 與實際寫入不一致）。
    let db_loc: Option<String> =
        sqlx::query_scalar("SELECT pen_location FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query pen_location");
    assert_eq!(db_loc, None, "DB 中 pen_location 應為 NULL");
}

/// 反向保護：轉入終態的同一請求若還想指定欄位 → 明確拒絕（不是靜默忽略）。
#[tokio::test]
#[serial]
async fn generic_update_to_terminal_status_rejects_pen_assignment() {
    let app = TestApp::spawn().await;

    let pen = format!("WW{}", Uuid::new_v4().to_string()[..4].to_uppercase());
    let animal_id = seed_penned_animal(&app, AnimalStatus::Unassigned, &pen).await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let actor = ActorContext::User(CurrentUser {
        id: user_id,
        email: "terminal-test@test.local".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    });

    let req = UpdateAnimalRequest {
        status: Some(AnimalStatus::Euthanized),
        pen_location: Some("A-99".into()),
        pen_id: None,
        species_id: None,
        iacuc_no: None,
        experiment_date: None,
        remark: None,
        version: None,
    };
    let err = match AnimalService::update(&app.db_pool, &actor, animal_id, &req).await {
        Ok(_) => panic!("轉入終態同時指定欄位應被拒絕"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("無法移動到欄位"),
        "錯誤訊息應說明終態不可移欄，實際：{err}"
    );
}

/// 建最小 facility→building→zone→pen 鏈，回傳可分配的 pen id（capacity 0 = 無限制）。
async fn seed_pen(app: &TestApp) -> Uuid {
    let fid = Uuid::new_v4();
    let bid = Uuid::new_v4();
    let zid = Uuid::new_v4();
    let pid = Uuid::new_v4();
    let s = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '終態測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '終態測試棟')",
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
    sqlx::query("INSERT INTO pens (id, zone_id, code, capacity) VALUES ($1, $2, $3, 0)")
        .bind(pid)
        .bind(zid)
        .bind(format!("P{}", &s[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert pen");
    pid
}

/// 回歸（CodeRabbit #1006 Major）：轉入終態的同一請求不得指派 pen_id——SQL 走
/// COALESCE($10, pen_id)，放行會讓離場動物被指到新欄位。既有 pen_id 保留不清
/// （與 medical/euthanasia 專用流程一致，欄位計數以 status 過濾自癒）。
#[tokio::test]
#[serial]
async fn generic_update_to_terminal_status_rejects_pen_id_assignment() {
    let app = TestApp::spawn().await;

    let pen = format!("VV{}", Uuid::new_v4().to_string()[..4].to_uppercase());
    let animal_id = seed_penned_animal(&app, AnimalStatus::Unassigned, &pen).await;
    let pen_id = seed_pen(&app).await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let actor = ActorContext::User(CurrentUser {
        id: user_id,
        email: "terminal-test@test.local".into(),
        roles: vec!["admin".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    });

    // 有效 pen（通過 validate_pen_for_assignment）+ 轉安樂死 → 應被終態守衛擋下。
    let req = UpdateAnimalRequest {
        status: Some(AnimalStatus::Euthanized),
        pen_location: None,
        pen_id: Some(pen_id),
        species_id: None,
        iacuc_no: None,
        experiment_date: None,
        remark: None,
        version: None,
    };
    let err = match AnimalService::update(&app.db_pool, &actor, animal_id, &req).await {
        Ok(_) => panic!("轉入終態同時指派 pen_id 應被拒絕"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("無法指派欄位"),
        "錯誤訊息應說明終態不可指派欄位，實際：{err}"
    );

    // 動物未被改動：狀態仍 unassigned、pen_id 未被寫入。
    let (status, db_pen_id): (String, Option<Uuid>) =
        sqlx::query_as("SELECT status::text, pen_id FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query animal");
    assert_eq!(status, "unassigned", "拒絕後狀態不得改變");
    assert_eq!(db_pen_id, None, "拒絕後 pen_id 不得被寫入");
}
