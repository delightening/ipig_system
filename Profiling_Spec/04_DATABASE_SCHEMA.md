# Database Schema

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Audience**: Database Administrators, Developers

---

## 1. Overview

The iPig database runs on PostgreSQL 15 and consists of 10 migration files organized by module:

| Migration | Description |
|-----------|-------------|
| 001_aup_system.sql | Core schema: users, roles, ERP, protocols, pigs, notifications |
| 002_erp_base_data.sql | SKU categories, product categories seed data |
| 003_seed_accounts.sql | Initial admin account and roles |
| 004_hr_system.sql | Attendance, overtime, leave management |
| 005_calendar_sync.sql | Google Calendar integration |
| 006_audit_system.sql | GLP-compliant audit logging |
| 007_seed_data.sql | Reference data (pig sources, permissions) |
| 008_reset_admin.sql | Admin password reset |
| 009_add_roles_is_active.sql | Role active flag |
| 010_add_deleted_at_column.sql | Soft delete for pigs |

---

## 2. Custom Types (Enums)

### Partner & ERP Types
```sql
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');
```

### Protocol Types
```sql
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');
CREATE TYPE protocol_status AS ENUM (
    'DRAFT', 'SUBMITTED', 'PRE_REVIEW', 'UNDER_REVIEW',
    'REVISION_REQUIRED', 'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS',
    'DEFERRED', 'REJECTED', 'SUSPENDED', 'CLOSED', 'DELETED'
);
```

### Animal Types
```sql
CREATE TYPE pig_status AS ENUM ('unassigned', 'assigned', 'in_experiment', 'completed', 'transferred', 'deceased');
CREATE TYPE pig_breed AS ENUM ('miniature', 'white', 'LYD', 'other');
CREATE TYPE pig_gender AS ENUM ('male', 'female');
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');
CREATE TYPE pig_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology');
CREATE TYPE pig_file_type AS ENUM ('photo', 'attachment', 'report');
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology');
```

### HR Types
```sql
CREATE TYPE leave_type AS ENUM (
    'ANNUAL', 'PERSONAL', 'SICK', 'COMPENSATORY', 'MARRIAGE',
    'BEREAVEMENT', 'MATERNITY', 'PATERNITY', 'MENSTRUAL', 'OFFICIAL', 'UNPAID'
);
CREATE TYPE leave_status AS ENUM (
    'DRAFT', 'PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM',
    'APPROVED', 'REJECTED', 'CANCELLED', 'REVOKED'
);
```

### Notification & Report Types
```sql
CREATE TYPE notification_type AS ENUM (
    'low_stock', 'expiry_warning', 'document_approval',
    'protocol_status', 'vet_recommendation', 'system_alert', 'monthly_report'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM (
    'stock_on_hand', 'stock_ledger', 'purchase_summary',
    'cost_summary', 'expiry_report', 'low_stock_report'
);
```

---

## 3. Core Tables

### 3.1 Users & Authentication

```sql
-- Users table
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_internal BOOLEAN NOT NULL DEFAULT true,
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    module VARCHAR(50),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Junction tables
CREATE TABLE role_permissions (role_id UUID, permission_id UUID, PRIMARY KEY);
CREATE TABLE user_roles (user_id UUID, role_id UUID, assigned_at TIMESTAMPTZ, assigned_by UUID, PRIMARY KEY);
CREATE TABLE refresh_tokens (id UUID, user_id UUID, token_hash VARCHAR, expires_at TIMESTAMPTZ, revoked_at TIMESTAMPTZ);
CREATE TABLE password_reset_tokens (id UUID, user_id UUID, token_hash VARCHAR, expires_at TIMESTAMPTZ, used_at TIMESTAMPTZ);
```

### 3.2 ERP Tables

