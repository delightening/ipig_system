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
    (gen_random_uuid(), 'animal.animal.view_all', '查看所有動物', 'animal', '可查看所有動物資料', NOW()),
    (gen_random_uuid(), 'animal.animal.view_project', '查看計畫內動物', 'animal', '可查看計畫內的動物', NOW()),
    (gen_random_uuid(), 'animal.animal.create', '新增動物', 'animal', '可新增動物', NOW()),
    (gen_random_uuid(), 'animal.animal.edit', '編輯動物資料', 'animal', '可編輯動物基本資料', NOW()),
    (gen_random_uuid(), 'animal.animal.assign', '分配動物至計畫', 'animal', '可將動物分配至計畫', NOW()),
    (gen_random_uuid(), 'animal.animal.import', '匯入動物資料', 'animal', '可批次匯入動物資料', NOW()),
    (gen_random_uuid(), 'animal.animal.delete', '刪除動物', 'animal', '可刪除動物資料', NOW()),
    (gen_random_uuid(), 'animal.record.view', '查看紀錄', 'animal', '可查看動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.create', '新增紀錄', 'animal', '可新增動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.edit', '編輯紀錄', 'animal', '可編輯動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.delete', '刪除紀錄', 'animal', '可刪除動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.observation', '新增觀察紀錄', 'animal', '可新增觀察紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.surgery', '新增手術紀錄', 'animal', '可新增手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.weight', '新增體重紀錄', 'animal', '可新增體重紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.vaccine', '新增疫苗紀錄', 'animal', '可新增疫苗紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.sacrifice', '新增犧牲紀錄', 'animal', '可新增犧牲紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.recommend', '新增獸醫師建議', 'animal', '可新增獸醫師建議', NOW()),
    (gen_random_uuid(), 'animal.vet.read', '標記獸醫師已讀', 'animal', '可標記紀錄已讀', NOW()),
    (gen_random_uuid(), 'animal.export.medical', '匯出病歷', 'animal', '可匯出動物病歷', NOW()),
    (gen_random_uuid(), 'animal.export.observation', '匯出觀察紀錄', 'animal', '可匯出觀察紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.surgery', '匯出手術紀錄', 'animal', '可匯出手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.experiment', '匯出實驗紀錄', 'animal', '可匯出實驗紀錄', NOW())
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
-- ============================================
-- Migration 004: HR System
-- 
-- ?嚗?-- - 隢?憿?????-- - ?箏?蝝??-- - ?蝝??-- - 撟游?憿漲
-- - 鋆?擗?
-- - 隢??唾?
-- - 隢?撖拇蝝??-- - ?賊??賢?
-- ============================================

-- ============================================
-- 1. 隢?憿???
-- ============================================

CREATE TYPE leave_type AS ENUM (
    'ANNUAL',           -- ?嫣???    'PERSONAL',         -- 鈭?
    'SICK',             -- ??
    'COMPENSATORY',     -- 鋆???    'MARRIAGE',         -- 憍?
    'BEREAVEMENT',      -- ?芸?
    'MATERNITY',        -- ?Ｗ?
    'PATERNITY',        -- ?芰??    'MENSTRUAL',        -- ????    'OFFICIAL',         -- ?砍?
    'UNPAID'            -- ?∟??);

-- ============================================
-- 2. 隢??????-- ============================================

CREATE TYPE leave_status AS ENUM (
    'DRAFT',            -- ?阮
    'PENDING_L1',       -- 敺?蝝祟?賂??游惇銝餌恣嚗?    'PENDING_L2',       -- 敺?蝝祟?賂??券?銝餌恣嚗?    'PENDING_HR',       -- 敺??踹祟??    'PENDING_GM',       -- 敺蜇蝬??詨?
    'APPROVED',         -- 撌脫??    'REJECTED',         -- 撌脤???    'CANCELLED',        -- 撌脣?瘨?    'REVOKED'           -- 撌脤??);

-- ============================================
-- 3. ?箏?蝝?”
-- ============================================

CREATE TABLE attendance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Date and time
    work_date DATE NOT NULL,
    clock_in_time TIMESTAMPTZ,
    clock_out_time TIMESTAMPTZ,
    
    -- Calculated fields
    regular_hours NUMERIC(5,2) DEFAULT 0,
    overtime_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Status
    status VARCHAR(20) DEFAULT 'normal', -- 'normal', 'late', 'early_leave', 'absent', 'leave', 'holiday'
    
    -- Source tracking
    clock_in_source VARCHAR(20), -- 'web', 'mobile', 'import', 'manual'
    clock_in_ip INET,
    clock_out_source VARCHAR(20),
    clock_out_ip INET,
    
    -- Notes
    remark TEXT,
    
    -- Approval for manual corrections
    is_corrected BOOLEAN DEFAULT false,
    corrected_by UUID REFERENCES users(id),
    corrected_at TIMESTAMPTZ,
    correction_reason TEXT,
    original_clock_in TIMESTAMPTZ,
    original_clock_out TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, work_date)
);

CREATE INDEX idx_attendance_user_date ON attendance_records(user_id, work_date DESC);
CREATE INDEX idx_attendance_date ON attendance_records(work_date DESC);
CREATE INDEX idx_attendance_status ON attendance_records(status);

-- ============================================
-- 4. ?蝝?”
-- ============================================

CREATE TABLE overtime_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    attendance_id UUID REFERENCES attendance_records(id),
    
    -- When
    overtime_date DATE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    
    -- Duration
    hours NUMERIC(5,2) NOT NULL,
    
    -- Type and multiplier
    overtime_type VARCHAR(20) NOT NULL, -- 'weekday', 'weekend', 'holiday'
    multiplier NUMERIC(3,2) DEFAULT 1.0, -- 1.0, 1.33, 1.66, 2.0
    
    -- Comp time generation
    comp_time_hours NUMERIC(5,2) NOT NULL,
    comp_time_expires_at DATE NOT NULL,
    comp_time_used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Approval
    status VARCHAR(20) DEFAULT 'draft', -- 'draft', 'pending', 'approved', 'rejected'
    submitted_at TIMESTAMPTZ,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejected_by UUID REFERENCES users(id),
    rejected_at TIMESTAMPTZ,
    rejection_reason TEXT,
    
    -- Reason
    reason TEXT NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_overtime_user ON overtime_records(user_id, overtime_date DESC);
CREATE INDEX idx_overtime_status ON overtime_records(status);
CREATE INDEX idx_overtime_expires ON overtime_records(comp_time_expires_at) 
    WHERE status = 'approved' AND comp_time_used_hours < comp_time_hours;

-- ============================================
-- 5. 撟游?憿漲銵?-- ============================================

CREATE TABLE annual_leave_entitlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Entitlement period
    entitlement_year INTEGER NOT NULL,
    
    -- Days
    entitled_days NUMERIC(5,2) NOT NULL,
    used_days NUMERIC(5,2) DEFAULT 0,
    
    -- Expiration
    expires_at DATE NOT NULL,
    
    -- Source
    calculation_basis VARCHAR(50), -- 'seniority', 'prorated', 'manual', 'carry_forward'
    seniority_years NUMERIC(4,2),
    
    -- Status
    is_expired BOOLEAN DEFAULT false,
    expired_days NUMERIC(5,2) DEFAULT 0,
    expiry_processed_at TIMESTAMPTZ,
    
    -- Notes
    notes TEXT,
    adjustment_days NUMERIC(5,2) DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, entitlement_year)
);

CREATE INDEX idx_annual_leave_user ON annual_leave_entitlements(user_id, entitlement_year DESC);
CREATE INDEX idx_annual_leave_expires ON annual_leave_entitlements(expires_at) 
    WHERE NOT is_expired AND (entitled_days - used_days) > 0;

-- ============================================
-- 6. 鋆?擗?銵?-- ============================================

