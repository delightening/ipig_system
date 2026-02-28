# 資料庫實體關係圖 (ERD)

> **版本**：7.0  
> **最後更新**：2026-03-01

---

## 1. 總覽

```mermaid
erDiagram
    %% ===== 核心架構 =====
    users ||--o{ user_roles : has
    roles ||--o{ user_roles : assigned
    roles ||--o{ role_permissions : has
    permissions ||--o{ role_permissions : granted
    users ||--o{ user_preferences : has
    users ||--o{ refresh_tokens : has

    %% ===== AUP 審查 =====
    users ||--o{ protocols : "owns (PI)"
    users ||--o{ user_protocols : participates
    protocols ||--o{ user_protocols : has_members
    protocols ||--o{ protocol_versions : versioned
    protocols ||--o{ review_assignments : reviewed
    protocols ||--o{ review_comments : has_comments
    protocols ||--o{ protocol_activities : tracked
    protocols ||--o{ amendments : amended

    %% ===== 動物管理 =====
    animal_sources ||--o{ animals : provides
    animals ||--o{ animal_observations : has
    animals ||--o{ animal_surgeries : undergoes
    animals ||--o{ animal_weights : measured
    animals ||--o{ animal_vaccinations : receives
    animals ||--o{ animal_blood_tests : tested
    animals ||--|| animal_sacrifices : "may have"
    animals ||--|| animal_pathology_reports : "may have"
    animals ||--o{ euthanasia_orders : "may have"
    animals ||--|| animal_sudden_deaths : "may have"
    animals ||--o{ animal_transfers : "may transfer"
    animal_transfers ||--o{ transfer_vet_evaluations : evaluated
    blood_test_templates ||--o{ animal_blood_test_items : used_in
    blood_test_panels ||--o{ blood_test_panel_items : contains

    %% ===== ERP =====
    warehouses ||--o{ storage_locations : has
    warehouses ||--o{ documents : stores
    partners ||--o{ documents : transacts
    products ||--o{ document_lines : referenced
    documents ||--o{ document_lines : contains
    documents ||--o{ stock_ledger : records

    %% ===== HR =====
    users ||--o{ attendance_records : has
    users ||--o{ leave_requests : applies
    users ||--o{ overtime_records : submits
    users ||--o{ leave_balances : has

    %% ===== 稽核 =====
    users ||--o{ user_activity_logs : generates
    users ||--o{ user_sessions : has
    users ||--o{ login_events : triggers
```

---

## 2. 核心架構 ERD

```mermaid
erDiagram
    users {
        UUID id PK
        VARCHAR email UK
        VARCHAR password_hash
        VARCHAR display_name
        VARCHAR phone
        VARCHAR organization
        UUID department_id FK
        UUID direct_manager_id FK
        BOOLEAN is_internal
        BOOLEAN is_active
        BOOLEAN must_change_password
        TIMESTAMPTZ last_login_at
        INTEGER login_attempts
        TIMESTAMPTZ locked_until
        VARCHAR theme_preference
        VARCHAR language_preference
        DATE entry_date
        VARCHAR position
        JSONB trainings
        TIMESTAMPTZ deleted_at
        BOOLEAN totp_enabled
        TEXT totp_secret_encrypted
        TEXT[] totp_backup_codes
    }

    roles {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        BOOLEAN is_system
        BOOLEAN is_internal
    }

    permissions {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        VARCHAR module
    }

    user_roles {
        UUID user_id FK
        UUID role_id FK
        TIMESTAMPTZ assigned_at
        UUID assigned_by FK
    }

    role_permissions {
        UUID role_id FK
        UUID permission_id FK
    }

    user_preferences {
        UUID id PK
        UUID user_id FK
        VARCHAR preference_key
        JSONB preference_value
    }

    notifications {
        UUID id PK
        UUID user_id FK
        notification_type type
        VARCHAR title
        TEXT message
        JSONB data
        BOOLEAN is_read
    }

    refresh_tokens {
        UUID id PK
        UUID user_id FK
        VARCHAR token_hash
        TIMESTAMPTZ expires_at
    }

    users ||--o{ user_roles : has
    users ||--o{ user_preferences : has
    roles ||--o{ user_roles : assigned
    roles ||--o{ role_permissions : has
    permissions ||--o{ role_permissions : granted
    users ||--o{ notifications : receives
    users ||--o{ refresh_tokens : has
```

---

