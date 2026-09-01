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
//! ## ⚠️ 這支守衛自己壞過五次，而且每次都是同一個錯
//!
//! **豁免的作用範圍大於它要豁免的那一件事。**
//!
//! | # | 豁免/判定單位 | 失效方式 | 誰發現 |
//! |---|---|---|---|
//! | v1 | 整個檔案（檔名清單） | 該檔日後新增的任何比對自動過關 | CodeRabbit 第 1 輪 |
//! | v2 | 標記往下 3 行／同敘述有 token | 一個豁免蓋掉範圍內的**每一個**比對 | CodeRabbit 第 2 輪 + 自己掃 |
//! | v3 | 逐行 boolean「有沒有豁免」 | 同一行的第二個比對白拿第一個的豁免 | 本檔的回歸測試 |
//! | v4 | 判定直接吃整行（未剝 `//`） | 行尾註解讓敘述併吞下一個、註解裡的 token 被當 fallback | CodeRabbit 第 4 輪 |
//! | v5 | 只剝了 `//`，沒剝 `/* */` | 同 v4，換成區塊註解就照樣成立 | CodeRabbit 第 5 輪 |
//!
//! v5 特別值得記：v4 的修正**看起來完整**（三個出口一起補、三支回歸測試、
//! mutation 也驗過），但它只涵蓋了兩種 Rust 註解裡的一種。
//! **補一半的修正與補完整的修正，在測試結果上完全一樣。**
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

