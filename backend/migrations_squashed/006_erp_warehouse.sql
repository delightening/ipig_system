-- Migration 006: ERP Warehouse & Accounting / Squashed from: 007(7.2), 009(partial), 011, 012, 021, 022(partial)

-- ============================================
-- 1. 倉庫與產品基礎表
-- ============================================

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
CREATE INDEX idx_product_categories_parent_id ON product_categories(parent_id);

-- SKU 品類 (from 009)
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
CREATE INDEX idx_product_uom_conversions_product_id ON product_uom_conversions(product_id);

-- ============================================
-- 2. 合作夥伴
-- ============================================

CREATE TABLE partners (
    id UUID PRIMARY KEY,
    partner_type partner_type NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    supplier_category supplier_category,
    customer_category customer_category,
    tax_id VARCHAR(50),
    phone VARCHAR(50),
    phone_ext VARCHAR(20),
    email VARCHAR(255),
    address TEXT,
    payment_terms VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_partners_code ON partners(code);
CREATE INDEX idx_partners_partner_type ON partners(partner_type);

-- ============================================
-- 3. 儲位
-- ============================================

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

-- ============================================
-- 4. 單據（含 protocol_id from 011）
-- ============================================

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
    protocol_id UUID REFERENCES protocols(id) ON DELETE SET NULL,
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
CREATE INDEX idx_documents_protocol_id ON documents(protocol_id);
-- FK indexes (from 022)
CREATE INDEX idx_documents_warehouse_id ON documents(warehouse_id);
CREATE INDEX idx_documents_warehouse_from_id ON documents(warehouse_from_id);
CREATE INDEX idx_documents_warehouse_to_id ON documents(warehouse_to_id);
CREATE INDEX idx_documents_partner_id ON documents(partner_id);
CREATE INDEX idx_documents_source_doc_id ON documents(source_doc_id);

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
-- Indexes from 012 and 022
CREATE INDEX idx_document_lines_product_id ON document_lines(product_id);
CREATE INDEX idx_document_lines_storage_location_id ON document_lines(storage_location_id);

-- ============================================
-- 5. 儲位庫存
-- ============================================

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

-- ============================================
-- 6. 庫存分類帳
-- ============================================

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
-- Indexes from 012
CREATE INDEX idx_stock_ledger_wh_prod_dir ON stock_ledger(warehouse_id, product_id, direction);
CREATE INDEX idx_stock_ledger_wh_prod_date ON stock_ledger(warehouse_id, product_id, trx_date DESC);
CREATE INDEX idx_stock_ledger_doc_id ON stock_ledger(doc_id);
-- FK index from 022
CREATE INDEX idx_stock_ledger_line_id ON stock_ledger(line_id);

-- ============================================
-- 7. 庫存快照
-- ============================================

CREATE TABLE inventory_snapshots (
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    product_id UUID NOT NULL REFERENCES products(id),
    on_hand_qty_base NUMERIC(18, 4) NOT NULL DEFAULT 0,
    avg_cost NUMERIC(18, 4),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (warehouse_id, product_id)
);

-- ============================================
-- 8. Views（最終版本，合併 012 / 021）
-- ============================================

-- 採購單收貨狀態 (from 007)
CREATE OR REPLACE VIEW v_purchase_order_receipt_status AS
SELECT po.id AS po_id, po.doc_no AS po_no, po.status AS po_status, po.partner_id, po.warehouse_id, po.doc_date AS po_date,
    COALESCE(SUM(pol.qty), 0) AS ordered_qty, COALESCE(SUM(grnl.received_qty), 0) AS received_qty,
    CASE WHEN COALESCE(SUM(grnl.received_qty), 0) = 0 THEN 'pending' WHEN COALESCE(SUM(grnl.received_qty), 0) < COALESCE(SUM(pol.qty), 0) THEN 'partial' ELSE 'complete' END AS receipt_status
FROM documents po LEFT JOIN document_lines pol ON po.id = pol.document_id
LEFT JOIN (SELECT grn.source_doc_id, grnl.product_id, SUM(grnl.qty) AS received_qty FROM documents grn JOIN document_lines grnl ON grn.id = grnl.document_id WHERE grn.doc_type = 'GRN' AND grn.status = 'approved' GROUP BY grn.source_doc_id, grnl.product_id) grnl ON po.id = grnl.source_doc_id AND pol.product_id = grnl.product_id
WHERE po.doc_type = 'PO' GROUP BY po.id, po.doc_no, po.status, po.partner_id, po.warehouse_id, po.doc_date;

-- 低庫存警告 (012 最終版 — 使用 inventory_snapshots)
CREATE OR REPLACE VIEW v_low_stock_alerts AS
SELECT
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    inv.warehouse_id,
    w.code AS warehouse_code,
    inv.on_hand_qty_base AS on_hand_qty,
    p.base_uom,
    CASE
        WHEN inv.on_hand_qty_base <= 0 THEN 'out_of_stock'
        WHEN p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock THEN 'below_safety'
        WHEN p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point THEN 'below_reorder'
    END AS stock_status
FROM inventory_snapshots inv
JOIN products p ON inv.product_id = p.id
JOIN warehouses w ON inv.warehouse_id = w.id
WHERE p.is_active AND w.is_active
  AND (
    inv.on_hand_qty_base <= 0
    OR (p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock)
    OR (p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point)
  );

-- 庫存總覽 (012 最終版)
CREATE OR REPLACE VIEW v_inventory_summary AS
SELECT
    w.id AS warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.base_uom,
    p.category_code,
    inv.on_hand_qty_base,
    inv.avg_cost,
    p.safety_stock,
    p.reorder_point
FROM inventory_snapshots inv
JOIN warehouses w ON inv.warehouse_id = w.id
JOIN products p ON inv.product_id = p.id
WHERE p.is_active AND w.is_active
ORDER BY w.code, p.sku;

-- 效期預警 (021 最終版)
CREATE VIEW v_expiry_alerts AS
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
    SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) AS on_hand_qty,
    p.base_uom,
    sl.expiry_date - CURRENT_DATE AS days_until_expiry,
    CASE WHEN sl.expiry_date < CURRENT_DATE THEN 'expired' ELSE 'expiring_soon' END AS expiry_status,
    COALESCE(inv.on_hand_qty_base, 0) AS total_qty
