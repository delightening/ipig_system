-- ============================================
-- Migration 032: Default Role Permissions Seed
-- 
-- 為所有角色設定預設權限
-- ============================================

-- ============================================
-- 1. 系統管理員 (admin) - 所有權限
-- ============================================
-- 已在 027_admin_all_permissions.sql 處理

-- ============================================
-- 2. 行政 (ADMIN_STAFF) - 全部 HR + 庫存報表 Audit + 管理階級 Audit
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'ADMIN_STAFF'
  AND p.code IN (
    -- HR 權限（全部 18 個）
    'hr.attendance.view',
    'hr.attendance.view_all',
    'hr.attendance.clock',
    'hr.attendance.correct',
    'hr.overtime.view',
    'hr.overtime.create',
    'hr.overtime.approve',
    'hr.leave.view',
    'hr.leave.view_all',
    'hr.leave.create',
    'hr.leave.approve',
    'hr.leave.manage',
    'hr.balance.view',
    'hr.balance.manage',
    'hr.calendar.config',
    'hr.calendar.view',
    'hr.calendar.sync',
    'hr.calendar.conflicts',
    -- 庫存報表 Audit 權限
    'erp.stock.view',
    'erp.report.view',
    -- 管理階級 Audit 權限（全部 5 個）
    'audit.logs.view',
    'audit.logs.export',
    'audit.timeline.view',
    'audit.alerts.view',
    'audit.alerts.manage'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 3. 執行秘書 (IACUC_STAFF) - 所有 AUP 權限
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'IACUC_STAFF'
  AND p.code IN (
    -- AUP 計畫管理（全部）
    'aup.protocol.view_all',
    'aup.protocol.view_own',
    'aup.protocol.create',
    'aup.protocol.edit',
    'aup.protocol.submit',
    'aup.protocol.review',
    'aup.protocol.approve',
    'aup.protocol.change_status',
    'aup.protocol.delete',
    -- AUP 審查流程（全部）
    'aup.review.view',
    'aup.review.assign',
    'aup.review.comment',
    'aup.review.reply',
    -- AUP 附件管理（全部）
    'aup.attachment.view',
    'aup.attachment.download',
    'aup.attachment.upload',
    'aup.attachment.delete',
    -- AUP 版本管理
    'aup.version.view',
    'aup.version.restore',
    -- AUP 額外功能
    'aup.amendment.classify',
    'aup.coeditor.assign'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 4. 試驗工作人員 (EXPERIMENT_STAFF) - 動物管理
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
  AND p.code IN (
    -- 計畫權限（查看自己的）
    'aup.protocol.view_own',
    'aup.attachment.view',
    'aup.attachment.download',
    -- 動物完整權限
    'animal.animal.view_all',
    'animal.animal.view_project',
    'animal.animal.create',
    'animal.animal.edit',
    'animal.animal.assign',
    'animal.animal.import',
    'animal.animal.delete',
    'animal.record.view',
    'animal.record.create',
    'animal.record.edit',
    'animal.record.delete',
    'animal.record.observation',
    'animal.record.surgery',
    'animal.record.weight',
    'animal.record.vaccine',
    'animal.record.sacrifice',
    'animal.export.medical',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    -- HR 個人權限
    'hr.attendance.view',
    'hr.attendance.clock',
    'hr.overtime.view',
    'hr.overtime.create',
    'hr.leave.view',
    'hr.leave.create',
    'hr.balance.view',
    -- ERP 查看權限
    'erp.product.view',
    'erp.stock.view',
    -- 通知
    'notification.view'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 5. 獸醫師 (VET) - 審查計畫、動物查看、獸醫建議、緊急處置
-- 只看、給建議，不參與現場工作
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'VET'
  AND p.code IN (
    -- AUP 計畫審查
    'aup.protocol.view_all',
    'aup.protocol.review',
    'aup.review.view',
    'aup.review.comment',
    -- AUP 附件
    'aup.attachment.view',
    'aup.attachment.download',
    -- AUP 版本
    'aup.version.view',
    -- Amendment 變更申請（審查、檢視）
    'amendment.read',
    'amendment.review',
    -- 動物管理（只看）
    'animal.animal.view_all',
    'animal.animal.view_project',
    'animal.record.view',
    -- 匯出（所有紀錄）
    'animal.export.medical',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    -- 獸醫師功能（所有）
    'animal.vet.recommend',
    'animal.vet.read',
    -- 緊急處置
    'animal.emergency.stop',
    'animal.euthanasia.recommend',
    'animal.euthanasia.approve'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 6. 倉庫管理員 (WAREHOUSE_MANAGER) - ERP 完整權限
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER'
  AND p.code IN (
    -- ERP 完整權限
    'erp.warehouse.view',
    'erp.warehouse.create',
    'erp.warehouse.edit',
    'erp.product.view',
    'erp.product.create',
    'erp.product.edit',
    'erp.partner.view',
    'erp.partner.create',
    'erp.partner.edit',
    'erp.document.view',
    'erp.document.create',
    'erp.document.edit',
    'erp.document.submit',
    'erp.document.approve',
    'erp.purchase.create',
    'erp.purchase.approve',
    'erp.grn.create',
    'erp.pr.create',
    'erp.stock.in',
    'erp.stock.out',
    'erp.stock.view',
    'erp.stock.adjust',
    'erp.stock.transfer',
    'erp.stocktake.create',
    'erp.report.view',
    'erp.report.export',
    'erp.report.download'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 7. 採購人員 (PURCHASING) - ERP 採購相關權限
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'PURCHASING'
  AND p.code IN (
    -- ERP 採購權限
    'erp.warehouse.view',
    'erp.product.view',
    'erp.partner.view',
    'erp.partner.create',
    'erp.partner.edit',
    'erp.document.view',
    'erp.document.create',
    'erp.document.edit',
    'erp.document.submit',
    'erp.purchase.create',
    'erp.grn.create',
    'erp.pr.create',
    'erp.stock.view',
    'erp.report.view'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 8. 計畫主持人 (PI) - 自己的計畫管理
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'PI'
  AND p.code IN (
    -- 計畫權限（自己的）
    'aup.protocol.view_own',
    'aup.protocol.create',
    'aup.protocol.edit',
    'aup.protocol.submit',
    'aup.attachment.view',
    'aup.attachment.download',
    'aup.version.view',
    -- 動物權限（計畫內）
    'animal.animal.view_project',
    'animal.record.view',
    -- 通知
    'notification.view'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 9. 審查委員 (REVIEWER) - 查看所有計畫、計畫審查權限
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'REVIEWER'
  AND p.code IN (
    -- 計畫審查權限（查看所有計畫）
    'aup.protocol.view_all',
    'aup.protocol.review',
    'aup.review.view',
    'aup.review.comment',
    'aup.attachment.view',
    'aup.attachment.download',
    'aup.version.view',
    -- Amendment 變更申請（審查、檢視）
    'amendment.read',
    'amendment.review'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 10. IACUC 主席 (CHAIR) - 審查決策
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'CHAIR'
  AND p.code IN (
    -- 計畫完整審查權限
    'aup.protocol.view_all',
    'aup.protocol.review',
    'aup.protocol.approve',
    'aup.protocol.change_status',
    'aup.review.view',
    'aup.review.assign',
    'aup.review.comment',
    'aup.attachment.view',
    'aup.attachment.download',
    'aup.version.view',
    -- HR 個人權限
    'hr.attendance.view',
    'hr.attendance.clock',
    'hr.overtime.view',
    'hr.overtime.create',
    'hr.leave.view',
    'hr.leave.create',
    'hr.balance.view',
    -- 通知
    'notification.view'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 11. 委託人 (CLIENT) - 查看委託計畫
-- ============================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'CLIENT'
  AND p.code IN (
    -- 計畫權限（自己的）
    'aup.protocol.view_own',
    'aup.attachment.view',
    -- 動物權限（計畫內）
    'animal.animal.view_project',
    'animal.record.view',
    -- 通知
    'notification.view'
  )
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- ============================================
-- 完成
-- ============================================
-- ============================================
-- Migration 027: Ensure Admin Has All Permissions
-- 
-- 蝣箔? admin 閫??鞈?摨思葉??????-- ?蝘餅??冽?甈⊥憓???蝣箔? admin 隞????湔???-- ============================================

-- ??admin 閫?晷??????INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ??銋 SYSTEM_ADMIN 閫嚗????剁??晷?????INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'SYSTEM_ADMIN'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 憿舐內蝯?
DO $$
DECLARE
    admin_permission_count INTEGER;
    total_permission_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_permission_count FROM permissions;
    
    SELECT COUNT(*) INTO admin_permission_count 
    FROM role_permissions rp
    JOIN roles r ON rp.role_id = r.id
    WHERE r.code = 'admin';
    
    RAISE NOTICE 'Admin role now has % out of % total permissions', admin_permission_count, total_permission_count;
END $$;
-- Migration: 033_rename_pig_to_animal.sql
-- Description: 撠???pig.* 甈?隞?Ⅳ??賢???animal.*
-- Date: 2026-02-06

-- ============================================
-- 1. ????????pig.export.medical ??animal.export.medical嚗?
-- ============================================

-- 撠?pig.export.medical ???脫???蝘餌策 animal.export.medical
INSERT INTO role_permissions (role_id, permission_id)
SELECT rp.role_id, p2.id
FROM role_permissions rp
JOIN permissions p1 ON rp.permission_id = p1.id
JOIN permissions p2 ON p2.code = 'animal.export.medical'
WHERE p1.code = 'pig.export.medical'
ON CONFLICT DO NOTHING;

-- ?芷 pig.export.medical
DELETE FROM permissions WHERE code = 'pig.export.medical';

-- ============================================
-- 2. ?????pig.* 甈???animal.*
-- ============================================

-- pig.pig.* ??animal.animal.*
UPDATE permissions SET code = 'animal.animal.view_all', module = 'animal' WHERE code = 'pig.pig.view_all';
UPDATE permissions SET code = 'animal.animal.view_project', module = 'animal' WHERE code = 'pig.pig.view_project';
UPDATE permissions SET code = 'animal.animal.create', module = 'animal' WHERE code = 'pig.pig.create';
UPDATE permissions SET code = 'animal.animal.edit', module = 'animal' WHERE code = 'pig.pig.edit';
UPDATE permissions SET code = 'animal.animal.assign', module = 'animal' WHERE code = 'pig.pig.assign';
UPDATE permissions SET code = 'animal.animal.import', module = 'animal' WHERE code = 'pig.pig.import';
UPDATE permissions SET code = 'animal.animal.delete', module = 'animal' WHERE code = 'pig.pig.delete';

-- pig.record.* ??animal.record.*
UPDATE permissions SET code = 'animal.record.view', module = 'animal' WHERE code = 'pig.record.view';
UPDATE permissions SET code = 'animal.record.create', module = 'animal' WHERE code = 'pig.record.create';
UPDATE permissions SET code = 'animal.record.edit', module = 'animal' WHERE code = 'pig.record.edit';
UPDATE permissions SET code = 'animal.record.delete', module = 'animal' WHERE code = 'pig.record.delete';
UPDATE permissions SET code = 'animal.record.observation', module = 'animal' WHERE code = 'pig.record.observation';
UPDATE permissions SET code = 'animal.record.surgery', module = 'animal' WHERE code = 'pig.record.surgery';
UPDATE permissions SET code = 'animal.record.weight', module = 'animal' WHERE code = 'pig.record.weight';
UPDATE permissions SET code = 'animal.record.vaccine', module = 'animal' WHERE code = 'pig.record.vaccine';
UPDATE permissions SET code = 'animal.record.sacrifice', module = 'animal' WHERE code = 'pig.record.sacrifice';

-- pig.export.* ??animal.export.*
UPDATE permissions SET code = 'animal.export.observation', module = 'animal' WHERE code = 'pig.export.observation';
UPDATE permissions SET code = 'animal.export.surgery', module = 'animal' WHERE code = 'pig.export.surgery';
UPDATE permissions SET code = 'animal.export.experiment', module = 'animal' WHERE code = 'pig.export.experiment';

-- pig.vet.* ??animal.vet.*
UPDATE permissions SET code = 'animal.vet.recommend', module = 'animal' WHERE code = 'pig.vet.recommend';
UPDATE permissions SET code = 'animal.vet.read', module = 'animal' WHERE code = 'pig.vet.read';

-- ============================================
-- 3. ?湔?膩銝剔??惇?颯???押?
-- ============================================
UPDATE permissions SET description = REPLACE(description, '鞊祇', '?') WHERE code LIKE 'animal.%';

-- ============================================
-- 摰?
-- ============================================