CREATE TABLE comp_time_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id) ON DELETE CASCADE,
    
    -- Hours
    original_hours NUMERIC(5,2) NOT NULL,
    used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Expiration
    earned_date DATE NOT NULL,
    expires_at DATE NOT NULL,
    
    -- Status
    is_expired BOOLEAN DEFAULT false,
    expired_hours NUMERIC(5,2) DEFAULT 0,
    converted_to_pay BOOLEAN DEFAULT false,
    expiry_processed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(overtime_record_id)
);

CREATE INDEX idx_comp_time_user ON comp_time_balances(user_id, earned_date DESC);
CREATE INDEX idx_comp_time_expires ON comp_time_balances(expires_at) 
    WHERE NOT is_expired AND (original_hours - used_hours) > 0;
CREATE INDEX idx_comp_time_fifo ON comp_time_balances(user_id, earned_date ASC) 
    WHERE NOT is_expired AND (original_hours - used_hours) > 0;

-- ============================================
-- 7. 隢??唾?銵?-- ============================================

CREATE TABLE leave_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    proxy_user_id UUID REFERENCES users(id),
    
    -- Leave details
    leave_type leave_type NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    
    -- Duration
    total_days NUMERIC(5,2) NOT NULL,
    total_hours NUMERIC(5,2),
    
    -- Reason and documents
    reason TEXT,
    supporting_documents JSONB DEFAULT '[]',
    
    -- For comp time usage
    comp_time_source_ids UUID[],
    
    -- For annual leave usage
    annual_leave_source_id UUID REFERENCES annual_leave_entitlements(id),
    
    -- Flags
    is_urgent BOOLEAN DEFAULT false,
    is_retroactive BOOLEAN DEFAULT false,
    
    -- Status
    status leave_status DEFAULT 'DRAFT',
    
    -- Approval chain
    current_approver_id UUID REFERENCES users(id),
    
    -- Timestamps
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    
    -- Notes
    cancellation_reason TEXT,
    revocation_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_leave_user ON leave_requests(user_id, start_date DESC);
CREATE INDEX idx_leave_status ON leave_requests(status);
CREATE INDEX idx_leave_date_range ON leave_requests(start_date, end_date);
CREATE INDEX idx_leave_approver ON leave_requests(current_approver_id) 
    WHERE status IN ('PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM');

-- ============================================
-- 8. 隢?撖拇蝝?”
-- ============================================

CREATE TABLE leave_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- Approver
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL,
    
    -- Action
    action VARCHAR(20) NOT NULL, -- 'APPROVE', 'REJECT', 'REQUEST_REVISION', 'ESCALATE'
    comments TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_approval_request ON leave_approvals(leave_request_id, created_at);
CREATE INDEX idx_approval_approver ON leave_approvals(approver_id, created_at DESC);

-- ============================================
-- 9. 隢?擗?雿輻蝝?”
-- ============================================

CREATE TABLE leave_balance_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id),
    
    -- Source
    source_type VARCHAR(20) NOT NULL, -- 'annual', 'comp_time'
    annual_leave_entitlement_id UUID REFERENCES annual_leave_entitlements(id),
    comp_time_balance_id UUID REFERENCES comp_time_balances(id),
    
    -- Amount used
    days_used NUMERIC(5,2),
    hours_used NUMERIC(5,2),
    
    -- Action
    action VARCHAR(20) NOT NULL, -- 'deduct', 'restore'
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_request ON leave_balance_usage(leave_request_id);
CREATE INDEX idx_usage_annual ON leave_balance_usage(annual_leave_entitlement_id);
CREATE INDEX idx_usage_comp ON leave_balance_usage(comp_time_balance_id);

-- ============================================
-- 10. Helper Functions
-- ============================================

