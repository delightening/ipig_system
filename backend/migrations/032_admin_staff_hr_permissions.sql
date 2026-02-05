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
