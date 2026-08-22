//! 預定試驗（動物預約與試驗規劃 Phase 1）CRUD + Anonymous 拒絕回歸測試。

mod common;

use common::TestApp;
use serial_test::serial;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{
    CreatePlannedExperimentRequest, ReservableQuery, ReserveAnimalsRequest,
    UnreserveAnimalsRequest, UpdatePlannedExperimentRequest,
};
use erp_backend::services::{AnimalService, PlannedExperimentService};
use erp_backend::AppError;
use uuid::Uuid;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "planned_experiment_test",
};

fn create_req(unit: &str) -> CreatePlannedExperimentRequest {
    CreatePlannedExperimentRequest {
        unit: unit.to_string(),
        description: Some("測試試驗概述".to_string()),
        demand_count: 5,
        protocol_id: None,
    }
}

#[tokio::test]
#[serial]
async fn planned_experiment_crud() {
    let app = TestApp::spawn().await;
    // unit 帶隨機尾碼，避免跨次測試殘留互相干擾
    let uniq = &uuid::Uuid::new_v4().to_string()[..8];

    let a = PlannedExperimentService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("昱展-{uniq}")),
    )
    .await
    .expect("create a");
    assert_eq!(a.demand_count, 5);
    assert!(a.protocol_id.is_none());

    let b = PlannedExperimentService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("國醫-{uniq}")),
    )
    .await
    .expect("create b");

    // 列表含兩者
    let list = PlannedExperimentService::list(&app.db_pool)
        .await
        .expect("list");
    assert!(list.iter().any(|r| r.id == a.id));
    assert!(list.iter().any(|r| r.id == b.id));

    // get
    let got = PlannedExperimentService::get(&app.db_pool, a.id)
        .await
        .expect("get a");
    assert_eq!(got.unit, format!("昱展-{uniq}"));

    // update：改單位 + 需求數；description 為全量取代語意（送 None = 清空）
    let updated = PlannedExperimentService::update(
        &app.db_pool,
        &SYSTEM_TEST,
        a.id,
        &UpdatePlannedExperimentRequest {
            unit: Some(format!("昱展新藥-{uniq}")),
            description: None,
            demand_count: Some(8),
        },
    )
    .await
    .expect("update a");
    assert_eq!(updated.unit, format!("昱展新藥-{uniq}"));
    assert_eq!(updated.demand_count, 8);
    assert!(
        updated.description.is_none(),
        "description 全量取代應清為 None"
    );

    // delete
    PlannedExperimentService::delete(&app.db_pool, &SYSTEM_TEST, a.id)
        .await
        .expect("delete a");
    let err = PlannedExperimentService::get(&app.db_pool, a.id)
        .await
        .expect_err("已刪除應找不到");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應 NotFound，實得：{err:?}"
    );

    // cleanup b
    PlannedExperimentService::delete(&app.db_pool, &SYSTEM_TEST, b.id)
        .await
        .expect("delete b");
}

#[tokio::test]
#[serial]
async fn planned_experiment_rejects_anonymous() {
    let app = TestApp::spawn().await;
    let err = PlannedExperimentService::create(
        &app.db_pool,
        &ActorContext::Anonymous,
        &create_req("anon-unit"),
    )
    .await
    .expect_err("Anonymous 不可建立");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
}

// ── Phase 2：預約 / 解除 / 搜尋 ──

/// 直接插一隻未分配動物，回其 id（測試 fixture）。
async fn insert_unassigned_animal(app: &TestApp) -> Uuid {
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date, created_by)
           VALUES ($1, $2, 'unassigned'::animal_status, 'miniature', 'male', '2024-01-01', '2024-06-01', $3)"#,
    )
    .bind(aid)
    .bind(format!("PE{}", &aid.to_string()[..6]))
    .bind(owner)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

#[tokio::test]
#[serial]
async fn reserve_then_unreserve_flow() {
    let app = TestApp::spawn().await;
    let uniq = &Uuid::new_v4().to_string()[..8];
    let pe = PlannedExperimentService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("預約-{uniq}")),
    )
    .await
    .expect("create pe");
    let aid = insert_unassigned_animal(&app).await;

    // 預約
    let reserved = PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![aid],
            planned_experiment_id: Some(pe.id),
            protocol_id: None,
        },
    )
    .await
    .expect("reserve");
    assert_eq!(reserved.len(), 1);
    assert_eq!(reserved[0].reserved_planned_experiment_id, Some(pe.id));

    // 已預約動物不應再被預約（WHERE reserved IS NULL 過濾掉）
    let again = PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![aid],
            planned_experiment_id: Some(pe.id),
            protocol_id: None,
        },
    )
    .await
    .expect("reserve again");
    assert_eq!(again.len(), 0, "已預約動物不應再被預約");

    // 解除預約
    let unreserved = PlannedExperimentService::unreserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &UnreserveAnimalsRequest {
            animal_ids: vec![aid],
        },
    )
    .await
    .expect("unreserve");
    assert_eq!(unreserved.len(), 1);
    assert!(unreserved[0].reserved_planned_experiment_id.is_none());

    sqlx::query("DELETE FROM animals WHERE id = $1")
        .bind(aid)
        .execute(&app.db_pool)
        .await
        .ok();
    PlannedExperimentService::delete(&app.db_pool, &SYSTEM_TEST, pe.id)
        .await
        .ok();
}