-- Get remaining annual leave for user
CREATE OR REPLACE FUNCTION get_annual_leave_balance(p_user_id UUID) 
RETURNS TABLE (
    entitlement_year INTEGER,
    entitled_days NUMERIC,
    used_days NUMERIC,
    remaining_days NUMERIC,
    expires_at DATE,
    days_until_expiry INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ale.entitlement_year,
        ale.entitled_days,
        ale.used_days,
        (ale.entitled_days - ale.used_days) AS remaining_days,
        ale.expires_at,
        (ale.expires_at - CURRENT_DATE)::INTEGER AS days_until_expiry
    FROM annual_leave_entitlements ale
    WHERE ale.user_id = p_user_id
      AND NOT ale.is_expired
      AND (ale.entitled_days - ale.used_days) > 0
    ORDER BY ale.expires_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Get remaining comp time for user (FIFO order)
CREATE OR REPLACE FUNCTION get_comp_time_balance(p_user_id UUID) 
RETURNS TABLE (
    id UUID,
    earned_date DATE,
    original_hours NUMERIC,
    used_hours NUMERIC,
    remaining_hours NUMERIC,
    expires_at DATE,
    days_until_expiry INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ctb.id,
        ctb.earned_date,
        ctb.original_hours,
        ctb.used_hours,
        (ctb.original_hours - ctb.used_hours) AS remaining_hours,
        ctb.expires_at,
        (ctb.expires_at - CURRENT_DATE)::INTEGER AS days_until_expiry
    FROM comp_time_balances ctb
    WHERE ctb.user_id = p_user_id
      AND NOT ctb.is_expired
      AND (ctb.original_hours - ctb.used_hours) > 0
    ORDER BY ctb.earned_date ASC;
END;
$$ LANGUAGE plpgsql;

-- Calculate total comp time remaining
CREATE OR REPLACE FUNCTION get_total_comp_time_hours(p_user_id UUID) 
RETURNS NUMERIC AS $$
DECLARE
    v_total NUMERIC;
BEGIN
    SELECT COALESCE(SUM(original_hours - used_hours), 0) INTO v_total
    FROM comp_time_balances
    WHERE user_id = p_user_id
      AND NOT is_expired
      AND (original_hours - used_hours) > 0;
    
    RETURN v_total;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 11. HR ?賊?甈?
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'hr.attendance.view', '?亦??箏蝝??, 'hr', '?舀??斤???, NOW()),
    (gen_random_uuid(), 'hr.attendance.view_all', '?亦?????, 'hr', '?舀???犖??斤???, NOW()),
    (gen_random_uuid(), 'hr.attendance.clock', '?', 'hr', '?舫脰?銝??剜???, NOW()),
    (gen_random_uuid(), 'hr.attendance.correct', '?湔迤?', 'hr', '?舀甇???∠???, NOW()),
    (gen_random_uuid(), 'hr.overtime.view', '?亦??蝝??, 'hr', '?舀???剔???, NOW()),
    (gen_random_uuid(), 'hr.overtime.create', '?唾??', 'hr', '?舐隢???, NOW()),
    (gen_random_uuid(), 'hr.overtime.approve', '撖拇?', 'hr', '?臬祟?詨??剔隢?, NOW()),
    (gen_random_uuid(), 'hr.leave.view', '?亦?隢?', 'hr', '?舀??????, NOW()),
    (gen_random_uuid(), 'hr.leave.view_all', '?亦??????, 'hr', '?舀???犖??????, NOW()),
    (gen_random_uuid(), 'hr.leave.create', '?唾?隢?', 'hr', '?舐隢???, NOW()),
    (gen_random_uuid(), 'hr.leave.approve', '撖拇隢?', 'hr', '?臬祟?貉??隢?, NOW()),
    (gen_random_uuid(), 'hr.leave.manage', '蝞∠??', 'hr', '?舐恣???亥身摰?, NOW()),
    (gen_random_uuid(), 'hr.balance.view', '?亦?擗?', 'hr', '?舀????憿?, NOW()),
    (gen_random_uuid(), 'hr.balance.manage', '蝞∠?擗?', 'hr', '?舐恣????憿?, NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 摰?
-- ============================================
-- ============================================
-- Migration 005: Google Calendar Sync
-- 
-- ?嚗?-- - ?函頂蝯望?身摰?-- - 鈭辣?郊餈質馱
-- - 銵?蝞∠?
-- - ?郊甇瑕
-- - ?賊?閫貊??-- ============================================

-- ============================================
-- 1. ?函頂蝯望?身摰”
-- ============================================

CREATE TABLE google_calendar_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Calendar access
    calendar_id VARCHAR(255) NOT NULL,
    calendar_name VARCHAR(100),
    calendar_description TEXT,
    
    -- Authentication
    auth_method VARCHAR(20) DEFAULT 'gmail_account',
    auth_email VARCHAR(255),
    is_configured BOOLEAN DEFAULT false,
    
    -- Sync settings
    sync_enabled BOOLEAN DEFAULT true,
    sync_schedule_morning TIME DEFAULT '08:00:00',
    sync_schedule_evening TIME DEFAULT '18:00:00',
    sync_timezone VARCHAR(50) DEFAULT 'Asia/Taipei',
    
    -- What to sync
    sync_approved_leaves BOOLEAN DEFAULT true,
    sync_overtime BOOLEAN DEFAULT false,
    
    -- Event format
    event_title_template VARCHAR(255) DEFAULT '[隢?] {employee_name} - {leave_type}',
    event_color_id VARCHAR(10),
    
    -- Status
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20),
    last_sync_error TEXT,
    last_sync_events_pushed INTEGER DEFAULT 0,
    last_sync_events_pulled INTEGER DEFAULT 0,
    last_sync_conflicts INTEGER DEFAULT 0,
    last_sync_duration_ms INTEGER,
    
    -- Next scheduled sync
    next_sync_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Only allow one configuration row
CREATE UNIQUE INDEX idx_calendar_config_singleton ON google_calendar_config ((true));

-- Insert default config (disabled until configured)
INSERT INTO google_calendar_config (
    calendar_id, 
    calendar_name, 
    is_configured, 
    sync_enabled
) VALUES (
    'not-configured@placeholder.com',
    '隢?銵???,
    false,
    false
);

-- ============================================
-- 2. 鈭辣?郊餈質馱銵?-- ============================================

CREATE TABLE calendar_event_sync (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Local reference
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- Google Calendar reference
    google_event_id VARCHAR(255),
    google_event_etag VARCHAR(255),
    google_event_link VARCHAR(500),
    
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    local_updated_at TIMESTAMPTZ NOT NULL,
    google_updated_at TIMESTAMPTZ,
    
    -- Event data
    last_synced_data JSONB,
    
    -- Status
    sync_status VARCHAR(20) DEFAULT 'pending_create',
    
    -- Error tracking
    last_error TEXT,
    error_count INTEGER DEFAULT 0,
    last_error_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(leave_request_id)
);

CREATE INDEX idx_calendar_sync_status ON calendar_event_sync(sync_status);
CREATE INDEX idx_calendar_sync_pending ON calendar_event_sync(sync_status) 
    WHERE sync_status IN ('pending_create', 'pending_update', 'pending_delete');
CREATE INDEX idx_calendar_sync_google ON calendar_event_sync(google_event_id) 
    WHERE google_event_id IS NOT NULL;

-- ============================================
-- 3. 銵?餈質馱銵?-- ============================================

CREATE TABLE calendar_sync_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    calendar_event_sync_id UUID REFERENCES calendar_event_sync(id) ON DELETE SET NULL,
    leave_request_id UUID REFERENCES leave_requests(id) ON DELETE SET NULL,
    
    -- Conflict details
    conflict_type VARCHAR(50) NOT NULL,
    
    -- Data comparison
    ipig_data JSONB NOT NULL,
    google_data JSONB,
    
    -- Difference summary
    difference_summary TEXT,
    
    -- Resolution
    status VARCHAR(20) DEFAULT 'pending',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    -- If accepted Google changes
    requires_new_approval BOOLEAN DEFAULT false,
    new_approval_request_id UUID REFERENCES leave_requests(id),
    
    -- Detection
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_pending ON calendar_sync_conflicts(status) WHERE status = 'pending';
CREATE INDEX idx_sync_conflicts_leave ON calendar_sync_conflicts(leave_request_id);

-- ============================================
-- 4. ?郊甇瑕銵?-- ============================================

CREATE TABLE calendar_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Job details
    job_type VARCHAR(20) NOT NULL,
    triggered_by UUID REFERENCES users(id),
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Results
    status VARCHAR(20) DEFAULT 'running',
    
    -- Push stats
    events_created INTEGER DEFAULT 0,
    events_updated INTEGER DEFAULT 0,
    events_deleted INTEGER DEFAULT 0,
    
    -- Pull stats
    events_checked INTEGER DEFAULT 0,
    conflicts_detected INTEGER DEFAULT 0,
    
    -- Errors
    errors_count INTEGER DEFAULT 0,
    error_messages JSONB DEFAULT '[]',
    
    -- Progress
    progress_percentage INTEGER DEFAULT 0,
    current_operation VARCHAR(100),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_history_date ON calendar_sync_history(started_at DESC);
CREATE INDEX idx_sync_history_status ON calendar_sync_history(status);

-- ============================================
-- 5. 閫貊?? 隢?????湔????郊
-- ============================================

CREATE OR REPLACE FUNCTION queue_calendar_sync_on_leave_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only sync approved leaves
    IF NEW.status = 'APPROVED' THEN
        INSERT INTO calendar_event_sync (leave_request_id, local_updated_at, sync_status)
        VALUES (NEW.id, NOW(), 'pending_create')
        ON CONFLICT (leave_request_id) DO UPDATE SET
            local_updated_at = NOW(),
            sync_status = CASE 
                WHEN calendar_event_sync.google_event_id IS NULL THEN 'pending_create'
                ELSE 'pending_update'
            END,
            updated_at = NOW();
    
    ELSIF OLD.status = 'APPROVED' AND NEW.status IN ('CANCELLED', 'REVOKED') THEN
        UPDATE calendar_event_sync
        SET sync_status = 'pending_delete',
            local_updated_at = NOW(),
            updated_at = NOW()
        WHERE leave_request_id = NEW.id
          AND google_event_id IS NOT NULL;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_queue_calendar_sync
    AFTER INSERT OR UPDATE OF status ON leave_requests
    FOR EACH ROW
    EXECUTE FUNCTION queue_calendar_sync_on_leave_change();

-- ============================================
-- 6. ?交??郊?賊?甈?
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'hr.calendar.config', '閮剖?銵???甇?, 'hr', '?航身摰?Google ?交??郊', NOW()),
    (gen_random_uuid(), 'hr.calendar.view', '瑼Ｚ?銵???甇亦???, 'hr', '?舀炎閬??甇亦???, NOW()),
    (gen_random_uuid(), 'hr.calendar.sync', '??閫貊銵???甇?, 'hr', '?舀??孛?潭??甇?, NOW()),
    (gen_random_uuid(), 'hr.calendar.conflicts', '??銵???甇亥?蝒?, 'hr', '?航????甇亥?蝒?, NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 摰?
-- ============================================
-- ============================================
-- Migration 006: Audit System
-- 
-- ?嚗?-- - 雿輻?暑?隤?(??銵?
-- - ?餃鈭辣
-- - 雿輻??閰?-- - 瘣餃???
-- - 摰霅血
-- - 頛?賢?
-- ============================================

-- ============================================
-- 1. 雿輻?暑?隤?(??銵?
-- ============================================

CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    
    -- Actor information
    actor_user_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_display_name VARCHAR(100),
    actor_roles JSONB,
    
    -- Session context
    session_id UUID,
    session_started_at TIMESTAMPTZ,
    
    -- Event classification
    event_category VARCHAR(50) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_severity VARCHAR(20) DEFAULT 'info',
    
    -- Target entity
    entity_type VARCHAR(50),
    entity_id UUID,
    entity_display_name VARCHAR(255),
    
    -- Change tracking
    before_data JSONB,
    after_data JSONB,
    changed_fields TEXT[],
    
    -- Context
    ip_address INET,
    user_agent TEXT,
    request_path VARCHAR(500),
    request_method VARCHAR(10),
    response_status INTEGER,
    
    -- Geolocation
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    
    -- Security flags
    is_suspicious BOOLEAN DEFAULT false,
    suspicious_reason TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Partitioning key
    partition_date DATE NOT NULL DEFAULT CURRENT_DATE,
    
    -- Composite primary key
    PRIMARY KEY (id, partition_date)
) PARTITION BY RANGE (partition_date);

-- Create partitions for 2 years of data (2026-2027)
CREATE TABLE user_activity_logs_2026_q1 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
CREATE TABLE user_activity_logs_2026_q2 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');
CREATE TABLE user_activity_logs_2026_q3 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');
CREATE TABLE user_activity_logs_2026_q4 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');
CREATE TABLE user_activity_logs_2027_q1 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');
CREATE TABLE user_activity_logs_2027_q2 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');
CREATE TABLE user_activity_logs_2027_q3 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');
CREATE TABLE user_activity_logs_2027_q4 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');

-- Default partition
CREATE TABLE user_activity_logs_default PARTITION OF user_activity_logs DEFAULT;

-- Indexes
CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX idx_activity_category ON user_activity_logs(event_category, created_at DESC);
CREATE INDEX idx_activity_event_type ON user_activity_logs(event_type, created_at DESC);
CREATE INDEX idx_activity_suspicious ON user_activity_logs(is_suspicious) WHERE is_suspicious = true;
CREATE INDEX idx_activity_ip ON user_activity_logs(ip_address, created_at DESC);
CREATE INDEX idx_activity_date ON user_activity_logs(partition_date, created_at DESC);

-- ============================================
-- 2. ?餃鈭辣銵?-- ============================================

CREATE TABLE login_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    
    -- Event details
    event_type VARCHAR(20) NOT NULL,
    
    -- Device/Network
    ip_address INET,
    user_agent TEXT,
    device_type VARCHAR(50),
    browser VARCHAR(50),
    os VARCHAR(50),
    
    -- Geolocation
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    geo_timezone VARCHAR(50),
    
    -- Security analysis
    is_unusual_time BOOLEAN DEFAULT false,
    is_unusual_location BOOLEAN DEFAULT false,
    is_new_device BOOLEAN DEFAULT false,
    device_fingerprint VARCHAR(255),
    failure_reason VARCHAR(100),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_login_user ON login_events(user_id, created_at DESC);
CREATE INDEX idx_login_email ON login_events(email, created_at DESC);
CREATE INDEX idx_login_ip ON login_events(ip_address, created_at DESC);
CREATE INDEX idx_login_type ON login_events(event_type, created_at DESC);
CREATE INDEX idx_login_unusual ON login_events(user_id) 
    WHERE is_unusual_time OR is_unusual_location OR is_new_device;
CREATE INDEX idx_login_failure ON login_events(email, created_at DESC) 
    WHERE event_type = 'login_failure';

-- ============================================
-- 3. 雿輻??閰梯”
-- ============================================

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Session details
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Token reference
    refresh_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    
    -- Device/Network
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    
    -- Activity summary
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    ended_reason VARCHAR(50)
);

CREATE INDEX idx_sessions_user ON user_sessions(user_id, started_at DESC);
CREATE INDEX idx_sessions_active ON user_sessions(is_active, last_activity_at DESC) WHERE is_active = true;
CREATE INDEX idx_sessions_token ON user_sessions(refresh_token_id);

-- ============================================
-- 4. 瘣餃???銵?(瘥蝯梯?)
-- ============================================

CREATE TABLE user_activity_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    aggregate_date DATE NOT NULL,
    
    -- Counts
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    session_count INTEGER DEFAULT 0,
    total_session_minutes INTEGER DEFAULT 0,
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    
    -- Breakdowns
    actions_by_category JSONB DEFAULT '{}',
    pages_visited JSONB DEFAULT '[]',
    entities_modified JSONB DEFAULT '[]',
    
    -- Security
    unique_ip_count INTEGER DEFAULT 0,
    unusual_activity_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, aggregate_date)
);

CREATE INDEX idx_aggregates_user_date ON user_activity_aggregates(user_id, aggregate_date DESC);
CREATE INDEX idx_aggregates_date ON user_activity_aggregates(aggregate_date DESC);

-- ============================================
-- 5. 摰霅血銵?-- ============================================

CREATE TABLE security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Alert details
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'warning',
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Related entities
    user_id UUID REFERENCES users(id),
    activity_log_id UUID,
    login_event_id UUID REFERENCES login_events(id),
    
    -- Context
    context_data JSONB,
    
    -- Resolution
    status VARCHAR(20) DEFAULT 'open',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_status ON security_alerts(status, created_at DESC);
CREATE INDEX idx_alerts_user ON security_alerts(user_id, created_at DESC);
CREATE INDEX idx_alerts_severity ON security_alerts(severity, status) 
    WHERE status IN ('open', 'acknowledged', 'investigating');

-- ============================================
-- 6. Helper Functions
-- ============================================

-- Function to log an activity
CREATE OR REPLACE FUNCTION log_activity(
    p_actor_user_id UUID,
    p_event_category VARCHAR(50),
    p_event_type VARCHAR(100),
    p_entity_type VARCHAR(50),
    p_entity_id UUID,
    p_entity_display_name VARCHAR(255),
    p_before_data JSONB DEFAULT NULL,
    p_after_data JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    v_id UUID;
    v_actor_email VARCHAR(255);
    v_actor_display_name VARCHAR(100);
    v_actor_roles JSONB;
    v_changed_fields TEXT[];
BEGIN
    -- Get actor info
    SELECT email, display_name INTO v_actor_email, v_actor_display_name
    FROM users WHERE id = p_actor_user_id;
    
    -- Get actor roles
    SELECT jsonb_agg(r.code) INTO v_actor_roles
    FROM user_roles ur
    JOIN roles r ON ur.role_id = r.id
    WHERE ur.user_id = p_actor_user_id;
    
    -- Calculate changed fields
    IF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        SELECT array_agg(key) INTO v_changed_fields
        FROM (
            SELECT key FROM jsonb_each(p_after_data)
            EXCEPT
            SELECT key FROM jsonb_each(p_before_data) WHERE p_before_data->key = p_after_data->key
        ) changed_keys;
    END IF;
    
    -- Insert log entry
    INSERT INTO user_activity_logs (
        actor_user_id, actor_email, actor_display_name, actor_roles,
        event_category, event_type,
        entity_type, entity_id, entity_display_name,
        before_data, after_data, changed_fields,
        ip_address, user_agent
    ) VALUES (
        p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles,
        p_event_category, p_event_type,
        p_entity_type, p_entity_id, p_entity_display_name,
        p_before_data, p_after_data, v_changed_fields,
        p_ip_address, p_user_agent
    ) RETURNING id INTO v_id;
    
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

-- Function to check for brute force attacks
CREATE OR REPLACE FUNCTION check_brute_force(p_email VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE
    v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count
    FROM login_events
    WHERE email = p_email
      AND event_type = 'login_failure'
      AND created_at > NOW() - INTERVAL '15 minutes';
    
    RETURN v_failed_count >= 5;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 7. 撖抵??賊?甈?
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'audit.logs.view', '?亦?蝔賣?亥?', 'audit', '?舀?里?豢隤?, NOW()),
    (gen_random_uuid(), 'audit.logs.export', '?臬蝔賣?亥?', 'audit', '?臬?箇里?豢隤?, NOW()),
    (gen_random_uuid(), 'audit.timeline.view', '?亦?瘣餃???頠?, 'audit', '?舀?蝙?刻暑???遘', NOW()),
    (gen_random_uuid(), 'audit.alerts.view', '?亦?摰霅血', 'audit', '?舀???刻郎??, NOW()),
    (gen_random_uuid(), 'audit.alerts.manage', '蝞∠?摰霅血', 'audit', '?航????刻郎??, NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 8. GLP ??隤芣?
-- ============================================

COMMENT ON TABLE user_activity_logs IS 'GLP Compliance: Retention policy - 2 years hot storage, 5 years cold archive, 7 years total. Partitioned by quarter for efficient archival.';
COMMENT ON TABLE login_events IS 'GLP Compliance: Retention policy - 2 years hot storage, 5 years cold archive, 7 years total.';
COMMENT ON TABLE user_sessions IS 'Session tracking for security analysis. Sessions older than 90 days can be archived.';
COMMENT ON TABLE user_activity_aggregates IS 'Daily aggregates for dashboard. Can be regenerated from activity logs if needed.';

-- ============================================
-- 摰?
-- ============================================
-- GLP ???嚗??芷???游???撘瑕祟閮隤?-- Migration: 013_glp_compliance.sql
-- ?交?: 2026-01-19

-- ============================================
-- 1. ?箏??拍?”?潭憓??芷甈?
-- ============================================

-- pigs 銵典歇??deleted_at嚗憓?文????芷??ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_observations 銵?ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_surgeries 銵?ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_weights 銵?ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_vaccinations 銵?ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_sacrifices 銵?ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- vet_recommendations 銵?ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- ============================================
-- 2. 撱箇?霈??銵?-- ============================================

CREATE TABLE IF NOT EXISTS change_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,      -- 'pig', 'observation', 'surgery', etc.
    entity_id VARCHAR(50) NOT NULL,        -- ?臭誑??UUID ??INTEGER
    change_type VARCHAR(20) NOT NULL,      -- 'UPDATE', 'DELETE'
    reason TEXT NOT NULL,
    old_values JSONB,                       -- 霈????    new_values JSONB,                       -- 霈敺???    changed_fields TEXT[],                  -- 霈??雿?蝔?    changed_by UUID NOT NULL REFERENCES users(id),
    changed_at TIMESTAMPTZ DEFAULT NOW()
);

-- 蝝Ｗ?
CREATE INDEX IF NOT EXISTS idx_change_reasons_entity ON change_reasons(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_change_reasons_changed_at ON change_reasons(changed_at);
CREATE INDEX IF NOT EXISTS idx_change_reasons_changed_by ON change_reasons(changed_by);

-- ============================================
-- 3. 憓撥撖抵??亥?銵?-- ============================================

-- audit_logs 銵典?撘?ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS old_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS new_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS changed_fields TEXT[];
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS change_reason TEXT;

-- user_activity_logs 銵典?撘?ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS old_value JSONB;
ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS new_value JSONB;
ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS change_reason TEXT;

-- ============================================
-- 4. ?箸靘??餃?蝪賜?????摰???雿?-- ============================================

-- 鞈???甈?嚗hase 2 雿輻嚗?ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

-- ============================================
-- 5. ?餃?蝪賜?銵剁?Phase 2 雿輻嚗?-- ============================================

CREATE TABLE IF NOT EXISTS electronic_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,      -- 'sacrifice', 'protocol_approval'
    entity_id VARCHAR(50) NOT NULL,
    signer_id UUID NOT NULL REFERENCES users(id),
    signature_type VARCHAR(50) NOT NULL,   -- 'APPROVE', 'CONFIRM', 'WITNESS'
    content_hash VARCHAR(64) NOT NULL,     -- SHA-256 of content at signing time
    signature_data TEXT NOT NULL,          -- Encrypted signature (base64)
    ip_address VARCHAR(45),
    user_agent TEXT,
    signed_at TIMESTAMPTZ DEFAULT NOW(),
    is_valid BOOLEAN DEFAULT true,
    invalidated_reason TEXT,
    invalidated_at TIMESTAMPTZ,
    invalidated_by UUID REFERENCES users(id)
);

-- 蝝Ｗ?
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_entity ON electronic_signatures(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_signer ON electronic_signatures(signer_id);
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_valid ON electronic_signatures(is_valid) WHERE is_valid = true;

-- 蝪賜?撽??亥?
CREATE TABLE IF NOT EXISTS signature_verification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature_id UUID NOT NULL REFERENCES electronic_signatures(id),
    verified_by UUID REFERENCES users(id),
    verification_result BOOLEAN NOT NULL,
    verification_method VARCHAR(50) NOT NULL,
    failure_reason TEXT,
    verified_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 6. 閮??酉/?湔迤銵剁?Phase 2 雿輻嚗?-- ============================================

CREATE TABLE IF NOT EXISTS record_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(50) NOT NULL,      -- 'observation', 'surgery', 'sacrifice'
    record_id INTEGER NOT NULL,
    annotation_type VARCHAR(20) NOT NULL,  -- 'NOTE', 'CORRECTION', 'ADDENDUM'
    content TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    signature_id UUID REFERENCES electronic_signatures(id)  -- 憒???CORRECTION嚗?閬偷蝡?);

CREATE INDEX IF NOT EXISTS idx_record_annotations_record ON record_annotations(record_type, record_id);

-- ============================================
-- 摰?
-- ============================================
-- Migration: 017_pig_uuid_migration.sql
-- Purpose: Migrate pig module from INTEGER to UUID primary keys
-- Created: 2026-02-02
-- 
-- This migration:
-- 1. Adds UUID as the new primary key for pigs table
-- 2. Adds pig_no (SERIAL) for user-friendly display
-- 3. Updates all related tables to use UUID for pig references

-- ============================================
-- STEP 1: Add new columns to pigs table
-- ============================================

-- Add uuid column (will become the new primary key)
ALTER TABLE pigs ADD COLUMN uuid UUID DEFAULT gen_random_uuid();

-- Ensure all existing rows have UUIDs
UPDATE pigs SET uuid = gen_random_uuid() WHERE uuid IS NULL;

-- Make uuid NOT NULL
ALTER TABLE pigs ALTER COLUMN uuid SET NOT NULL;

-- Add pig_no column for display purposes (auto-increment)
-- We'll use a sequence to continue from the max existing id
DO $$
DECLARE
    max_id INTEGER;
BEGIN
    SELECT COALESCE(MAX(id), 0) INTO max_id FROM pigs;
    EXECUTE format('CREATE SEQUENCE IF NOT EXISTS pig_no_seq START WITH %s', max_id + 1);
END $$;

ALTER TABLE pigs ADD COLUMN pig_no INTEGER;

-- Copy existing id values to pig_no
UPDATE pigs SET pig_no = id;

-- Set pig_no to use the sequence for new records
ALTER TABLE pigs ALTER COLUMN pig_no SET DEFAULT nextval('pig_no_seq');
ALTER TABLE pigs ALTER COLUMN pig_no SET NOT NULL;

-- Create unique index on pig_no
CREATE UNIQUE INDEX idx_pigs_pig_no ON pigs(pig_no);

-- ============================================
-- STEP 2: Add pig_uuid columns to related tables
-- ============================================

-- pig_observations
ALTER TABLE pig_observations ADD COLUMN pig_uuid UUID;

-- pig_surgeries
ALTER TABLE pig_surgeries ADD COLUMN pig_uuid UUID;

-- pig_weights
ALTER TABLE pig_weights ADD COLUMN pig_uuid UUID;

-- pig_vaccinations
ALTER TABLE pig_vaccinations ADD COLUMN pig_uuid UUID;

-- pig_sacrifices
ALTER TABLE pig_sacrifices ADD COLUMN pig_uuid UUID;

-- pig_pathology_reports
ALTER TABLE pig_pathology_reports ADD COLUMN pig_uuid UUID;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'pig_export_records') THEN
        EXECUTE 'ALTER TABLE pig_export_records ADD COLUMN pig_uuid UUID';
    END IF;
END $$;

-- ============================================
-- STEP 3: Migrate foreign key data
-- ============================================

UPDATE pig_observations po 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = po.pig_id);

UPDATE pig_surgeries ps 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = ps.pig_id);

UPDATE pig_weights pw 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pw.pig_id);

UPDATE pig_vaccinations pv 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pv.pig_id);

UPDATE pig_sacrifices ps 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = ps.pig_id);

UPDATE pig_pathology_reports pp 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pp.pig_id);

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_uuid') THEN
        EXECUTE 'UPDATE pig_export_records pe 
                 SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pe.pig_id)
                 WHERE pig_id IS NOT NULL';
    END IF;
END $$;

-- ============================================
-- STEP 4: Make pig_uuid NOT NULL where appropriate
-- ============================================

ALTER TABLE pig_observations ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_surgeries ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_weights ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_vaccinations ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_sacrifices ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_pathology_reports ALTER COLUMN pig_uuid SET NOT NULL;

-- ============================================
-- STEP 5: Drop old foreign key constraints
-- ============================================

ALTER TABLE pig_observations DROP CONSTRAINT IF EXISTS pig_observations_pig_id_fkey;
ALTER TABLE pig_surgeries DROP CONSTRAINT IF EXISTS pig_surgeries_pig_id_fkey;
ALTER TABLE pig_weights DROP CONSTRAINT IF EXISTS pig_weights_pig_id_fkey;
ALTER TABLE pig_vaccinations DROP CONSTRAINT IF EXISTS pig_vaccinations_pig_id_fkey;
ALTER TABLE pig_sacrifices DROP CONSTRAINT IF EXISTS pig_sacrifices_pig_id_fkey;
ALTER TABLE pig_pathology_reports DROP CONSTRAINT IF EXISTS pig_pathology_reports_pig_id_fkey;

-- Drop unique constraint on pig_sacrifices if exists
ALTER TABLE pig_sacrifices DROP CONSTRAINT IF EXISTS pig_sacrifices_pig_id_key;
ALTER TABLE pig_pathology_reports DROP CONSTRAINT IF EXISTS pig_pathology_reports_pig_id_key;

-- ============================================
-- STEP 6: Drop old indexes that reference pig_id
-- ============================================

DROP INDEX IF EXISTS idx_pig_observations_pig_id;
DROP INDEX IF EXISTS idx_pig_surgeries_pig_id;
DROP INDEX IF EXISTS idx_pig_weights_pig_id;
DROP INDEX IF EXISTS idx_pig_vaccinations_pig_id;
DROP INDEX IF EXISTS idx_pig_sacrifices_pig_id;

-- ============================================
-- STEP 7: Drop old pig_id columns from related tables
-- ============================================

ALTER TABLE pig_observations DROP COLUMN pig_id;
ALTER TABLE pig_surgeries DROP COLUMN pig_id;
ALTER TABLE pig_weights DROP COLUMN pig_id;
ALTER TABLE pig_vaccinations DROP COLUMN pig_id;
ALTER TABLE pig_sacrifices DROP COLUMN pig_id;
ALTER TABLE pig_pathology_reports DROP COLUMN pig_id;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_id') THEN
        EXECUTE 'ALTER TABLE pig_export_records DROP COLUMN pig_id';
    END IF;
END $$;

-- ============================================
-- STEP 8: Rename pig_uuid to pig_id
-- ============================================

ALTER TABLE pig_observations RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_surgeries RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_weights RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_vaccinations RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_sacrifices RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_pathology_reports RENAME COLUMN pig_uuid TO pig_id;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_uuid') THEN
        EXECUTE 'ALTER TABLE pig_export_records RENAME COLUMN pig_uuid TO pig_id';
    END IF;
END $$;

-- ============================================
-- STEP 9: Update pigs table primary key
-- ============================================

-- Drop old primary key
ALTER TABLE pigs DROP CONSTRAINT pigs_pkey;

-- Drop old id column
ALTER TABLE pigs DROP COLUMN id;

-- Rename uuid to id
ALTER TABLE pigs RENAME COLUMN uuid TO id;

-- Add new primary key
ALTER TABLE pigs ADD PRIMARY KEY (id);

-- ============================================
-- STEP 10: Add new foreign key constraints
-- ============================================

ALTER TABLE pig_observations 
    ADD CONSTRAINT pig_observations_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_surgeries 
    ADD CONSTRAINT pig_surgeries_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_weights 
    ADD CONSTRAINT pig_weights_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_vaccinations 
    ADD CONSTRAINT pig_vaccinations_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_sacrifices 
    ADD CONSTRAINT pig_sacrifices_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_pathology_reports 
    ADD CONSTRAINT pig_pathology_reports_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

-- Add unique constraint for pig_sacrifices and pig_pathology_reports (one per pig)
ALTER TABLE pig_sacrifices ADD CONSTRAINT pig_sacrifices_pig_id_key UNIQUE (pig_id);
ALTER TABLE pig_pathology_reports ADD CONSTRAINT pig_pathology_reports_pig_id_key UNIQUE (pig_id);

-- ============================================
-- STEP 11: Create new indexes
-- ============================================

CREATE INDEX idx_pig_observations_pig_id ON pig_observations(pig_id);
CREATE INDEX idx_pig_surgeries_pig_id ON pig_surgeries(pig_id);
CREATE INDEX idx_pig_weights_pig_id ON pig_weights(pig_id);
CREATE INDEX idx_pig_vaccinations_pig_id ON pig_vaccinations(pig_id);
CREATE INDEX idx_pig_sacrifices_pig_id ON pig_sacrifices(pig_id);

-- ============================================
-- STEP 12: Update pig_record_attachments
-- (Uses record_type + record_id pattern, no direct pig_id FK)
-- No changes needed for this table
-- ============================================

-- ============================================
-- STEP 13: Update deleted_by reference (already UUID, no change needed)
-- ============================================

-- ============================================
-- STEP 14: Verify migration
-- ============================================

-- Verify pigs table structure
DO $$
BEGIN
    -- Check if id is now UUID
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'pigs' 
        AND column_name = 'id' 
        AND data_type = 'uuid'
    ) THEN
        RAISE EXCEPTION 'Migration failed: pigs.id is not UUID';
    END IF;
    
    -- Check if pig_no exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'pigs' 
        AND column_name = 'pig_no'
    ) THEN
        RAISE EXCEPTION 'Migration failed: pigs.pig_no does not exist';
    END IF;
    
    RAISE NOTICE 'Migration verification passed!';
END $$;
-- ============================================
-- Migration 021: Emergency Medication, Euthanasia & Review System
-- 
-- ?嚗?-- - 摰?甇餃??”??-- - 撖拇憪瘙箄降銵冽
-- - ?典?降隢?銵冽
-- - ?憿??游?
-- ============================================

-- ============================================
-- 1. ?芾?憿? (Custom Types)
-- ============================================

-- 摰?甇餃????CREATE TYPE euthanasia_order_status AS ENUM (
    'pending_pi',        -- 蝑? PI ??
    'approved',          -- PI ???瑁?
    'appealed',          -- PI ?唾??怎楨
    'chair_arbitration', -- CHAIR 隞脰?銝?    'executed',          -- 撌脣銵?    'cancelled'          -- 撌脣?瘨?);

-- ============================================
-- 2. 摰?甇餃?”
-- ============================================

CREATE TABLE euthanasia_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    pi_user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status euthanasia_order_status NOT NULL DEFAULT 'pending_pi',
    deadline_at TIMESTAMPTZ NOT NULL,           -- 24 撠?敺?    pi_responded_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    executed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_orders_pig_id ON euthanasia_orders(pig_id);
CREATE INDEX idx_euthanasia_orders_status ON euthanasia_orders(status);
CREATE INDEX idx_euthanasia_orders_deadline ON euthanasia_orders(deadline_at);
CREATE INDEX idx_euthanasia_orders_pi_user_id ON euthanasia_orders(pi_user_id);

-- ============================================
-- 3. 摰?甇餅蝺拍隢”
-- ============================================

CREATE TABLE euthanasia_appeals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES euthanasia_orders(id) ON DELETE CASCADE,
    pi_user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    attachment_path VARCHAR(500),
    chair_user_id UUID REFERENCES users(id),
    chair_decision VARCHAR(20),                  -- 'approve_appeal', 'reject_appeal'
    chair_decided_at TIMESTAMPTZ,
    chair_deadline_at TIMESTAMPTZ,              -- CHAIR 鋆捱??
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_appeals_order_id ON euthanasia_appeals(order_id);
CREATE INDEX idx_euthanasia_appeals_chair_deadline ON euthanasia_appeals(chair_deadline_at);

-- ============================================
-- 4. 撖拇憪瘙箄降銵剁?餈質馱?梯?瘙綽?
-- ============================================

CREATE TABLE reviewer_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_version_id UUID NOT NULL REFERENCES protocol_versions(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    decision VARCHAR(20) NOT NULL,               -- 'approve', 'revision_required', 'reject'
    comment TEXT,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (protocol_version_id, reviewer_id)
);

CREATE INDEX idx_reviewer_decisions_protocol_version ON reviewer_decisions(protocol_version_id);
CREATE INDEX idx_reviewer_decisions_reviewer ON reviewer_decisions(reviewer_id);

-- ============================================
-- 5. ?典?降隢?銵?-- ============================================

CREATE TABLE meeting_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'scheduled', 'completed', 'cancelled'
    meeting_date TIMESTAMPTZ,
    chair_decision TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_requests_protocol_id ON meeting_requests(protocol_id);
CREATE INDEX idx_meeting_requests_status ON meeting_requests(status);

-- ============================================
-- 6. ?游??憿?
-- ============================================

-- 雿輻 DO block 靘??典?啣? enum ?潘?憒?銝??剁?
DO $$
BEGIN
    -- ?啣? emergency_medication ?憿?
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'emergency_medication' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'emergency_medication';
    END IF;
END$$;

DO $$
BEGIN
    -- ?啣? euthanasia_order ?憿?
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'euthanasia_order' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'euthanasia_order';
    END IF;
END$$;

DO $$
BEGIN
    -- ?啣? euthanasia_appeal ?憿?
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'euthanasia_appeal' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'euthanasia_appeal';
    END IF;
END$$;

DO $$
BEGIN
    -- ?啣? meeting_request ?憿?
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'meeting_request' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'meeting_request';
    END IF;
END$$;

-- ============================================
-- 7. ?啣? review_comments ??甈?嚗???摮嚗?-- ============================================

-- ?啣? parent_comment_id 甈?隞交?游?閬???DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'review_comments' AND column_name = 'parent_comment_id') THEN
        ALTER TABLE review_comments ADD COLUMN parent_comment_id UUID REFERENCES review_comments(id);
    END IF;
END$$;

-- ?啣? replied_by 甈?
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'review_comments' AND column_name = 'replied_by') THEN
        ALTER TABLE review_comments ADD COLUMN replied_by UUID REFERENCES users(id);
    END IF;
END$$;
-- ============================================
-- Migration 022: Amendment System
-- 
-- 霈?唾?蝟餌絞嚗?-- - 霈?唾?銝餉” (amendments)
-- - 霈?唾?? (amendment_versions)
-- - 霈?唾?撖拇?晷 (amendment_review_assignments)
-- - 霈?唾???風蝔?(amendment_status_history)
-- - 靽格 review_comments 銵冽?渲?蝔踹???-- ============================================

-- ============================================
-- 1. ?啣??芾?憿?
-- ============================================

-- 霈?唾?憿?
CREATE TYPE amendment_type AS ENUM (
    'MAJOR',    -- ?之霈嚗??券?撖拇憪撖拇嚗?    'MINOR',    -- 撠??湛?銵撖拇嚗?    'PENDING'   -- 敺?憿?);

-- 霈?唾????CREATE TYPE amendment_status AS ENUM (
    'DRAFT',              -- ?阮
    'SUBMITTED',          -- 撌脫?鈭歹?敺?憿?    'CLASSIFIED',         -- 撌脣?憿?敺祟??    'UNDER_REVIEW',       -- 撖拇銝哨??之霈嚗?    'REVISION_REQUIRED',  -- ?靽株?
    'RESUBMITTED',        -- 撌脤???    'APPROVED',           -- ?詨?嚗?憭扯??湛?
    'REJECTED',           -- ?行捱
    'ADMIN_APPROVED'      -- 銵撖拇?詨?嚗?霈嚗?);

-- ============================================
-- 2. 霈?唾?銝餉”
-- ============================================

CREATE TABLE amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    amendment_no VARCHAR(30) NOT NULL UNIQUE,
    revision_number INTEGER NOT NULL,
    amendment_type amendment_type NOT NULL DEFAULT 'PENDING',
    status amendment_status NOT NULL DEFAULT 'DRAFT',
    title VARCHAR(200) NOT NULL,
    description TEXT,
    change_items TEXT[],
    changes_content JSONB,
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    classified_by UUID REFERENCES users(id),
    classified_at TIMESTAMPTZ,
    classification_remark TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 蝣箔???閮??revision_number ?臭?
    CONSTRAINT amendments_protocol_revision_unique UNIQUE (protocol_id, revision_number)
);

CREATE INDEX idx_amendments_protocol_id ON amendments(protocol_id);
CREATE INDEX idx_amendments_status ON amendments(status);
CREATE INDEX idx_amendments_amendment_type ON amendments(amendment_type);
CREATE INDEX idx_amendments_submitted_by ON amendments(submitted_by);

COMMENT ON TABLE amendments IS '霈?唾?銝餉”';
COMMENT ON COLUMN amendments.amendment_no IS '霈蝺刻?嚗撘?PIG-114001-R01';
COMMENT ON COLUMN amendments.revision_number IS '蝚砍嗾甈∟???(1, 2, 3...)';
COMMENT ON COLUMN amendments.amendment_type IS '霈憿?嚗AJOR=?之霈, MINOR=撠??? PENDING=敺?憿?;
COMMENT ON COLUMN amendments.change_items IS '霈?皜';
COMMENT ON COLUMN amendments.changes_content IS '霈閰喟敦?批捆嚗SONB 蝯?嚗?;

-- ============================================
-- 3. 霈?唾??銵?-- ============================================

CREATE TABLE amendment_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    
    CONSTRAINT amendment_versions_unique UNIQUE (amendment_id, version_no)
);

