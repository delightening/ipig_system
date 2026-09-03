//! 全庫 IDXF 匯出服務
//!
//! 一鍵輸出整個資料庫為 iPig Data Exchange Format (IDXF)，
//! 可在不同 migration 版本間讀取。

use chrono::{FixedOffset, Utc};
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
            // R30-19：預設改為 true 以符合 GLP / 21 CFR §11.10(c) 對「準確完整紀錄副本」的要求。
            // 不含 audit 重建後 HMAC chain 會斷裂，無法通過完整性驗證；僅在非合規用途
            // （如資料遷移）才應顯式關閉。
            include_audit: true,
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
    // 004（R106-1）人員隸屬身分：類型主檔須在身分列之前（FK），
    // 身分列同時依賴 users 與 departments，故排在兩者之後。
    "affiliation_types",
    "personnel_affiliations",
    "role_permissions",
    "user_roles",
    "user_preferences",
    "user_mcp_keys",
    "invitations",
    "invitation_roles",
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
    "care_medication_records",
    "record_versions",
    "import_jobs",
    "export_jobs",
    "euthanasia_orders",
    "euthanasia_appeals",
    // R53-1 廢棄物再利用紀錄（FK → euthanasia_orders + animals + protocols + users）
    "euthanasia_byproduct_samples",
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
    "vet_patrol_photos",
    // R39: entry-level 照片附件（FK → vet_patrol_entries），順序在 entries 之後
    "vet_patrol_entry_photos",
    // R39+++ 多動物 junction（FK → vet_patrol_entries + animals），順序在 entries 之後
    "vet_patrol_entry_animals",
    // R40-A 站內信
    "message_threads",
    "message_thread_participants",
    "messages",
    "message_attachments",
    // 003 - AUP
    "protocols",
    // 117 - 動物預約與試驗規劃（FK → protocols + users；被 animals.reserved_planned_experiment_id 參照）
    "planned_experiments",
    "user_protocols",
    "protocol_versions",
    "protocol_template_versions",
    // 098 - 申請須知版次（FK → attachments / users）
    "application_notices",
    "review_assignments",
    "review_comments",
    "protocol_attachments",
    "pi_account_invites",
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
    // 097 - HMAC chain 已知斷鏈白名單（須隨備份還原，否則 verifier 會對歷史 35 筆重新告警）
    "audit_chain_known_breaks",
    // 006 - ERP 明細
    "storage_locations",
    "products",
    "product_uom_conversions",
    "documents",
    "document_lines",
    // 131 - 上架分配審計（FK → document_lines / storage_locations / products / users）
    "line_shelf_allocations",
    "storage_location_inventory",
    "stock_ledger",
    "inventory_snapshots",
    "expiry_notification_config",
    "expiry_monthly_snapshots",
    // 007 - 補充
    "notification_routing",
    "electronic_signatures",
    // 098 - 須知簽署紀錄（FK → protocols / application_notices / users / electronic_signatures / attachments）
    "protocol_notice_acknowledgements",
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
    // 042 - R30-17 retention policy（系統設定，無 FK）
    "data_retention_policies",
    // 053 - R32-A6 PDF artifacts（GLP 永久存證；FK → users / electronic_signatures /
    // attachments，皆已在前面建立）
    "pdf_artifacts",
];

/// 稽核相關大表（include_audit=false 時略過）
const AUDIT_HEAVY_TABLES: &[&str] = &[
    "user_activity_logs",
    "user_sessions",
    "user_activity_aggregates",
    "login_events",
];

