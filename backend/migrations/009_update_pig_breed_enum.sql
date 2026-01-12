-- Update pig_breed enum to use 'minipig' instead of 'miniature'
-- Version: 0.9
-- Date: 2026-01-10

-- 由於 PostgreSQL 不支援直接修改 ENUM 值，我們需要：
-- 1. 添加新的 enum 值
-- 2. 更新現有資料
-- 3. 刪除舊的 enum 值

-- 但這很複雜，更簡單的方法是：
-- 如果沒有現有資料，刪除並重建 enum 類型
-- 如果有現有資料，需要遷移資料

-- 檢查是否有現有資料
DO $$
BEGIN
    -- 如果沒有使用 'miniature' 的資料，可以直接修改
    IF NOT EXISTS (SELECT 1 FROM pigs WHERE breed::text = 'miniature') THEN
        -- 刪除並重建 enum（僅在沒有資料時安全）
        -- 注意：這會刪除整個 enum 類型，需要先刪除依賴的表或列
        -- 所以我們改用添加新值並更新現有資料的方式
        
        -- 添加新值（如果不存在）
        IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'minipig' AND enumtypid = 'pig_breed'::regtype) THEN
            ALTER TYPE pig_breed ADD VALUE IF NOT EXISTS 'minipig';
        END IF;
        
        -- 更新所有使用 'miniature' 的資料為 'minipig'
        -- 注意：PostgreSQL 不允許直接更新 enum 值，所以需要：
        -- 1. 添加臨時列
        -- 2. 複製資料並轉換
        -- 3. 刪除舊列
        -- 4. 重命名新列
        
        -- 由於這很複雜，我們改為在應用層處理映射
        -- 但為了向後兼容，我們同時支持 'miniature' 和 'minipig'
        
    ELSE
        -- 有現有資料，添加新值作為別名
        IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'minipig' AND enumtypid = 'pig_breed'::regtype) THEN
            ALTER TYPE pig_breed ADD VALUE IF NOT EXISTS 'minipig';
        END IF;
    END IF;
END $$;

-- 如果沒有現有資料，我們可以直接重建 enum
-- 但為了安全，我們採用更保守的方法：添加 'minipig' 作為新值
-- 應用層會將 'miniature' 映射到 'minipig'
