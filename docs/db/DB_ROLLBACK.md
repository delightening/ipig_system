# iPIG 資料庫 Rollback 參考手冊

> **用途**：本文件記錄 UP migration 的反向 SQL，供手動回退資料庫結構時參考。涵蓋 001–014、022、033–037。
> **TODO**：Migration 015–021（HR/AUP 擴充）、023–032（QAU / R19 邀請 / R21 環境監控 / R22 攻擊偵測等）尚未補齊，列為後續文件補強。
> **SQLx 注意**：SQLx（Rust）不原生支援 DOWN migration，因此所有回退操作需透過手動執行 SQL 完成。
> **R30-26（041 起）**：新 migration 必須於 `backend/migrations/down/NNN_xxx.sql` 附對應 down SQL，CI `migration-down-guard` 會強制檢查。撰寫規範見 [backend/migrations/down/README.md](../../backend/migrations/down/README.md)。本文件保留作為 001-040 既有 migration 的歷史 down SQL 索引（R30-26b backlog 將補齊缺漏項）。

---

## Migration checksum 錯誤（「was previously applied but has been modified」）

當 `sqlx migrate run` 出現此錯誤，通常是 migration 檔案的 **CRLF/LF 換行符** 在 Windows/Linux 間不一致。

**解法（開發環境）：**

```powershell
cd backend
cargo run --bin fix_migration_checksum
sqlx migrate run
```

`fix_migration_checksum` 會將 `_sqlx_migrations` 中**所有已套用** migration 的 checksum 更新為當前檔案的 checksum。  
為避免未來發生，`migrations/.gitattributes` 已設定 `*.sql text eol=lf`，請確保 Git 有正確 checkout。

### 若出現 "relation X does not exist" 於後續 migration

表示 `_sqlx_migrations` 紀錄的已套用 migrations 與實際 schema 不一致（例如 DB 曾被還原或手動清空）。  
**開發環境**建議直接重設 DB 並從頭執行 migrations：

```powershell
# 取得 DATABASE_URL 中的資料庫名稱（例如 ipig_db）
# 然後：
psql -U postgres -c "DROP DATABASE IF EXISTS ipig_db;"
psql -U postgres -c "CREATE DATABASE ipig_db;"
cd backend
sqlx migrate run
```

若使用 Docker 或其他連線方式，請依環境調整 `psql` 參數。

---

## ⚠️ 重要警告

1. **資料不可逆**：大部分 rollback 會 `DROP TABLE`，表中所有資料將**永久遺失**且無法復原。
2. **外鍵相依**：必須按照 **逆序（037 → 001，含已文檔化之 001–014、022、033–037）** 執行 rollback，否則會因外鍵約束而失敗。
3. **先備份**：執行任何 rollback 之前，請先使用 `pg_dump` 完整備份資料庫。
4. **ENUM 刪除風險**：刪除自訂類型（ENUM）會影響所有引用該類型的表；需確保所有相關表已先行刪除。
5. **分區表**：`user_activity_logs` 為分區表，必須先刪除所有分區子表，再刪除主表。
6. **Views 先於 Tables**：如果有 VIEW 依賴某張表，需先 `DROP VIEW` 再 `DROP TABLE`。
7. **CASTs / Functions**：自訂 CAST 和 FUNCTION 必須手動移除，PostgreSQL 不會自動清理。

---

## 建議回退流程

```text
1. 停止所有後端服務（確保無連線寫入）
2. 完整備份：pg_dump -Fc -f backup_$(date +%Y%m%d_%H%M%S).dump ipig_db
3. 在 psql 或 pgAdmin 中開啟交易：BEGIN;
4. 按逆序執行所需的 rollback SQL（從已文檔化的最新版本向下至目標版本）
5. 確認無報錯後：COMMIT;（如有問題則 ROLLBACK;）
6. 手動更新 SQLx 的 _sqlx_migrations 表（刪除對應的 migration 紀錄）：
   DELETE FROM _sqlx_migrations WHERE version >= <目標版本>;
7. 重啟後端服務
```

---

## Migrations 033–037: R26 Service-driven Audit Refactor

