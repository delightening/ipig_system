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

    // 期望 1..=最大號；任何沒出現的號碼都是缺號。
    if let Some(&max) = by_version.keys().next_back() {
        let missing: Vec<String> = (1..=max)
            .filter(|v| !by_version.contains_key(v))
            .map(|v| v.to_string())
            .collect();
        if !missing.is_empty() {
            out.push(format!("缺號（跳號）：{}", missing.join("、")));
        }
    }
    out
}

#[test]
fn migration_versions_are_contiguous_and_unique() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let entries = fs::read_dir(&dir).unwrap_or_else(|e| panic!("讀不到 {}：{e}", dir.display()));

    let names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|n| n.ends_with(".sql"))
        .collect();
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
fn malformed_names_are_reported() {
    let p = problems(&["001_a.sql", "abc_b.sql", "002.sql", "_c.sql"]);
    assert_eq!(p.len(), 3, "三個檔名都不符格式。實際：{p:?}");
}
