-- 進銷存系統初始 Schema
-- Version: 0.1

-- ============================================
-- 自訂類型 (Custom Types)
-- ============================================

-- 夥伴類型
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');

-- 單據類型
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ');

-- 單據狀態
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');

-- 庫存流水方向
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

-- ============================================
-- 用戶與權限相關表
-- ============================================

-- 用戶表
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- 角色表
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 權限表
CREATE TABLE permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
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

-- ============================================
-- 基礎資料表 (Master Data)
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

-- 產品表
CREATE TABLE products (
    id UUID PRIMARY KEY,
    sku VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    spec TEXT,
    category_id UUID REFERENCES product_categories(id),
    base_uom VARCHAR(20) NOT NULL DEFAULT 'pcs',
    track_batch BOOLEAN NOT NULL DEFAULT false,
    track_expiry BOOLEAN NOT NULL DEFAULT false,
    safety_stock NUMERIC(18, 4),
    reorder_point NUMERIC(18, 4),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_category_id ON products(category_id);
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
-- 單據相關表
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
    doc_date DATE NOT NULL,
    remark TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ
);

CREATE INDEX idx_documents_doc_type ON documents(doc_type);
CREATE INDEX idx_documents_doc_no ON documents(doc_no);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_doc_date ON documents(doc_date);
CREATE INDEX idx_documents_warehouse_id ON documents(warehouse_id);
CREATE INDEX idx_documents_partner_id ON documents(partner_id);
CREATE INDEX idx_documents_created_by ON documents(created_by);

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
-- 庫存相關表
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

-- 庫存快照表 (可選，用於提升查詢效能)
CREATE TABLE inventory_snapshots (
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    product_id UUID NOT NULL REFERENCES products(id),
    on_hand_qty_base NUMERIC(18, 4) NOT NULL DEFAULT 0,
    avg_cost NUMERIC(18, 4),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (warehouse_id, product_id)
);

-- ============================================
-- 稽核日誌表
-- ============================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    actor_user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    before_data JSONB,
    after_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- ============================================
-- 初始資料 (Seed Data)
-- ============================================

-- 預設權限
INSERT INTO permissions (id, code, name, created_at) VALUES
    -- 用戶管理
    (gen_random_uuid(), 'user.create', '建立用戶', NOW()),
    (gen_random_uuid(), 'user.read', '檢視用戶', NOW()),
    (gen_random_uuid(), 'user.update', '更新用戶', NOW()),
    (gen_random_uuid(), 'user.delete', '刪除用戶', NOW()),
    -- 角色管理
    (gen_random_uuid(), 'role.create', '建立角色', NOW()),
    (gen_random_uuid(), 'role.read', '檢視角色', NOW()),
    (gen_random_uuid(), 'role.update', '更新角色', NOW()),
    (gen_random_uuid(), 'role.delete', '刪除角色', NOW()),
    -- 倉庫管理
    (gen_random_uuid(), 'warehouse.create', '建立倉庫', NOW()),
    (gen_random_uuid(), 'warehouse.read', '檢視倉庫', NOW()),
    (gen_random_uuid(), 'warehouse.update', '更新倉庫', NOW()),
    (gen_random_uuid(), 'warehouse.delete', '刪除倉庫', NOW()),
    -- 產品管理
    (gen_random_uuid(), 'product.create', '建立產品', NOW()),
    (gen_random_uuid(), 'product.read', '檢視產品', NOW()),
    (gen_random_uuid(), 'product.update', '更新產品', NOW()),
    (gen_random_uuid(), 'product.delete', '刪除產品', NOW()),
    -- 夥伴管理
    (gen_random_uuid(), 'partner.create', '建立夥伴', NOW()),
    (gen_random_uuid(), 'partner.read', '檢視夥伴', NOW()),
    (gen_random_uuid(), 'partner.update', '更新夥伴', NOW()),
    (gen_random_uuid(), 'partner.delete', '刪除夥伴', NOW()),
    -- 單據管理
    (gen_random_uuid(), 'document.read', '檢視單據', NOW()),
    (gen_random_uuid(), 'document.update', '更新單據', NOW()),
    (gen_random_uuid(), 'document.submit', '送審單據', NOW()),
    (gen_random_uuid(), 'document.approve', '核准單據', NOW()),
    (gen_random_uuid(), 'document.cancel', '作廢單據', NOW()),
    -- 採購單
    (gen_random_uuid(), 'po.create', '建立採購單', NOW()),
    (gen_random_uuid(), 'grn.create', '建立採購入庫', NOW()),
    (gen_random_uuid(), 'pr.create', '建立採購退貨', NOW()),
    -- 銷售單
    (gen_random_uuid(), 'so.create', '建立銷售單', NOW()),
    (gen_random_uuid(), 'do.create', '建立銷售出庫', NOW()),
    (gen_random_uuid(), 'sr.create', '建立銷售退貨', NOW()),
    -- 倉儲作業
    (gen_random_uuid(), 'tr.create', '建立調撥單', NOW()),
    (gen_random_uuid(), 'stk.create', '建立盤點單', NOW()),
    (gen_random_uuid(), 'adj.create', '建立調整單', NOW()),
    -- 庫存
    (gen_random_uuid(), 'stock.read', '檢視庫存', NOW()),
    (gen_random_uuid(), 'stock.adjust', '調整庫存', NOW()),
    -- 報表
    (gen_random_uuid(), 'report.read', '檢視報表', NOW()),
    (gen_random_uuid(), 'report.export', '匯出報表', NOW());