#[tokio::test]
#[serial]
async fn reserve_requires_exactly_one_target() {
    let app = TestApp::spawn().await;
    // 兩個目標都給 → Validation
    let err = PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![Uuid::new_v4()],
            planned_experiment_id: Some(Uuid::new_v4()),
            protocol_id: Some(Uuid::new_v4()),
        },
    )
    .await
    .expect_err("兩目標應報錯");
    assert!(matches!(err, AppError::Validation(_)));
    // 都不給 → Validation
    let err2 = PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![Uuid::new_v4()],
            planned_experiment_id: None,
            protocol_id: None,
        },
    )
    .await
    .expect_err("無目標應報錯");
    assert!(matches!(err2, AppError::Validation(_)));
}

#[tokio::test]
#[serial]
async fn search_reservable_excludes_reserved() {
    let app = TestApp::spawn().await;
    let uniq = &Uuid::new_v4().to_string()[..8];
    let pe = PlannedExperimentService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("搜尋-{uniq}")),
    )
    .await
    .expect("create pe");
    let aid = insert_unassigned_animal(&app).await;
    let q = ReservableQuery {
        gender: None,
        age_months_min: None,
        age_months_max: None,
        weight_min: None,
        weight_max: None,
    };

    let before = PlannedExperimentService::search_reservable(&app.db_pool, &q)
        .await
        .expect("search");
    assert!(
        before.iter().any(|r| r.id == aid),
        "未預約動物應在可預約清單"
    );

    PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![aid],
            planned_experiment_id: Some(pe.id),
            protocol_id: None,
        },
    )
    .await
    .expect("reserve");

    let after = PlannedExperimentService::search_reservable(&app.db_pool, &q)
        .await
        .expect("search after");
    assert!(
        !after.iter().any(|r| r.id == aid),
        "已預約動物不應在可預約清單"
    );

    sqlx::query("DELETE FROM animals WHERE id = $1")
        .bind(aid)
        .execute(&app.db_pool)
        .await
        .ok();
    PlannedExperimentService::delete(&app.db_pool, &SYSTEM_TEST, pe.id)
        .await
        .ok();
}

#[tokio::test]
#[serial]
async fn reservation_planning_shows_planned_group_with_reserved() {
    let app = TestApp::spawn().await;
    let uniq = &Uuid::new_v4().to_string()[..8];
    // create_req 的 demand_count = 5
    let pe = PlannedExperimentService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("規劃頁-{uniq}")),
    )
    .await
    .expect("create pe");
    let aid = insert_unassigned_animal(&app).await;
    PlannedExperimentService::reserve(
        &app.db_pool,
        &SYSTEM_TEST,
        &ReserveAnimalsRequest {
            animal_ids: vec![aid],
            planned_experiment_id: Some(pe.id),
            protocol_id: None,
        },
    )
    .await
    .expect("reserve");

    let groups = PlannedExperimentService::get_reservation_planning(&app.db_pool)
        .await
        .expect("planning");
    let g = groups
        .iter()
        .find(|g| g.id == pe.id)
        .expect("找到規劃試驗群組");
    assert_eq!(g.group_type, "planned");
    assert_eq!(g.demand, 5, "demand 應為 create_req 的 demand_count");
    assert_eq!(g.reserved_count, 1);
    assert_eq!(g.in_experiment_count, 0);
    assert_eq!(g.completed_count, 0);
    assert!(
        g.animals
            .iter()
            .any(|a| a.id == aid && a.relation == "reserved"),
        "群組動物列應含已預約動物"
    );

    sqlx::query("DELETE FROM animals WHERE id = $1")
        .bind(aid)
        .execute(&app.db_pool)
        .await
        .ok();
    PlannedExperimentService::delete(&app.db_pool, &SYSTEM_TEST, pe.id)
        .await
        .ok();
}

/// 直接插一隻指定狀態 + iacuc_no 的動物（繞過 service 驗證，供規劃查詢測試）。
async fn insert_animal_status_iacuc(app: &TestApp, status: &str, iacuc_no: Option<&str>) -> Uuid {
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date, created_by, iacuc_no)
           VALUES ($1, $2, $3::animal_status, 'miniature', 'male', '2024-01-01', '2024-06-01', $4, $5)"#,
    )
    .bind(aid)
    .bind(format!("PT{}", &aid.to_string()[..6]))
    .bind(status)
    .bind(owner)
    .bind(iacuc_no)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

/// 直接插一個指定 status + iacuc_no 的 protocol（最小欄位）。回 protocol id。
async fn insert_protocol(app: &TestApp, iacuc_no: &str, status: &str) -> Uuid {
    let owner: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("需要一個既有 user");
    let pid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, working_content)
           VALUES ($1, $2, $3, $4, $5::protocol_status, $6, $6, '{}'::jsonb)"#,
    )
    .bind(pid)
    .bind(format!("AP-{}", &pid.to_string()[..8]))
    .bind(iacuc_no)
    .bind("測試計畫")
    .bind(status)
    .bind(owner)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    pid
}

