//! 全庫 IDXF 匯入服務
//!
//! 支援單一 JSON 或 Zip 分包格式。
//! 匯入到有資料的庫時，會自動建立 ID 對應（source_id -> target_id），
//! 並在子表插入前 remap 外鍵欄位。

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use serde::Deserialize;
use sqlx::PgPool;
use zip::ZipArchive;

use crate::constants::FILE_MAX_DATA_IMPORT;
use crate::services::data_export::EXPORT_TABLE_ORDER;
use crate::services::schema_mapping;
use crate::{AppError, Result};

/// 匯入模式：遇重複則取代（ON CONFLICT DO UPDATE）
#[derive(Debug, Clone, Copy)]
pub enum ImportMode {
    Append,
}

/// 略過項目說明
#[derive(Debug, serde::Serialize)]
pub struct SkippedDetail {
    /// 表名
    pub table: String,
    /// 略過原因：如「未知表，已略過」「重複鍵」
    pub reason: String,
    /// 略過筆數（重複鍵時有值）
    pub count: Option<u64>,
}

/// 匯入結果
#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub tables_processed: usize,
    pub rows_inserted: u64,
    pub rows_skipped: u64,
    pub errors: Vec<String>,
    /// 略過項目明細：未知表、各表重複筆數
    pub skipped_details: Vec<SkippedDetail>,
}

// R26-7: IDXF 匯入 metadata — format_version 由 serde 反序列化入但 Rust 端
// 未讀取（保留給將來 format 版本相容檢查）。
#[derive(Debug, Deserialize)]
struct IdxfMeta {
    format: Option<String>,
    #[allow(dead_code)]
    format_version: Option<String>,
    schema_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IdxfTable {
    name: String,
    #[serde(rename = "columns")]
    _columns: Vec<String>,
    rows: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct IdxfRoot {
    meta: IdxfMeta,
    tables: Vec<IdxfTable>,
}

/// 表名 -> ON CONFLICT 欄位（遇重複則取代）
fn get_conflict_columns(table: &str) -> Option<&'static [&'static str]> {
    let cols: &[&str] = match table {
        "roles" | "permissions" | "animal_sources" | "blood_test_templates" | "product_categories"
        | "sku_categories" | "warehouses" | "partners"
        | "species" | "chart_of_accounts" | "departments" => &["code"],
        "sku_subcategories" => &["category_code", "code"],
        // facilities/buildings/zones/pens 使用 partial unique index，
        // 無法作為 ON CONFLICT arbiter，改由 cleanup_partial_unique_tables 先清除再用 PK 匯入
        "blood_test_panels" => &["key"],
        "users" => &["email"],
        "notification_routing" => &["event_type", "role_code"],
        "role_permissions" => &["role_id", "permission_id"],
        "user_roles" => &["user_id", "role_id"],
        "treatment_drug_options" => &["id"],
        _ => return None,
    };
    Some(cols)
}

/// 有 natural key 的表（可建立 ID 對應）
fn has_id_remap_table(table: &str) -> bool {
    get_conflict_columns(table).is_some()
}

/// 處理使用者表匯入前的欄位調整（所有使用者含管理員均正常匯入）
fn prepare_user_rows(rows: &[serde_json::Value]) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|row| {
            let mut user_row = row.clone();
            // 全庫匯入後不強制既有使用者變更密碼（保留來源密碼 hash）
            if let Some(o) = user_row.as_object_mut() {
                o.insert(
                    "must_change_password".to_string(),
                    serde_json::Value::Bool(false),
                );
            }
            user_row
        })
        .collect()
}


/// ID 對應：ref_table -> (source_id -> target_id)
type IdMapping = HashMap<String, HashMap<String, String>>;

/// FK 設定：(table_name, column_name) -> ref_table_name
type FkConfig = HashMap<(String, String), String>;

/// 從 information_schema 取得 FK 對應（僅包含參照 ref_table.id 的欄位）
async fn fetch_fk_config(pool: &PgPool) -> Result<FkConfig> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT tc.table_name::text, kcu.column_name::text,
               ccu.table_name::text AS ref_table, ccu.column_name::text AS ref_column
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
        JOIN information_schema.constraint_column_usage ccu
          ON tc.constraint_name = ccu.constraint_name AND tc.table_schema = ccu.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
          AND tc.table_schema = 'public'
          AND ccu.table_schema = 'public'
          AND ccu.column_name = 'id'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Fetch FK config: {}", e)))?;

    let mut config = FkConfig::new();
    for (tbl, col, ref_tbl, _ref_col) in rows {
        if has_id_remap_table(&ref_tbl) {
            config.insert((tbl, col), ref_tbl);
        }
    }
    Ok(config)
}

