# 一鍵全庫輸出規劃（通用格式）

> **建立日期：** 2026-03-01  
> **目的：** 一鍵輸出整個資料庫內容為通用格式，可在不同 migrations（不同 schema 版本）環境下讀取同一筆資料。

---

## 一、需求摘要

| 項目 | 說明 |
|------|------|
| **一鍵輸出** | 單一 API / 按鈕即可觸發 |
| **全庫涵蓋** | AUP、動物資訊與歷史、倉庫進銷存、使用者、員工教育訓練等系統會用到的全部資料 |
| **通用格式** | 與 PostgreSQL schema 解耦，可被不同 migration 版本的匯入器讀取 |
| **跨 migration 相容** | 匯出於 migration N，可在 migration M（M ≠ N）匯入時正確映射欄位 |

---

## 二、匯出範圍（依 migrations 對應表）

### 2.1 完整資料表清單

依 migrations 001–010 整理，共約 80+ 張表：

| Migration | 模組 | 資料表 |
|-----------|------|--------|
| **001** | 核心 | `users`, `roles`, `permissions`, `role_permissions`, `user_roles`, `refresh_tokens`, `password_reset_tokens`, `notifications`, `notification_settings`, `attachments`, `audit_logs`, `user_preferences` |
| **002** | 動物 | `animal_sources`, `animals`, `animal_observations`, `animal_surgeries`, `animal_weights`, `animal_vaccinations`, `animal_sacrifices`, `animal_pathology_reports`, `animal_record_attachments`, `vet_recommendations`, `care_medication_records`, `record_versions`, `import_jobs`, `export_jobs`, `euthanasia_orders`, `euthanasia_appeals`, `animal_import_batches`, `change_reasons`, `observation_vet_reads`, `surgery_vet_reads`, `blood_test_templates`, `animal_blood_tests`, `animal_blood_test_items`, `blood_test_panels`, `blood_test_panel_items`, `animal_sudden_deaths`, `animal_transfers`, `transfer_vet_evaluations` |
| **003** | AUP | `protocols`, `user_protocols`, `protocol_versions`, `protocol_status_history`, `review_assignments`, `review_comments`, `protocol_attachments`, `amendments`, `amendment_review_assignments`, `amendment_versions`, `amendment_status_history`, `user_aup_profiles`, `scheduled_reports`, `report_history`, `system_settings`, `vet_review_assignments`, `protocol_activities`, `review_round_history` |
| **004** | HR | `attendance_records`, `overtime_records`, `overtime_approvals`, `annual_leave_entitlements`, `comp_time_balances`, `leave_requests`, `leave_approvals`, `leave_balance_usage`, `google_calendar_config`, `calendar_event_sync`, `calendar_sync_conflicts`, `calendar_sync_history` |
| **005** | 稽核 | `user_activity_logs` (含 partitions), `login_events`, `user_sessions`, `user_activity_aggregates`, `security_alerts` |
| **006** | ERP | `warehouses`, `product_categories`, `sku_categories`, `sku_subcategories`, `sku_sequences`, `products`, `product_uom_conversions`, `partners`, `storage_locations`, `documents`, `document_lines`, `storage_location_inventory`, `stock_ledger`, `inventory_snapshots` |
| **007** | 補充 | `notification_routing`, `electronic_signatures`, `record_annotations`, `treatment_drug_options`, `jwt_blacklist` |
| **010** | GLP/QAU | `training_records`, `equipment`, `equipment_calibrations`, `chart_of_accounts`, `journal_entries`, `journal_entry_lines`, `ap_payments`, `ar_receipts` |

**排除**：
- `jwt_blacklist`、`refresh_tokens`、`password_reset_tokens`：session / token 類，不適合作為「資料」攜帶
- 可選：`user_activity_logs`、`login_events` 等大批量稽核檔，提供「含稽核 / 不含稽核」選項

---

## 三、通用格式設計

### 3.1 為什麼需要通用格式？

- `pg_dump` 輸出為 PostgreSQL 專用 SQL，不同 schema 版本匯入時易有衝突
- 目標：輸出為 **與 DB 實作無關的結構化資料**，由匯入端依「schema 版本」決定如何映射

### 3.2 格式：iPig Data Exchange Format (IDXF)

採用 **JSON** 作為主要載體，結構如下：

```json
{
  "meta": {
    "format": "ipig-idxf",
    "format_version": "1.0",
    "schema_version": "010",
    "exported_at": "2026-03-01T10:00:00Z",
    "source": "ipig_system",
    "migration_applied": "010"
  },
  "tables": [
    {
      "name": "users",
      "columns": ["id", "email", "display_name", "..."],
      "rows": [
        { "id": "...", "email": "...", "display_name": "...", "..." },
        ...
      ]
    },
    {
      "name": "animals",
      "columns": ["id", "ear_tag", "status", "..."],
      "rows": [...]
    },
    ...
  ]
}
```

### 3.3 設計原則

| 原則 | 說明 |
|------|------|
| **自描述** | 每張表附 `columns`，明確欄位順序與名稱 |
| **schema_version** | 標註匯出當下的 migration 版本，供匯入器選擇對應 mapper |
| **逐表輸出** | 以 table 為單位，便於匯入時逐表映射、跳過不支援表 |
| **型別保留** | 值以 JSON 原生型別表示（string, number, boolean, null），timestamp 用 ISO 8601 |
| **UUID 字串** | 一律輸出為字串，避免數值精度問題 |
| **不包含 DDL** | 僅資料，不含 CREATE TABLE / ENUM，schema 由目標 DB migration 決定 |

### 3.4 大型資料與 streaming

- 若資料量大，可採 **JSON Lines (NDJSON)** 或 **分檔**（每表一檔）：
  - `manifest.json`：meta + 各表檔名
  - `tables/users.jsonl`、`tables/animals.jsonl` 等

