//! 全庫 IDXF 匯出服務
//!
//! 一鍵輸出整個資料庫為 iPig Data Exchange Format (IDXF)，
//! 可在不同 migration 版本間讀取。

use chrono::Utc;
use serde_json::Value;
use sqlx::PgPool;

use crate::{AppError, Result};

/// 匯出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// 單一 JSON 檔
    Json,
    /// Zip 分包（manifest + 每表一檔，大表用 NDJSON）
    Zip,
}

/// 匯出參數
#[derive(Debug, Clone)]
pub struct ExportParams {
    /// 是否包含大量稽核資料
    pub include_audit: bool,
    /// 匯出格式
    pub format: ExportFormat,
}

impl Default for ExportParams {
    fn default() -> Self {
        Self {
            include_audit: false,
            format: ExportFormat::Json,
        }
    }
}

/// 大表門檻：超過此行數的表以 NDJSON 儲存於 zip
const LARGE_TABLE_THRESHOLD: usize = 10_000;

/// 依 FK 依賴順序排列的資料表清單
///
/// 故意排除（敏感 token / 內部表，非業務資料）：
/// - `jwt_blacklist`、`refresh_tokens`、`password_reset_tokens`
/// - `_sqlx_migrations`
///
/// 不在此清單但會被 `INTENTIONALLY_EXCLUDED_TABLES`（見測試）涵蓋。
pub const EXPORT_TABLE_ORDER: &[&str] = &[
    // 001 - 核心，無 FK
    "roles",
    "permissions",
    // 002 - 動物基礎，無 FK（或僅 self-ref）
    "animal_sources",
    "species",
    "blood_test_templates",
    "blood_test_panels",
    "blood_test_panel_items",
    "blood_test_presets",
    // 設施鏈：facilities → buildings → zones → pens（animals 依賴 pens + species）
    "facilities",
    "buildings",
    "zones",
    "pens",
    // 006 - ERP 基礎
    "product_categories",
    "sku_categories",
    "sku_subcategories",
    "sku_sequences",
    "warehouses",
    "partners",
    "chart_of_accounts",
    // 001 - users 需在 role_permissions, user_roles 之前
    "users",
    "departments",
    "role_permissions",
    "user_roles",
    "user_preferences",
    "user_mcp_keys",
    "invitations",
    "ai_api_keys",
    "ai_query_logs",
    "notifications",
    "notification_settings",
    "attachments",
    "audit_logs",
    "change_reasons",
    "login_events",
    // 002 - 動物（依賴 users, animal_sources, pens, species）
    "animals",
    "animal_observations",
    "animal_surgeries",
    "animal_weights",
    "animal_vaccinations",
    "animal_sacrifices",
    "animal_pathology_reports",
    "animal_record_attachments",
    "vet_recommendations",
    "care_medication_records",
    "record_versions",
    "import_jobs",
    "export_jobs",
    "euthanasia_orders",
    "euthanasia_appeals",
    "animal_import_batches",
    "observation_vet_reads",
    "surgery_vet_reads",
    "animal_blood_tests",
    "animal_blood_test_items",
    "animal_sudden_deaths",
    "animal_transfers",
    "transfer_vet_evaluations",
    "animal_field_correction_requests",
    "animal_vet_advices",
    "animal_vet_advice_records",
    "vet_patrol_reports",
    "vet_patrol_entries",
    // 003 - AUP
    "protocols",
    "user_protocols",
    "protocol_versions",
    "protocol_status_history",
    "review_assignments",
    "review_comments",
    "protocol_attachments",
    "amendments",
    "amendment_review_assignments",
    "amendment_versions",
    "amendment_status_history",
    "user_aup_profiles",
    "scheduled_reports",
    "report_history",
    "system_settings",
    "vet_review_assignments",
    "protocol_activities",
    "review_round_history",
    "protocol_ai_reviews",
    // 004 - HR
    "attendance_records",
    "overtime_records",
    "overtime_approvals",
    "annual_leave_entitlements",
    "comp_time_balances",
    "leave_requests",
    "leave_approvals",
    "leave_balance_usage",
    "google_calendar_config",
    "calendar_event_sync",
    "calendar_sync_conflicts",
    "calendar_sync_history",
    // 005 - 稽核（可選，量大）
    "user_activity_logs",
    "user_sessions",
    "user_activity_aggregates",
    "security_alerts",
    "security_alert_config",
    "security_notification_channels",
    "ip_blocklist",
    // 006 - ERP 明細
    "storage_locations",
    "products",
    "product_uom_conversions",
    "documents",
    "document_lines",
    "storage_location_inventory",
    "stock_ledger",
    "inventory_snapshots",
    "expiry_notification_config",
    "expiry_monthly_snapshots",
    // 007 - 補充
    "notification_routing",
    "electronic_signatures",
    "record_annotations",
    "treatment_drug_options",
    // 010 - GLP/QAU
    "training_records",
    "equipment",
    "equipment_calibrations",
    "equipment_suppliers",
    "equipment_status_logs",
    "equipment_maintenance_records",
    "equipment_disposals",
    "equipment_annual_plans",
    "equipment_idle_requests",
    "journal_entries",
    "journal_entry_lines",
    "ap_payments",
    "ar_receipts",
    // 011 - QA 計畫
    "qa_inspections",
    "qa_inspection_items",
    "qa_non_conformances",
    "qa_capa",
    "qa_audit_schedules",
    "qa_schedule_items",
    "qa_sop_documents",
    "qa_sop_acknowledgments",
    // 016 - GLP/法規合規（依賴 users / protocols / products / buildings / zones / electronic_signatures）
    "reference_standards",
    "controlled_documents",
    "document_revisions",
    "document_acknowledgments",
    "management_reviews",
    "risk_register",
    "change_requests",
    "environment_monitoring_points",
    "environment_readings",
    "competency_assessments",
    "role_training_requirements",
    "study_final_reports",
    "formulation_records",
];

