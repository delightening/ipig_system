//! 欄位對齊工具（services::animal::pen_alignment）的 DB 整合測試。
//!
//! 斷言：apply 走 `AnimalService::update`（pen_location 與 pen_id 一致、舊欄／新欄
//! current_count 重算、留 ANIMAL_UPDATE 稽核）；容量不足的不改；重跑冪等；
//! 不動 status。

mod common;

use common::TestApp;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::services::pen_alignment::{
    apply_plan, build_plan_excluding, load_affected_pens, load_snapshots, LegacyRow, Outcome,
};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'pen align actor', true, false)"#,
    )
    .bind(id)
    .bind(format!("pa-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .execute(pool)
    .await
    .expect("insert user");
    id
}

async fn seed_zone(pool: &PgPool) -> Uuid {
    let (fid, bid, zid) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let s = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '對齊測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '對齊測試棟')",
    )
    .bind(bid)
    .bind(fid)
    .bind(format!("B{}", &s[..8]))
    .execute(pool)
    .await
    .expect("insert building");
    sqlx::query("INSERT INTO zones (id, building_id, code) VALUES ($1, $2, $3)")
        .bind(zid)
        .bind(bid)
        .bind(format!("Z{}", &s[..8]))
        .execute(pool)
        .await
        .expect("insert zone");
    zid
}

async fn seed_pen(pool: &PgPool, zone: Uuid, code: &str, capacity: i32) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO pens (id, zone_id, code, capacity) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(zone)
        .bind(code)
        .bind(capacity)
        .execute(pool)
        .await
        .expect("insert pen");
    id
}

async fn seed_animal(
    pool: &PgPool,
    by: Uuid,
    ear: &str,
    pen_id: Option<Uuid>,
    loc: Option<&str>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, birth_date, created_by, pen_id, pen_location)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', '2025-03-03', $3, $4, $5)"#,
    )
    .bind(id)
    .bind(ear)
    .bind(by)
    .bind(pen_id)
    .bind(loc)
    .execute(pool)
    .await
    .expect("insert animal");
    id
}

/// 測試前置：讓 current_count 等於實際在場數，之後只看工具有沒有維護它。
async fn recount(pool: &PgPool, pens: &[Uuid]) {
    for p in pens {
        sqlx::query(
            "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 \
             AND deleted_at IS NULL AND status NOT IN ('euthanized','sudden_death','transferred')) \
             WHERE id = $1",
        )
        .bind(p)
        .execute(pool)
        .await
        .expect("recount");
    }
}

async fn pen_row(pool: &PgPool, animal: Uuid) -> (Option<Uuid>, Option<String>, String) {
    sqlx::query_as("SELECT pen_id, pen_location, status::text FROM animals WHERE id = $1")
        .bind(animal)
        .fetch_one(pool)
        .await
        .expect("animal row")
}

async fn current_count(pool: &PgPool, pen: Uuid) -> i32 {
    sqlx::query_scalar("SELECT current_count FROM pens WHERE id = $1")
        .bind(pen)
        .fetch_one(pool)
        .await
        .expect("current_count")
}

fn row(old_pen: &str, ear: &str) -> LegacyRow {
    LegacyRow {
        old_pen: old_pen.into(),
        ear: ear.into(),
        birth: chrono::NaiveDate::from_ymd_opt(2025, 3, 3).expect("date"),
    }
}

