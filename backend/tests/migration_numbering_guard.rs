//! 守衛：`backend/migrations/` 的版本號必須從 1 起連續、不得重號、不得跳號。
//!
//! ## 這支測試防的 bug
//!
//! 選號規則原本只寫在 `docs/agents/RULES_BACKEND.md` §9（文字規則），踩過兩種：
//!
//! - **撞號**（2026-07-01）：兩支分支各自取了同一個號碼。sqlx 以版本號為鍵，
//!   兩支都合進來後只會套用其中一支，另一支的 schema 變更靜默消失。
//! - **跳號**（2026-08-10）：下一號被未合分支佔用，另一支自取再下一號並先合併部署。
//!   之後佔號那支合進來時，它的版本號**小於**資料庫已套用的最大值，
//!   各環境的套用順序因此不一致。
//!
//! 兩者在合併當下都不會有任何錯誤訊息，所以改成在 `cargo test` 裡擋。
//!
//! ## 判準
//!
//! 只看 `backend/migrations/` **第一層**的 `.sql` 檔名，不看內容、不連資料庫：
//!
//! 1. 檔名必須是 `<數字>_<描述>.sql`（sqlx 的格式）。
//! 2. 版本號照**數值**比較（`001` 與 `1` 算同一號），不得重複。
//! 3. 排序後必須恰好是 `1, 2, …, N`——缺號即是跳號。
//!
//! ⚠️ 本守衛擋不到「未合分支已佔用下一號」：那需要看別的分支，屬於選號流程
//! （`RULES_BACKEND.md` §9）。它擋的是「兩支都合進來之後」留下的重號或缺號。
//!
//! 和 `system_admin_role_code_guard.rs` 同一個原則：除了掃真實目錄，
//! 也用合成輸入直接測判定函式——「目錄是乾淨的」與「判定壞掉」外觀完全一樣。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// 從檔名取出版本號；不是 `<數字>_<描述>.sql` 就回傳 `None`。
fn parse_version(name: &str) -> Option<u64> {
    let stem = name.strip_suffix(".sql")?;
    let (digits, desc) = stem.split_once('_')?;
    if digits.is_empty() || desc.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// 對一組檔名做判定，回傳所有問題（空 = 通過）。
fn problems(names: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut by_version: BTreeMap<u64, Vec<&str>> = BTreeMap::new();

    for name in names {
        match parse_version(name) {
            Some(v) => by_version.entry(v).or_default().push(name),
            None => out.push(format!("檔名不符 `<數字>_<描述>.sql`：{name}")),
        }
    }

    for (v, files) in &by_version {
        if files.len() > 1 {
            out.push(format!("重號 {v}：{}", files.join("、")));
        }
    }

    // 版本號從 1 起算；0 不合法（缺號檢查從 1 開始，0 若不單獨擋會被當成合法而漏報）。
    if let Some(files) = by_version.get(&0) {
        out.push(format!(
            "版本號 0 不合法（須從 1 起）：{}",
            files.join("、")
        ));
    }

    // 缺號：比對排序後**相鄰**的版本號，回報缺號區間。
    // ⚠️ 不可列舉 1..=最大號：時間戳形狀的檔名（`20260810120000_x.sql`）會讓列舉跑上兆次
    // 並逐一配置字串，`cargo test` 卡死或耗盡記憶體，而不是給出有用的失敗。
    // 這樣寫工作量只與檔案數成正比，與最大號無關。
    let mut gaps = Vec::new();
    let mut prev = 0u64;
    for &v in by_version.keys().filter(|v| **v > 0) {
        let expected = prev + 1;
        if v > expected {
            let last = v - 1;
            gaps.push(if expected == last {
                expected.to_string()
            } else {
                format!("{expected}–{last}")
            });
        }
        prev = v;
    }
    if !gaps.is_empty() {
        out.push(format!("缺號（跳號）：{}", gaps.join("、")));
    }
    out
}

/// 列出目錄第一層的 `.sql` 檔名。任何一筆讀取失敗都回傳錯誤（帶路徑），
/// 不靜默略過——漏掉一支 migration，剩下的序列可能剛好連續而通過。
fn sql_file_names(dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("讀不到 {}：{e}", dir.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("列舉 {} 失敗：{e}", dir.display()))?;
        let path = entry.path();
        let meta =
            fs::metadata(&path).map_err(|e| format!("讀不到 {} 的屬性：{e}", path.display()))?;
        if meta.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|n| format!("檔名不是 UTF-8：{n:?}"))?;
        if name.ends_with(".sql") {
            names.push(name);
        }
    }
    Ok(names)
}