```sql
-- Warehouses
CREATE TABLE warehouses (id UUID PRIMARY KEY, code VARCHAR(50) UNIQUE, name VARCHAR(200), address TEXT, is_active BOOLEAN);

-- SKU Categories
CREATE TABLE sku_categories (code CHAR(3) PRIMARY KEY, name VARCHAR(50), sort_order INTEGER, is_active BOOLEAN);
CREATE TABLE sku_subcategories (id SERIAL, category_code CHAR(3), code CHAR(3), name VARCHAR(50));
CREATE TABLE sku_sequences (category_code CHAR(3), subcategory_code CHAR(3), last_sequence INTEGER);

-- Products
CREATE TABLE products (
    id UUID PRIMARY KEY,
    sku VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    spec TEXT,
    category_code CHAR(3),
    subcategory_code CHAR(3),
    base_uom VARCHAR(20) NOT NULL DEFAULT 'pcs',
    track_batch BOOLEAN DEFAULT false,
    track_expiry BOOLEAN DEFAULT false,
    safety_stock NUMERIC(18,4),
    status VARCHAR(20) DEFAULT 'active',
    is_active BOOLEAN DEFAULT true
);

-- Partners
CREATE TABLE partners (
    id UUID PRIMARY KEY,
    partner_type partner_type NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    supplier_category supplier_category,
    is_active BOOLEAN DEFAULT true
);

-- Documents
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    doc_type doc_type NOT NULL,
    doc_no VARCHAR(50) NOT NULL UNIQUE,
    status doc_status DEFAULT 'draft',
    warehouse_id UUID,
    partner_id UUID,
    doc_date DATE NOT NULL,
    created_by UUID NOT NULL,
    approved_by UUID
);

CREATE TABLE document_lines (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL,
    line_no INTEGER NOT NULL,
    product_id UUID NOT NULL,
    qty NUMERIC(18,4) NOT NULL,
    uom VARCHAR(20) NOT NULL,
    unit_price NUMERIC(18,4)
);

-- Stock
CREATE TABLE stock_ledger (
    id UUID PRIMARY KEY,
    warehouse_id UUID NOT NULL,
    product_id UUID NOT NULL,
    trx_date TIMESTAMPTZ NOT NULL,
    doc_type doc_type NOT NULL,
    doc_id UUID NOT NULL,
    direction stock_direction NOT NULL,
    qty_base NUMERIC(18,4) NOT NULL
);

CREATE TABLE inventory_snapshots (
    warehouse_id UUID NOT NULL,
    product_id UUID NOT NULL,
    on_hand_qty_base NUMERIC(18,4) DEFAULT 0,
    PRIMARY KEY (warehouse_id, product_id)
);
```

### 3.3 Protocol Tables

```sql
CREATE TABLE protocols (
    id UUID PRIMARY KEY,
    protocol_no VARCHAR(50) NOT NULL UNIQUE,
    iacuc_no VARCHAR(50) UNIQUE,
    title VARCHAR(500) NOT NULL,
    status protocol_status DEFAULT 'DRAFT',
    pi_user_id UUID NOT NULL,
    working_content JSONB,
    start_date DATE,
    end_date DATE
);

CREATE TABLE user_protocols (
    user_id UUID NOT NULL,
    protocol_id UUID NOT NULL,
    role_in_protocol protocol_role NOT NULL,
    PRIMARY KEY (user_id, protocol_id)
);

CREATE TABLE protocol_versions (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE protocol_status_history (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL,
    from_status protocol_status,
    to_status protocol_status NOT NULL,
    changed_by UUID NOT NULL,
    remark TEXT
);

CREATE TABLE review_assignments (id UUID, protocol_id UUID, reviewer_id UUID, assigned_by UUID);
CREATE TABLE review_comments (id UUID, protocol_version_id UUID, reviewer_id UUID, content TEXT, is_resolved BOOLEAN);
```

### 3.4 Animal (Pig) Tables