/// 稽核相關大表（include_audit=false 時略過）
const AUDIT_HEAVY_TABLES: &[&str] = &["user_activity_logs", "login_events"];

/// 故意不匯出的表（敏感 token / SQLx 內部表 / 動態建立的分區子表）
///
/// - `jwt_blacklist` / `refresh_tokens` / `password_reset_tokens`：JWT/重設密碼短期 token，
///   匯出後恢復會造成驗證狀態錯亂；不視為業務資料。
/// - `_sqlx_migrations`：SQLx 內部表，由 migration 自動管理。
/// - `user_activity_logs_*` / `ai_query_logs_*`：父表（`user_activity_logs`、`ai_query_logs`）
///   為 partitioned table，子分區資料透過父表查詢即可，不需另外列出。
pub const INTENTIONALLY_EXCLUDED_TABLES: &[&str] = &[
    "jwt_blacklist",
    "refresh_tokens",
    "password_reset_tokens",
    "_sqlx_migrations",
];

/// 從 _sqlx_migrations 讀取最新 schema 版本，格式為 "001".."010"
pub async fn get_schema_version(pool: &PgPool) -> Result<String> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(version), 0)::bigint FROM _sqlx_migrations WHERE success = true",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match row {
        Some((v,)) if v > 0 => Ok(format!("{:03}", v)),
        _ => Err(AppError::Internal("No migration records found".into())),
    }
}

/// 匯出全庫
pub async fn export_full_database(pool: &PgPool, params: ExportParams) -> Result<Vec<u8>> {
    match params.format {
        ExportFormat::Json => export_as_single_json(pool, &params).await,
        ExportFormat::Zip => export_as_zip(pool, &params).await,
    }
}

async fn export_as_single_json(pool: &PgPool, params: &ExportParams) -> Result<Vec<u8>> {
    let schema_ver = get_schema_version(pool).await?;
    let mut tables = Vec::new();

    for &table in EXPORT_TABLE_ORDER {
        if !params.include_audit && AUDIT_HEAVY_TABLES.contains(&table) {
            continue;
        }
        match export_table(pool, table).await {
            Ok(t) => tables.push(t),
            Err(e) => tracing::warn!("Skip table {}: {}", table, e),
        }
    }

    let meta = meta_json(&schema_ver);
    let output = serde_json::json!({ "meta": meta, "tables": tables });
    serde_json::to_vec_pretty(&output).map_err(|e| AppError::Internal(format!("JSON serialize: {}", e)))
}

