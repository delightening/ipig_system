//! 防呆：**程式碼裡出現的每一個權限碼，都必須真的被定義出來**。
//!
//! ## 為什麼需要這個測試
//!
//! 2026-08-08 稽核（`docs/audit/dead-permission-codes-2026-08-08.md`）發現
//! **13 個死權限碼** —— 被 `require_permission!` / `has_permission` 檢查、
//! 或被寫進角色授予清單，卻從來不存在於 `permissions` 表。`has_permission`
//! 對它們永遠回 `false`，功能只靠 `is_admin()` 短路才能用：
//! 「只有管理員做得到」是意外，不是設計。其中 `facility.manage` 有 19 個呼叫點
//! 且零 fallback，整個設施管理模組在無人察覺下變成管理員專屬。
//!
//! 根因是**兩端都是裸字串、沒有交叉驗證**：
//!
//! 1. `ensure_all_role_permissions` 的授予 SQL 是
//!    `INSERT ... SELECT ... FROM roles CROSS JOIN permissions WHERE p.code = ANY($2)`。
//!    JOIN 的是 `permissions` 表 —— 清單裡有不存在的碼時 JOIN 直接不產生列，
//!    **沒有錯誤、沒有警告、`rows_affected` 少一筆也沒人看**。
//! 2. handler 那端寫 `require_permission!(user, "some.new.code")` 時，
//!    同樣沒有任何機制確認這個字串真的存在。
//!
//! ## 為什麼比對「原始碼」而不是資料庫
//!
//! 第一版是連上測試 DB 跑 migration + seed 後比對 `permissions` 表。那個做法有一個
//! **假綠**破口：`sqlx::migrate!` 不會刪掉既有的 `permissions` 列。若測試庫殘留前次
//! 執行寫入的權限，之後就算把某個碼從目錄移掉，它在 DB 裡仍然找得到 —— 測試照樣綠。
//! CI 每次都是全新容器所以不受影響，但本機重跑就會被殘留資料騙過去；
//! 而「本機綠、CI 才紅」正是這類防呆最不該有的性質。
//!
//! 改為純靜態比對後：不需要資料庫、跑得更快、且**沒有任何狀態可以殘留**。
//! 已驗證靜態抽取涵蓋 DB 現有全部權限碼（差集 0）。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// 權限碼的形狀：小寫 / 數字 / 底線的區段，以 `.` 分隔，至少兩段。
const CODE_PATTERN: &str = r"[a-z0-9_]+(?:\.[a-z0-9_]+)+";

fn code_regex(wrapper: &str) -> regex::Regex {
    regex::Regex::new(&wrapper.replace("CODE", CODE_PATTERN)).expect("regex")
}