-- 預設角色
INSERT INTO roles (id, code, name, created_at, updated_at) VALUES
    (gen_random_uuid(), 'admin', '系統管理員', NOW(), NOW()),
    (gen_random_uuid(), 'warehouse', '倉管人員', NOW(), NOW()),
    (gen_random_uuid(), 'purchasing', '採購人員', NOW(), NOW()),
    (gen_random_uuid(), 'sales', '業務人員', NOW(), NOW()),
    (gen_random_uuid(), 'approver', '主管', NOW(), NOW());

-- 為管理員角色指派所有權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'admin';

-- 為倉管角色指派權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'warehouse' 
  AND p.code IN (
    'warehouse.read', 'product.read', 'partner.read',
    'document.read', 'document.update', 'document.submit',
    'grn.create', 'do.create', 'tr.create', 'stk.create', 'adj.create',
    'stock.read', 'stock.adjust'
  );

-- 為採購角色指派權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'purchasing' 
  AND p.code IN (
    'warehouse.read', 'product.read', 'partner.read', 'partner.create', 'partner.update',
    'document.read', 'document.update', 'document.submit',
    'po.create', 'grn.create', 'pr.create',
    'stock.read', 'report.read'
  );

-- 為業務角色指派權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'sales' 
  AND p.code IN (
    'warehouse.read', 'product.read', 'partner.read', 'partner.create', 'partner.update',
    'document.read', 'document.update', 'document.submit',
    'so.create', 'do.create', 'sr.create',
    'stock.read', 'report.read'
  );

-- 為主管角色指派權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'approver' 
  AND p.code IN (
    'warehouse.read', 'product.read', 'partner.read',
    'document.read', 'document.approve', 'document.cancel',
    'stock.read', 'report.read', 'report.export'
  );

-- 建立預設管理員帳號 (密碼: admin123)
-- 密碼 hash 使用 argon2: $argon2id$v=19$m=19456,t=2,p=1$...
INSERT INTO users (id, email, password_hash, display_name, is_active, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    'admin@erp.local',
    '$argon2id$v=19$m=19456,t=2,p=1$Z/2b+2ciQvX6LNhEnutXxA$6h0UrmyUFr2YG1KOWuRQo2kaZUqw/ohhP4+bZblmiZM',
    '系統管理員',
    true,
    NOW(),
    NOW()
);

-- 為預設管理員指派管理員角色
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id 
FROM users u, roles r 
WHERE u.email = 'admin@erp.local' AND r.code = 'admin';
