-- ============================================
-- Migration 001: AUP System Schema
-- 
-- 包含：
-- - 所有自訂類型 (Custom Types)
-- - 用戶與權限相關表
-- - ERP 基礎架構表
-- - AUP 審查系統表
-- - 實驗動物管理系統表
-- - 通知與報表系統表
-- - 所有視圖 (Views)
-- ============================================

-- ============================================
-- 1. 自訂類型 (Custom Types)
-- ============================================

-- 夥伴類型
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');

-- 供應商類別
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');

-- 單據類型
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RTN');

-- 單據狀態
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');

-- 庫存流水方向
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

-- 計畫中角色類型
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');

-- 計畫狀態類型
CREATE TYPE protocol_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'PRE_REVIEW',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'DEFERRED',
    'REJECTED',
    'SUSPENDED',
    'CLOSED',
    'DELETED'
);

-- 豬隻狀態類型
CREATE TYPE pig_status AS ENUM ('unassigned', 'assigned', 'in_experiment', 'completed', 'transferred', 'deceased');

-- 豬隻品種類型
CREATE TYPE pig_breed AS ENUM ('miniature', 'white', 'LYD', 'other');

-- 豬隻性別類型
CREATE TYPE pig_gender AS ENUM ('male', 'female');

-- 觀察紀錄類型
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');

-- 豬隻紀錄類型
CREATE TYPE pig_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology');

-- 檔案類型
CREATE TYPE pig_file_type AS ENUM ('photo', 'attachment', 'report');

-- 獸醫紀錄類型
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');

-- 照護紀錄模式
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');

-- 版本紀錄類型
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology');

-- 通知類型
CREATE TYPE notification_type AS ENUM (
    'low_stock',
    'expiry_warning',
    'document_approval',
    'protocol_status',
    'vet_recommendation',
    'system_alert',
    'monthly_report'
);

-- 排程類型
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');

-- 報表類型
CREATE TYPE report_type AS ENUM (
    'stock_on_hand',
    'stock_ledger',
    'purchase_summary',
    'cost_summary',
    'expiry_report',
    'low_stock_report'
);

-- ============================================
-- 2. 用戶與權限相關表
-- ============================================

-- 用戶表
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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_theme_preference CHECK (theme_preference IN ('light', 'dark', 'system')),
    CONSTRAINT chk_language_preference CHECK (language_preference IN ('zh-TW', 'en'))
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- 角色表
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_internal BOOLEAN NOT NULL DEFAULT true,
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 權限表
CREATE TABLE permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    module VARCHAR(50),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 角色權限關聯表
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- 用戶角色關聯表
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, role_id)
);

-- Refresh Token 表
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- 密碼重設 Token 表
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);

-- ============================================
-- 3. 基礎資料表 (Master Data)
-- ============================================

