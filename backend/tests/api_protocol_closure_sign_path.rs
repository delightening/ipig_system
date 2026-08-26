//! 結案雙簽：**寫入端 → 讀取端**接得起來嗎。
//!
//! # 為什麼要單獨一支
//!
//! `api_protocol_closure_dual_sign.rs` 那 11 支測的是 gate（讀取端）**判得對不對**，
//! 而它們的簽章列全部是 raw INSERT 造出來的。也就是說：
//!
//! > 到目前為止，**沒有任何測試呼叫過真正的寫入端 `sign_closure`**。
//!
//! 那 11 支即使全綠，也不排除「寫入端產出的列，gate 一列都不接受」——
//! 那樣的話結案永遠簽不完，而測試不會有任何反應。
//!
//! # 實測：這支到底補上了什麼（2026-08-27 mutation 驗證）
//!
//! 用兩個 mutation 量的，結果一個推翻了本檔原本寫的理由、一個證實了它：
//!
//! | mutation | 既有 11 支 | 本檔 |
//! |---|---|---|
//! | `CLOSURE_ENTITY_TYPE` 改成 `"protocol"` | 🔴 **1 支紅** | 🔴 紅 |
//! | 寫入端 `entity_id` 改大寫（讀取端不動） | ✅ **11 支全綠** | 🔴 紅 |
//!
//! 🔴 **第一列推翻了本檔初版寫的理由。** 初版寫「常數改掉時兩邊仍然一致、
//! 既有測試仍然綠」——**那是錯的**：`approval_signature_cannot_masquerade_as_closure`
//! 硬編字面 `"protocol"` 造偽裝簽章，常數一改 gate 就接受了它，那支會紅。
//! 字面值斷言仍然值得寫（在寫入點就講明意圖），但它**不是**唯一防線。
//!
//! ✅ **第二列才是本檔真正無可取代的地方。** 寫入端把 `entity_id` 寫成大寫，
//! gate 用 `entity_id = $3` 逐字比對就一列都撈不到 → **結案永遠簽不完**，
//! 而外鍵仍然成立、沒有任何錯誤訊息、既有 11 支全綠。
//! 那 11 支的簽章列都是自己 raw INSERT 的，格式當然對得上——
//! **它們證明不了真正的寫入端寫出來的格式也對得上。**
//!
//! 斷言一律用**字面值**而不是 `CLOSURE_ENTITY_TYPE`：拿共用常數斷言是「用 A 證明 A」，
//! 就算它不是唯一防線，也沒有理由在這裡自願放棄鑑別力。
//!
//! # 這支涵蓋到哪、沒涵蓋到哪
//!
//! - ✅ 密碼簽署路徑（`sign_record_tx` 的 `has_password` 分支）——真實使用者的主要路徑
//! - 🔴 **手寫簽名路徑未涵蓋**（`handwriting_svg`，走的是另一段 INSERT）。
//!   它寫的 `entity_type` / `entity_id` / `signature_type` 綁定與密碼路徑相同，
//!   但 `signature_method` 不同，且**沒有測試證明它也接得起來**。
//! - 🔴 HTTP 層（`authorize_closure_signer` 的權責檢查）未涵蓋，本檔從 service 層進入。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::services::{protocol_closure_sign, AuthService, ClosureSigner};

const TEST_PASSWORD: &str = "iPig$ecure1";

/// 一列簽章在 DB 裡的實際樣子。gate 讀的就是這幾欄。
#[derive(Debug, sqlx::FromRow)]
struct SigRow {
    entity_type: String,
    entity_id: String,
    signature_type: String,
    is_valid: bool,
    signer_id: Uuid,
}

async fn fetch_sig(app: &TestApp, id: Uuid) -> SigRow {
    sqlx::query_as::<_, SigRow>(
        "SELECT entity_type, entity_id, signature_type, is_valid, signer_id
           FROM electronic_signatures WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&app.db_pool)
    .await
    .expect("簽章列應該存在")
}