/// 遞迴收集某目錄下所有指定副檔名的檔案。
fn collect(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

/// **權限碼的定義來源**（＝這些碼真的會進 `permissions` 表）：
///
/// 1. `startup/permissions.rs` 的 `required_permissions` 目錄，形如
///    `("code", "name", "module", "description")`
/// 2. migration 裡的 `INSERT INTO permissions (id, code, ...) VALUES (<id>, 'code', ...)`
///    ——`<id>` 可以是 `gen_random_uuid()` 或帶引號的 UUID 字面值（squash 後的種子檔用後者）
fn defined_codes() -> BTreeSet<String> {
    let mut defined = BTreeSet::new();

    let src = std::fs::read_to_string("src/startup/permissions.rs").expect("read permissions.rs");
    // 目錄是 tuple，第一欄就是碼；授予清單是裸字串（不以 `(` 開頭），不會誤配
    for cap in code_regex(r#"\(\s*"(CODE)"\s*,"#).captures_iter(&src) {
        defined.insert(cap[1].to_string());
    }

    let mut migrations = Vec::new();
    collect(Path::new("migrations"), "sql", &mut migrations);
    // 第一個 VALUES 欄位（id）可能是 `gen_random_uuid()`（手寫 migration）或
    // 帶引號的 UUID 字面值（squash 後 pg_dump 產生的種子檔），兩種都要接住。
    let re =
        code_regex(r#"INSERT INTO "?permissions"?\s*\([^)]*\)\s*VALUES\s*\(\s*[^,]+,\s*'(CODE)'"#);
    for path in &migrations {
        let Ok(sql) = std::fs::read_to_string(path) else {
            continue;
        };
        for cap in re.captures_iter(&sql) {
            defined.insert(cap[1].to_string());
        }
    }

    assert!(
        defined.len() > 150,
        "只抽到 {} 個已定義權限碼，明顯偏少 —— 抽取失效時本測試會全面假綠，必須當失敗處理",
        defined.len()
    );
    defined
}

/// 掃出所有**被授權檢查引用**的權限碼，回傳 code → 出現位置。
///
/// 對**整份檔案內容**跑 regex 而非逐行掃：`require_permission!` 被 rustfmt 折成多行時
/// （`require_permission!(\n    current_user,\n    "code",\n)`），逐行掃會完全看不到它，
/// 而那正是「漏掉的碼沒被檢查到」的假綠來源。目前庫內雖然還沒有多行寫法，
/// 但那只是行長度的巧合，不該當成保證。
fn referenced_codes() -> BTreeMap<String, String> {
    let mut files = Vec::new();
    collect(Path::new("src"), "rs", &mut files);
    assert!(
        !files.is_empty(),
        "掃不到 src/ 下的 .rs 檔，路徑或 CWD 有問題"
    );

    let re = code_regex(
        r#"(?s)(?:require_permission!\s*\(\s*[^,)]+,\s*|has_permission\s*\(\s*)"(CODE)""#,
    );
    let mut found = BTreeMap::new();
    for path in &files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        // 逐行砍掉 `//` 註解（含 `///` doc comment）再掃：文件裡追述舊寫法用的
        // `has_permission("code")` 字面文字（如 crud.rs 說明 admin 短路副作用的那段），
        // 不砍的話會被誤判成真的呼叫。squash 前這個假陽性被 defined_codes() 掃到同一支
        // 碼早已刪除的歷史 INSERT 恰好蓋住；squash 把歷史收斂成單一現況後才第一次爆出來。
        let content: String = raw
            .lines()
            .map(|line| line.find("//").map_or(line, |idx| &line[..idx]))
            .collect::<Vec<_>>()
            .join("\n");
        for cap in re.captures_iter(&content) {
            let byte = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let line = content[..byte].lines().count().max(1);
            found
                .entry(cap[1].to_string())
                .or_insert_with(|| format!("{}:{}", path.display(), line));
        }
    }

    assert!(
        found.len() > 50,
        "只抓到 {} 個被檢查的權限碼，明顯偏少 —— 抽取失效時本測試會假綠，必須當失敗處理",
        found.len()
    );
    found
}

/// 掃出**角色授予清單**裡的權限碼，回傳 code → 行號。
///
/// 授予項目長這樣（整行只有一個字串字面值）：
/// ```text
///                 "aup.review.reply",
///                 "aup.amendment.approve", // H6：可對被指派的修正案投票
/// ```
/// **必須容忍行尾註解**：`permissions.rs` 現有 3 筆就是這種寫法，
/// 第一版解析器把它們整行跳過了 —— 那三個碼剛好都存在所以沒出事，
/// 但只要有人用同樣寫法加一個新碼，防呆就看不見它。
///
/// 目錄定義是 tuple（以 `(` 開頭），以此與授予項目區分。
fn granted_codes() -> BTreeMap<String, usize> {
    let src = std::fs::read_to_string("src/startup/permissions.rs").expect("read permissions.rs");
    let re = code_regex(r#"^"(CODE)"\s*,?\s*(?://.*)?$"#);

    let mut granted = BTreeMap::new();
    for (n, line) in src.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('(') {
            continue; // 目錄定義，不是授予項目
        }
        if let Some(cap) = re.captures(trimmed) {
            granted.entry(cap[1].to_string()).or_insert(n + 1);
        }
    }

    assert!(
        granted.len() > 50,
        "只抓到 {} 個授予碼，明顯偏少 —— 抽取失效時本測試會假綠，必須當失敗處理",
        granted.len()
    );
    granted
}

/// handler / service 檢查的每一個權限碼都必須有定義。
///
/// 失敗時的修法：把缺的碼加進 `startup/permissions.rs` 的 `required_permissions`，
/// **不要**把檢查改掉了事 —— 那只是把「悄悄變成管理員專屬」換成「悄悄沒有防護」。
#[test]
fn every_checked_permission_code_is_defined() {
    let defined = defined_codes();
    let missing: Vec<String> = referenced_codes()
        .iter()
        .filter(|(code, _)| !defined.contains(*code))
        .map(|(code, at)| format!("  {code}  ({at})"))
        .collect();

    assert!(
        missing.is_empty(),
        "以下權限碼被程式碼檢查，但沒有任何地方定義它們（既不在 startup/permissions.rs\n\
         的目錄裡，也不在 migration 的 INSERT INTO permissions 裡）。\n\
         has_permission 對它們永遠回 false —— 功能會靜默變成「只有管理員做得到」，\n\
         而且啟動正常、CI 全綠、沒有任何徵兆。\n\
         修法：把它們加進 startup/permissions.rs 的 required_permissions，\n\
         不要把檢查刪掉了事。\n\n{}",
        missing.join("\n")
    );
}

/// 角色授予清單裡的每一個權限碼也必須有定義。
///
/// 這條抓的是 `aup.review.reply` 那種：碼被寫進五個角色的清單，
/// 卻因為不在目錄裡，授予 SQL 的 JOIN 直接不產生列 —— 五筆授予全部靜默落空，
/// 「這五種角色應該能回覆審查意見」的意圖從來沒生效過。
#[test]
fn every_granted_permission_code_is_defined() {
    let defined = defined_codes();
    let missing: Vec<String> = granted_codes()
        .iter()
        .filter(|(code, _)| !defined.contains(*code))
        .map(|(code, line)| format!("  {code}  (startup/permissions.rs:{line})"))
        .collect();

    assert!(
        missing.is_empty(),
        "以下權限碼出現在角色授予清單中，但沒有任何地方定義它們。\n\
         授予 SQL 是 `... FROM roles CROSS JOIN permissions WHERE p.code = ANY($2)`，\n\
         JOIN 不到就靜默不產生列 —— 那些角色永遠拿不到這個權限，且沒有任何錯誤訊息。\n\
         修法：把它們加進同檔的 required_permissions 目錄。\n\n{}",
        missing.join("\n")
    );
}

/// 解析器本身的回歸測試。
///
/// 上面三個抽取函式都靠 regex，而 regex 壞掉的症狀是**抓到的東西變少**，
/// 也就是「假綠」。各函式已有數量下限斷言擋住全面失效，這裡再鎖住兩個
/// 已知會漏、且真的在庫內出現過的寫法。
#[test]
fn parser_handles_known_tricky_shapes() {
    let granted = granted_codes();
    // permissions.rs 內確實存在帶行尾註解的授予項目（第一版解析器整行跳過）
    assert!(
        granted.contains_key("aup.amendment.approve"),
        "帶行尾註解的授予項目被漏掉了（例：\"aup.amendment.approve\", // H6：…）"
    );

    let referenced = referenced_codes();
    // 跨行 macro：目前庫內沒有，用合成字串直接驗 regex 本身
    let re = code_regex(
        r#"(?s)(?:require_permission!\s*\(\s*[^,)]+,\s*|has_permission\s*\(\s*)"(CODE)""#,
    );
    let multiline =
        "require_permission!(\n        current_user,\n        \"some.new.code\",\n    );";
    assert_eq!(
        re.captures(multiline).map(|c| c[1].to_string()),
        Some("some.new.code".to_string()),
        "跨行的 require_permission! 沒被抓到 —— rustfmt 折行後防呆就會失效"
    );
    // 順帶確認一般寫法沒被上面的多行支援弄壞
    assert!(referenced.contains_key("aup.review.assign"));
}