> **R26 Epic 整體 Rollback 警告**：
> - 033–037 屬同一 epic，**強烈建議整批回滾**（依序 037 → 036 → 035 → 034 → 033）。
> - 已部署到 production 後 **不建議 rollback**：HMAC chain 已寫入新版 row，DROP COLUMN 會永久遺失資料。
> - 緊急 rollback 限「dev/staging 部署 < 24h 內」場景；prod 緊急狀況請先聯絡資安團隊評估。

---

### Migration 037: HMAC 編碼版本化

**原始操作**：對 `user_activity_logs` 新增 `hmac_version SMALLINT` 欄位 + partial index，標示 HMAC 編碼版本（1=legacy string-concat, 2=length-prefix canonical）。

```sql
-- Rollback 037: 移除 HMAC 版本欄位與索引
DROP INDEX IF EXISTS idx_user_activity_logs_hmac_version;
ALTER TABLE user_activity_logs DROP COLUMN IF EXISTS hmac_version;
```

**Backfill runbook**（forward 部署後執行；尚未自動化）：

> **⚠️ 分區表限制**：`user_activity_logs` 為 PostgreSQL 分區表，**分區表上的 `UPDATE` 語句不支援 `LIMIT` 子句**。需用子查詢 `WHERE id IN (SELECT ... LIMIT)` 迂迴。另因 `(id, partition_date)` 為複合 PK，子查詢需回傳兩者。

```sql
-- 方式 A：psql 腳本迴圈（推薦，可監控進度）
-- 將既有 row 標記為 legacy v1（idempotent，可分批執行避免長時間鎖表）
DO $$
DECLARE
  batch_size INT := 100000;
  rows_affected INT;
BEGIN
  LOOP
    UPDATE user_activity_logs
    SET hmac_version = 1
    WHERE (id, partition_date) IN (
      SELECT id, partition_date
      FROM user_activity_logs
      WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL
      LIMIT batch_size
    );
    GET DIAGNOSTICS rows_affected = ROW_COUNT;
    RAISE NOTICE 'Backfilled % rows', rows_affected;
    EXIT WHEN rows_affected = 0;
    PERFORM pg_sleep(2);  -- 每批 sleep 2 秒讓線上流量喘息
  END LOOP;
END $$;

-- 方式 B：直接 UPDATE 全表（小系統 / 低流量時段）
UPDATE user_activity_logs
SET hmac_version = 1
WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL;
```

> **注意**：未 backfill 也可運作，verifier 對 `hmac_version IS NULL AND integrity_hash IS NOT NULL` 的 row 採 **try-both** 策略 — 先比對 canonical (v=2)，不符再 fallback legacy (v=1)。此設計涵蓋兩類 legacy row：migration 037 前由新版 `log_activity_tx` 寫入（實為 v=2）、以及更早期舊版 `log_activity` 寫入（v=1）。Backfill 後所有 row 都有明確 `hmac_version`，verifier 即可單一路徑比對，也讓 SQL 報表可直接用 `hmac_version = 1` 篩選 legacy row。

---

### Migration 036: log_activity stored proc v3

**原始操作**：DROP + CREATE `log_activity()` 函式（12 參數版本），修正 `changed_fields` fallback 漏偵測「被刪除的 key」。

```sql
-- Rollback 036: 還原為 035 (v2) 的 stored proc
DROP FUNCTION IF EXISTS log_activity(
    UUID, VARCHAR, VARCHAR, VARCHAR, UUID, VARCHAR,
    JSONB, JSONB, INET, TEXT, UUID, TEXT[]
);

-- 重新套用 035_audit_log_activity_v2.sql（從 v3 退回 v2）
-- 完整 SQL 見 backend/migrations/035_audit_log_activity_v2.sql
-- 注意：v2 的 changed_fields fallback 會漏偵測被刪除的 key（這是 035 → 036 修補的 bug）
```

> **⚠️ 警告**：rollback 到 v2 後，UPDATE 類 audit row 在「app 層未提供 `changed_fields`」時會缺少被刪除的 key。對於使用 `DataDiff::compute` 的呼叫者（R26-3 後 100% 都是）無影響，但對於早期 fire-and-forget call sites 會 regress。

---

### Migration 035: log_activity stored proc v2

**原始操作**：DROP + CREATE `log_activity()` 函式（10 參數 → 12 參數），新增 `impersonated_by_user_id` + `changed_fields` 參數，使 HMAC 計算涵蓋這兩欄。