```sql
CREATE TABLE pig_sources (
    id UUID PRIMARY KEY,
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE pigs (
    id SERIAL PRIMARY KEY,
    ear_tag VARCHAR(10) NOT NULL,
    status pig_status DEFAULT 'unassigned',
    breed pig_breed NOT NULL,
    source_id UUID,
    gender pig_gender NOT NULL,
    birth_date DATE,
    entry_date DATE NOT NULL,
    entry_weight NUMERIC(5,1),
    pen_location VARCHAR(10),
    pre_experiment_code VARCHAR(20),
    iacuc_no VARCHAR(20),
    experiment_date DATE,
    is_deleted BOOLEAN DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID
);

CREATE TABLE pig_observations (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id),
    event_date DATE NOT NULL,
    record_type record_type NOT NULL,
    content TEXT NOT NULL,
    no_medication_needed BOOLEAN DEFAULT false,
    stop_medication BOOLEAN DEFAULT false,
    treatments JSONB,
    vet_read BOOLEAN DEFAULT false
);

CREATE TABLE pig_surgeries (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id),
    is_first_experiment BOOLEAN DEFAULT true,
    surgery_date DATE NOT NULL,
    surgery_site VARCHAR(200) NOT NULL,
    induction_anesthesia JSONB,
    anesthesia_maintenance JSONB,
    vital_signs JSONB,
    vet_read BOOLEAN DEFAULT false
);

CREATE TABLE pig_weights (id SERIAL, pig_id INTEGER, measure_date DATE, weight NUMERIC(5,1));
CREATE TABLE pig_vaccinations (id SERIAL, pig_id INTEGER, administered_date DATE, vaccine VARCHAR(100));
CREATE TABLE pig_sacrifices (id SERIAL, pig_id INTEGER UNIQUE, sacrifice_date DATE, confirmed_sacrifice BOOLEAN);
CREATE TABLE pig_pathology_reports (id SERIAL, pig_id INTEGER UNIQUE);
CREATE TABLE pig_record_attachments (id UUID, record_type pig_record_type, record_id INTEGER, file_path VARCHAR);
CREATE TABLE vet_recommendations (id SERIAL, record_type vet_record_type, record_id INTEGER, content TEXT);
CREATE TABLE record_versions (id SERIAL, record_type version_record_type, record_id INTEGER, version_no INTEGER, snapshot JSONB);
```

### 3.5 HR Tables

```sql
CREATE TABLE attendance_records (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    work_date DATE NOT NULL,
    clock_in_time TIMESTAMPTZ,
    clock_out_time TIMESTAMPTZ,
    regular_hours NUMERIC(5,2) DEFAULT 0,
    overtime_hours NUMERIC(5,2) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'normal',
    UNIQUE(user_id, work_date)
);

CREATE TABLE overtime_records (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    overtime_date DATE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    hours NUMERIC(5,2) NOT NULL,
    overtime_type VARCHAR(20) NOT NULL, -- 'weekday', 'weekend', 'holiday'
    multiplier NUMERIC(3,2) DEFAULT 1.0,
    comp_time_hours NUMERIC(5,2) NOT NULL,
    comp_time_expires_at DATE NOT NULL,
    status VARCHAR(20) DEFAULT 'draft'
);

CREATE TABLE annual_leave_entitlements (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    entitlement_year INTEGER NOT NULL,
    entitled_days NUMERIC(5,2) NOT NULL,
    used_days NUMERIC(5,2) DEFAULT 0,
    expires_at DATE NOT NULL,
    is_expired BOOLEAN DEFAULT false,
    UNIQUE(user_id, entitlement_year)
);

CREATE TABLE comp_time_balances (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id),
    original_hours NUMERIC(5,2) NOT NULL,
    used_hours NUMERIC(5,2) DEFAULT 0,
    earned_date DATE NOT NULL,
    expires_at DATE NOT NULL,
    is_expired BOOLEAN DEFAULT false
);

CREATE TABLE leave_requests (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    proxy_user_id UUID REFERENCES users(id),
    leave_type leave_type NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    total_days NUMERIC(5,2) NOT NULL,
    reason TEXT,
    status leave_status DEFAULT 'DRAFT',
    current_approver_id UUID REFERENCES users(id)
);

CREATE TABLE leave_approvals (
    id UUID PRIMARY KEY,
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id),
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL,
    action VARCHAR(20) NOT NULL -- 'APPROVE', 'REJECT', etc.
);
```

### 3.6 Calendar Sync Tables

```sql
CREATE TABLE google_calendar_config (
    id UUID PRIMARY KEY,
    calendar_id VARCHAR(255) NOT NULL,
    calendar_name VARCHAR(100),
    is_configured BOOLEAN DEFAULT false,
    sync_enabled BOOLEAN DEFAULT true,
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20)
);

CREATE TABLE calendar_event_sync (
    id UUID PRIMARY KEY,
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id),
    google_event_id VARCHAR(255),
    sync_version INTEGER DEFAULT 0,
    sync_status VARCHAR(20) DEFAULT 'pending_create',
    UNIQUE(leave_request_id)
);

CREATE TABLE calendar_sync_conflicts (
    id UUID PRIMARY KEY,
    leave_request_id UUID REFERENCES leave_requests(id),
    conflict_type VARCHAR(50) NOT NULL,
    ipig_data JSONB NOT NULL,
    google_data JSONB,
    status VARCHAR(20) DEFAULT 'pending'
);

CREATE TABLE calendar_sync_history (
    id UUID PRIMARY KEY,
    job_type VARCHAR(20) NOT NULL,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'running',
    events_created INTEGER DEFAULT 0
);
```

