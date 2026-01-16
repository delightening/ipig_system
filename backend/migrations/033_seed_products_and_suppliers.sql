-- ============================================
-- 遷移 033: 建立五個產品和五個供應商
-- ============================================

-- ============================================
-- 1. 插入五個產品
-- ============================================

INSERT INTO products (
    id,
    sku,
    name,
    spec,
    base_uom,
    track_batch,
    track_expiry,
    safety_stock,
    reorder_point,
    status,
    is_active,
    created_at,
    updated_at
) VALUES
    (
        gen_random_uuid(),
        'DRG-001-001',
        '抗生素注射液',
        '每瓶10ml，濃度100mg/ml，用於治療細菌感染',
        'pcs',
        true,
        true,
        50.0000,
        20.0000,
        'active',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'CNS-002-001',
        '一次性注射器',
        '規格：5ml，無菌包裝，每盒100支',
        'pcs',
        false,
        false,
        500.0000,
        200.0000,
        'active',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'FED-003-001',
        '成長期飼料',
        '蛋白質含量18%，適用於8-20週齡豬隻，每包25kg',
        'kg',
        true,
        true,
        1000.0000,
        500.0000,
        'active',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'EQP-004-001',
        '電子體重計',
        '最大量程500kg，精度0.1kg，適用於豬隻體重測量',
        'pcs',
        false,
        false,
        2.0000,
        1.0000,
        'active',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'CNS-005-001',
        '手術手套',
        '無粉乳膠手套，無菌包裝，尺寸L，每盒100雙',
        'pcs',
        false,
        false,
        200.0000,
        100.0000,
        'active',
        true,
        NOW(),
        NOW()
    )
ON CONFLICT (sku) DO NOTHING;

-- ============================================
-- 2. 插入五個供應商
-- ============================================

INSERT INTO partners (
    id,
    partner_type,
    code,
    name,
    supplier_category,
    tax_id,
    phone,
    email,
    address,
    payment_terms,
    is_active,
    created_at,
    updated_at
) VALUES
    (
        gen_random_uuid(),
        'supplier',
        'SUP001',
        '生技藥品股份有限公司',
        'drug',
        '12345678',
        '02-2345-6789',
        'contact@biopharm.com.tw',
        '台北市信義區信義路五段7號',
        '月結30天',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'supplier',
        'SUP002',
        '醫療器材供應商',
        'consumable',
        '23456789',
        '03-3456-7890',
        'sales@medsupply.com.tw',
        '桃園市中壢區中正路100號',
        '月結45天',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'supplier',
        'SUP003',
        '優質飼料有限公司',
        'feed',
        '34567890',
        '04-4567-8901',
        'info@feedquality.com.tw',
        '台中市西屯區台灣大道三段99號',
        '現金交易',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'supplier',
        'SUP004',
        '精密儀器設備公司',
        'equipment',
        '45678901',
        '06-5678-9012',
        'service@precision.com.tw',
        '台南市東區中華東路200號',
        '月結60天',
        true,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'supplier',
        'SUP005',
        '綜合醫療用品商行',
        'consumable',
        '56789012',
        '07-6789-0123',
        'order@medsupplies.com.tw',
        '高雄市前金區中正四路300號',
        '月結30天',
        true,
        NOW(),
        NOW()
    )
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 完成
-- ============================================
