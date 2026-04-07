-- 018: 更新 care_medication_records 評估欄位，符合 TU-03-05-03B 試驗豬隻疼痛評估紀錄表
-- 舊欄位（spirit, mobility_standing, mobility_walking 等）與 PDF 表單不符，全部替換

ALTER TABLE care_medication_records
    DROP COLUMN IF EXISTS spirit,
    DROP COLUMN IF EXISTS mobility_standing,
    DROP COLUMN IF EXISTS mobility_walking,
    DROP COLUMN IF EXISTS attitude_behavior,
    DROP COLUMN IF EXISTS appetite,
    ADD COLUMN incision             SMALLINT,
    ADD COLUMN attitude_behavior    SMALLINT,
    ADD COLUMN appetite             SMALLINT,
    ADD COLUMN feces                SMALLINT,
    ADD COLUMN urine                SMALLINT,
    ADD COLUMN pain_score           SMALLINT,
    ADD COLUMN injection_ketorolac  BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN injection_meloxicam  BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN oral_meloxicam       BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN care_medication_records.incision             IS '傷口狀況 0=正常 1=輕微透明滲出液/鮮紅血液 2=不透明滲出液/暗褐血液 3=膿樣分泌物';
COMMENT ON COLUMN care_medication_records.attitude_behavior    IS '態度/行為 0=正常 1=皮膚外觀改變 2=步態/姿勢異常 3=反應遲鈍持續自殘 4=焦慮緊張 5=叫聲迴避攻擊';
COMMENT ON COLUMN care_medication_records.appetite             IS '食慾 0=正常 1=飼料未吃完 2=飼料不吃且對零食無興趣';
COMMENT ON COLUMN care_medication_records.feces                IS '排便 0=正常 1=量減少 2=排便異常(軟便/下痢/血便) 3=未排便';
COMMENT ON COLUMN care_medication_records.urine                IS '排尿 0=正常 1=次數增加或減少 2=尿色異常 3=未排尿';
COMMENT ON COLUMN care_medication_records.pain_score           IS '疼痛分數 1=第一級 2=第二級 3=第三級 4=第四級';
COMMENT ON COLUMN care_medication_records.injection_ketorolac  IS '術後給藥：注射 Ketorolac';
COMMENT ON COLUMN care_medication_records.injection_meloxicam  IS '術後給藥：注射 Meloxicam';
COMMENT ON COLUMN care_medication_records.oral_meloxicam       IS '術後給藥：口服 Meloxicam';