```sql
-- Rollback 035: 還原為 v1 的 stored proc（10 參數版本）
DROP FUNCTION IF EXISTS log_activity(
    UUID, VARCHAR, VARCHAR, VARCHAR, UUID, VARCHAR,
    JSONB, JSONB, INET, TEXT, UUID, TEXT[]
);

-- 重新套用 v1（migration 006 的版本）
-- 完整 SQL 見 backend/migrations_old/006_audit_system.sql 或 git history
```

> **⚠️ 嚴重警告**：rollback 到 v1 後 `impersonated_by_user_id` 會無法寫入 audit log；攻擊者能繞過 SEC-11 impersonate 追蹤。**僅限 dev/staging 緊急情況使用**。

---

### Migration 034: Audit Impersonation Column

**原始操作**：對 `user_activity_logs` 新增 `impersonated_by_user_id UUID REFERENCES users(id)` 欄位 + partial index，記錄 SEC-11 impersonate 操作真正執行的管理員。

```sql
-- Rollback 034: 移除 impersonation 追蹤欄位與索引
DROP INDEX IF EXISTS idx_user_activity_logs_impersonated_by;
ALTER TABLE user_activity_logs DROP COLUMN IF EXISTS impersonated_by_user_id;
```

> **注意**：`user_activity_logs` 為大表（hypertable），DROP COLUMN 在 prod 會鎖定每個 partition，建議在低流量時段執行。已寫入的 impersonate 追蹤資料將永久遺失，影響 GLP §11.10 合規。

---

### Migration 033: System User

**原始操作**：INSERT 一筆 reserved UUID `00000000-0000-0000-0000-000000000001` 作為 SYSTEM actor，供 scheduler / bin / migration 等非使用者觸發的 audit 用。

```sql
-- Rollback 033: 移除 SYSTEM actor user
--
-- ⚠️ 步驟 1：確認無引用。若以下 COUNT > 0，表示 SYSTEM actor 已寫過 audit 或
-- 曾被 impersonate — **不應強制 rollback**（會破壞 audit 完整性）。
SELECT COUNT(*) FROM user_activity_logs
WHERE actor_user_id = '00000000-0000-0000-0000-000000000001'
   OR impersonated_by_user_id = '00000000-0000-0000-0000-000000000001';

-- 步驟 2：若步驟 1 返回 0，執行下列 DELETE。
-- 注意：這裡沒加 NOT EXISTS guard，讓 FK violation 以明確錯誤呈現（避免靜默
-- 跳過讓 Ops 誤以為 rollback 成功）。
DELETE FROM users WHERE id = '00000000-0000-0000-0000-000000000001';

-- 若步驟 1 返回 > 0 且**確定要強制清理（僅限 dev/staging）**：
-- BEGIN;
--   DELETE FROM user_activity_logs WHERE actor_user_id = '00000000-0000-0000-0000-000000000001';
--   DELETE FROM users WHERE id = '00000000-0000-0000-0000-000000000001';
-- COMMIT;  -- 或 ROLLBACK 反悔
```

> **⚠️ 嚴重警告**：rollback 033 會破壞 SCHEDULER / 系統觸發的 audit 鏈。**強烈建議只在 dev 環境執行**。Prod 一旦寫入 SYSTEM actor 的 audit row 後，此 user 應視為永久不可移除。

---

## Migration 022: QAU 與財務模組（整合）

**原始操作**：QAU 角色權限、會計基礎（account_type、chart_of_accounts、journal_entries、journal_entry_lines）、AP/AR 付款收款表（ap_payments、ar_receipts）。

```sql
-- Rollback 022: 依建立順序逆序刪除
DROP TABLE IF EXISTS ar_receipts;
DROP TABLE IF EXISTS ap_payments;
DROP SEQUENCE IF EXISTS ar_receipt_no_seq;
DROP SEQUENCE IF EXISTS ap_payment_no_seq;
DROP TABLE IF EXISTS journal_entry_lines;
DROP TABLE IF EXISTS journal_entries;
DROP SEQUENCE IF EXISTS journal_entry_no_seq;
DROP TABLE IF EXISTS chart_of_accounts;
DROP TYPE IF EXISTS account_type;

DELETE FROM role_permissions WHERE role_id IN (SELECT id FROM roles WHERE code = 'QAU');
DELETE FROM user_roles WHERE role_id IN (SELECT id FROM roles WHERE code = 'QAU');
DELETE FROM roles WHERE code = 'QAU';
DELETE FROM permissions WHERE code IN ('qau.dashboard.view', 'qau.protocol.view', 'qau.audit.view', 'qau.animal.view');
```