async fn export_as_zip(pool: &PgPool, params: &ExportParams) -> Result<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let schema_ver = get_schema_version(pool).await?;
    let meta = meta_json(&schema_ver);
    let opts = FileOptions::default().unix_permissions(0o644);

    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    let mut table_entries: Vec<serde_json::Value> = Vec::new();
    let mut table_contents: Vec<(String, String)> = Vec::new();

    for &table in EXPORT_TABLE_ORDER {
        if !params.include_audit && AUDIT_HEAVY_TABLES.contains(&table) {
            continue;
        }
        let t = match export_table(pool, table).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Skip table {}: {}", table, e);
                continue;
            }
        };
        let rows_arr = t.rows.as_array().map(|a| a.len()).unwrap_or(0);
        let use_ndjson = rows_arr > LARGE_TABLE_THRESHOLD;
        let ext = if use_ndjson { "jsonl" } else { "json" };
        let path = format!("tables/{}.{}", table, ext);

        let content = if use_ndjson {
            let arr = t.rows.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
            arr.iter()
                .map(|r| serde_json::to_string(r).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            serde_json::to_string(&t.rows).map_err(|e| AppError::Internal(e.to_string()))?
        };

        table_entries.push(serde_json::json!({
            "name": t.name,
            "file": path,
            "format": if use_ndjson { "ndjson" } else { "json" },
            "columns": t.columns,
        }));
        table_contents.push((path, content));
    }

    let manifest = serde_json::json!({ "meta": meta, "tables": table_entries });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| AppError::Internal(e.to_string()))?;
    zip.start_file("manifest.json", opts).map_err(|e| AppError::Internal(format!("zip: {}", e)))?;
    zip.write_all(&manifest_bytes).map_err(|e| AppError::Internal(format!("zip write: {}", e)))?;

    for (path, content) in table_contents {
        zip.start_file(&path, opts).map_err(|e| AppError::Internal(format!("zip: {}", e)))?;
        zip.write_all(content.as_bytes()).map_err(|e| AppError::Internal(format!("zip write: {}", e)))?;
    }

    zip.finish().map_err(|e| AppError::Internal(format!("zip finish: {}", e)))?;
    drop(zip);
    Ok(buf.into_inner())
}

fn meta_json(schema_ver: &str) -> serde_json::Value {
    serde_json::json!({
        "format": "ipig-idxf",
        "format_version": "1.0",
        "schema_version": schema_ver,
        "exported_at": Utc::now().to_rfc3339(),
        "source": "ipig_system",
        "migration_applied": schema_ver,
    })
}

#[derive(serde::Serialize)]
struct TableExport {
    name: String,
    columns: Vec<String>,
    rows: Value,
}

