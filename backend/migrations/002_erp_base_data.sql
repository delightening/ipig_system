-- ============================================
-- Migration 002: ERP 基礎資料
-- 
-- 包含：
-- - SKU 類別與子類別
-- - 豬隻來源
-- - 倉庫
-- - 供應商
-- - 產品
-- ============================================

-- ============================================
-- 1. SKU 類別資料
-- ============================================

-- 插入 SKU 主類別
INSERT INTO sku_categories (code, name, sort_order) VALUES
    ('MED', '藥品', 1),
    ('MSP', '醫材', 2),
    ('FED', '飼料', 3),
    ('EQP', '器材', 4),
    ('CON', '耗材', 5),
    ('CHM', '化學品', 6),
    ('OTH', '其他', 7)
ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 藥品
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MED', 'ANE', '麻醉劑', 1),
    ('MED', 'ANT', '抗生素', 2),
    ('MED', 'VAC', '疫苗', 3),
    ('MED', 'PAI', '止痛劑', 4),
    ('MED', 'DEW', '驅蟲劑', 5),
    ('MED', 'OPH', '眼科藥', 6),
    ('MED', 'TOP', '外用藥', 7),
    ('MED', 'INJ', '注射劑', 8),
    ('MED', 'ORL', '口服藥', 9),
    ('MED', 'OTH', '其他', 10)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 醫材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MSP', 'SYR', '注射器材', 1),
    ('MSP', 'BND', '敷料繃帶', 2),
    ('MSP', 'TUB', '導管管路', 3),
    ('MSP', 'MON', '監測耗材', 4),
    ('MSP', 'SUR', '手術耗材', 5),
    ('MSP', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 飼料
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('FED', 'PIG', '豬用飼料', 1),
    ('FED', 'MIN', '迷你豬飼料', 2),
    ('FED', 'SUP', '營養補充', 3),
    ('FED', 'OTH', '其他', 4)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 器材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('EQP', 'SUR', '手術器材', 1),
    ('EQP', 'MON', '監測設備', 2),
    ('EQP', 'IMG', '影像設備', 3),
    ('EQP', 'ANE', '麻醉設備', 4),
    ('EQP', 'RES', '保定設備', 5),
    ('EQP', 'WEI', '量測設備', 6),
    ('EQP', 'OTH', '其他', 7)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 耗材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CON', 'SYR', '針筒注射', 1),
    ('CON', 'GLV', '手套', 2),
    ('CON', 'GAU', '紗布敷料', 3),
    ('CON', 'TUB', '管路耗材', 4),
    ('CON', 'CLN', '清潔消毒', 5),
    ('CON', 'TAG', '標示耗材', 6),
    ('CON', 'LAB', '實驗耗材', 7),
    ('CON', 'OTH', '其他', 8)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 化學品
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CHM', 'RGT', '試劑', 1),
    ('CHM', 'SOL', '溶劑', 2),
    ('CHM', 'STD', '標準品', 3),
    ('CHM', 'BUF', '緩衝液', 4),
    ('CHM', 'DYE', '染劑', 5),
    ('CHM', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 其他
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('OTH', 'GEN', '一般', 1)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- ============================================
-- 2. 豬隻來源
-- ============================================

INSERT INTO pig_sources (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'TAITUNG', '台東種畜繁殖場', 1),
    (gen_random_uuid(), 'QINGXIN', '青欣牧場', 2),
    (gen_random_uuid(), 'PIGMODEL', '豬博士畜牧場', 3),
    (gen_random_uuid(), 'PINGSHUN', '平順牧場', 4)
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 3. 倉庫
-- ============================================

INSERT INTO warehouses (id, code, name, address, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'WH001', '主倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH002', '藥物倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH003', '飼料倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH004', '醫療器材倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH005', '備用倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 4. 供應商
-- ============================================

INSERT INTO partners (id, partner_type, code, name, supplier_category, tax_id, phone, email, address, payment_terms, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'supplier', 'SUP001', '生技藥品股份有限公司', 'drug', '12345678', '02-2345-6789', 'contact@biopharm.com.tw', '台北市信義區信義路五段7號', '月結30天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP002', '醫療器材供應商', 'consumable', '23456789', '03-3456-7890', 'sales@medsupply.com.tw', '桃園市中壢區中正路100號', '月結45天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP003', '優質飼料有限公司', 'feed', '34567890', '04-4567-8901', 'info@feedquality.com.tw', '台中市西屯區台灣大道三段99號', '現金交易', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP004', '精密儀器設備公司', 'equipment', '45678901', '06-5678-9012', 'service@precision.com.tw', '台南市東區中華東路200號', '月結60天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP005', '綜合醫療用品商行', 'consumable', '56789012', '07-6789-0123', 'order@medsupplies.com.tw', '高雄市前金區中正四路300號', '月結30天', true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 5. 產品
-- ============================================

INSERT INTO products (id, sku, name, spec, base_uom, category_code, subcategory_code, track_batch, track_expiry, default_expiry_days, safety_stock, reorder_point, status, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'MED-ANT-001', '抗生素注射液', '每瓶10ml，濃度100mg/ml，用於治療細菌感染', 'pcs', 'MED', 'ANT', true, true, 365, 50.0000, 20.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'CON-SYR-001', '一次性注射器', '規格：5ml，無菌包裝，每盒100支', 'pcs', 'CON', 'SYR', false, false, NULL, 500.0000, 200.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'FED-PIG-001', '成長期飼料', '蛋白質含量18%，適用於8-20週齡豬隻，每包25kg', 'kg', 'FED', 'PIG', true, true, 180, 1000.0000, 500.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'EQP-WEI-001', '電子體重計', '最大量程500kg，精度0.1kg，適用於豬隻體重測量', 'pcs', 'EQP', 'WEI', false, false, NULL, 2.0000, 1.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'CON-GLV-001', '手術手套', '無粉乳膠手套，無菌包裝，尺寸L，每盒100雙', 'pcs', 'CON', 'GLV', false, false, NULL, 200.0000, 100.0000, 'active', true, NOW(), NOW())
ON CONFLICT (sku) DO NOTHING;

-- ============================================
-- 6. 預設月報表排程
-- ============================================

INSERT INTO scheduled_reports (id, report_type, schedule_type, day_of_month, hour_of_day, recipients, is_active, created_at)
VALUES 
    (gen_random_uuid(), 'stock_on_hand', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'purchase_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'cost_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW())
ON CONFLICT DO NOTHING;

-- ============================================
-- 完成
-- ============================================
