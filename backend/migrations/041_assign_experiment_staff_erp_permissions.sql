-- ============================================
-- 遷移 041: 為試驗工作人員(EXPERIMENT_STAFF)分配 ERP 權限
-- 說明: 
--   為 EXPERIMENT_STAFF 角色分配 ERP 管理系統權限，包括：
--   1. 倉庫管理：查看、編輯
--   2. 產品管理：查看、建立、編輯、刪除、上傳圖片
--   3. 夥伴管理：查看
--   4. 單據管理：查看、建立、編輯、送審、作廢、建立採購單、建立銷售單、建立銷售出庫
--   5. 庫存作業：查看庫存、盤點
--   6. 報表管理：查看報表
-- ============================================

-- ============================================
-- 1. 為 EXPERIMENT_STAFF 分配 ERP 權限
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    -- 倉庫管理
    'erp.warehouse.view',        -- 查看倉庫
    'erp.warehouse.edit',        -- 編輯倉庫
    
    -- 產品管理
    'erp.product.view',          -- 查看產品/檢視產品
    'erp.product.create',        -- 建立產品
    'erp.product.edit',          -- 編輯產品/更新產品
    'erp.product.delete',        -- 刪除產品
    'erp.product.upload_image',  -- 上傳產品圖片
    
    -- 夥伴管理
    'erp.partner.view',          -- 查看夥伴/檢視夥伴
    
    -- 單據管理
    'erp.document.view',         -- 查看單據
    'erp.document.create',       -- 建立單據
    'erp.document.edit',         -- 編輯單據
    'erp.document.submit',       -- 送審單據
    'erp.document.cancel',       -- 作廢單據
    'erp.purchase.create',       -- 建立採購單
    'erp.sales.create',          -- 建立銷售單
    'erp.do.create',             -- 建立銷售出庫
    
    -- 庫存作業
    'erp.stock.view',            -- 查看庫存
    'erp.stocktake.create',      -- 盤點
    
    -- 報表管理
    'erp.report.view'            -- 查看報表
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 2. 說明與注意事項
-- ============================================

-- 已完成的權限分配：
-- 
-- EXPERIMENT_STAFF (試驗工作人員) ERP 權限：
--   ✅ 倉庫管理: 查看、編輯
--   ✅ 產品管理: 查看、建立、編輯、刪除、上傳圖片
--   ✅ 夥伴管理: 查看
--   ✅ 單據管理: 查看、建立、編輯、送審、作廢、建立採購單、建立銷售單、建立銷售出庫
--   ✅ 庫存作業: 查看庫存、盤點
--   ✅ 報表管理: 查看報表
-- 
-- 注意：
--   1. 所有權限代碼都已在 migration 011 中定義，不需要建立新權限
--   2. 不包含核准權限（如 erp.document.approve、erp.sales.approve 等），
--      試驗工作人員不應有核准權限，僅能建立和送審單據
--   3. 庫存作業僅包含查看和盤點功能，不包含入庫、出庫、調整、調撥等操作
--   4. 報表管理僅包含查看功能，不包含匯出、排程、下載等功能