## 3. AUP 審查系統 ERD

```mermaid
erDiagram
    protocols {
        UUID id PK
        VARCHAR protocol_no UK
        VARCHAR iacuc_no
        VARCHAR title
        protocol_status status
        UUID pi_user_id FK
        JSONB working_content
        DATE start_date
        DATE end_date
        BOOLEAN is_deleted
        INTEGER version
    }

    protocol_versions {
        UUID id PK
        UUID protocol_id FK
        INTEGER version_number
        JSONB content
        UUID created_by FK
    }

    user_protocols {
        UUID id PK
        UUID protocol_id FK
        UUID user_id FK
        protocol_role role
    }

    review_assignments {
        UUID id PK
        UUID protocol_id FK
        UUID reviewer_id FK
        VARCHAR review_type
        VARCHAR status
        INTEGER comments_count
        BOOLEAN is_locked
    }

    review_comments {
        UUID id PK
        UUID assignment_id FK
        UUID protocol_id FK
        UUID user_id FK
        UUID parent_id FK
        VARCHAR section
        TEXT content
        BOOLEAN is_resolved
    }

    review_comment_drafts {
        UUID id PK
        UUID assignment_id FK
        UUID user_id FK
        JSONB content
    }

    protocol_activities {
        UUID id PK
        UUID protocol_id FK
        protocol_activity_type activity_type
        UUID user_id FK
        JSONB details
    }

    protocol_attachments {
        UUID id PK
        UUID protocol_id FK
        VARCHAR filename
        VARCHAR filepath
        UUID uploaded_by FK
    }

    amendments {
        UUID id PK
        UUID protocol_id FK
        VARCHAR amendment_no
        amendment_type type
        amendment_status status
        JSONB content
        JSONB previous_content
        UUID submitted_by FK
    }

    amendment_versions {
        UUID id PK
        UUID amendment_id FK
        INTEGER version_number
        JSONB content
    }

    protocols ||--o{ protocol_versions : versioned
    protocols ||--o{ user_protocols : has_members
    protocols ||--o{ review_assignments : reviewed
    protocols ||--o{ review_comments : commented
    protocols ||--o{ protocol_activities : tracked
    protocols ||--o{ protocol_attachments : has
    protocols ||--o{ amendments : amended
    review_assignments ||--o{ review_comments : has
    review_comments ||--o{ review_comments : replies
    amendments ||--o{ amendment_versions : versioned
```

---

## 4. 動物管理 ERD