/// 建立指定表的 ID 對應（查詢目標庫既有資料）
async fn build_id_mappings(
    pool: &PgPool,
    table: &str,
    rows: &[serde_json::Value],
    mapping: &mut IdMapping,
) -> Result<()> {
    let Some(conflict_cols) = get_conflict_columns(table) else {
        return Ok(());
    };

    let table_mapping = mapping.entry(table.to_string()).or_default();

    for row in rows {
        let obj = match row.as_object() {
            Some(o) => o,
            None => continue,
        };

        let source_id = match obj.get("id") {
            Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        let mut key_values: Vec<String> = Vec::with_capacity(conflict_cols.len());
        for col in conflict_cols {
            match obj.get(*col) {
                Some(serde_json::Value::String(s)) => key_values.push(s.clone()),
                Some(serde_json::Value::Null) | None => break,
                _ => continue,
            }
        }
        if key_values.len() != conflict_cols.len() {
            continue;
        }

        // R7-P0-2: table 與 conflict_cols 來自 get_conflict_columns() 白名單，
        // 非使用者輸入。加入 debug_assert 確保不會被誤用於非白名單表。
        debug_assert!(
            get_conflict_columns(table).is_some(),
            "build_id_mappings called with non-whitelisted table: {}",
            table
        );

        let target_id: Option<String> = match conflict_cols.len() {
            1 => {
                sqlx::query_scalar(&format!(
                    r#"SELECT id::text FROM "{}" WHERE "{}" = $1"#,
                    table, conflict_cols[0]
                ))
                .bind(&key_values[0])
                .fetch_optional(pool)
                .await?
            }
            2 => {
                sqlx::query_scalar(&format!(
                    r#"SELECT id::text FROM "{}" WHERE "{}" = $1 AND "{}" = $2"#,
                    table, conflict_cols[0], conflict_cols[1]
                ))
                .bind(&key_values[0])
                .bind(&key_values[1])
                .fetch_optional(pool)
                .await?
            }
            _ => continue,
        };

        if let Some(tid) = target_id {
            if tid != source_id {
                table_mapping.insert(normalize_uuid(&source_id), tid);
            }
        }
    }
    Ok(())
}

fn normalize_uuid(s: &str) -> String {
    s.replace('-', "").to_lowercase()
}

/// 對 rows 中的 FK 欄位進行 ID remap
fn remap_foreign_keys(
    table: &str,
    rows: &mut [serde_json::Value],
    fk_config: &FkConfig,
    mapping: &IdMapping,
) {
    for row in rows.iter_mut() {
        let obj = match row.as_object_mut() {
            Some(o) => o,
            None => continue,
        };

        for ((tbl, col), ref_tbl) in fk_config.iter() {
            if tbl != table {
                continue;
            }
            let Some(ref_map) = mapping.get(ref_tbl) else {
                continue;
            };
            let Some(serde_json::Value::String(source_id)) = obj.get(col) else {
                continue;
            };
            let key = normalize_uuid(source_id);
            let Some(target_id) = ref_map.get(&key) else {
                continue;
            };
            obj.insert(col.clone(), serde_json::Value::String(target_id.clone()));
        }
    }
}

/// Zip 檔 magic bytes
const ZIP_MAGIC: [u8; 2] = [0x50, 0x4B]; // PK

/// 匯入 IDXF（自動偵測 JSON 或 Zip）
pub async fn import_idxf(pool: &PgPool, bytes: &[u8], mode: ImportMode) -> Result<ImportResult> {
    if bytes.len() > FILE_MAX_DATA_IMPORT {
        return Err(AppError::Validation(format!(
            "檔案過大，最大 {} MB",
            FILE_MAX_DATA_IMPORT / 1024 / 1024
        )));
    }
    if bytes.starts_with(&ZIP_MAGIC) {
        import_from_zip(pool, bytes, mode).await
    } else {
        import_from_json(pool, bytes, mode).await
    }
}

/// 匯入 Zip 分包格式
async fn import_from_zip(pool: &PgPool, bytes: &[u8], _mode: ImportMode) -> Result<ImportResult> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| AppError::Validation(format!("無效的 Zip: {}", e)))?;

    let manifest_bytes = {
        let mut r = archive
            .by_name("manifest.json")
            .map_err(|_| AppError::Validation("Zip 缺少 manifest.json".into()))?;
        let mut v = Vec::new();
        r.read_to_end(&mut v).map_err(|e| AppError::Validation(format!("讀取 manifest: {}", e)))?;
        v
    };

    // R26-7: columns 由 serde 反序列化入但 Rust 端未讀取
    // （保留給將來 column-level 驗證用）。
    #[derive(Deserialize)]
    struct ManifestTable {
        name: String,
        file: String,
        format: Option<String>,
        #[allow(dead_code)]
        columns: Vec<String>,
    }
    #[derive(Deserialize)]
    struct Manifest {
        meta: IdxfMeta,
        tables: Vec<ManifestTable>,
    }
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| AppError::Validation(format!("manifest.json 格式錯誤: {}", e)))?;

    if manifest.meta.format.as_deref() != Some("ipig-idxf") {
        return Err(AppError::Validation("非 ipig-idxf 格式".into()));
    }

    let source_version = manifest.meta.schema_version.as_deref().unwrap_or("010");
    let target_version = crate::services::data_export::get_schema_version(pool).await?;

    // 匯入含設施鏈資料時，先清除 partial unique index 表的既有種子資料
    let has_partial_tables = manifest.tables.iter().any(|t| {
        PARTIAL_UNIQUE_TABLES.contains(&t.name.as_str())
    });
    if has_partial_tables {
        cleanup_partial_unique_tables(pool).await?;
    }

    let fk_config = fetch_fk_config(pool).await?;
    let mut id_mapping: IdMapping = HashMap::new();

    let mut result = ImportResult {
        tables_processed: 0,
        rows_inserted: 0,
        rows_skipped: 0,
        errors: vec![],
        skipped_details: vec![],
    };

    for tbl in manifest.tables {
        if !is_export_table(&tbl.name) {
            result.skipped_details.push(SkippedDetail {
                table: tbl.name.clone(),
                reason: "未知表，已略過".into(),
                count: None,
            });
            continue;
        }
        let file_bytes = {
            let mut r = archive
                .by_name(&tbl.file)
                .map_err(|e| AppError::Validation(format!("無法讀取 {}: {}", tbl.file, e)))?;
            let mut v = Vec::new();
            r.read_to_end(&mut v).map_err(|e| AppError::Internal(e.to_string()))?;
            v
        };

        let rows = if tbl.format.as_deref() == Some("ndjson") {
            parse_ndjson(&file_bytes)?
        } else {
            let v: serde_json::Value = serde_json::from_slice(&file_bytes)
                .map_err(|e| AppError::Validation(format!("{}: {}", tbl.file, e)))?;
            v.as_array().cloned().unwrap_or_default()
        };

        if rows.is_empty() {
            continue;
        }

        let mut rows = rows;
        if source_version != target_version.as_str() {
            for row in rows.iter_mut() {
                schema_mapping::transform_row(source_version, &tbl.name, row);
            }
        }
        if tbl.name == "change_reasons" {
            sanitize_change_reasons(pool, &mut rows).await?;
        }
        if tbl.name == "users" {
            rows = prepare_user_rows(&rows);
        }

        if tbl.name == "treatment_drug_options" {
            let (filtered, name_cat_skipped) =
                filter_treatment_drug_options_by_name_category(pool, rows).await?;
            rows = filtered;
            if name_cat_skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: tbl.name.clone(),
                    reason: "名稱+分類重複，已略過".into(),
                    count: Some(name_cat_skipped),
                });
            }
            if rows.is_empty() {
                continue;
            }
        }
        if tbl.name == "login_events" {
            let (filtered, skipped) = filter_orphan_login_events(pool, rows).await?;
            rows = filtered;
            if skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: tbl.name.clone(),
                    reason: "user_id 不存在，已略過".into(),
                    count: Some(skipped),
                });
            }
            if rows.is_empty() { continue; }
        }
        if tbl.name == "journal_entry_lines" {
            let (filtered, skipped) = filter_orphan_journal_entry_lines(pool, rows).await?;
            rows = filtered;
            if skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: tbl.name.clone(),
                    reason: "account_id 不存在，已略過".into(),
                    count: Some(skipped),
                });
            }
            if rows.is_empty() { continue; }
        }
        if tbl.name == "animals" {
            let repaired = repair_animal_pen_id(pool, &mut rows).await?;
            if repaired > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: tbl.name.clone(),
                    reason: format!("pen_id 修復（pen_location 對應）{} 筆", repaired),
                    count: Some(repaired),
                });
            }
        }

        build_id_mappings(pool, &tbl.name, &rows, &mut id_mapping).await?;
        remap_foreign_keys(&tbl.name, &mut rows, &fk_config, &id_mapping);

        match import_table(pool, &tbl.name, rows).await {
            Ok((ins, skip)) => {
                result.tables_processed += 1;
                result.rows_inserted += ins;
                result.rows_skipped += skip;
                if skip > 0 {
                    result.skipped_details.push(SkippedDetail {
                        table: tbl.name.clone(),
                        reason: "重複鍵".into(),
                        count: Some(skip),
                    });
                }
            }
            Err(e) => result.errors.push(format!("{}: {}", tbl.name, e)),
        }
    }

    Ok(result)
}

