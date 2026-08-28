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
//! 2. **判定**：該行有幾個 `SYSTEM_ADMIN`，就要有幾個豁免——
//!    同敘述內的 legacy fallback token，或正上方 3 行內的 `SYSTEM_ADMIN-ONLY` 標記。
//!    **每個豁免只能用一次**，見 [`violations`] 的說明。
//!
//! ## ⚠️ 這支守衛自己壞過三次，而且是同一個錯
//!
//! v1 用整個檔案豁免、v2 用「往下 3 行」與「同敘述有 token」豁免、
//! v3 的第一版又用「這行有沒有豁免」的 boolean 判斷——**每一次都是
//! 豁免的作用範圍大於它要豁免的那一件事**。
//!
//! 三次的發現方式都不同：檔案層是 CodeRabbit 指出、標記層是 CodeRabbit 指出、
//! fallback token 層是照著「還有哪裡用範圍」自己掃出來的、
//! 而 boolean 粒度那次是**本檔新增的回歸測試第一次跑就抓到**。
//!
//! 所以本檔除了掃真實原始碼，還用合成輸入直接測判定函式——
//! 「掃完沒找到違規」與「判定壞掉」在外觀上完全一樣，前者不能當成後者的證據。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 逐行豁免標記：候選行的前 3 行內出現它，就視為**刻意**只比對 `SYSTEM_ADMIN`。
///
/// ⚠️ **原本是用檔名清單豁免**（`src/services/user.rs` / `src/services/access.rs`），
/// 那讓整個檔案跳過掃描——**豁免範圍大於它要豁免的那件事**。
/// 那兩個檔各有一千多行，日後在裡面新增的任何一處未防護比對都會安靜通過，
/// 而測試看起來仍然涵蓋它們（CodeRabbit 於 #32 指出）。
///
/// 改成標記之後，豁免與被豁免的那一行綁在一起：同檔案的其他比對照樣被掃，
/// 而且理由寫在它旁邊，讀 code 的人看得到，不必翻到測試檔才知道為什麼。
///
/// 目前的使用者：角色指派授權「只有 SYSTEM_ADMIN 能指派 SYSTEM_ADMIN」
/// （`services/user.rs` 與 `services/access.rs` 各一半）。該角色不存在使它目前是
/// 惰性的但無害——legacy admin 分支才是實際生效的那條。
/// 若日後採「刪掉常數」的根治方案，這幾段要重新設計，無腦刪等於把該授權規則一併刪掉。
const INTENTIONAL_MARKER: &str = "SYSTEM_ADMIN-ONLY";

/// 讓一行「提到 SYSTEM_ADMIN」升級成「拿它跟什麼比對」的訊號。
const COMPARISON_SIGNALS: &[&str] = &[
    "==",
    ".contains(",
    "get_users_by_role(",
    ".bind(",
    "= '",
    "IN (",
    "ANY(",
];

/// 候選行所屬的**敘述**範圍（`[起, 迄)`，含註解行）。
///
/// ## 為什麼不能用「前後 N 行」
///
/// 本守衛原本判定「候選行 ±8 行內有沒有 fallback token」。2026-08-26 mutation 實測
/// 發現那條規則有一整類假陰性：
///
/// ```text
/// let assigns_system_admin = codes.iter().any(|c| c == ROLE_SYSTEM_ADMIN);   ← 候選
/// let assigns_legacy_admin = codes.iter().any(|c| c == ROLE_ADMIN_LEGACY);   ← 不同變數
/// ```
///
/// 第二行的 fallback token 屬於**另一個判斷**，跟第一行毫無關係，但落在 8 行內
/// 就讓第一行通過。淨結果：**任何新增的未防護比對，只要寫在既有防護比對旁邊就滑過去**
/// ——那正是本守衛存在理由的那種 bug。守衛看起來在守，實際上放行。
///
/// 收緊成敘述範圍之後，fallback 必須跟候選比對在**同一個運算式**裡才算數。
/// 代價是 fallback 真的分散在多個敘述的地方（`bin/create_admin.rs` 的兩段式查詢）
/// 需要顯式標記——那是好事：那種寫法本來就該讓讀的人看見。
///
/// 邊界判定：往前找到上一個以 `;` `{` `}` 結尾或空白的行，往後找到第一個以 `;` 結尾的行。
/// 註解行不算邊界（會被跨過），但在比對 fallback 時會被濾掉——
/// 否則標記註解裡的字就會被當成 fallback。
fn statement_span(lines: &[&str], i: usize) -> (usize, usize) {
    let mut lo = i;
    while lo > 0 {
        let prev = lines[lo - 1].trim_end();
        if prev.trim_start().starts_with("//") {
            lo -= 1;
            continue;
        }
        if prev.is_empty()
            || prev.ends_with(';')
            || prev.ends_with('{')
            || prev.ends_with('}')
            || prev.ends_with(',')
        {
            break;
        }
        lo -= 1;
    }
    let mut hi = i + 1;
    while hi < lines.len() {
        let prev = lines[hi - 1].trim_end();
        if prev.ends_with(';') || prev.ends_with('{') || prev.ends_with('}') {
            break;
        }
        hi += 1;
    }
    (lo, hi.min(lines.len()))
}

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

