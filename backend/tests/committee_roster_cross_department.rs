//! IACUC 委員名冊：委員身分來自角色，與部門編制正交。
//!
//! 這支測試存在的理由是一個真實踩到的建模問題：**IACUC 委員可以同時是獸醫**，
//! 但 `users.department_id` 是單值的，一個人裝不進兩個部門。若把「委員會成員」
//! 用部門membership 表達，就會逼出「這個人到底放哪個部門」的假問題。
//!
//! 正解是兩者分開：部門樹呈現人事歸屬（決定請假找誰簽核、行事曆看得到誰），
//! 委員名冊依角色查出（決定誰審查、誰收通知）。同一個人可以同時出現在兩處。
//!
//! 本測試鎖住這個契約——特別是「歸屬別的部門的人也要出現在名冊裡」那條。
//! 若有人日後把名冊改成查 IACUC 部門的成員，兼任的獸醫會**靜默消失**於名冊，
//! 而畫面上只是少一列，不會有任何錯誤。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::UserService;

async fn seed_user(app: &TestApp, label: &str, roles: &[&str], internal: bool) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active)
           VALUES ($1, $2, 'x', $3, $4, true)"#,
    )
    .bind(id)
    .bind(format!("roster-{label}-{unique}@test.local"))
    .bind(format!("名冊{label}"))
    .bind(internal)
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    for code in roles {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
        )
        .bind(id)
        .bind(code)
        .execute(&app.db_pool)
        .await
        .expect("assign role");
    }
    id
}

async fn seed_department(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query("INSERT INTO departments (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("ROSTER_{label}_{unique}"))
        .bind(format!("名冊測試{label}"))
        .execute(&app.db_pool)
        .await
        .expect("insert dept");
    id
}

async fn set_department(app: &TestApp, user_id: Uuid, dept_id: Uuid) {
    sqlx::query("UPDATE users SET department_id = $2 WHERE id = $1")
        .bind(user_id)
        .bind(dept_id)
        .execute(&app.db_pool)
        .await
        .expect("set department");
}

/// 核心契約：委員名冊涵蓋**所有持有委員角色的人**，不論他歸屬哪個部門。
#[tokio::test]
#[serial]
async fn roster_includes_members_from_any_department() {
    let app = TestApp::spawn().await;

    let vet_dept = seed_department(&app, "vet").await;
    let other_dept = seed_department(&app, "other").await;

    // 兼任的獸醫：歸屬獸醫部，同時持有 REVIEWER —— 本測試的主角
    let dual = seed_user(&app, "dual", &["VET", "REVIEWER"], true).await;
    set_department(&app, dual, vet_dept).await;

    // 執行秘書：歸屬別的部門，仍是委員會核心
    let staff = seed_user(&app, "staff", &["IACUC_STAFF"], true).await;
    set_department(&app, staff, other_dept).await;

    // 外聘委員：沒有部門
    let external = seed_user(&app, "external", &["REVIEWER"], false).await;

    // 非委員：不該出現
    let unrelated = seed_user(&app, "unrelated", &["EXPERIMENT_STAFF"], true).await;
    set_department(&app, unrelated, vet_dept).await;

    let roster = UserService::list_committee_members(&app.db_pool)
        .await
        .expect("list_committee_members");
    let ids: Vec<Uuid> = roster.iter().map(|m| m.id).collect();

    assert!(
        ids.contains(&dual),
        "歸屬獸醫部但持有 REVIEWER 的人必須出現在名冊——這是本功能存在的理由"
    );
    assert!(ids.contains(&staff), "執行秘書歸屬其他部門，仍應出現在名冊");
    assert!(ids.contains(&external), "無部門的外聘委員應出現在名冊");
    assert!(
        !ids.contains(&unrelated),
        "只有 EXPERIMENT_STAFF 的人不是委員，不該出現"
    );

    // 名冊要標出人事歸屬——沒有這個就看不出「這位委員平時在哪」
    let dual_row = roster
        .iter()
        .find(|m| m.id == dual)
        .expect("dual in roster");
    assert_eq!(
        dual_row.department_name.as_deref(),
        Some("名冊測試vet"),
        "應顯示他實際歸屬的部門，而非 IACUC"
    );
    let external_row = roster
        .iter()
        .find(|m| m.id == external)
        .expect("external in roster");
    assert_eq!(
        external_row.department_name, None,
        "未編入部門者的 department_name 應為 None，讓 UI 顯示「未編入部門」"
    );
    assert!(
        !external_row.is_internal,
        "外聘委員的 is_internal 應為 false"
    );
}

/// 一人持有多個委員角色時只列一次（DISTINCT）。
///
/// 主席通常同時掛 REVIEWER，不去重會在名冊上出現兩列同一個人。
#[tokio::test]
#[serial]
async fn roster_deduplicates_multi_role_members() {
    let app = TestApp::spawn().await;

    let chair = seed_user(&app, "chair", &["IACUC_CHAIR", "REVIEWER"], true).await;

    let roster = UserService::list_committee_members(&app.db_pool)
        .await
        .expect("list_committee_members");

    let occurrences = roster.iter().filter(|m| m.id == chair).count();
    assert_eq!(
        occurrences, 1,
        "同時持有主席與審查委員角色者應只列一次，實得 {occurrences} 列"
    );
}