fn parse_ndjson(bytes: &[u8]) -> Result<Vec<serde_json::Value>> {
    let s = std::str::from_utf8(bytes).map_err(|e| AppError::Validation(format!("NDJSON UTF-8: {}", e)))?;
    let mut out = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).map_err(|e| AppError::Validation(format!("NDJSON line: {}", e)))?;
        out.push(v);
    }
    Ok(out)
}

/// 匯入單一 JSON 檔
async fn import_from_json(pool: &PgPool, json_bytes: &[u8], _mode: ImportMode) -> Result<ImportResult> {
    let root: IdxfRoot = serde_json::from_slice(json_bytes)
        .map_err(|e| AppError::Validation(format!("無效的 IDXF JSON: {}", e)))?;

    if root.meta.format.as_deref() != Some("ipig-idxf") {
        return Err(AppError::Validation("非 ipig-idxf 格式".into()));
    }

    let source_version = root.meta.schema_version.as_deref().unwrap_or("010");
    let _target_version = crate::services::data_export::get_schema_version(pool).await?;

    // 匯入含設施鏈資料時，先清除 partial unique index 表的既有種子資料
    let has_partial_tables = root.tables.iter().any(|t| {
        PARTIAL_UNIQUE_TABLES.contains(&t.name.as_str())
            && t.rows.as_array().is_some_and(|a| !a.is_empty())
    });
    if has_partial_tables {
        cleanup_partial_unique_tables(pool).await?;
    }

    let fk_config = fetch_fk_config(pool).await?;
    let mut id_mapping: IdMapping = HashMap::new();

    let mut result = ImportResult {
        tables_processed: 0,
        rows_inserted: 0,
        rows_skipped: 0,
        errors: vec![],
        skipped_details: vec![],
    };

    let mut tables_with_rows: Vec<(String, Vec<serde_json::Value>)> = root
        .tables
        .into_iter()
        .filter_map(|t| {
            let arr = t.rows.as_array()?;
            if arr.is_empty() {
                return None;
            }
            Some((t.name, arr.clone()))
        })
        .collect();

    // 依 EXPORT_TABLE_ORDER 排序，確保父表先於子表
    tables_with_rows.sort_by(|a, b| {
        let ai = EXPORT_TABLE_ORDER.iter().position(|&x| x == a.0).unwrap_or(999);
        let bi = EXPORT_TABLE_ORDER.iter().position(|&x| x == b.0).unwrap_or(999);
        ai.cmp(&bi)
    });

    for (table_name, mut rows) in tables_with_rows {
        if !is_export_table(&table_name) {
            result.skipped_details.push(SkippedDetail {
                table: table_name.clone(),
                reason: "未知表，已略過".into(),
                count: None,
            });
            continue;
        }
        if source_version != _target_version.as_str() {
            for row in rows.iter_mut() {
                schema_mapping::transform_row(source_version, &table_name, row);
            }
        }
        if table_name == "change_reasons" {
            sanitize_change_reasons(pool, &mut rows).await?;
        }
        if table_name == "users" {
            rows = prepare_user_rows(&rows);
        }

        if table_name == "treatment_drug_options" {
            let (filtered, name_cat_skipped) =
                filter_treatment_drug_options_by_name_category(pool, rows).await?;
            rows = filtered;
            if name_cat_skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: table_name.clone(),
                    reason: "名稱+分類重複，已略過".into(),
                    count: Some(name_cat_skipped),
                });
            }
            if rows.is_empty() {
                continue;
            }
        }
        if table_name == "login_events" {
            let (filtered, skipped) = filter_orphan_login_events(pool, rows).await?;
            rows = filtered;
            if skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: table_name.clone(),
                    reason: "user_id 不存在，已略過".into(),
                    count: Some(skipped),
                });
            }
            if rows.is_empty() { continue; }
        }
        if table_name == "journal_entry_lines" {
            let (filtered, skipped) = filter_orphan_journal_entry_lines(pool, rows).await?;
            rows = filtered;
            if skipped > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: table_name.clone(),
                    reason: "account_id 不存在，已略過".into(),
                    count: Some(skipped),
                });
            }
            if rows.is_empty() { continue; }
        }
        if table_name == "animals" {
            let repaired = repair_animal_pen_id(pool, &mut rows).await?;
            if repaired > 0 {
                result.skipped_details.push(SkippedDetail {
                    table: table_name.clone(),
                    reason: format!("pen_id 修復（pen_location 對應）{} 筆", repaired),
                    count: Some(repaired),
                });
            }
        }

        build_id_mappings(pool, &table_name, &rows, &mut id_mapping).await?;
        remap_foreign_keys(&table_name, &mut rows, &fk_config, &id_mapping);

        match import_table(pool, &table_name, rows).await {
            Ok((ins, skip)) => {
                result.tables_processed += 1;
                result.rows_inserted += ins;
                result.rows_skipped += skip;
                if skip > 0 {
                    result.skipped_details.push(SkippedDetail {
                        table: table_name.clone(),
                        reason: "重複鍵".into(),
                        count: Some(skip),
                    });
                }
            }
            Err(e) => result.errors.push(format!("{}: {}", table_name, e)),
        }
    }

    Ok(result)
}