```mermaid
erDiagram
    animal_sources ||--o{ animals : "provides"
    animals ||--o{ animal_observations : "has"
    animals ||--o{ animal_surgeries : "undergoes"
    animals ||--o{ animal_weights : "measured"
    animals ||--o{ animal_vaccinations : "receives"
    animals ||--|| animal_sacrifices : "may have"
    animals ||--|| animal_pathology_reports : "may have"
    animals ||--|| animal_sudden_deaths : "may have"
    animals ||--o{ animal_transfers : "may transfer"
    animal_transfers ||--o{ transfer_vet_evaluations : evaluated
    animal_observations ||--o{ animal_record_attachments : "attached"
    animal_surgeries ||--o{ animal_record_attachments : "attached"
    animal_observations ||--o{ vet_recommendations : "has"
    animal_surgeries ||--o{ vet_recommendations : "has"
    
    animals {
        UUID id PK
        VARCHAR ear_tag
        animal_status status
        animal_breed breed
        UUID source_id FK
        animal_gender gender
        DATE birth_date
        DATE entry_date
        NUMERIC entry_weight
        VARCHAR pen_location
        UUID pen_id FK
        VARCHAR pre_experiment_code
        VARCHAR iacuc_no
        DATE experiment_date
        BOOLEAN is_deleted
        TIMESTAMPTZ deleted_at
        UUID deleted_by FK
        INTEGER version
    }

    animal_sources {
        UUID id PK
        VARCHAR name
        VARCHAR supplier_type
        JSONB contact_info
        BOOLEAN is_active
    }

    animal_observations {
        UUID id PK
        UUID animal_id FK
        DATE event_date
        record_type record_type
        TEXT content
        BOOLEAN no_medication_needed
        BOOLEAN stop_medication
        JSONB treatments
        BOOLEAN vet_read
        UUID created_by FK
        UUID updated_by FK
        INTEGER version
    }

    animal_surgeries {
        UUID id PK
        UUID animal_id FK
        BOOLEAN is_first_experiment
        DATE surgery_date
        VARCHAR surgery_site
        JSONB induction_anesthesia
        JSONB anesthesia_maintenance
        JSONB vital_signs
        BOOLEAN vet_read
        UUID created_by FK
        INTEGER version
    }

    animal_weights {
        UUID id PK
        UUID animal_id FK
        DATE measure_date
        NUMERIC weight
        VARCHAR weight_category
        UUID created_by FK
    }

    animal_vaccinations {
        UUID id PK
        UUID animal_id FK
        VARCHAR vaccine_name
        DATE vaccination_date
        VARCHAR vet_name
        UUID created_by FK
    }

    animal_sacrifices {
        UUID id PK
        UUID animal_id FK
        DATE sacrifice_date
        VARCHAR method
        VARCHAR executor
        TEXT notes
        UUID created_by FK
    }

    animal_pathology_reports {
        UUID id PK
        UUID animal_id FK
        DATE report_date
        JSONB findings
        UUID created_by FK
    }

    animal_record_attachments {
        UUID id PK
        animal_record_type record_type
        UUID record_id
        animal_file_type file_type
        VARCHAR file_name
        VARCHAR file_path
        INTEGER file_size
        VARCHAR mime_type
    }

    vet_recommendations {
        UUID id PK
        vet_record_type record_type
        UUID record_id
        TEXT recommendation
        VARCHAR status
        UUID created_by FK
    }

    animal_care_records {
        UUID id PK
        UUID animal_id FK
        DATE record_date
        care_record_mode mode
        JSONB content
        UUID created_by FK
    }

    animal_sudden_deaths {
        UUID id PK
        UUID animal_id FK "UNIQUE"
        DATE death_date
        TEXT description
        UUID discovered_by FK
        UUID reported_by FK
    }

    animal_transfers {
        UUID id PK
        UUID animal_id FK
        animal_transfer_status status
        UUID from_protocol_id FK
        UUID to_protocol_id FK
        TEXT reason
        UUID initiated_by FK
        TIMESTAMPTZ transfer_date
    }

    transfer_vet_evaluations {
        UUID id PK
        UUID transfer_id FK
        UUID vet_id FK
        TEXT health_status
        BOOLEAN is_fit_for_transfer
        TEXT notes
    }
```

---

## 5. 血液檢查 ERD

```mermaid
erDiagram
    animals ||--o{ animal_blood_tests : tested
    animal_blood_tests ||--o{ animal_blood_test_items : contains
    blood_test_templates ||--o{ animal_blood_test_items : used_in
    blood_test_panels ||--o{ blood_test_panel_items : contains
    blood_test_templates ||--o{ blood_test_panel_items : grouped

    animal_blood_tests {
        UUID id PK
        UUID animal_id FK
        DATE test_date
        VARCHAR lab_name
        VARCHAR lab_report_no
        VARCHAR status
        TEXT remark
        BOOLEAN vet_read
        BOOLEAN is_deleted
        UUID created_by FK
    }

    animal_blood_test_items {
        UUID id PK
        UUID blood_test_id FK
        UUID template_id FK
        VARCHAR item_name
        VARCHAR result_value
        VARCHAR unit
        VARCHAR reference_range
        BOOLEAN is_abnormal
        NUMERIC unit_price
        TEXT remark
        INTEGER sort_order
    }

    blood_test_templates {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        VARCHAR name_en
        VARCHAR default_unit
        VARCHAR reference_range
        NUMERIC default_price
        INTEGER sort_order
        BOOLEAN is_active
    }

    blood_test_panels {
        UUID id PK
        VARCHAR key UK
        VARCHAR name
        VARCHAR icon
        INTEGER sort_order
        BOOLEAN is_active
    }

    blood_test_panel_items {
        UUID id PK
        UUID panel_id FK
        UUID template_id FK
        INTEGER sort_order
    }
```

---

## 6. 安樂死管理 ERD

```mermaid
erDiagram
    animals ||--o{ euthanasia_orders : "may have"
    users ||--o{ euthanasia_orders : "requests"
    users ||--o{ euthanasia_orders : "approves"

    euthanasia_orders {
        UUID id PK
        INTEGER animal_id FK
        euthanasia_order_status status
        TEXT reason
        UUID requested_by FK
        UUID approved_by FK
        TEXT pi_notes
        TIMESTAMPTZ approved_at
        TIMESTAMPTZ executed_at
    }
```

