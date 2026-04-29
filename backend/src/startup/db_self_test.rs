//! R30-24：啟動期 DB schema / role / permission self-test。
//!
//! ## 目的
//!
//! 啟動成功 ≠ DB 正確。以下情境後端會啟動但業務 fail：
//! 1. DB 從備份還原但漏跑 migration → schema 落後 binary
//! 2. `--skip-migration-check=true` 誤用於 production
//! 3. seed 漏跑 → `system_user` 不存在 → audit 寫入全部炸 FK
//! 4. role / permission row 被誤刪 → admin 自己沒 SYSTEM_ADMIN role
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

use crate::middleware::SYSTEM_USER_ID;
use crate::Result;

/// R30-24 self-test 失敗時，依 R30-23 production fail-fast 規則決定是否 exit。
///
/// **本函式只負責檢查 + 印 log**；caller (main.rs) 依 `is_production()` 決定 exit。
/// 拆開可在 dev / staging 環境只 warn 不 exit，方便除錯。
///
/// 回傳：失敗檢查項數量。0 = 全通過。
pub async fn run_db_self_test(pool: &PgPool) -> Result<usize> {
    let mut failures: Vec<String> = Vec::new();

    // 1. system_user 存在（migration 033 seed）
    let sys_user_exists: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
    )
    .bind(SYSTEM_USER_ID)
    .fetch_optional(pool)
    .await?;
    let sys_ok = sys_user_exists.map(|(b,)| b).unwrap_or(false);
    if !sys_ok {
        failures.push(format!(
            "system_user (id={}) 不存在 — migration 033 (system_user seed) 未跑。\n     \
             audit 寫入 / Anonymous → SYSTEM 替代會全部 FK fail。",
            SYSTEM_USER_ID
        ));
    }

    // 2. 必要 role 存在（SYSTEM_ADMIN / GUEST）
    let role_codes: Vec<(String,)> = sqlx::query_as(
        "SELECT code FROM roles WHERE code IN ('SYSTEM_ADMIN', 'GUEST')",
    )
    .fetch_all(pool)
    .await?;
    let codes: Vec<String> = role_codes.into_iter().map(|(c,)| c).collect();
    for required in ["SYSTEM_ADMIN", "GUEST"] {
        if !codes.iter().any(|c| c == required) {
            failures.push(format!(
                "essential role '{}' 不存在 — migration 002 / role seed 未完整。",
                required
            ));
        }
    }

    // 3. permissions 表非空（避免 truncate / 大規模刪除後啟動）
    let perm_count: Option<(i64,)> =
        sqlx::query_as("SELECT COUNT(*) FROM permissions").fetch_optional(pool).await?;
    let perm_n = perm_count.map(|(n,)| n).unwrap_or(0);
    if perm_n == 0 {
        failures.push(
            "permissions 表空 — seed 失敗或 truncate 後啟動。所有 has_permission 將回 false。"
                .to_string(),
        );
    }

    // 4. R30-10 schema 完整性：electronic_signatures.meaning column 存在
    let meaning_col: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'electronic_signatures'
              AND column_name = 'meaning'
        )",
    )
    .fetch_optional(pool)
    .await?;
    if !meaning_col.map(|(b,)| b).unwrap_or(false) {
        failures.push(
            "electronic_signatures.meaning column 不存在 — migration 043 未跑。\n     \
             SignatureService::sign 寫入會 fail。"
                .to_string(),
        );
    }

    // 5. R30-7 schema 完整性：electronic_signatures.hmac_version column 存在
    let hmac_col: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'electronic_signatures'
              AND column_name = 'hmac_version'
        )",
    )
    .fetch_optional(pool)
    .await?;
    if !hmac_col.map(|(b,)| b).unwrap_or(false) {
        failures.push(
            "electronic_signatures.hmac_version column 不存在 — migration 042 未跑。\n     \
             簽章寫入 / verify dispatch 會 fail。"
                .to_string(),
        );
    }

    // 6. R26-6 schema 完整性：user_activity_logs.hmac_version column 存在
    let activity_hmac_col: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'user_activity_logs'
              AND column_name = 'hmac_version'
        )",
    )
    .fetch_optional(pool)
    .await?;
    if !activity_hmac_col.map(|(b,)| b).unwrap_or(false) {
        failures.push(
            "user_activity_logs.hmac_version column 不存在 — migration 037 未跑。\n     \
             audit chain verify 會無法 dispatch 版本。"
                .to_string(),
        );
    }

    // 輸出總結
    let n = failures.len();
    if n == 0 {
        tracing::info!(
            "[R30-24] ✅ DB self-test 全部通過（system_user / roles / permissions / schema）"
        );
    } else {
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
            n,
            numbered
        );
    }
    Ok(n)
}
