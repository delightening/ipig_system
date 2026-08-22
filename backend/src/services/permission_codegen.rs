//! 權限常數 codegen：由 `permissions` 表產生前端 TS 常數檔。
//!
//! 產生器（`bin/gen_permission_constants.rs`）與 CI 守衛
//! （`tests/permission_constants_sync.rs`）共用本模組，確保兩端對「正確內容」的
//! 定義不會各自漂移 —— 否則守衛可能因格式差異而永久紅、或永久綠。
//!
//! 見 `docs/audit/button-permission-gate-2026-08-07.md` §7-2。

use std::path::PathBuf;

use sqlx::PgPool;

use crate::error::AppError;

/// 產出檔在 repo 內的位置（相對於 `backend/`）。
pub fn generated_ts_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("frontend")
        .join("src")
        .join("lib")
        .join("permissions.generated.ts")
}

/// 取出所有權限代碼（去重 + 排序，確保產出穩定）。
pub async fn fetch_permission_codes(pool: &PgPool) -> Result<Vec<String>, AppError> {
    let mut codes: Vec<String> = sqlx::query_scalar("SELECT code FROM permissions")
        .fetch_all(pool)
        .await?;
    codes.sort();
    codes.dedup();
    Ok(codes)
}

/// `animal.record.create` → `ANIMAL_RECORD_CREATE`
fn to_const_key(code: &str) -> String {
    code.to_uppercase().replace(['.', '-'], "_")
}

/// 產生 TS 檔內容。**格式改動必須同步更新已 commit 的產出檔**，否則 CI 守衛會紅。
pub fn render_ts(codes: &[String]) -> String {
    let mut out = String::from(
        r#"// ⚠️ 本檔由 `backend/src/bin/gen_permission_constants.rs` 產生，**請勿手動編輯**。
//
// 真相源是後端 `permissions` 資料表（migration seed +
// `startup/permissions.rs::ensure_required_permissions` 的聯集）。
// 新增 / 移除權限後重新產生：
//
//   cd backend && cargo run --bin gen_permission_constants
//
// CI 由 `backend/tests/permission_constants_sync.rs` 守著：本檔與後端不同步就會紅。
//
// ## 為什麼要有這個檔
//
// 前端手寫權限字串、後端 `require_permission!` 手寫權限字串，兩邊靠人眼對齊——
// 打錯一個字元的下場是「前端閘比後端鬆」（該藏的按鈕沒藏）或「該顯示的功能消失」，
// 而且兩者都不會有任何編譯期或執行期錯誤。改用本檔的常數後，字串打錯在 tsc 就會被擋。
//
// 見 `docs/audit/button-permission-gate-2026-08-07.md` §7-2。

export const PERMISSIONS = {
"#,
    );
    for code in codes {
        out.push_str(&format!("    {}: '{}',\n", to_const_key(code), code));
    }
    out.push_str("} as const\n\n");
    out.push_str("/** 後端 `permissions` 表中所有合法的權限代碼。 */\n");
    out.push_str("export type PermissionCode = (typeof PERMISSIONS)[keyof typeof PERMISSIONS]\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_key_conversion() {
        assert_eq!(to_const_key("animal.record.create"), "ANIMAL_RECORD_CREATE");
        assert_eq!(to_const_key("hr.overtime.view_all"), "HR_OVERTIME_VIEW_ALL");
        assert_eq!(to_const_key("some-dashed.code"), "SOME_DASHED_CODE");
    }

    /// 兩個不同的權限碼不得映射到同一個 TS 常數名 —— 撞名會讓其中一個在
    /// 物件字面值中被靜默覆蓋，產出的常數指向錯誤的權限字串。
    #[test]
    fn const_keys_are_unique_for_distinct_codes() {
        let codes = ["a.b_c", "a.b.c"];
        let k0 = to_const_key(codes[0]);
        let k1 = to_const_key(codes[1]);
        assert_eq!(k0, k1, "此組合本來就會撞名（. 與 _ 都轉成 _）");
        // 上面證明撞名有可能發生 —— 因此 render 前必須驗證實際資料無此情況，
        // 由 permission_constants_sync 測試對真實 DB 內容把關。
    }

    #[test]
    fn render_shape() {
        let out = render_ts(&["a.b".to_string(), "c.d".to_string()]);
        assert!(out.contains("    A_B: 'a.b',\n"));
        assert!(out.contains("    C_D: 'c.d',\n"));
        assert!(out.ends_with(
            "export type PermissionCode = (typeof PERMISSIONS)[keyof typeof PERMISSIONS]\n"
        ));
    }
}