---

## 7. GLP 合規 ERD

```mermaid
erDiagram
    electronic_signatures {
        UUID id PK
        VARCHAR record_type
        INTEGER record_id
        UUID signer_id FK
        JSONB signature_data
        VARCHAR signature_method
        TEXT handwriting_svg
        JSONB stroke_data
        TIMESTAMPTZ signed_at
    }

    record_annotations {
        UUID id PK
        VARCHAR record_type
        INTEGER record_id
        TEXT content
        UUID created_by FK
        TIMESTAMPTZ created_at
    }

    record_versions {
        UUID id PK
        version_record_type record_type
        INTEGER record_id
        INTEGER version_number
        JSONB data
        UUID created_by FK
        TIMESTAMPTZ created_at
    }
```

---

## 8. ERP 進銷存 ERD

```mermaid
erDiagram
    products {
        UUID id PK
        VARCHAR sku UK
        VARCHAR name
        CHAR category_code
        CHAR subcategory_code
        VARCHAR base_uom
        NUMERIC safety_stock
        BOOLEAN track_batch
        BOOLEAN track_expiry
    }

    warehouses {
        UUID id PK
        VARCHAR code UK
        VARCHAR name
        VARCHAR address
        JSONB layout
    }

    storage_locations {
        UUID id PK
        UUID warehouse_id FK
        VARCHAR code
        VARCHAR name
        VARCHAR zone
        INTEGER row_no
        INTEGER col_no
        INTEGER level_no
    }

    storage_location_inventory {
        UUID id PK
        UUID location_id FK
        UUID product_id FK
        NUMERIC quantity
        VARCHAR batch_no
        DATE expiry_date
    }

    partners {
        UUID id PK
        partner_type type
        VARCHAR code UK
        VARCHAR name
        VARCHAR contact_name
        VARCHAR contact_email
        VARCHAR contact_phone
    }

    documents {
        UUID id PK
        doc_type doc_type
        VARCHAR doc_no UK
        doc_status status
        UUID warehouse_id FK
        UUID partner_id FK
        DATE doc_date
        NUMERIC total_amount
        UUID created_by FK
    }

    document_lines {
        UUID id PK
        UUID document_id FK
        UUID product_id FK
        NUMERIC quantity
        NUMERIC unit_price
        NUMERIC total
        VARCHAR batch_no
        DATE expiry_date
    }

    stock_ledger {
        UUID id PK
        UUID product_id FK
        UUID warehouse_id FK
        UUID document_id FK
        stock_direction direction
        NUMERIC quantity
        NUMERIC running_balance
    }

    warehouses ||--o{ storage_locations : has
    warehouses ||--o{ documents : stores
    storage_locations ||--o{ storage_location_inventory : contains
    partners ||--o{ documents : transacts
    documents ||--o{ document_lines : contains
    documents ||--o{ stock_ledger : records
    products ||--o{ document_lines : referenced
    products ||--o{ stock_ledger : tracked
```

---

## 9. HR 人事管理 ERD

```mermaid
erDiagram
    users ||--o{ attendance_records : has
    users ||--o{ leave_requests : applies
    users ||--o{ overtime_records : submits
    users ||--o{ leave_balances : has

    attendance_records {
        UUID id PK
        UUID user_id FK
        DATE date
        TIMESTAMPTZ clock_in
        TIMESTAMPTZ clock_out
        NUMERIC work_hours
    }

    leave_requests {
        UUID id PK
        UUID user_id FK
        UUID proxy_user_id FK
        leave_type leave_type
        DATE start_date
        DATE end_date
        NUMERIC total_days
        leave_status status
        UUID current_approver_id FK
        JSONB approval_history
    }

    overtime_records {
        UUID id PK
        UUID user_id FK
        DATE overtime_date
        TIME start_time
        TIME end_time
        NUMERIC total_hours
        VARCHAR compensation_type
        VARCHAR status
    }

    leave_balances {
        UUID id PK
        UUID user_id FK
        INTEGER year
        leave_type leave_type
        NUMERIC total_days
        NUMERIC used_days
        NUMERIC remaining_days
    }

    calendar_settings {
        UUID id PK
        UUID user_id FK
        VARCHAR calendar_id
        BOOLEAN auto_sync
        JSONB sync_config
    }
```

---

## 10. 稽核系統 ERD