fn is_export_table(name: &str) -> bool {
    EXPORT_TABLE_ORDER.contains(&name)
}

/// 藥物選單：略過「名稱＋分類」已存在的列（與 DB 既有或同批重複皆略過）
async fn filter_treatment_drug_options_by_name_category(
    pool: &PgPool,
    rows: Vec<serde_json::Value>,
) -> Result<(Vec<serde_json::Value>, u64)> {
    if rows.is_empty() {
        return Ok((rows, 0));
    }
    #[derive(sqlx::FromRow)]
    struct Row {
        name: String,
        category: Option<String>,
    }
    let existing: Vec<Row> = sqlx::query_as(
        "SELECT name, category FROM treatment_drug_options",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("查詢既有藥物選項: {}", e)))?;

    let mut existing_keys: HashSet<(String, String)> = existing
        .into_iter()
        .map(|r| {
            (
                r.name.trim().to_lowercase(),
                r.category.unwrap_or_default().trim().to_string(),
            )
        })
        .collect();

    let mut out = Vec::with_capacity(rows.len());
    let mut skipped = 0u64;
    for row in rows {
        let key = row
            .as_object()
            .map(|obj| {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
                let category = obj
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                (name, category)
            });
        let key = match key {
            Some(k) => k,
            None => {
                out.push(row);
                continue;
            }
        };
        if existing_keys.contains(&key) {
            skipped += 1;
            continue;
        }
        existing_keys.insert(key);
        out.push(row);
    }
    Ok((out, skipped))
}