---

## 四、匯出順序（FK 依賴）

匯出時依 **依賴順序** 排列，確保匯入時可依序插入：

1. 無 FK：`roles`, `permissions`, `animal_sources`, `product_categories`, `sku_*`, `warehouses`, `partners`, `storage_locations`, `blood_test_templates`, `blood_test_panels`, `change_reasons`, `system_settings`, `chart_of_accounts` 等
2. 依賴 users：`users` → `user_roles`, `role_permissions`, `user_preferences`, `protocols`, ...
3. 依賴其他表：`animals` → `animal_observations`, `animal_surgeries`, ...；`protocols` → `protocol_versions`, `review_assignments`, ...
4. 依賴多表：`document_lines`（依賴 documents, products），`stock_ledger` 等

實作時可透過 `information_schema` 或靜態依賴表取得正確順序。

---

## 五、跨 Migration 讀取策略

### 5.1 版本對應表（Column Mapper）

匯入端維護 `schema_mappings`：

```json
{
  "010": {
    "users": { "2fa_secret": "optional" },
    "animals": { "new_field": "default_value" }
  }
}
```

- **新欄位**：匯出檔無該欄 → 匯入時填入預設值或 NULL
- **移除欄位**：匯出檔有、目標 schema 無 → 匯入時略過
- **改名**：透過 alias 映射（例：`old_name` → `new_name`）

### 5.2 匯入流程（概念）

```
1. 讀取 manifest / meta.schema_version
2. 載入對應 schema_version 的 column mapper
3. 對每張 table：
   a. 若目標 DB 無此表 → 略過（或紀錄到 skip 報告）
   b. 遍歷 rows，依 mapper 轉成目標 schema 的 row
   c. INSERT（可批次、可處理 conflicts）
4. 回報成功 / 略過 / 錯誤
```

### 5.3 ENUM 與型別

- ENUM 以 **字串** 輸出，匯入時目標 DB 需已執行 migration 建立對應 ENUM
- 若目標 migration 新增 ENUM 值：舊匯出檔的值仍為有效字串即可

---

## 六、API 與權限設計

### 6.1 匯出 API

| 項目 | 說明 |
|------|------|
| **路徑** | `GET /api/v1/admin/data-export` 或 `POST /api/v1/admin/data-export` |
| **權限** | `admin.data.export`（僅超級管理員或指定角色） |
| **參數** | `include_audit=false`（預設不含大量稽核）, `format=json|jsonl` |
| **回應** | `application/json` 或 `application/zip`（若多檔打包） |
| **檔名** | `ipig_export_{YYYYMMDD_HHMMSS}.json` |

### 6.2 匯入 API（Phase 2）

| 項目 | 說明 |
|------|------|
| **路徑** | `POST /api/v1/admin/data-import` |
| **權限** | `admin.data.import` |
| **Body** | multipart JSON 或上傳 zip |
| **模式** | `merge`（有則更新） / `replace`（清空後插入） / `append`（僅新增） |

### 6.3 稽核

- 匯出 / 匯入均寫入 `audit_logs`：操作者、時間、範圍、成功/失敗

---

## 七、實作階段建議

| 階段 | 項目 | 說明 |
|------|------|------|
| **Phase 1** | 匯出 API | 一鍵匯出全庫為單一 JSON（或 zip 分包），含 meta + tables |
| **Phase 1** | 匯出順序 | 靜態定義 table 依賴順序，依序 SELECT 輸出 |
| **Phase 1** | 前端按鈕 | Admin 設定頁或獨立「資料匯出」頁，一鍵下載 |
| **Phase 2** | schema_version | 從 migration 版本或設定檔讀取，寫入 meta |
| **Phase 2** | 匯入 API | 解析 IDXF，依 schema_version 映射後 INSERT |
| **Phase 3** | Column mapper | 維護 001–010 各版對新版的映射表 | ✅ |
| **Phase 3** | Streaming | 大表改為 NDJSON 或分檔，避免記憶體爆滿 | ✅ |

**Phase 3 實作摘要：**
- `schema_mapping.rs`：`transform_row(source_version, table, row)` 欄位改名映射
- 匯出 `format=zip`：manifest.json + tables/*.json 或 *.jsonl（>10k 行用 NDJSON）
- 匯入自動偵測 Zip（magic bytes PK），支援 NDJSON 逐行解析

---

## 八、技術注意事項

### 8.1 敏感性資料

- `password_hash`、`2fa_secret` 等敏感欄位可選「脫敏匯出」（僅管理員選擇「含密碼」時才包含）
- 匯出檔應加密傳輸、限制存取，符合 `DATA_RETENTION_POLICY`

### 8.2 效能

- 全庫可能數 GB，需考慮：
  - Streaming 回應（chunked transfer）
  - 非同步 job：觸發後寫入檔庫，完成後提供下載連結
  - 背景 worker 處理，避免阻塞 API

### 8.3 檔案附件

- `attachments` 表存 metadata，實體檔案在 `uploads/`
- 全庫匯出可採兩種模式：
  - **僅 metadata**：只輸出 DB，不含實體檔
  - **含檔案**：輸出 JSON + 打包 `uploads/` 為 zip（體積大）

---

## 九、與既有備份的關係

| 用途 | 工具 | 格式 |
|------|------|------|
| 災難復原、同 schema 還原 | `pg_dump` / `pg_restore` | PostgreSQL custom / SQL |
| 跨 migration 攜帶、應用層讀取、稽核封存 | 本規劃之全庫匯出 | IDXF (JSON) |

兩者互補：pg_dump 做快速還原，IDXF 做可攜、可讀的通用格式。
