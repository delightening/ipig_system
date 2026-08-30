//! AUP 計畫結案守門回歸測試。
//!
//! 需求：計畫結案（`CLOSED`）前，該 IACUC 下所有動物必須皆已離場終態
//! （安樂死 / 猝死 / 已轉讓）；若仍有存活動物（未分配 / 實驗中 / 實驗完成），
//! 結案應被拒絕。存活判定對齊 `AnimalStatus::is_active_in_facility`
//! （`status NOT IN euthanized/sudden_death/transferred`）。
//! 守門實作於 `services/protocol/status.rs::change_status_tx`。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{AnimalStatus, ChangeStatusRequest, ProtocolStatus};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "protocol_close_animal_guard",
};

/// R89-8：以指定 pi_user_id / study_director_user_id 建立一個 APPROVED 計畫，
/// 回傳 protocol_id。與 `seed_approved_protocol` 的差異是可控制擁有人身分，
/// 用於驗證「計畫擁有人結案窄縫」的本人判斷。
async fn seed_approved_protocol_owned_by(
    app: &TestApp,
    pi_user_id: Uuid,
    study_director_user_id: Option<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, study_director_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'r89-8 close own guard', 'APPROVED'::protocol_status, $4, $5, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR-{unique}"))
    .bind(format!("IACUC-R89-8-{unique}"))
    .bind(pi_user_id)
    .bind(study_director_user_id)
    .execute(&app.db_pool)
    .await
    .expect("insert approved protocol owned by given user");
    id
}

/// R89-8：帶指定角色/權限的測試 actor（不帶 `aup.protocol.change_status`，
/// 只帶 `aup.protocol.close_own`，模擬 PI/SD 登入後的權限快照）。
fn close_own_actor(user_id: Uuid) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: user_id,
        email: "r89-8-test@example.com".into(),
        roles: vec!["PI".into()],
        permissions: vec!["aup.protocol.close_own".into()],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// R89-8（Qodo/PR-Agent review 發現）：擁有人身分成立、但權限快照裡連
/// `aup.protocol.close_own` 都沒有的 actor——模擬 `ProtocolService::change_status`
/// 被 handler 以外的路徑直接呼叫（例如 `services/mcp/tools.rs`、
/// `services/protocol/ai_review.rs`）時，service 層若只驗擁有人+狀態、不驗權限本身，
/// 會讓這類呼叫繞過 close_own 檢查。
fn owner_without_close_own_actor(user_id: Uuid) -> ActorContext {
    ActorContext::User(CurrentUser {
        id: user_id,
        email: "r89-8-no-perm-test@example.com".into(),
        roles: vec![],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    })
}

/// 建立一個 APPROVED 且帶指定 iacuc_no 的計畫，回傳 (protocol_id, iacuc_no)。
async fn seed_approved_protocol(app: &TestApp) -> (Uuid, String) {
    let admin_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    let iacuc_no = format!("IACUC-CLOSE-{unique}");
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'close animal guard', 'APPROVED'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(format!("PR-{unique}"))
    .bind(&iacuc_no)
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("insert approved protocol");
    (id, iacuc_no)
}

/// 於指定 iacuc_no 下建立一隻指定狀態的動物。
async fn seed_animal(app: &TestApp, iacuc_no: &str, status: AnimalStatus) {
    let unique = &Uuid::new_v4().to_string()[..6];
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no)
           VALUES (gen_random_uuid(), $1, $2, 'miniature', 'male', NOW(), $3)"#,
    )
    .bind(format!("E{unique}"))
    .bind(status)
    .bind(iacuc_no)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
}

/// 於指定 protocol 下建立一隻「已預約但未分配」動物：只設 reserved_protocol_id，
/// iacuc_no 保持 NULL（預約不寫 iacuc_no，僅正式分配才寫）。用於驗證守門涵蓋 earmark。
async fn seed_reserved_animal(app: &TestApp, protocol_id: Uuid) {
    let unique = &Uuid::new_v4().to_string()[..6];
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, reserved_protocol_id)
           VALUES (gen_random_uuid(), $1, $2, 'miniature', 'male', NOW(), $3)"#,
    )
    .bind(format!("R{unique}"))
    .bind(AnimalStatus::Unassigned)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("insert reserved animal");
}

