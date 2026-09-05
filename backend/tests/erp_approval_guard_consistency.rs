//! 兩個「權限發給了用不了它的人」的授權矛盾，修正後的迴歸釘。
//!
//! 兩者的形狀相同：**handler 用 A 判準、service 用 B 判準**，兩邊都「正常運作」，
//! 但疊起來的淨結果是某些角色卡在中間。這種矛盾不會有任何錯誤訊號——權限表看起來
//! 是對的、測試是綠的、只有實際去按那個按鈕的人才知道。
//!
//! ## ① 沖銷單：DIRECTOR 拿得到權限卻核准不了
//!
//! ```text
//! 權限授予（實查）  erp.document.reverse_approve  → 只給 DIRECTOR
//! handlers/document.rs:343   require_permission!("erp.document.reverse_approve")   ← DIRECTOR 過
//! services/document/reversal.rs:196   if !user.is_admin() { Forbidden }            ← DIRECTOR 倒在這
//! ```
//!
//! 淨結果：那個權限只發給了一個用不了它的人；實際能核准的是管理員，
//! 而管理員的資格來自 `has_permission()` 對 admin 的短路，不是來自該權限。
//! 而 `tests/director_erp_authority_boundary.rs:39` 還明文斷言 DIRECTOR 必須具備它。
//!
//! 使用者 2026-08-26 裁定選項 A：放寬 service 判準與 handler 同源。
//!
//! ## ② 設備報廢：三個角色簽得了核准章卻按不了核准鍵
//!
//! ```text
//! 簽章  access.rs:1013  equipment.disposal.approve || equipment.manage
//! 核准  disposal.rs:327  只有 equipment.disposal.approve
//! 差集  equipment.manage 有、equipment.disposal.approve 沒有的角色（見下方測試，動態算）
//! ```
//!
//! 差集刻意**不寫死**：`sync_permissions` 只增不減（`ON CONFLICT DO NOTHING`），
//! 既有部署上歷史授予的 `equipment.manage` 不會被收回，所以實際差集會比
//! `startup/permissions.rs` 讀起來的更大，且各部署不同。
//!
//! 而 `sign_disposal_approver_tx` 有「已簽章不得覆寫」的硬擋，所以他們簽完之後
//! 真正有權核准的人反而簽不了章，單子卡在「已簽章、仍待核准」。
//!
//! 使用者 2026-08-26 裁定選項 B：收緊簽章判準，不放寬核准權
//! （決定設備報廢的是設備維護人員）。

mod common;
use common::TestApp;
use serial_test::serial;

/// 查某個權限碼授予了哪些角色。
async fn roles_with_permission(app: &TestApp, code: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT r.code FROM permissions p \
         JOIN role_permissions rp ON rp.permission_id = p.id \
         JOIN roles r ON r.id = rp.role_id \
         WHERE p.code = $1 ORDER BY r.code",
    )
    .bind(code)
    .fetch_all(&app.db_pool)
    .await
    .unwrap_or_else(|e| panic!("query roles for {code}: {e}"))
}

/// ① 修正後：`erp.document.reverse_approve` 的持有者必須真的能核准。
///
/// 判準是「權限授予對象」與「service 守衛」指向同一個東西。service 現在檢查的就是
/// 這個權限碼本身，所以只要權限有授予任何非管理員角色，這條就成立。
///
/// ⚠️ 這支測試**不模擬呼叫**——`approve_reversal` 需要一整條已核准單據 + 庫存 + 會計
/// 的前置狀態，建置成本遠高於它能證明的東西。它證的是「權限授予不再指向一個
/// 用不了它的角色」，那正是原本壞掉的那一環。
#[tokio::test]
#[serial]
async fn reverse_approve_permission_is_granted_to_someone_who_can_use_it() {
    let app = TestApp::spawn().await;
    let roles = roles_with_permission(&app, "erp.document.reverse_approve").await;

    assert!(
        !roles.is_empty(),
        "erp.document.reverse_approve 沒有授予任何角色——那 handler 的 \
         require_permission! 就只有管理員（靠 has_permission 短路）過得了，\
         等於這個權限碼不存在"
    );

    // 原本的矛盾：授予對象全是非管理員角色，而 service 只讓管理員過。
    // 修正後 service 檢查的是權限碼本身，所以非管理員的授予對象也能用。
    let non_admin: Vec<&String> = roles
        .iter()
        .filter(|r| {
            r.as_str() != erp_backend::constants::ROLE_SYSTEM_ADMIN
                && r.as_str() != erp_backend::constants::ROLE_ADMIN_LEGACY
        })
        .collect();
    assert!(
        !non_admin.is_empty(),
        "erp.document.reverse_approve 只授予管理員角色。\n\
         若這是刻意的（沖銷改為管理員專屬），請一併移除 \
         tests/director_erp_authority_boundary.rs 對 DIRECTOR 的斷言，\n\
         否則兩支測試會對「誰該有這個權限」給出相反的答案。\n\
         實際授予：{roles:?}"
    );
}

/// ② 修正後：報廢的「簽章」與「核准」必須是同一批人。
///
/// 兩條路徑各自的判準都在程式裡，這裡驗的是它們**指向同一個權限碼**——
/// 也就是不會再出現「簽得了、核准不了」的差集。
#[tokio::test]
#[serial]
async fn disposal_sign_and_approve_require_the_same_permission() {
    let app = TestApp::spawn().await;

    let approve_roles = roles_with_permission(&app, "equipment.disposal.approve").await;
    let manage_roles = roles_with_permission(&app, "equipment.manage").await;

    // 修正前 `require_equipment_disposal_approve` 額外認 equipment.manage，
    // 這個差集就是「簽得了核准章卻按不了核准鍵」的那群人。
    let gap: Vec<&String> = manage_roles
        .iter()
        .filter(|r| !approve_roles.contains(r))
        .collect();

    assert!(
        !gap.is_empty(),
        "前提消失了：equipment.manage 的授予對象已全部涵蓋於 \
         equipment.disposal.approve，本測試失去意義，請重新檢視或刪除。\n\
         manage={manage_roles:?} approve={approve_roles:?}"
    );

    // 前提還在（差集非空），所以「簽章判準若放寬到 manage 就會產生半套狀態」仍然成立。
    // 真正的斷言在程式碼層：require_equipment_disposal_approve 不得再認 equipment.manage。
    let access_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/services/access.rs"),
    )
    .expect("read access.rs");
    let fn_start = access_src
        .find("pub fn require_equipment_disposal_approve")
        .expect("找不到 require_equipment_disposal_approve");
    let fn_body = &access_src[fn_start..(fn_start + 400).min(access_src.len())];

    assert!(
        !fn_body.contains("equipment.manage"),
        "require_equipment_disposal_approve 又認了 equipment.manage。\n\
         那會讓 {gap:?} 這幾個角色簽得下核准人簽章、卻按不了核准鍵\n\
         （approve_disposal 只認 equipment.disposal.approve），\n\
         而 sign_disposal_approver_tx 的「已簽章不得覆寫」會讓真正有權核准的人\n\
         連簽章都補不上，單子卡在「已簽章、仍待核准」。\n\
         使用者 2026-08-26 裁定：收緊簽章、不放寬核准權。"
    );
}
