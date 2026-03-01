-- ============================================
-- Migration 008: 稽核系統 + ERP 倉儲
-- ============================================

-- 8.1 稽核系統
CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    actor_user_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_display_name VARCHAR(100),
    actor_roles JSONB,
    session_id UUID,
    session_started_at TIMESTAMPTZ,
    event_category VARCHAR(50) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_severity VARCHAR(20) DEFAULT 'info',
    entity_type VARCHAR(50),
    entity_id UUID,
    entity_display_name VARCHAR(255),
    before_data JSONB,
    after_data JSONB,
    changed_fields TEXT[],
    ip_address INET,
    user_agent TEXT,
    request_path VARCHAR(500),
    request_method VARCHAR(10),
    response_status INTEGER,
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    is_suspicious BOOLEAN DEFAULT false,
    suspicious_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    partition_date DATE NOT NULL DEFAULT CURRENT_DATE,
    integrity_hash VARCHAR(128),
    previous_hash VARCHAR(128),
    PRIMARY KEY (id, partition_date)
) PARTITION BY RANGE (partition_date);

CREATE TABLE user_activity_logs_2026_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
CREATE TABLE user_activity_logs_2026_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');
CREATE TABLE user_activity_logs_2026_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');
CREATE TABLE user_activity_logs_2026_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');
CREATE TABLE user_activity_logs_2027_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');
CREATE TABLE user_activity_logs_2027_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');
CREATE TABLE user_activity_logs_2027_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');
CREATE TABLE user_activity_logs_2027_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');

CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX idx_activity_category ON user_activity_logs(event_category, created_at DESC);
CREATE INDEX idx_activity_event_type ON user_activity_logs(event_type, created_at DESC);
CREATE INDEX idx_activity_suspicious ON user_activity_logs(is_suspicious) WHERE is_suspicious = true;
CREATE INDEX idx_activity_ip ON user_activity_logs(ip_address, created_at DESC);
CREATE INDEX idx_activity_date ON user_activity_logs(partition_date, created_at DESC);
CREATE INDEX idx_activity_logs_integrity ON user_activity_logs (created_at, integrity_hash);

CREATE TABLE login_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    event_type VARCHAR(20) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    device_type VARCHAR(50),
    browser VARCHAR(50),
    os VARCHAR(50),
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    geo_timezone VARCHAR(50),
    is_unusual_time BOOLEAN DEFAULT false,
    is_unusual_location BOOLEAN DEFAULT false,
    is_new_device BOOLEAN DEFAULT false,
    is_mass_login BOOLEAN DEFAULT false,
    device_fingerprint VARCHAR(255),
    failure_reason VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_login_user ON login_events(user_id, created_at DESC);
CREATE INDEX idx_login_email ON login_events(email, created_at DESC);
CREATE INDEX idx_login_ip ON login_events(ip_address, created_at DESC);
CREATE INDEX idx_login_type ON login_events(event_type, created_at DESC);

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    refresh_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    ended_reason VARCHAR(50)
);
CREATE INDEX idx_sessions_user ON user_sessions(user_id, started_at DESC);
CREATE INDEX idx_sessions_active ON user_sessions(is_active, last_activity_at DESC) WHERE is_active = true;

CREATE TABLE user_activity_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    aggregate_date DATE NOT NULL,
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    session_count INTEGER DEFAULT 0,
    total_session_minutes INTEGER DEFAULT 0,
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    actions_by_category JSONB DEFAULT '{}',
    pages_visited JSONB DEFAULT '[]',
    entities_modified JSONB DEFAULT '[]',
    unique_ip_count INTEGER DEFAULT 0,
    unusual_activity_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, aggregate_date)
);
CREATE INDEX idx_aggregates_user_date ON user_activity_aggregates(user_id, aggregate_date DESC);
CREATE INDEX idx_aggregates_date ON user_activity_aggregates(aggregate_date DESC);

CREATE TABLE security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'warning',
    title VARCHAR(255) NOT NULL,
    description TEXT,
    user_id UUID REFERENCES users(id),
    activity_log_id UUID,
    login_event_id UUID REFERENCES login_events(id),
    context_data JSONB,
    status VARCHAR(20) DEFAULT 'open',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_severity CHECK (severity IN ('info', 'warning', 'critical')),
    CONSTRAINT chk_alert_status CHECK (status IN ('open', 'acknowledged', 'investigating', 'resolved', 'false_positive'))
);
CREATE INDEX idx_alerts_status ON security_alerts(status, created_at DESC);

