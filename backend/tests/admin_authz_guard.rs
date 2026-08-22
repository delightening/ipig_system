// CSO 防護測試：確保所有 handler 使用 Extension(current_user) 時都有權限檢查。
// 偵測模式：Extension(_current_user)（帶底線）= 取出但未使用 = 沒有權限檢查。
// 新增 handler 若用此 pattern 會讓測試紅燈，必須改用 current_user 並加權限檢查，
// 或經安全審計確認後加入 ALLOWED 白名單。

use std::collections::HashSet;
use std::path::Path;

// 白名單 key = "<相對 handlers 的路徑>::<函式名>"。
// 用「路徑 + 函式名」而非只用函式名，避免不同檔案出現同名 handler 時被錯誤放行（假陰性）。
// 以下經 CSO 第五輪審計確認為 read-only reference data，對所有已登入使用者開放，不含敏感資料。
const ALLOWED_UNCHECKED: &[&str] = &[
    "notification.rs::list_low_stock_alerts",
    "notification.rs::list_expiry_alerts",
    "system_features.rs::get_system_features",
    "amendment.rs::get_pending_count",
    "animal/blood_test.rs::list_blood_test_templates",
    "animal/blood_test.rs::list_blood_test_panels",
    "animal/blood_test.rs::list_blood_test_presets",
    "animal/source.rs::list_animal_sources",
];

#[test]
fn no_unchecked_current_user_in_handlers() {
    let handler_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/handlers");
    let allowed: HashSet<&str> = ALLOWED_UNCHECKED.iter().copied().collect();
    let mut violations = Vec::new();

    visit_dir(&handler_dir, &handler_dir, &allowed, &mut violations);

    assert!(
        violations.is_empty(),
        "\n\n{} handler(s) 使用 Extension(_current_user) 但未做權限檢查：\n{}\n\n\
         修復方式：將 _current_user 改為 current_user 並加入 require_permission! / is_admin() 檢查。\n\
         若經安全審計確認為 read-only reference data，可加入 ALLOWED_UNCHECKED 白名單\n\
         （key 格式：\"相對路徑::函式名\"）。\n",
        violations.len(),
        violations.join("\n")
    );
}

// I/O 失敗一律 panic（讓測試紅燈）。這是安全回歸測試，吞掉 read 錯誤會造成「部分 handler 沒被掃描卻仍綠燈」的假陰性。
fn visit_dir(root: &Path, dir: &Path, allowed: &HashSet<&str>, violations: &mut Vec<String>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("authz guard: 無法讀取目錄 {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("authz guard: 讀取目錄項目失敗: {e}"));
        let path = entry.path();
        if path.is_dir() {
            visit_dir(root, &path, allowed, violations);
        } else if path.extension().is_some_and(|e| e == "rs") {
            check_file(root, &path, allowed, violations);
        }
    }
}

fn check_file(root: &Path, path: &Path, allowed: &HashSet<&str>, violations: &mut Vec<String>) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("authz guard: 無法讀取 {}: {e}", path.display()));

    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !line.contains("Extension(_current_user)") {
            continue;
        }
        let fn_name = find_enclosing_fn(&lines, i);
        let key = format!("{rel}::{fn_name}");
        if allowed.contains(key.as_str()) {
            continue;
        }
        violations.push(format!("  {rel}:{} — fn {fn_name}", i + 1));
    }
}

fn find_enclosing_fn(lines: &[&str], target_line: usize) -> String {
    for i in (0..=target_line).rev() {
        let line = lines[i].trim();
        if line.starts_with("pub async fn ")
            || line.starts_with("pub fn ")
            || line.starts_with("async fn ")
        {
            if let Some(name) = line.split('(').next() {
                let name = name.trim();
                if let Some(fn_name) = name.split_whitespace().last() {
                    return fn_name.to_string();
                }
            }
        }
    }
    "<unknown>".to_string()
}
