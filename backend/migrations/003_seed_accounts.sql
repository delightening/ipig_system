-- ============================================
-- Migration 003: Seed Accounts
-- 
-- 包含：
-- - 系統管理員帳號
-- - 開發環境預設帳號（由 Rust 在啟動時建立）
-- ============================================

-- ============================================
-- 1. 建立預設管理員帳號
-- 帳號: admin@ipig.local
-- 密碼: admin123
-- ============================================

INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    'admin@ipig.local',
    '$argon2id$v=19$m=19456,t=2,p=1$Z/2b+2ciQvX6LNhEnutXxA$6h0UrmyUFr2YG1KOWuRQo2kaZUqw/ohhP4+bZblmiZM',
    '系統管理員',
    true,
    true,
    false,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO NOTHING;

-- ============================================
-- 2. 為管理員指派 admin 角色
-- ============================================

INSERT INTO user_roles (user_id, role_id, assigned_at)
SELECT u.id, r.id, NOW()
FROM users u, roles r
WHERE u.email = 'admin@ipig.local' AND r.code = 'admin'
ON CONFLICT DO NOTHING;

-- ============================================
-- 3. 為管理員角色指派所有權限
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin'
ON CONFLICT DO NOTHING;

-- ============================================
-- 4. 為管理員建立通知設定
-- ============================================

INSERT INTO notification_settings (user_id)
SELECT id FROM users WHERE email = 'admin@ipig.local'
ON CONFLICT (user_id) DO NOTHING;

-- ============================================
-- 5. 開發環境預設帳號說明
-- 這些帳號由 Rust 程式在啟動時動態建立
-- 以便正確產生 argon2 密碼 hash
-- ============================================

-- 帳號列表 (由 Rust main.rs 在啟動時建立):
-- 
-- 1. 怡均 <monkey20531@gmail.com>
--    - 密碼: test123
--    - 角色: 執行秘書 (IACUC_STAFF), 試驗工作人員 (EXPERIMENT_STAFF)
--
-- 2. 莉珊 <lisa82103031@gmail.com>
--    - 密碼: test123
--    - 角色: 試驗工作人員 (EXPERIMENT_STAFF)
--
-- 3. 芮蓁 <museum1925@gmail.com>
--    - 密碼: test123
--    - 角色: 試驗工作人員 (EXPERIMENT_STAFF)
--
-- 4. 映潔 <keytyne@gmail.com>
--    - 密碼: test123
--    - 角色: 試驗工作人員 (EXPERIMENT_STAFF), 倉庫管理員 (WAREHOUSE_MANAGER)
--
-- 5. 永發 <raying80@gmail.com>
--    - 密碼: test123
--    - 角色: 試驗工作人員 (EXPERIMENT_STAFF)
--
-- 6. 意萍 <smen1971@gmail.com>
--    - 密碼: test123
--    - 角色: 試驗工作人員 (EXPERIMENT_STAFF), 倉庫管理員 (WAREHOUSE_MANAGER), 採購人員 (PURCHASING)

-- ============================================
-- 完成
-- ============================================