/// 匯入 change_reasons 前，將不存在的 changed_by 設為 null，避免 FK 違反
async fn sanitize_change_reasons(pool: &PgPool, rows: &mut [serde_json::Value]) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let valid_user_ids: std::collections::HashSet<String> = sqlx::query_scalar::<_, String>("SELECT id::text FROM users")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Fetch users for change_reasons sanitize: {}", e)))?
        .into_iter()
        .map(|s| normalize_uuid(&s))
        .collect();

    for row in rows.iter_mut() {
        let obj = match row.as_object_mut() {
            Some(o) => o,
            None => continue,
        };
        if let Some(serde_json::Value::String(id)) = obj.get("changed_by") {
            if !valid_user_ids.contains(&normalize_uuid(id)) {
                obj.insert("changed_by".to_string(), serde_json::Value::Null);
            }
        }
    }
    Ok(())
}

/// login_events 前過濾 user_id 不存在的孤兒列（FK 違反保護）
async fn filter_orphan_login_events(
    pool: &PgPool,
    rows: Vec<serde_json::Value>,
) -> Result<(Vec<serde_json::Value>, u64)> {
    if rows.is_empty() {
        return Ok((rows, 0));
    }
    let valid_ids: std::collections::HashSet<String> =
        sqlx::query_scalar::<_, String>("SELECT id::text FROM users")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Fetch users for login_events sanitize: {}", e)))?
            .into_iter()
            .map(|s| normalize_uuid(&s))
            .collect();

    let mut out = Vec::with_capacity(rows.len());
    let mut skipped = 0u64;
    for row in rows {
        let uid = row.as_object()
            .and_then(|o| o.get("user_id"))
            .and_then(|v| v.as_str())
            .map(normalize_uuid);
        if uid.as_ref().is_some_and(|id| !valid_ids.contains(id)) {
            skipped += 1;
        } else {
            out.push(row);
        }
    }
    Ok((out, skipped))
}