```mermaid
erDiagram
    users ||--o{ user_activity_logs : generates
    users ||--o{ user_sessions : has
    users ||--o{ login_events : triggers
    users ||--o{ security_alerts : about

    user_activity_logs {
        UUID id PK
        UUID user_id FK
        VARCHAR action
        VARCHAR entity_type
        VARCHAR entity_id
        JSONB details
        INET ip_address
        TEXT user_agent
        UUID session_id FK
        TIMESTAMPTZ created_at
    }

    user_sessions {
        UUID id PK
        UUID user_id FK
        UUID token_jti
        INET ip_address
        TEXT user_agent
        JSONB geoip
        TIMESTAMPTZ last_activity_at
        TIMESTAMPTZ expired_at
        BOOLEAN is_active
    }

    login_events {
        UUID id PK
        UUID user_id FK
        VARCHAR email
        BOOLEAN success
        INET ip_address
        TEXT user_agent
        JSONB geoip
        TEXT failure_reason
        TIMESTAMPTZ created_at
    }

    security_alerts {
        UUID id PK
        UUID user_id FK
        VARCHAR alert_type
        VARCHAR severity
        JSONB details
        BOOLEAN is_resolved
        UUID resolved_by FK
        TIMESTAMPTZ resolved_at
    }
```

---

## 11. 設施管理 ERD

```mermaid
erDiagram
    species ||--o{ facilities : supports
    facilities ||--o{ buildings : has
    buildings ||--o{ zones : has
    zones ||--o{ pens : has
    pens ||--o{ animals : houses

    species {
        UUID id PK
        VARCHAR name UK
        VARCHAR code UK
        TEXT description
        BOOLEAN is_active
    }

    facilities {
        UUID id PK
        VARCHAR name
        VARCHAR code UK
        VARCHAR address
        UUID species_id FK
    }

    buildings {
        UUID id PK
        UUID facility_id FK
        VARCHAR name
        VARCHAR code
    }

    zones {
        UUID id PK
        UUID building_id FK
        VARCHAR name
        VARCHAR code
    }

    pens {
        UUID id PK
        UUID zone_id FK
        VARCHAR name
        VARCHAR code
        INTEGER capacity
    }
```

---

## 12. 系統設定與輔助表 ERD

```mermaid
erDiagram
    system_settings {
        VARCHAR key PK
        JSONB value
        TEXT description
        TIMESTAMPTZ updated_at
        UUID updated_by FK
    }

    jwt_blacklist {
        VARCHAR jti PK
        TIMESTAMPTZ expires_at
        TIMESTAMPTZ revoked_at
    }

    treatment_drug_options {
        UUID id PK
        VARCHAR name
        VARCHAR display_name
        VARCHAR default_dosage_unit
        TEXT[] available_units
        VARCHAR category
        INTEGER sort_order
        BOOLEAN is_active
        UUID erp_product_id FK
    }

    products ||--o{ treatment_drug_options : "optional"
```

---

## 13. 資料表彙總

| 模組 | 資料表數量 | 主要表名 |
|------|------------|----------|
| 核心架構 | 9 | users, roles, permissions, notifications, attachments, refresh_tokens, user_preferences |
| 動物管理 | 15 | animals, animal_observations, animal_surgeries, animal_weights, animal_vaccinations, animal_sacrifices, animal_pathology_reports, animal_record_attachments, animal_care_records, animal_sudden_deaths, animal_transfers, transfer_vet_evaluations |
| 血液檢查 | 5 | animal_blood_tests, animal_blood_test_items, blood_test_templates, blood_test_panels, blood_test_panel_items |
| 安樂死 | 1 | euthanasia_orders |
| GLP 合規 | 3 | electronic_signatures, record_annotations, record_versions |
| AUP 系統 | 12 | protocols, protocol_versions, user_protocols, review_assignments, review_comments, amendments, system_settings |
| ERP 系統 | 14 | products, warehouses, storage_locations, partners, documents, stock_ledger, skus |
| HR 系統 | 8 | attendance_records, leave_requests, overtime_records, leave_balances, calendar_settings |
| 稽核系統 | 4 | user_activity_logs, user_sessions, login_events, security_alerts |
| 設施管理 | 5 | species, facilities, buildings, zones, pens |
| 輔助 | 3 | jwt_blacklist, treatment_drug_options, notification_routing |
| **合計** | **~73** | |

---

*回上層：[README](./README.md)*

*最後更新：2026-03-01*