/// Phase 5：備註 inline 編輯持久化 + 走 audit（ANIMAL_REMARK_UPDATE）+ 空白正規化為 NULL。
#[tokio::test]
#[serial]
async fn update_remark_persists_and_audits() {
    let app = TestApp::spawn().await;
    let aid = insert_unassigned_animal(&app).await;

    let updated = AnimalService::update_remark(
        &app.db_pool,
        &SYSTEM_TEST,
        aid,
        Some("預計 6/25 淘汰".to_string()),
    )
    .await
    .expect("update remark");
    assert_eq!(updated.remark.as_deref(), Some("預計 6/25 淘汰"));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = 'ANIMAL_REMARK_UPDATE' AND entity_id::text = $1",
    )
    .bind(aid.to_string())
    .fetch_one(&app.db_pool)
    .await
    .expect("count audit");
    assert!(
        audit_count >= 1,
        "備註更新須寫入 ANIMAL_REMARK_UPDATE audit"
    );

    // 空白備註正規化為 NULL
    let cleared =
        AnimalService::update_remark(&app.db_pool, &SYSTEM_TEST, aid, Some("   ".to_string()))
            .await
            .expect("clear remark");
    assert!(cleared.remark.is_none(), "空白備註應正規化為 NULL");

    sqlx::query("DELETE FROM animals WHERE id = $1")
        .bind(aid)
        .execute(&app.db_pool)
        .await
        .ok();
}

/// Phase 5：備註更新拒絕 Anonymous。
#[tokio::test]
#[serial]
async fn update_remark_rejects_anonymous() {
    let app = TestApp::spawn().await;
    let aid = insert_unassigned_animal(&app).await;
    let err = AnimalService::update_remark(
        &app.db_pool,
        &ActorContext::Anonymous,
        aid,
        Some("x".to_string()),
    )
    .await
    .expect_err("Anonymous 應被拒");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應拒絕匿名（Forbidden），實得：{err:?}"
    );
    sqlx::query("DELETE FROM animals WHERE id = $1")
        .bind(aid)
        .execute(&app.db_pool)
        .await
        .ok();
}

/// Phase 5：空的已核准計劃仍顯示（露缺口）；completed 豬計入 completed_count；
/// 掛在已結案計畫下的存活動物落入 orphan catch-all 組。
#[tokio::test]
#[serial]
async fn reservation_planning_completed_and_orphan() {
    let app = TestApp::spawn().await;
    let uniq = &Uuid::new_v4().to_string()[..8];

    // 已核准計劃 + 一隻 completed 豬（掛其 iacuc）
    let appr_iacuc = format!("PIG-A{uniq}");
    let appr_pid = insert_protocol(&app, &appr_iacuc, "APPROVED").await;
    let completed_aid = insert_animal_status_iacuc(&app, "completed", Some(&appr_iacuc)).await;

    // 已結案計劃 + 一隻 completed 豬（掛其 iacuc）→ 應落 orphan
    let closed_iacuc = format!("PIG-C{uniq}");
    let closed_pid = insert_protocol(&app, &closed_iacuc, "CLOSED").await;
    let orphan_aid = insert_animal_status_iacuc(&app, "completed", Some(&closed_iacuc)).await;

    let groups = PlannedExperimentService::get_reservation_planning(&app.db_pool)
        .await
        .expect("planning");

    // 已核准組出現（即便只有 completed 豬、無 reserved/in_experiment），completed 計入
    let appr = groups
        .iter()
        .find(|g| g.id == appr_pid)
        .expect("已核准計劃組應顯示");
    assert_eq!(appr.group_type, "approved");
    assert_eq!(
        appr.completed_count, 1,
        "completed 豬應計入 completed_count"
    );
    assert!(appr
        .animals
        .iter()
        .any(|a| a.id == completed_aid && a.relation == "completed"));

    // orphan 組接住已結案計劃下的存活豬
    let orphan = groups
        .iter()
        .find(|g| g.group_type == "orphan")
        .expect("orphan 組應存在");
    assert!(
        orphan.animals.iter().any(|a| a.id == orphan_aid),
        "已結案計劃下的存活豬應落入 orphan 組"
    );
    // 已結案計劃本身不應以計劃組顯示
    assert!(
        !groups
            .iter()
            .any(|g| g.iacuc_no.as_deref() == Some(closed_iacuc.as_str())),
        "已結案計劃不應顯示為計劃組"
    );

    sqlx::query("DELETE FROM animals WHERE id = ANY($1)")
        .bind(vec![completed_aid, orphan_aid])
        .execute(&app.db_pool)
        .await
        .ok();
    sqlx::query("DELETE FROM protocols WHERE id = ANY($1)")
        .bind(vec![appr_pid, closed_pid])
        .execute(&app.db_pool)
        .await
        .ok();
}