/// 故意不匯出的表（敏感 token / SQLx 內部表 / 動態建立的分區子表）
///
/// - `jwt_blacklist` / `refresh_tokens` / `password_reset_tokens`：JWT/重設密碼短期 token，
///   匯出後恢復會造成驗證狀態錯亂；不視為業務資料。
/// - `_sqlx_migrations`：SQLx 內部表，由 migration 自動管理。
/// - `user_activity_logs_*` / `ai_query_logs_*`：父表（`user_activity_logs`、`ai_query_logs`）
///   為 partitioned table，子分區資料透過父表查詢即可，不需另外列出。
#[cfg(test)]
const INTENTIONALLY_EXCLUDED_TABLES: &[&str] = &[
    "jwt_blacklist",
    "refresh_tokens",
    "password_reset_tokens",
    "_sqlx_migrations",
    // R30-27c：bridge session 為 5min TTL 短期 token，匯出後恢復狀態錯亂、且 plaintext
    // payload 含密碼即便 hash 也屬敏感；不視為業務資料。
    "signature_bridge_sessions",
    // R30-3a：event_outbox 為 worker 內部佇列（短命操作狀態），匯出後恢復會
    // re-send 已送出的通知 / webhook，造成重複通知；不視為業務資料。
    "event_outbox",
    // migration 007 建立，但從未有寫入端（空表）；狀態轉移時間軸改由 protocol_activities 承載。
    // migration 080 已 DROP TABLE，此處保留讓測試掃描器略過 migration 007 的 CREATE TABLE 宣告。
    "protocol_status_history",
    // migration 006 建立，migration 130 已 DROP TABLE（vet_recommendations 功能退役，獸醫建議
    // 單一來源改為 animal_vet_advice_records）；此處保留讓測試掃描器略過 006 的 CREATE TABLE 宣告。
    "vet_recommendations",
    // migration 011 的回退用鷹架，不是業務資料：記下該次遷移改寫掉的單位別名原值
    // （`BX` → `盒`）、被去重刪掉的列，以及實際插入的列 id，供人工回退腳本精確還原。
    //
    // 為什麼排除而不是匯出：它們存的是**遷移當下的暫態副本**。匯出後還原會把過期的舊寫法
    // 帶回一個已經正規化過的資料庫，反而製造出正規列與別名列並存的髒資料
    // ——那正是 011 要消滅的東西。
    //
    // ⚠️ 已知代價：整庫還原之後這三張表會是空的，屆時 011 的回退腳本只刪得掉它插入的列、
    // 還原不了別名原值。這個順序（先整庫還原、再回退 011）極罕見，且實查正式環境的換算表
    // 本身就是空的、備份內容為空。011 確認不再需要回退後，這三張表可自行 DROP。
    "mig011_conversion_backup",
    "mig011_pack_unit_backup",
    "mig011_inserted_conversion",
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

    // GMT+8：在單一交易內 SET LOCAL TIME ZONE，使所有 timestamptz 以台灣時間 (+08:00)
    // 輸出（同一瞬間）。SET LOCAL 僅作用於本交易，commit 後自動還原，不洩漏連線池。
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin export tx: {}", e)))?;
    sqlx::query("SET LOCAL TIME ZONE 'Asia/Taipei'")
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("set export tz: {}", e)))?;

    for &table in EXPORT_TABLE_ORDER {
        if !params.include_audit && AUDIT_HEAVY_TABLES.contains(&table) {
            continue;
        }
        match export_table(&mut tx, table).await {
            Ok(t) => tables.push(t),
            Err(e) => tracing::warn!("Skip table {}: {}", table, e),
        }
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit export tx: {}", e)))?;

    let meta = meta_json(&schema_ver);
    let output = serde_json::json!({ "meta": meta, "tables": tables });
    serde_json::to_vec_pretty(&output)
        .map_err(|e| AppError::Internal(format!("JSON serialize: {}", e)))
}

async fn export_as_zip(pool: &PgPool, params: &ExportParams) -> Result<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let schema_ver = get_schema_version(pool).await?;
    let meta = meta_json(&schema_ver);
    let opts = SimpleFileOptions::default().unix_permissions(0o644);

    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    let mut table_entries: Vec<serde_json::Value> = Vec::new();
    let mut table_contents: Vec<(String, String)> = Vec::new();

    // GMT+8：同 single-json 路徑，SET LOCAL TIME ZONE 於單一交易內
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin export tx: {}", e)))?;
    sqlx::query("SET LOCAL TIME ZONE 'Asia/Taipei'")
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("set export tz: {}", e)))?;

    for &table in EXPORT_TABLE_ORDER {
        if !params.include_audit && AUDIT_HEAVY_TABLES.contains(&table) {
            continue;
        }
        let t = match export_table(&mut tx, table).await {
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
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit export tx: {}", e)))?;

    let manifest = serde_json::json!({ "meta": meta, "tables": table_entries });
    let manifest_bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| AppError::Internal(e.to_string()))?;
    zip.start_file("manifest.json", opts)
        .map_err(|e| AppError::Internal(format!("zip: {}", e)))?;
    zip.write_all(&manifest_bytes)
        .map_err(|e| AppError::Internal(format!("zip write: {}", e)))?;

    for (path, content) in table_contents {
        zip.start_file(&path, opts)
            .map_err(|e| AppError::Internal(format!("zip: {}", e)))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| AppError::Internal(format!("zip write: {}", e)))?;
    }

    zip.finish()
        .map_err(|e| AppError::Internal(format!("zip finish: {}", e)))?;
    Ok(buf.into_inner())
}

