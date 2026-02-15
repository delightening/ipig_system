# 資料庫綱要

> **版本**：4.0  
> **最後更新**：2026-02-16  
> **對象**：資料庫管理員、開發人員

---

## 1. 概覽

iPig 資料庫執行於 **PostgreSQL 16**，由 10 個遷移檔案按模組組織：

| 遷移 | 說明 | 主要資料表 |
|------|------|-----------| 
| 001 | 核心架構 | users, roles, permissions, notifications, audit_logs, attachments |
| 002 | 核心權限 | role_permissions 種子資料 |
| 003 | 動物管理 | animals, animal_observations, animal_surgeries, animal_weights, animal_vaccinations, animal_sources |
| 004 | AUP 系統 | protocols, protocol_versions, user_protocols, review_assignments, review_comments |
| 005 | 人事系統 | attendance_records, leave_requests, overtime_records, leave_balances, departments |
| 006 | 稽核系統 | user_activity_logs (分割表), login_events, user_sessions, security_alerts |
| 007 | ERP/倉庫 | products, warehouses, partners, documents, document_lines, stock_ledger, skus |
| 008 | AUP 審查增強 | scheduled_reports, report_history, review_assignments_v2 |
| 009 | 血液檢查 | blood_test_templates, animal_blood_tests, animal_blood_test_items, blood_test_panels |
| 010 | 使用者偏好 | user_preferences |

---

## 2. 自訂類型 (ENUMs)

### 2.1 ERP 類型

```sql
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE customer_category AS ENUM ('internal', 'external', 'research', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');
```

### 2.2 AUP/計畫書類型

```sql
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');
CREATE TYPE protocol_status AS ENUM (
    'DRAFT', 'SUBMITTED', 'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW', 'VET_REVISION_REQUIRED', 'UNDER_REVIEW', 'REVISION_REQUIRED',
    'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS',
    'DEFERRED', 'REJECTED', 'SUSPENDED', 'CLOSED', 'DELETED'
);
CREATE TYPE protocol_activity_type AS ENUM (
    'CREATED', 'UPDATED', 'SUBMITTED', 'RESUBMITTED', 'APPROVED',
    'APPROVED_WITH_CONDITIONS', 'CLOSED', 'REJECTED', 'SUSPENDED', 'DELETED',
    'STATUS_CHANGED', 'REVIEWER_ASSIGNED', 'VET_ASSIGNED',
    'COEDITOR_ASSIGNED', 'COEDITOR_REMOVED',
    'COMMENT_ADDED', 'COMMENT_REPLIED', 'COMMENT_RESOLVED',
    'ATTACHMENT_UPLOADED', 'ATTACHMENT_DELETED',
    'VERSION_CREATED', 'VERSION_RECOVERED',
    'AMENDMENT_CREATED', 'AMENDMENT_SUBMITTED',
    'ANIMAL_ASSIGNED', 'ANIMAL_UNASSIGNED'
);
```

### 2.3 動物管理類型

```sql
CREATE TYPE animal_status AS ENUM ('unassigned', 'in_experiment', 'completed');
CREATE TYPE animal_breed AS ENUM ('miniature', 'white', 'LYD', 'other');
CREATE TYPE animal_gender AS ENUM ('male', 'female');
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');
CREATE TYPE animal_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology', 'blood_test');
CREATE TYPE animal_file_type AS ENUM ('photo', 'attachment', 'report');
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');
CREATE TYPE version_record_type AS ENUM (
    'observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology', 'blood_test'
);
```

### 2.4 變更申請/安樂死類型

```sql
CREATE TYPE amendment_type AS ENUM ('MAJOR', 'MINOR', 'PENDING');
CREATE TYPE amendment_status AS ENUM (
    'DRAFT', 'SUBMITTED', 'CLASSIFIED', 'UNDER_REVIEW',
    'REVISION_REQUIRED', 'RESUBMITTED', 'APPROVED', 'REJECTED', 'ADMIN_APPROVED'
);
CREATE TYPE euthanasia_order_status AS ENUM (
    'pending_pi', 'appealed', 'chair_arbitration',
    'approved', 'rejected', 'executed', 'cancelled'
);
```

### 2.5 HR 類型

```sql
CREATE TYPE leave_type AS ENUM (
    'ANNUAL', 'PERSONAL', 'SICK', 'COMPENSATORY', 'MARRIAGE',
    'BEREAVEMENT', 'MATERNITY', 'PATERNITY', 'MENSTRUAL', 'OFFICIAL'
);
CREATE TYPE leave_status AS ENUM (
    'DRAFT', 'PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM',
    'APPROVED', 'REJECTED', 'CANCELLED', 'REVOKED'
);
```

### 2.6 通知/報表/匯入匯出類型

```sql
CREATE TYPE notification_type AS ENUM (
    'low_stock', 'expiry_warning', 'document_approval', 'protocol_status',
    'protocol_submitted', 'review_assignment', 'review_comment',
    'leave_approval', 'overtime_approval', 'vet_recommendation',
    'system_alert', 'monthly_report'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM (
    'stock_on_hand', 'stock_ledger', 'purchase_summary',
    'cost_summary', 'expiry_report', 'low_stock_report'
);
CREATE TYPE import_type AS ENUM ('animal_basic', 'animal_weight');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
CREATE TYPE export_format AS ENUM ('pdf', 'excel');
```

---

## 3. 核心架構 (Migration 001)

### 3.1 使用者表 (users)

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    organization VARCHAR(200),
    is_internal BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    must_change_password BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    theme_preference VARCHAR(20) NOT NULL DEFAULT 'light',
    language_preference VARCHAR(10) NOT NULL DEFAULT 'zh-TW',
    -- 員工相關（AUP Section 8 經驗）
    entry_date DATE,
    position VARCHAR(100),
    aup_roles VARCHAR(255)[] DEFAULT '{}',
    years_experience INTEGER NOT NULL DEFAULT 0,
    trainings JSONB NOT NULL DEFAULT '[]',
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 3.2 角色與權限

| 表名 | PK | 主要欄位 | 說明 |
|------|-----|---------|------|
| roles | UUID | code, name, is_system, is_internal | 角色定義 |
| permissions | UUID | code, name, module | 權限定義 |
| role_permissions | (role_id, permission_id) | — | 角色權限關聯 |
| user_roles | (user_id, role_id) | assigned_at, assigned_by | 使用者角色 |
| refresh_tokens | UUID | user_id, token_hash, expires_at | JWT 更新令牌 |
| password_reset_tokens | UUID | user_id, token_hash, expires_at | 密碼重設令牌 |

### 3.3 通知與附件

| 表名 | PK | 說明 |
|------|-----|------|
| notifications | UUID | 站內通知 |
| notification_settings | user_id | 每用戶通知偏好 |
| attachments | UUID | 通用附件（category 分類）|
| audit_logs | UUID | 簡易稽核日誌 |

### 3.4 預設角色

系統預設 11 個角色：`admin`、`ADMIN_STAFF`、`WAREHOUSE_MANAGER`、`PURCHASING`、`PI`、`VET`、`REVIEWER`、`IACUC_CHAIR`、`IACUC_STAFF`、`EXPERIMENT_STAFF`、`CLIENT`

---

## 4. 動物管理 (Migration 003)

### 4.1 核心資料表

| 表名 | PK 類型 | 主要欄位 |
|------|---------|---------| 
| animal_sources | UUID | name, supplier_type, contact_info |
| animals | SERIAL | ear_tag, status, breed, gender, source_id, pen_id, iacuc_no |
| animal_observations | SERIAL | animal_id, event_date, record_type, content, treatments(JSONB) |
| animal_surgeries | SERIAL | animal_id, surgery_date, surgery_site, anesthesia(JSONB), vital_signs(JSONB) |
| animal_weights | SERIAL | animal_id, weight_date, weight, weight_category |
| animal_vaccinations | SERIAL | animal_id, vaccine_name, vaccination_date, vet_name |
| animal_sacrifices | SERIAL | animal_id, sacrifice_date, method, executor |
| animal_pathology_reports | SERIAL | animal_id, report_date, findings(JSONB) |
| animal_care_records | SERIAL | animal_id, record_date, mode, content(JSONB) |
| animal_files | UUID | file_type, animal_id, record_type, record_id |
| record_versions | UUID | record_type, record_id, version_number, data(JSONB) |
| electronic_signatures | UUID | record_type, record_id, signer_id, signature_data(JSONB) |
| record_annotations | UUID | record_type, record_id, content, created_by |
| vet_recommendations | UUID | record_type, record_id, recommendation, status |
| euthanasia_orders | UUID | animal_id, status, reason, requested_by |
| import_batches | UUID | import_type, status, total_rows |
| export_requests | UUID | iacuc_no, export_type, format |

### 4.2 設施管理

| 表名 | PK | 說明 |
|------|-----|------|
| species | UUID | 物種定義 |
| facilities | UUID | 設施（含物種關聯）|
| buildings | UUID | 棟舍 → 設施 |
| zones | UUID | 區域 → 棟舍 |
| pens | UUID | 欄位 → 區域 |
| departments | UUID | 部門定義 |

---

## 5. AUP 系統 (Migration 004 + 008)

### 5.1 核心資料表