/// 寫一張合格的結案簽章，回傳其 id。
///
/// 欄位值對齊 `services/protocol/closure.rs::dual_signature_ready` 的 7 條件：
/// `entity_type='protocol_closure'`（**不是** 核准簽章的 `'protocol'`）、
/// `entity_id` 為 protocol id 的字串、`signature_type='CONFIRM'`、`is_valid=true`。
async fn insert_closure_signature(app: &TestApp, protocol_id: Uuid, signer_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO electronic_signatures
             (id, entity_type, entity_id, signer_id, signature_type,
              content_hash, signature_data, signature_method, meaning, is_valid)
           VALUES ($1, 'protocol_closure', $2, $3, 'CONFIRM', 'hash', 'data', 'password',
                   'CONFIRM'::signature_meaning, true)"#,
    )
    .bind(id)
    .bind(protocol_id.to_string())
    .bind(signer_id)
    .execute(&app.db_pool)
    .await
    .expect("insert closure signature");
    id
}

/// 讓計畫具備齊備且有效的結案雙簽。
///
/// ⚠️ **為什麼本檔的 `close_own` 案例需要這個。**
/// 結案雙簽（#38）之後，`close_own` 擁有人檢查**不再足以結案**——它降級為
/// 「誰可以嘗試」的粗篩，真正放行的是雙簽 gate。少了這個 fixture，
/// R89-8 那幾條案例會倒在「需要 PI 與 SD 雙方各自簽章」，
/// 驗不到它們真正的受測點（擁有人身分與 `close_own` 權限）。
///
/// 雙簽本身的 7 條件由 `api_protocol_closure_dual_sign.rs` 逐條各一支測試守，
/// 本檔不重複驗，只需要一組合格的簽章把案例送到下一關。
async fn seed_closure_dual_signatures(
    app: &TestApp,
    protocol_id: Uuid,
    pi_user_id: Uuid,
    sd_user_id: Uuid,
) {
    assert_ne!(
        pi_user_id, sd_user_id,
        "雙簽的兩張簽章必須是不同人（gate 條件 7），fixture 給同一人會讓案例失去意義"
    );
    let pi_sig = insert_closure_signature(app, protocol_id, pi_user_id).await;
    let sd_sig = insert_closure_signature(app, protocol_id, sd_user_id).await;
    sqlx::query(
        "UPDATE protocols SET close_pi_signature_id = $2, close_sd_signature_id = $3 WHERE id = $1",
    )
    .bind(protocol_id)
    .bind(pi_sig)
    .bind(sd_sig)
    .execute(&app.db_pool)
    .await
    .expect("link closure signatures to protocol");
}

/// 建一個測試用帳號，回傳 id。
async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', $3, true, false)",
    )
    .bind(id)
    .bind(format!("r89-8-{label}-{}@test.local", &id.to_string()[..8]))
    .bind(format!("r89-8 {label}"))
    .execute(&app.db_pool)
    .await
    .expect("seed user");
    id
}

/// ⚠️ `remark` 是必填的，不是裝飾。
///
/// 結案雙簽（#38）上線後，`SYSTEM_TEST` 這類 admin/System actor 結案走的是
/// `status.rs` 的「繞過雙簽」那條路，而那條路**強制要求具名理由**
/// （理由會落 audit 與 `protocol_activities`，讓稽核查得出哪些結案是繞過的）。
/// 留 `None` 的話案例會倒在「繞過結案雙簽必須填寫理由」，
/// **根本走不到本檔真正要驗的動物守門**——而錯誤訊息看起來會像動物守門壞了。
fn close_req() -> ChangeStatusRequest {
    ChangeStatusRequest {
        to_status: ProtocolStatus::Closed,
        remark: Some("動物守門回歸測試：以系統身分繞過結案雙簽".to_string()),
        reviewer_ids: None,
        vet_id: None,
    }
}

/// 斷言這個錯誤**來自動物守門**，而不只是「某個 BusinessRule」。
///
/// ⚠️ 只寫 `matches!(err, AppError::BusinessRule(_))` 不夠。結案路徑上排在動物守門
/// **之前**的還有結案雙簽 gate（`status.rs`，#38），它吐的也是 `BusinessRule`。
/// 只驗型別的話，「案例根本沒走到動物守門就被雙簽擋下」會被判為通過——
/// 本檔就會在完全沒有守到動物守門的情況下全綠。
fn assert_blocked_by_animal_guard(err: &AppError) {
    match err {
        AppError::BusinessRule(msg) => assert!(
            msg.contains("存活動物"),
            "應被動物守門擋下，實得另一個 BusinessRule：{msg}"
        ),
        other => panic!("應為動物守門的 BusinessRule，實得：{other:?}"),
    }
}

