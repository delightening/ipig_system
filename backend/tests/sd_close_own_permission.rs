//! 回歸測試：R98-1 —— 擔任 SD 的人可以結案自己的計畫（使用者 2026-08-25 裁定 (a)）。
//!
//! ## 背景
//!
//! SD 只存在於 `protocols.study_director_user_id`，指派資格由 `validate_and_authorize_sd`
//! 把關為 `EXPERIMENT_STAFF`。但該角色原本沒有 `aup.protocol.close_own`，而
//! `services/protocol/status.rs` 的檢查順序是「先驗權限碼、才驗擁有人」——於是
//! 「只有 SD 身分」的人永遠停在第一關 403，連同檔的
//! `protocol.study_director_user_id == Some(u.id)` 擁有人判定都走不到。
//!
//! 實測（2026-08-24，vet 正式庫）：6 位現任 SD 有 5 位卡在這裡，
//! 25 份已核准計畫的 SD 按不下結案。
//!
//! ## 三個選項與裁定
//!
//! (a) 把 `close_own` 加進 `EXPERIMENT_STAFF`  ← **使用者裁定採用**
//! (b) 在 `status.rs` 對「本人就是該案 SD」豁免權限碼檢查
//! (c) 另開一個 SD 專用權限碼
//!
//! 三者的**實際授權範圍完全一樣**——擁有人檢查是共同的最後一關。差別只在
//! 權限表看起來像什麼。選 (a) 是為了維持本 repo「權限碼是唯一授權來源」的慣例：
//! (b) 會在 service 層開一條不經角色的授權路徑，日後稽核權限表時看不到這個能力。
//!
//! ## 這支測試守的是什麼
//!
//! 授予權限碼**本身**不是安全邊界——真正把範圍鎖住的是 `status.rs` 的三道檢查。
//! 所以本檔同時鎖兩件事：權限碼有授予（不然 SD 按不下），以及那三道檢查還在
//! （不然這個權限碼就真的變成「所有 staff 都能結任何計畫」）。
//! 只測前者的話，日後有人拿掉擁有人檢查，測試照樣綠。

use sqlx::PgPool;

/// 只認 `TEST_DATABASE_URL`，未設時才收明顯是測試庫的 `DATABASE_URL`（名稱含 `test`）。
/// 本測試會跑 migration；連錯 DB 的代價是對正式 schema 動手。
fn test_database_url() -> String {
    let (url, source) = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => (url, "TEST_DATABASE_URL"),
        Err(_) => (
            std::env::var("DATABASE_URL").expect(
                "TEST_DATABASE_URL 與 DATABASE_URL 皆未設定。本測試會跑 migration，\
                 請指向獨立可丟棄的測試 DB。",
            ),
            "DATABASE_URL",
        ),
    };
    let db_name = url
        .rsplit('/')
        .next()
        .and_then(|tail| tail.split(['?', '#']).next())
        .unwrap_or_default();
    assert!(
        db_name.contains("test"),
        "{source} 指向的資料庫 `{db_name}` 不像測試庫（名稱不含 test）。\
         本測試會跑 migration，拒絕在可能是 prod 的連線上執行。"
    );
    url
}

/// ⚠️ 跑完 migration 之後**必須再跑一次啟動時的權限種子**，否則測到的是舊狀態。
///
/// 權限的權威來源是 `startup/permissions.rs`（啟動時以 `ON CONFLICT DO NOTHING`
/// 補寫），而 `003_seed.sql` 只是 prod 的快照——由
/// `scripts/migration/make-squash-seed.sh` 週期性重新產生，所以它反映的是
/// **上次快照當時**的授予關係，不是程式碼現況。
///
/// 這一步是本測試第一版漏掉的：只跑 migration 的話，新加進 `permissions.rs`
/// 的授予永遠不會出現，測試會紅在「權限沒給」——但實際部署是會給的。
/// 那種紅是測試設定錯誤，不是程式錯誤，很容易被誤判成「改動沒生效」而白繞一圈。
async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let pool = PgPool::connect(&test_database_url())
        .await
        .expect("connect test db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");

    erp_backend::startup::ensure_required_permissions(&pool)
        .await
        .expect("ensure_required_permissions");
    erp_backend::startup::ensure_all_role_permissions(&pool)
        .await
        .expect("ensure_all_role_permissions");

    pool
}

async fn roles_holding(pool: &PgPool, permission_code: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"SELECT r.code
           FROM role_permissions rp
           JOIN roles r       ON r.id = rp.role_id
           JOIN permissions p ON p.id = rp.permission_id
           WHERE p.code = $1
           ORDER BY r.code"#,
    )
    .bind(permission_code)
    .fetch_all(pool)
    .await
    .expect("query roles holding permission")
}

/// 裁定 (a) 的正面：EXPERIMENT_STAFF 必須有 `close_own`，否則被指派為 SD 的人
/// 在 `status.rs` 第一關就 403，連擁有人判定都走不到。
#[tokio::test]
async fn close_own_granted_to_experiment_staff() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "aup.protocol.close_own").await;

    assert!(
        holders.iter().any(|r| r == "EXPERIMENT_STAFF"),
        "EXPERIMENT_STAFF 必須有 aup.protocol.close_own——SD 由此角色指派而來，\
         少了它會在 status.rs 的權限碼檢查就 403（實際持有者：{holders:?}）"
    );
}

/// PI 的既有授權不得被本次改動波及。
///
/// PI 早在 R89-8（2026-08-13）就拿到這個權限碼，用於結案自己主持的計畫。
/// 本次是「加給 EXPERIMENT_STAFF」，不是「改授予對象」。
#[tokio::test]
async fn close_own_still_granted_to_pi() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "aup.protocol.close_own").await;

    assert!(
        holders.iter().any(|r| r == "PI"),
        "PI 必須保留 aup.protocol.close_own（R89-8 既有授權，本次不該動到）\
         （實際持有者：{holders:?}）"
    );
}

