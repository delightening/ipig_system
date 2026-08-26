//! 守衛：不得再出現「單獨比對 `SYSTEM_ADMIN` 而沒有 legacy `admin` fallback」的判斷。
//!
//! ## 這支測試防的 bug
//!
//! `constants.rs:184-185` 定義了兩個管理員角色代碼：
//!
//! ```text
//! ROLE_SYSTEM_ADMIN = "SYSTEM_ADMIN"
//! ROLE_ADMIN_LEGACY = "admin"
//! ```
//!
//! 但 **`roles` 表只有 `admin`**：
//! - 2026-08-26 實查正式庫：16 個角色，`WHERE code IN ('SYSTEM_ADMIN','admin')` 只回 `admin`
//! - `backend/migrations/` 全目錄 grep `SYSTEM_ADMIN`：**0 命中**；`003_seed.sql:181` 只 seed `'admin'`
//!
//! 所以任何單獨比對 `SYSTEM_ADMIN` 的分支，在任何從 migration 建起來的部署上**恆為 false**。
//! 方向是 fail-closed（管理員拿不到權限、通知發不出去），**不會誤放行**，但功能是死的
//! 而且**沒有任何錯誤訊息**——這正是它能潛伏這麼久的原因。
//!
//! 2026-08-26 全域掃描找到 11 處（8 處授權判斷 + 3 處通知收件人），本 PR 一次修完。
//!
//! ## 為什麼掃字串而不是掃常數名
//!
//! 裸 SQL 字串（`repositories/hr.rs:96`、`handlers/hr/dashboard.rs:313` 等）**不含
//! `ROLE_SYSTEM_ADMIN` 這個識別字**。第一次掃描就是掃常數名，漏了 10 個檔案而不自知
//! ——結論剛好沒錯純屬運氣。編譯器也管不到裸 SQL，所以「刪掉常數」不能取代本測試。
//!
//! ## 判準
//!
//! 兩段式，**先縮小候選再判定**——第一版只做「同段落有沒有 fallback」，把註解、
//! `use` 行、以及拿 `ROLE_SYSTEM_ADMIN` 當**顯示用 i18n key** 的地方全部誤判成違規
//! （6 個假陽性）。那是「用樣式直接產出結論」的老毛病。
//!
//! 1. **候選**：非註解行，且該行同時含 `SYSTEM_ADMIN` 與一個**比對訊號**
//!    （`==` / `.contains(` / `get_users_by_role(` / `.bind(` / SQL 的 `= '` / `IN (` / `ANY(`）。
//!    純粹「提到」它的行（註解、import、`Some(ROLE_SYSTEM_ADMIN)` 這種回傳值）不算。
//! 2. **判定**：候選行前後 ±8 行內必須出現 legacy fallback
//!    （`ROLE_ADMIN_LEGACY` / `"admin"` / `'admin'` / `is_admin`）。
//!    用行距而非段落，因為 `bin/create_admin.rs` 的 fallback 隔了一個空行。

use std::fs;
use std::path::{Path, PathBuf};

/// 已知且**刻意**只比對 `SYSTEM_ADMIN` 的位置，不算違規。
///
/// 這兩處的語意是「**只有 SYSTEM_ADMIN 能指派 SYSTEM_ADMIN**」——刻意區分兩個代碼，
/// 不是漏 fallback。目前因該角色不存在而是惰性的，但無害（legacy admin 分支才是
/// 實際生效的那條）。⚠️ 若日後採「刪掉常數」的根治方案，這兩段要重新設計，
/// 無腦刪等於把該條授權規則一併刪掉。
const INTENTIONAL: &[&str] = &[
    "src/services/user.rs",   // assigns_system_admin && !actor_is_system_admin
    "src/services/access.rs", // 同上，角色指派授權的另一半
];

/// 本檔自己會提到 SYSTEM_ADMIN（doc comment），不掃。
fn is_self(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "system_admin_role_code_guard.rs")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") && !is_self(&path) {
            out.push(path);
        }
    }
}

#[test]
fn no_system_admin_comparison_without_legacy_fallback() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    assert!(!files.is_empty(), "沒掃到任何 .rs，路徑設定有問題");

    let mut offenders: Vec<String> = Vec::new();

    for path in &files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if INTENTIONAL.iter().any(|i| rel.ends_with(i)) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        if !text.contains("SYSTEM_ADMIN") {
            continue;
        }

        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // 註解與 use 行只是「提到」，不是比對。
            if trimmed.starts_with("//") || trimmed.starts_with("use ") {
                continue;
            }
            if !line.contains("SYSTEM_ADMIN") {
                continue;
            }
            // 比對訊號：沒有這些的話（例如 `Some(ROLE_SYSTEM_ADMIN)` 這種顯示用回傳值）
            // 就不是一個「拿它跟使用者角色比對」的地方。
            let is_comparison = [
                "==",
                ".contains(",
                "get_users_by_role(",
                ".bind(",
                "= '",
                "IN (",
                "ANY(",
            ]
            .iter()
            .any(|sig| line.contains(sig));
            if !is_comparison {
                continue;
            }

            // fallback 未必在同一行（`.bind(A)` / `.bind(B)` 相鄰、SQL 的 OR 分行寫）。
            let lo = i.saturating_sub(8);
            let hi = (i + 9).min(lines.len());
            let window = lines[lo..hi].join("\n");
            // ⚠️ 必須是 `.is_admin()` 這個**呼叫**，不能只找子字串 `is_admin`。
            // 2026-08-26 mutation 實測：把 `handlers/mcp.rs::is_admin_role` 改回壞寫法，
            // 本守衛仍是綠的——因為**函式自己的名字** `fn is_admin_role(` 含有 `is_admin`，
            // 被當成 fallback。守衛在它存在理由的那個檔案裡失效，而且看起來完全正常。
            //
            // `"admin"` / `'admin'` 帶引號已足夠精確：權限碼字串是 `"admin.user.edit"`
            // （含 `"admin.` 而非 `"admin"`），不會誤命中。
            let has_fallback = window.contains("ROLE_ADMIN_LEGACY")
                || window.contains("\"admin\"")
                || window.contains("'admin'")
                || window.contains(".is_admin()");
            if !has_fallback {
                offenders.push(format!("{rel}:{}  {}", i + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "以下位置比對了 `SYSTEM_ADMIN` 卻沒有 legacy `admin` fallback，\n\
         那些分支在正式機上恆為 false（`roles` 表只有 `admin`，實查 + migration 皆可證）：\n\
         {}\n\n\
         修法：用 `CurrentUser::is_admin()`；SQL 用 `code = ANY(...)` 或 `IN ('admin','SYSTEM_ADMIN')`；\n\
         取管理員收件人用 `NotificationService::get_admin_users()`，\n\
         **不要**用 `get_users_by_role(ROLE_SYSTEM_ADMIN)`（恆回空清單，且不會報錯）。\n\
         若確實是「只有 SYSTEM_ADMIN 能做」的刻意區分，加進本檔的 INTENTIONAL 並說明理由。",
        offenders.join("\n")
    );
}