FROM stock_ledger sl
JOIN products p ON sl.product_id = p.id
JOIN warehouses w ON sl.warehouse_id = w.id
LEFT JOIN inventory_snapshots inv ON inv.product_id = p.id AND inv.warehouse_id = sl.warehouse_id
WHERE p.track_expiry = true AND sl.expiry_date IS NOT NULL AND p.is_active = true
GROUP BY p.id, p.sku, p.name, p.spec, p.category_code, sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date, p.base_uom, inv.on_hand_qty_base
HAVING SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) > 0 AND sl.expiry_date <= CURRENT_DATE + 60;

-- ============================================
-- 9. 會計 (from 009)
-- ============================================

CREATE TABLE chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    account_type account_type NOT NULL,
    parent_id UUID REFERENCES chart_of_accounts(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chart_of_accounts_code ON chart_of_accounts(code);

CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_no VARCHAR(50) NOT NULL UNIQUE,
    entry_date DATE NOT NULL,
    description TEXT,
    source_entity_type VARCHAR(50),
    source_entity_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_journal_entries_date ON journal_entries(entry_date);
CREATE SEQUENCE IF NOT EXISTS journal_entry_no_seq START 1;

CREATE TABLE journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    line_no INTEGER NOT NULL,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    debit_amount NUMERIC(18, 4) NOT NULL DEFAULT 0,
    credit_amount NUMERIC(18, 4) NOT NULL DEFAULT 0,
    description TEXT,
    UNIQUE (journal_entry_id, line_no),
    CONSTRAINT chk_debit_credit CHECK ((debit_amount >= 0 AND credit_amount >= 0) AND ((debit_amount > 0 AND credit_amount = 0) OR (debit_amount = 0 AND credit_amount > 0)))
);
CREATE INDEX idx_journal_entry_lines_entry ON journal_entry_lines(journal_entry_id);

