-- ============================================
-- 遷移 017: 開發環境預設帳號
-- 注意: 這些帳號只在開發環境使用
-- 密碼由 Rust 應用程式在啟動時設定
-- ============================================

-- ============================================
-- 1. 確保採購人員角色存在
-- ============================================

INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'PURCHASING', '採購人員', '負責採購作業、建立採購單、管理供應商', true, true, NOW(), NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_internal = EXCLUDED.is_internal,
    is_system = EXCLUDED.is_system,
    updated_at = NOW();

-- ============================================
-- 2. 為採購人員角色指派 ERP 採購相關權限
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'PURCHASING'
  AND p.code IN (
    -- 基礎資料查看
    'erp.warehouse.view',
    'erp.product.view',
    'erp.partner.view',
    'erp.partner.create',
    'erp.partner.edit',
    -- 採購相關
    'erp.document.view',
    'erp.document.create',
    'erp.document.edit',
    'erp.document.submit',
    'erp.purchase.create',
    'erp.grn.create',
    'erp.pr.create',
    -- 庫存查看
    'erp.stock.view',
    -- 報表
    'erp.report.view',
    'erp.report.download'
  )
ON CONFLICT DO NOTHING;

-- ============================================
-- 3. 更新角色權限分配
-- ============================================

-- 3.1 執行秘書（IACUC_STAFF）增加權限確認
-- 執行秘書已有 AUP 完整權限，這裡確認一下
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'IACUC_STAFF'
  AND p.code IN (
    'aup.protocol.view_all',
    'aup.protocol.change_status',
    'aup.review.view',
    'aup.review.assign',
    'aup.attachment.view',
    'aup.attachment.download',
    'aup.version.view',
    'pig.pig.view_all'
  )
ON CONFLICT DO NOTHING;

-- 3.2 試驗工作人員（EXPERIMENT_STAFF）增加豬隻管理權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
  AND p.code IN (
    -- 豬隻基本操作
    'pig.pig.view_all',
    'pig.pig.edit',
    -- 紀錄管理
    'pig.record.view',
    'pig.record.create',
    'pig.record.edit',
    'pig.record.observation',
    'pig.record.surgery',
    'pig.record.weight',
    'pig.record.vaccine',
    'pig.record.sacrifice',
    -- 匯出
    'pig.export.medical',
    'pig.export.observation',
    'pig.export.surgery',
    'pig.export.experiment'
  )
ON CONFLICT DO NOTHING;

-- 3.3 倉庫管理員（WAREHOUSE_MANAGER）增加完整 ERP 權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'WAREHOUSE_MANAGER'
  AND p.module = 'erp'
ON CONFLICT DO NOTHING;

-- ============================================
-- 4. 預設開發帳號資訊（由 Rust 程式動態建立）
-- ============================================

-- 這裡只是記錄預設帳號的規格，實際建立由 Rust 應用程式處理
-- 以便能正確設定 argon2 密碼 hash

-- 帳號列表：
-- 1. 怡均 <monkey20531@gmail.com>    - 執行秘書、試驗工作人員
-- 2. 莉珊 <lisa82103031@gmail.com>   - 試驗工作人員
-- 3. 芮蓁 <museum1925@gmail.com>     - 試驗工作人員
-- 4. 映潔 <keytyne@gmail.com>        - 試驗工作人員、倉庫管理員
-- 5. 永發 <raying80@gmail.com>       - 試驗工作人員
-- 6. 意萍 <smen1971@gmail.com>       - 試驗工作人員、倉庫管理員、採購人員
