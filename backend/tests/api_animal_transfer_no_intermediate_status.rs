//! 回歸測試（issue #180）：`transferred` 不再是轉讓流程的中間態。
//!
//! 舊行為：`initiate_transfer` 立刻把 `animals.status` 設為 `'transferred'`，但動物在整個
//! 簽核期間（發起 → 獸醫評估 → 指定計劃 → PI 同意）仍實際待在原欄。後果是欄舍頭數在流程
//! 期間偏低（靠三處補償性 recalc 硬壓）、前端得把「已轉讓」特判成「轉讓申請中」、駁回時
//! 還要回滾動物狀態。
//!
//! 新行為：
//!   - 簽核期間 `animals.status` 維持 `completed`，`pens.current_count` 全程不變。
//!   - 「轉讓申請中」改由 `animal_transfers` 未結案列表示
//!     （`AnimalListItem::pending_transfer_status`）。
//!   - 完成時才改狀態：internal → `in_experiment`（進計畫 B）；
//!     external → `transferred`（交付其他機構、實際離場的終態）。
//!   - 駁回為 no-op（沒有動過的狀態就不需要回滾）。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::constants::ROLE_VET;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{
    AnimalQuery, AnimalStatus, AnimalTransferStatus, AssignTransferPlanRequest,
    CreateTransferRequest, RejectTransferRequest, VetEvaluateTransferRequest,
};
use erp_backend::services::{access, AnimalService, AnimalTransferService};