/// 建一個**密碼可用**的使用者。
///
/// ⚠️ 不能沿用鄰檔 `seed_user` 的 `password_hash = 'fake'`：
/// `sign_record_tx` 會呼叫 `AuthService::verify_password_by_id`，
/// 假 hash 會讓整條路徑在簽章寫入前就失敗——那會讓這支測試變成在測密碼驗證，
/// 而不是在測寫入端與讀取端接不接得起來。
async fn seed_signer(app: &TestApp, role_code: &str) -> Uuid {
    let id = Uuid::new_v4();
    let hash = AuthService::hash_password(TEST_PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name,
                              is_active, is_internal, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(format!(
        "signpath-{}@example.com",
        &Uuid::new_v4().to_string()[..8]
    ))
    .bind(&hash)
    .bind(format!("signpath-{role_code}"))
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

async fn seed_protocol(app: &TestApp) -> (Uuid, Uuid, Uuid) {
    let pi = seed_signer(app, "PI").await;
    let sd = seed_signer(app, "EXPERIMENT_STAFF").await;
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols
             (id, protocol_no, title, status, pi_user_id, created_by,
              study_director_user_id, import_pending)
           VALUES ($1, $2, $3, 'APPROVED'::protocol_status, $4, $4, $5, false)"#,
    )
    .bind(id)
    .bind(format!("SIGNPATH-{}", &Uuid::new_v4().to_string()[..8]))
    .bind("結案雙簽寫入路徑測試")
    .bind(pi)
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    (id, pi, sd)
}

/// 依這個使用者**實際被指派的角色**，從 DB 讀出他的角色碼與權限碼，
/// 組成登入後會拿到的那份快照。
///
/// # ⚠️ 為什麼不照鄰檔硬編 `permissions: vec!["aup.protocol.close_own".into()]`
///
/// 硬編等於把權限矩陣整個繞過——**矩陣哪天不再把 `close_own` 授給
/// `EXPERIMENT_STAFF`，這支測試仍然全綠，而正式環境的 SD 結不了案**。
/// 那樣的測試守不住「角色 → 權限 → 這條路徑走得通」這串接線裡的任何一段。
///
/// 這裡多花一個 query 換到的是：本測試對權限矩陣的假設**由 DB 現況決定**，
/// 假設不成立時會紅在下面那句 `assert!`，而不是紅在無關的地方。
async fn actor(app: &TestApp, id: Uuid) -> ActorContext {
    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.code FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
    )
    .bind(id)
    .fetch_all(&app.db_pool)
    .await
    .expect("load roles");
    let permissions: Vec<String> = sqlx::query_scalar(
        r#"SELECT DISTINCT p.code
             FROM user_roles ur
             JOIN role_permissions rp ON rp.role_id = ur.role_id
             JOIN permissions p ON p.id = rp.permission_id
            WHERE ur.user_id = $1"#,
    )
    .bind(id)
    .fetch_all(&app.db_pool)
    .await
    .expect("load permissions");

    // 前提斷言：沒有這個權限的話，下面失敗的會是狀態轉移的權限檢查，
    // 而錯誤訊息（「缺少結案自己計畫所需權限」）指向的是權限問題，
    // **不是**本檔要測的「寫入端與讀取端接不接得起來」。
    // 明寫成前提，讀的人才不會把矩陣問題誤判成簽章路徑壞掉。
    assert!(
        permissions.iter().any(|p| p == "aup.protocol.close_own"),
        "測試前提不成立：使用者 {id} 的角色 {roles:?} 在 DB 裡沒有 aup.protocol.close_own。\
         本檔測的是簽章寫入路徑，不是權限矩陣——請先確認 role_permissions 的內容"
    );

    ActorContext::User(CurrentUser {
        id,
        email: "signpath-actor@example.com".to_string(),
        roles,
        permissions,
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    })
}

