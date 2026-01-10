-- 前端需求補充 Schema
-- Version: 0.3
-- 日期: 2026-01-08

-- ============================================
-- 1. 擴展 products 表
-- ============================================

-- 新增產品擴展欄位
ALTER TABLE products ADD COLUMN IF NOT EXISTS image_url VARCHAR(500);
ALTER TABLE products ADD COLUMN IF NOT EXISTS license_no VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS storage_condition VARCHAR(50);
ALTER TABLE products ADD COLUMN IF NOT EXISTS tags TEXT[];
ALTER TABLE products ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'active';
ALTER TABLE products ADD COLUMN IF NOT EXISTS default_expiry_days INTEGER;
ALTER TABLE products ADD COLUMN IF NOT EXISTS category_code CHAR(3);
ALTER TABLE products ADD COLUMN IF NOT EXISTS subcategory_code CHAR(3);
ALTER TABLE products ADD COLUMN IF NOT EXISTS barcode VARCHAR(50);
ALTER TABLE products ADD COLUMN IF NOT EXISTS created_by UUID REFERENCES users(id);
ALTER TABLE products ADD COLUMN IF NOT EXISTS safety_stock_uom VARCHAR(20);
ALTER TABLE products ADD COLUMN IF NOT EXISTS reorder_point_uom VARCHAR(20);
ALTER TABLE products ADD COLUMN IF NOT EXISTS remark TEXT;

-- 新增產品狀態檢查約束
ALTER TABLE products DROP CONSTRAINT IF EXISTS chk_product_status;
ALTER TABLE products ADD CONSTRAINT chk_product_status 
    CHECK (status IN ('active', 'inactive', 'discontinued'));

-- 新增索引
CREATE INDEX IF NOT EXISTS idx_products_category_code ON products(category_code);
CREATE INDEX IF NOT EXISTS idx_products_subcategory_code ON products(subcategory_code);
CREATE INDEX IF NOT EXISTS idx_products_status ON products(status);
CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode);

-- ============================================
-- 2. 擴展 users 表（登入鎖定 + 偏好設定）
-- ============================================

-- 登入失敗鎖定
ALTER TABLE users ADD COLUMN IF NOT EXISTS login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;

-- 使用者偏好設定
ALTER TABLE users ADD COLUMN IF NOT EXISTS theme_preference VARCHAR(20) NOT NULL DEFAULT 'light';
ALTER TABLE users ADD COLUMN IF NOT EXISTS language_preference VARCHAR(10) NOT NULL DEFAULT 'zh-TW';

-- 主題檢查約束
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_theme_preference;
ALTER TABLE users ADD CONSTRAINT chk_theme_preference 
    CHECK (theme_preference IN ('light', 'dark', 'system'));

-- 語言檢查約束
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_language_preference;
ALTER TABLE users ADD CONSTRAINT chk_language_preference 
    CHECK (language_preference IN ('zh-TW', 'en'));

-- ============================================
-- 3. 擴展 documents 表
-- ============================================

-- 來源單據關聯（採購單 → 入庫單）
ALTER TABLE documents ADD COLUMN IF NOT EXISTS source_doc_id UUID REFERENCES documents(id);

-- 盤點範圍（循環盤點用）
ALTER TABLE documents ADD COLUMN IF NOT EXISTS stocktake_scope JSONB;

-- 採購單入庫狀態
ALTER TABLE documents ADD COLUMN IF NOT EXISTS receipt_status VARCHAR(20);
ALTER TABLE documents DROP CONSTRAINT IF EXISTS chk_receipt_status;
ALTER TABLE documents ADD CONSTRAINT chk_receipt_status 
    CHECK (receipt_status IS NULL OR receipt_status IN ('pending', 'partial', 'complete'));

-- 新增索引
CREATE INDEX IF NOT EXISTS idx_documents_source_doc_id ON documents(source_doc_id);

-- ============================================
-- 4. 通知系統表
-- ============================================

-- 通知類型
CREATE TYPE notification_type AS ENUM (
    'low_stock',           -- 低庫存
    'expiry_warning',      -- 效期預警
    'document_approval',   -- 單據待核准
    'protocol_status',     -- 計畫狀態變更
    'vet_recommendation',  -- 獸醫師建議
    'system_alert',        -- 系統提醒
    'monthly_report'       -- 月報表
);

