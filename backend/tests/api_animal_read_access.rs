//! 驗收/回歸測試：R70 follow-up — 動物紀錄讀寫存取分層 + 「查無此豬」404。
//!
//! 規格（依使用者裁定的權限矩陣）：
//! - 內部 staff（具 `animal.animal.view_all`，如 EXPERIMENT_STAFF）→ 可**跨計畫讀取**
//!   動物紀錄（`require_animal_read_access`）。
//! - **計畫綁定紀錄**（手術／犧牲／病理／照護／獸醫單／轉讓）的**寫入**仍限自己計畫
//!   （`require_animal_access`，判斷鑰匙為 `has_protocol_view_all`、不含 `animal.animal.view_all`）。
//! - **基礎紀錄**（體重／疫苗／血檢／觀察／猝死）的讀與寫共用 `require_animal_read_access`，
//!   故內部 staff 亦可**跨計畫寫入**基礎紀錄（沿用 #713 免計畫放寬 + 符合「整場」矩陣）。
//! - PI / CLIENT（僅 `view_project`、非成員）→ 讀取亦限自己計畫（收緊 #713「存在即放行」）。
//! - 任何角色（含 view_all）對**不存在的動物**一律得 `NotFound("Animal not found")`。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::CurrentUser;
use erp_backend::services::access;
use erp_backend::services::AnimalService;
use erp_backend::AppError;

/// 以指定 roles / permissions 組出 CurrentUser（id 不必對應真實 users 列，
/// 存取守衛僅以 roles/permissions 與 protocol 成員關係判斷）。
fn make_user(id: Uuid, roles: &[&str], perms: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "read-access@test.local".into(),
        roles: roles.iter().map(|s| s.to_string()).collect(),
        permissions: perms.iter().map(|s| s.to_string()).collect(),
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
    .bind(format!("ra-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("read access {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建立 APPROVED 計畫，`pi_user_id = pi_id`。回傳 (protocol_id, iacuc_no)。
async fn seed_protocol(app: &TestApp, pi_id: Uuid) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc = format!("IACUC-RA-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'read access test', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR-RA-{unique}"))
    .bind(&iacuc)
    .bind(pi_id)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    (id, iacuc)
}

/// 建立一隻動物（指派 `iacuc_no`）。回傳 animal_id。
async fn seed_animal(app: &TestApp, iacuc_no: &str, created_by: Uuid) -> Uuid {
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, created_by)
           VALUES ($1, $2, 'in_experiment'::animal_status, 'miniature', 'male', '2024-06-01', $3, $4)"#,
    )
    .bind(aid)
    .bind(format!("RA{}", &aid.to_string()[..6]))
    .bind(iacuc_no)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

// ── 讀取放寬：具 animal.animal.view_all 的內部 staff（非計畫成員）可跨計畫讀取 ──────────
#[tokio::test]
#[serial]
async fn read_access_allows_cross_project_for_animal_view_all() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;

    // EXPERIMENT_STAFF 模型：非該計畫成員，但具 animal.animal.view_all
    let staff = make_user(Uuid::new_v4(), &[], &["animal.animal.view_all"]);
    access::require_animal_read_access(&app.db_pool, &staff, animal_id)
        .await
        .expect("具 animal.animal.view_all 的內部 staff 應可跨計畫讀取動物");
}

// ── 讀取收緊：無 view_all 的非成員（PI/CLIENT 模型）不可跨計畫讀取（#713 存在即放行已收緊）─
#[tokio::test]
#[serial]
async fn read_access_rejects_cross_project_for_non_view_all() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "owner-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;

    let outsider = make_user(Uuid::new_v4(), &[], &[]);
    let err = access::require_animal_read_access(&app.db_pool, &outsider, animal_id)
        .await
        .expect_err("無 view_all 的非成員不應跨計畫讀取");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（PI/CLIENT 僅限自己計畫），實得：{err:?}"
    );
}

// ── 正向：計畫 PI（非 view_all）讀取自己計畫的動物 → Ok ────────────────────────────────
#[tokio::test]
#[serial]
async fn read_access_grants_protocol_member() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;

    access::require_animal_read_access(&app.db_pool, &make_user(pi, &[], &[]), animal_id)
        .await
        .expect("計畫 PI 應可讀取自己計畫的動物");
}