---

## Migration 014: Optimistic Locking

**原始操作**：對 `animals`、`protocols`、`animal_observations`、`animal_surgeries` 新增 `version` 欄位。

```sql
-- Rollback 014: 移除 optimistic locking version 欄位
ALTER TABLE animals DROP COLUMN IF EXISTS version;
ALTER TABLE protocols DROP COLUMN IF EXISTS version;
ALTER TABLE animal_observations DROP COLUMN IF EXISTS version;
ALTER TABLE animal_surgeries DROP COLUMN IF EXISTS version;
```

---

## Migration 013: Fix Cast Ambiguity

**原始操作**：將 012 建立的 IMPLICIT CAST 改為 ASSIGNMENT CAST。

```sql
-- Rollback 013: 移除 ASSIGNMENT CASTs（恢復為 012 的 IMPLICIT 版本）
DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

-- 重新建立 012 的 IMPLICIT CASTs
CREATE CAST (version_record_type AS text)
    WITH FUNCTION version_record_type_to_text(version_record_type) AS IMPLICIT;
CREATE CAST (text AS version_record_type)
    WITH FUNCTION text_to_version_record_type(text) AS IMPLICIT;
CREATE CAST (animal_record_type AS text)
    WITH FUNCTION animal_record_type_to_text(animal_record_type) AS IMPLICIT;
CREATE CAST (record_type AS text)
    WITH FUNCTION record_type_to_text(record_type) AS IMPLICIT;
```

---

## Migration 012: Fix Enum Casts

**原始操作**：建立 ENUM ↔ TEXT 之間的隱式轉型函式與 CAST。

```sql
-- Rollback 012: 移除所有 ENUM CAST 與轉型函式
DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

DROP FUNCTION IF EXISTS version_record_type_to_text(version_record_type);
DROP FUNCTION IF EXISTS text_to_version_record_type(text);
DROP FUNCTION IF EXISTS animal_record_type_to_text(animal_record_type);
DROP FUNCTION IF EXISTS record_type_to_text(record_type);
```

---

## Migration 011: Audit Integrity

**原始操作**：對 `user_activity_logs` 新增 `integrity_hash` 與 `previous_hash` 欄位及索引。

```sql
-- Rollback 011: 移除稽核完整性欄位與索引
DROP INDEX IF EXISTS idx_activity_logs_integrity;

ALTER TABLE user_activity_logs DROP COLUMN IF EXISTS integrity_hash;
ALTER TABLE user_activity_logs DROP COLUMN IF EXISTS previous_hash;
```

> **注意**：`user_activity_logs` 為分區表，ALTER TABLE 會套用至所有分區。已寫入的雜湊鏈資料將遺失。

---

## Migration 010: JWT Blacklist

**原始操作**：建立 `jwt_blacklist` 表。

```sql
-- Rollback 010: 移除 JWT 黑名單表
-- ⚠️ 刪除後，已撤銷的 JWT 將無法被偵測，可能造成安全風險
DROP TABLE IF EXISTS jwt_blacklist;
```

> **安全警告**：回退此 migration 後，請確保立即重新部署帶有 JWT 黑名單功能的版本，或強制所有使用者重新登入。

---

## Migration 009: Treatment Drug Options

**原始操作**：建立 `treatment_drug_options` 表與 seed 資料，並清空既有觀察/手術紀錄中的藥物 JSONB。

```sql
-- Rollback 009: 移除藥物選項表
-- ⚠️ 此 migration 曾清空 animal_observations.treatments 與 animal_surgeries 的藥物 JSONB 欄位
-- 該清空操作不可逆，原始用藥資料已永久遺失
DROP TABLE IF EXISTS treatment_drug_options;
```

> **資料遺失警告**：此 migration 的 UP 腳本包含 `UPDATE` 語句清除了 `animal_observations.treatments` 及 `animal_surgeries` 的多個藥物 JSONB 欄位。即使 rollback，這些資料也**無法還原**。

---

## Migration 008: Supplementary