async fn fetch_status(app: &TestApp, protocol_id: Uuid) -> String {
    sqlx::query_scalar::<_, String>("SELECT status::text FROM protocols WHERE id = $1")
        .bind(protocol_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("fetch status")
}

// ── 存活動物存在時結案應被拒絕 ────────────────────────────────────────────────
#[tokio::test]
#[serial]
async fn close_blocked_when_animal_in_experiment() {
    let app = TestApp::spawn().await;
    let (protocol_id, iacuc_no) = seed_approved_protocol(&app).await;
    // 一隻已犧牲（安樂死）+ 一隻實驗中（存活）→ 存活動物存在，結案應被拒
    seed_animal(&app, &iacuc_no, AnimalStatus::Euthanized).await;
    seed_animal(&app, &iacuc_no, AnimalStatus::InExperiment).await;

    let err = ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
        .await
        .expect_err("計畫下仍有存活動物時結案應被拒絕");
    assert_blocked_by_animal_guard(&err);
    assert_eq!(
        fetch_status(&app, protocol_id).await,
        "APPROVED",
        "守門失效時計畫會被錯誤結案"
    );
}

// ── 未分配 / 實驗完成 同屬「存活」（未離場）→ 亦應擋下 ──────────────────────────
#[tokio::test]
#[serial]
async fn close_blocked_when_animal_completed_or_unassigned() {
    let app = TestApp::spawn().await;

    for alive_status in [AnimalStatus::Completed, AnimalStatus::Unassigned] {
        let (protocol_id, iacuc_no) = seed_approved_protocol(&app).await;
        seed_animal(&app, &iacuc_no, alive_status).await;

        let label = alive_status.display_name();
        let err =
            ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
                .await
                .expect_err(&format!("狀態 {label} 未離場，結案應被拒絕"));
        assert_blocked_by_animal_guard(&err);
        assert_eq!(
            fetch_status(&app, protocol_id).await,
            "APPROVED",
            "守門失效時計畫會被錯誤結案（狀態 {label}）"
        );
    }
}

// ── 已預約（earmark）但未分配的動物亦應阻擋結案（iacuc_no NULL、走 reserved_protocol_id） ──
#[tokio::test]
#[serial]
async fn close_blocked_when_animal_reserved() {
    let app = TestApp::spawn().await;
    let (protocol_id, _iacuc_no) = seed_approved_protocol(&app).await;
    // 一隻已預約給此計畫、尚未正式分配（iacuc_no NULL）→ 守門須經 reserved_protocol_id 涵蓋
    seed_reserved_animal(&app, protocol_id).await;

    let err = ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
        .await
        .expect_err("有動物預約給此計畫時結案應被拒絕");
    assert_blocked_by_animal_guard(&err);
    assert_eq!(
        fetch_status(&app, protocol_id).await,
        "APPROVED",
        "守門未涵蓋 reserved earmark → 計畫被錯誤結案"
    );
}

// ── 所有動物皆已離場（安樂死 / 猝死 / 已轉讓）→ 可結案 ───────────────────────────
#[tokio::test]
#[serial]
async fn close_allowed_when_all_animals_left() {
    let app = TestApp::spawn().await;
    let (protocol_id, iacuc_no) = seed_approved_protocol(&app).await;
    seed_animal(&app, &iacuc_no, AnimalStatus::Euthanized).await;
    seed_animal(&app, &iacuc_no, AnimalStatus::SuddenDeath).await;
    seed_animal(&app, &iacuc_no, AnimalStatus::Transferred).await;

    ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
        .await
        .expect("所有動物皆已離場時結案應成功");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

// ── 計畫下完全沒有動物 → 可結案 ─────────────────────────────────────────────
#[tokio::test]
#[serial]
async fn close_allowed_when_no_animals() {
    let app = TestApp::spawn().await;
    let (protocol_id, _iacuc_no) = seed_approved_protocol(&app).await;

    ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
        .await
        .expect("無動物時結案應成功");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

// ── 已軟刪除的存活動物不應阻擋結案 ──────────────────────────────────────────────
#[tokio::test]
#[serial]
async fn close_allowed_when_alive_animal_soft_deleted() {
    let app = TestApp::spawn().await;
    let (protocol_id, iacuc_no) = seed_approved_protocol(&app).await;
    // 建立一隻實驗中動物後軟刪除 → 不計入存活
    let unique = &Uuid::new_v4().to_string()[..6];
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, status, breed, gender, entry_date, iacuc_no, deleted_at)
           VALUES (gen_random_uuid(), $1, $2, 'miniature', 'male', NOW(), $3, NOW())"#,
    )
    .bind(format!("E{unique}"))
    .bind(AnimalStatus::InExperiment)
    .bind(&iacuc_no)
    .execute(&app.db_pool)
    .await
    .expect("insert soft-deleted animal");

    ProtocolService::change_status(&app.db_pool, &SYSTEM_TEST, protocol_id, &close_req())
        .await
        .expect("僅存軟刪除動物時結案應成功");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

// ── R89-8：計畫擁有人（PI/SD）結案自己已核准的計畫 ──────────────────────────────

#[tokio::test]
#[serial]
async fn owner_pi_can_close_own_approved_protocol_without_change_status_permission() {
    let app = TestApp::spawn().await;
    let pi_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 pi', true, false)",
    )
    .bind(pi_id)
    .bind(format!("r89-8-pi-{}@test.local", &pi_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed pi user");
    // 計畫需要有 SD，否則雙簽在設計上永遠湊不齊（gate 條件 1 要求 SD 那一簽存在），
    // 本案例的受測點會被雙簽擋在門外而驗不到。
    let sd_id = seed_user(&app, "pi-case-sd").await;
    let protocol_id = seed_approved_protocol_owned_by(&app, pi_id, Some(sd_id)).await;
    seed_closure_dual_signatures(&app, protocol_id, pi_id, sd_id).await;

    ProtocolService::change_status(
        &app.db_pool,
        &close_own_actor(pi_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect("計畫擁有人（PI）應可用 close_own 窄縫結案自己已核准的計畫");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

/// R89-8（Qodo/PR-Agent review 發現的權限繞過缺口）：即使擁有人身分與狀態都符合，
/// actor 完全沒有 `aup.protocol.close_own`（也沒有 `aup.protocol.change_status`）時
/// 仍應被拒絕——這條鎖的是「直接呼叫 service、繞過 handler 權限閘」的路徑
/// （現況 repo 內確實有 `services/mcp/tools.rs`、`services/protocol/ai_review.rs`
/// 兩處直接呼叫 `ProtocolService::change_status`）。
#[tokio::test]
#[serial]
async fn owner_without_close_own_permission_cannot_close_via_direct_service_call() {
    let app = TestApp::spawn().await;
    let pi_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 pi no perm', true, false)",
    )
    .bind(pi_id)
    .bind(format!("r89-8-noperm-{}@test.local", &pi_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed pi user");
    let protocol_id = seed_approved_protocol_owned_by(&app, pi_id, None).await;

    let err = ProtocolService::change_status(
        &app.db_pool,
        &owner_without_close_own_actor(pi_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect_err("擁有人身分成立，但缺少 close_own 權限時仍應被拒絕（防繞過 handler 閘門）");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );
    assert_eq!(
        fetch_status(&app, protocol_id).await,
        "APPROVED",
        "缺少權限時不得結案，狀態不應變更"
    );
}

#[tokio::test]
#[serial]
async fn owner_study_director_can_close_own_approved_protocol() {
    let app = TestApp::spawn().await;
    let admin_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");
    let sd_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 sd', true, false)",
    )
    .bind(sd_id)
    .bind(format!("r89-8-sd-{}@test.local", &sd_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed sd user");
    // pi_user_id 刻意指向別人（admin），驗證 SD 身分本身即可通過（不必同時是 PI）。
    let protocol_id = seed_approved_protocol_owned_by(&app, admin_id, Some(sd_id)).await;
    seed_closure_dual_signatures(&app, protocol_id, admin_id, sd_id).await;

    ProtocolService::change_status(
        &app.db_pool,
        &close_own_actor(sd_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect("計畫的 study_director 應可用 close_own 窄縫結案");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

#[tokio::test]
#[serial]
async fn non_owner_with_close_own_permission_cannot_close_others_protocol() {
    let app = TestApp::spawn().await;
    let owner_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");
    let other_pi_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 other pi', true, false)",
    )
    .bind(other_pi_id)
    .bind(format!("r89-8-other-{}@test.local", &other_pi_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed other pi user");
    // 計畫擁有人是 owner_id（admin），other_pi_id 只是同樣持有 close_own 權限的另一個 PI。
    let protocol_id = seed_approved_protocol_owned_by(&app, owner_id, None).await;

    let err = ProtocolService::change_status(
        &app.db_pool,
        &close_own_actor(other_pi_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect_err("close_own 不可用來結案別人的計畫");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );
    assert_eq!(
        fetch_status(&app, protocol_id).await,
        "APPROVED",
        "非本人不得結案，狀態不應變更"
    );
}

#[tokio::test]
#[serial]
async fn delegate_pi_via_user_protocols_can_close_own_protocol() {
    let app = TestApp::spawn().await;
    let owner_id: Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE email LIKE '%admin%' LIMIT 1")
            .fetch_one(&app.db_pool)
            .await
            .expect("fetch admin");
    let delegate_pi_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 delegate pi', true, false)",
    )
    .bind(delegate_pi_id)
    .bind(format!("r89-8-delegate-{}@test.local", &delegate_pi_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed delegate pi user");
    // 同上：雙簽要湊得齊，計畫得先有 SD。委派 PI 不是簽章的一方——
    // 簽章仍是 protocols 上記的 PI 與 SD，這正是「委派 PI 只影響誰能按，
    // 不影響誰要簽」的體現。
    let sd_id = seed_user(&app, "delegate-case-sd").await;
    let protocol_id = seed_approved_protocol_owned_by(&app, owner_id, Some(sd_id)).await;
    seed_closure_dual_signatures(&app, protocol_id, owner_id, sd_id).await;
    // 與 /my-projects 的 can_edit 投影同一種委派 PI：user_protocols.role_in_protocol='PI'，
    // 不是 protocols.pi_user_id 本人。
    sqlx::query(
        "INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol) \
         VALUES ($1, $2, 'PI')",
    )
    .bind(delegate_pi_id)
    .bind(protocol_id)
    .execute(&app.db_pool)
    .await
    .expect("seed delegate PI membership");

    ProtocolService::change_status(
        &app.db_pool,
        &close_own_actor(delegate_pi_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect("user_protocols 委派 PI 應可用 close_own 窄縫結案，與前端 can_edit 判定一致");
    assert_eq!(fetch_status(&app, protocol_id).await, "CLOSED");
}

#[tokio::test]
#[serial]
async fn owner_with_close_own_permission_cannot_close_suspended_protocol() {
    let app = TestApp::spawn().await;
    let pi_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'fake', 'r89-8 pi suspended', true, false)",
    )
    .bind(pi_id)
    .bind(format!("r89-8-susp-{}@test.local", &pi_id.to_string()[..8]))
    .execute(&app.db_pool)
    .await
    .expect("seed pi user");
    let protocol_id = seed_approved_protocol_owned_by(&app, pi_id, None).await;
    sqlx::query("UPDATE protocols SET status = 'SUSPENDED'::protocol_status WHERE id = $1")
        .bind(protocol_id)
        .execute(&app.db_pool)
        .await
        .expect("force protocol into SUSPENDED");

    // state machine 本身允許 SUSPENDED → CLOSED（給有完整 change_status 權限的角色），
    // 但 close_own 窄縫只開放 APPROVED / APPROVED_WITH_CONDITIONS 來源狀態。
    let err = ProtocolService::change_status(
        &app.db_pool,
        &close_own_actor(pi_id),
        protocol_id,
        &close_req(),
    )
    .await
    .expect_err("close_own 窄縫不涵蓋 SUSPENDED 來源狀態，即使操作者是本人");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應為 Forbidden，實得：{err:?}"
    );
    assert_eq!(
        fetch_status(&app, protocol_id).await,
        "SUSPENDED",
        "窄縫應擋下，狀態不應變更"
    );
}
