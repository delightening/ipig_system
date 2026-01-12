-- ============================================
-- 遷移 011: 重新組織權限為四個模組
-- 將權限分成：AUP、豬隻管理、ERP管理、程式設計
-- ============================================

-- ============================================
-- 1. 更新權限模組分類
-- ============================================

-- 將現有權限的 module 欄位更新為新的分類
UPDATE permissions SET module = 'aup' WHERE code LIKE 'aup.%';
UPDATE permissions SET module = 'pig' WHERE code LIKE 'animal.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'erp.%' OR code LIKE 'warehouse.%' OR code LIKE 'product.%' OR code LIKE 'partner.%' OR code LIKE 'document.%' OR code LIKE 'po.%' OR code LIKE 'grn.%' OR code LIKE 'pr.%' OR code LIKE 'so.%' OR code LIKE 'do.%' OR code LIKE 'sr.%' OR code LIKE 'tr.%' OR code LIKE 'stk.%' OR code LIKE 'adj.%' OR code LIKE 'stock.%' OR code LIKE 'report.%';
UPDATE permissions SET module = 'dev' WHERE code LIKE 'admin.%' OR code LIKE 'user.%' OR code LIKE 'role.%' OR code LIKE 'notification.%';

-- ============================================
-- 2. AUP 系統權限（細分）
-- ============================================

