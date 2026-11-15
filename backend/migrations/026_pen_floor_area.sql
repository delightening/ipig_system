-- 欄位地板面積：每個欄位記錄長與寬（公尺），面積由程式算出，不另存。
--
-- 用途：依動物體重與同欄隻數，對照最小地板面積表判斷欄位是否超標（只警示，不擋操作）。
-- 兩欄皆可為空：沒填尺寸的欄位不做面積檢查，沿用整數容量（pens.capacity）。
-- 不預填任何設施的實際尺寸；由管理員透過欄位管理頁輸入或「套用到整個區」。
--
-- 只新增可空欄位與一條檢查，不改既有資料，套用後既有欄位全部維持空白。

ALTER TABLE pens
    ADD COLUMN length_m numeric(5,2),
    ADD COLUMN width_m numeric(5,2);

ALTER TABLE pens
    ADD CONSTRAINT pens_dimensions_both_or_none CHECK (
        (length_m IS NULL AND width_m IS NULL)
        OR (length_m > 0 AND width_m > 0)
    );

COMMENT ON COLUMN pens.length_m IS '欄位長度（公尺）；與 width_m 同時有值或同時為空';
COMMENT ON COLUMN pens.width_m IS '欄位寬度（公尺）；與 length_m 同時有值或同時為空';
