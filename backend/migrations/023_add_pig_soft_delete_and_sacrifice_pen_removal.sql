-- Add soft delete support for pigs and automatic pen removal on sacrifice
-- 為動物添加軟刪除支援，並在犧牲時自動移除欄位編號

-- Add deleted_at column to pigs table for soft delete
-- 為 pigs 表添加 deleted_at 欄位以支援軟刪除
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Create index for soft delete queries (only index non-deleted records)
-- 為軟刪除查詢創建索引（僅索引未刪除的記錄）
CREATE INDEX IF NOT EXISTS idx_pigs_deleted_at ON pigs(deleted_at) WHERE deleted_at IS NULL;

-- Add trigger function to automatically remove pen_location and mark as dead when sacrifice record is created
-- 添加觸發器函數，當犧牲記錄創建時自動移除欄位編號並標記為死亡
CREATE OR REPLACE FUNCTION handle_sacrifice_record()
RETURNS TRIGGER AS $$
BEGIN
    -- When a sacrifice record is created (INSERT), always mark the animal as dead and remove pen_location
    -- 當犧牲記錄創建時（INSERT），總是標記動物為死亡並移除欄位編號
    IF TG_OP = 'INSERT' THEN
        -- Remove pen_location and update status to completed
        -- 移除欄位編號並更新狀態為已完成
        UPDATE pigs
        SET pen_location = NULL,
            status = 'completed',
            updated_at = NOW()
        WHERE id = NEW.pig_id
        AND deleted_at IS NULL;
    END IF;
    
    -- When a sacrifice record is updated, ensure pen_location is removed if it wasn't already
    -- 當犧牲記錄更新時，確保欄位編號已被移除（如果尚未移除）
    IF TG_OP = 'UPDATE' THEN
        UPDATE pigs
        SET pen_location = NULL,
            status = 'completed',
            updated_at = NOW()
        WHERE id = NEW.pig_id
        AND deleted_at IS NULL
        AND pen_location IS NOT NULL;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on pig_sacrifices table
-- 在 pig_sacrifices 表上創建觸發器
DROP TRIGGER IF EXISTS trigger_handle_sacrifice_record ON pig_sacrifices;
CREATE TRIGGER trigger_handle_sacrifice_record
    AFTER INSERT OR UPDATE ON pig_sacrifices
    FOR EACH ROW
    EXECUTE FUNCTION handle_sacrifice_record();

-- Update existing sacrifice records to remove pen_location if they exist
-- 更新現有的犧牲記錄，如果存在則移除欄位編號
UPDATE pigs
SET pen_location = NULL,
    status = 'completed',
    updated_at = NOW()
WHERE id IN (SELECT pig_id FROM pig_sacrifices)
AND deleted_at IS NULL
AND pen_location IS NOT NULL;