/// 一個檔案裡所有「比對了 `SYSTEM_ADMIN` 卻沒有 legacy fallback」的位置。
///
/// # 豁免是**可消耗的資源**，不是範圍
///
/// 本函式的前兩版都栽在同一件事上：**豁免範圍大於它要豁免的那一件事**。
///
/// | 版本 | 豁免單位 | 失效方式 |
/// |---|---|---|
/// | v1 | 整個檔案（`INTENTIONAL` 檔名清單） | 該檔日後新增的任何比對都自動過關 |
/// | v2 | 標記往下 3 行 | 一個標記豁免掉那 3 行內的**每一個**比對 |
/// | v2 | 同一敘述內有 fallback token | 一個 token 豁免掉該敘述內的**每一個**比對 |
/// | v3 | **一次消耗一個** | 兩個比對要兩個豁免 |
///
/// 前兩次都是 CodeRabbit 在 #32 指出（第 1、2 輪），形狀相同、層級不同。
/// v3 改成「每個標記、每個 fallback token 各只能保護一個比對」，
/// 才真正把「範圍」換成「配對」。
fn violations(rel: &str, text: &str) -> Vec<String> {
    if !text.contains("SYSTEM_ADMIN") {
        return Vec::new();
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut consumed_markers: HashSet<usize> = HashSet::new();
    let mut fallback_budget: HashMap<(usize, usize), usize> = HashMap::new();

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
        let is_comparison = COMPARISON_SIGNALS.iter().any(|sig| line.contains(sig));
        if !is_comparison {
            continue;
        }

        // ⚠️ 一行可以有**兩個**比對（`a == A || b == A`），所以要的是「幾個」豁免，
        // 不是「有沒有」豁免。這一點是本檔自己的回歸測試抓到的
        // ——先前版本逐行判斷，同一行的第二個比對會白拿第一個的豁免。
        let need = line.matches("SYSTEM_ADMIN").count();
        let mut covered = 0usize;

        // ① 逐行標記，**且每個標記只能用一次**。往上最多 3 行找還沒被消耗的。
        let mark_lo = i.saturating_sub(3);
        while covered < need {
            let Some(m) = (mark_lo..i)
                .rev()
                .find(|m| lines[*m].contains(INTENTIONAL_MARKER) && !consumed_markers.contains(m))
            else {
                break;
            };
            consumed_markers.insert(m);
            covered += 1;
        }

        // ② 同一敘述內的 fallback token，**每個也只能保護一個比對**。
        let span = statement_span(&lines, i);
        let budget = fallback_budget
            .entry(span)
            .or_insert_with(|| count_fallbacks(&lines[span.0..span.1]));
        while covered < need && *budget > 0 {
            *budget -= 1;
            covered += 1;
        }

        if covered < need {
            out.push(format!(
                "{rel}:{}  ({covered}/{need} 有豁免)  {}",
                i + 1,
                line.trim()
            ));
        }
    }
    out
}

