-- ============================================
-- Migration 007: ERP Warehouse System
-- 
-- 包含：
-- - 倉庫表
-- - 產品類別與產品表
-- - 夥伴表 (供應商/客戶)
-- - 單據表
-- - 庫存流水表
-- - 庫存快照表
-- - 儲位管理
-- - 視圖 (Views)
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 倉庫表
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
CREATE INDEX idx_warehouses_is_active ON warehouses(is_active);

-- ============================================
-- 2. 產品類別表
-- ============================================

CREATE TABLE product_categories (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES product_categories(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_product_categories_parent ON product_categories(parent_id);

-- ============================================
-- 3. SKU 類別表
-- ============================================

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


-- ============================================

-- ============================================
-- 4. 產品表
-- ============================================

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

-- ============================================
-- 5. 夥伴表 (供應商/客戶)
-- ============================================

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
-- 6. 單據表
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
-- 7. 庫存流水表
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
CREATE INDEX idx_stock_ledger_doc_id ON stock_ledger(doc_id);
CREATE INDEX idx_stock_ledger_product_id ON stock_ledger(product_id);

-- ============================================
-- 8. 庫存快照表
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
-- 9. 儲位管理
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
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(warehouse_id, code)
);

CREATE INDEX idx_storage_locations_warehouse ON storage_locations(warehouse_id);
CREATE INDEX idx_storage_locations_zone ON storage_locations(zone);

-- 儲位庫存表
CREATE TABLE storage_location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    storage_location_id UUID NOT NULL REFERENCES storage_locations(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    on_hand_qty NUMERIC(18, 4) NOT NULL DEFAULT 0,
    batch_no VARCHAR(50),
    expiry_date DATE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date))
);

CREATE INDEX idx_storage_location_inventory_location ON storage_location_inventory(storage_location_id);
CREATE INDEX idx_storage_location_inventory_product ON storage_location_inventory(product_id);

-- ============================================
-- 10. 視圖 (Views)
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

-- 庫存總覽視圖
CREATE OR REPLACE VIEW v_inventory_summary AS
SELECT 
    w.id AS warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.spec,
    p.category_code,
    p.base_uom,
    COALESCE(inv.on_hand_qty_base, 0) AS on_hand_qty,
    COALESCE(inv.avg_cost, 0) AS avg_cost,
    COALESCE(inv.on_hand_qty_base, 0) * COALESCE(inv.avg_cost, 0) AS total_value,
    inv.updated_at AS last_updated
FROM products p
CROSS JOIN warehouses w
LEFT JOIN inventory_snapshots inv ON p.id = inv.product_id AND w.id = inv.warehouse_id
WHERE p.is_active = true AND w.is_active = true;

-- ============================================
-- 完成
-- ============================================