// ── 計畫綁定寫入未放寬：具 animal.animal.view_all 的 staff，require_animal_access 仍須計畫成員 → Forbidden ─
// （基礎紀錄寫入走 require_animal_read_access、允許跨計畫，已由上方 read_access 跨計畫測試涵蓋。）
#[tokio::test]
#[serial]
async fn write_access_still_rejects_cross_project_for_animal_view_all() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "owner-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;

    let staff = make_user(Uuid::new_v4(), &[], &["animal.animal.view_all"]);
    let err = access::require_animal_access(&app.db_pool, &staff, animal_id)
        .await
        .expect_err("animal.animal.view_all 僅放寬讀取；寫入仍須計畫成員");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（寫入限自己計畫），實得：{err:?}"
    );
}

// ── 查無此豬：view_all 角色對不存在的動物 → NotFound（修復前短路後回空集合/200）──────────
#[tokio::test]
#[serial]
async fn require_animal_access_missing_animal_is_not_found_for_view_all() {
    let app = TestApp::spawn().await;
    // VET 為 view_all 角色；不存在的 animal_id 應得 NotFound（查無此豬）
    let vet = make_user(Uuid::new_v4(), &["VET"], &[]);
    let err = access::require_animal_access(&app.db_pool, &vet, Uuid::new_v4())
        .await
        .expect_err("view_all 角色對不存在動物應得 NotFound");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應為 NotFound（查無此豬），實得：{err:?}"
    );
}

// ── 查無此豬：具 animal.animal.view_all 者對不存在的動物 → NotFound ───────────────────────
#[tokio::test]
#[serial]
async fn read_access_missing_animal_is_not_found_for_animal_view_all() {
    let app = TestApp::spawn().await;
    let staff = make_user(Uuid::new_v4(), &[], &["animal.animal.view_all"]);
    let err = access::require_animal_read_access(&app.db_pool, &staff, Uuid::new_v4())
        .await
        .expect_err("具 animal.animal.view_all 者對不存在動物應得 NotFound");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應為 NotFound（查無此豬），實得：{err:?}"
    );
}

/// 直接以 SQL 植入觀察紀錄（繞過 service guard）。
/// `record_type` 取 `record_type` enum 的真值（`migrations/001_enums.sql:35`）。
async fn seed_observation_direct(app: &TestApp, animal_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animal_observations \
         (id, animal_id, event_date, record_type, content, created_by, created_at, updated_at) \
         VALUES ($1, $2, '2024-01-01', 'observation'::record_type, 'scoped guard test', $3, NOW(), NOW())",
    )
    .bind(id)
    .bind(animal_id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert observation directly");
    id
}

// ── R94-4：`Scoped::<AnimalRead|AnimalWrite>::from_observation` 的強度必須不同 ──
//    這對測試存在的唯一理由是**釘住讀寫強度**：若有人把 Write 變體內部改成
//    `require_animal_read_access`（或呼叫端把 turbofish 從 Write 誤寫成 Read），
//    下面第二個斷言會立刻紅。缺這對測試時，強度互換不會被任何既有測試抓到。
#[tokio::test]
#[serial]
async fn from_observation_read_allows_view_all_but_write_rejects() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "fo-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;
    let obs_id = seed_observation_direct(&app, animal_id, pi).await;

    // EXPERIMENT_STAFF 模型：非該計畫成員，但具 animal.animal.view_all
    let staff = make_user(Uuid::new_v4(), &[], &["animal.animal.view_all"]);

    // 讀取變體：跨計畫放行，且解析出的 id 必須是 animal_id（非 observation_id）
    let read_scope =
        access::Scoped::<access::AnimalRead>::from_observation(&app.db_pool, &staff, obs_id)
            .await
            .expect("AnimalRead::from_observation 應對 view_all 跨計畫放行");
    assert_eq!(
        read_scope.id(),
        animal_id,
        "Scoped 的 id 應為 animal_id，下游 service 依賴此語意"
    );

    // 寫入變體：同一使用者、同一紀錄必須被擋——這是讀寫強度差異的斷言點。
    // 用 `.err().expect()` 而非 `.expect_err()`：後者要求 Ok 型別實作 Debug，
    // 而 `Scoped<T>` 未實作（已授權證明不該被隨手印出）。
    let err = access::Scoped::<access::AnimalWrite>::from_observation(&app.db_pool, &staff, obs_id)
        .await
        .err()
        .expect("AnimalWrite::from_observation 不應對非計畫成員放行");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（寫入限自己計畫），實得：{err:?}"
    );
}

