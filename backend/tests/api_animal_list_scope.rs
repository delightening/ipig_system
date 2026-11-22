//! 動物列表（`GET /api/v1/animals`）的讀取範圍回歸測試。
//!
//! 守的規則：沒有 `animal.animal.view_all` 的人，列表只回自己可存取之計畫底下的動物——
//! 別人計畫的動物與未掛計畫的動物都不回，填別人的計畫編號也拿不到；
//! 分頁的總數要與實際可見的列一致。
//!
//! 放行對照組用非管理員的全場角色（VET），不用 admin（admin 繞過所有權限檢查）。
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
    .bind(format!("al-scope-{label}-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("al scope {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "al-scope-{label}-{}@test.local",
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
    .bind(format!("al scope {label}"))
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

async fn seed_protocol(app: &TestApp, pi: Uuid) -> String {
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc_no = format!("IACUCAL-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'animal list scope', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(format!("PAL-{unique}"))
    .bind(&iacuc_no)
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    iacuc_no
}

async fn seed_animal(app: &TestApp, ear_tag: &str, iacuc_no: Option<&str>) -> String {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animals (id, ear_tag, breed, gender, entry_date, iacuc_no) \
         VALUES ($1, $2, 'LYD', 'male', CURRENT_DATE, $3)",
    )
    .bind(id)
    .bind(ear_tag)
    .bind(iacuc_no)
    .execute(&app.db_pool)
    .await
    .expect("seed animal");
    id.to_string()
}

struct Scene {
    /// 三隻動物耳號的共同前綴，當 `keyword` 用，把查詢範圍縮到這個 scene。
    keyword: String,
    pi_token: String,
    own_animal: String,
    other_animal: String,
    unassigned_animal: String,
    other_iacuc_no: String,
}

async fn seed_scene(app: &TestApp) -> Scene {
    // animals.ear_tag 是 varchar(10)：1 碼前綴 + 6 碼 hex + 1 碼區分 = 8。
    let keyword = format!("L{}", &Uuid::new_v4().simple().to_string()[..6]);
    let (pi, pi_token) = seed_login_user(app, "pi", "PI").await;
    let other_pi = seed_user(app, "other-pi").await;

    let own_iacuc_no = seed_protocol(app, pi).await;
    let other_iacuc_no = seed_protocol(app, other_pi).await;

    let own_animal = seed_animal(app, &format!("{keyword}a"), Some(&own_iacuc_no)).await;
    let other_animal = seed_animal(app, &format!("{keyword}b"), Some(&other_iacuc_no)).await;
    let unassigned_animal = seed_animal(app, &format!("{keyword}u"), None).await;

    Scene {
        keyword,
        pi_token,
        own_animal,
        other_animal,
        unassigned_animal,
        other_iacuc_no,
    }
}

/// 打列表並回傳 (animal id 集合, total)。`extra` 是附加的查詢字串（以 `&` 開頭或空）。
async fn list_ids(app: &TestApp, token: &str, keyword: &str, extra: &str) -> (Vec<String>, i64) {
    let res = app
        .auth_get(&format!("/api/v1/animals?keyword={keyword}{extra}"), token)
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(status, 200, "列表應回 200，實得 {status}，body={body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("parse list response");
    let mut ids: Vec<String> = json["data"]
        .as_array()
        .expect("data 應為陣列")
        .iter()
        .filter_map(|a| a["id"].as_str().map(str::to_owned))
        .collect();
    ids.sort();
    let total = json["total"].as_i64().expect("total 應為整數");
    (ids, total)
}

// ── 只看得到自己計畫的動物，分頁總數一致 ──────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_list_contains_only_own_protocol_animals() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    // 非分頁
    let (ids, total) = list_ids(&app, &scene.pi_token, &scene.keyword, "").await;
    assert_eq!(
        ids,
        vec![scene.own_animal.clone()],
        "只應看到自己計畫的那一隻"
    );
    assert_eq!(total, 1);

    // 分頁：總數必須是可見的列數，不是全場符合條件的列數
    let (ids, total) = list_ids(&app, &scene.pi_token, &scene.keyword, "&page=1&per_page=50").await;
    assert_eq!(ids, vec![scene.own_animal.clone()]);
    assert_eq!(total, 1, "分頁總數應與可見列數一致");
}

// ── 指名別人的計畫編號：不得越過邊界 ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_cannot_list_other_protocol_by_naming_it() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let (ids, total) = list_ids(
        &app,
        &scene.pi_token,
        &scene.keyword,
        &format!("&iacuc_no={}&page=1&per_page=50", scene.other_iacuc_no),
    )
    .await;
    assert!(ids.is_empty(), "填別人的計畫編號應得到空結果，實得 {ids:?}");
    assert_eq!(total, 0);
}

// ── 不關聯任何計畫：空結果 ────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_without_any_protocol_gets_empty_list() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;
    let (_, lonely_token) = seed_login_user(&app, "lonely", "PI").await;

    let (ids, total) = list_ids(&app, &lonely_token, &scene.keyword, "").await;
    assert!(
        ids.is_empty(),
        "不關聯任何計畫的人應得到空結果，實得 {ids:?}"
    );
    assert_eq!(total, 0);
}

// ── 不得過度封鎖：全場角色照舊看得到全部（含未掛計畫的） ──────────────────

#[tokio::test]
#[serial]
async fn facility_wide_reader_still_sees_everything() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;
    let (_, vet_token) = seed_login_user(&app, "vet", "VET").await;

    let (ids, total) = list_ids(&app, &vet_token, &scene.keyword, "&page=1&per_page=50").await;
    let mut expected = vec![
        scene.own_animal.clone(),
        scene.other_animal.clone(),
        scene.unassigned_animal.clone(),
    ];
    expected.sort();
    assert_eq!(ids, expected, "全場角色應看到三隻");
    assert_eq!(total, 3);
}
