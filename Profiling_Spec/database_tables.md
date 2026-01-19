# iPig System 資料庫結構表格

> **版本**: 2.0 | **更新日期**: 2026-01-19

---

## 目錄

1. [核心認證模組](#1-核心認證模組)
2. [ERP 庫存管理模組](#2-erp-庫存管理模組)
3. [實驗計畫模組](#3-實驗計畫模組)
4. [動物管理模組](#4-動物管理模組)
5. [人資管理模組](#5-人資管理模組)
6. [日曆同步與通知模組](#6-日曆同步與通知模組)
7. [稽核與安全模組](#7-稽核與安全模組)
8. [列舉類型](#8-列舉類型-enums)

---

## 1. 核心認證模組

### 1.1 users (使用者)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| email | VARCHAR(255) | UK, NOT NULL | 電子郵件 |
| password_hash | VARCHAR(255) | NOT NULL | 密碼雜湊 |
| display_name | VARCHAR(100) | NOT NULL | 顯示名稱 |
| phone | VARCHAR(20) | | 電話 |
| organization | VARCHAR(200) | | 組織 |
| is_internal | BOOLEAN | DEFAULT true | 是否內部人員 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |
| must_change_password | BOOLEAN | DEFAULT true | 需變更密碼 |
| last_login_at | TIMESTAMPTZ | | 最後登入時間 |
| login_attempts | INTEGER | DEFAULT 0 | 登入嘗試次數 |
| locked_until | TIMESTAMPTZ | | 鎖定至 |
| theme_preference | VARCHAR(20) | DEFAULT 'light' | 主題偏好 |
| language_preference | VARCHAR(10) | DEFAULT 'zh-TW' | 語言偏好 |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 建立時間 |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | 更新時間 |

---

### 1.2 roles (角色)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| code | VARCHAR(50) | UK, NOT NULL | 角色代碼 |
| name | VARCHAR(100) | NOT NULL | 角色名稱 |
| description | TEXT | | 說明 |
| is_internal | BOOLEAN | DEFAULT true | 內部角色 |
| is_system | BOOLEAN | DEFAULT false | 系統角色 |
| is_deleted | BOOLEAN | DEFAULT false | 已刪除 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 建立時間 |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | 更新時間 |

---

### 1.3 permissions (權限)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| code | VARCHAR(100) | UK, NOT NULL | 權限代碼 |
| name | VARCHAR(200) | NOT NULL | 權限名稱 |
| module | VARCHAR(50) | | 模組 |
| description | TEXT | | 說明 |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 建立時間 |

---

### 1.4 user_roles (使用者角色)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| user_id | UUID | PK, FK → users | 使用者 ID |
| role_id | UUID | PK, FK → roles | 角色 ID |
| assigned_at | TIMESTAMPTZ | | 指派時間 |
| assigned_by | UUID | FK → users | 指派者 |

---

### 1.5 role_permissions (角色權限)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| role_id | UUID | PK, FK → roles | 角色 ID |
| permission_id | UUID | PK, FK → permissions | 權限 ID |

---

### 1.6 refresh_tokens (更新令牌)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users | 使用者 ID |
| token_hash | VARCHAR | | 令牌雜湊 |
| expires_at | TIMESTAMPTZ | | 過期時間 |
| revoked_at | TIMESTAMPTZ | | 撤銷時間 |

---

### 1.7 password_reset_tokens (密碼重設令牌)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users | 使用者 ID |
| token_hash | VARCHAR | | 令牌雜湊 |
| expires_at | TIMESTAMPTZ | | 過期時間 |
| used_at | TIMESTAMPTZ | | 使用時間 |

---

## 2. ERP 庫存管理模組

### 2.1 warehouses (倉庫)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| code | VARCHAR(50) | UK | 倉庫代碼 |
| name | VARCHAR(200) | | 倉庫名稱 |
| address | TEXT | | 地址 |
| is_active | BOOLEAN | | 是否啟用 |

---

### 2.2 sku_categories (SKU 分類)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| code | CHAR(3) | PK | 分類代碼 |
| name | VARCHAR(50) | | 分類名稱 |
| sort_order | INTEGER | | 排序順序 |
| is_active | BOOLEAN | | 是否啟用 |

---

### 2.3 sku_subcategories (SKU 子分類)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| category_code | CHAR(3) | FK → sku_categories | 分類代碼 |
| code | CHAR(3) | | 子分類代碼 |
| name | VARCHAR(50) | | 子分類名稱 |

---

### 2.4 sku_sequences (SKU 序號)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| category_code | CHAR(3) | | 分類代碼 |
| subcategory_code | CHAR(3) | | 子分類代碼 |
| last_sequence | INTEGER | | 最後序號 |

---

### 2.5 products (產品)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| sku | VARCHAR(50) | UK, NOT NULL | SKU 編號 |
| name | VARCHAR(200) | NOT NULL | 產品名稱 |
| spec | TEXT | | 規格 |
| category_code | CHAR(3) | FK → sku_categories | 分類代碼 |
| subcategory_code | CHAR(3) | FK → sku_subcategories | 子分類代碼 |
| base_uom | VARCHAR(20) | DEFAULT 'pcs' | 基本單位 |
| track_batch | BOOLEAN | DEFAULT false | 追蹤批號 |
| track_expiry | BOOLEAN | DEFAULT false | 追蹤效期 |
| safety_stock | NUMERIC(18,4) | | 安全庫存 |
| status | VARCHAR(20) | DEFAULT 'active' | 狀態 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |

---

### 2.6 partners (夥伴/供應商/客戶)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| partner_type | partner_type | NOT NULL | 夥伴類型 |
| code | VARCHAR(50) | UK, NOT NULL | 夥伴代碼 |
| name | VARCHAR(200) | NOT NULL | 夥伴名稱 |
| supplier_category | supplier_category | | 供應商分類 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |

---

### 2.7 documents (單據)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| doc_type | doc_type | NOT NULL | 單據類型 |
| doc_no | VARCHAR(50) | UK, NOT NULL | 單據編號 |
| status | doc_status | DEFAULT 'draft' | 狀態 |
| warehouse_id | UUID | FK → warehouses | 倉庫 ID |
| partner_id | UUID | FK → partners | 夥伴 ID |
| doc_date | DATE | NOT NULL | 單據日期 |
| created_by | UUID | FK → users, NOT NULL | 建立者 |
| approved_by | UUID | FK → users | 核准者 |

---

### 2.8 document_lines (單據明細)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| document_id | UUID | FK → documents, NOT NULL | 單據 ID |
| line_no | INTEGER | NOT NULL | 行號 |
| product_id | UUID | FK → products, NOT NULL | 產品 ID |
| qty | NUMERIC(18,4) | NOT NULL | 數量 |
| uom | VARCHAR(20) | NOT NULL | 單位 |
| unit_price | NUMERIC(18,4) | | 單價 |

---

### 2.9 stock_ledger (庫存分類帳)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| warehouse_id | UUID | FK → warehouses, NOT NULL | 倉庫 ID |
| product_id | UUID | FK → products, NOT NULL | 產品 ID |
| trx_date | TIMESTAMPTZ | NOT NULL | 交易日期 |
| doc_type | doc_type | NOT NULL | 單據類型 |
| doc_id | UUID | NOT NULL | 單據 ID |
| direction | stock_direction | NOT NULL | 異動方向 |
| qty_base | NUMERIC(18,4) | NOT NULL | 基本單位數量 |

---

### 2.10 inventory_snapshots (庫存快照)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| warehouse_id | UUID | PK, FK → warehouses | 倉庫 ID |
| product_id | UUID | PK, FK → products | 產品 ID |
| on_hand_qty_base | NUMERIC(18,4) | DEFAULT 0 | 在手數量 |

---

## 3. 實驗計畫模組

### 3.1 protocols (實驗計畫)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| protocol_no | VARCHAR(50) | UK, NOT NULL | 計畫編號 |
| iacuc_no | VARCHAR(50) | UK | IACUC 編號 |
| title | VARCHAR(500) | NOT NULL | 計畫標題 |
| status | protocol_status | DEFAULT 'DRAFT' | 狀態 |
| pi_user_id | UUID | FK → users, NOT NULL | 主持人 ID |
| working_content | JSONB | | 工作內容 |
| start_date | DATE | | 開始日期 |
| end_date | DATE | | 結束日期 |

---

### 3.2 user_protocols (使用者計畫關聯)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| user_id | UUID | PK, FK → users | 使用者 ID |
| protocol_id | UUID | PK, FK → protocols | 計畫 ID |
| role_in_protocol | protocol_role | NOT NULL | 計畫角色 |

---

### 3.3 protocol_versions (計畫版本)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| protocol_id | UUID | FK → protocols, NOT NULL | 計畫 ID |
| version_no | INTEGER | NOT NULL | 版本號 |
| content_snapshot | JSONB | NOT NULL | 內容快照 |
| submitted_at | TIMESTAMPTZ | DEFAULT NOW() | 提交時間 |

---

### 3.4 protocol_status_history (計畫狀態歷程)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| protocol_id | UUID | FK → protocols, NOT NULL | 計畫 ID |
| from_status | protocol_status | | 原狀態 |
| to_status | protocol_status | NOT NULL | 新狀態 |
| changed_by | UUID | FK → users, NOT NULL | 變更者 |
| remark | TEXT | | 備註 |

---

### 3.5 review_assignments (審查指派)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| protocol_id | UUID | FK → protocols | 計畫 ID |
| reviewer_id | UUID | FK → users | 審查員 ID |
| assigned_by | UUID | FK → users | 指派者 |

---

### 3.6 review_comments (審查意見)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| protocol_version_id | UUID | FK → protocol_versions | 版本 ID |
| reviewer_id | UUID | FK → users | 審查員 ID |
| content | TEXT | | 意見內容 |
| is_resolved | BOOLEAN | | 已解決 |

---

## 4. 動物管理模組

### 4.1 pig_sources (豬隻來源)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| code | VARCHAR(20) | UK, NOT NULL | 來源代碼 |
| name | VARCHAR(100) | NOT NULL | 來源名稱 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |

---

### 4.2 pigs (豬隻)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| ear_tag | VARCHAR(10) | NOT NULL | 耳標 |
| status | pig_status | DEFAULT 'unassigned' | 狀態 |
| breed | pig_breed | NOT NULL | 品種 |
| source_id | UUID | FK → pig_sources | 來源 ID |
| gender | pig_gender | NOT NULL | 性別 |
| birth_date | DATE | | 出生日期 |
| entry_date | DATE | NOT NULL | 入場日期 |
| entry_weight | NUMERIC(5,1) | | 入場體重 |
| pen_location | VARCHAR(10) | | 欄位位置 |
| pre_experiment_code | VARCHAR(20) | | 預試驗代碼 |
| iacuc_no | VARCHAR(20) | | IACUC 編號 |
| experiment_date | DATE | | 實驗日期 |
| is_deleted | BOOLEAN | DEFAULT false | 已刪除 |
| deleted_at | TIMESTAMPTZ | | 刪除時間 |
| deleted_by | UUID | FK → users | 刪除者 |

---

### 4.3 pig_observations (豬隻觀察紀錄)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs, NOT NULL | 豬隻 ID |
| event_date | DATE | NOT NULL | 事件日期 |
| record_type | record_type | NOT NULL | 紀錄類型 |
| content | TEXT | NOT NULL | 內容 |
| no_medication_needed | BOOLEAN | DEFAULT false | 無需用藥 |
| stop_medication | BOOLEAN | DEFAULT false | 停止用藥 |
| treatments | JSONB | | 治療處置 |
| vet_read | BOOLEAN | DEFAULT false | 獸醫已讀 |

---

### 4.4 pig_surgeries (豬隻手術紀錄)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs, NOT NULL | 豬隻 ID |
| is_first_experiment | BOOLEAN | DEFAULT true | 首次實驗 |
| surgery_date | DATE | NOT NULL | 手術日期 |
| surgery_site | VARCHAR(200) | NOT NULL | 手術部位 |
| induction_anesthesia | JSONB | | 誘導麻醉 |
| anesthesia_maintenance | JSONB | | 麻醉維持 |
| vital_signs | JSONB | | 生命徵象 |
| vet_read | BOOLEAN | DEFAULT false | 獸醫已讀 |

---

### 4.5 pig_weights (豬隻體重)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs | 豬隻 ID |
| measure_date | DATE | | 測量日期 |
| weight | NUMERIC(5,1) | | 體重 (kg) |

---

### 4.6 pig_vaccinations (豬隻疫苗)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs | 豬隻 ID |
| administered_date | DATE | | 施打日期 |
| vaccine | VARCHAR(100) | | 疫苗名稱 |

---

### 4.7 pig_sacrifices (豬隻犧牲)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs, UK | 豬隻 ID |
| sacrifice_date | DATE | | 犧牲日期 |
| confirmed_sacrifice | BOOLEAN | | 已確認犧牲 |

---

### 4.8 pig_pathology_reports (豬隻病理報告)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| pig_id | INTEGER | FK → pigs, UK | 豬隻 ID |

---

### 4.9 pig_record_attachments (豬隻紀錄附件)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| record_type | pig_record_type | | 紀錄類型 |
| record_id | INTEGER | | 紀錄 ID |
| file_path | VARCHAR | | 檔案路徑 |

---

### 4.10 vet_recommendations (獸醫建議)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| record_type | vet_record_type | | 紀錄類型 |
| record_id | INTEGER | | 紀錄 ID |
| content | TEXT | | 建議內容 |

---

### 4.11 record_versions (紀錄版本)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | SERIAL | PK | 主鍵 |
| record_type | version_record_type | | 紀錄類型 |
| record_id | INTEGER | | 紀錄 ID |
| version_no | INTEGER | | 版本號 |
| snapshot | JSONB | | 快照 |

---

## 5. 人資管理模組

### 5.1 attendance_records (出勤紀錄)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| work_date | DATE | NOT NULL | 工作日期 |
| clock_in_time | TIMESTAMPTZ | | 上班打卡 |
| clock_out_time | TIMESTAMPTZ | | 下班打卡 |
| regular_hours | NUMERIC(5,2) | DEFAULT 0 | 正常工時 |
| overtime_hours | NUMERIC(5,2) | DEFAULT 0 | 加班工時 |
| status | VARCHAR(20) | DEFAULT 'normal' | 狀態 |

> **唯一約束**: (user_id, work_date)

---

### 5.2 overtime_records (加班紀錄)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| overtime_date | DATE | NOT NULL | 加班日期 |
| start_time | TIMESTAMPTZ | NOT NULL | 開始時間 |
| end_time | TIMESTAMPTZ | NOT NULL | 結束時間 |
| hours | NUMERIC(5,2) | NOT NULL | 時數 |
| overtime_type | VARCHAR(20) | NOT NULL | 類型 (weekday/weekend/holiday) |
| multiplier | NUMERIC(3,2) | DEFAULT 1.0 | 倍率 |
| comp_time_hours | NUMERIC(5,2) | NOT NULL | 補休時數 |
| comp_time_expires_at | DATE | NOT NULL | 補休到期日 |
| status | VARCHAR(20) | DEFAULT 'draft' | 狀態 |

---

### 5.3 annual_leave_entitlements (年假額度)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| entitlement_year | INTEGER | NOT NULL | 年度 |
| entitled_days | NUMERIC(5,2) | NOT NULL | 應得天數 |
| used_days | NUMERIC(5,2) | DEFAULT 0 | 已用天數 |
| expires_at | DATE | NOT NULL | 到期日 |
| is_expired | BOOLEAN | DEFAULT false | 已過期 |

> **唯一約束**: (user_id, entitlement_year)

---

### 5.4 comp_time_balances (補休餘額)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| overtime_record_id | UUID | FK → overtime_records, NOT NULL | 加班紀錄 ID |
| original_hours | NUMERIC(5,2) | NOT NULL | 原始時數 |
| used_hours | NUMERIC(5,2) | DEFAULT 0 | 已用時數 |
| earned_date | DATE | NOT NULL | 取得日期 |
| expires_at | DATE | NOT NULL | 到期日 |
| is_expired | BOOLEAN | DEFAULT false | 已過期 |

---

### 5.5 leave_requests (請假單)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 申請人 ID |
| proxy_user_id | UUID | FK → users | 代理人 ID |
| leave_type | leave_type | NOT NULL | 假別 |
| start_date | DATE | NOT NULL | 開始日期 |
| end_date | DATE | NOT NULL | 結束日期 |
| total_days | NUMERIC(5,2) | NOT NULL | 總天數 |
| reason | TEXT | | 原因 |
| status | leave_status | DEFAULT 'DRAFT' | 狀態 |
| current_approver_id | UUID | FK → users | 目前簽核人 |

---

### 5.6 leave_approvals (請假簽核)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| leave_request_id | UUID | FK → leave_requests, NOT NULL | 請假單 ID |
| approver_id | UUID | FK → users, NOT NULL | 簽核人 ID |
| approval_level | VARCHAR(20) | NOT NULL | 簽核層級 |
| action | VARCHAR(20) | NOT NULL | 動作 (APPROVE/REJECT) |

---

## 6. 日曆同步與通知模組

### 6.1 google_calendar_config (Google 日曆設定)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| calendar_id | VARCHAR(255) | NOT NULL | 日曆 ID |
| calendar_name | VARCHAR(100) | | 日曆名稱 |
| is_configured | BOOLEAN | DEFAULT false | 已設定 |
| sync_enabled | BOOLEAN | DEFAULT true | 同步啟用 |
| last_sync_at | TIMESTAMPTZ | | 最後同步時間 |
| last_sync_status | VARCHAR(20) | | 最後同步狀態 |

---

### 6.2 calendar_event_sync (日曆事件同步)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| leave_request_id | UUID | FK → leave_requests, UK | 請假單 ID |
| google_event_id | VARCHAR(255) | | Google 事件 ID |
| sync_version | INTEGER | DEFAULT 0 | 同步版本 |
| sync_status | VARCHAR(20) | DEFAULT 'pending_create' | 同步狀態 |

---

### 6.3 calendar_sync_conflicts (日曆同步衝突)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| leave_request_id | UUID | FK → leave_requests | 請假單 ID |
| conflict_type | VARCHAR(50) | NOT NULL | 衝突類型 |
| ipig_data | JSONB | NOT NULL | iPig 資料 |
| google_data | JSONB | | Google 資料 |
| status | VARCHAR(20) | DEFAULT 'pending' | 狀態 |

---

### 6.4 calendar_sync_history (日曆同步歷程)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| job_type | VARCHAR(20) | NOT NULL | 工作類型 |
| started_at | TIMESTAMPTZ | DEFAULT NOW() | 開始時間 |
| status | VARCHAR(20) | DEFAULT 'running' | 狀態 |
| events_created | INTEGER | DEFAULT 0 | 建立事件數 |

---

### 6.5 notifications (通知)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| type | notification_type | NOT NULL | 通知類型 |
| title | VARCHAR(200) | NOT NULL | 標題 |
| content | TEXT | | 內容 |
| is_read | BOOLEAN | DEFAULT false | 已讀 |
| related_entity_type | VARCHAR(50) | | 關聯實體類型 |
| related_entity_id | UUID | | 關聯實體 ID |

---

### 6.6 notification_settings (通知設定)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| user_id | UUID | PK, FK → users | 使用者 ID |
| email_low_stock | BOOLEAN | DEFAULT true | 低庫存郵件 |
| email_expiry_warning | BOOLEAN | DEFAULT true | 效期警告郵件 |
| email_document_approval | BOOLEAN | DEFAULT true | 單據核准郵件 |
| email_protocol_status | BOOLEAN | DEFAULT true | 計畫狀態郵件 |
| expiry_warning_days | INTEGER | DEFAULT 30 | 效期警告天數 |

---

### 6.7 scheduled_reports (排程報表)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| name | VARCHAR(100) | NOT NULL | 報表名稱 |
| report_type | report_type | NOT NULL | 報表類型 |
| schedule_type | schedule_type | NOT NULL | 排程類型 |
| schedule_config | JSONB | | 排程設定 |
| recipients | TEXT[] | | 收件人 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 |

---

## 7. 稽核與安全模組

### 7.1 user_activity_logs (使用者活動日誌)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| actor_user_id | UUID | FK → users | 操作者 ID |
| actor_email | VARCHAR(255) | | 操作者郵件 |
| event_category | VARCHAR(50) | NOT NULL | 事件分類 |
| event_type | VARCHAR(100) | NOT NULL | 事件類型 |
| entity_type | VARCHAR(50) | | 實體類型 |
| entity_id | UUID | | 實體 ID |
| before_data | JSONB | | 變更前資料 |
| after_data | JSONB | | 變更後資料 |
| changed_fields | TEXT[] | | 變更欄位 |
| ip_address | INET | | IP 位址 |
| partition_date | DATE | NOT NULL | 分區日期 |

> **注意**: 此表按季度分區

---

### 7.2 login_events (登入事件)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users | 使用者 ID |
| email | VARCHAR(255) | NOT NULL | 電子郵件 |
| event_type | VARCHAR(20) | NOT NULL | 事件類型 |
| ip_address | INET | | IP 位址 |
| is_unusual_time | BOOLEAN | DEFAULT false | 異常時間 |
| is_unusual_location | BOOLEAN | DEFAULT false | 異常位置 |
| is_new_device | BOOLEAN | DEFAULT false | 新裝置 |
| failure_reason | VARCHAR(100) | | 失敗原因 |

---

### 7.3 user_sessions (使用者工作階段)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| user_id | UUID | FK → users, NOT NULL | 使用者 ID |
| started_at | TIMESTAMPTZ | DEFAULT NOW() | 開始時間 |
| ended_at | TIMESTAMPTZ | | 結束時間 |
| last_activity_at | TIMESTAMPTZ | DEFAULT NOW() | 最後活動 |
| is_active | BOOLEAN | DEFAULT true | 是否活躍 |
| ended_reason | VARCHAR(50) | | 結束原因 |

---

### 7.4 security_alerts (安全警報)

| 欄位名稱 | 資料類型 | 約束 | 說明 |
|----------|----------|------|------|
| id | UUID | PK | 主鍵 |
| alert_type | VARCHAR(50) | NOT NULL | 警報類型 |
| severity | VARCHAR(20) | DEFAULT 'warning' | 嚴重程度 |
| title | VARCHAR(255) | NOT NULL | 標題 |
| user_id | UUID | FK → users | 相關使用者 |
| status | VARCHAR(20) | DEFAULT 'open' | 狀態 |

---

## 8. 列舉類型 (Enums)

### 8.1 夥伴相關

| 列舉名稱 | 值 |
|----------|-----|
| partner_type | `supplier`, `customer` |
| supplier_category | `drug`, `consumable`, `feed`, `equipment`, `other` |

### 8.2 單據相關

| 列舉名稱 | 值 |
|----------|-----|
| doc_type | `PO` (採購單), `GRN` (進貨單), `PR` (採購申請), `SO` (銷貨單), `DO` (出貨單), `SR` (銷貨退回), `TR` (調撥單), `STK` (盤點單), `ADJ` (調整單), `RTN` (退貨單) |
| doc_status | `draft`, `submitted`, `approved`, `cancelled` |
| stock_direction | `in`, `out`, `transfer_in`, `transfer_out`, `adjust_in`, `adjust_out` |

### 8.3 實驗計畫相關

| 列舉名稱 | 值 |
|----------|-----|
| protocol_role | `PI` (主持人), `CLIENT` (委託者), `CO_EDITOR` (共同編輯) |
| protocol_status | `DRAFT`, `SUBMITTED`, `PRE_REVIEW`, `UNDER_REVIEW`, `REVISION_REQUIRED`, `RESUBMITTED`, `APPROVED`, `APPROVED_WITH_CONDITIONS`, `DEFERRED`, `REJECTED`, `SUSPENDED`, `CLOSED`, `DELETED` |

### 8.4 動物相關

| 列舉名稱 | 值 |
|----------|-----|
| pig_status | `unassigned` (未指派), `assigned` (已指派), `in_experiment` (實驗中), `completed` (完成), `transferred` (轉出), `deceased` (死亡) |
| pig_breed | `miniature` (迷你豬), `white` (白豬), `LYD`, `other` |
| pig_gender | `male`, `female` |
| record_type | `abnormal` (異常), `experiment` (實驗), `observation` (觀察) |
| pig_record_type | `observation`, `surgery`, `sacrifice`, `pathology` |
| vet_record_type | `observation`, `surgery` |

### 8.5 人資相關

| 列舉名稱 | 值 |
|----------|-----|
| leave_type | `ANNUAL` (特休), `PERSONAL` (事假), `SICK` (病假), `COMPENSATORY` (補休), `MARRIAGE` (婚假), `BEREAVEMENT` (喪假), `MATERNITY` (產假), `PATERNITY` (陪產假), `MENSTRUAL` (生理假), `OFFICIAL` (公假), `UNPAID` (無薪假) |
| leave_status | `DRAFT`, `PENDING_L1`, `PENDING_L2`, `PENDING_HR`, `PENDING_GM`, `APPROVED`, `REJECTED`, `CANCELLED`, `REVOKED` |

### 8.6 通知相關

| 列舉名稱 | 值 |
|----------|-----|
| notification_type | `low_stock`, `expiry_warning`, `document_approval`, `protocol_status`, `vet_recommendation`, `system_alert`, `monthly_report` |
| schedule_type | `daily`, `weekly`, `monthly` |
| report_type | `stock_on_hand`, `stock_ledger`, `purchase_summary`, `cost_summary`, `expiry_report`, `low_stock_report` |

---

## 統計摘要

| 模組 | 資料表數量 |
|------|-----------|
| 核心認證 | 7 |
| ERP 庫存 | 10 |
| 實驗計畫 | 6 |
| 動物管理 | 11 |
| 人資管理 | 6 |
| 日曆通知 | 7 |
| 稽核安全 | 4 |
| **總計** | **51** |