-- 倉庫表
CREATE TABLE warehouses (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    address TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_warehouses_code ON warehouses(code);
CREATE INDEX idx_warehouses_is_active ON warehouses(is_active);

-- 產品類別表
CREATE TABLE product_categories (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES product_categories(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SKU 主類別
CREATE TABLE sku_categories (
    code CHAR(3) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SKU 子類別
CREATE TABLE sku_subcategories (
    id SERIAL PRIMARY KEY,
    category_code CHAR(3) NOT NULL REFERENCES sku_categories(code) ON DELETE CASCADE,
    code CHAR(3) NOT NULL,
    name VARCHAR(50) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category_code, code)
);

-- SKU 流水號追蹤
CREATE TABLE sku_sequences (
    category_code CHAR(3) NOT NULL,
    subcategory_code CHAR(3) NOT NULL,
    last_sequence INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (category_code, subcategory_code)
);

-- 產品表
CREATE TABLE products (
    id UUID PRIMARY KEY,
    sku VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    spec TEXT,
    category_id UUID REFERENCES product_categories(id),
    category_code CHAR(3),
    subcategory_code CHAR(3),
    base_uom VARCHAR(20) NOT NULL DEFAULT 'pcs',
    pack_unit VARCHAR(20),
    pack_qty INTEGER,
    track_batch BOOLEAN NOT NULL DEFAULT false,
    track_expiry BOOLEAN NOT NULL DEFAULT false,
    default_expiry_days INTEGER,
    safety_stock NUMERIC(18, 4),
    safety_stock_uom VARCHAR(20),
    reorder_point NUMERIC(18, 4),
    reorder_point_uom VARCHAR(20),
    image_url VARCHAR(500),
    license_no VARCHAR(100),
    storage_condition VARCHAR(50),
    barcode VARCHAR(50),
    tags TEXT[],
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    remark TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_product_status CHECK (status IN ('active', 'inactive', 'discontinued'))
);

CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_products_category_code ON products(category_code);
CREATE INDEX idx_products_subcategory_code ON products(subcategory_code);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_barcode ON products(barcode);
CREATE INDEX idx_products_is_active ON products(is_active);

-- 產品單位換算表
CREATE TABLE product_uom_conversions (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    uom VARCHAR(20) NOT NULL,
    factor_to_base NUMERIC(18, 6) NOT NULL,
    UNIQUE (product_id, uom)
);

CREATE INDEX idx_product_uom_conversions_product_id ON product_uom_conversions(product_id);

-- 夥伴表 (供應商/客戶)
CREATE TABLE partners (
    id UUID PRIMARY KEY,
    partner_type partner_type NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    supplier_category supplier_category,
    tax_id VARCHAR(50),
    phone VARCHAR(50),
    email VARCHAR(255),
    address TEXT,
    payment_terms VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_partners_code ON partners(code);
CREATE INDEX idx_partners_partner_type ON partners(partner_type);
CREATE INDEX idx_partners_is_active ON partners(is_active);

-- ============================================
-- 4. 單據相關表
-- ============================================

-- 單據頭表
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    doc_type doc_type NOT NULL,
    doc_no VARCHAR(50) NOT NULL UNIQUE,
    status doc_status NOT NULL DEFAULT 'draft',
    warehouse_id UUID REFERENCES warehouses(id),
    warehouse_from_id UUID REFERENCES warehouses(id),
    warehouse_to_id UUID REFERENCES warehouses(id),
    partner_id UUID REFERENCES partners(id),
    source_doc_id UUID REFERENCES documents(id),
    doc_date DATE NOT NULL,
    receipt_status VARCHAR(20),
    stocktake_scope JSONB,
    remark TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    CONSTRAINT chk_receipt_status CHECK (receipt_status IS NULL OR receipt_status IN ('pending', 'partial', 'complete'))
);

CREATE INDEX idx_documents_doc_type ON documents(doc_type);
CREATE INDEX idx_documents_doc_no ON documents(doc_no);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_doc_date ON documents(doc_date);
CREATE INDEX idx_documents_warehouse_id ON documents(warehouse_id);
CREATE INDEX idx_documents_partner_id ON documents(partner_id);
CREATE INDEX idx_documents_created_by ON documents(created_by);
CREATE INDEX idx_documents_source_doc_id ON documents(source_doc_id);

-- 單據明細表
CREATE TABLE document_lines (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    line_no INTEGER NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id),
    qty NUMERIC(18, 4) NOT NULL,
    uom VARCHAR(20) NOT NULL,
    unit_price NUMERIC(18, 4),
    batch_no VARCHAR(50),
    expiry_date DATE,
    remark TEXT,
    UNIQUE (document_id, line_no)
);

CREATE INDEX idx_document_lines_document_id ON document_lines(document_id);
CREATE INDEX idx_document_lines_product_id ON document_lines(product_id);

-- ============================================
-- 5. 庫存相關表
-- ============================================

-- 庫存流水表
CREATE TABLE stock_ledger (
    id UUID PRIMARY KEY,
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    product_id UUID NOT NULL REFERENCES products(id),
    trx_date TIMESTAMPTZ NOT NULL,
    doc_type doc_type NOT NULL,
    doc_id UUID NOT NULL REFERENCES documents(id),
    doc_no VARCHAR(50) NOT NULL,
    line_id UUID REFERENCES document_lines(id),
    direction stock_direction NOT NULL,
    qty_base NUMERIC(18, 4) NOT NULL,
    unit_cost NUMERIC(18, 4),
    batch_no VARCHAR(50),
    expiry_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stock_ledger_warehouse_product ON stock_ledger(warehouse_id, product_id);
CREATE INDEX idx_stock_ledger_trx_date ON stock_ledger(trx_date);
CREATE INDEX idx_stock_ledger_doc_id ON stock_ledger(doc_id);
CREATE INDEX idx_stock_ledger_product_id ON stock_ledger(product_id);

-- 庫存快照表
CREATE TABLE inventory_snapshots (
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    product_id UUID NOT NULL REFERENCES products(id),
    on_hand_qty_base NUMERIC(18, 4) NOT NULL DEFAULT 0,
    avg_cost NUMERIC(18, 4),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (warehouse_id, product_id)
);

-- ============================================
-- 6. 稽核日誌表
-- ============================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    actor_user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    before_data JSONB,
    after_data JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- ============================================
-- 7. AUP 審查系統表
-- ============================================

-- 計畫書主表
CREATE TABLE protocols (
    id UUID PRIMARY KEY,
    protocol_no VARCHAR(50) NOT NULL UNIQUE,
    iacuc_no VARCHAR(50) UNIQUE,
    title VARCHAR(500) NOT NULL,
    status protocol_status NOT NULL DEFAULT 'DRAFT',
    pi_user_id UUID NOT NULL REFERENCES users(id),
    working_content JSONB,
    start_date DATE,
    end_date DATE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocols_status ON protocols(status);
CREATE INDEX idx_protocols_pi_user_id ON protocols(pi_user_id);
CREATE INDEX idx_protocols_iacuc_no ON protocols(iacuc_no);

-- 用戶計畫關聯表
CREATE TABLE user_protocols (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    role_in_protocol protocol_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, protocol_id)
);

CREATE INDEX idx_user_protocols_user_id ON user_protocols(user_id);
CREATE INDEX idx_user_protocols_protocol_id ON user_protocols(protocol_id);

-- 計畫版本快照
CREATE TABLE protocol_versions (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (protocol_id, version_no)
);

CREATE INDEX idx_protocol_versions_protocol_id ON protocol_versions(protocol_id);

-- 計畫狀態歷程
CREATE TABLE protocol_status_history (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    from_status protocol_status,
    to_status protocol_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocol_status_history_protocol_id ON protocol_status_history(protocol_id);

-- 審查人員指派
CREATE TABLE review_assignments (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    UNIQUE (protocol_id, reviewer_id)
);

CREATE INDEX idx_review_assignments_protocol_id ON review_assignments(protocol_id);
CREATE INDEX idx_review_assignments_reviewer_id ON review_assignments(reviewer_id);

-- 審查意見
CREATE TABLE review_comments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID NOT NULL REFERENCES protocol_versions(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_review_comments_protocol_version_id ON review_comments(protocol_version_id);
CREATE INDEX idx_review_comments_reviewer_id ON review_comments(reviewer_id);

-- 計畫附件表
CREATE TABLE protocol_attachments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID REFERENCES protocol_versions(id) ON DELETE CASCADE,
    protocol_id UUID REFERENCES protocols(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocol_attachments_protocol_id ON protocol_attachments(protocol_id);
CREATE INDEX idx_protocol_attachments_protocol_version_id ON protocol_attachments(protocol_version_id);

-- ============================================
-- 8. 實驗動物管理系統表
-- ============================================

-- 豬隻來源表
CREATE TABLE pig_sources (
    id UUID PRIMARY KEY,
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    address TEXT,
    contact VARCHAR(100),
    phone VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 豬隻主表
CREATE TABLE pigs (
    id SERIAL PRIMARY KEY,
    ear_tag VARCHAR(10) NOT NULL,
    status pig_status NOT NULL DEFAULT 'unassigned',
    breed pig_breed NOT NULL,
    source_id UUID REFERENCES pig_sources(id),
    gender pig_gender NOT NULL,
    birth_date DATE,
    entry_date DATE NOT NULL,
    entry_weight NUMERIC(5, 1),
    pen_location VARCHAR(10),
    pre_experiment_code VARCHAR(20),
    iacuc_no VARCHAR(20),
    experiment_date DATE,
    remark TEXT,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    vet_weight_viewed_at TIMESTAMPTZ,
    vet_vaccine_viewed_at TIMESTAMPTZ,
    vet_sacrifice_viewed_at TIMESTAMPTZ,
    vet_last_viewed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pigs_ear_tag ON pigs(ear_tag);
CREATE INDEX idx_pigs_status ON pigs(status);
CREATE INDEX idx_pigs_iacuc_no ON pigs(iacuc_no);
CREATE INDEX idx_pigs_pen_location ON pigs(pen_location);
CREATE INDEX idx_pigs_is_deleted ON pigs(is_deleted);

-- 觀察試驗紀錄表
CREATE TABLE pig_observations (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    event_date DATE NOT NULL,
    record_type record_type NOT NULL,
    equipment_used JSONB,
    anesthesia_start TIMESTAMPTZ,
    anesthesia_end TIMESTAMPTZ,
    content TEXT NOT NULL,
    no_medication_needed BOOLEAN NOT NULL DEFAULT false,
    stop_medication BOOLEAN NOT NULL DEFAULT false,
    treatments JSONB,
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_observations_pig_id ON pig_observations(pig_id);
CREATE INDEX idx_pig_observations_event_date ON pig_observations(event_date);

-- 手術紀錄表
CREATE TABLE pig_surgeries (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    is_first_experiment BOOLEAN NOT NULL DEFAULT true,
    surgery_date DATE NOT NULL,
    surgery_site VARCHAR(200) NOT NULL,
    induction_anesthesia JSONB,
    pre_surgery_medication JSONB,
    positioning VARCHAR(100),
    anesthesia_maintenance JSONB,
    anesthesia_observation TEXT,
    vital_signs JSONB,
    reflex_recovery TEXT,
    respiration_rate INTEGER,
    post_surgery_medication JSONB,
    remark TEXT,
    no_medication_needed BOOLEAN NOT NULL DEFAULT false,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_surgeries_pig_id ON pig_surgeries(pig_id);
CREATE INDEX idx_pig_surgeries_surgery_date ON pig_surgeries(surgery_date);

-- 體重紀錄表
CREATE TABLE pig_weights (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    measure_date DATE NOT NULL,
    weight NUMERIC(5, 1) NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_weights_pig_id ON pig_weights(pig_id);
CREATE INDEX idx_pig_weights_measure_date ON pig_weights(measure_date);

-- 疫苗/驅蟲紀錄表
CREATE TABLE pig_vaccinations (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    administered_date DATE NOT NULL,
    vaccine VARCHAR(100),
    deworming_dose VARCHAR(100),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_vaccinations_pig_id ON pig_vaccinations(pig_id);

-- 犧牲/採樣紀錄表
CREATE TABLE pig_sacrifices (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
    sacrifice_date DATE,
    zoletil_dose VARCHAR(50),
    method_electrocution BOOLEAN NOT NULL DEFAULT false,
    method_bloodletting BOOLEAN NOT NULL DEFAULT false,
    method_other TEXT,
    sampling TEXT,
    sampling_other TEXT,
    blood_volume_ml NUMERIC(6, 1),
    confirmed_sacrifice BOOLEAN NOT NULL DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_sacrifices_pig_id ON pig_sacrifices(pig_id);

-- 病理組織報告表
CREATE TABLE pig_pathology_reports (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 紀錄附件通用表
CREATE TABLE pig_record_attachments (
    id UUID PRIMARY KEY,
    record_type pig_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    file_type pig_file_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_record_attachments_record ON pig_record_attachments(record_type, record_id);

-- 獸醫師建議表
CREATE TABLE vet_recommendations (
    id SERIAL PRIMARY KEY,
    record_type vet_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vet_recommendations_record ON vet_recommendations(record_type, record_id);

-- 照護給藥紀錄表
CREATE TABLE care_medication_records (
    id SERIAL PRIMARY KEY,
    record_type vet_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    record_mode care_record_mode NOT NULL DEFAULT 'pain_assessment',
    post_op_days INTEGER,
    time_period VARCHAR(20),
    spirit VARCHAR(50),
    appetite VARCHAR(50),
    mobility_standing VARCHAR(50),
    mobility_walking VARCHAR(50),
    attitude_behavior VARCHAR(50),
    vet_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_care_medication_records_record ON care_medication_records(record_type, record_id);

-- 紀錄版本歷史表
CREATE TABLE record_versions (
    id SERIAL PRIMARY KEY,
    record_type version_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    version_no INTEGER NOT NULL,
    snapshot JSONB NOT NULL,
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_record_versions_record ON record_versions(record_type, record_id);

-- ============================================
-- 9. 通知系統表
-- ============================================

-- 通知表
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type notification_type NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT,
    is_read BOOLEAN NOT NULL DEFAULT false,
    read_at TIMESTAMPTZ,
    related_entity_type VARCHAR(50),
    related_entity_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_is_read ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);

-- 通知設定表
CREATE TABLE notification_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_low_stock BOOLEAN NOT NULL DEFAULT true,
    email_expiry_warning BOOLEAN NOT NULL DEFAULT true,
    email_document_approval BOOLEAN NOT NULL DEFAULT true,
    email_protocol_status BOOLEAN NOT NULL DEFAULT true,
    email_monthly_report BOOLEAN NOT NULL DEFAULT true,
    expiry_warning_days INTEGER NOT NULL DEFAULT 30,
    low_stock_notify_immediately BOOLEAN NOT NULL DEFAULT true,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 10. 定期報表系統表
-- ============================================

-- 定期報表設定
CREATE TABLE scheduled_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type report_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    day_of_week INTEGER,
    day_of_month INTEGER,
    hour_of_day INTEGER NOT NULL DEFAULT 6,
    parameters JSONB,
    recipients UUID[] NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scheduled_reports_next_run ON scheduled_reports(next_run_at) WHERE is_active = true;

-- 報表歷史記錄
CREATE TABLE report_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scheduled_report_id UUID REFERENCES scheduled_reports(id) ON DELETE SET NULL,
    report_type report_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER,
    parameters JSONB,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generated_by UUID REFERENCES users(id)
);

CREATE INDEX idx_report_history_type ON report_history(report_type);
CREATE INDEX idx_report_history_generated_at ON report_history(generated_at);

-- ============================================
-- 11. 通用附件表
-- ============================================

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(50) NOT NULL,
    entity_id UUID,
    entity_type VARCHAR(50),
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_category ON attachments(category);
CREATE INDEX idx_attachments_entity ON attachments(entity_type, entity_id);

-- ============================================
-- 12. 視圖 (Views)
-- ============================================

-- 採購單入庫狀態視圖
CREATE OR REPLACE VIEW v_purchase_order_receipt_status AS
SELECT 
    po.id AS po_id,
    po.doc_no AS po_no,
    po.status AS po_status,
    po.partner_id,
    po.warehouse_id,
    po.doc_date AS po_date,
    COALESCE(SUM(pol.qty), 0) AS ordered_qty,
    COALESCE(SUM(grnl.received_qty), 0) AS received_qty,
    CASE 
        WHEN COALESCE(SUM(grnl.received_qty), 0) = 0 THEN 'pending'
        WHEN COALESCE(SUM(grnl.received_qty), 0) < COALESCE(SUM(pol.qty), 0) THEN 'partial'
        ELSE 'complete'
    END AS receipt_status
FROM documents po
LEFT JOIN document_lines pol ON po.id = pol.document_id
LEFT JOIN (
    SELECT 
        grn.source_doc_id,
        grnl.product_id,
        SUM(grnl.qty) AS received_qty
    FROM documents grn
    JOIN document_lines grnl ON grn.id = grnl.document_id
    WHERE grn.doc_type = 'GRN' AND grn.status = 'approved'
    GROUP BY grn.source_doc_id, grnl.product_id
) grnl ON po.id = grnl.source_doc_id AND pol.product_id = grnl.product_id
WHERE po.doc_type = 'PO'
GROUP BY po.id, po.doc_no, po.status, po.partner_id, po.warehouse_id, po.doc_date;

-- 低庫存預警視圖
CREATE OR REPLACE VIEW v_low_stock_alerts AS
SELECT 
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.spec,
    p.category_code,
    p.safety_stock,
    p.safety_stock_uom,
    p.reorder_point,
    p.reorder_point_uom,
    w.id AS warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    COALESCE(inv.on_hand_qty_base, 0) AS on_hand_qty,
    p.base_uom,
    CASE 
        WHEN COALESCE(inv.on_hand_qty_base, 0) <= 0 THEN 'out_of_stock'
        WHEN p.safety_stock IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.safety_stock THEN 'below_safety'
        WHEN p.reorder_point IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.reorder_point THEN 'below_reorder'
        ELSE 'normal'
    END AS stock_status
FROM products p
CROSS JOIN warehouses w
LEFT JOIN inventory_snapshots inv ON p.id = inv.product_id AND w.id = inv.warehouse_id
WHERE p.is_active = true AND w.is_active = true
  AND (
    COALESCE(inv.on_hand_qty_base, 0) <= 0
    OR (p.safety_stock IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.safety_stock)
    OR (p.reorder_point IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.reorder_point)
  );

-- 效期預警視圖
CREATE OR REPLACE VIEW v_expiry_alerts AS
SELECT 
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.spec,
    p.category_code,
    sl.warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    sl.batch_no,
    sl.expiry_date,
    SUM(CASE 
        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base 
        ELSE -sl.qty_base 
    END) AS on_hand_qty,
    p.base_uom,
    sl.expiry_date - CURRENT_DATE AS days_until_expiry,
    CASE 
        WHEN sl.expiry_date < CURRENT_DATE THEN 'expired'
        WHEN sl.expiry_date <= CURRENT_DATE + 30 THEN 'expiring_soon'
        WHEN sl.expiry_date <= CURRENT_DATE + 60 THEN 'expiring_60days'
        ELSE 'normal'
    END AS expiry_status
FROM stock_ledger sl
JOIN products p ON sl.product_id = p.id
JOIN warehouses w ON sl.warehouse_id = w.id
WHERE p.track_expiry = true 
  AND sl.expiry_date IS NOT NULL
  AND p.is_active = true
GROUP BY p.id, p.sku, p.name, p.spec, p.category_code, 
         sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date, p.base_uom
HAVING SUM(CASE 
    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base 
    ELSE -sl.qty_base 
END) > 0
  AND sl.expiry_date <= CURRENT_DATE + 60;

-- ============================================
-- 13. 觸發器 (Triggers)
-- ============================================

-- 新使用者建立時自動建立通知設定
CREATE OR REPLACE FUNCTION create_default_notification_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO notification_settings (user_id)
    VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_create_notification_settings
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_notification_settings();

-- ============================================
-- 14. 插入基礎角色定義
-- ============================================

INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'admin', '系統管理員', '全系統最高權限，使用者管理、系統維運', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'WAREHOUSE_MANAGER', '倉庫管理員', '專責 ERP 進銷存系統（採購、庫存、盤點、報表）', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PURCHASING', '採購人員', '負責採購作業、建立採購單、管理供應商', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PI', '計畫主持人', '提交計畫、管理自己的計畫與豬隻', false, true, NOW(), NOW()),
    (gen_random_uuid(), 'VET', '獸醫師', '審查計畫、豬隻健康管理、提供建議', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'REVIEWER', '審查委員', 'IACUC 計畫審查', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'CHAIR', 'IACUC 主席', '主導審查決策', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_STAFF', '執行秘書', '行政流程管理、管理所有計劃進度', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'EXPERIMENT_STAFF', '試驗工作人員', '執行實驗操作、記錄數據、查詢 ERP 物資現況', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'CLIENT', '委託人', '查看委託計畫與豬隻紀錄', false, true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 15. 插入權限定義
-- ============================================

-- 系統管理權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'admin.user.view', '查看使用者', 'admin', '可查看使用者列表', NOW()),
    (gen_random_uuid(), 'admin.user.create', '建立使用者', 'admin', '可建立新使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.edit', '編輯使用者', 'admin', '可編輯使用者資料', NOW()),
    (gen_random_uuid(), 'admin.user.delete', '停用使用者', 'admin', '可停用使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.reset_password', '重設密碼', 'admin', '可重設他人密碼', NOW()),
    (gen_random_uuid(), 'admin.role.manage', '管理角色', 'admin', '可管理角色定義', NOW()),
    (gen_random_uuid(), 'admin.permission.manage', '管理權限', 'admin', '可管理權限定義', NOW()),
    (gen_random_uuid(), 'admin.audit.view', '查看稽核紀錄', 'admin', '可查看系統稽核紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- AUP 系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.protocol.view_all', '查看所有計畫', 'aup', '可查看系統中所有計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.view_own', '查看自己的計畫', 'aup', '可查看自己相關的計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.create', '建立計畫', 'aup', '可建立新計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.edit', '編輯計畫', 'aup', '可編輯計畫草稿', NOW()),
    (gen_random_uuid(), 'aup.protocol.submit', '提交計畫', 'aup', '可提交計畫送審', NOW()),
    (gen_random_uuid(), 'aup.protocol.review', '審查計畫', 'aup', '可審查計畫並提供意見', NOW()),
    (gen_random_uuid(), 'aup.protocol.approve', '核准/否決', 'aup', '可核准或否決計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.change_status', '變更狀態', 'aup', '可變更計畫狀態', NOW()),
    (gen_random_uuid(), 'aup.protocol.delete', '刪除計畫', 'aup', '可刪除計畫', NOW()),
    (gen_random_uuid(), 'aup.review.view', '查看審查', 'aup', '可查看審查意見', NOW()),
    (gen_random_uuid(), 'aup.review.assign', '指派審查人員', 'aup', '可指派審查人員', NOW()),
    (gen_random_uuid(), 'aup.review.comment', '新增審查意見', 'aup', '可新增審查意見', NOW()),
    (gen_random_uuid(), 'aup.attachment.view', '查看附件', 'aup', '可查看計畫附件', NOW()),
    (gen_random_uuid(), 'aup.attachment.download', '下載附件', 'aup', '可下載計畫附件', NOW()),
    (gen_random_uuid(), 'aup.version.view', '查看版本', 'aup', '可查看計畫版本歷史', NOW())
ON CONFLICT (code) DO NOTHING;

-- 實驗動物管理系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.pig.view_all', '查看所有豬隻', 'pig', '可查看所有豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.pig.view_project', '查看計畫內豬隻', 'pig', '可查看計畫內的豬隻', NOW()),
    (gen_random_uuid(), 'pig.pig.create', '新增豬隻', 'pig', '可新增豬隻', NOW()),
    (gen_random_uuid(), 'pig.pig.edit', '編輯豬隻資料', 'pig', '可編輯豬隻基本資料', NOW()),
    (gen_random_uuid(), 'pig.pig.assign', '分配豬隻至計畫', 'pig', '可將豬隻分配至計畫', NOW()),
    (gen_random_uuid(), 'pig.pig.import', '匯入豬隻資料', 'pig', '可批次匯入豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.pig.delete', '刪除豬隻', 'pig', '可刪除豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.record.view', '查看紀錄', 'pig', '可查看豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.create', '新增紀錄', 'pig', '可新增豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.edit', '編輯紀錄', 'pig', '可編輯豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.delete', '刪除紀錄', 'pig', '可刪除豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.observation', '新增觀察紀錄', 'pig', '可新增觀察紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.surgery', '新增手術紀錄', 'pig', '可新增手術紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.weight', '新增體重紀錄', 'pig', '可新增體重紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.vaccine', '新增疫苗紀錄', 'pig', '可新增疫苗紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.sacrifice', '新增犧牲紀錄', 'pig', '可新增犧牲紀錄', NOW()),
    (gen_random_uuid(), 'pig.vet.recommend', '新增獸醫師建議', 'pig', '可新增獸醫師建議', NOW()),
    (gen_random_uuid(), 'pig.vet.read', '標記獸醫師已讀', 'pig', '可標記紀錄已讀', NOW()),
    (gen_random_uuid(), 'pig.export.medical', '匯出病歷', 'pig', '可匯出豬隻病歷', NOW()),
    (gen_random_uuid(), 'pig.export.observation', '匯出觀察紀錄', 'pig', '可匯出觀察紀錄', NOW()),
    (gen_random_uuid(), 'pig.export.surgery', '匯出手術紀錄', 'pig', '可匯出手術紀錄', NOW()),
    (gen_random_uuid(), 'pig.export.experiment', '匯出實驗紀錄', 'pig', '可匯出實驗紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- ERP 系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'erp.warehouse.view', '查看倉庫', 'erp', '可查看倉庫資料', NOW()),
    (gen_random_uuid(), 'erp.warehouse.create', '建立倉庫', 'erp', '可建立倉庫', NOW()),
    (gen_random_uuid(), 'erp.warehouse.edit', '編輯倉庫', 'erp', '可編輯倉庫', NOW()),
    (gen_random_uuid(), 'erp.product.view', '查看產品', 'erp', '可查看產品資料', NOW()),
    (gen_random_uuid(), 'erp.product.create', '建立產品', 'erp', '可建立產品', NOW()),
    (gen_random_uuid(), 'erp.product.edit', '編輯產品', 'erp', '可編輯產品', NOW()),
    (gen_random_uuid(), 'erp.partner.view', '查看夥伴', 'erp', '可查看夥伴資料', NOW()),
    (gen_random_uuid(), 'erp.partner.create', '建立夥伴', 'erp', '可建立夥伴', NOW()),
    (gen_random_uuid(), 'erp.partner.edit', '編輯夥伴', 'erp', '可編輯夥伴', NOW()),
    (gen_random_uuid(), 'erp.document.view', '查看單據', 'erp', '可查看單據', NOW()),
    (gen_random_uuid(), 'erp.document.create', '建立單據', 'erp', '可建立單據', NOW()),
    (gen_random_uuid(), 'erp.document.edit', '編輯單據', 'erp', '可編輯單據', NOW()),
    (gen_random_uuid(), 'erp.document.submit', '送審單據', 'erp', '可送審單據', NOW()),
    (gen_random_uuid(), 'erp.document.approve', '核准單據', 'erp', '可核准單據', NOW()),
    (gen_random_uuid(), 'erp.inventory.view', '查看庫存', 'erp', '可查看庫存現況', NOW()),
    (gen_random_uuid(), 'erp.purchase.create', '建立採購單', 'erp', '可建立採購單', NOW()),
    (gen_random_uuid(), 'erp.purchase.approve', '核准採購單', 'erp', '可核准採購單', NOW()),
    (gen_random_uuid(), 'erp.grn.create', '建立進貨單', 'erp', '可建立進貨單', NOW()),
    (gen_random_uuid(), 'erp.pr.create', '建立採購退貨', 'erp', '可建立採購退貨', NOW()),
    (gen_random_uuid(), 'erp.stock.in', '入庫操作', 'erp', '可執行入庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.out', '出庫操作', 'erp', '可執行出庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.view', '查看庫存', 'erp', '可查看庫存', NOW()),
    (gen_random_uuid(), 'erp.stock.adjust', '庫存調整', 'erp', '可執行庫存調整', NOW()),
    (gen_random_uuid(), 'erp.stock.transfer', '調撥', 'erp', '可執行庫存調撥', NOW()),
    (gen_random_uuid(), 'erp.stocktake.create', '盤點', 'erp', '可執行庫存盤點', NOW()),
    (gen_random_uuid(), 'erp.report.view', '查看報表', 'erp', '可查看 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.export', '匯出報表', 'erp', '可匯出 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.download', '下載報表', 'erp', '可下載報表', NOW())
ON CONFLICT (code) DO NOTHING;

-- 通知系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'notification.view', '查看通知', 'notification', '可查看自己的通知', NOW()),
    (gen_random_uuid(), 'notification.manage', '管理通知設定', 'notification', '可管理通知設定', NOW()),
    (gen_random_uuid(), 'notification.send', '發送通知', 'notification', '可發送系統通知', NOW())
ON CONFLICT (code) DO NOTHING;

-- 報表系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'report.schedule', '排程報表', 'report', '可設定定期報表', NOW()),
    (gen_random_uuid(), 'report.download', '下載報表', 'report', '可下載報表檔案', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 完成
-- ============================================