**原始操作**：建立通知路由規則表（`notification_routing`）、電子簽章表（`electronic_signatures`）、記錄附註表（`record_annotations`）及 seed 資料。

```sql
-- Rollback 008: 移除補充模組表
-- 注意順序：record_annotations 引用 electronic_signatures，須先刪

DROP TABLE IF EXISTS record_annotations;
DROP TABLE IF EXISTS electronic_signatures;
DROP TABLE IF EXISTS notification_routing;
```

---

## Migration 007: ERP Warehouse System

**原始操作**：建立 ERP 倉庫完整系統（倉庫、產品、夥伴、單據、庫存、儲位、Views）。

```sql
-- Rollback 007: 移除 ERP 倉庫系統
-- ⚠️ 所有倉庫、產品、庫存、單據資料將永久遺失

-- 先移除 Views
DROP VIEW IF EXISTS v_inventory_summary;
DROP VIEW IF EXISTS v_expiry_alerts;
DROP VIEW IF EXISTS v_low_stock_alerts;
DROP VIEW IF EXISTS v_purchase_order_receipt_status;

-- 依外鍵順序刪除表
DROP TABLE IF EXISTS inventory_snapshots;
DROP TABLE IF EXISTS stock_ledger;
DROP TABLE IF EXISTS document_lines;
DROP TABLE IF EXISTS storage_location_inventory;
DROP TABLE IF EXISTS storage_locations;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS partners;
DROP TABLE IF EXISTS product_uom_conversions;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS sku_sequences;
DROP TABLE IF EXISTS sku_subcategories;
DROP TABLE IF EXISTS sku_categories;
DROP TABLE IF EXISTS product_categories;
DROP TABLE IF EXISTS warehouses;
```

---

## Migration 006: Audit System

**原始操作**：建立稽核系統（使用者活動日誌分區表、登入事件、會話、活動聚合、安全警報、輔助函式）。

```sql
-- Rollback 006: 移除稽核系統
-- ⚠️ 所有稽核日誌、登入事件、安全警報將永久遺失

-- 移除函式
DROP FUNCTION IF EXISTS check_brute_force(VARCHAR);
DROP FUNCTION IF EXISTS log_activity(UUID, VARCHAR, VARCHAR, VARCHAR, UUID, VARCHAR, JSONB, JSONB, INET, TEXT);

-- 移除表（注意 security_alerts 引用 login_events，須先刪）
DROP TABLE IF EXISTS security_alerts;
DROP TABLE IF EXISTS user_activity_aggregates;
DROP TABLE IF EXISTS user_sessions;
DROP TABLE IF EXISTS login_events;

-- 移除分區表（先刪子分區再刪主表）
DROP TABLE IF EXISTS user_activity_logs_2027_q4;
DROP TABLE IF EXISTS user_activity_logs_2027_q3;
DROP TABLE IF EXISTS user_activity_logs_2027_q2;
DROP TABLE IF EXISTS user_activity_logs_2027_q1;
DROP TABLE IF EXISTS user_activity_logs_2026_q4;
DROP TABLE IF EXISTS user_activity_logs_2026_q3;
DROP TABLE IF EXISTS user_activity_logs_2026_q2;
DROP TABLE IF EXISTS user_activity_logs_2026_q1;
DROP TABLE IF EXISTS user_activity_logs;
```

> **注意**：分區表的子分區會在刪除主表時一併刪除（`CASCADE`），但為安全起見上方列出個別刪除。實務上可直接 `DROP TABLE IF EXISTS user_activity_logs CASCADE;`。

---

## Migration 005: HR System

**原始操作**：建立 HR 系統（出勤、加班、年假、補休、請假、Google Calendar 同步、輔助函式）。

```sql
-- Rollback 005: 移除 HR 系統
-- ⚠️ 所有出勤、請假、加班紀錄將永久遺失

-- 移除函式
DROP FUNCTION IF EXISTS get_total_comp_time_hours(UUID);
DROP FUNCTION IF EXISTS get_comp_time_balance(UUID);
DROP FUNCTION IF EXISTS get_annual_leave_balance(UUID);

-- Google Calendar 同步表
DROP TABLE IF EXISTS calendar_sync_history;
DROP TABLE IF EXISTS calendar_sync_conflicts;
DROP TABLE IF EXISTS calendar_event_sync;
DROP TABLE IF EXISTS google_calendar_config;

-- 請假相關（依外鍵順序）
DROP TABLE IF EXISTS leave_balance_usage;
DROP TABLE IF EXISTS leave_approvals;
DROP TABLE IF EXISTS leave_requests;

-- 補休餘額
DROP TABLE IF EXISTS comp_time_balances;

-- 年假額度
DROP TABLE IF EXISTS annual_leave_entitlements;

-- 加班相關
DROP TABLE IF EXISTS overtime_approvals;
DROP TABLE IF EXISTS overtime_records;

-- 出勤
DROP TABLE IF EXISTS attendance_records;
```