/// 一段程式碼裡「legacy fallback」出現幾次（註解行不算）。
///
/// ⚠️ 必須是 `.is_admin()` 這個**呼叫**，不能只找子字串 `is_admin`。
/// 2026-08-26 mutation 實測：把 `handlers/mcp.rs::is_admin_role` 改回壞寫法，
/// 本守衛仍是綠的——因為**函式自己的名字** `fn is_admin_role(` 含有 `is_admin`，
/// 被當成 fallback。守衛在它存在理由的那個檔案裡失效，而且看起來完全正常。
///
/// `"admin"` / `'admin'` 帶引號已足夠精確：權限碼字串是 `"admin.user.edit"`
/// （含 `"admin.` 而非 `"admin"`），不會誤命中。
fn count_fallbacks(lines: &[&str]) -> usize {
    lines
        .iter()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| {
            l.matches("ROLE_ADMIN_LEGACY").count()
                + l.matches("\"admin\"").count()
                + l.matches("'admin'").count()
                + l.matches(".is_admin()").count()
        })
        .sum()
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
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        offenders.extend(violations(&rel, &text));
    }

    assert!(
        offenders.is_empty(),
        "以下位置比對了 `SYSTEM_ADMIN` 卻沒有 legacy `admin` fallback，
         那些分支在正式機上恆為 false（`roles` 表只有 `admin`，實查 + migration 皆可證）：
         {}

         修法：用 `CurrentUser::is_admin()`；SQL 用 `code = ANY(...)` 或 `IN ('admin','SYSTEM_ADMIN')`；
         取管理員收件人用 `NotificationService::get_admin_users()`，
         **不要**用 `get_users_by_role(ROLE_SYSTEM_ADMIN)`（恆回空清單，且不會報錯）。
         若確實是「只有 SYSTEM_ADMIN 能做」的刻意區分，在該行正上方加一行註解含
         `SYSTEM_ADMIN-ONLY` 並寫明理由——**一個標記只保護一個比對**。",
        offenders.join("
")
    );
}

// ── 守衛自己的判定測試 ──
//
// ⚠️ 在這些之前，本守衛**只有一種驗證方式：跑在真實原始碼上，然後看它是綠的**。
// 綠的意思是「沒找到違規」，而那跟「判定壞掉」外觀完全一樣。
// 前兩版的假陰性都是靠 mutation 或 CodeRabbit 才發現，不是靠它自己。
//
// 下面用合成輸入直接餵判定函式，每一條都對應一個**曾經真的漏掉**的形狀。

#[test]
fn one_marker_protects_exactly_one_comparison() {
    // CodeRabbit 於 #32 第 2 輪指出：標記原本豁免「接下來 3 行內的每一個」比對。
    let src = "        // SYSTEM_ADMIN-ONLY：刻意的
        let a = codes.iter().any(|c| c == ROLE_SYSTEM_ADMIN);
        let b = other.iter().any(|c| c == ROLE_SYSTEM_ADMIN);
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "一個標記只該保護第一個比對，第二個必須被抓到。實際：{v:?}"
    );
    assert!(v[0].contains("t.rs:3"), "被抓的應是第二個。實際：{v:?}");
}

#[test]
fn one_fallback_token_protects_exactly_one_comparison() {
    // 同一族的另一半：同敘述內有 fallback 就豁免整個敘述。
    let src = "        let x = a == ROLE_SYSTEM_ADMIN || b == ROLE_ADMIN_LEGACY || c == ROLE_SYSTEM_ADMIN;
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "一行兩個比對只有一個 fallback，第二個必須被抓到。實際：{v:?}"
    );
}

#[test]
fn paired_comparison_and_fallback_is_clean() {
    let src = "        let x = r == ROLE_SYSTEM_ADMIN || r == ROLE_ADMIN_LEGACY;
";
    assert!(violations("t.rs", src).is_empty(), "標準寫法不該被誤報");
}

#[test]
fn bind_pair_across_lines_is_clean() {
    // `.bind(A)` / `.bind(B)` 相鄰是既有的常見寫法，必須被 statement_span 涵蓋。
    let src = "        let q = sqlx::query(SQL)
            .bind(ROLE_SYSTEM_ADMIN)
            .bind(ROLE_ADMIN_LEGACY)
            .fetch_all(pool);
";
    assert!(
        violations("t.rs", src).is_empty(),
        "跨行的 bind 配對不該被誤報"
    );
}

#[test]
fn mentions_without_comparison_are_not_candidates() {
    // v1 的 6 個假陽性全部是這一類：註解、use、顯示用回傳值。
    let src = "        // 說明裡提到 SYSTEM_ADMIN
        use crate::constants::ROLE_SYSTEM_ADMIN;
        let label = Some(ROLE_SYSTEM_ADMIN);
";
    assert!(
        violations("t.rs", src).is_empty(),
        "只是提到、沒有比對的地方不該被判違規"
    );
}

#[test]
fn function_named_is_admin_does_not_count_as_fallback() {
    // v2 的漏報：`fn is_admin_role(` 含子字串 `is_admin`，被當成 fallback，
    // 守衛因此在它存在理由的那個檔案裡失效。
    let src = "        fn is_admin_role(user: &CurrentUser) -> bool {
            user.roles.iter().any(|r| r == ROLE_SYSTEM_ADMIN)
        }
";
    assert_eq!(
        violations("t.rs", src).len(),
        1,
        "函式名裡的 is_admin 不是 fallback，這個比對必須被抓到"
    );
}
