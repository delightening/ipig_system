//! 血檢分析報表（`GET /api/v1/reports/blood-test-analysis`）的讀取範圍回歸測試。
//!
//! 守的規則：批次報表與單筆動物紀錄用同一個讀取判準
//! （`access::animal_read_boundary`）——無法跨計畫讀取的人，不論是不帶條件、
//! 填別人的計畫編號、還是填別人的動物 id，都拿不到自己計畫以外的資料。
//!
//! 放行對照組用**非管理員**的可跨計畫角色（VET），不用 admin：
//! admin 會繞過所有權限檢查，拿它當對照組測不出權限代碼寫錯。
//!
//! 角色代碼來源：`src/startup/permissions.rs` 的 `"PI"`、`"VET"` 區塊。
//! `breed` / `gender` 的值照抄 `tests/audit_chain_no_truncate.rs` 的動物 fixture。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

const PASSWORD: &str = "iPig$ecure1";

// ── fixtures ──────────────────────────────────────────────────────────────

async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("bt-scope-{label}-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("bt scope {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "bt-scope-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password(PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("bt scope {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    let assigned = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    // 角色代碼打錯時會安靜地寫入 0 列、使用者變成零角色；直接擋下。
    assert_eq!(
        assigned.rows_affected(),
        1,
        "角色代碼 {role_code} 不存在，fixture 無效"
    );
    let token = app.login(&email, PASSWORD).await.expect("login 應成功");
    (id, token)
}

/// 建立一份已核准的計畫，回傳它的 iacuc_no。
async fn seed_protocol(app: &TestApp, pi: Uuid) -> String {
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc_no = format!("IACUCBT-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'bt scope regression', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(format!("PBT-{unique}"))
    .bind(&iacuc_no)
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    iacuc_no
}

/// 建立一隻掛在 `iacuc_no` 底下的動物，並給牠一筆項目名為 `marker` 的血檢結果。
/// 回傳 animal id。
async fn seed_animal_with_blood_test(app: &TestApp, iacuc_no: &str, marker: &str) -> Uuid {
    let animal = Uuid::new_v4();
    let test = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no) \
         VALUES ($1, $2, 'LYD', 'male', CURRENT_DATE, $3)",
    )
    .bind(animal)
    // animals.ear_tag 是 varchar(10)：前綴 2 碼 + 8 碼 hex 剛好用滿。
    .bind(format!("BT{}", &Uuid::new_v4().simple().to_string()[..8]))
    .bind(iacuc_no)
    .execute(&app.db_pool)
    .await
    .expect("seed animal");
    sqlx::query(
        "INSERT INTO animal_blood_tests (id, animal_id, test_date, status) \
         VALUES ($1, $2, CURRENT_DATE, 'completed')",
    )
    .bind(test)
    .bind(animal)
    .execute(&app.db_pool)
    .await
    .expect("seed blood test");
    sqlx::query(
        "INSERT INTO animal_blood_test_items (id, blood_test_id, item_name, result_value) \
         VALUES ($1, $2, $3, '1.0')",
    )
    .bind(Uuid::new_v4())
    .bind(test)
    .bind(marker)
    .execute(&app.db_pool)
    .await
    .expect("seed blood test item");
    animal
}

/// 打報表並回傳結果中的 animal id 集合。`extra` 是附加的查詢字串（以 `&` 開頭或空）。
///
/// 每次都帶 `item_name=<marker>`：測試庫可能累積大量血檢列，而端點有筆數上限，
/// 不縮小範圍的話「沒看到某隻動物」可能只是被截斷，不是被授權擋下。
async fn report_animal_ids(app: &TestApp, token: &str, marker: &str, extra: &str) -> Vec<String> {
    let res = app
        .auth_get(
            &format!("/api/v1/reports/blood-test-analysis?item_name={marker}{extra}"),
            token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(status, 200, "報表應回 200，實得 {status}，body={body}");
    let rows: Vec<serde_json::Value> = serde_json::from_str(&body).expect("parse report rows");
    rows.iter()
        .filter_map(|r| r["animal_id"].as_str().map(str::to_owned))
        .collect()
}

struct Scene {
    marker: String,
    pi_token: String,
    own_animal: String,
    other_animal: String,
    other_iacuc_no: String,
}

async fn seed_scene(app: &TestApp) -> Scene {
    let marker = format!("scope-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (pi, pi_token) = seed_login_user(app, "pi", "PI").await;
    let other_pi = seed_user(app, "other-pi").await;

    let own_iacuc_no = seed_protocol(app, pi).await;
    let other_iacuc_no = seed_protocol(app, other_pi).await;

    let own_animal = seed_animal_with_blood_test(app, &own_iacuc_no, &marker).await;
    let other_animal = seed_animal_with_blood_test(app, &other_iacuc_no, &marker).await;

    Scene {
        marker,
        pi_token,
        own_animal: own_animal.to_string(),
        other_animal: other_animal.to_string(),
        other_iacuc_no,
    }
}

// ── 不帶條件：只看得到自己計畫的動物 ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_sees_only_own_protocol_animals_without_filter() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let ids = report_animal_ids(&app, &scene.pi_token, &scene.marker, "").await;
    assert!(
        ids.contains(&scene.own_animal),
        "對照：自己計畫的動物應在結果內（否則下面的「看不到」沒有意義），實得 {ids:?}"
    );
    assert!(
        !ids.contains(&scene.other_animal),
        "別人計畫的動物不得出現在結果內，實得 {ids:?}"
    );
}

// ── 指名別人的計畫編號 / 動物 id：不得越過邊界 ────────────────────────────

#[tokio::test]
#[serial]
async fn pi_cannot_reach_other_protocol_by_naming_it() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let by_iacuc = report_animal_ids(
        &app,
        &scene.pi_token,
        &scene.marker,
        &format!("&iacuc_no={}", scene.other_iacuc_no),
    )
    .await;
    assert!(
        by_iacuc.is_empty(),
        "填別人的計畫編號應得到空結果，實得 {by_iacuc:?}"
    );

    let by_animal = report_animal_ids(
        &app,
        &scene.pi_token,
        &scene.marker,
        &format!("&animal_id={}", scene.other_animal),
    )
    .await;
    assert!(
        by_animal.is_empty(),
        "填別人的動物 id 應得到空結果，實得 {by_animal:?}"
    );

    // 對照組：同形狀的請求指名自己的動物 → 拿得到。
    let own = report_animal_ids(
        &app,
        &scene.pi_token,
        &scene.marker,
        &format!("&animal_id={}", scene.own_animal),
    )
    .await;
    assert_eq!(own, vec![scene.own_animal.clone()]);
}

// ── 不關聯任何計畫：空結果，不是全部 ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_without_any_protocol_gets_empty_result() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;
    let (_, lonely_token) = seed_login_user(&app, "lonely", "PI").await;

    let ids = report_animal_ids(&app, &lonely_token, &scene.marker, "").await;
    assert!(
        ids.is_empty(),
        "不關聯任何計畫的人應得到空結果（空邊界不是不限），實得 {ids:?}"
    );
}

// ── 不得過度封鎖：可跨計畫讀取的角色照舊看得到全部 ────────────────────────

#[tokio::test]
#[serial]
async fn cross_protocol_reader_still_sees_everything() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;
    let (_, vet_token) = seed_login_user(&app, "vet", "VET").await;

    let ids = report_animal_ids(&app, &vet_token, &scene.marker, "").await;
    assert!(
        ids.contains(&scene.own_animal) && ids.contains(&scene.other_animal),
        "可跨計畫讀取的角色應同時看到兩份計畫的動物，實得 {ids:?}"
    );
}