/// PI 簽 → SD 簽 → 自動 CLOSED，且中間那兩列簽章的欄位形狀符合 gate 的預期。
#[tokio::test]
#[serial]
async fn pi_then_sd_signature_path_produces_rows_the_gate_accepts() {
    let app = TestApp::spawn().await;
    let (protocol_id, pi, sd) = seed_protocol(&app).await;

    // ---- 第一簽：PI ----
    let after_pi = protocol_closure_sign(
        &app.db_pool,
        &actor(&app, pi).await,
        protocol_id,
        ClosureSigner::Pi,
        pi,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("PI 簽結案應成功");

    // ⚠️ 只簽一邊**不可以**結案。這條斷言是這支測試有意義的前提：
    // 若第一簽就轉 CLOSED，後面「第二簽讓它 CLOSED」的斷言就毫無鑑別力。
    assert_eq!(after_pi.status.as_str(), "APPROVED", "只有 PI 簽時不該結案");
    let pi_sig_id = after_pi.close_pi_signature_id.expect("PI 簽章欄應已寫入");
    assert!(
        after_pi.close_sd_signature_id.is_none(),
        "PI 簽不該順手寫到 SD 那一欄"
    );

    // ---- 檢查那一列的形狀（字面值，不用常數）----
    let row = fetch_sig(&app, pi_sig_id).await;
    assert_eq!(
        row.entity_type, "protocol_closure",
        "結案簽章的 entity_type 必須與核准簽章的 'protocol' 分開——\
         寫成 'protocol' 的話，簽過結案就等於預先滿足核准閘門"
    );
    assert_eq!(
        row.entity_id,
        protocol_id.to_string(),
        "entity_id 必須是本計畫的 id"
    );
    // gate 用 `entity_id = $3` 直接比字串，所以格式必須逐字相符。
    // 大寫或加括號的 UUID 會讓 gate 一列都撈不到，而外鍵仍然成立——不會有任何錯誤訊息。
    assert_eq!(
        row.entity_id.len(),
        36,
        "entity_id 應是 36 字元的 UUID 字串"
    );
    assert_eq!(
        row.entity_id,
        row.entity_id.to_lowercase(),
        "entity_id 應為小寫"
    );
    assert_eq!(
        row.signature_type, "CONFIRM",
        "結案是雙方確認（§11.50 responsibility），不是核准"
    );
    assert!(row.is_valid, "新簽章應為有效（is_valid 靠 DB 預設值寫入）");
    assert_eq!(row.signer_id, pi, "簽章的 signer 應是 PI 本人");

    // ---- 第二簽：SD ----
    let after_sd = protocol_closure_sign(
        &app.db_pool,
        &actor(&app, sd).await,
        protocol_id,
        ClosureSigner::StudyDirector,
        sd,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("SD 簽結案應成功");

    assert_eq!(
        after_sd.status.as_str(),
        "CLOSED",
        "兩簽齊備後應自動轉 CLOSED"
    );
    let sd_sig_id = after_sd.close_sd_signature_id.expect("SD 簽章欄應已寫入");
    assert_eq!(
        after_sd.close_pi_signature_id,
        Some(pi_sig_id),
        "第二簽不該蓋掉第一簽"
    );
    assert_ne!(pi_sig_id, sd_sig_id, "兩簽應是不同的簽章列");

    let sd_row = fetch_sig(&app, sd_sig_id).await;
    assert_eq!(sd_row.entity_type, "protocol_closure");
    assert_eq!(sd_row.entity_id, protocol_id.to_string());
    assert_eq!(sd_row.signature_type, "CONFIRM");
    assert!(sd_row.is_valid);
    assert_eq!(sd_row.signer_id, sd, "第二簽的 signer 應是 SD 本人");
    assert_ne!(sd_row.signer_id, row.signer_id, "兩簽不可同一人");
}

/// 同一邊不可重複簽。
///
/// 為什麼要有：`sign_closure` 擋重複簽的分支若壞掉，上面那支測試**仍然全綠**——
/// 它從頭到尾只各簽一次，走不到那個分支。
#[tokio::test]
#[serial]
async fn same_side_cannot_sign_twice() {
    let app = TestApp::spawn().await;
    let (protocol_id, pi, _sd) = seed_protocol(&app).await;

    protocol_closure_sign(
        &app.db_pool,
        &actor(&app, pi).await,
        protocol_id,
        ClosureSigner::Pi,
        pi,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect("第一次 PI 簽應成功");

    let err = protocol_closure_sign(
        &app.db_pool,
        &actor(&app, pi).await,
        protocol_id,
        ClosureSigner::Pi,
        pi,
        Some(TEST_PASSWORD),
        None,
        None,
    )
    .await
    .expect_err("重複簽應被擋下");
    assert!(
        err.to_string().contains("重複簽"),
        "錯誤訊息應說明是重複簽，實際：{err}"
    );
}