CREATE INDEX idx_amendment_versions_amendment_id ON amendment_versions(amendment_id);

COMMENT ON TABLE amendment_versions IS '霈?唾??敹怎';

-- ============================================
-- 4. 霈?唾?撖拇?晷銵?-- ============================================

CREATE TABLE amendment_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decision VARCHAR(20),
    decided_at TIMESTAMPTZ,
    comment TEXT,
    
    CONSTRAINT amendment_review_assignments_unique UNIQUE (amendment_id, reviewer_id),
    CONSTRAINT amendment_decision_check CHECK (decision IS NULL OR decision IN ('APPROVE', 'REJECT', 'REVISION'))
);

CREATE INDEX idx_amendment_review_assignments_amendment_id ON amendment_review_assignments(amendment_id);
CREATE INDEX idx_amendment_review_assignments_reviewer_id ON amendment_review_assignments(reviewer_id);

COMMENT ON TABLE amendment_review_assignments IS '霈?唾?撖拇憪?晷';
COMMENT ON COLUMN amendment_review_assignments.decision IS '撖拇瘙箏?嚗PPROVE=?詨?, REJECT=?行捱, REVISION=?靽株?';

-- ============================================
-- 5. 霈?唾???風蝔”
-- ============================================

CREATE TABLE amendment_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    from_status amendment_status,
    to_status amendment_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_amendment_status_history_amendment_id ON amendment_status_history(amendment_id);