/// 把整份原始碼變成「只剩程式碼」的逐行版本：`//` 行註解與 `/* … */` 區塊註解
/// 都拿掉（區塊註解可跨行、可巢狀），字串字面值內的內容原樣保留。
///
/// 回傳長度與 `text.lines()` 相同，索引一一對應。
///
/// ## ⚠️ 為什麼判定一定要先過這一關
///
/// 註解只是「提到」，不是程式碼。直接吃整行會讓判定從三個方向壞掉，
/// 而且**三個方向都是同一件事：豁免的作用範圍大於它要豁免的那一件事**：
///
/// | 出口 | 沒剝註解的後果 |
/// |---|---|
/// | [`statement_span`] 邊界 | `…; // 說明` 不以 `;` 結尾 → 併吞下一敘述，白拿它的 fallback |
/// | [`count_fallbacks`] | 註解裡的 `"admin"` / `ROLE_ADMIN_LEGACY` 被當成真的 fallback |
/// | [`violations`] 候選 | 只在註解裡提到 `SYSTEM_ADMIN` 的行被判成比對（假陽性） |
///
/// 兩輪各補一半：**行註解**是 CodeRabbit 於 #32 第 4 輪指出，
/// **區塊註解**是第 5 輪指出——第 4 輪的修法只處理 `//`，
/// `let a = r == ROLE_SYSTEM_ADMIN; /* ROLE_ADMIN_LEGACY */` 照樣過關。
/// 補一半的修正在外觀上與補完整完全一樣（測試全綠），這正是本檔反覆在講的事。
///
/// ## 已知極限（刻意不做完美的 Rust lexer）
///
/// 字串追蹤只認雙引號與 `\` 跳脫。**raw string（`r#"…"#`）內若含單獨的 `"`
/// 會讓引號配對錯位**，此時該行的 `//` 可能被誤剝。方向是**少看程式碼**＝
/// 可能漏報而非誤報。本守衛掃的是 `backend/src`，該處目前沒有這種寫法
/// （新增後若掃真實原始碼那支測試仍綠、但你確信該行該被抓，先查這裡）。
fn strip_comments(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    // Rust 的區塊註解可巢狀（`/* /* */ */`），所以要記深度不是布林。
    let mut depth = 0usize;
    for line in text.lines() {
        let b = line.as_bytes();
        let mut kept = String::with_capacity(line.len());
        let mut in_str = false;
        let mut i = 0usize;
        // 目前這段「要保留」的起點；遇到註解開頭就把 `[seg, i)` 收進 kept。
        let mut seg = 0usize;
        while i < b.len() {
            if depth > 0 {
                if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    i += 2;
                    seg = i;
                } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            match b[i] {
                // 跳過跳脫序列，`\"` 不能被當成字串結束。
                // UTF-8 是自我同步的（後續位元組皆 >= 0x80），跳進多位元組字元中間
                // 也不會誤命中任何 ASCII 標記，切片位置仍落在字元邊界上。
                b'\\' if in_str => i += 2,
                b'"' => {
                    in_str = !in_str;
                    i += 1;
                }
                b'/' if !in_str && b.get(i + 1) == Some(&b'/') => {
                    kept.push_str(&line[seg..i]);
                    seg = line.len();
                    i = b.len();
                }
                b'/' if !in_str && b.get(i + 1) == Some(&b'*') => {
                    kept.push_str(&line[seg..i]);
                    depth += 1;
                    i += 2;
                    seg = i;
                }
                _ => i += 1,
            }
        }
        // 行尾仍在區塊註解內 → 尾巴整段是註解，不保留。
        if depth == 0 && seg < line.len() {
            kept.push_str(&line[seg..]);
        }
        out.push(kept);
    }
    out
}

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
///
/// 吃的是 [`strip_comments`] 產出的**無註解**版本；`comment_only[i]` 標記
/// 「原本非空、剝完只剩空白」的行（整行都是註解）——那種行要**跨過**，
/// 不能當成邊界，否則敘述中間插一行註解就會把敘述切斷。
fn statement_span(code: &[String], comment_only: &[bool], i: usize) -> (usize, usize) {
    let mut lo = i;
    while lo > 0 {
        if comment_only[lo - 1] {
            lo -= 1;
            continue;
        }
        let prev = code[lo - 1].trim_end();
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
    while hi < code.len() {
        let prev = code[hi - 1].trim_end();
        if prev.ends_with(';') || prev.ends_with('{') || prev.ends_with('}') {
            break;
        }
        hi += 1;
    }
    (lo, hi.min(code.len()))
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
    // 一切判定都跑在無註解版本上；原始行只用於「找豁免標記」（標記本來就寫在註解裡）
    // 與錯誤訊息的顯示。
    let stripped = strip_comments(text);
    let comment_only: Vec<bool> = lines
        .iter()
        .zip(&stripped)
        .map(|(raw, c)| c.trim().is_empty() && !raw.trim().is_empty())
        .collect();
    let mut out = Vec::new();
    let mut consumed_markers: HashSet<usize> = HashSet::new();
    let mut fallback_budget: HashMap<(usize, usize), usize> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let code = stripped[i].as_str();
        // use 行只是「提到」，不是比對。（純註解行剝完是空的，自然不會進來。）
        if code.trim_start().starts_with("use ") {
            continue;
        }
        if !code.contains("SYSTEM_ADMIN") {
            continue;
        }
        // 比對訊號：沒有這些的話（例如 `Some(ROLE_SYSTEM_ADMIN)` 這種顯示用回傳值）
        // 就不是一個「拿它跟使用者角色比對」的地方。
        let is_comparison = COMPARISON_SIGNALS.iter().any(|sig| code.contains(sig));
        if !is_comparison {
            continue;
        }

        // ⚠️ 一行可以有**兩個**比對（`a == A || b == A`），所以要的是「幾個」豁免，
        // 不是「有沒有」豁免。這一點是本檔自己的回歸測試抓到的
        // ——先前版本逐行判斷，同一行的第二個比對會白拿第一個的豁免。
        let need = code.matches("SYSTEM_ADMIN").count();
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
        let span = statement_span(&stripped, &comment_only, i);
        let budget = fallback_budget
            .entry(span)
            .or_insert_with(|| count_fallbacks(&stripped[span.0..span.1]));
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
///
/// ⚠️ 吃的是 [`strip_comments`] 的產出，所以**任何形式的註解都不算**——
/// 只寫著「日後再改用 `ROLE_ADMIN_LEGACY`」的說明不是 fallback，
/// 卻曾經能豁免掉旁邊那個真的未防護的比對（`//` 見第 4 輪、`/* */` 見第 5 輪）。
fn count_fallbacks(lines: &[String]) -> usize {
    lines
        .iter()
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
fn trailing_comment_does_not_merge_adjacent_statements() {
    // CodeRabbit 於 #32 第 4 輪指出：行尾註解讓候選行不以 `;` 結尾，
    // statement_span 於是把下一個敘述併進來，白拿它的 fallback token。
    let src = "        let a = r == ROLE_SYSTEM_ADMIN; // 這裡刻意留個說明
        let b = other == ROLE_ADMIN_LEGACY;
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "下一個敘述的 fallback 不屬於這個比對，必須被抓到。實際：{v:?}"
    );
    assert!(v[0].contains("t.rs:1"), "被抓的應是第一行。實際：{v:?}");
}

#[test]
fn fallback_token_inside_comment_does_not_count() {
    // 只是「寫著日後要加 fallback」的註解，不是 fallback。
    let src = "        let a = r == ROLE_SYSTEM_ADMIN; // TODO: 之後補 ROLE_ADMIN_LEGACY
";
    assert_eq!(
        violations("t.rs", src).len(),
        1,
        "註解裡的 fallback token 不該豁免真的未防護比對"
    );
}

#[test]
fn comparison_mentioned_only_in_trailing_comment_is_not_a_candidate() {
    // 反向：真正的比對已改掉，SYSTEM_ADMIN 只留在行尾註解裡。
    //
    // ⚠️ 程式碼側刻意**不含任何 fallback token**。原本寫的是 `user.is_admin()`，
    // 那樣即使剝註解的邏輯壞掉，這行也會因為 `.is_admin()` 被算成 fallback 而過關
    // ——測試是綠的，但綠的理由跟它要測的事無關（mutation 實測）。
    let src = "        let ok = user.roles.iter().any(|r| r == ROLE_VET); // 取代原本的 r == ROLE_SYSTEM_ADMIN
";
    assert!(
        violations("t.rs", src).is_empty(),
        "只在行尾註解提到的比對不該被判違規"
    );
}

#[test]
fn fallback_token_inside_block_comment_does_not_count() {
    // CodeRabbit 於 #32 第 5 輪指出：第 4 輪只剝了 `//`，`/* */` 照樣被當成程式碼。
    let src = "        let a = r == ROLE_SYSTEM_ADMIN; /* 之後補 ROLE_ADMIN_LEGACY */
";
    assert_eq!(
        violations("t.rs", src).len(),
        1,
        "區塊註解裡的 fallback token 不該豁免真的未防護比對"
    );
}

#[test]
fn block_comment_does_not_merge_adjacent_statements() {
    // 與行尾註解同一件事：行尾的 `/* */` 讓該行不以 `;` 結尾，敘述範圍就會併吞下一個。
    let src = "        let a = r == ROLE_SYSTEM_ADMIN; /* 說明 */
        let b = other == ROLE_ADMIN_LEGACY;
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "下一個敘述的 fallback 不屬於這個比對，必須被抓到。實際：{v:?}"
    );
    assert!(v[0].contains("t.rs:1"), "被抓的應是第一行。實際：{v:?}");
}

#[test]
fn code_inside_multi_line_block_comment_is_not_a_candidate() {
    // 被整段註解掉的程式碼不是程式碼——區塊註解要能跨行，否則中間那行會被當成候選。
    let src = "        /* 這段先關掉
        let a = r == ROLE_SYSTEM_ADMIN;
        */
        let b = user.is_admin();
";
    assert!(
        violations("t.rs", src).is_empty(),
        "被註解掉的比對不該被判違規"
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