// ── R94-4：不存在的 observation → NotFound，且訊息與 service 層一致 ──
#[tokio::test]
#[serial]
async fn from_observation_missing_record_is_not_found() {
    let app = TestApp::spawn().await;
    let vet = make_user(Uuid::new_v4(), &["VET"], &[]);

    let err =
        access::Scoped::<access::AnimalRead>::from_observation(&app.db_pool, &vet, Uuid::new_v4())
            .await
            .err()
            .expect("不存在的觀察紀錄應得 NotFound");

    // 訊息需與 `services/animal/observation.rs` 的 get_by_id 同字串——
    // access.rs 刻意讓各分支訊息一致，避免成為角色探測 / IDOR probe 訊號。
    match err {
        AppError::NotFound(msg) => assert_eq!(
            msg, "Observation not found",
            "訊息應與 service 層 get_by_id 一致"
        ),
        other => panic!("預期 NotFound，實得 {other:?}"),
    }
}

/// 直接以 SQL 植入手術紀錄（繞過 service guard），供 copy 來源 IDOR 測試用。
async fn seed_surgery_direct(app: &TestApp, animal_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animal_surgeries \
         (id, animal_id, is_first_experiment, surgery_date, surgery_site, created_by, created_at, updated_at) \
         VALUES ($1, $2, false, '2024-01-01', 'test site', $3, NOW(), NOW())",
    )
    .bind(id)
    .bind(animal_id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert surgery directly");
    id
}

// ── copy 來源 IDOR 防護（R75）：`copy_animal_surgery` 由 `req.source_id` 反查來源動物後，
//    對來源走 `require_animal_read_access`。無 view_all 的他計畫人員被擋（防跨計畫複製其內容）；
//    具 `animal.animal.view_all` 的全場人員仍放行（保住全場 copy）。──
#[tokio::test]
#[serial]
async fn surgery_copy_source_requires_read_access_on_source_animal() {
    let app = TestApp::spawn().await;
    let owner = seed_user(&app, "copy-src-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, owner).await;
    let source_animal = seed_animal(&app, &iacuc, owner).await;
    let surgery_id = seed_surgery_direct(&app, source_animal, owner).await;

    // handler 第一步：由 source surgery id 反查來源動物
    let resolved = access::get_surgery_animal_id(&app.db_pool, surgery_id)
        .await
        .expect("resolve source surgery animal");
    assert_eq!(resolved, source_animal, "來源手術應反查回其所屬動物");

    // 具 copy 權但無 view_all、且非來源計畫成員者 → 來源讀取檢查擋下（Forbidden）
    let outsider = make_user(Uuid::new_v4(), &[], &["animal.record.copy"]);
    let err = access::require_animal_read_access(&app.db_pool, &outsider, resolved)
        .await
        .expect_err("無 view_all 的非成員不應通過 copy 來源讀取檢查");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden（防跨計畫複製來源內容），實得：{err:?}"
    );

    // 具 animal.animal.view_all 的全場人員 → 來源讀取放行（保住全場 copy 訴求）
    let staff = make_user(
        Uuid::new_v4(),
        &[],
        &["animal.record.copy", "animal.animal.view_all"],
    );
    access::require_animal_read_access(&app.db_pool, &staff, resolved)
        .await
        .expect("具 animal.animal.view_all 的全場人員應可通過 copy 來源讀取檢查");
}

// ── mark_vet_read 寫對欄位（回歸）：舊版誤寫不存在的 animals.vet_read_at → SQL 失敗；
//    應更新列表所讀的 vet_last_viewed_at。──
#[tokio::test]
#[serial]
async fn mark_vet_read_updates_vet_last_viewed_at() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "vr-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;

    // 前置狀態：新動物尚未被獸醫檢視，vet_last_viewed_at 應為 NULL
    // （避免「mark 前就已非 NULL」造成 assert 偽陽性）。
    let before: bool =
        sqlx::query_scalar("SELECT vet_last_viewed_at IS NULL FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch pre-state vet_last_viewed_at");
    assert!(before, "前置：新動物 vet_last_viewed_at 應為 NULL");

    // 具 animal.animal.view_all 的全場人員可取得讀取 scope
    let staff = make_user(Uuid::new_v4(), &[], &["animal.animal.view_all"]);
    let scope = access::Scoped::<access::AnimalRead>::authorize(&app.db_pool, &staff, animal_id)
        .await
        .expect("authorize read scope");

    // 不可因參照不存在欄位而 SQL 失敗
    AnimalService::mark_vet_read(&app.db_pool, scope)
        .await
        .expect("mark_vet_read 應成功");

    let set: bool =
        sqlx::query_scalar("SELECT vet_last_viewed_at IS NOT NULL FROM animals WHERE id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch vet_last_viewed_at");
    assert!(
        set,
        "mark_vet_read 應設定 animals.vet_last_viewed_at（列表所讀欄位）"
    );
}

