-- 品類／子類種子資料（與前端「新增產品」一致，單一來源；業界主流：主資料集中於 DB）
-- 執行後 GET /api/sku/categories 與 subcategories 有資料，匯入／新增產品品類下拉與顯示名稱一致

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

-- MED 名義上無子類，以 MED-MED 表示，故插入一筆 (MED, MED) 供 SKU 產生與驗證
