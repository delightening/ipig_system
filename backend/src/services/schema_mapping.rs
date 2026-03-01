//! Schema 版本對應與 Column Mapper
//!
//! 跨 migration 匯入時，將來源 schema 的欄位映射至目標 schema。
//! - 欄位改名：old_name -> new_name
//! - 新欄位：目標有、來源無 → json_populate_recordset 自動補 NULL
//! - 移除欄位：來源有、目標無 → json_populate_recordset 自動忽略

use std::collections::HashMap;

/// 來源 schema 版本 -> 表名 -> (舊欄位名 -> 新欄位名)
/// 當 migration 有欄位改名時，在此註冊
fn column_rename_mappings() -> HashMap<&'static str, HashMap<&'static str, HashMap<&'static str, &'static str>>> {
    let m = HashMap::new();
    // 範例：若 009->010 時 users 表的 foo 改為 bar，可加入：
    // m.entry("009").or_default().entry("users").or_default().insert("foo", "bar");
    m
}

/// 依 source_schema_version 轉換 row JSON 的欄位名
pub fn transform_row(source_schema_version: &str, table: &str, row: &mut serde_json::Value) {
    let mappings = column_rename_mappings();
    let Some(table_map) = mappings.get(source_schema_version) else {
        return;
    };
    let Some(renames) = table_map.get(table) else {
        return;
    };
    let Some(obj) = row.as_object_mut() else {
        return;
    };
    for (old_key, new_key) in renames {
        if let Some(v) = obj.remove(*old_key) {
            obj.insert((*new_key).to_string(), v);
        }
    }
}