---

## Migration 004: AUP System

**原始操作**：建立 AUP（動物使用計畫）系統完整結構。

```sql
-- Rollback 004: 移除 AUP 系統
-- ⚠️ 所有計畫書、審查、修正案、活動歷程資料將永久遺失

-- 活動歷程與審查
DROP TABLE IF EXISTS review_round_history;
DROP TABLE IF EXISTS protocol_activities;
DROP TABLE IF EXISTS vet_review_assignments;

-- 系統設定
DROP TABLE IF EXISTS system_settings;

-- 報表
DROP TABLE IF EXISTS report_history;
DROP TABLE IF EXISTS scheduled_reports;

-- 使用者 AUP Profile
DROP TABLE IF EXISTS user_aup_profiles;

-- 修正案相關
DROP TABLE IF EXISTS amendment_status_history;
DROP TABLE IF EXISTS amendment_versions;
DROP TABLE IF EXISTS amendment_review_assignments;
DROP TABLE IF EXISTS amendments;

-- 審查與版本
DROP TABLE IF EXISTS protocol_attachments;
DROP TABLE IF EXISTS review_comments;
DROP TABLE IF EXISTS review_assignments;
DROP TABLE IF EXISTS protocol_status_history;
DROP TABLE IF EXISTS protocol_versions;
DROP TABLE IF EXISTS user_protocols;
DROP TABLE IF EXISTS protocols;
```

---

## Migration 003: Animal Management System

**原始操作**：建立完整動物管理系統（來源、動物、觀察、手術、體重、疫苗、犧牲、病理、血液檢查、匯入匯出、安樂死、轉讓等）。

```sql
-- Rollback 003: 移除動物管理系統
-- ⚠️ 所有動物、紀錄、血檢、轉讓等資料將永久遺失

-- 轉讓系統
DROP TABLE IF EXISTS transfer_vet_evaluations;
DROP TABLE IF EXISTS animal_transfers;

-- 猝死記錄
DROP TABLE IF EXISTS animal_sudden_deaths;

-- 血液檢查系統
DROP TABLE IF EXISTS blood_test_panel_items;
DROP TABLE IF EXISTS blood_test_panels;
DROP TABLE IF EXISTS animal_blood_test_items;
DROP TABLE IF EXISTS animal_blood_tests;
DROP TABLE IF EXISTS blood_test_templates;

-- 觀察/手術獸醫已讀
DROP TABLE IF EXISTS surgery_vet_reads;
DROP TABLE IF EXISTS observation_vet_reads;

-- 變更原因記錄
DROP TABLE IF EXISTS change_reasons;

-- 匯入匯出
DROP TABLE IF EXISTS animal_import_batches;
DROP TABLE IF EXISTS export_jobs;
DROP TABLE IF EXISTS import_jobs;

-- 紀錄版本歷史
DROP TABLE IF EXISTS record_versions;

-- 照護給藥紀錄
DROP TABLE IF EXISTS care_medication_records;

-- 獸醫師建議
DROP TABLE IF EXISTS vet_recommendations;

-- 紀錄附件
DROP TABLE IF EXISTS animal_record_attachments;

-- 緊急安樂死
DROP TABLE IF EXISTS euthanasia_appeals;
DROP TABLE IF EXISTS euthanasia_orders;

-- 病理組織報告
DROP TABLE IF EXISTS animal_pathology_reports;

-- 犧牲/採樣
DROP TABLE IF EXISTS animal_sacrifices;

-- 疫苗
DROP TABLE IF EXISTS animal_vaccinations;

-- 體重
DROP TABLE IF EXISTS animal_weights;

-- 手術
DROP TABLE IF EXISTS animal_surgeries;

-- 觀察
DROP TABLE IF EXISTS animal_observations;

-- 動物主表
DROP TABLE IF EXISTS animals;

-- 動物來源
DROP TABLE IF EXISTS animal_sources;
```