-- 通知表
CREATE TABLE IF NOT EXISTS notifications (
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

CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(user_id, is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_type ON notifications(type);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);

-- 通知設定表
CREATE TABLE IF NOT EXISTS notification_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- Email 通知設定
    email_low_stock BOOLEAN NOT NULL DEFAULT true,
    email_expiry_warning BOOLEAN NOT NULL DEFAULT true,
    email_document_approval BOOLEAN NOT NULL DEFAULT true,
    email_protocol_status BOOLEAN NOT NULL DEFAULT true,
    email_monthly_report BOOLEAN NOT NULL DEFAULT true,
    -- 效期預警設定
    expiry_warning_days INTEGER NOT NULL DEFAULT 30,
    -- 低庫存預警設定
    low_stock_notify_immediately BOOLEAN NOT NULL DEFAULT true,
    -- 更新時間
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 5. 定期報表系統表
-- ============================================

-- 排程類型
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');

-- 報表類型
CREATE TYPE report_type AS ENUM (
    'stock_on_hand',      -- 庫存現況
    'stock_ledger',       -- 庫存流水
    'purchase_summary',   -- 採購彙總
    'cost_summary',       -- 成本彙總
    'expiry_report',      -- 效期報表
    'low_stock_report'    -- 低庫存報表
);

-- 定期報表設定
CREATE TABLE IF NOT EXISTS scheduled_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type report_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    -- 排程設定
    day_of_week INTEGER,      -- 0-6 (週日-週六), weekly 用
    day_of_month INTEGER,     -- 1-31, monthly 用
    hour_of_day INTEGER NOT NULL DEFAULT 6,  -- 0-23, 執行時間
    -- 報表參數
    parameters JSONB,
    -- 接收者
    recipients UUID[] NOT NULL,
    -- 狀態
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    -- 稽核
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduled_reports_next_run ON scheduled_reports(next_run_at) WHERE is_active = true;

-- 報表歷史記錄
CREATE TABLE IF NOT EXISTS report_history (
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

CREATE INDEX IF NOT EXISTS idx_report_history_type ON report_history(report_type);
CREATE INDEX IF NOT EXISTS idx_report_history_generated_at ON report_history(generated_at);

-- ============================================
-- 6. 補充 SKU 類別（醫材、化學品）
-- ============================================

-- 新增醫材類別
INSERT INTO sku_categories (code, name, sort_order) VALUES
    ('MSP', '醫材', 2)
ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 新增化學品類別
INSERT INTO sku_categories (code, name, sort_order) VALUES
    ('CHM', '化學品', 6)
ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 更新其他類別排序
UPDATE sku_categories SET sort_order = 3 WHERE code = 'FED';
UPDATE sku_categories SET sort_order = 4 WHERE code = 'EQP';
UPDATE sku_categories SET sort_order = 5 WHERE code = 'CON';
UPDATE sku_categories SET sort_order = 7 WHERE code = 'OTH';

-- 補充醫材子類別
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MSP', 'SYR', '注射器材', 1),
    ('MSP', 'BND', '敷料繃帶', 2),
    ('MSP', 'TUB', '導管管路', 3),
    ('MSP', 'MON', '監測耗材', 4),
    ('MSP', 'SUR', '手術耗材', 5),
    ('MSP', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 補充化學品子類別
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CHM', 'RGT', '試劑', 1),
    ('CHM', 'SOL', '溶劑', 2),
    ('CHM', 'STD', '標準品', 3),
    ('CHM', 'BUF', '緩衝液', 4),
    ('CHM', 'DYE', '染劑', 5),
    ('CHM', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 補充實驗耗材子類別到 CON
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CON', 'LAB', '實驗耗材', 8)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- ============================================
-- 7. 新增預設通知設定 Function
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

DROP TRIGGER IF EXISTS trg_create_notification_settings ON users;
CREATE TRIGGER trg_create_notification_settings
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_notification_settings();

-- 為現有使用者建立通知設定
INSERT INTO notification_settings (user_id)
SELECT id FROM users
ON CONFLICT (user_id) DO NOTHING;

-- ============================================
-- 8. 新增採購入庫關聯視圖
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

-- ============================================
-- 9. 新增庫存預警視圖
-- ============================================

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
-- 10. 新增預設月報表排程
-- ============================================

-- 建立預設月報表（每月 1 日早上 6 點產生）
INSERT INTO scheduled_reports (id, report_type, schedule_type, day_of_month, hour_of_day, recipients, is_active, created_at)
VALUES 
    (gen_random_uuid(), 'stock_on_hand', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'purchase_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'cost_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW())
ON CONFLICT DO NOTHING;

-- ============================================
-- 11. 權限補充
-- ============================================

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

-- 產品圖片權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'product.upload_image', '上傳產品圖片', 'erp', '可上傳產品圖片', NOW())
ON CONFLICT (code) DO NOTHING;