/// 直接以 SQL 植入觀察紀錄（繞過 service guard）。
/// `record_type` 取 `record_type` enum 的真值（`migrations/001_enums.sql:35`：
/// `'abnormal' / 'experiment' / 'observation'`）。
async fn seed_observation_direct(app: &TestApp, animal_id: Uuid, created_by: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO animal_observations \
         (id, animal_id, event_date, record_type, content, created_by, created_at, updated_at) \
         VALUES ($1, $2, '2024-01-01', 'observation'::record_type, 'soft delete guard test', $3, NOW(), NOW())",
    )
    .bind(id)
    .bind(animal_id)
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert observation directly");
    id
}

// ── R94-3：`get_observation_animal_id` **刻意不過濾軟刪除**（回歸鎖）──
//    它是 IDOR 守衛而非存在性檢查：`get_observation_versions` 先靠它授權、再查
//    `record_versions` 表（不受 animal_observations.deleted_at 影響），所以已刪紀錄的
//    版本歷史必須仍查得到——這正是軟刪除保留稽核軌跡的用途，且與 surgery 行為一致
//    （`get_surgery_versions` 走的 `AnimalSurgeryService::get_by_id` 同樣不過濾）。
//
//    本測試存在的理由：2026-08-11 曾誤加 `AND deleted_at IS NULL`，把「已刪紀錄的
//    版本歷史」變成 404。若日後有人再次加上該過濾，下面第二段斷言會立刻紅。
#[tokio::test]
#[serial]
async fn get_observation_animal_id_keeps_resolving_after_soft_delete() {
    let app = TestApp::spawn().await;
    let pi = seed_user(&app, "obs-sd-pi").await;
    let (_pid, iacuc) = seed_protocol(&app, pi).await;
    let animal_id = seed_animal(&app, &iacuc, pi).await;
    let obs_id = seed_observation_direct(&app, animal_id, pi).await;

    // 前置：未刪除時可正常解析（確保後續斷言不是被 fixture 建錯所掩蓋）
    let resolved = access::get_observation_animal_id(&app.db_pool, obs_id)
        .await
        .expect("未刪除的觀察紀錄應可解析出 animal_id");
    assert_eq!(resolved, animal_id, "前置：應解析回該筆觀察紀錄的動物");

    sqlx::query("UPDATE animal_observations SET deleted_at = NOW() WHERE id = $1")
        .bind(obs_id)
        .execute(&app.db_pool)
        .await
        .expect("soft delete observation");

    // 核心斷言：軟刪除後**仍應解析得到**，稽核軌跡（版本歷史）才讀得到
    let after_delete = access::get_observation_animal_id(&app.db_pool, obs_id)
        .await
        .expect("軟刪除後仍應解析出 animal_id（版本歷史等稽核路徑依賴此行為）");
    assert_eq!(
        after_delete, animal_id,
        "軟刪除不應改變歸屬解析結果；需要擋已刪紀錄的地方由 service 層負責"
    );

    // 對照：真正不存在的 id 才回 NotFound，訊息與 service 層 get_by_id 同字串
    // （access.rs 刻意讓各分支訊息一致，避免成為角色探測 / IDOR probe 訊號）。
    let err = access::get_observation_animal_id(&app.db_pool, Uuid::new_v4())
        .await
        .expect_err("不存在的觀察紀錄才應回 NotFound");
    match err {
        AppError::NotFound(msg) => assert_eq!(
            msg, "Observation not found",
            "訊息應與 service 層 get_by_id 一致"
        ),
        other => panic!("預期 NotFound，實得 {other:?}"),
    }
}
