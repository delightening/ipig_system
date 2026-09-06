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
//!    （`==` / `.contains(` / `get_users_by_role(` / `.bind(` / SQL 的 `= '` / `ANY(` /
//!    大小寫與空白皆不拘的 SQL `IN`，見 [`contains_sql_in_operator`]）。
//!    純粹「提到」它的行（註解、import、`Some(ROLE_SYSTEM_ADMIN)` 這種回傳值）不算。
//! 2. **判定**：該行有幾個 `SYSTEM_ADMIN`，就要有幾個豁免——
//!    同敘述內的 legacy fallback token，或正上方 3 行內、**真的在註解裡**的
//!    `SYSTEM_ADMIN-ONLY` 標記。**每個豁免只能用一次**，見 [`violations`] 的說明。
//!
//! ## ⚠️ 這支守衛自己壞過八次，分屬兩個失效族群
//!
//! **族群一：豁免的作用範圍大於它要豁免的那一件事**（v1–v5、v6b、v8b）。
//! **族群二：候選判定本身漏看了一種寫法**（v6a、v8a）——該處從一開始就沒被當成
//! 「拿 SYSTEM_ADMIN 跟什麼比對」，連豁免規則都還沒輪到就已經看不見。
//!
//! | # | 判定/豁免單位 | 失效方式 | 誰發現 |
//! |---|---|---|---|
//! | v1 | 整個檔案（檔名清單） | 該檔日後新增的任何比對自動過關 | CodeRabbit 第 1 輪 |
//! | v2 | 標記往下 3 行／同敘述有 token | 一個豁免蓋掉範圍內的**每一個**比對 | CodeRabbit 第 2 輪 + 自己掃 |
//! | v3 | 逐行 boolean「有沒有豁免」 | 同一行的第二個比對白拿第一個的豁免 | 本檔的回歸測試 |
//! | v4 | 判定直接吃整行（未剝 `//`） | 行尾註解讓敘述併吞下一個、註解裡的 token 被當 fallback | CodeRabbit 第 4 輪 |
//! | v5 | 只剝了 `//`，沒剝 `/* */` | 同 v4，換成區塊註解就照樣成立 | CodeRabbit 第 5 輪 |
//! | v6a | `IN (` 固定大小寫＋固定一個空白 | 小寫或無空白的 SQL `IN` 偵測不到候選 | CodeRabbit 第 6 輪 |
//! | v6b | 標記比對吃原始行、未排除字串字面值 | `let x = "SYSTEM_ADMIN-ONLY";` 被當成刻意標記 | CodeRabbit 第 6 輪 |
//! | v8a | 字串狀態只追雙引號，不認 char 字面值 | `'"'` 讓 `in_str` 翻轉，該行的 `//` 不再被當註解 | CodeRabbit 第 8 輪 |
//! | v8b | 敘述邊界只在**行**邊界停 | 同一行兩個 `;` 敘述共用一份 fallback 預算 | CodeRabbit 第 8 輪 |
//!
//! v6a／v8a 提醒同一件事：本檔多數教訓都在講「豁免」，但守衛的第一道防線是
//! **輸入的切分與候選判定**——切錯或漏看的寫法，連豁免規則都沒有機會出錯，
//! 因為它從來沒被列入判定。
//!
//! v5 也值得記：v4 的修正**看起來完整**（三個出口一起補、三支回歸測試、
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
///
/// ⚠️ SQL 的 `IN` 不在這裡——它大小寫與空白都不固定，字面值清單表達不了，
/// 另外用 [`contains_sql_in_operator`] 判定。
const COMPARISON_SIGNALS: &[&str] = &[
    "==",
    ".contains(",
    "get_users_by_role(",
    ".bind(",
    "= '",
    "ANY(",
];