COMMENT ON TABLE amendment_status_history IS '霈?唾?????湔風蝔?;

-- ============================================
-- 6. 靽格 review_comments 銵冽?渲?蝔踹???-- ============================================

-- ?啣??阮?賊?甈?
ALTER TABLE review_comments 
ADD COLUMN draft_content TEXT,
ADD COLUMN drafted_by UUID REFERENCES users(id),
ADD COLUMN draft_updated_at TIMESTAMPTZ;

CREATE INDEX idx_review_comments_drafted_by ON review_comments(drafted_by) WHERE drafted_by IS NOT NULL;

COMMENT ON COLUMN review_comments.draft_content IS '???阮?批捆嚗? PI/Coeditor ?航?嚗?;
COMMENT ON COLUMN review_comments.drafted_by IS '?阮?啣神??;
COMMENT ON COLUMN review_comments.draft_updated_at IS '?阮?敺?唳???;

-- ============================================
-- 7. ?啣?甈?
-- ============================================

-- 霈?唾??賊?甈?
INSERT INTO permissions (id, code, name, module, description) VALUES
    (gen_random_uuid(), 'amendment.create', '撱箇?霈?唾?', 'AUP', '?迂撱箇?霈?唾?嚗I嚗?),
    (gen_random_uuid(), 'amendment.read', '瑼Ｚ?霈?唾?', 'AUP', '?迂瑼Ｚ?霈?唾?'),
    (gen_random_uuid(), 'amendment.update', '?湔霈?唾?', 'AUP', '?迂?湔霈?唾?'),
    (gen_random_uuid(), 'amendment.classify', '??霈?唾?', 'AUP', '?迂??霈?唾??粹?憭?撠??湛?IACUC_STAFF嚗?),
    (gen_random_uuid(), 'amendment.review', '撖拇霈?唾?', 'AUP', '?迂撖拇霈?唾?嚗祟?亙??∴?'),
    (gen_random_uuid(), 'amendment.approve', '?詨?霈?唾?', 'AUP', '?迂?詨?霈?唾?'),
    (gen_random_uuid(), 'amendment.admin_approve', '銵撖拇霈?唾?', 'AUP', '?迂銵撖拇?詨?撠??湛?IACUC_STAFF嚗?);

-- ?箄??脣?????-- PI
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'PI' AND p.code IN ('amendment.create', 'amendment.read', 'amendment.update');

-- IACUC_STAFF
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'IACUC_STAFF' AND p.code IN ('amendment.read', 'amendment.classify', 'amendment.admin_approve');

-- REVIEWER
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'REVIEWER' AND p.code IN ('amendment.read', 'amendment.review');

-- VET
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'VET' AND p.code IN ('amendment.read', 'amendment.review');

-- CHAIR
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'CHAIR' AND p.code IN ('amendment.read', 'amendment.review', 'amendment.approve');

-- SYSTEM_ADMIN
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin' AND p.code LIKE 'amendment.%';

-- ============================================
-- 8. 撱箇?閬?嚗??渡隢?銵?-- ============================================

CREATE OR REPLACE VIEW amendment_list_view AS
SELECT 
    a.id,
    a.protocol_id,
    a.amendment_no,
    a.revision_number,
    a.amendment_type,
    a.status,
    a.title,
    a.description,
    a.change_items,
    a.submitted_at,
    a.classified_at,
    a.created_at,
    a.updated_at,
    p.iacuc_no AS protocol_iacuc_no,
    p.title AS protocol_title,
    u.display_name AS submitted_by_name,
    c.display_name AS classified_by_name
FROM amendments a
JOIN protocols p ON a.protocol_id = p.id
LEFT JOIN users u ON a.submitted_by = u.id
LEFT JOIN users c ON a.classified_by = c.id;

COMMENT ON VIEW amendment_list_view IS '霈?唾??”閬?嚗?鞈?嚗?;
-- Storage Locations (?脖?/鞎冽) 鞈?銵?-- ?冽?澈?折鞎冽閬死??撅蝞∠?

CREATE TABLE storage_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200),
    location_type VARCHAR(50) NOT NULL DEFAULT 'shelf',
    row_index INTEGER NOT NULL DEFAULT 0,
    col_index INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 2,
    height INTEGER NOT NULL DEFAULT 2,
    capacity INTEGER,
    current_count INTEGER DEFAULT 0,
    color VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    config JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(warehouse_id, code)
);

-- Indexes
CREATE INDEX idx_storage_locations_warehouse ON storage_locations(warehouse_id);
CREATE INDEX idx_storage_locations_type ON storage_locations(location_type);
CREATE INDEX idx_storage_locations_active ON storage_locations(is_active);

-- Comments
COMMENT ON TABLE storage_locations IS '?澈?脖?/鞎冽鞈?銵剁??冽閬死?像?Ｗ?雿?';
COMMENT ON COLUMN storage_locations.location_type IS '?脖?憿?: shelf(鞎冽), rack(?脩??, zone(???, bin(?脩??';
COMMENT ON COLUMN storage_locations.row_index IS '蝬脫銵漣璅??冽 react-grid-layout 閬死??;
COMMENT ON COLUMN storage_locations.col_index IS '蝬脫?漣璅??冽 react-grid-layout 閬死??;
COMMENT ON COLUMN storage_locations.width IS '雿蝬脫撖砍漲 (?)';
COMMENT ON COLUMN storage_locations.height IS '雿蝬脫擃漲 (銵)';
COMMENT ON COLUMN storage_locations.config IS '憿??蔭 JSON (憒澈摨衣???摨西?瘙?)';

-- ?啣?甈?
INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES 
    (gen_random_uuid(), 'erp.storage_location.view', '瑼Ｚ??脖?', 'ERP', '瑼Ｚ??澈?脖?/鞎冽鞈?', NOW()),
    (gen_random_uuid(), 'erp.storage_location.create', '撱箇??脖?', 'ERP', '撱箇??啁??脖?/鞎冽', NOW()),
    (gen_random_uuid(), 'erp.storage_location.edit', '蝺刻摩?脖?', 'ERP', '蝺刻摩?脖?/鞎冽鞈???撅', NOW()),
    (gen_random_uuid(), 'erp.storage_location.delete', '?芷?脖?', 'ERP', '?芷?脖?/鞎冽', NOW())
ON CONFLICT (code) DO NOTHING;

-- 蝯?admin 閫?啣?甈?
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin' 
  AND p.code IN ('erp.storage_location.view', 'erp.storage_location.create', 'erp.storage_location.edit', 'erp.storage_location.delete')
ON CONFLICT DO NOTHING;

-- 蝯血澈蝞∠???(WAREHOUSE_MANAGER) 閫?啣?甈?
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER' 
  AND p.code IN ('erp.storage_location.view', 'erp.storage_location.create', 'erp.storage_location.edit', 'erp.storage_location.delete')
ON CONFLICT DO NOTHING;
-- =============================================================================
-- Migration 029: Import/Export Tables for Animal Management
-- ?啣?鞊祇?臬?寞活閮???箄???”?澆? enum 憿?
-- =============================================================================

-- ?臬憿? enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'import_type') THEN
        CREATE TYPE import_type AS ENUM ('pig_basic', 'pig_weight');
    END IF;
