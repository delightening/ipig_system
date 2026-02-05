-- Storage Locations (儲位/貨架) 資料表
-- 用於倉庫內部貨架視覺化佈局管理

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
COMMENT ON TABLE storage_locations IS '倉庫儲位/貨架資料表，用於視覺化平面圖佈局';
COMMENT ON COLUMN storage_locations.location_type IS '儲位類型: shelf(貨架), rack(儲物架), zone(區域), bin(儲物格)';
COMMENT ON COLUMN storage_locations.row_index IS '網格行座標，用於 react-grid-layout 視覺化';
COMMENT ON COLUMN storage_locations.col_index IS '網格列座標，用於 react-grid-layout 視覺化';
COMMENT ON COLUMN storage_locations.width IS '佔用網格寬度 (列數)';
COMMENT ON COLUMN storage_locations.height IS '佔用網格高度 (行數)';
COMMENT ON COLUMN storage_locations.config IS '額外配置 JSON (如溫度範圍、濕度要求等)';

-- 新增權限
INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES 
    (gen_random_uuid(), 'erp.storage_location.view', '檢視儲位', 'ERP', '檢視倉庫儲位/貨架資料', NOW()),
    (gen_random_uuid(), 'erp.storage_location.create', '建立儲位', 'ERP', '建立新的儲位/貨架', NOW()),
    (gen_random_uuid(), 'erp.storage_location.edit', '編輯儲位', 'ERP', '編輯儲位/貨架資料與佈局', NOW()),
    (gen_random_uuid(), 'erp.storage_location.delete', '刪除儲位', 'ERP', '刪除儲位/貨架', NOW())
ON CONFLICT (code) DO NOTHING;

-- 給 admin 角色新增權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin' 
  AND p.code IN ('erp.storage_location.view', 'erp.storage_location.create', 'erp.storage_location.edit', 'erp.storage_location.delete')
ON CONFLICT DO NOTHING;

-- 給倉庫管理員 (WAREHOUSE_MANAGER) 角色新增權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER' 
  AND p.code IN ('erp.storage_location.view', 'erp.storage_location.create', 'erp.storage_location.edit', 'erp.storage_location.delete')
ON CONFLICT DO NOTHING;