fn meta_json(schema_ver: &str) -> serde_json::Value {
    serde_json::json!({
        "format": "ipig-idxf",
        "format_version": "1.0",
        "schema_version": schema_ver,
        // GMT+8 台灣時間（同一瞬間，+08:00 offset）
        "exported_at": Utc::now()
            .with_timezone(&FixedOffset::east_opt(8 * 3600).expect("valid +08:00 offset"))
            .to_rfc3339(),
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

async fn export_table(conn: &mut sqlx::PgConnection, table: &str) -> Result<TableExport> {
    // 顯式白名單檢查
    if !EXPORT_TABLE_ORDER.contains(&table) {
        return Err(AppError::BadRequest(format!(
            "Table '{}' is not in export whitelist",
            table
        )));
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
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(format!("Columns for {}: {}", table, e)))?;

    let columns: Vec<String> = col_rows.into_iter().map(|r| r.0).collect();

    if columns.is_empty() {
        return Err(AppError::Internal(format!(
            "Table {} has no columns or does not exist",
            table
        )));
    }

    // 使用 row_to_json 取得整表 JSON，避免逐欄型別轉換
    // 表名來自常數 EXPORT_TABLE_ORDER，非使用者輸入
    let sql = format!(
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (SELECT * FROM "{}") t"#,
        table
    );
    let rows: Option<serde_json::Value> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(&mut *conn)
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

    /// 剝掉 SQL 內的 `--` 行註解，但**保留** single-quote 字串字面值內的 `--`。
    /// 處理 SQL 標準的 `''` 雙引號 escape。
    fn strip_line_comments(content: &str) -> String {
        let mut out = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_string = false;
        while let Some(c) = chars.next() {
            if in_string {
                out.push(c);
                if c == '\'' {
                    if chars.peek() == Some(&'\'') {
                        // SQL 標準：`''` 為 escape，仍在字串內
                        if let Some(next_c) = chars.next() {
                            out.push(next_c);
                        }
                    } else {
                        in_string = false;
                    }
                }
            } else if c == '\'' {
                out.push(c);
                in_string = true;
            } else if c == '-' && chars.peek() == Some(&'-') {
                // 行註解：消耗 `--` 與其後內容到行尾（保留換行）
                chars.next();
                while let Some(&next_c) = chars.peek() {
                    if next_c == '\n' {
                        break;
                    }
                    chars.next();
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    /// 從單一 SQL 檔抽出所有 `CREATE TABLE` 表名。
    ///
    /// 以「statement = `;` 切」為單位掃描，支援多行 `CREATE TABLE`、多空格、
    /// 引號識別子（`"users"`）、schema-qualified（`public.users`）、以及
    /// `IF NOT EXISTS`。
    ///
    /// **排除分區子表，兩種寫法都要認得**：
    ///   1. `CREATE TABLE x PARTITION OF y ...`（手寫 migration 的慣用寫法）
    ///   2. `CREATE TABLE x (...)` ＋另一句 `ALTER TABLE y ATTACH PARTITION x ...`
    ///      （`pg_dump` 的輸出形式——它把建表與掛載拆成兩句）
    ///
    /// 2026-08-20 踩過：migration squash 後 `002_schema.sql` 改由 `pg_dump` 產生，
    /// 全檔 0 個 `PARTITION OF`、210 個 `ATTACH PARTITION`，導致 14 張分區子表
    /// （`user_activity_logs_*` / `ai_query_logs_*`）被誤判為未歸類的業務表，
    /// `export_covers_all_business_tables` 失敗。
    fn extract_create_table_names(content: &str) -> BTreeSet<String> {
        // 先剝掉 `--` 行註解（注意保留字串字面值內的 `--`），
        // 避免註解行混入 statement 開頭影響後續 split(';') 後的 token 判斷。
        let stripped = strip_line_comments(content);

        // 正規化識別子：去引號、去 schema 前綴、只留合法字元。
        // `public."my_table"` / `"my_table"` / `my_table(` 都會得到 `my_table`。
        fn bare_name(tok: &str) -> String {
            let unquoted = tok.trim_matches('"');
            let bare = unquoted.rsplit('.').next().unwrap_or(unquoted);
            bare.trim_matches('"')
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect()
        }

        // 第一輪：收集所有「被 ATTACH 成分區」的子表名。
        // pg_dump 把分區拆成兩句寫，光看 CREATE TABLE 分不出它是不是子分區，
        // 必須先掃過 ALTER ... ATTACH PARTITION 才知道。
        let mut attached_partitions = BTreeSet::new();
        for stmt in stripped.split(';') {
            let upper = stmt.to_ascii_uppercase();
            if !upper.contains("ATTACH PARTITION") {
                continue;
            }
            // 形如：ALTER TABLE ONLY public.parent ATTACH PARTITION public.child FOR VALUES ...
            let toks: Vec<&str> = stmt.split_whitespace().collect();
            for i in 0..toks.len().saturating_sub(2) {
                if toks[i].eq_ignore_ascii_case("ATTACH")
                    && toks[i + 1].eq_ignore_ascii_case("PARTITION")
                {
                    let child = bare_name(toks[i + 2]);
                    if !child.is_empty() {
                        attached_partitions.insert(child);
                    }
                }
            }
        }

        let mut names = BTreeSet::new();
        for stmt in stripped.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            let upper = trimmed.to_ascii_uppercase();
            if !upper.contains("CREATE") || !upper.contains("TABLE") {
                continue;
            }
            if upper.contains("PARTITION OF") {
                continue;
            }
            let mut toks = trimmed.split_whitespace();
            if !matches!(toks.next(), Some(t) if t.eq_ignore_ascii_case("CREATE")) {
                continue;
            }
            if !matches!(toks.next(), Some(t) if t.eq_ignore_ascii_case("TABLE")) {
                continue;
            }
            let mut name_tok = toks.next().unwrap_or_default();
            if name_tok.eq_ignore_ascii_case("IF") {
                let _ = toks.next(); // NOT
                let _ = toks.next(); // EXISTS
                name_tok = toks.next().unwrap_or_default();
            }
            let name = bare_name(name_tok);
            // 被 ATTACH 成分區的子表不算業務表（pg_dump 形式；`PARTITION OF` 形式已在上面擋掉）
            if !name.is_empty() && !attached_partitions.contains(&name) {
                names.insert(name);
            }
        }
        names
    }

    /// 從 migrations 目錄掃出所有 `CREATE TABLE` 表名（排除 partition 子表）。
    fn scan_migrations_for_create_tables() -> BTreeSet<String> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let migrations_dir = PathBuf::from(manifest_dir).join("migrations");
        let mut tables = BTreeSet::new();

        let entries = fs::read_dir(&migrations_dir)
            .unwrap_or_else(|e| panic!("read_dir {:?}: {}", migrations_dir, e));

        for entry in entries {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("sql") {
                continue;
            }
            let content =
                fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
            tables.extend(extract_create_table_names(&content));
        }
        tables
    }

    /// 分區子表的兩種寫法都必須被排除。
    ///
    /// 2026-08-20 迴歸來源：migration squash 後 `002_schema.sql` 改由 `pg_dump` 產生，
    /// 它把分區拆成「一般 CREATE TABLE」＋「ALTER ... ATTACH PARTITION」兩句寫，
    /// 而原本的掃描器只認 `PARTITION OF`，於是 14 張分區子表被當成未歸類的業務表。
    #[test]
    fn scanner_excludes_partitions_in_both_syntaxes() {
        // 形式 1：手寫 migration 的慣用寫法
        let inline = r#"
            CREATE TABLE parent_tbl (id uuid) PARTITION BY RANGE (created_at);
            CREATE TABLE child_inline PARTITION OF parent_tbl FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
            CREATE TABLE normal_tbl (id uuid);
        "#;
        let got = extract_create_table_names(inline);
        assert!(got.contains("parent_tbl"), "父表應該被列入");
        assert!(got.contains("normal_tbl"), "一般表應該被列入");
        assert!(
            !got.contains("child_inline"),
            "PARTITION OF 形式的子分區不該被列入"
        );

        // 形式 2：pg_dump 的輸出形式——建表與掛載分成兩句
        let pg_dump_style = r#"
            CREATE TABLE public.parent_tbl (id uuid) PARTITION BY RANGE (created_at);
            CREATE TABLE public.child_attached (id uuid);
            CREATE TABLE public.normal_tbl (id uuid);
            ALTER TABLE ONLY public.parent_tbl ATTACH PARTITION public.child_attached FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
        "#;
        let got = extract_create_table_names(pg_dump_style);
        assert!(got.contains("parent_tbl"), "父表應該被列入");
        assert!(got.contains("normal_tbl"), "一般表應該被列入");
        assert!(
            !got.contains("child_attached"),
            "ATTACH PARTITION 形式的子分區不該被列入（掃描器必須先掃 ATTACH 才知道它是子分區）"
        );
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
    fn strip_line_comments_preserves_string_literals() {
        // 字串字面值內的 `--` 不可被當行註解切掉
        let sql = "INSERT INTO foo (msg) VALUES ('-- not a comment'); CREATE TABLE bar (id INT);";
        let stripped = strip_line_comments(sql);
        assert!(stripped.contains("'-- not a comment'"));
        // 解析後應抓到 bar
        let names = extract_create_table_names(sql);
        assert!(names.contains("bar"));
    }

    #[test]
    fn strip_line_comments_handles_escaped_single_quote() {
        // SQL 標準 `''` escape：字串內的 `''` 不結束字串
        let sql = "INSERT INTO foo VALUES ('it''s -- ok'); CREATE TABLE baz (id INT);";
        let names = extract_create_table_names(sql);
        assert!(names.contains("baz"));
    }

    #[test]
    fn strip_line_comments_strips_real_comments() {
        let sql = "-- header comment\nCREATE TABLE qux (id INT); -- trailing\n";
        let names = extract_create_table_names(sql);
        assert!(names.contains("qux"));
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
