//! 全庫 IDXF 匯入服務
//!
//! 支援單一 JSON 或 Zip 分包格式。

use std::io::{Cursor, Read};
use serde::Deserialize;
use sqlx::PgPool;
use zip::ZipArchive;

use crate::constants::FILE_MAX_DATA_IMPORT;
use crate::services::schema_mapping;
use crate::{AppError, Result};

/// 匯入模式
#[derive(Debug, Clone, Copy)]
pub enum ImportMode {
    /// 僅新增，遇到衝突略過（ON CONFLICT DO NOTHING）
    Append,
}

/// 匯入結果
#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub tables_processed: usize,
    pub rows_inserted: u64,
    pub rows_skipped: u64,
    pub errors: Vec<String>,
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

    let mut result = ImportResult {
        tables_processed: 0,
        rows_inserted: 0,
        rows_skipped: 0,
        errors: vec![],
    };

    for tbl in manifest.tables {
        if !is_export_table(&tbl.name) {
            result.errors.push(format!("略過未知表: {}", tbl.name));
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
        match import_table(pool, &tbl.name, rows).await {
            Ok((ins, skip)) => {
                result.tables_processed += 1;
                result.rows_inserted += ins;
                result.rows_skipped += skip;
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
    let target_version = crate::services::data_export::get_schema_version(pool).await;

    let mut result = ImportResult {
        tables_processed: 0,
        rows_inserted: 0,
        rows_skipped: 0,
        errors: vec![],
    };

    let rows_arr = root.tables.into_iter().filter_map(|t| {
        let arr = t.rows.as_array()?;
        if arr.is_empty() {
            return None;
        }
        Some((t.name, arr.clone()))
    });

    for (table_name, mut rows) in rows_arr {
        if !is_export_table(&table_name) {
            result.errors.push(format!("略過未知表: {}", table_name));
            continue;
        }
        // 跨版本時套用 column mapper
        if source_version != target_version.as_str() {
            for row in rows.iter_mut() {
                schema_mapping::transform_row(source_version, &table_name, row);
            }
        }
        match import_table(pool, &table_name, rows).await {
            Ok((ins, skip)) => {
                result.tables_processed += 1;
                result.rows_inserted += ins;
                result.rows_skipped += skip;
            }
            Err(e) => {
                result.errors.push(format!("{}: {}", table_name, e));
            }
        }
    }

    Ok(result)
}

fn is_export_table(name: &str) -> bool {
    crate::services::data_export::EXPORT_TABLE_ORDER
        .iter()
        .any(|t| *t == name)
}

async fn import_table(
    pool: &PgPool,
    table: &str,
    rows: Vec<serde_json::Value>,
) -> Result<(u64, u64)> {
    if rows.is_empty() {
        return Ok((0, 0));
    }

    let pk_cols = get_primary_key_columns(pool, table).await?;
    let rows_json = serde_json::to_value(&rows).map_err(|e| AppError::Internal(e.to_string()))?;

    let sql = if pk_cols.is_empty() {
        format!(
            r#"INSERT INTO "{}" SELECT * FROM json_populate_recordset(null::"{}", $1::jsonb)"#,
            table, table
        )
    } else {
        let conflict_cols = pk_cols
            .iter()
            .map(|c| format!(r#""{}""#, c))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"INSERT INTO "{}" SELECT * FROM json_populate_recordset(null::"{}", $1::jsonb) ON CONFLICT ({}) DO NOTHING"#,
            table, table, conflict_cols
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
