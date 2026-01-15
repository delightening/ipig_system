-- ============================================
-- 遷移 032: 重置管理員密碼為 admin123
-- ============================================
-- 此遷移將管理員 (admin@ipig.local) 的密碼重置為 admin123
-- 僅在系統已建立（管理員用戶存在）時執行

DO $$
DECLARE
    v_admin_id UUID;
    v_password_hash VARCHAR(255);
BEGIN
    -- 獲取管理員用戶 ID
    SELECT id INTO v_admin_id FROM users WHERE email = 'admin@ipig.local' LIMIT 1;
    
    IF v_admin_id IS NULL THEN
        RAISE NOTICE 'Admin user (admin@ipig.local) not found. Skipping password reset.';
        RETURN;
    END IF;
    
    -- 設定密碼 hash (admin123)
    -- 使用 Argon2 生成的 hash
    v_password_hash := '$argon2id$v=19$m=19456,t=2,p=1$01WvBNb8/GLMqDRD99P4zQ$U2WL0+jzVscb22eSubJRRMe/l6+eSA2yjsg5KPqONU0';
    
    -- 更新管理員密碼
    UPDATE users
    SET password_hash = v_password_hash,
        must_change_password = false,
        updated_at = NOW()
    WHERE id = v_admin_id;
    
    RAISE NOTICE 'Admin password has been reset to admin123';
END $$;
