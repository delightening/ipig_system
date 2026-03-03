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

#[derive(Debug, Deserialize)]
struct IdxfMeta {
    format: Option<String>,
    format_version: Option<String>,
    schema_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IdxfTable {
    name: String,
    #[allow(dead_code)]
    columns: Vec<String>,
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
        | "sku_categories" | "sku_subcategories" | "warehouses" | "partners" => &["code"],
        "blood_test_panels" => &["key"],
        "users" => &["email"],
        "notification_routing" => &["event_type", "role_code"],
        _ => return None,
    };
    Some(cols)
}

/// 有 natural key 的表（可建立 ID 對應）
fn has_id_remap_table(table: &str) -> bool {
    get_conflict_columns(table).is_some()
}

/// 預設管理員 email（匯入時排除，保留目標庫管理員）
fn admin_email() -> &'static str {
    option_env!("ADMIN_EMAIL").unwrap_or("admin@ipig.local")
}

/// 管理員角色 code（匯入時排除）
const ADMIN_ROLE_CODES: &[&str] = &["admin", "SYSTEM_ADMIN"];

/// 排除管理員相關內容，並建立 ID 對應供子表 remap
async fn filter_admin_content(
    pool: &PgPool,
    table: &str,
    rows: &[serde_json::Value],
    excluded_user_ids: &mut HashSet<String>,
    excluded_role_ids: &mut HashSet<String>,
    id_mapping: &mut IdMapping,
) -> Result<(Vec<serde_json::Value>, u64)> {
    let mut out = Vec::with_capacity(rows.len());
    let mut skipped = 0u64;

    match table {
        "users" => {
            let admin = admin_email();
            for row in rows {
                let obj = match row.as_object() {
                    Some(o) => o,
                    None => {
                        out.push(row.clone());
                        continue;
                    }
                };
                let email = obj
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if email.eq_ignore_ascii_case(admin) {
                    if let Some(serde_json::Value::String(id)) = obj.get("id") {
                        let sid = normalize_uuid(id);
                        excluded_user_ids.insert(sid.clone());
                        // 建立 ID 對應供子表 remap
                        let target_id: Option<String> = sqlx::query_scalar(r#"SELECT id::text FROM users WHERE email = $1"#)
                            .bind(admin)
                            .fetch_optional(pool)
                            .await?;
                        if let Some(tid) = target_id {
                            id_mapping
                                .entry("users".to_string())
                                .or_default()
                                .insert(sid, tid);
                        }
                    }
                    skipped += 1;
                    continue;
                }
                // 全庫匯入後不強制既有使用者變更密碼（保留來源密碼 hash）
                let mut user_row = row.clone();
                if let Some(o) = user_row.as_object_mut() {
                    o.insert("must_change_password".to_string(), serde_json::Value::Bool(false));
                }
                out.push(user_row);
            }
        }
        "roles" => {
            for row in rows {
                let obj = match row.as_object() {
                    Some(o) => o,
                    None => {
                        out.push(row.clone());
                        continue;
                    }
                };
                let code = obj
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if ADMIN_ROLE_CODES.iter().any(|&c| c.eq_ignore_ascii_case(code)) {
                    if let Some(serde_json::Value::String(id)) = obj.get("id") {
                        let sid = normalize_uuid(id);
                        excluded_role_ids.insert(sid.clone());
                        // 建立 ID 對應供子表 remap
                        let target_id: Option<String> =
                            sqlx::query_scalar(r#"SELECT id::text FROM roles WHERE code = $1"#)
                                .bind(code)
                                .fetch_optional(pool)
                                .await?;
                        if let Some(tid) = target_id {
                            id_mapping
                                .entry("roles".to_string())
                                .or_default()
                                .insert(sid, tid);
                        }
                    }
                    skipped += 1;
                    continue;
                }
                out.push(row.clone());
            }
        }
        "user_roles" => {
            for row in rows {
                let obj = match row.as_object() {
                    Some(o) => o,
                    None => {
                        out.push(row.clone());
                        continue;
                    }
                };
                let user_id = obj
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(normalize_uuid);
                let role_id = obj
                    .get("role_id")
                    .and_then(|v| v.as_str())
                    .map(normalize_uuid);
                let skip = user_id
                    .as_ref()
                    .map_or(false, |u| excluded_user_ids.contains(u))
                    || role_id
                        .as_ref()
                        .map_or(false, |r| excluded_role_ids.contains(r));
                if skip {
                    skipped += 1;
                    continue;
                }
                out.push(row.clone());
            }
        }
        "role_permissions" => {
            for row in rows {
                let obj = match row.as_object() {
                    Some(o) => o,
                    None => {
                        out.push(row.clone());
                        continue;
                    }
                };
                let role_id = obj
                    .get("role_id")
                    .and_then(|v| v.as_str())
                    .map(normalize_uuid);
                if role_id
                    .as_ref()
                    .map_or(false, |r| excluded_role_ids.contains(r))
                {
                    skipped += 1;
                    continue;
                }
                out.push(row.clone());
            }
        }
        _ => return Ok((rows.to_vec(), 0)),
    }

    Ok((out, skipped))
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

    #[derive(Deserialize)]
    struct ManifestTable {
        name: String,
        file: String,
        format: Option<String>,
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
    let target_version = crate::services::data_export::get_schema_version(pool).await;

    let fk_config = fetch_fk_config(pool).await?;
    let mut id_mapping: IdMapping = HashMap::new();
    let mut excluded_user_ids: HashSet<String> = HashSet::new();
    let mut excluded_role_ids: HashSet<String> = HashSet::new();

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

        let (mut rows, admin_skipped) = filter_admin_content(
            pool,
            &tbl.name,
            &rows,
            &mut excluded_user_ids,
            &mut excluded_role_ids,
            &mut id_mapping,
        )
        .await?;
        if admin_skipped > 0 {
            result.skipped_details.push(SkippedDetail {
                table: tbl.name.clone(),
                reason: "管理員相關，已略過".into(),
                count: Some(admin_skipped),
            });
        }

        if rows.is_empty() {
            continue;
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
    let _target_version = crate::services::data_export::get_schema_version(pool).await;

    let fk_config = fetch_fk_config(pool).await?;
    let mut id_mapping: IdMapping = HashMap::new();
    let mut excluded_user_ids: HashSet<String> = HashSet::new();
    let mut excluded_role_ids: HashSet<String> = HashSet::new();

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

        let (mut rows, admin_skipped) = filter_admin_content(
            pool,
            &table_name,
            &rows,
            &mut excluded_user_ids,
            &mut excluded_role_ids,
            &mut id_mapping,
        )
        .await?;
        if admin_skipped > 0 {
            result.skipped_details.push(SkippedDetail {
                table: table_name.clone(),
                reason: "管理員相關，已略過".into(),
                count: Some(admin_skipped),
            });
        }

        if rows.is_empty() {
            continue;
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
    EXPORT_TABLE_ORDER.iter().any(|t| *t == name)
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
        format!(
            r#"INSERT INTO "{}" SELECT * FROM json_populate_recordset(null::"{}", $1::json)"#,
            table, table
        )
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
            format!(" DO UPDATE SET {}", update_set.join(", "))
        };
        format!(
            r#"INSERT INTO "{}" SELECT * FROM json_populate_recordset(null::"{}", $1::json) ON CONFLICT ({}){}"#,
            table, table, conflict_expr, update_clause
        )
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