CREATE OR REPLACE FUNCTION log_activity(
    p_actor_user_id UUID, p_event_category VARCHAR(50), p_event_type VARCHAR(100),
    p_entity_type VARCHAR(50), p_entity_id UUID, p_entity_display_name VARCHAR(255),
    p_before_data JSONB DEFAULT NULL, p_after_data JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL, p_user_agent TEXT DEFAULT NULL
) RETURNS UUID AS $$
DECLARE v_id UUID; v_actor_email VARCHAR(255); v_actor_display_name VARCHAR(100); v_actor_roles JSONB; v_changed_fields TEXT[];
BEGIN
    SELECT email, display_name INTO v_actor_email, v_actor_display_name FROM users WHERE id = p_actor_user_id;
    SELECT jsonb_agg(r.code) INTO v_actor_roles FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = p_actor_user_id;
    IF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        SELECT array_agg(key) INTO v_changed_fields FROM (SELECT key FROM jsonb_each(p_after_data) EXCEPT SELECT key FROM jsonb_each(p_before_data) WHERE p_before_data->key = p_after_data->key) changed_keys;
    END IF;
    INSERT INTO user_activity_logs (actor_user_id, actor_email, actor_display_name, actor_roles, event_category, event_type, entity_type, entity_id, entity_display_name, before_data, after_data, changed_fields, ip_address, user_agent)
    VALUES (p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles, p_event_category, p_event_type, p_entity_type, p_entity_id, p_entity_display_name, p_before_data, p_after_data, v_changed_fields, p_ip_address, p_user_agent) RETURNING id INTO v_id;
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION check_brute_force(p_email VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count FROM login_events WHERE email = p_email AND event_type = 'login_failure' AND created_at > NOW() - INTERVAL '15 minutes';
    RETURN v_failed_count >= 5;
END;
$$ LANGUAGE plpgsql;

-- 8.2 ERP 倉儲
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

CREATE TABLE product_categories (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES product_categories(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE sku_categories (
    code CHAR(3) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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

CREATE TABLE sku_sequences (
    category_code CHAR(3) NOT NULL,
    subcategory_code CHAR(3) NOT NULL,
    last_sequence INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (category_code, subcategory_code)
);

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
CREATE INDEX idx_products_status ON products(status);

CREATE TABLE product_uom_conversions (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    uom VARCHAR(20) NOT NULL,
    factor_to_base NUMERIC(18, 6) NOT NULL,
    UNIQUE (product_id, uom)
);

CREATE TABLE partners (
    id UUID PRIMARY KEY,
    partner_type partner_type NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    supplier_category supplier_category,
    customer_category customer_category,
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
    zone VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(warehouse_id, code)
);
CREATE INDEX idx_storage_locations_warehouse ON storage_locations(warehouse_id);

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
    iacuc_no VARCHAR(20),
    requires_manager_approval BOOLEAN DEFAULT FALSE,
    scrap_total_amount DECIMAL,
    manager_approval_status VARCHAR(20),
    manager_approved_by UUID REFERENCES users(id),
    manager_approved_at TIMESTAMPTZ,
    manager_reject_reason TEXT,
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
    storage_location_id UUID REFERENCES storage_locations(id),
    UNIQUE (document_id, line_no)
);
CREATE INDEX idx_document_lines_document_id ON document_lines(document_id);

CREATE TABLE storage_location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    storage_location_id UUID NOT NULL REFERENCES storage_locations(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    on_hand_qty NUMERIC(18, 4) NOT NULL DEFAULT 0,
    batch_no VARCHAR(50),
    expiry_date DATE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_storage_location_inventory_unique ON storage_location_inventory (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date));

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

CREATE TABLE inventory_snapshots (
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    product_id UUID NOT NULL REFERENCES products(id),
    on_hand_qty_base NUMERIC(18, 4) NOT NULL DEFAULT 0,
    avg_cost NUMERIC(18, 4),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (warehouse_id, product_id)
);

CREATE OR REPLACE VIEW v_purchase_order_receipt_status AS
SELECT po.id AS po_id, po.doc_no AS po_no, po.status AS po_status, po.partner_id, po.warehouse_id, po.doc_date AS po_date,
    COALESCE(SUM(pol.qty), 0) AS ordered_qty, COALESCE(SUM(grnl.received_qty), 0) AS received_qty,
    CASE WHEN COALESCE(SUM(grnl.received_qty), 0) = 0 THEN 'pending' WHEN COALESCE(SUM(grnl.received_qty), 0) < COALESCE(SUM(pol.qty), 0) THEN 'partial' ELSE 'complete' END AS receipt_status
FROM documents po LEFT JOIN document_lines pol ON po.id = pol.document_id
LEFT JOIN (SELECT grn.source_doc_id, grnl.product_id, SUM(grnl.qty) AS received_qty FROM documents grn JOIN document_lines grnl ON grn.id = grnl.document_id WHERE grn.doc_type = 'GRN' AND grn.status = 'approved' GROUP BY grn.source_doc_id, grnl.product_id) grnl ON po.id = grnl.source_doc_id AND pol.product_id = grnl.product_id
WHERE po.doc_type = 'PO' GROUP BY po.id, po.doc_no, po.status, po.partner_id, po.warehouse_id, po.doc_date;

CREATE OR REPLACE VIEW v_low_stock_alerts AS
SELECT p.id AS product_id, p.sku, p.name AS product_name, p.spec, p.category_code, p.safety_stock, p.safety_stock_uom, p.reorder_point, p.reorder_point_uom, w.id AS warehouse_id, w.code AS warehouse_code, w.name AS warehouse_name,
    COALESCE(inv.on_hand_qty_base, 0) AS on_hand_qty, p.base_uom, CASE WHEN COALESCE(inv.on_hand_qty_base, 0) <= 0 THEN 'out_of_stock' WHEN p.safety_stock IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.safety_stock THEN 'below_safety' WHEN p.reorder_point IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.reorder_point THEN 'below_reorder' ELSE 'normal' END AS stock_status
FROM products p CROSS JOIN warehouses w LEFT JOIN inventory_snapshots inv ON p.id = inv.product_id AND w.id = inv.warehouse_id
WHERE p.is_active = true AND w.is_active = true AND (COALESCE(inv.on_hand_qty_base, 0) <= 0 OR (p.safety_stock IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.safety_stock) OR (p.reorder_point IS NOT NULL AND COALESCE(inv.on_hand_qty_base, 0) < p.reorder_point));

CREATE OR REPLACE VIEW v_expiry_alerts AS
SELECT p.id AS product_id, p.sku, p.name AS product_name, sl.warehouse_id, w.code AS warehouse_code, sl.batch_no, sl.expiry_date, SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) AS on_hand_qty, p.base_uom, sl.expiry_date - CURRENT_DATE AS days_until_expiry
FROM stock_ledger sl JOIN products p ON sl.product_id = p.id JOIN warehouses w ON sl.warehouse_id = w.id
WHERE p.track_expiry = true AND sl.expiry_date IS NOT NULL AND p.is_active = true
GROUP BY p.id, p.sku, p.name, sl.warehouse_id, w.code, sl.batch_no, sl.expiry_date, p.base_uom
HAVING SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) > 0 AND sl.expiry_date <= CURRENT_DATE + 60;

CREATE OR REPLACE VIEW v_inventory_summary AS
SELECT w.id AS warehouse_id, w.code AS warehouse_code, w.name AS warehouse_name, p.id AS product_id, p.sku, p.name AS product_name, p.spec, p.category_code, p.base_uom, COALESCE(inv.on_hand_qty_base, 0) AS on_hand_qty, COALESCE(inv.avg_cost, 0) AS avg_cost, COALESCE(inv.on_hand_qty_base, 0) * COALESCE(inv.avg_cost, 0) AS total_value, inv.updated_at AS last_updated
FROM products p CROSS JOIN warehouses w LEFT JOIN inventory_snapshots inv ON p.id = inv.product_id AND w.id = inv.warehouse_id WHERE p.is_active = true AND w.is_active = true;
