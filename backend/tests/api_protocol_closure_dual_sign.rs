//! 結案雙簽 gate 的回歸測試（設計 A / migration 008）。
//!
//! # 這一組守的是什麼
//!
//! `close_pi_signature_id` / `close_sd_signature_id` 是外鍵，**外鍵只保證那一列
//! 簽章存在**——不保證它是這份計畫的、結案用的、還有效的。設計文件 §5.2a 因此
//! 列了 7 條條件，本檔**每一條各一支測試**。
//!
//! ⚠️ 為什麼要逐條測而不是只測「正常流程會成功」：
//! 正常流程那支測試在 gate 只寫「兩欄非 NULL」時**也會通過**。
//! 能區分「有 gate」與「gate 寫對了」的，只有那些刻意違反單一條件的案例。
//!
//! # ⚠️ 這些測試繞過 HTTP 層直接操作 DB
//!
//! 建簽章列時直接 INSERT，而不是呼叫簽章端點——因為要造出「entity_type 錯」、
//! 「signer 不對」這些**端點根本不會產生**的資料。gate 存在的理由正是那些資料
//! 可能從別的路徑進來（資料修補、未來的新端點、程式 bug），
//! 所以測試必須能造出它們。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::protocol::closure::{dual_signature_ready, CLOSURE_ENTITY_TYPE};

/// 建一份「可以簽結案」的計畫：APPROVED、補登完成、PI 與 SD 各一人。
async fn seed_protocol(app: &TestApp) -> (Uuid, Uuid, Uuid) {
    let pi = seed_user(app, "PI").await;
    let sd = seed_user(app, "EXPERIMENT_STAFF").await;
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by,
              study_director_user_id, import_pending)
           VALUES ($1, $2, $3, 'APPROVED'::protocol_status, $4, $4, $5, false)"#,
    )
    .bind(id)
    .bind(format!("CLOSE-{}", &Uuid::new_v4().to_string()[..8]))
    .bind("結案雙簽測試計畫")
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    (id, pi, sd)
}

async fn seed_user(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, true, false)"#,
    )
    .bind(id)
    .bind(format!("close-{}@example.com", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("close-{role_code}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    id
}

/// 直接 INSERT 一列簽章。所有欄位都可以指定，才造得出違反各條件的資料。
#[allow(clippy::too_many_arguments)]
async fn seed_signature(
    app: &TestApp,
    entity_type: &str,
    entity_id: &str,
    signer_id: Uuid,
    signature_type: &str,
    is_valid: bool,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO electronic_signatures
             (id, entity_type, entity_id, signer_id, signature_type,
              content_hash, signature_data, signature_method, meaning, is_valid)
           VALUES ($1, $2, $3, $4, $5, 'hash', 'data', 'password',
                   'CONFIRM'::signature_meaning, $6)"#,
    )
    .bind(id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(signer_id)
    .bind(signature_type)
    .bind(is_valid)
    .execute(&app.db_pool)
    .await
    .expect("insert signature");
    id
}

/// 跑 gate。包成 helper 讓每支測試只表達「改了哪一個變因」。
async fn gate(
    app: &TestApp,
    protocol_id: Uuid,
    pi: Uuid,
    sd: Option<Uuid>,
    pi_sig: Option<Uuid>,
    sd_sig: Option<Uuid>,
) -> bool {
    let mut tx = app.db_pool.begin().await.expect("begin");
    let ready = dual_signature_ready(&mut tx, protocol_id, pi, sd, pi_sig, sd_sig)
        .await
        .expect("gate");
    tx.rollback().await.expect("rollback");
    ready
}

/// 基準線：7 條都滿足 → 放行。
///
/// ⚠️ 這支**單獨存在時證明力很弱**——gate 只寫「兩欄非 NULL」它也會過。
/// 它的作用是當底下每支「違反一條」的測試變紅時，用來確認紅的原因是那一條，
/// 而不是整個 helper 壞掉。
#[tokio::test]
#[serial]
async fn all_conditions_met_allows_closure() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let pi_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(gate(&app, p, pi, Some(sd), Some(pi_sig), Some(sd_sig)).await);
}

/// 條件 1：缺任一簽 → 不放行。
#[tokio::test]
#[serial]
async fn missing_either_signature_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let pi_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(
        !gate(&app, p, pi, Some(sd), None, Some(sd_sig)).await,
        "缺 PI 簽應擋"
    );
    assert!(
        !gate(&app, p, pi, Some(sd), Some(pi_sig), None).await,
        "缺 SD 簽應擋"
    );
    assert!(
        !gate(&app, p, pi, Some(sd), None, None).await,
        "兩者皆缺應擋"
    );
}

