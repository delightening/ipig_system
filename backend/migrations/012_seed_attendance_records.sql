-- ============================================
-- Migration 012: Seed Attendance Records
-- 
-- 為所有 EXPERIMENT_STAFF 使用者建立過去一個月的
-- 出勤紀錄（空白佔位符，不含打卡時間）
-- ============================================

-- 先清除現有的出勤紀錄
DELETE FROM attendance_records;

-- 為 EXPERIMENT_STAFF 角色的使用者建立過去30天的出勤紀錄
DO $$
DECLARE
    v_user_id UUID;
    v_work_date DATE;
    v_start_date DATE := CURRENT_DATE - INTERVAL '30 days';
    v_end_date DATE := CURRENT_DATE;
BEGIN
    -- 遍歷所有具有 EXPERIMENT_STAFF 角色的使用者
    FOR v_user_id IN 
        SELECT DISTINCT u.id 
        FROM users u
        INNER JOIN user_roles ur ON u.id = ur.user_id
        INNER JOIN roles r ON ur.role_id = r.id
        WHERE r.code = 'EXPERIMENT_STAFF' 
          AND u.is_active = true
    LOOP
        -- 為每位使用者建立過去30天的出勤紀錄
        FOR v_work_date IN 
            SELECT generate_series(v_start_date, v_end_date, '1 day'::interval)::date
        LOOP
            -- 只為工作日建立紀錄（排除週六週日）
            IF EXTRACT(DOW FROM v_work_date) NOT IN (0, 6) THEN
                -- 插入空白出勤紀錄（無打卡時間）
                INSERT INTO attendance_records (
                    id, 
                    user_id, 
                    work_date, 
                    clock_in_time,
                    clock_out_time,
                    regular_hours,
                    overtime_hours,
                    status,
                    created_at, 
                    updated_at
                ) VALUES (
                    gen_random_uuid(),
                    v_user_id,
                    v_work_date,
                    NULL,  -- 空白打卡時間
                    NULL,  -- 空白打卡時間
                    0,
                    0,
                    'normal',
                    NOW(),
                    NOW()
                )
                ON CONFLICT (user_id, work_date) DO NOTHING;
            END IF;
        END LOOP;
    END LOOP;
    
    RAISE NOTICE 'Attendance records created for EXPERIMENT_STAFF users';
END $$;

-- ============================================
-- 完成
-- ============================================
