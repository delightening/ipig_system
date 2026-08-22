//! R30-24：啟動期 DB schema / role / permission self-test。
//!
//! ## 目的
//!
//! 啟動成功 ≠ DB 正確。以下情境後端會啟動但業務 fail：
//! 1. DB 從備份還原但漏跑 migration → schema 落後 binary
//! 2. `--skip-migration-check=true` 誤用於 production
//! 3. seed 漏跑 → `system_user` 不存在 → audit 寫入全部炸 FK
//! 4. role / permission row 被誤刪 → admin 自己沒 admin / SYSTEM_ADMIN role
//! 5. 後端升級但 DB 跑舊 schema → 新 column / enum 不存在
//!
//! 這些「啟動 OK 但業務 broken」情境非常難 debug。Self-test 在啟動期
//! 跑幾個小 query 就能 catch，配合 R30-23 production fail-fast 形成
//! 完整啟動期防線。
//!
//! ## 對應合規
//!
//! - GLP §1.4：「系統啟動前驗證 schema / role / permission 完整性」
//! - 21 CFR §11.10(c)（保護紀錄完整性）：preventing schema drift
//!
//! ## 不做什麼
//!
//! - 不檢查所有 table 完整 row count（昂貴 + 過度約束）
//! - 不驗 individual user perms（runtime auth_middleware 的工作）
//! - 不重複 migration 自身的 schema check（sqlx::migrate! 已 fail-fast）

use sqlx::PgPool;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_SYSTEM_ADMIN};
use crate::middleware::SYSTEM_USER_ID;
use crate::Result;

/// Helper：檢查 `public.<table>.<column>` 是否存在。
async fn column_exists(pool: &PgPool, table: &str, column: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = $1
              AND column_name = $2
        )",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// system_user (migration 033 seed) 存在性檢查。
async fn check_system_user(pool: &PgPool) -> Result<Option<String>> {
    let ok: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(SYSTEM_USER_ID)
        .fetch_one(pool)
        .await?;
    if ok {
        Ok(None)
    } else {
        Ok(Some(format!(
            "system_user (id={}) 不存在 — migration 033 (system_user seed) 未跑。\n     \
             audit 寫入 / Anonymous → SYSTEM 替代會全部 FK fail。",
            SYSTEM_USER_ID
        )))
    }
}

/// Admin role 存在性檢查。
///
/// 接受 `SYSTEM_ADMIN`（canonical 名稱）或 `admin`（legacy 名稱，migration 002 seed 用）。
/// 整個 code base 都以 `ROLE_SYSTEM_ADMIN || ROLE_ADMIN_LEGACY` OR 檢查（見
/// middleware/auth.rs / services/auth/session.rs / login_tracker.rs 等），self-test
/// 必須對齊；只接 SYSTEM_ADMIN 會在每個 fresh deploy 都 fail（migration 002 從沒
/// rename `admin` → `SYSTEM_ADMIN`，那是 legacy 設計留下來的雙名共存）。
///
/// GUEST role 已於 R37-9 deprecation 棄用（見 `project_guest_role_deprecated` memory），
/// 不再列為 essential role；fresh deploy 缺 GUEST 為預期行為。
async fn check_required_roles(pool: &PgPool) -> Result<Vec<String>> {
    let has_admin: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM roles WHERE code IN ($1, $2))")
            .bind(ROLE_SYSTEM_ADMIN)
            .bind(ROLE_ADMIN_LEGACY)
            .fetch_one(pool)
            .await?;
    if has_admin {
        Ok(Vec::new())
    } else {
        Ok(vec![format!(
            "essential admin role 不存在 — migration 002 / role seed 未完整 \
             （接受 '{}' 或 legacy '{}'，兩者皆無）。",
            ROLE_SYSTEM_ADMIN, ROLE_ADMIN_LEGACY
        )])
    }
}

/// permissions 表非空檢查（truncate / 大規模刪除偵測）。
async fn check_permissions_table(pool: &PgPool) -> Result<Option<String>> {
    let has_rows: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM permissions LIMIT 1)")
        .fetch_one(pool)
        .await?;
    if !has_rows {
        Ok(Some(
            "permissions 表空 — seed 失敗或 truncate 後啟動。所有 has_permission 將回 false。"
                .to_string(),
        ))
    } else {
        Ok(None)
    }
}

/// Schema column 完整性（migration drift 偵測）。
async fn check_required_columns(pool: &PgPool) -> Result<Vec<String>> {
    const REQUIRED_COLUMNS: &[(&str, &str, &str, &str)] = &[
        (
            "electronic_signatures",
            "meaning",
            "043",
            "SignatureService::sign 寫入會 fail（R30-10 §11.50 meaning 欄缺失）。",
        ),
        (
            "electronic_signatures",
            "hmac_version",
            "042",
            "簽章寫入 / verify dispatch 會 fail（R30-7 HMAC 版本化欄缺失）。",
        ),
        (
            "user_activity_logs",
            "hmac_version",
            "037",
            "audit chain verify 會無法 dispatch 版本（R26-6 欄缺失）。",
        ),
    ];
    let mut out = Vec::new();
    for (table, column, mig, impact) in REQUIRED_COLUMNS {
        if !column_exists(pool, table, column).await? {
            out.push(format!(
                "{}.{} column 不存在 — migration {} 未跑。\n     {}",
                table, column, mig, impact
            ));
        }
    }
    Ok(out)
}

/// 輸出 self-test 結果到 tracing。
fn log_self_test_result(failures: &[String]) {
    if failures.is_empty() {
        tracing::info!(
            "[R30-24] ✅ DB self-test 全部通過（system_user / roles / permissions / schema）"
        );
        return;
    }
    let numbered = failures
        .iter()
        .enumerate()
        .map(|(i, w)| format!("  {}. {}", i + 1, w))
        .collect::<Vec<_>>()
        .join("\n");
    tracing::error!(
        "\n╔════════════════════════════════════════════════════════════╗\n\
           ║  ❌ DB self-test 失敗：{} 項                                ║\n\
           ╠════════════════════════════════════════════════════════════╣\n\
           {}\n\
           ╚════════════════════════════════════════════════════════════╝",
        failures.len(),
        numbered
    );
}

/// R30-24 self-test 失敗時，依 R30-23 production fail-fast 規則決定是否 exit。
///
/// **本函式只負責檢查 + 印 log**；caller (main.rs) 依 `is_production()` 決定 exit。
/// 拆開可在 dev / staging 環境只 warn 不 exit，方便除錯。
///
/// 回傳：失敗檢查項數量。0 = 全通過。
pub async fn run_db_self_test(pool: &PgPool) -> Result<usize> {
    let mut failures: Vec<String> = Vec::new();

    if let Some(msg) = check_system_user(pool).await? {
        failures.push(msg);
    }
    failures.extend(check_required_roles(pool).await?);
    if let Some(msg) = check_permissions_table(pool).await? {
        failures.push(msg);
    }
    failures.extend(check_required_columns(pool).await?);

    log_self_test_result(&failures);
    Ok(failures.len())
}
