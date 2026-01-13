-- Normalize existing ear tags: format numbers < 100 to 3 digits with leading zeros
-- 規範已存在的耳號：將 < 100 的數字補零至三位數

-- Update ear tags that are pure numbers < 100
UPDATE pigs
SET ear_tag = LPAD(ear_tag, 3, '0')
WHERE ear_tag ~ '^[0-9]+$'  -- Only pure numbers
  AND ear_tag::INTEGER < 100  -- Only numbers < 100
  AND LENGTH(ear_tag) < 3;    -- Only if not already 3 digits