#[tokio::test]
#[serial]
async fn apply_aligns_pen_via_update_path_and_is_idempotent() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let uid = seed_user(pool).await;
    let actor = ActorContext::User(CurrentUser {
        id: uid,
        email: "pa@test.local".into(),
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });

    // 欄 code 用 Q 開頭（不在 [A-G] 內，format_pen_location 原樣保留）且帶隨機尾碼，
    // 避免撞到 migration seed 與其他測試。
    let zone = seed_zone(pool).await;
    let suffix = Uuid::new_v4().to_string()[..4].to_uppercase();
    let c_new = format!("Q{suffix}1");
    let c_old = format!("Q{suffix}2");
    let c_full = format!("Q{suffix}3");
    let c_grp = format!("Q{suffix}4");
    let p_new = seed_pen(pool, zone, &c_new, 1).await;
    let p_old = seed_pen(pool, zone, &c_old, 2).await;
    let p_full = seed_pen(pool, zone, &c_full, 1).await;
    let p_grp = seed_pen(pool, zone, &c_grp, 2).await;

    // 耳號用非數字的隨機值：共用測試庫裡可能已有 801 之類的數字耳號，會被判成「多筆」。
    let (e1, e2, e3, e4) = (
        format!("PA{suffix}1"),
        format!("PA{suffix}2"),
        format!("PA{suffix}3"),
        format!("PA{suffix}4"),
    );
    let e_excl = format!("PA{suffix}X");
    // a：在 p_old，要搬去空欄 p_new
    let a = seed_animal(pool, uid, &e1, Some(p_old), Some(&c_old)).await;
    // b：文字對、pen_id 為 NULL，要補 pen_id
    let b = seed_animal(pool, uid, &e2, None, Some(&c_grp)).await;
    // c 佔滿 p_full（容量 1）；d 要去 p_full → 容量不足，不改
    let _c = seed_animal(pool, uid, &e3, Some(p_full), Some(&c_full)).await;
    let d = seed_animal(pool, uid, &e4, Some(p_old), Some(&c_old)).await;
    recount(pool, &[p_new, p_old, p_full, p_grp]).await;

    let legacy = vec![
        row(&c_new, &e1),
        row(&c_grp, &e2),
        row(&c_full, &e4),
        row(&c_new, &e_excl), // 指定排除
    ];
    let excluded = vec![e_excl.clone()];
    let snaps = load_snapshots(pool).await.expect("snapshots");
    let plan = build_plan_excluding(&legacy, &snaps.animals, &snaps.pens, &excluded);
    assert!(matches!(
        plan.items.iter().find(|i| i.ear == e4).map(|i| &i.outcome),
        Some(Outcome::Blocked { .. })
    ));
    assert!(matches!(
        plan.items
            .iter()
            .find(|i| i.ear == e_excl)
            .map(|i| &i.outcome),
        Some(Outcome::Manual(_))
    ));
    assert_eq!(plan.moves_in_order().len(), 2);

    let before = load_affected_pens(pool, &plan).await.expect("before");
    assert_eq!(before.len(), 3, "受影響欄 = p_old、p_new、p_grp");

    let out = apply_plan(pool, &actor, &plan).await;
    assert_eq!(out.applied, 2, "failed: {:?}", out.failed);
    assert!(out.failed.is_empty());

    // a：pen_id 與 pen_location 一致地指向新欄；status 不變
    let (pid, loc, status) = pen_row(pool, a).await;
    assert_eq!(pid, Some(p_new));
    assert_eq!(loc.as_deref(), Some(c_new.as_str()));
    assert_eq!(status, "unassigned");
    // b：補上 pen_id
    let (pid_b, loc_b, _) = pen_row(pool, b).await;
    assert_eq!(pid_b, Some(p_grp));
    assert_eq!(loc_b.as_deref(), Some(c_grp.as_str()));
    // d：被容量擋 → 原封不動
    let (pid_d, loc_d, _) = pen_row(pool, d).await;
    assert_eq!(pid_d, Some(p_old));
    assert_eq!(loc_d.as_deref(), Some(c_old.as_str()));

    // current_count 由 update 路徑重算：p_new=1、p_grp=1；p_old 少了 a、仍有 d → 1
    assert_eq!(current_count(pool, p_new).await, 1);
    assert_eq!(current_count(pool, p_grp).await, 1);
    assert_eq!(current_count(pool, p_old).await, 1);

    // 稽核：與一般欄位變更同一事件類別 ANIMAL_UPDATE
    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs \
         WHERE event_type = 'ANIMAL_UPDATE' AND entity_type = 'animal' AND entity_id = ANY($1)",
    )
    .bind(vec![a, b])
    .fetch_one(pool)
    .await
    .expect("audit count");
    assert_eq!(audits, 2);

    // 冪等：重跑後 a、b 已對齊，沒有東西可改（d 仍被擋）
    let snaps2 = load_snapshots(pool).await.expect("snapshots 2");
    let plan2 = build_plan_excluding(&legacy, &snaps2.animals, &snaps2.pens, &excluded);
    assert!(plan2.moves_in_order().is_empty());
    assert!(matches!(
        plan2.items.iter().find(|i| i.ear == e1).map(|i| &i.outcome),
        Some(Outcome::Aligned)
    ));
}
