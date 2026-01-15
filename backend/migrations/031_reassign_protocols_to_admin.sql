-- ============================================
-- 遷移 031: 將10個草稿協議重新分配給管理員
-- ============================================
-- 此遷移確保由 030_create_ten_draft_protocols.sql 創建的協議
-- 都分配給管理員用戶 (admin@ipig.local)

DO $$
DECLARE
    v_admin_id UUID;
    v_protocol_ids UUID[];
    v_protocol_id UUID;
BEGIN
    -- 獲取管理員用戶 ID
    SELECT id INTO v_admin_id FROM users WHERE email = 'admin@ipig.local' LIMIT 1;
    
    IF v_admin_id IS NULL THEN
        RAISE EXCEPTION 'Admin user (admin@ipig.local) not found in database';
    END IF;
    
    -- 找出所有標題包含 "草稿測試協議" 且狀態為 DRAFT 的協議
    -- 這些應該是 migration 030 創建的協議
    SELECT ARRAY_AGG(id) INTO v_protocol_ids
    FROM protocols
    WHERE title LIKE '草稿測試協議 - %'
      AND status = 'DRAFT'
      AND (title LIKE '%營養代謝研究%'
        OR title LIKE '%生長性能研究%'
        OR title LIKE '%肉質評估研究%'
        OR title LIKE '%腸道健康研究%'
        OR title LIKE '%應激反應研究%'
        OR title LIKE '%飼料效率研究%'
        OR title LIKE '%免疫系統研究%'
        OR title LIKE '%骨骼發育研究%'
        OR title LIKE '%心血管健康研究%'
        OR title LIKE '%遺傳育種研究%');
    
    -- 如果找到協議，則重新分配給管理員
    IF v_protocol_ids IS NOT NULL AND array_length(v_protocol_ids, 1) > 0 THEN
        FOREACH v_protocol_id IN ARRAY v_protocol_ids
        LOOP
            -- 更新協議的 PI 用戶 ID
            UPDATE protocols
            SET pi_user_id = v_admin_id,
                updated_at = NOW()
            WHERE id = v_protocol_id;
            
            -- 刪除舊的 user_protocols 關聯
            DELETE FROM user_protocols
            WHERE protocol_id = v_protocol_id;
            
            -- 創建新的 user_protocols 關聯，將協議分配給管理員
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES (v_admin_id, v_protocol_id, 'PI', NOW(), v_admin_id)
            ON CONFLICT (user_id, protocol_id) DO NOTHING;
            
            RAISE NOTICE 'Reassigned protocol % to admin user', v_protocol_id;
        END LOOP;
        
        RAISE NOTICE 'Successfully reassigned % protocols to admin user', array_length(v_protocol_ids, 1);
    ELSE
        RAISE NOTICE 'No protocols found to reassign. They may not have been created yet or already assigned correctly.';
    END IF;
END $$;