/// journal_entry_lines 前過濾 account_id 不存在的孤兒列（FK 違反保護）
async fn filter_orphan_journal_entry_lines(
    pool: &PgPool,
    rows: Vec<serde_json::Value>,
) -> Result<(Vec<serde_json::Value>, u64)> {
    if rows.is_empty() {
        return Ok((rows, 0));
    }
    let valid_ids: std::collections::HashSet<String> =
        sqlx::query_scalar::<_, String>("SELECT id::text FROM chart_of_accounts")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Fetch chart_of_accounts for journal_entry_lines sanitize: {}", e)))?
            .into_iter()
            .map(|s| normalize_uuid(&s))
            .collect();

    let mut out = Vec::with_capacity(rows.len());
    let mut skipped = 0u64;
    for row in rows {
        let aid = row.as_object()
            .and_then(|o| o.get("account_id"))
            .and_then(|v| v.as_str())
            .map(normalize_uuid);
        if aid.as_ref().is_some_and(|id| !valid_ids.contains(id)) {
            skipped += 1;
        } else {
            out.push(row);
        }
    }
    Ok((out, skipped))
}

/// animals 匯入前，利用 pen_location → pens.code 修復失效的 pen_id
async fn repair_animal_pen_id(
    pool: &PgPool,
    rows: &mut [serde_json::Value],
) -> Result<u64> {
    if rows.is_empty() {
        return Ok(0);
    }
    // 取得目標庫中 code -> id 對應
    let pen_map: std::collections::HashMap<String, String> =
        sqlx::query_as::<_, (String, String)>("SELECT code, id::text FROM pens WHERE is_active = true")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Fetch pens for animal repair: {}", e)))?
            .into_iter()
            .collect();

    // 取得目前有效的 pen_id 集合（含軟刪除）
    let valid_pen_ids: std::collections::HashSet<String> =
        sqlx::query_scalar::<_, String>("SELECT id::text FROM pens")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Fetch pen ids for animal repair: {}", e)))?
            .into_iter()
            .map(|s| normalize_uuid(&s))
            .collect();

    let mut repaired = 0u64;
    for row in rows.iter_mut() {
        let obj = match row.as_object_mut() {
            Some(o) => o,
            None => continue,
        };
        let pen_id_valid = obj.get("pen_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| valid_pen_ids.contains(&normalize_uuid(id)));

        if pen_id_valid {
            continue;
        }
        // pen_id 無效或為 null，嘗試用 pen_location 修復
        let location = obj.get("pen_location").and_then(|v| v.as_str()).map(String::from);
        if let Some(loc) = location {
            if let Some(correct_id) = pen_map.get(&loc) {
                obj.insert("pen_id".to_string(), serde_json::Value::String(correct_id.clone()));
                repaired += 1;
            } else {
                obj.insert("pen_id".to_string(), serde_json::Value::Null);
            }
        }
    }
    Ok(repaired)
}

/// 使用 partial unique index 的表（ON CONFLICT 無法直接使用，需先清除衝突列）
/// 清除順序：子表先刪，父表後刪（反向 FK 順序）
const PARTIAL_UNIQUE_TABLES: &[&str] = &["pens", "zones", "buildings", "facilities"];

/// 匯入前清除 partial unique index 表的既有資料，避免 ON CONFLICT 無法匹配
async fn cleanup_partial_unique_tables(pool: &PgPool) -> Result<()> {
    for &table in PARTIAL_UNIQUE_TABLES {
        // 表名來自常數白名單，非使用者輸入
        let mut sql = String::from(r#"DELETE FROM ""#);
        sql.push_str(table);
        sql.push('"');
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Cleanup {}: {}", table, e)))?;
    }
    Ok(())
}

async fn import_table(
    pool: &PgPool,
    table: &str,
    rows: Vec<serde_json::Value>,
) -> Result<(u64, u64)> {
    if rows.is_empty() {
        return Ok((0, 0));
    }

    let conflict_cols: Vec<String> = match get_conflict_columns(table) {
        Some(cols) => cols.iter().map(|s| (*s).to_string()).collect(),
        None => get_primary_key_columns(pool, table).await?,
    };

    let all_cols = get_table_columns(pool, table).await?;
    let rows_json = serde_json::to_value(&rows).map_err(|e| AppError::Internal(e.to_string()))?;

    // json_populate_recordset 第二參數需為 json 型別（非 jsonb）
    // 遇重複則取代（ON CONFLICT DO UPDATE SET）
    let sql = if conflict_cols.is_empty() {
        let mut s = String::from("INSERT INTO \"");
        s.push_str(table);
        s.push_str("\" SELECT * FROM json_populate_recordset(null::\"");
        s.push_str(table);
        s.push_str("\", $1::json)");
        s
    } else {
        let conflict_expr = conflict_cols
            .iter()
            .map(|c| format!(r#""{}""#, c))
            .collect::<Vec<_>>()
            .join(", ");
        let conflict_set: std::collections::HashSet<_> = conflict_cols.iter().map(|s| s.as_str()).collect();
        let pk_cols: std::collections::HashSet<_> = get_primary_key_columns(pool, table)
            .await?
            .into_iter()
            .collect();
        // 排除 conflict 欄位與主鍵：主鍵不可在 upsert 時被覆寫，否則會破壞子表 FK 參照
        let update_set: Vec<String> = all_cols
            .iter()
            .filter(|c| !conflict_set.contains(c.as_str()) && !pk_cols.contains(c.as_str()))
            .map(|c| format!(r#""{}" = EXCLUDED."{}""#, c, c))
            .collect();
        let update_clause = if update_set.is_empty() {
            " DO NOTHING".to_string()
        } else {
            [" DO UPDATE SET ", &update_set.join(", ")].concat()
        };
        let mut s = String::from("INSERT INTO \"");
        s.push_str(table);
        s.push_str("\" SELECT * FROM json_populate_recordset(null::\"");
        s.push_str(table);
        s.push_str("\", $1::json) ON CONFLICT (");
        s.push_str(&conflict_expr);
        s.push(')');
        s.push_str(&update_clause);
        s
    };

    let q = sqlx::query(&sql).bind(rows_json);
    let res = q.execute(pool).await.map_err(|e| {
        AppError::Internal(format!("Import table {}: {}", table, e))
    })?;

    let ins = res.rows_affected();
    let total = rows.len() as u64;
    let skipped = total.saturating_sub(ins);
    Ok((ins, skipped))
}

async fn get_table_columns(pool: &PgPool, table: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT column_name::text FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = $1
        ORDER BY ordinal_position
        "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Columns for {}: {}", table, e)))?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

async fn get_primary_key_columns(pool: &PgPool, table: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT a.attname::text
        FROM pg_index i
        JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
        WHERE i.indrelid = $1::regclass
          AND i.indisprimary
          AND a.attnum > 0
          AND NOT a.attisdropped
        ORDER BY array_position(i.indkey, a.attnum)
        "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("PK columns for {}: {}", table, e)))?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}
