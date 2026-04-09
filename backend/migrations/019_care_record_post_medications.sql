-- Migration 019: 疼痛評估紀錄新增自由格式術後給藥欄位
-- 原有 injection_ketorolac / injection_meloxicam / oral_meloxicam 保留（向後相容）
-- 新增 post_medications JSONB 供 Repeater + DrugCombobox 自由填入

ALTER TABLE care_medication_records
    ADD COLUMN IF NOT EXISTS post_medications JSONB;
