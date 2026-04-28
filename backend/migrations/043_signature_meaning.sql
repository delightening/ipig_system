-- R30-10 (D7=A): electronic_signatures.meaning + signature_meaning enum
-- Why: 21 CFR §11.50(a)(3) 要求每筆電子簽章記錄「the meaning (such as review, approval,
--      responsibility, or authorship) associated with the signature」。
--      既有 signature_type (APPROVE/CONFIRM/WITNESS) 是「動作型別」而非「§11.50 字面 meaning」，
--      無法直接對應 review / authorship / responsibility / invalidate 等語意。
-- How: 新增 signature_meaning ENUM（6 個正式值對齊 §11.50(a)(3) 用語 + 1 個 LEGACY 標記
--      升級前的歷史資料），electronic_signatures 加 meaning 欄位。既有資料 backfill
--      'LEGACY_PRE_R30_10'（D8=B 最誠實策略，明示為升級前紀錄）。新資料 NOT NULL。
--
-- meaning 與 signature_type 的關係：
--   meaning 由 service 層依 SignatureType 推導（APPROVE→APPROVE, CONFIRM→CONFIRM,
--   WITNESS→WITNESS）；REVIEW / AUTHOR / INVALIDATE 由 caller 顯式指定（如 invalidate
--   流程使用 INVALIDATE）。前端不需新增欄位。

-- Step 1：建立 signature_meaning ENUM
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'signature_meaning') THEN
        CREATE TYPE signature_meaning AS ENUM (
            'APPROVE',           -- 核准（§11.50 "approval"）
            'REVIEW',            -- 審閱（§11.50 "review"）
            'WITNESS',           -- 見證
            'AUTHOR',            -- 作者責任（§11.50 "authorship"）
            'INVALIDATE',        -- 失效標記（簽章作廢的意思表示）
            'CONFIRM',           -- 確認（§11.50 "responsibility"，如完成記錄）
            'LEGACY_PRE_R30_10'  -- R30-10 升級前的歷史資料（D8=B：最誠實 backfill 策略）
        );
    END IF;
END$$;

-- Step 2：加 meaning 欄（先 nullable，方便 backfill）
ALTER TABLE electronic_signatures
    ADD COLUMN IF NOT EXISTS meaning signature_meaning;

-- Step 3：backfill 既有資料 = 'LEGACY_PRE_R30_10'
UPDATE electronic_signatures
   SET meaning = 'LEGACY_PRE_R30_10'
 WHERE meaning IS NULL;

-- Step 4：加 NOT NULL constraint（backfill 完才能加）
ALTER TABLE electronic_signatures
    ALTER COLUMN meaning SET NOT NULL;

COMMENT ON COLUMN electronic_signatures.meaning IS
'21 CFR §11.50(a)(3) 簽章意義（review/approval/responsibility/authorship 等）。
 由 service 層依 SignatureType 推導或 caller 顯式指定。
 LEGACY_PRE_R30_10 = R30-10 升級前的歷史資料（無顯式 meaning）。';

-- 索引：合規報表常用「按 meaning 查」（例如「列出所有 APPROVE 簽章」）
CREATE INDEX IF NOT EXISTS idx_esig_meaning ON electronic_signatures(meaning);
