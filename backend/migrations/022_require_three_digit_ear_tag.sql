-- Require ear_tag to be exactly 3 digits for new pig creation
-- 要求新建立的動物耳號必須為三位數

-- First, normalize any existing ear tags that are not exactly 3 digits
-- 首先，規範任何不是三位數的現有耳號
UPDATE pigs
SET ear_tag = LPAD(ear_tag, 3, '0')
WHERE ear_tag ~ '^[0-9]+$'  -- Only pure numbers
  AND LENGTH(ear_tag) != 3;  -- Only if not already 3 digits

-- Add CHECK constraint to ensure ear_tag is exactly 3 digits
-- 添加 CHECK 約束以確保耳號必須為三位數
ALTER TABLE pigs
ADD CONSTRAINT check_ear_tag_three_digits
CHECK (ear_tag ~ '^[0-9]{3}$');
