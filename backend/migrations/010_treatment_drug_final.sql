-- ============================================
-- Migration 010: 治療藥物選項去重與業務鍵唯一約束
-- ============================================
-- 依業務鍵 (name, category) 分組合併重複項，再建立部分唯一索引，確保啟用中僅一筆。

-- 10.1 建立合併對照表並更新引用後軟刪除重複列
CREATE TEMP TABLE drug_merge_map (canonical_id UUID, duplicate_id UUID);

INSERT INTO drug_merge_map (canonical_id, duplicate_id)
WITH ranked AS (
  SELECT id,
    ROW_NUMBER() OVER (
      PARTITION BY lower(trim(name)), COALESCE(category, '')
      ORDER BY (erp_product_id IS NOT NULL) DESC NULLS LAST, created_at ASC
    ) AS rn
  FROM treatment_drug_options
),
canonical AS (
  SELECT id, lower(trim(name)) AS nk, COALESCE(category, '') AS ck FROM ranked WHERE rn = 1
),
dups AS (
  SELECT id, lower(trim(name)) AS nk, COALESCE(category, '') AS ck FROM ranked WHERE rn > 1
)
SELECT c.id, d.id FROM dups d JOIN canonical c ON d.nk = c.nk AND d.ck = c.ck;

-- 10.2 更新 animal_observations.treatments 中的 drug_option_id
DO $$
DECLARE
  r RECORD;
  obs_rec RECORD;
  new_treatments jsonb;
  elem jsonb;
BEGIN
  FOR r IN SELECT canonical_id, duplicate_id FROM drug_merge_map
  LOOP
    FOR obs_rec IN
      SELECT id, treatments FROM animal_observations
      WHERE treatments IS NOT NULL AND treatments != '[]'::jsonb
        AND treatments::text LIKE '%' || r.duplicate_id::text || '%'
    LOOP
      new_treatments := '[]'::jsonb;
      FOR elem IN SELECT * FROM jsonb_array_elements(obs_rec.treatments)
      LOOP
        IF (elem->>'drug_option_id') IS NOT NULL AND (elem->>'drug_option_id')::uuid = r.duplicate_id THEN
          elem := jsonb_set(elem, '{drug_option_id}', to_jsonb(r.canonical_id::text));
        END IF;
        new_treatments := new_treatments || jsonb_build_array(elem);
      END LOOP;
      UPDATE animal_observations SET treatments = new_treatments WHERE id = obs_rec.id;
    END LOOP;
  END LOOP;
END $$;

-- 10.3 更新 animal_surgeries 的 JSONB 欄位（將重複 option id 替換為 canonical）
DO $$
DECLARE
  r RECORD;
BEGIN
  FOR r IN SELECT canonical_id, duplicate_id FROM drug_merge_map
  LOOP
    UPDATE animal_surgeries SET
      induction_anesthesia = replace(induction_anesthesia::text, '"' || r.duplicate_id::text || '"', '"' || r.canonical_id::text || '"')::jsonb
    WHERE induction_anesthesia IS NOT NULL AND induction_anesthesia::text LIKE '%' || r.duplicate_id::text || '%';

    UPDATE animal_surgeries SET
      pre_surgery_medication = replace(pre_surgery_medication::text, '"' || r.duplicate_id::text || '"', '"' || r.canonical_id::text || '"')::jsonb
    WHERE pre_surgery_medication IS NOT NULL AND pre_surgery_medication::text LIKE '%' || r.duplicate_id::text || '%';

    UPDATE animal_surgeries SET
      anesthesia_maintenance = replace(anesthesia_maintenance::text, '"' || r.duplicate_id::text || '"', '"' || r.canonical_id::text || '"')::jsonb
    WHERE anesthesia_maintenance IS NOT NULL AND anesthesia_maintenance::text LIKE '%' || r.duplicate_id::text || '%';

    UPDATE animal_surgeries SET
      post_surgery_medication = replace(post_surgery_medication::text, '"' || r.duplicate_id::text || '"', '"' || r.canonical_id::text || '"')::jsonb
    WHERE post_surgery_medication IS NOT NULL AND post_surgery_medication::text LIKE '%' || r.duplicate_id::text || '%';
  END LOOP;
END $$;

-- 10.4 軟刪除重複的藥物選項
UPDATE treatment_drug_options SET is_active = false, updated_at = NOW()
WHERE id IN (SELECT duplicate_id FROM drug_merge_map);

-- 10.5 業務鍵唯一約束：同一「名稱 + 分類」僅允許一筆啟用中的藥物選項
CREATE UNIQUE INDEX IF NOT EXISTS idx_treatment_drug_options_business_key
ON treatment_drug_options (lower(trim(name)), COALESCE(category, ''))
WHERE is_active = true;