END $$;

-- ?臬???enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'import_status') THEN
        CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
    END IF;
END $$;

-- ?臬憿? enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'export_type') THEN
        CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
    END IF;
END $$;

-- ?臬?澆? enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'export_format') THEN
        CREATE TYPE export_format AS ENUM ('pdf', 'excel');
    END IF;
END $$;

-- 鞊祇?臬?寞活閮?銵?CREATE TABLE IF NOT EXISTS pig_import_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_type import_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    status import_status NOT NULL DEFAULT 'pending',
    error_details JSONB,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- 鞊祇?臬閮?銵?CREATE TABLE IF NOT EXISTS pig_export_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID REFERENCES pigs(id) ON DELETE SET NULL,
    iacuc_no VARCHAR(50),
    export_type export_type NOT NULL,
    export_format export_format NOT NULL,
    file_path VARCHAR(500),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 蝝Ｗ?
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_by ON pig_import_batches(created_by);
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_at ON pig_import_batches(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_pig_id ON pig_export_records(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_iacuc_no ON pig_export_records(iacuc_no);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_created_at ON pig_export_records(created_at DESC);

-- ???賊?甈?
INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.info.import', '?臬??箸鞈?', '撖阡??蝞∠?', '?舀甈∪?亥惇?餃?祈???, NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.info.import');

INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.weight.import', '?臬?擃?鞈?', '撖阡??蝞∠?', '?舀甈∪?亥惇?駁?????, NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.weight.import');

INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.export.medical', '?臬??怎?鞈?', '撖阡??蝞∠?', '?臬?箄惇?餌?甇瑁???, NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.export.medical');

-- ?? SYSTEM_ADMIN 閫??甈?
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'SYSTEM_ADMIN'
  AND p.code IN ('animal.info.import', 'animal.weight.import', 'animal.export.medical')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ?? ADMIN_STAFF 閫??甈?
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'ADMIN_STAFF'
  AND p.code IN ('animal.info.import', 'animal.weight.import', 'animal.export.medical')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );
