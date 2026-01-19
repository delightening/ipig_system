# iPig System 資料庫實體關係圖

> **版本**: 2.0 | **更新日期**: 2026-01-19

---

## 系統模組概覽

```mermaid
flowchart TB
    subgraph Core["🔐 核心模組"]
        Users["Users 使用者"]
        Roles["Roles 角色"]
        Permissions["Permissions 權限"]
    end
    
    subgraph ERP["📦 ERP 模組"]
        Products["Products 產品"]
        Partners["Partners 夥伴"]
        Documents["Documents 單據"]
        Stock["Stock 庫存"]
    end
    
    subgraph Protocol["📋 實驗計畫模組"]
        Protocols["Protocols 計畫"]
        Reviews["Reviews 審查"]
    end
    
    subgraph Animal["🐷 動物管理模組"]
        Pigs["Pigs 豬隻"]
        Records["Records 紀錄"]
    end
    
    subgraph HR["👥 人資模組"]
        Attendance["Attendance 出勤"]
        Leave["Leave 請假"]
        Overtime["Overtime 加班"]
    end
    
    subgraph Audit["📊 稽核模組"]
        Logs["Activity Logs 活動日誌"]
        Sessions["Sessions 工作階段"]
    end
    
    Core --> ERP
    Core --> Protocol
    Core --> Animal
    Core --> HR
    Core --> Audit
```

---

## 1. 核心認證模組 (Core Authentication)

```mermaid
erDiagram
    users ||--o{ user_roles : "has"
    users ||--o{ refresh_tokens : "owns"
    users ||--o{ password_reset_tokens : "requests"
    roles ||--o{ user_roles : "assigned to"
    roles ||--o{ role_permissions : "has"
    permissions ||--o{ role_permissions : "granted to"
    
    users {
        UUID id PK
        VARCHAR email UK
        VARCHAR password_hash
        VARCHAR display_name
        VARCHAR phone
        VARCHAR organization
        BOOLEAN is_internal
        BOOLEAN is_active
        BOOLEAN must_change_password
        TIMESTAMPTZ last_login_at
        INTEGER login_attempts
        TIMESTAMPTZ locked_until
    }
    
    roles {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        TEXT description
        BOOLEAN is_internal
        BOOLEAN is_system
        BOOLEAN is_deleted
        BOOLEAN is_active
    }
    
    permissions {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        VARCHAR module
        TEXT description
    }
    
    user_roles {
        UUID user_id PK,FK
        UUID role_id PK,FK
        TIMESTAMPTZ assigned_at
        UUID assigned_by FK
    }
    
    role_permissions {
        UUID role_id PK,FK
        UUID permission_id PK,FK
    }
    
    refresh_tokens {
        UUID id PK
        UUID user_id FK
        VARCHAR token_hash
        TIMESTAMPTZ expires_at
        TIMESTAMPTZ revoked_at
    }
    
    password_reset_tokens {
        UUID id PK
        UUID user_id FK
        VARCHAR token_hash
        TIMESTAMPTZ expires_at
        TIMESTAMPTZ used_at
    }
```

---

## 2. ERP 庫存管理模組

```mermaid
erDiagram
    warehouses ||--o{ documents : "stores"
    warehouses ||--o{ stock_ledger : "tracks"
    warehouses ||--o{ inventory_snapshots : "snapshots"
    products ||--o{ document_lines : "ordered in"
    products ||--o{ stock_ledger : "tracked"
    products ||--o{ inventory_snapshots : "counted"
    partners ||--o{ documents : "involved in"
    documents ||--o{ document_lines : "contains"
    documents ||--o{ stock_ledger : "generates"
    sku_categories ||--o{ sku_subcategories : "has"
    sku_categories ||--o{ products : "categorizes"
    users ||--o{ documents : "creates"
    
    warehouses {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        TEXT address
        BOOLEAN is_active
    }
    
    sku_categories {
        CHAR3 code PK
        VARCHAR name
        INTEGER sort_order
        BOOLEAN is_active
    }
    
    sku_subcategories {
        SERIAL id PK
        CHAR3 category_code FK
        CHAR3 code
        VARCHAR name
    }
    
    products {
        UUID id PK
        VARCHAR sku UK
        VARCHAR name
        TEXT spec
        CHAR3 category_code FK
        CHAR3 subcategory_code FK
        VARCHAR base_uom
        BOOLEAN track_batch
        BOOLEAN track_expiry
        NUMERIC safety_stock
        VARCHAR status
        BOOLEAN is_active
    }
    
    partners {
        UUID id PK
        partner_type partner_type
        VARCHAR code UK
        VARCHAR name
        supplier_category supplier_category
        BOOLEAN is_active
    }
    
    documents {
        UUID id PK
        doc_type doc_type
        VARCHAR doc_no UK
        doc_status status
        UUID warehouse_id FK
        UUID partner_id FK
        DATE doc_date
        UUID created_by FK
        UUID approved_by FK
    }
    
    document_lines {
        UUID id PK
        UUID document_id FK
        INTEGER line_no
        UUID product_id FK
        NUMERIC qty
        VARCHAR uom
        NUMERIC unit_price
    }
    
    stock_ledger {
        UUID id PK
        UUID warehouse_id FK
        UUID product_id FK
        TIMESTAMPTZ trx_date
        doc_type doc_type
        UUID doc_id FK
        stock_direction direction
        NUMERIC qty_base
    }
    
    inventory_snapshots {
        UUID warehouse_id PK,FK
        UUID product_id PK,FK
        NUMERIC on_hand_qty_base
    }
```

---

## 3. 實驗計畫模組 (Protocol)

```mermaid
erDiagram
    users ||--o{ protocols : "owns as PI"
    users ||--o{ user_protocols : "participates"
    protocols ||--o{ user_protocols : "has members"
    protocols ||--o{ protocol_versions : "has versions"
    protocols ||--o{ protocol_status_history : "tracks status"
    protocols ||--o{ review_assignments : "reviewed by"
    protocol_versions ||--o{ review_comments : "receives"
    users ||--o{ review_assignments : "assigned as reviewer"
    users ||--o{ review_comments : "writes"
    
    protocols {
        UUID id PK
        VARCHAR protocol_no UK
        VARCHAR iacuc_no UK
        VARCHAR title
        protocol_status status
        UUID pi_user_id FK
        JSONB working_content
        DATE start_date
        DATE end_date
    }
    
    user_protocols {
        UUID user_id PK,FK
        UUID protocol_id PK,FK
        protocol_role role_in_protocol
    }
    
    protocol_versions {
        UUID id PK
        UUID protocol_id FK
        INTEGER version_no
        JSONB content_snapshot
        TIMESTAMPTZ submitted_at
    }
    
    protocol_status_history {
        UUID id PK
        UUID protocol_id FK
        protocol_status from_status
        protocol_status to_status
        UUID changed_by FK
        TEXT remark
    }
    
    review_assignments {
        UUID id PK
        UUID protocol_id FK
        UUID reviewer_id FK
        UUID assigned_by FK
    }
    
    review_comments {
        UUID id PK
        UUID protocol_version_id FK
        UUID reviewer_id FK
        TEXT content
        BOOLEAN is_resolved
    }
```

---

## 4. 動物管理模組 (Animal/Pig)

```mermaid
erDiagram
    pig_sources ||--o{ pigs : "provides"
    pigs ||--o{ pig_observations : "has"
    pigs ||--o{ pig_surgeries : "undergoes"
    pigs ||--o{ pig_weights : "measured"
    pigs ||--o{ pig_vaccinations : "receives"
    pigs ||--|| pig_sacrifices : "may have"
    pigs ||--|| pig_pathology_reports : "may have"
    pig_observations ||--o{ pig_record_attachments : "attached"
    pig_surgeries ||--o{ pig_record_attachments : "attached"
    pig_observations ||--o{ vet_recommendations : "has"
    pig_surgeries ||--o{ vet_recommendations : "has"
    
    pig_sources {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        BOOLEAN is_active
    }
    
    pigs {
        SERIAL id PK
        VARCHAR ear_tag
        pig_status status
        pig_breed breed
        UUID source_id FK
        pig_gender gender
        DATE birth_date
        DATE entry_date
        NUMERIC entry_weight
        VARCHAR pen_location
        VARCHAR pre_experiment_code
        VARCHAR iacuc_no
        DATE experiment_date
        BOOLEAN is_deleted
        TIMESTAMPTZ deleted_at
        UUID deleted_by FK
    }
    
    pig_observations {
        SERIAL id PK
        INTEGER pig_id FK
        DATE event_date
        record_type record_type
        TEXT content
        BOOLEAN no_medication_needed
        BOOLEAN stop_medication
        JSONB treatments
        BOOLEAN vet_read
    }
    
    pig_surgeries {
        SERIAL id PK
        INTEGER pig_id FK
        BOOLEAN is_first_experiment
        DATE surgery_date
        VARCHAR surgery_site
        JSONB induction_anesthesia
        JSONB anesthesia_maintenance
        JSONB vital_signs
        BOOLEAN vet_read
    }
    
    pig_weights {
        SERIAL id PK
        INTEGER pig_id FK
        DATE measure_date
        NUMERIC weight
    }
    
    pig_vaccinations {
        SERIAL id PK
        INTEGER pig_id FK
        DATE administered_date
        VARCHAR vaccine
    }
    
    pig_sacrifices {
        SERIAL id PK
        INTEGER pig_id FK,UK
        DATE sacrifice_date
        BOOLEAN confirmed_sacrifice
    }
    
    pig_pathology_reports {
        SERIAL id PK
        INTEGER pig_id FK,UK
    }
    
    pig_record_attachments {
        UUID id PK
        pig_record_type record_type
        INTEGER record_id
        VARCHAR file_path
    }
    
    vet_recommendations {
        SERIAL id PK
        vet_record_type record_type
        INTEGER record_id
        TEXT content
    }
```