/// 條件 1 的延伸：計畫根本沒有 SD → 不放行。
///
/// ⚠️ 這不是理論案例。設計文件 §5.4 指出 SD 離職會讓 GLP 案永遠結不了案，
/// 而「沒有 SD」是同一個問題的更早階段。擋在這裡是對的——放行等於讓
/// 「單方簽名」冒充雙簽。
#[tokio::test]
#[serial]
async fn protocol_without_sd_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, _sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let pi_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;
    let other = seed_user(&app, "EXPERIMENT_STAFF").await;
    let other_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, other, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, None, Some(pi_sig), Some(other_sig)).await);
}

/// 🔴 條件 2：拿**核准簽章**（entity_type='protocol'）充數 → 不放行。
///
/// 這是 entity_type 分離（裁定 5）的實際價值：若不驗，一份計畫只要被核准簽過，
/// 就可以把那張簽章的 id 寫進結案欄位而通過雙簽。
#[tokio::test]
#[serial]
async fn approval_signature_cannot_masquerade_as_closure() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    // 注意 entity_type 是 'protocol'，不是 'protocol_closure'
    let approval = seed_signature(&app, "protocol", &eid, pi, "CONFIRM", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(approval), Some(sd_sig)).await);
}

/// 🔴 條件 3：拿**別份計畫**的結案簽章 → 不放行。
#[tokio::test]
#[serial]
async fn signature_from_another_protocol_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let (other_p, _, _) = seed_protocol(&app).await;
    let eid = p.to_string();
    // PI 這張掛在**另一份計畫**上
    let foreign = seed_signature(
        &app,
        CLOSURE_ENTITY_TYPE,
        &other_p.to_string(),
        pi,
        "CONFIRM",
        true,
    )
    .await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(foreign), Some(sd_sig)).await);
}

/// 條件 4：signature_type 不是 CONFIRM → 不放行。
///
/// 語意問題（§11.50 的 meaning）：結案是「確認試驗完成」（responsibility），
/// 不是「核准」（approval）。用錯型別會讓稽核報表把兩者混為一談。
#[tokio::test]
#[serial]
async fn wrong_signature_type_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let approve_type = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "APPROVE", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(approve_type), Some(sd_sig)).await);
}

/// 🔴 條件 5：已被作廢（is_valid=false）的簽章 → 不放行。
///
/// 沒有這條的話，`SignatureService::invalidate` 就等於沒有作用——
/// 作廢一張結案簽章之後，計畫仍然結得了案。
#[tokio::test]
#[serial]
async fn invalidated_signature_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let invalidated = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", false).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(invalidated), Some(sd_sig)).await);
}

/// 🔴 條件 6：無關人員代簽 → 不放行。
#[tokio::test]
#[serial]
async fn signature_by_unrelated_person_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let stranger = seed_user(&app, "EXPERIMENT_STAFF").await;
    let eid = p.to_string();
    let by_stranger =
        seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, stranger, "CONFIRM", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(by_stranger), Some(sd_sig)).await);
}

/// 🔴 條件 6 的關鍵形式：**兩欄互換** → 不放行。
///
/// ⚠️ 這支是條件 6 真正的價值所在。若 gate 只檢查「兩個 signer 分別是 PI 與 SD」
/// （而不是逐欄對應），把 SD 的簽章塞進 PI 欄、PI 的塞進 SD 欄**會通過**——
/// 兩個 signer 集合完全正確，只是各自簽錯了欄位。
///
/// 實作時我差一點就寫成集合比對。
#[tokio::test]
#[serial]
async fn swapped_columns_block() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let pi_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;
    let sd_sig = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, sd, "CONFIRM", true).await;

    // 故意互換：PI 欄放 SD 的簽章，SD 欄放 PI 的
    assert!(!gate(&app, p, pi, Some(sd), Some(sd_sig), Some(pi_sig)).await);
}

/// 🔴 條件 7：同一人簽兩次 → 不放行（自簽自證）。
///
/// ⚠️ 這條在 PI≠SD 規則（裁定 16）之外**仍然必要**：那條規則管的是「指派當下」，
/// 而兩個欄位可能在指派之後才變成同一人（存量資料、或日後放寬規則）。
#[tokio::test]
#[serial]
async fn same_person_signing_both_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, _sd) = seed_protocol(&app).await;
    // 把 SD 也設成 PI 本人（模擬存量資料）
    sqlx::query("UPDATE protocols SET study_director_user_id = $2 WHERE id = $1")
        .bind(p)
        .bind(pi)
        .execute(&app.db_pool)
        .await
        .expect("set sd = pi");
    let eid = p.to_string();
    let sig_a = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;
    let sig_b = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(pi), Some(sig_a), Some(sig_b)).await);
}

/// 條件 7 的退化形式：兩欄指向**同一張**簽章 → 不放行。
#[tokio::test]
#[serial]
async fn same_signature_in_both_columns_blocks() {
    let app = TestApp::spawn().await;
    let (p, pi, sd) = seed_protocol(&app).await;
    let eid = p.to_string();
    let one = seed_signature(&app, CLOSURE_ENTITY_TYPE, &eid, pi, "CONFIRM", true).await;

    assert!(!gate(&app, p, pi, Some(sd), Some(one), Some(one)).await);
}