-- 2.1 計畫管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.protocol.view_all', '查看所有計畫', 'aup', '可查看系統中所有計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.view_own', '查看自己的計畫', 'aup', '可查看自己相關的計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.create', '建立計畫', 'aup', '可建立新計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.edit', '編輯計畫', 'aup', '可編輯計畫草稿', NOW()),
    (gen_random_uuid(), 'aup.protocol.delete', '刪除計畫', 'aup', '可刪除計畫草稿', NOW()),
    (gen_random_uuid(), 'aup.protocol.submit', '提交計畫', 'aup', '可提交計畫送審', NOW()),
    (gen_random_uuid(), 'aup.protocol.change_status', '變更狀態', 'aup', '可變更計畫狀態', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 2.2 審查流程
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.review.view', '查看審查意見', 'aup', '可查看計畫審查意見', NOW()),
    (gen_random_uuid(), 'aup.review.assign', '指派審查人員', 'aup', '可指派審查人員', NOW()),
    (gen_random_uuid(), 'aup.review.comment', '新增審查意見', 'aup', '可新增審查意見', NOW()),
    (gen_random_uuid(), 'aup.review.edit', '編輯審查意見', 'aup', '可編輯自己的審查意見', NOW()),
    (gen_random_uuid(), 'aup.review.delete', '刪除審查意見', 'aup', '可刪除自己的審查意見', NOW()),
    (gen_random_uuid(), 'aup.protocol.review', '審查計畫', 'aup', '可審查計畫並提供意見', NOW()),
    (gen_random_uuid(), 'aup.protocol.approve', '核准/否決', 'aup', '可核准或否決計畫', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 2.3 附件管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.attachment.upload', '上傳附件', 'aup', '可上傳計畫相關附件', NOW()),
    (gen_random_uuid(), 'aup.attachment.view', '查看附件', 'aup', '可查看計畫附件', NOW()),
    (gen_random_uuid(), 'aup.attachment.download', '下載附件', 'aup', '可下載計畫附件', NOW()),
    (gen_random_uuid(), 'aup.attachment.delete', '刪除附件', 'aup', '可刪除計畫附件', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 2.4 版本管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.version.view', '查看版本歷史', 'aup', '可查看計畫版本歷史', NOW()),
    (gen_random_uuid(), 'aup.version.restore', '還原版本', 'aup', '可還原到指定版本', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- ============================================
-- 3. 豬隻管理系統權限（細分）
-- ============================================

-- 3.1 豬隻資料管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.pig.view_all', '查看所有豬隻', 'pig', '可查看系統中所有豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.pig.view_project', '查看計畫內豬隻', 'pig', '可查看計畫內的豬隻', NOW()),
    (gen_random_uuid(), 'pig.pig.create', '新增豬隻', 'pig', '可新增豬隻', NOW()),
    (gen_random_uuid(), 'pig.pig.edit', '編輯豬隻資料', 'pig', '可編輯豬隻基本資料', NOW()),
    (gen_random_uuid(), 'pig.pig.delete', '刪除豬隻', 'pig', '可刪除豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.pig.assign', '分配豬隻至計畫', 'pig', '可將豬隻分配至計畫', NOW()),
    (gen_random_uuid(), 'pig.pig.unassign', '解除豬隻分配', 'pig', '可解除豬隻的計畫分配', NOW()),
    (gen_random_uuid(), 'pig.pig.import', '匯入豬隻資料', 'pig', '可批次匯入豬隻資料', NOW()),
    (gen_random_uuid(), 'pig.pig.export', '匯出豬隻資料', 'pig', '可匯出豬隻資料', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 3.2 紀錄管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.record.view', '查看紀錄', 'pig', '可查看豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.create', '新增紀錄', 'pig', '可新增豬隻相關紀錄（觀察、手術、體重等）', NOW()),
    (gen_random_uuid(), 'pig.record.edit', '編輯紀錄', 'pig', '可編輯豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.delete', '刪除紀錄', 'pig', '可刪除豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.copy', '複製紀錄', 'pig', '可複製觀察/手術紀錄作為範本', NOW()),
    (gen_random_uuid(), 'pig.record.observation', '觀察紀錄', 'pig', '可新增/編輯觀察試驗紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.surgery', '手術紀錄', 'pig', '可新增/編輯手術紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.weight', '體重紀錄', 'pig', '可新增/編輯體重紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.vaccine', '疫苗紀錄', 'pig', '可新增/編輯疫苗紀錄', NOW()),
    (gen_random_uuid(), 'pig.record.sacrifice', '犧牲紀錄', 'pig', '可新增/編輯犧牲紀錄', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 3.3 獸醫師功能
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.vet.recommend', '新增獸醫師建議', 'pig', '可新增獸醫師建議', NOW()),
    (gen_random_uuid(), 'pig.vet.edit', '編輯獸醫師建議', 'pig', '可編輯獸醫師建議', NOW()),
    (gen_random_uuid(), 'pig.vet.delete', '刪除獸醫師建議', 'pig', '可刪除獸醫師建議', NOW()),
    (gen_random_uuid(), 'pig.vet.read', '標記已讀', 'pig', '可標記紀錄已讀', NOW()),
    (gen_random_uuid(), 'pig.vet.upload_attachment', '上傳附件', 'pig', '可上傳獸醫師建議附件', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 3.4 匯出功能
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.export.medical', '匯出病歷', 'pig', '可匯出豬隻病歷總表', NOW()),
    (gen_random_uuid(), 'pig.export.observation', '匯出觀察紀錄', 'pig', '可匯出豬隻觀察紀錄', NOW()),
    (gen_random_uuid(), 'pig.export.surgery', '匯出手術紀錄', 'pig', '可匯出豬隻手術紀錄', NOW()),
    (gen_random_uuid(), 'pig.export.experiment', '匯出試驗紀錄', 'pig', '可匯出豬隻試驗紀錄', NOW()),
    (gen_random_uuid(), 'pig.export.all', '匯出所有資料', 'pig', '可匯出豬隻所有相關資料', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 3.5 病理報告
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'pig.pathology.view', '查看病理報告', 'pig', '可查看豬隻病理組織報告', NOW()),
    (gen_random_uuid(), 'pig.pathology.upload', '上傳病理報告', 'pig', '可上傳豬隻病理組織報告', NOW()),
    (gen_random_uuid(), 'pig.pathology.edit', '編輯病理報告', 'pig', '可編輯病理組織報告', NOW()),
    (gen_random_uuid(), 'pig.pathology.delete', '刪除病理報告', 'pig', '可刪除病理組織報告', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- ============================================
-- 4. ERP 管理系統權限（細分）
-- ============================================

-- 4.1 基礎資料管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- 倉庫管理
    (gen_random_uuid(), 'erp.warehouse.view', '查看倉庫', 'erp', '可查看倉庫列表', NOW()),
    (gen_random_uuid(), 'erp.warehouse.create', '建立倉庫', 'erp', '可建立新倉庫', NOW()),
    (gen_random_uuid(), 'erp.warehouse.edit', '編輯倉庫', 'erp', '可編輯倉庫資料', NOW()),
    (gen_random_uuid(), 'erp.warehouse.delete', '刪除倉庫', 'erp', '可刪除倉庫', NOW()),
    -- 產品管理
    (gen_random_uuid(), 'erp.product.view', '查看產品', 'erp', '可查看產品列表', NOW()),
    (gen_random_uuid(), 'erp.product.create', '建立產品', 'erp', '可建立新產品', NOW()),
    (gen_random_uuid(), 'erp.product.edit', '編輯產品', 'erp', '可編輯產品資料', NOW()),
    (gen_random_uuid(), 'erp.product.delete', '刪除產品', 'erp', '可刪除產品', NOW()),
    (gen_random_uuid(), 'erp.product.upload_image', '上傳產品圖片', 'erp', '可上傳產品圖片', NOW()),
    -- 夥伴管理
    (gen_random_uuid(), 'erp.partner.view', '查看夥伴', 'erp', '可查看夥伴列表', NOW()),
    (gen_random_uuid(), 'erp.partner.create', '建立夥伴', 'erp', '可建立新夥伴（供應商/客戶）', NOW()),
    (gen_random_uuid(), 'erp.partner.edit', '編輯夥伴', 'erp', '可編輯夥伴資料', NOW()),
    (gen_random_uuid(), 'erp.partner.delete', '刪除夥伴', 'erp', '可刪除夥伴', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 4.2 單據管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- 單據基本操作
    (gen_random_uuid(), 'erp.document.view', '查看單據', 'erp', '可查看所有單據', NOW()),
    (gen_random_uuid(), 'erp.document.create', '建立單據', 'erp', '可建立新單據', NOW()),
    (gen_random_uuid(), 'erp.document.edit', '編輯單據', 'erp', '可編輯草稿單據', NOW()),
    (gen_random_uuid(), 'erp.document.delete', '刪除單據', 'erp', '可刪除草稿單據', NOW()),
    (gen_random_uuid(), 'erp.document.submit', '送審單據', 'erp', '可提交單據送審', NOW()),
    (gen_random_uuid(), 'erp.document.approve', '核准單據', 'erp', '可核准單據', NOW()),
    (gen_random_uuid(), 'erp.document.cancel', '作廢單據', 'erp', '可作廢單據', NOW()),
    -- 採購單據
    (gen_random_uuid(), 'erp.purchase.create', '建立採購單', 'erp', '可建立採購單（PO）', NOW()),
    (gen_random_uuid(), 'erp.purchase.approve', '核准採購單', 'erp', '可核准採購單', NOW()),
    (gen_random_uuid(), 'erp.grn.create', '建立採購入庫', 'erp', '可建立採購入庫單（GRN）', NOW()),
    (gen_random_uuid(), 'erp.pr.create', '建立採購退貨', 'erp', '可建立採購退貨單（PR）', NOW()),
    -- 銷售單據
    (gen_random_uuid(), 'erp.sales.create', '建立銷售單', 'erp', '可建立銷售單（SO）', NOW()),
    (gen_random_uuid(), 'erp.sales.approve', '核准銷售單', 'erp', '可核准銷售單', NOW()),
    (gen_random_uuid(), 'erp.do.create', '建立銷售出庫', 'erp', '可建立銷售出庫單（DO）', NOW()),
    (gen_random_uuid(), 'erp.sr.create', '建立銷售退貨', 'erp', '可建立銷售退貨單（SR）', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 4.3 庫存作業
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'erp.stock.view', '查看庫存', 'erp', '可查看庫存現況', NOW()),
    (gen_random_uuid(), 'erp.stock.in', '入庫操作', 'erp', '可執行入庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.out', '出庫操作', 'erp', '可執行出庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.adjust', '庫存調整', 'erp', '可執行庫存調整', NOW()),
    (gen_random_uuid(), 'erp.stock.transfer', '調撥', 'erp', '可執行庫存調撥', NOW()),
    (gen_random_uuid(), 'erp.stocktake.create', '盤點', 'erp', '可執行庫存盤點', NOW()),
    (gen_random_uuid(), 'erp.stocktake.approve', '核准盤點', 'erp', '可核准盤點結果', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 4.4 報表管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'erp.report.view', '查看報表', 'erp', '可查看 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.export', '匯出報表', 'erp', '可匯出 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.schedule', '排程報表', 'erp', '可設定定期報表', NOW()),
    (gen_random_uuid(), 'erp.report.download', '下載報表', 'erp', '可下載報表檔案', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- ============================================
-- 5. 程式設計系統權限（細分）
-- ============================================

-- 5.1 使用者管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.user.view', '查看使用者', 'dev', '可查看使用者列表', NOW()),
    (gen_random_uuid(), 'dev.user.create', '建立使用者', 'dev', '可建立新使用者帳號', NOW()),
    (gen_random_uuid(), 'dev.user.edit', '編輯使用者', 'dev', '可編輯使用者資料', NOW()),
    (gen_random_uuid(), 'dev.user.delete', '停用使用者', 'dev', '可停用使用者帳號', NOW()),
    (gen_random_uuid(), 'dev.user.reset_password', '重設密碼', 'dev', '可重設他人密碼', NOW()),
    (gen_random_uuid(), 'dev.user.assign_role', '指派角色', 'dev', '可指派角色給使用者', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 5.2 角色與權限管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.role.view', '查看角色', 'dev', '可查看角色列表', NOW()),
    (gen_random_uuid(), 'dev.role.create', '建立角色', 'dev', '可建立新角色', NOW()),
    (gen_random_uuid(), 'dev.role.edit', '編輯角色', 'dev', '可編輯角色資料', NOW()),
    (gen_random_uuid(), 'dev.role.delete', '刪除角色', 'dev', '可刪除角色', NOW()),
    (gen_random_uuid(), 'dev.role.assign_permission', '指派權限', 'dev', '可為角色指派權限', NOW()),
    (gen_random_uuid(), 'dev.permission.view', '查看權限', 'dev', '可查看權限列表', NOW()),
    (gen_random_uuid(), 'dev.permission.manage', '管理權限', 'dev', '可管理權限定義', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 5.3 系統設定
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.system.view', '查看系統設定', 'dev', '可查看系統設定', NOW()),
    (gen_random_uuid(), 'dev.system.edit', '編輯系統設定', 'dev', '可編輯系統設定', NOW()),
    (gen_random_uuid(), 'dev.system.backup', '備份系統', 'dev', '可執行系統備份', NOW()),
    (gen_random_uuid(), 'dev.system.restore', '還原系統', 'dev', '可執行系統還原', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 5.4 稽核與日誌
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.audit.view', '查看稽核紀錄', 'dev', '可查看系統稽核紀錄', NOW()),
    (gen_random_uuid(), 'dev.audit.export', '匯出稽核紀錄', 'dev', '可匯出稽核紀錄', NOW()),
    (gen_random_uuid(), 'dev.log.view', '查看系統日誌', 'dev', '可查看系統日誌', NOW()),
    (gen_random_uuid(), 'dev.log.download', '下載系統日誌', 'dev', '可下載系統日誌', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 5.5 通知管理
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.notification.view', '查看通知', 'dev', '可查看自己的通知', NOW()),
    (gen_random_uuid(), 'dev.notification.manage', '管理通知設定', 'dev', '可管理通知設定', NOW()),
    (gen_random_uuid(), 'dev.notification.send', '發送通知', 'dev', '可發送系統通知', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- 5.6 資料庫管理（進階）
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'dev.database.view', '查看資料庫資訊', 'dev', '可查看資料庫基本資訊', NOW()),
    (gen_random_uuid(), 'dev.database.query', '執行查詢', 'dev', '可執行資料庫查詢（唯讀）', NOW()),
    (gen_random_uuid(), 'dev.database.migrate', '執行遷移', 'dev', '可執行資料庫遷移', NOW()),
    (gen_random_uuid(), 'dev.database.seed', '執行種子資料', 'dev', '可執行種子資料', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- ============================================
-- 6. 更新舊權限代碼對應關係
-- ============================================

-- 將舊的權限代碼更新為新的模組分類（保持向後兼容）
-- 注意：這裡只更新 module，不改變 code，以保持 API 兼容性

-- AUP 權限（已存在，只需更新 module）
UPDATE permissions SET module = 'aup' WHERE code LIKE 'aup.%';

-- 豬隻管理權限（從 animal.* 更新 module 為 'pig'，但保留舊代碼以兼容）
-- 注意：舊代碼 animal.* 仍然有效，只是 module 更新為 'pig'
UPDATE permissions SET module = 'pig' WHERE code LIKE 'animal.%';

-- ERP 權限（更新所有相關權限的 module）
UPDATE permissions SET module = 'erp' WHERE 
    code LIKE 'erp.%' 
    OR code LIKE 'warehouse.%' 
    OR code LIKE 'product.%' 
    OR code LIKE 'partner.%' 
    OR code LIKE 'document.%'
    OR code LIKE 'po.%'
    OR code LIKE 'grn.%'
    OR code LIKE 'pr.%'
    OR code LIKE 'so.%'
    OR code LIKE 'do.%'
    OR code LIKE 'sr.%'
    OR code LIKE 'tr.%'
    OR code LIKE 'stk.%'
    OR code LIKE 'adj.%'
    OR code LIKE 'stock.%'
    OR code LIKE 'report.%';

-- 程式設計權限（從 admin.* 更新 module 為 'dev'，但保留舊代碼以兼容）
-- 注意：舊代碼 admin.*, user.*, role.* 仍然有效，只是 module 更新為 'dev'
UPDATE permissions SET module = 'dev' WHERE 
    code LIKE 'admin.%' 
    OR code LIKE 'user.%' 
    OR code LIKE 'role.%'
    OR code LIKE 'notification.%';

-- ============================================
-- 7. 權限代碼說明（v1.0 之後不再兼容舊代碼）
-- ============================================

-- v1.0 起統一使用新權限代碼（pig.*, dev.*, erp.* 等）
-- 舊代碼（animal.*, admin.*, user.*, role.*）已由後續 migration 清理