### 3.7 Audit Tables

```sql
-- Partitioned by quarter for performance
CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    actor_user_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    event_category VARCHAR(50) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    entity_type VARCHAR(50),
    entity_id UUID,
    before_data JSONB,
    after_data JSONB,
    changed_fields TEXT[],
    ip_address INET,
    partition_date DATE NOT NULL DEFAULT CURRENT_DATE,
    PRIMARY KEY (id, partition_date)
) PARTITION BY RANGE (partition_date);

CREATE TABLE login_events (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    event_type VARCHAR(20) NOT NULL, -- 'login_success', 'login_failure', 'logout'
    ip_address INET,
    is_unusual_time BOOLEAN DEFAULT false,
    is_unusual_location BOOLEAN DEFAULT false,
    is_new_device BOOLEAN DEFAULT false,
    failure_reason VARCHAR(100)
);

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    ended_reason VARCHAR(50) -- 'logout', 'expired', 'forced_logout'
);

CREATE TABLE security_alerts (
    id UUID PRIMARY KEY,
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) DEFAULT 'warning',
    title VARCHAR(255) NOT NULL,
    user_id UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'open'
);
```

### 3.8 Notification Tables

```sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    type notification_type NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT,
    is_read BOOLEAN DEFAULT false,
    related_entity_type VARCHAR(50),
    related_entity_id UUID
);

CREATE TABLE notification_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    email_low_stock BOOLEAN DEFAULT true,
    email_expiry_warning BOOLEAN DEFAULT true,
    email_document_approval BOOLEAN DEFAULT true,
    email_protocol_status BOOLEAN DEFAULT true,
    expiry_warning_days INTEGER DEFAULT 30
);

CREATE TABLE scheduled_reports (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    report_type report_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    schedule_config JSONB,
    recipients TEXT[],
    is_active BOOLEAN DEFAULT true
);
```

---

## 4. Key Indexes

```sql
-- Users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- Pigs
CREATE INDEX idx_pigs_ear_tag ON pigs(ear_tag);
CREATE INDEX idx_pigs_status ON pigs(status);
CREATE INDEX idx_pigs_iacuc_no ON pigs(iacuc_no);
CREATE INDEX idx_pigs_pen_location ON pigs(pen_location);
CREATE INDEX idx_pigs_is_deleted ON pigs(is_deleted);

-- Protocols
CREATE INDEX idx_protocols_status ON protocols(status);
CREATE INDEX idx_protocols_pi_user_id ON protocols(pi_user_id);
CREATE INDEX idx_protocols_iacuc_no ON protocols(iacuc_no);

-- HR
CREATE INDEX idx_attendance_user_date ON attendance_records(user_id, work_date DESC);
CREATE INDEX idx_leave_user ON leave_requests(user_id, start_date DESC);
CREATE INDEX idx_leave_status ON leave_requests(status);

-- Audit (partitioned)
CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
```

---

## 5. Database Functions

```sql
-- Get remaining annual leave for user
CREATE FUNCTION get_annual_leave_balance(p_user_id UUID) RETURNS TABLE (...);

-- Get remaining comp time for user (FIFO order)
CREATE FUNCTION get_comp_time_balance(p_user_id UUID) RETURNS TABLE (...);

-- Calculate total comp time remaining
CREATE FUNCTION get_total_comp_time_hours(p_user_id UUID) RETURNS NUMERIC;

-- Log an activity
CREATE FUNCTION log_activity(...) RETURNS UUID;

-- Check for brute force attacks
CREATE FUNCTION check_brute_force(p_email VARCHAR) RETURNS BOOLEAN;
```

---

## 6. Triggers

```sql
-- Queue calendar sync when leave status changes
CREATE TRIGGER trg_queue_calendar_sync
    AFTER INSERT OR UPDATE OF status ON leave_requests
    FOR EACH ROW
    EXECUTE FUNCTION queue_calendar_sync_on_leave_change();
```

---

*Next: [API Specification](./05_API_SPECIFICATION.md)*