---

## Migration 002: Core Permissions

**原始操作**：插入所有權限定義（`permissions` 表）與角色預設權限（`role_permissions` 表）。

```sql
-- Rollback 002: 移除權限 seed 資料
-- ⚠️ 這不是 DROP TABLE，而是清除 seed 資料
-- 注意：如果有其他 migration 依賴這些權限資料，需一併處理

-- 清除角色權限關聯
DELETE FROM role_permissions;

-- 清除權限定義
DELETE FROM permissions;
```

> **注意**：002 只做了 `INSERT` 操作（seed 資料），不涉及建表。`roles`、`permissions`、`role_permissions`、`user_roles` 表結構在 001 中建立。回退 002 僅清除權限 seed 資料。

---

## Migration 001: Core Schema

**原始操作**：建立所有 ENUM 類型、核心表（users、roles、permissions、通知、附件、稽核、觸發器、使用者偏好），並插入角色 seed。

```sql
-- Rollback 001: 移除核心 Schema
-- ⚠️ 此操作將徹底清除整個系統的基礎架構
-- ⚠️ 請確認所有其他 migration (002-014) 已先行回退

-- 移除觸發器
DROP TRIGGER IF EXISTS trg_create_notification_settings ON users;
DROP FUNCTION IF EXISTS create_default_notification_settings();

-- 移除表（依外鍵反向順序）
DROP TABLE IF EXISTS user_preferences;
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS notification_settings;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS password_reset_tokens;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS role_permissions;
DROP TABLE IF EXISTS permissions;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS users;

-- 移除所有自訂 ENUM 類型
DROP TYPE IF EXISTS animal_transfer_status;
DROP TYPE IF EXISTS export_format;
DROP TYPE IF EXISTS export_type;
DROP TYPE IF EXISTS import_status;
DROP TYPE IF EXISTS import_type;
DROP TYPE IF EXISTS euthanasia_order_status;
DROP TYPE IF EXISTS amendment_status;
DROP TYPE IF EXISTS amendment_type;
DROP TYPE IF EXISTS leave_status;
DROP TYPE IF EXISTS leave_type;
DROP TYPE IF EXISTS report_type;
DROP TYPE IF EXISTS schedule_type;
DROP TYPE IF EXISTS notification_type;
DROP TYPE IF EXISTS protocol_activity_type;
DROP TYPE IF EXISTS care_record_mode;
DROP TYPE IF EXISTS vet_record_type;
DROP TYPE IF EXISTS version_record_type;
DROP TYPE IF EXISTS animal_file_type;
DROP TYPE IF EXISTS animal_record_type;
DROP TYPE IF EXISTS record_type;
DROP TYPE IF EXISTS animal_gender;
DROP TYPE IF EXISTS animal_breed;
DROP TYPE IF EXISTS animal_status;
DROP TYPE IF EXISTS protocol_status;
DROP TYPE IF EXISTS protocol_role;
DROP TYPE IF EXISTS stock_direction;
DROP TYPE IF EXISTS doc_status;
DROP TYPE IF EXISTS doc_type;
DROP TYPE IF EXISTS customer_category;
DROP TYPE IF EXISTS supplier_category;
DROP TYPE IF EXISTS partner_type;
```

> **嚴重警告**：執行 001 rollback 等同於**清除整個資料庫結構**。此操作不可逆，所有使用者帳號、角色、權限及所有業務資料將永久遺失。

---

## 附錄：完整逆序 Rollback（全部回退）

如需將資料庫恢復到空白狀態，請依序執行以上已文檔化的 rollback SQL（037 → 033 → 022 → 014 → 001）。

```sql
BEGIN;

-- 依序執行每個 migration 的 rollback（037 → 033 → 022 → 014 → 001）
-- ... 貼上上方各段 SQL ...

-- 最後清除 SQLx migration 追蹤表
DROP TABLE IF EXISTS _sqlx_migrations;

COMMIT;
```

---

*文件最後更新：2026-04-24（R26 epic 收尾）*  
*已文檔化 migration 範圍：001–014、022、033–037（015–021 / 023–032 為 TODO）*