/// VET 具 view_all，`Scoped::<AnimalWrite>::authorize` 會短路放行，
/// 測試不必再鋪計畫成員關係。角色代碼取自 `constants::ROLE_VET`。
fn vet_user(id: Uuid, label: &str) -> CurrentUser {
    CurrentUser {
        id,
        email: format!("{label}@transfer-test.local"),
        roles: vec![ROLE_VET.to_string()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

/// 只查耳號 / 未結案轉讓的最小 query（`AnimalQuery` 無 `Default`，欄位需逐一列出）。
fn query_by_keyword(keyword: String) -> AnimalQuery {
    AnimalQuery {
        status: None,
        breed: None,
        gender: None,
        iacuc_no: None,
        pen_location: None,
        keyword: Some(keyword),
        is_on_medication: None,
        page: None,
        per_page: None,
        sort_by: None,
        sort_order: None,
    }
}

async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("tr-{label}-{}@test.local", &id.to_string()[..6]))
    .bind(format!("transfer {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建一張 APPROVED 計畫，回傳其 `iacuc_no`。
async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> String {
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc_no = format!("IACUC-TR-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, '轉讓測試計畫', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(format!("PR-TR-{unique}"))
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
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '轉讓測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '轉讓測試棟')",
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

/// 建一隻「實驗完成」、在欄、已掛 `iacuc_no` 的動物（＝可發起轉讓的前提）。
async fn seed_completed_animal(app: &TestApp, iacuc_no: &str, pen_id: Uuid, owner: Uuid) -> Uuid {
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, birth_date, entry_date,
                                iacuc_no, pen_id, pen_location, created_by)
           VALUES ($1, $2, 'completed'::animal_status, 'miniature', 'male',
                   '2024-01-01', '2024-06-01', $3, $4, 'TRF-01', $5)"#,
    )
    .bind(aid)
    .bind(format!("TR{}", &aid.to_string()[..6]))
    .bind(iacuc_no)
    .bind(pen_id)
    .bind(owner)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

async fn animal_status(app: &TestApp, animal_id: Uuid) -> String {
    sqlx::query_scalar("SELECT status::text FROM animals WHERE id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query animal status")
}

async fn pen_count(app: &TestApp, pen_id: Uuid) -> i32 {
    sqlx::query_scalar("SELECT current_count FROM pens WHERE id = $1")
        .bind(pen_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query pen count")
}

/// 一次備妥：兩個使用者（發起者 / 核准者，SoD 需不同人）、來源計畫、欄位、動物。
struct Fixture {
    initiator: CurrentUser,
    approver: CurrentUser,
    pen_id: Uuid,
    animal_id: Uuid,
}

async fn setup(app: &TestApp) -> Fixture {
    let initiator_id = seed_user(app, "initiator").await;
    let approver_id = seed_user(app, "approver").await;
    let from_iacuc = seed_protocol(app, initiator_id).await;
    let pen_id = seed_pen(app).await;
    let animal_id = seed_completed_animal(app, &from_iacuc, pen_id, initiator_id).await;
    Fixture {
        initiator: vet_user(initiator_id, "initiator"),
        approver: vet_user(approver_id, "approver"),
        pen_id,
        animal_id,
    }
}

async fn scope_for(
    app: &TestApp,
    user: &CurrentUser,
    animal_id: Uuid,
) -> access::Scoped<access::AnimalWrite> {
    access::Scoped::<access::AnimalWrite>::authorize(&app.db_pool, user, animal_id)
        .await
        .expect("VET 應可取得動物寫入授權")
}

/// 推進到「PI 已同意」（步驟 1–4），回傳 transfer_id。
async fn advance_to_pi_approved(
    app: &TestApp,
    fx: &Fixture,
    transfer_type: &str,
    to_iacuc: &str,
) -> Uuid {
    let initiator_actor = ActorContext::User(fx.initiator.clone());
    let approver_actor = ActorContext::User(fx.approver.clone());

    let record = AnimalTransferService::initiate_transfer(
        &app.db_pool,
        &initiator_actor,
        scope_for(app, &fx.initiator, fx.animal_id).await,
        &CreateTransferRequest {
            reason: "測試轉讓".into(),
            remark: None,
            transfer_type: transfer_type.into(),
        },
    )
    .await
    .expect("發起轉讓");

    AnimalTransferService::vet_evaluate_transfer(
        &app.db_pool,
        &initiator_actor,
        scope_for(app, &fx.initiator, fx.animal_id).await,
        record.id,
        &VetEvaluateTransferRequest {
            health_status: "健康".into(),
            is_fit_for_transfer: true,
            conditions: None,
        },
    )
    .await
    .expect("獸醫評估");

    AnimalTransferService::assign_transfer_plan(
        &app.db_pool,
        &initiator_actor,
        scope_for(app, &fx.initiator, fx.animal_id).await,
        record.id,
        &AssignTransferPlanRequest {
            to_iacuc_no: to_iacuc.into(),
        },
    )
    .await
    .expect("指定新計劃");

    // SoD：核准者須與發起者不同人。
    AnimalTransferService::approve_transfer(
        &app.db_pool,
        &approver_actor,
        scope_for(app, &fx.approver, fx.animal_id).await,
        record.id,
    )
    .await
    .expect("PI 同意");

    record.id
}

/// 核心回歸：發起轉讓後動物狀態與欄舍頭數都不得變動。
#[tokio::test]
#[serial]
async fn initiate_transfer_leaves_animal_status_and_pen_count_untouched() {
    let app = TestApp::spawn().await;
    let fx = setup(&app).await;

    let count_before = pen_count(&app, fx.pen_id).await;

    let record = AnimalTransferService::initiate_transfer(
        &app.db_pool,
        &ActorContext::User(fx.initiator.clone()),
        scope_for(&app, &fx.initiator, fx.animal_id).await,
        &CreateTransferRequest {
            reason: "測試轉讓".into(),
            remark: None,
            transfer_type: "internal".into(),
        },
    )
    .await
    .expect("發起轉讓");

    assert_eq!(
        animal_status(&app, fx.animal_id).await,
        "completed",
        "簽核期間動物仍在原欄，狀態必須維持 completed（不得寫成中間態 transferred）"
    );
    assert_eq!(
        pen_count(&app, fx.pen_id).await,
        count_before,
        "動物實體沒有離開欄位，current_count 不得變動"
    );
    assert_eq!(record.status, AnimalTransferStatus::Pending);
}

/// 待轉讓改由 `animal_transfers` 未結案列呈現，而非動物狀態。
#[tokio::test]
#[serial]
async fn pending_transfer_is_exposed_on_animal_list() {
    let app = TestApp::spawn().await;
    let fx = setup(&app).await;

    let ear_tag: String = sqlx::query_scalar("SELECT ear_tag FROM animals WHERE id = $1")
        .bind(fx.animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("query ear_tag");
    let query = query_by_keyword(ear_tag);

    let before = AnimalService::list(&app.db_pool, &query)
        .await
        .expect("list before");
    let item = before
        .data
        .iter()
        .find(|a| a.id == fx.animal_id)
        .expect("動物應在列表中");
    assert_eq!(
        item.pending_transfer_status, None,
        "尚未發起轉讓時不應有 pending_transfer_status"
    );

    AnimalTransferService::initiate_transfer(
        &app.db_pool,
        &ActorContext::User(fx.initiator.clone()),
        scope_for(&app, &fx.initiator, fx.animal_id).await,
        &CreateTransferRequest {
            reason: "測試轉讓".into(),
            remark: None,
            transfer_type: "internal".into(),
        },
    )
    .await
    .expect("發起轉讓");

    let after = AnimalService::list(&app.db_pool, &query)
        .await
        .expect("list after");
    let item = after
        .data
        .iter()
        .find(|a| a.id == fx.animal_id)
        .expect("動物應在列表中");
    assert_eq!(
        item.pending_transfer_status.as_deref(),
        Some("pending"),
        "發起轉讓後應由 pending_transfer_status 表示「轉讓申請中」"
    );
    assert_eq!(
        item.status,
        AnimalStatus::Completed,
        "動物狀態本身不得被轉讓流程改動"
    );
}

/// 內部轉讓完成 → 進計畫 B（in_experiment），全程留在原欄、頭數不變。
#[tokio::test]
#[serial]
async fn internal_transfer_completes_into_new_protocol_without_pen_drift() {
    let app = TestApp::spawn().await;
    let fx = setup(&app).await;
    let to_iacuc = seed_protocol(&app, fx.approver.id).await;

    let count_before = pen_count(&app, fx.pen_id).await;
    let transfer_id = advance_to_pi_approved(&app, &fx, "internal", &to_iacuc).await;

    assert_eq!(
        pen_count(&app, fx.pen_id).await,
        count_before,
        "整段簽核期間頭數不得漂移"
    );

    AnimalTransferService::complete_transfer(
        &app.db_pool,
        &ActorContext::User(fx.approver.clone()),
        scope_for(&app, &fx.approver, fx.animal_id).await,
        transfer_id,
    )
    .await
    .expect("完成轉讓");

    assert_eq!(
        animal_status(&app, fx.animal_id).await,
        "in_experiment",
        "內部轉讓完成後動物直接進入新計畫，不經 transferred 中轉"
    );
    let (iacuc, pen): (Option<String>, Option<Uuid>) =
        sqlx::query_as("SELECT iacuc_no, pen_id FROM animals WHERE id = $1")
            .bind(fx.animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query animal");
    assert_eq!(iacuc.as_deref(), Some(to_iacuc.as_str()));
    assert_eq!(pen, Some(fx.pen_id), "內部轉讓動物仍在原欄");
    assert_eq!(
        pen_count(&app, fx.pen_id).await,
        count_before,
        "內部轉讓 completed → in_experiment 兩者皆計入頭數，count 不變"
    );
}

/// 外部轉讓完成 → `transferred` 終態、離開欄位、頭數減 1。
///
/// 修正的既有缺陷：舊版此處也寫 `in_experiment`，已交付其他機構的動物永遠顯示「實驗中」，
/// 並被 `repositories/ai.rs` 的 active_animals 統計成在養動物。
#[tokio::test]
#[serial]
async fn external_transfer_completes_into_terminal_transferred() {
    let app = TestApp::spawn().await;
    let fx = setup(&app).await;
    let to_iacuc = seed_protocol(&app, fx.approver.id).await;

    let count_before = pen_count(&app, fx.pen_id).await;
    let transfer_id = advance_to_pi_approved(&app, &fx, "external", &to_iacuc).await;

    AnimalTransferService::complete_transfer(
        &app.db_pool,
        &ActorContext::User(fx.approver.clone()),
        scope_for(&app, &fx.approver, fx.animal_id).await,
        transfer_id,
    )
    .await
    .expect("完成轉讓");

    assert_eq!(
        animal_status(&app, fx.animal_id).await,
        "transferred",
        "外部轉讓完成＝動物實際離場，應落在 transferred 終態"
    );
    let (pen_id, pen_location): (Option<Uuid>, Option<String>) =
        sqlx::query_as("SELECT pen_id, pen_location FROM animals WHERE id = $1")
            .bind(fx.animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("query animal");
    assert_eq!(pen_id, None, "離場動物須清空 pen_id");
    assert_eq!(pen_location, None, "離場動物須清空 pen_location");
    assert_eq!(
        pen_count(&app, fx.pen_id).await,
        count_before - 1,
        "動物實際離場，舊欄頭數應減 1"
    );
}

/// 駁回是 no-op：狀態與頭數從頭到尾沒被動過，自然不需要回滾。
#[tokio::test]
#[serial]
async fn reject_transfer_is_a_no_op_for_animal_state() {
    let app = TestApp::spawn().await;
    let fx = setup(&app).await;

    let count_before = pen_count(&app, fx.pen_id).await;

    let record = AnimalTransferService::initiate_transfer(
        &app.db_pool,
        &ActorContext::User(fx.initiator.clone()),
        scope_for(&app, &fx.initiator, fx.animal_id).await,
        &CreateTransferRequest {
            reason: "測試轉讓".into(),
            remark: None,
            transfer_type: "internal".into(),
        },
    )
    .await
    .expect("發起轉讓");

    AnimalTransferService::reject_transfer(
        &app.db_pool,
        &ActorContext::User(fx.approver.clone()),
        scope_for(&app, &fx.approver, fx.animal_id).await,
        record.id,
        &RejectTransferRequest {
            reason: "健康狀況不適合".into(),
        },
    )
    .await
    .expect("駁回轉讓");

    assert_eq!(
        animal_status(&app, fx.animal_id).await,
        "completed",
        "駁回後動物狀態應仍是 completed（流程期間本就沒改過）"
    );
    assert_eq!(
        pen_count(&app, fx.pen_id).await,
        count_before,
        "駁回後頭數不得變動"
    );

    // 駁回後不再是「轉讓申請中」。
    let pending: Option<String> = sqlx::query_scalar(
        "SELECT status::text FROM animal_transfers \
         WHERE animal_id = $1 AND status NOT IN ('completed', 'rejected')",
    )
    .bind(fx.animal_id)
    .fetch_optional(&app.db_pool)
    .await
    .expect("query pending transfer");
    assert_eq!(pending, None, "駁回後不應留下未結案轉讓");

    // 可重新發起（舊版靠回滾狀態才做得到，現在是自然結果）。
    AnimalTransferService::initiate_transfer(
        &app.db_pool,
        &ActorContext::User(fx.initiator.clone()),
        scope_for(&app, &fx.initiator, fx.animal_id).await,
        &CreateTransferRequest {
            reason: "重新申請".into(),
            remark: None,
            transfer_type: "internal".into(),
        },
    )
    .await
    .expect("駁回後應可重新發起轉讓");
}
