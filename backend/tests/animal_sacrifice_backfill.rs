//! 犧牲補登工具（services::animal::sacrifice_backfill）的 DB 整合測試。
//!
//! 斷言：缺 IACUC 者先經 update 補上；經 `upsert_sacrifice` 寫入犧牲紀錄並轉 euthanized
//! （稽核 SACRIFICE_UPSERT／IACUC_CHANGE）；補登後重算 pens.current_count；
//! 無法判定者（新舊皆無 IACUC）不動；重跑冪等。

mod common;

use common::TestApp;
use erp_backend::services::sacrifice_backfill::{
    apply, evaluate, load_actor, read_legacy_csv, Decision, RowOutcome,
};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

async fn seed_chair(pool: &PgPool) -> String {
    let id = Uuid::new_v4();
    let email = format!("sb-{}@test.local", &Uuid::new_v4().to_string()[..8]);
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'sacrifice backfill actor', true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .execute(pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'IACUC_CHAIR'",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("assign role");
    email
}

async fn seed_pen(pool: &PgPool, code: &str) -> Uuid {
    let (fid, bid, zid, pid) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let s = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO facilities (id, code, name) VALUES ($1, $2, '補登測試場')")
        .bind(fid)
        .bind(format!("F{}", &s[..8]))
        .execute(pool)
        .await
        .expect("insert facility");
    sqlx::query(
        "INSERT INTO buildings (id, facility_id, code, name) VALUES ($1, $2, $3, '補登測試棟')",
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
    sqlx::query("INSERT INTO pens (id, zone_id, code, capacity) VALUES ($1, $2, $3, 4)")
        .bind(pid)
        .bind(zid)
        .bind(code)
        .execute(pool)
        .await
        .expect("insert pen");
    pid
}

async fn seed_animal(
    pool: &PgPool,
    ear: &str,
    status: &str,
    iacuc: Option<&str>,
    pen: Option<(Uuid, &str)>,
    created_by: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, birth_date, status,
                                iacuc_no, created_by, pen_id, pen_location)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', '2025-03-03', $3::animal_status,
                   $4, $5, $6, $7)"#,
    )
    .bind(id)
    .bind(ear)
    .bind(status)
    .bind(iacuc)
    .bind(created_by)
    .bind(pen.map(|p| p.0))
    .bind(pen.map(|p| p.1))
    .execute(pool)
    .await
    .expect("insert animal");
    id
}

async fn status_iacuc(pool: &PgPool, id: Uuid) -> (String, Option<String>) {
    sqlx::query_as("SELECT status::text, iacuc_no FROM animals WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("animal row")
}

async fn sacrifice_ids(pool: &PgPool, animals: &[Uuid]) -> Vec<Uuid> {
    sqlx::query_scalar("SELECT id FROM animal_sacrifices WHERE animal_id = ANY($1)")
        .bind(animals)
        .fetch_all(pool)
        .await
        .expect("sacrifice ids")
}

async fn audit_count(pool: &PgPool, event: &str, ids: &[Uuid]) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = $1 AND entity_id = ANY($2)",
    )
    .bind(event)
    .bind(ids)
    .fetch_one(pool)
    .await
    .expect("audit count")
}