CREATE TABLE ap_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_no VARCHAR(50) NOT NULL UNIQUE,
    partner_id UUID NOT NULL REFERENCES partners(id),
    payment_date DATE NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    reference TEXT,
    journal_entry_id UUID REFERENCES journal_entries(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE SEQUENCE IF NOT EXISTS ap_payment_no_seq START 1;
CREATE INDEX idx_ap_payments_partner ON ap_payments(partner_id);

CREATE TABLE ar_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receipt_no VARCHAR(50) NOT NULL UNIQUE,
    partner_id UUID NOT NULL REFERENCES partners(id),
    receipt_date DATE NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    reference TEXT,
    journal_entry_id UUID REFERENCES journal_entries(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE SEQUENCE IF NOT EXISTS ar_receipt_no_seq START 1;
CREATE INDEX idx_ar_receipts_partner ON ar_receipts(partner_id);

-- 會計科目種子資料
INSERT INTO chart_of_accounts (id, code, name, account_type) VALUES
(gen_random_uuid(),'1100','現金及約當現金','asset'),(gen_random_uuid(),'1200','應收帳款','asset'),(gen_random_uuid(),'1300','存貨','asset'),
(gen_random_uuid(),'2100','應付帳款','liability'),(gen_random_uuid(),'3100','業主權益','equity'),(gen_random_uuid(),'4100','銷貨收入','revenue'),
(gen_random_uuid(),'5100','進貨成本','expense'),(gen_random_uuid(),'5200','銷貨成本','expense')
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 10. SKU 品類種子資料 (from 009)
-- ============================================

INSERT INTO sku_categories (code, name, sort_order, is_active, created_at) VALUES
  ('GEN', '通用', 0, true, NOW()),
  ('DRG', '藥品', 10, true, NOW()),
  ('MED', '醫材', 20, true, NOW()),
  ('CON', '耗材', 30, true, NOW()),
  ('CHM', '化學品', 40, true, NOW()),
  ('EQP', '設備', 50, true, NOW())
ON CONFLICT (code) DO NOTHING;

INSERT INTO sku_subcategories (category_code, code, name, sort_order, is_active, created_at) VALUES
  ('GEN', 'OTH', '其他', 0, true, NOW()),
  ('DRG', 'ABX', '抗生素', 10, true, NOW()),
  ('DRG', 'ANL', '止痛藥', 20, true, NOW()),
  ('DRG', 'VIT', '維生素', 30, true, NOW()),
  ('DRG', 'OTH', '其他藥品', 40, true, NOW()),
  ('CON', 'GLV', '手套', 10, true, NOW()),
  ('CON', 'GAU', '紗布敷料', 20, true, NOW()),
  ('CON', 'CLN', '清潔消毒', 30, true, NOW()),
  ('CON', 'TAG', '標示耗材', 40, true, NOW()),
  ('CON', 'LAB', '實驗耗材', 50, true, NOW()),
  ('CON', 'OTH', '其他耗材', 60, true, NOW()),
  ('CHM', 'RGT', '試劑', 10, true, NOW()),
  ('CHM', 'SOL', '溶劑', 20, true, NOW()),
  ('CHM', 'STD', '標準品', 30, true, NOW()),
  ('CHM', 'OTH', '其他化學品', 40, true, NOW()),
  ('EQP', 'INS', '儀器', 10, true, NOW()),
  ('EQP', 'TOL', '工具', 20, true, NOW()),
  ('EQP', 'PRT', '零件', 30, true, NOW()),
  ('EQP', 'OTH', '其他設備', 40, true, NOW()),
  ('MED', 'MED', '醫材', 0, true, NOW())
ON CONFLICT (category_code, code) DO NOTHING;

-- ============================================
-- 跨模組 FK 約束（003 中 treatment_drug_options.erp_product_id 延遲到 products 建立後）
-- ============================================
ALTER TABLE treatment_drug_options ADD CONSTRAINT treatment_drug_options_erp_product_id_fkey
    FOREIGN KEY (erp_product_id) REFERENCES products(id);