---

## 5. 人資管理模組 (HR)

```mermaid
erDiagram
    users ||--o{ attendance_records : "has"
    users ||--o{ overtime_records : "submits"
    users ||--o{ annual_leave_entitlements : "entitled"
    users ||--o{ comp_time_balances : "earns"
    users ||--o{ leave_requests : "requests"
    users ||--o{ leave_requests : "proxies for"
    users ||--o{ leave_approvals : "approves"
    overtime_records ||--o{ comp_time_balances : "generates"
    leave_requests ||--o{ leave_approvals : "requires"
    leave_requests ||--o{ calendar_event_sync : "syncs to"
    
    attendance_records {
        UUID id PK
        UUID user_id FK
        DATE work_date
        TIMESTAMPTZ clock_in_time
        TIMESTAMPTZ clock_out_time
        NUMERIC regular_hours
        NUMERIC overtime_hours
        VARCHAR status
    }
    
    overtime_records {
        UUID id PK
        UUID user_id FK
        DATE overtime_date
        TIMESTAMPTZ start_time
        TIMESTAMPTZ end_time
        NUMERIC hours
        VARCHAR overtime_type
        NUMERIC multiplier
        NUMERIC comp_time_hours
        DATE comp_time_expires_at
        VARCHAR status
    }
    
    annual_leave_entitlements {
        UUID id PK
        UUID user_id FK
        INTEGER entitlement_year
        NUMERIC entitled_days
        NUMERIC used_days
        DATE expires_at
        BOOLEAN is_expired
    }
    
    comp_time_balances {
        UUID id PK
        UUID user_id FK
        UUID overtime_record_id FK
        NUMERIC original_hours
        NUMERIC used_hours
        DATE earned_date
        DATE expires_at
        BOOLEAN is_expired
    }
    
    leave_requests {
        UUID id PK
        UUID user_id FK
        UUID proxy_user_id FK
        leave_type leave_type
        DATE start_date
        DATE end_date
        NUMERIC total_days
        TEXT reason
        leave_status status
        UUID current_approver_id FK
    }
    
    leave_approvals {
        UUID id PK
        UUID leave_request_id FK
        UUID approver_id FK
        VARCHAR approval_level
        VARCHAR action
    }
```

---

## 6. 日曆同步與通知模組

```mermaid
erDiagram
    leave_requests ||--|| calendar_event_sync : "syncs"
    leave_requests ||--o{ calendar_sync_conflicts : "may have"
    users ||--o{ notifications : "receives"
    users ||--|| notification_settings : "configures"
    
    google_calendar_config {
        UUID id PK
        VARCHAR calendar_id
        VARCHAR calendar_name
        BOOLEAN is_configured
        BOOLEAN sync_enabled
        TIMESTAMPTZ last_sync_at
        VARCHAR last_sync_status
    }
    
    calendar_event_sync {
        UUID id PK
        UUID leave_request_id FK,UK
        VARCHAR google_event_id
        INTEGER sync_version
        VARCHAR sync_status
    }
    
    calendar_sync_conflicts {
        UUID id PK
        UUID leave_request_id FK
        VARCHAR conflict_type
        JSONB ipig_data
        JSONB google_data
        VARCHAR status
    }
    
    calendar_sync_history {
        UUID id PK
        VARCHAR job_type
        TIMESTAMPTZ started_at
        VARCHAR status
        INTEGER events_created
    }
    
    notifications {
        UUID id PK
        UUID user_id FK
        notification_type type
        VARCHAR title
        TEXT content
        BOOLEAN is_read
        VARCHAR related_entity_type
        UUID related_entity_id
    }
    
    notification_settings {
        UUID user_id PK,FK
        BOOLEAN email_low_stock
        BOOLEAN email_expiry_warning
        BOOLEAN email_document_approval
        BOOLEAN email_protocol_status
        INTEGER expiry_warning_days
    }
    
    scheduled_reports {
        UUID id PK
        VARCHAR name
        report_type report_type
        schedule_type schedule_type
        JSONB schedule_config
        TEXT recipients
        BOOLEAN is_active
    }
```