#[tokio::test]
#[serial]
async fn backfill_writes_sacrifice_via_service_path_and_is_idempotent() {
    let app = TestApp::spawn().await;
    let pool = &app.db_pool;
    let email = seed_chair(pool).await;
    let actor = load_actor(pool, &email).await.expect("actor");
    let uid = actor.require_user().expect("user").id;

    let suffix = Uuid::new_v4().to_string()[..4].to_uppercase();
    let code = format!("Q{suffix}1");
    let pen = seed_pen(pool, &code).await;
    let (e1, e2, e3) = (
        format!("SB{suffix}1"),
        format!("SB{suffix}2"),
        format!("SB{suffix}3"),
    );
    // a1：未分配、新系統沒有 IACUC → 先補 IACUC 再補登
    let a1 = seed_animal(pool, &e1, "unassigned", None, Some((pen, &code)), uid).await;
    // a2：實驗中、已有相同 IACUC → 只補登
    let a2 = seed_animal(
        pool,
        &e2,
        "in_experiment",
        Some("PIG-TEST-1"),
        Some((pen, &code)),
        uid,
    )
    .await;
    // a3：新舊都沒有 IACUC → 待人工，不動
    let a3 = seed_animal(pool, &e3, "unassigned", None, None, uid).await;
    sqlx::query("UPDATE pens SET current_count = 2 WHERE id = $1")
        .bind(pen)
        .execute(pool)
        .await
        .expect("seed count");

    let csv = format!(
        "ear,legacy_id,birth,iacuc,sac_date,zoletil,electro,bleed,other,sampling,sampling_other,blood_ml\n\
         {e1},1,2025-03-03,PIG-TEST-1,2026-06-01,4,0,1,KCl,\"心, 脾\",,\n\
         {e2},2,2025-03-03,PIG-TEST-1,2026-06-02,,0,0,KCl,,,\n\
         {e3},3,2025-03-03,,2026-06-03,,0,0,KCl,,,\n"
    );
    let rows = read_legacy_csv(csv.as_bytes()).expect("csv");
    let items = evaluate(pool, rows).await.expect("evaluate");
    assert!(matches!(
        &items[0].decision,
        Decision::Ready { set_iacuc: Some(s), .. } if s == "PIG-TEST-1"
    ));
    assert!(matches!(
        &items[1].decision,
        Decision::Ready {
            set_iacuc: None,
            ..
        }
    ));
    assert!(matches!(&items[2].decision, Decision::Manual(_)));

    let out = apply(pool, &actor, &items).await;
    assert!(matches!(out[0], RowOutcome::Done), "{:?}", out[0]);
    assert!(matches!(out[1], RowOutcome::Done), "{:?}", out[1]);
    assert!(matches!(out[2], RowOutcome::Skipped));

    let (s1, i1) = status_iacuc(pool, a1).await;
    assert_eq!(s1, "euthanized");
    assert_eq!(i1.as_deref(), Some("PIG-TEST-1"));
    assert_eq!(status_iacuc(pool, a2).await.0, "euthanized");
    assert_eq!(status_iacuc(pool, a3).await.0, "unassigned");

    // 犧牲紀錄內容
    let (date, zol, bleed, sampling, confirmed): (
        chrono::NaiveDate,
        Option<String>,
        bool,
        Option<String>,
        bool,
    ) = sqlx::query_as(
        "SELECT sacrifice_date, zoletil_dose, method_bloodletting, sampling, confirmed_sacrifice \
         FROM animal_sacrifices WHERE animal_id = $1",
    )
    .bind(a1)
    .fetch_one(pool)
    .await
    .expect("sacrifice row");
    assert_eq!(date.to_string(), "2026-06-01");
    assert_eq!(zol.as_deref(), Some("4"));
    assert!(bleed && confirmed);
    assert_eq!(sampling.as_deref(), Some("心, 脾"));

    // 稽核：SACRIFICE_UPSERT 的 entity 是犧牲紀錄（animal_sacrifice）；a1 另有 IACUC_CHANGE（entity 是動物）
    let sac_ids = sacrifice_ids(pool, &[a1, a2]).await;
    assert_eq!(sac_ids.len(), 2);
    assert_eq!(audit_count(pool, "SACRIFICE_UPSERT", &sac_ids).await, 2);
    assert_eq!(audit_count(pool, "IACUC_CHANGE", &[a1]).await, 1);

    // 補登後 current_count 被重算：兩隻都離場 → 0
    let cc: i32 = sqlx::query_scalar("SELECT current_count FROM pens WHERE id = $1")
        .bind(pen)
        .fetch_one(pool)
        .await
        .expect("current_count");
    assert_eq!(cc, 0);

    // 冪等：重跑 → a1、a2 已完成、a3 仍待人工；不再寫入
    let rows2 = read_legacy_csv(csv.as_bytes()).expect("csv 2");
    let items2 = evaluate(pool, rows2).await.expect("evaluate 2");
    assert!(matches!(items2[0].decision, Decision::AlreadyDone { .. }));
    assert!(matches!(items2[1].decision, Decision::AlreadyDone { .. }));
    assert!(matches!(items2[2].decision, Decision::Manual(_)));
    let out2 = apply(pool, &actor, &items2).await;
    assert!(out2.iter().all(|o| matches!(o, RowOutcome::Skipped)));
    assert_eq!(audit_count(pool, "SACRIFICE_UPSERT", &sac_ids).await, 2);
}
