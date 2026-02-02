-- ============================================
-- Migration 014: Add ADMIN_STAFF (行政) Role
-- 
-- 新增行政角色，可審核加班和請假申請
-- ============================================

-- 插入 ADMIN_STAFF 角色
INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'ADMIN_STAFF', '行政', '行政人員，可審核加班及請假申請', true, true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 完成
-- ============================================