---

## 7. 稽核與安全模組

```mermaid
erDiagram
    users ||--o{ user_activity_logs : "generates"
    users ||--o{ login_events : "triggers"
    users ||--o{ user_sessions : "has"
    users ||--o{ security_alerts : "may trigger"
    
    user_activity_logs {
        UUID id PK
        UUID actor_user_id FK
        VARCHAR actor_email
        VARCHAR event_category
        VARCHAR event_type
        VARCHAR entity_type
        UUID entity_id
        JSONB before_data
        JSONB after_data
        TEXT changed_fields
        INET ip_address
        DATE partition_date
    }
    
    login_events {
        UUID id PK
        UUID user_id FK
        VARCHAR email
        VARCHAR event_type
        INET ip_address
        BOOLEAN is_unusual_time
        BOOLEAN is_unusual_location
        BOOLEAN is_new_device
        VARCHAR failure_reason
    }
    
    user_sessions {
        UUID id PK
        UUID user_id FK
        TIMESTAMPTZ started_at
        TIMESTAMPTZ ended_at
        TIMESTAMPTZ last_activity_at
        BOOLEAN is_active
        VARCHAR ended_reason
    }
    
    security_alerts {
        UUID id PK
        VARCHAR alert_type
        VARCHAR severity
        VARCHAR title
        UUID user_id FK
        VARCHAR status
    }
```

---

## 資料類型列舉 (Enums)

| 類別 | 列舉名稱 | 值 |
|------|----------|-----|
| **夥伴** | `partner_type` | supplier, customer |
| **夥伴** | `supplier_category` | drug, consumable, feed, equipment, other |
| **單據** | `doc_type` | PO, GRN, PR, SO, DO, SR, TR, STK, ADJ, RTN |
| **單據** | `doc_status` | draft, submitted, approved, cancelled |
| **庫存** | `stock_direction` | in, out, transfer_in, transfer_out, adjust_in, adjust_out |
| **計畫** | `protocol_role` | PI, CLIENT, CO_EDITOR |
| **計畫** | `protocol_status` | DRAFT, SUBMITTED, PRE_REVIEW, UNDER_REVIEW, REVISION_REQUIRED, RESUBMITTED, APPROVED, APPROVED_WITH_CONDITIONS, DEFERRED, REJECTED, SUSPENDED, CLOSED, DELETED |
| **動物** | `pig_status` | unassigned, assigned, in_experiment, completed, transferred, deceased |
| **動物** | `pig_breed` | miniature, white, LYD, other |
| **動物** | `pig_gender` | male, female |
| **請假** | `leave_type` | ANNUAL, PERSONAL, SICK, COMPENSATORY, MARRIAGE, BEREAVEMENT, MATERNITY, PATERNITY, MENSTRUAL, OFFICIAL, UNPAID |
| **請假** | `leave_status` | DRAFT, PENDING_L1, PENDING_L2, PENDING_HR, PENDING_GM, APPROVED, REJECTED, CANCELLED, REVOKED |
| **通知** | `notification_type` | low_stock, expiry_warning, document_approval, protocol_status, vet_recommendation, system_alert, monthly_report |

---

## 統計摘要

| 模組 | 資料表數量 | 主要實體 |
|------|-----------|----------|
| 核心認證 | 7 | users, roles, permissions |
| ERP 庫存 | 9 | products, partners, documents, stock |
| 實驗計畫 | 6 | protocols, versions, reviews |
| 動物管理 | 10 | pigs, observations, surgeries |
| 人資管理 | 6 | attendance, leave, overtime |
| 日曆通知 | 7 | calendar sync, notifications |
| 稽核安全 | 4 | activity logs, sessions, alerts |
| **總計** | **49** | |