| 表名 | 說明 |
|------|------|
| protocols | 計畫書主表（working_content JSONB 存儲 Section 1-8 表單）|
| protocol_versions | 計畫版本快照 |
| user_protocols | 計畫成員（PI, CLIENT, CO_EDITOR）|
| review_assignments | 審查指派（V2: 含 comments_count, is_locked）|
| review_comments | 審查意見（含 parent_id 支援巢狀回覆）|
| review_comment_drafts | 審查意見草稿 |
| protocol_activities | 計畫活動紀錄 |
| protocol_attachments | 計畫附件 |
| amendments | 變更申請（含 content JSONB, previous_content JSONB）|
| amendment_versions | 變更版本快照 |

---

## 6. 人事系統 (Migration 005)

| 表名 | 說明 |
|------|------|
| attendance_records | 出勤打卡（clock_in, clock_out, work_hours）|
| leave_requests | 請假申請（多級審核：L1→L2→HR→GM）|
| overtime_records | 加班申請 |
| leave_balances | 假期餘額（按年度、假別）|
| leave_balance_adjustments | 餘額調整記錄 |
| calendar_settings | Google 行事曆同步設定 |
| google_calendar_sync_tokens | Google Calendar 同步令牌 |
| hr_scheduled_events | 排程活動 |

---

## 7. 稽核系統 (Migration 006)

### 7.1 活動記錄（分割表）

```sql
-- 主表（依日期分割）
CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50),
    entity_id VARCHAR(100),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    session_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);
```

自動維護分割表：每月建立一個新分割區、保留 6 個月，由 `partition_maintenance.rs` 排程管理。

### 7.2 安全相關表

| 表名 | 說明 |
|------|------|
| login_events | 登入事件（IP, GeoIP, 成功/失敗）|
| user_sessions | 使用者工作階段（token_jti, last_activity_at, geoip(JSONB)）|
| security_alerts | 安全警報（類型：多次失敗登入、異常登入地點...）|

---

## 8. ERP/倉庫 (Migration 007)

### 8.1 核心資料表

| 表名 | 說明 |
|------|------|
| products | 產品主表 |
| product_units | 產品單位換算 |
| warehouses | 倉庫主表 |
| storage_locations | 倉庫儲位 |
| storage_location_inventory | 儲位庫存 |
| partners | 供應商/客戶 |
| documents | 單據主表 |
| document_lines | 單據明細 |
| stock_ledger | 庫存異動紀錄 |
| skus | SKU 定義 |
| sku_categories | SKU 大類 |
| sku_subcategories | SKU 小類 |
| scheduled_reports | 排程報表 |
| report_history | 報表歷程 |

---

## 9. 血液檢查 (Migration 009)

```sql
CREATE TABLE blood_test_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    name_en VARCHAR(200),
    default_unit VARCHAR(50),
    reference_range VARCHAR(100),
    default_price NUMERIC(10,2),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ
);

CREATE TABLE animal_blood_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    test_date DATE NOT NULL,
    lab_name VARCHAR(200),
    lab_report_no VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    created_by UUID, updated_by UUID,
    created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ
);

CREATE TABLE animal_blood_test_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blood_test_id UUID NOT NULL REFERENCES animal_blood_tests(id) ON DELETE CASCADE,
    template_id UUID REFERENCES blood_test_templates(id),
    item_name VARCHAR(200) NOT NULL,
    result_value VARCHAR(50),
    unit VARCHAR(50),
    reference_range VARCHAR(100),
    is_abnormal BOOLEAN NOT NULL DEFAULT false,
    unit_price NUMERIC(10,2),
    remark TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE blood_test_panels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(30) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(100),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE blood_test_panel_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    panel_id UUID NOT NULL REFERENCES blood_test_panels(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES blood_test_templates(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(panel_id, template_id)
);
```

---

## 10. 使用者偏好 (Migration 010)

```sql
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key VARCHAR(100) NOT NULL,
    preference_value JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, preference_key)
);
```

---

## 11. 資料表統計

| 模組 | 資料表數 | 索引數 | 觸發器 |
|------|----------|--------|--------|
| 核心架構 (001) | 8 | 8 | 1 |
| 權限種子 (002) | — | — | — |
| 動物管理 (003) | 17+ | 15+ | 1 |
| AUP 系統 (004+008) | 10+ | 10+ | — |
| 人事系統 (005) | 8 | 8+ | — |
| 稽核系統 (006) | 4 | 5+ | — |
| ERP/倉庫 (007) | 14+ | 12+ | — |
| 血液檢查 (009) | 5 | 6 | — |
| 使用者偏好 (010) | 1 | 1 | — |
| **合計** | **65+** | **65+** | **2** |

---

*下一章：[API 規格](./05_API_SPECIFICATION.md)*