async fn export_table(pool: &PgPool, table: &str) -> Result<TableExport> {
    // 顯式白名單檢查
    if !EXPORT_TABLE_ORDER.contains(&table) {
        return Err(AppError::BadRequest(format!("Table '{}' is not in export whitelist", table)));
    }

    // 取得欄位名稱
    let col_rows: Vec<(String,)> = sqlx::query_as(
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

    let columns: Vec<String> = col_rows.into_iter().map(|r| r.0).collect();

    if columns.is_empty() {
        return Err(AppError::Internal(format!("Table {} has no columns or does not exist", table)));
    }

    // 使用 row_to_json 取得整表 JSON，避免逐欄型別轉換
    // 表名來自常數 EXPORT_TABLE_ORDER，非使用者輸入
    let sql = format!(
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (SELECT * FROM "{}") t"#,
        table
    );
    let rows: Option<serde_json::Value> = sqlx::query_scalar(&sql)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Query {}: {}", table, e)))?;

    Ok(TableExport {
        name: table.to_string(),
        columns,
        rows: rows.unwrap_or(serde_json::Value::Array(vec![])),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    /// 從 migrations 目錄掃出所有 `CREATE TABLE` 表名（排除 partition 子表）。
    ///
    /// 規則：
    /// - 匹配 `CREATE TABLE [IF NOT EXISTS] <name>`
    /// - 排除 `PARTITION OF` 子分區（其資料透過父表匯出）
    /// - 不處理 PL/pgSQL `EXECUTE format(...)` 內的動態 CREATE（partition 才會用，已被上一條排除）
    fn scan_migrations_for_create_tables() -> BTreeSet<String> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let migrations_dir = PathBuf::from(manifest_dir).join("migrations");
        let mut tables = BTreeSet::new();

        let entries = fs::read_dir(&migrations_dir)
            .unwrap_or_else(|e| panic!("read_dir {:?}: {}", migrations_dir, e));

        // 行內正規式：
        // ^\s*CREATE\s+TABLE\s+(IF\s+NOT\s+EXISTS\s+)?<ident>(...)
        // 用簡單字串掃描即可，避免引入 regex crate。
        for entry in entries {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("sql") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
            for raw_line in content.lines() {
                let line = raw_line.trim();
                // 跳過註解
                if line.starts_with("--") {
                    continue;
                }
                let upper = line.to_ascii_uppercase();
                if !upper.starts_with("CREATE TABLE") {
                    continue;
                }
                // 跳過 partition 子表
                if upper.contains("PARTITION OF") {
                    continue;
                }
                // 取出表名 token：去掉 CREATE TABLE [IF NOT EXISTS]
                let mut rest = line["CREATE TABLE".len()..].trim_start();
                let upper_rest = rest.to_ascii_uppercase();
                if upper_rest.starts_with("IF NOT EXISTS") {
                    rest = rest["IF NOT EXISTS".len()..].trim_start();
                }
                // 表名到第一個非 ident 字元為止
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    tables.insert(name);
                }
            }
        }
        tables
    }

    #[test]
    fn scan_migrations_finds_known_tables() {
        // sanity check: scanner 至少要找到幾個明顯存在的表
        let tables = scan_migrations_for_create_tables();
        for required in ["users", "animals", "protocols", "audit_logs"] {
            assert!(
                tables.contains(required),
                "scanner 沒找到 {}（migrations 目錄可能定位錯誤）",
                required
            );
        }
    }

    #[test]
    fn export_covers_all_business_tables() {
        let migrations_tables = scan_migrations_for_create_tables();
        let exported: BTreeSet<&str> = EXPORT_TABLE_ORDER.iter().copied().collect();
        let excluded: BTreeSet<&str> = INTENTIONALLY_EXCLUDED_TABLES.iter().copied().collect();

        let missing: Vec<&str> = migrations_tables
            .iter()
            .map(String::as_str)
            .filter(|t| !exported.contains(t) && !excluded.contains(t))
            .collect();

        assert!(
            missing.is_empty(),
            "Migrations 中有表未列入 EXPORT_TABLE_ORDER 也未列入 INTENTIONALLY_EXCLUDED_TABLES：{:?}\n\
             → 修法：在 services/data_export.rs 將表加入 EXPORT_TABLE_ORDER（依 FK 順序），\n\
             或加入 INTENTIONALLY_EXCLUDED_TABLES 並註明原因。",
            missing
        );
    }

    #[test]
    fn no_phantom_tables_in_export_order() {
        // EXPORT_TABLE_ORDER 中每張表都應實際存在於 migrations
        let migrations_tables = scan_migrations_for_create_tables();
        let phantom: Vec<&str> = EXPORT_TABLE_ORDER
            .iter()
            .copied()
            .filter(|t| !migrations_tables.contains(*t))
            .collect();
        assert!(
            phantom.is_empty(),
            "EXPORT_TABLE_ORDER 含有 migrations 中不存在的表：{:?}",
            phantom
        );
    }

    #[test]
    fn no_duplicate_in_export_order() {
        let mut seen = BTreeSet::new();
        let mut dup = Vec::new();
        for t in EXPORT_TABLE_ORDER {
            if !seen.insert(*t) {
                dup.push(*t);
            }
        }
        assert!(dup.is_empty(), "EXPORT_TABLE_ORDER 有重複：{:?}", dup);
    }
}
