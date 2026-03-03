-- 修正操作日誌中「疑苗紀錄」錯字為「疫苗紀錄」
-- 影響：user_activity_logs.entity_display_name
UPDATE user_activity_logs
SET entity_display_name = REPLACE(entity_display_name, '疑苗紀錄', '疫苗紀錄')
WHERE entity_display_name LIKE '%疑苗紀錄%';
