-- ============================================
-- 遷移 038: 為 IACUC_STAFF (執行秘書) 角色添加刪除計畫權限
-- 說明: 
--   執行秘書應具有以下完整 AUP 權限：
--   1. AUP 完整權限
--   2. 上傳附件、下載附件、刪除附件、查看附件
--   3. 刪除計畫、建立計畫（為他PI建立計畫）、查看所有計畫
--   4. 提交計畫、審查計畫、編輯計畫、變更狀態
--   5. 指派審查人員、查看審查意見、新增審查意見、編輯審查意見
--   6. 查看版本歷史、還原版本
-- 
--   此遷移添加缺失的「刪除計畫」權限 (aup.protocol.delete)
-- ============================================

-- ============================================
-- 1. 為 IACUC_STAFF 角色添加刪除計畫權限
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'IACUC_STAFF'
AND p.code = 'aup.protocol.delete'
ON CONFLICT DO NOTHING;

-- ============================================
-- 2. 驗證權限分配
-- ============================================

-- 確認 IACUC_STAFF 角色現在擁有所有必需的 AUP 權限
DO $$
DECLARE
    role_uuid UUID;
    permission_count INTEGER;
    required_permissions TEXT[] := ARRAY[
        'aup.protocol.view_all',
        'aup.protocol.view_own',
        'aup.protocol.create',
        'aup.protocol.edit',
        'aup.protocol.delete',
        'aup.protocol.submit',
        'aup.protocol.change_status',
        'aup.review.view',
        'aup.review.assign',
        'aup.review.comment',
        'aup.review.edit',
        'aup.review.delete',
        'aup.protocol.review',
        'aup.attachment.upload',
        'aup.attachment.view',
        'aup.attachment.download',
        'aup.attachment.delete',
        'aup.version.view',
        'aup.version.restore'
    ];
    perm TEXT;
    missing_permissions TEXT[];
BEGIN
    -- 獲取 IACUC_STAFF 角色的 UUID
    SELECT id INTO role_uuid
    FROM roles
    WHERE code = 'IACUC_STAFF';
    
    IF role_uuid IS NULL THEN
        RAISE WARNING 'IACUC_STAFF 角色不存在';
        RETURN;
    END IF;
    
    -- 檢查每個必需的權限
    FOREACH perm IN ARRAY required_permissions
    LOOP
        SELECT COUNT(*) INTO permission_count
        FROM role_permissions rp
        JOIN permissions p ON rp.permission_id = p.id
        WHERE rp.role_id = role_uuid
        AND p.code = perm;
        
        IF permission_count = 0 THEN
            missing_permissions := array_append(missing_permissions, perm);
        END IF;
    END LOOP;
    
    IF array_length(missing_permissions, 1) > 0 THEN
        RAISE WARNING 'IACUC_STAFF 角色缺少以下權限: %', array_to_string(missing_permissions, ', ');
    ELSE
        RAISE NOTICE 'IACUC_STAFF 角色已擁有所有必需的 AUP 權限（共 % 個）', array_length(required_permissions, 1);
    END IF;
END $$;

-- ============================================
-- 完成
-- ============================================