/// 偵測 SQL 的 `IN` 運算子——不分大小寫、`IN` 與 `(` 之間可以有 0 個以上空白。
///
/// CodeRabbit 於 #32 第 6 輪指出：舊版 `COMPARISON_SIGNALS` 裡的 `"IN ("` 是固定
/// 大小寫、固定一個空白的字面值，`WHERE code in ('SYSTEM_ADMIN')`（小寫）或
/// `IN('SYSTEM_ADMIN')`（無空白）都偵測不到——沒有其他比對訊號的裸 SQL 行，
/// 就完全不會被判定為候選，等於這條分支對守衛整個隱形（不是豁免太寬，
/// 是候選判定本身有洞，見檔頭 v6a）。
///
/// 用「前一個字元不是英數字/底線」當單字邊界，避免誤配 `WITHIN(`、`MIXIN(`、
/// `PIN(` 這類子字串。只比較 byte 值、不切片，多位元組字元混在裡面也不影響正確性。
fn contains_sql_in_operator(code: &str) -> bool {
    let bytes = code.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while i + 1 < bytes.len() {
        let is_in = (bytes[i] | 0x20) == b'i' && (bytes[i + 1] | 0x20) == b'n';
        let boundary_before = i == 0 || !is_word(bytes[i - 1]);
        if is_in && boundary_before {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// `b[start]` 是 `'` 時，判斷它開啟的是 **char 字面值**還是 **lifetime**；
/// 是字面值就回傳結尾那個 `'` 的索引，是 lifetime 回傳 `None`。
///
/// ⚠️ **這個分辨不能省**（CodeRabbit 於 #32 第 8 輪指出）。
/// [`strip_comments`] 原本只追雙引號，於是 `'"'` 這個合法的 char 字面值裡的雙引號
/// 會把 `in_str` 翻成 true，該行後面的 `//` 就不再被視為註解開頭——
/// `let q = '"'; let ok = r == ROLE_SYSTEM_ADMIN; // ROLE_ADMIN_LEGACY`
/// 整段註解被當成程式碼，[`count_fallbacks`] 數到註解裡的 token，比對就白拿豁免。
///
/// 但**不能無腦把每個 `'` 都當字面值**——Rust 的 lifetime（`'a`、`'static`、
/// `&'a str`）用的是同一個字元。把 lifetime 當成字面值開頭會吃掉後面一大段程式碼，
/// 破壞得比原本的 bug 更嚴重。判準是「配對的 `'` 必須落在字面值文法允許的**確切位置**」，
/// 不是「附近有沒有另一個 `'`」——後者會被 `&'a str = "it's"` 這種寫法騙過去。
fn char_literal_end(b: &[u8], start: usize) -> Option<usize> {
    let mut j = start + 1;
    if j >= b.len() {
        return None;
    }
    if b[j] == b'\\' {
        // 跳脫序列：'\n' '\\' '\'' '\xNN' '\u{...}'
        j += 1;
        if j >= b.len() {
            return None;
        }
        match b[j] {
            b'x' => j += 3,
            b'u' => {
                while j < b.len() && b[j] != b'}' {
                    j += 1;
                }
                j += 1;
            }
            _ => j += 1,
        }
    } else {
        // 一般字元，可能是多位元組 UTF-8：跳過 lead byte 之後的所有 continuation byte
        j += 1;
        while j < b.len() && (b[j] & 0xC0) == 0x80 {
            j += 1;
        }
    }
    if j < b.len() && b[j] == b'\'' {
        Some(j)
    } else {
        None
    }
}

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
/// 字串追蹤只認雙引號、`\` 跳脫與 char 字面值（見 [`char_literal_end`]）。
/// **raw string（`r#"…"#`）內若含單獨的 `"` 會讓引號配對錯位**，此時該行的 `//`
/// 可能被誤剝。方向是**少看程式碼**＝可能漏報而非誤報。本守衛掃的是 `backend/src`，
/// 該處目前沒有這種寫法（新增後若掃真實原始碼那支測試仍綠、但你確信該行該被抓，先查這裡）。
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
                // char 字面值整段跳過，裡面的 `"` 不能翻轉字串狀態（見 char_literal_end）。
                // lifetime 的 `'` 會回 None，當成普通字元前進一格。
                b'\'' if !in_str => match char_literal_end(b, i) {
                    Some(end) => i = end + 1,
                    None => i += 1,
                },
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

/// 把一行（已剝註解）依 `;` 切成敘述片段；字串字面值與 char 字面值內的 `;` 不算分隔點。
///
/// ⚠️ **一行可以有兩個敘述**（CodeRabbit 於 #32 第 8 輪指出，標為 Major）。
/// [`statement_span`] 只在行邊界停，於是
/// `let label = "admin"; let x = r == ROLE_SYSTEM_ADMIN;`
/// 這一行裡，前一個敘述的 `"admin"` 會被算進後一個比對的 fallback 預算——
/// **前後兩個敘述共用一份豁免**。這是 v1–v6 那個老毛病的第七次變形：
/// 豁免的作用範圍（整行）大於它要豁免的那一件事（單一敘述）。
///
/// 只有一個片段＝這行沒有行內分號，此時沿用原本的跨行 [`statement_span`]；
/// 有多個片段才改用「片段內」的預算，避免影響既有的跨行 `.bind()` 配對寫法。
fn split_statements(code: &str) -> Vec<String> {
    let b = code.as_bytes();
    let mut out = Vec::new();
    let mut in_str = false;
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b'\\' if in_str => i += 2,
            b'\'' if !in_str => match char_literal_end(b, i) {
                Some(end) => i = end + 1,
                None => i += 1,
            },
            b'"' => {
                in_str = !in_str;
                i += 1;
            }
            b';' if !in_str => {
                out.push(code[seg_start..=i].to_string());
                i += 1;
                seg_start = i;
            }
            _ => i += 1,
        }
    }
    if seg_start < code.len() {
        out.push(code[seg_start..].to_string());
    }
    if out.is_empty() {
        out.push(String::new());
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
    // key 的第三格區分兩種預算來源：true = 行內片段 (行號, 片段序號)，
    // false = 跨行敘述範圍 (span.0, span.1)。兩者的前兩格意義不同，不能混用同一個 key 空間。
    let mut fallback_budget: HashMap<(usize, usize, bool), usize> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let line_code = stripped[i].as_str();
        // use 行只是「提到」，不是比對。（純註解行剝完是空的，自然不會進來。）
        if line_code.trim_start().starts_with("use ") {
            continue;
        }
        if !line_code.contains("SYSTEM_ADMIN") {
            continue;
        }

        // 一行可能是多個以 `;` 分隔的敘述，每個敘述要各自算豁免預算（見 split_statements）。
        // 只有一個片段時代表沒有行內分號，沿用原本的跨行 statement_span。
        let segments = split_statements(line_code);
        let multi = segments.len() > 1;

        for (seg_idx, code) in segments.iter().enumerate() {
            let code = code.as_str();
            if !code.contains("SYSTEM_ADMIN") {
                continue;
            }
            // 比對訊號：沒有這些的話（例如 `Some(ROLE_SYSTEM_ADMIN)` 這種顯示用回傳值）
            // 就不是一個「拿它跟使用者角色比對」的地方。
            let is_comparison = COMPARISON_SIGNALS.iter().any(|sig| code.contains(sig))
                || contains_sql_in_operator(code);
            if !is_comparison {
                continue;
            }

            // ⚠️ 一個敘述可以有**兩個**比對（`a == A || b == A`），所以要的是「幾個」豁免，
            // 不是「有沒有」豁免。這一點是本檔自己的回歸測試抓到的
            // ——先前版本逐行判斷，同一行的第二個比對會白拿第一個的豁免。
            let need = code.matches("SYSTEM_ADMIN").count();
            let mut covered = 0usize;

            // ① 逐行標記，**且每個標記只能用一次**。往上最多 3 行找還沒被消耗的。
            //
            // ⚠️ 標記必須**真的在註解裡**（CodeRabbit 於 #32 第 6 輪指出，v6b）：
            // 只檢查 raw line 含不含這個子字串，`let note = "SYSTEM_ADMIN-ONLY";`
            // 這種字串字面值也會命中，把它當成刻意標記去豁免旁邊真的未防護的比對。
            // 判準是「raw 有、stripped（無註解版）沒有」——代表這段文字是被
            // strip_comments 剝掉的註解，不是留在程式碼裡的字串內容。
            let mark_lo = i.saturating_sub(3);
            while covered < need {
                let Some(m) = (mark_lo..i).rev().find(|m| {
                    lines[*m].contains(INTENTIONAL_MARKER)
                        && !stripped[*m].contains(INTENTIONAL_MARKER)
                        && !consumed_markers.contains(m)
                }) else {
                    break;
                };
                consumed_markers.insert(m);
                covered += 1;
            }

            // ② fallback token，**每個也只能保護一個比對**。
            //
            // 這一行有行內分號（多個敘述）時，預算只從**該片段**算——否則前一個敘述的
            // `"admin"` 會保護到後一個敘述的比對（第 8 輪的 Major finding）。
            // 沒有行內分號時沿用跨行 statement_span，維持既有的 `.bind()` 跨行配對行為。
            let (key, initial) = if multi {
                ((i, seg_idx, true), count_fallbacks_in(code))
            } else {
                let span = statement_span(&stripped, &comment_only, i);
                (
                    (span.0, span.1, false),
                    count_fallbacks(&stripped[span.0..span.1]),
                )
            };
            let budget = fallback_budget.entry(key).or_insert(initial);
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
    lines.iter().map(|l| count_fallbacks_in(l)).sum()
}

/// 單一段文字裡的 fallback token 數。[`count_fallbacks`] 逐行呼叫它；
/// 敘述片段（見 [`split_statements`]）則直接呼叫。
fn count_fallbacks_in(code: &str) -> usize {
    code.matches("ROLE_ADMIN_LEGACY").count()
        + code.matches("\"admin\"").count()
        + code.matches("'admin'").count()
        + code.matches(".is_admin()").count()
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
fn sql_in_operator_detection_is_case_and_space_insensitive() {
    // CodeRabbit 於 #32 第 6 輪指出：舊版 `"IN ("` 字面值偵測不到小寫或無空白的寫法。
    assert!(contains_sql_in_operator("WHERE code IN ('SYSTEM_ADMIN')"));
    assert!(contains_sql_in_operator("WHERE code in ('SYSTEM_ADMIN')"));
    assert!(contains_sql_in_operator("WHERE code IN('SYSTEM_ADMIN')"));
    assert!(contains_sql_in_operator(
        "WHERE code   IN   (  'SYSTEM_ADMIN'  )"
    ));
}

#[test]
fn sql_in_operator_detection_respects_word_boundary() {
    // 前一個字元是英數字/底線時不能算——否則 WITHIN(/MIXIN(/PIN( 都會誤判成 IN 運算子。
    assert!(!contains_sql_in_operator("x.within(SYSTEM_ADMIN_RANGE)"));
    assert!(!contains_sql_in_operator("MIXIN(Foo)"));
    assert!(!contains_sql_in_operator("PIN(1234)"));
}

#[test]
fn lowercase_sql_in_clause_is_a_candidate() {
    // 端到端驗證：光有 helper 綠燈不夠，還要確認它真的接進 violations() 的判定路徑。
    let src = "        let sql = \"WHERE code in ('SYSTEM_ADMIN')\";
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "小寫、無 fallback 的 IN 子句必須被判為候選，不能因為大小寫而對守衛隱形。實際：{v:?}"
    );
}

#[test]
fn marker_inside_string_literal_is_not_recognized() {
    // CodeRabbit 於 #32 第 6 輪指出（v6b）：標記比對只看 raw line 含不含子字串，
    // 沒有排除字串字面值——`let note = "SYSTEM_ADMIN-ONLY";` 會被當成刻意標記，
    // 豁免掉旁邊真的未防護的比對。
    let src = "        let note = \"SYSTEM_ADMIN-ONLY\";
        let a = codes.iter().any(|c| c == ROLE_SYSTEM_ADMIN);
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "字串字面值裡的 SYSTEM_ADMIN-ONLY 不是標記，這個比對必須被抓到。實際：{v:?}"
    );
}

#[test]
fn char_literal_quote_does_not_break_comment_stripping() {
    // CodeRabbit 於 #32 第 8 輪指出：'"' 這個合法 char 字面值裡的雙引號
    // 會把 in_str 翻成 true，之後的 // 不再被當註解，註解裡的 token 就成了假 fallback。
    let src = "        let q = '\"'; let ok = r == ROLE_SYSTEM_ADMIN; // ROLE_ADMIN_LEGACY
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "char 字面值裡的雙引號不該讓行尾註解變成程式碼。實際：{v:?}"
    );
}

#[test]
fn lifetime_is_not_mistaken_for_char_literal() {
    // 反向防護：char 字面值的判定不能把 lifetime 的 ' 吃掉，
    // 否則會跳過後面一大段程式碼，破壞得比原本的 bug 更嚴重。
    assert_eq!(char_literal_end(b"'a'", 0), Some(2), "'a' 是字面值");
    assert_eq!(char_literal_end(b"'\\n'", 0), Some(3), "'\\n' 是字面值");
    assert_eq!(char_literal_end(b"'static", 0), None, "'static 是 lifetime");
    assert_eq!(char_literal_end(b"&'a str", 1), None, "&'a str 是 lifetime");

    // 端到端：帶 lifetime 的簽章裡若有未防護比對，仍要抓得到。
    let src = "        fn f<'a>(r: &'a str) -> bool { r == ROLE_SYSTEM_ADMIN }
";
    let v = violations("t.rs", src);
    assert_eq!(v.len(), 1, "lifetime 不該讓這個比對隱形。實際：{v:?}");
}

#[test]
fn two_statements_on_one_line_do_not_share_fallback() {
    // CodeRabbit 於 #32 第 8 輪指出（Major）：statement_span 只在行邊界停，
    // 前一個敘述的 "admin" 會被算進後一個比對的預算——豁免範圍大於它要豁免的那件事。
    let src = "        let label = \"admin\"; let x = r == ROLE_SYSTEM_ADMIN;
";
    let v = violations("t.rs", src);
    assert_eq!(
        v.len(),
        1,
        "前一個敘述的 fallback 不該保護後一個敘述的比對。實際：{v:?}"
    );
}

#[test]
fn paired_fallback_within_same_inline_statement_is_clean() {
    // 反向防護：分號切分不能把「同一個敘述內的正當配對」切斷而誤報。
    let src = "        let a = 1; let x = r == ROLE_SYSTEM_ADMIN || r == ROLE_ADMIN_LEGACY;
";
    assert!(
        violations("t.rs", src).is_empty(),
        "同一敘述內的配對寫法不該被誤報"
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