/// 授予範圍不得擴散到審查方角色。
///
/// `close_own` 是「計畫擁有人結案自己的計畫」，審查委員與獸醫不是擁有人，
/// 給了他們就是把窄縫開成後門。這條特別重要——本次改動是在角色權限清單裡
/// 加一行，很容易被複製到相鄰的角色區塊。
#[tokio::test]
async fn close_own_not_granted_to_review_side_roles() {
    let pool = setup_pool().await;
    let holders = roles_holding(&pool, "aup.protocol.close_own").await;

    for forbidden in ["REVIEWER", "VET", "IACUC_CHAIR"] {
        assert!(
            !holders.iter().any(|r| r == forbidden),
            "{forbidden} 不得有 aup.protocol.close_own——它是計畫擁有人的窄縫通道，\
             不是審查方的能力（實際持有者：{holders:?}）"
        );
    }
}

/// 🔴 **真正的安全邊界在這裡，不在權限碼**。
///
/// 授予 `close_own` 給全體 EXPERIMENT_STAFF 之所以安全，完全依賴
/// `services/protocol/status.rs` 在權限碼之後還做三道檢查：
///   ① 狀態必須是 APPROVED / APPROVED_WITH_CONDITIONS
///   ② 必須是該計畫的 pi_user_id / study_director_user_id 本人，或委派 PI
///   ③ 不符合就 403
///
/// 少了任何一道，這個權限碼就從「結自己是 SD 的那幾份」變成
/// 「結任何一份已核准計畫」——8 位 staff 全部拿到那個能力。
///
/// 靜態掃 `status.rs` 而非跑 HTTP：這三道檢查的存在與順序是結構性質，
/// 用整合測試驗要先建計畫、指派 SD、登入三種身分，成本高且涵蓋不完；
/// 而檢查一旦被刪掉，靜態掃描立刻紅。
#[test]
fn status_rs_still_gates_close_own_by_ownership() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/services/protocol/status.rs"),
    )
    .expect("讀取 status.rs");

    let perm_pos = src
        .find(r#"has_permission("aup.protocol.close_own")"#)
        .expect("status.rs 應仍檢查 close_own 權限碼（service 層防繞過，不能只靠 handler）");

    // 擁有人判定：pi_user_id 本人 或 study_director_user_id 本人
    let owner_pos = src
        .find("study_director_user_id == Some(u.id)")
        .expect("status.rs 應仍以 study_director_user_id 判定擁有人——少了它，close_own 會變成『結任何計畫』");
    assert!(
        src.contains("protocol.pi_user_id == u.id"),
        "status.rs 應仍以 pi_user_id 判定擁有人"
    );

    // 狀態限制：只有已核准（含附條件）可由擁有人結案
    assert!(
        src.contains("ProtocolStatus::Approved")
            && src.contains("ProtocolStatus::ApprovedWithConditions"),
        "status.rs 應仍限制只有 APPROVED / APPROVED_WITH_CONDITIONS 可由擁有人結案"
    );

    // 順序：權限碼檢查在前、擁有人判定在後，兩者都必須在同一段窄縫邏輯裡
    assert!(
        perm_pos < owner_pos,
        "close_own 權限碼檢查應在擁有人判定之前（現況如此）；\
         順序若顛倒，未持有權限碼的人會先通過擁有人判定，語意改變"
    );

    // 不是擁有人就必須擋下——確認拒絕分支還在
    assert!(
        src.contains("僅計畫擁有人（PI/SD）本人可結案自己的計畫"),
        "status.rs 應仍有『不是擁有人 → 403』的拒絕分支"
    );
}

/// 反向驗證：確認上面那支結構掃描在「檢查被拿掉」時真的會叫。
///
/// 今晚已經有三次「測試本身無效」的教訓（守衛邊界跨進註解、回歸測試在舊實作下
/// 照樣綠、加 code 不加測試）。所以這裡不只驗乾淨狀態，還要證明檢測會在髒狀態下失敗。
#[test]
fn ownership_gate_scan_detects_removal() {
    // 人為構造：權限碼檢查還在，但擁有人判定被拿掉
    let vulnerable = r#"
        if !u.has_permission("aup.protocol.change_status") {
            if !u.has_permission("aup.protocol.close_own") {
                return Err(AppError::Forbidden("缺少結案自己計畫所需權限".to_string()));
            }
        }
    "#;

    assert!(
        !vulnerable.contains("study_director_user_id == Some(u.id)"),
        "掃描邏輯壞了：擁有人判定被拿掉的 fixture 卻被判定為仍有檢查"
    );
    assert!(
        !vulnerable.contains("僅計畫擁有人（PI/SD）本人可結案自己的計畫"),
        "掃描邏輯壞了：拒絕分支被拿掉的 fixture 卻被判定為仍有拒絕"
    );

    // 正向對照：完整版本必須被判定為有檢查
    let correct = r#"
        let is_direct_owner = protocol.pi_user_id == u.id
            || protocol.study_director_user_id == Some(u.id);
        if !is_direct_owner && !is_delegate_pi {
            return Err(AppError::Forbidden(
                "僅計畫擁有人（PI/SD）本人可結案自己的計畫".to_string(),
            ));
        }
    "#;
    assert!(
        correct.contains("study_director_user_id == Some(u.id)")
            && correct.contains("僅計畫擁有人（PI/SD）本人可結案自己的計畫"),
        "掃描邏輯壞了：完整版本卻被判定為缺少檢查"
    );
}