#[test]
fn migration_versions_are_contiguous_and_unique() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let names = sql_file_names(&dir).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !names.is_empty(),
        "migrations 目錄裡沒有任何 .sql，路徑設定有問題"
    );

    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let found = problems(&refs);
    assert!(
        found.is_empty(),
        "backend/migrations/ 的版本號必須從 1 起連續且不重複：\n{}\n\n\
         修法：照 docs/agents/RULES_BACKEND.md §9 重新選號（main 的最大號 +1）。\
         下一號被未合分支佔用時停下來問，不要自己跳號。",
        found.join("\n")
    );
}

// ── 守衛自己的判定測試 ──
//
// 每一條都對應一種真的發生過、或判定上必須分得清的形狀。

#[test]
fn contiguous_versions_pass() {
    assert!(problems(&["001_a.sql", "002_b.sql", "003_c.sql"]).is_empty());
}

#[test]
fn duplicate_version_is_reported() {
    // 2026-07-01 撞號：兩支分支取了同一號。
    let p = problems(&["001_a.sql", "002_b.sql", "002_other_branch.sql"]);
    assert_eq!(p.len(), 1, "應只回報一個重號。實際：{p:?}");
    assert!(p[0].contains("重號 2"), "實際：{p:?}");
}

#[test]
fn zero_padding_does_not_hide_a_duplicate() {
    // sqlx 照數值解析版本，`002` 與 `2` 會撞在一起。
    let p = problems(&["001_a.sql", "002_b.sql", "2_c.sql"]);
    assert!(p.iter().any(|m| m.contains("重號 2")), "實際：{p:?}");
}

#[test]
fn skipped_version_is_reported() {
    // 2026-08-10 跳號：下一號被未合分支佔用，另一支自取再下一號先合進來。
    let p = problems(&["001_a.sql", "002_b.sql", "004_d.sql"]);
    assert_eq!(p.len(), 1, "應只回報一個缺號。實際：{p:?}");
    assert!(p[0].contains("缺號") && p[0].contains('3'), "實際：{p:?}");
}

#[test]
fn not_starting_at_one_is_reported() {
    let p = problems(&["002_b.sql", "003_c.sql"]);
    assert!(
        p.iter().any(|m| m.contains("缺號") && m.contains('1')),
        "實際：{p:?}"
    );
}

#[test]
fn version_zero_is_reported() {
    // CodeRabbit #276：缺號檢查從 1 開始，`000` 加上連號的其餘檔案原本會通過。
    let p = problems(&["000_zero.sql", "001_a.sql"]);
    assert_eq!(p.len(), 1, "應只回報版本 0。實際：{p:?}");
    assert!(p[0].contains("版本號 0"), "實際：{p:?}");
}

#[test]
fn timestamp_shaped_name_reports_a_gap_range_without_enumerating() {
    // CodeRabbit #276（Major）：舊寫法列舉 1..=最大號，時間戳形狀的檔名會跑上兆次。
    // 這支測試若退回列舉，會卡住或耗盡記憶體而不是快速通過。
    let p = problems(&["001_a.sql", "20260810120000_new.sql"]);
    assert_eq!(p.len(), 1, "應回報一個缺號區間。實際：{p:?}");
    assert!(p[0].contains("2–20260810119999"), "實際：{p:?}");
}

#[test]
fn several_gaps_are_reported_as_ranges() {
    let p = problems(&["001_a.sql", "003_c.sql", "007_g.sql"]);
    assert_eq!(p.len(), 1, "實際：{p:?}");
    assert!(p[0].contains("2、4–6"), "實際：{p:?}");
}

#[test]
fn unreadable_directory_is_an_error_not_an_empty_list() {
    // CodeRabbit #276：掃描錯誤不可被靜默吞掉。
    let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-such-dir-for-guard-test");
    assert!(sql_file_names(&missing).is_err());
}

#[test]
fn malformed_names_are_reported() {
    let p = problems(&["001_a.sql", "abc_b.sql", "002.sql", "_c.sql"]);
    assert_eq!(p.len(), 3, "三個檔名都不符格式。實際：{p:?}");
}
