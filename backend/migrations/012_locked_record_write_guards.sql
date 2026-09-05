-- 已鎖定紀錄的資料庫層級寫入護欄（GLP §11.10(e)(1)：簽章後的紀錄不可修改）
--
-- 編號 012：origin/main 當時最大號為 009，010 被 PR #53
-- （`claude/adoring-northcutt-1fa8a8`，外部 PI 代簽授權）佔用、011 被未合的
-- `feat/uom-conversion-migration` 佔用。依 RULES_BACKEND §9 停下請示後，
-- 使用者 2026-09-02 裁定本支取 012。
-- ⚠️ 若那兩支最後沒有落地，prod 會出現缺號——同 §9 記載過 145 缺號的前例。
--
-- ## 這支 migration 補什麼
--
-- 下列 5 張表都有 `is_locked` 欄位，語意是「簽章鎖定後不可修改」：
--
--   animal_observations / animal_surgeries / animal_sacrifices /
--   animal_blood_tests / care_medication_records
--
-- 但在此之前，保護**完全只存在於 service 層**
-- （`SignatureService::ensure_not_locked_uuid` / `_tx`）。任何繞過 service 的寫入
-- 路徑——raw SQL、維運工具、未來忘記呼叫 guard 的新 handler——都能悄悄改掉已鎖定
-- 的紀錄，而且不會在資料庫層級留下任何錯誤或痕跡。
--
-- 既有先例：`journal_entries`、`journal_entry_lines`、`stock_ledger`、
-- `user_activity_logs`、`animal_blood_test_items` 都已經有 immutable / no_delete /
-- no_truncate 觸發器（見 002_schema.sql）。本檔補的是同樣精神的防線，
-- 但擋的是「鎖定後的 UPDATE / DELETE」而不是 TRUNCATE。
--
-- ## 為什麼不是「鎖定後完全禁止 UPDATE」
--
-- 鎖定紀錄上有**合法的旁側更新**：獸醫已讀標記。
-- `AnimalObservationService::mark_vet_read`（services/animal/observation.rs）與
-- `AnimalSurgeryService::mark_vet_read`（services/animal/surgery.rs）會對紀錄下
-- `SET vet_read = true, vet_read_at = NOW(), updated_at = NOW()`，而且**刻意不呼叫**
-- `ensure_not_locked`——已讀是「誰看過這筆」的旁註，不是紀錄內容，鎖定後仍必須能標記。
-- 一律擋 UPDATE 會直接打死這條合法路徑。
--
-- 因此採**逐表 allowlist**：鎖定後只有 allowlist 內的欄位可以動，其餘一律擋。
--
-- ## 為什麼比對整列 jsonb 而不是逐欄列舉
--
-- `check_blood_test_items_immutable` 是逐欄 `IS DISTINCT FROM` 列舉，那種寫法
-- **fail-open**：日後 ALTER TABLE 新增的欄位不在列舉裡，就自動變成沒人保護的破口，
-- 而且不會有任何徵兆。這裡改成「把整列轉 jsonb、扣掉 allowlist 再比對」，
-- 新增欄位會自動落進保護範圍——fail-closed。
--
-- ## 各表 allowlist
--
-- | 表 | 鎖定後仍可更新 | 理由 |
-- |---|---|---|
-- | animal_observations | vet_read, vet_read_at, updated_at | mark_vet_read 實際在走 |
-- | animal_surgeries | vet_read, vet_read_at, updated_at | mark_vet_read 實際在走 |
-- | animal_blood_tests | vet_read, vet_read_at, updated_at | 欄位存在、語意同上（目前尚無寫入端） |
-- | care_medication_records | vet_read | 只有這一欄；本表無 vet_read_at / updated_at |
-- | animal_sacrifices | （無） | 本表沒有 vet_read 類欄位，鎖定後全欄不可動 |
--
-- ⚠️ 附帶影響：`SignatureService::lock_record_uuid` 原本對已鎖定紀錄會再寫一次
-- `locked_at` / `locked_by`（多重簽章時第二簽會覆蓋第一簽的鎖定人）。鎖定欄位不在
-- 任何 allowlist 內，那個覆寫會被本觸發器擋下，所以同一支 PR 把 `lock_record_uuid`
-- 改成 idempotent（`WHERE ... AND is_locked = false`）——鎖定人由此變成「第一個簽的人」，
-- 這對稽核軌跡也才是正確的。

CREATE OR REPLACE FUNCTION public.check_locked_record_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    -- 逐表傳入的 allowlist（CREATE TRIGGER 的 EXECUTE FUNCTION 參數）。
    -- 未傳參數時 TG_ARGV 為空陣列，COALESCE 只是防呆。
    allowed_cols text[] := COALESCE(TG_ARGV, ARRAY[]::text[]);
    changed_cols text[];
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.is_locked THEN
            RAISE EXCEPTION
                '% (id=%) 已被 GLP 簽章鎖鎖定（is_locked = true），不可 DELETE',
                TG_TABLE_NAME, OLD.id
                USING ERRCODE = 'P0001',
                      HINT = '鎖定＝已簽章，紀錄一律保留。刪除請走 service 層的軟刪除，'
                             '而軟刪除本身在鎖定後也會被 ensure_not_locked 拒絕（409）。';
        END IF;
        RETURN OLD;
    END IF;

    -- 未鎖定：本觸發器完全不介入，維持原有行為（含首次上鎖那一次 UPDATE）
    IF NOT OLD.is_locked THEN
        RETURN NEW;
    END IF;

    -- 扣掉 allowlist 後逐欄比對；兩側 key 集合相同，(key, value) 有差就是被改的欄位
    SELECT array_agg(key ORDER BY key)
      INTO changed_cols
      FROM (
          SELECT key, value FROM jsonb_each(to_jsonb(OLD) - allowed_cols)
          EXCEPT
          SELECT key, value FROM jsonb_each(to_jsonb(NEW) - allowed_cols)
      ) AS diff;

    -- 只動到 allowlist 內的欄位（例：獸醫已讀標記）→ 放行
    IF changed_cols IS NULL THEN
        RETURN NEW;
    END IF;

    RAISE EXCEPTION
        '% (id=%) 已被 GLP 簽章鎖鎖定（is_locked = true），不可修改欄位：%',
        TG_TABLE_NAME, OLD.id, array_to_string(changed_cols, ', ')
        USING ERRCODE = 'P0001',
              HINT = format(
                  '本表鎖定後僅允許更新：%s。其餘欄位需更正請走 service 層的更正流程，勿直接 UPDATE。',
                  COALESCE(NULLIF(array_to_string(allowed_cols, ', '), ''), '（無）')
              );
END;
$$;

COMMENT ON FUNCTION public.check_locked_record_immutable() IS
    'GLP §11.10(e)(1)：is_locked = true 的紀錄不可 UPDATE（allowlist 欄位除外）、不可 DELETE。'
    'allowlist 由 CREATE TRIGGER 的 EXECUTE FUNCTION 參數逐表傳入；比對整列 jsonb 而非逐欄列舉，'
    '新增欄位自動納入保護（fail-closed）。';

-- ============================================================
-- 逐表掛載
-- ============================================================

CREATE TRIGGER check_animal_observations_locked_immutable_trigger
    BEFORE UPDATE OR DELETE ON public.animal_observations
    FOR EACH ROW
    EXECUTE FUNCTION public.check_locked_record_immutable('vet_read', 'vet_read_at', 'updated_at');

CREATE TRIGGER check_animal_surgeries_locked_immutable_trigger
    BEFORE UPDATE OR DELETE ON public.animal_surgeries
    FOR EACH ROW
    EXECUTE FUNCTION public.check_locked_record_immutable('vet_read', 'vet_read_at', 'updated_at');

CREATE TRIGGER check_animal_blood_tests_locked_immutable_trigger
    BEFORE UPDATE OR DELETE ON public.animal_blood_tests
    FOR EACH ROW
    EXECUTE FUNCTION public.check_locked_record_immutable('vet_read', 'vet_read_at', 'updated_at');

CREATE TRIGGER check_care_medication_records_locked_immutable_trigger
    BEFORE UPDATE OR DELETE ON public.care_medication_records
    FOR EACH ROW
    EXECUTE FUNCTION public.check_locked_record_immutable('vet_read');

-- animal_sacrifices 沒有 vet_read 類欄位：鎖定後全欄不可動
CREATE TRIGGER check_animal_sacrifices_locked_immutable_trigger
    BEFORE UPDATE OR DELETE ON public.animal_sacrifices
    FOR EACH ROW
    EXECUTE FUNCTION public.check_locked_record_immutable();

-- ============================================================
-- 欄位註解：把「保護在哪一層」更新成現況
-- ============================================================

COMMENT ON COLUMN public.animal_observations.is_locked IS
    'GLP 簽章鎖。true 後 update/delete 會被 service 層 ensure_not_locked guard 拒絕（409），'
    '並由 check_animal_observations_locked_immutable_trigger 在 DB 層兜底（vet_read 類欄位除外）。';
COMMENT ON COLUMN public.animal_surgeries.is_locked IS
    'GLP 簽章鎖。同 animal_observations.is_locked，DB 層由 '
    'check_animal_surgeries_locked_immutable_trigger 兜底。';
COMMENT ON COLUMN public.animal_blood_tests.is_locked IS
    'GLP 簽章鎖。同 animal_observations.is_locked，DB 層由 '
    'check_animal_blood_tests_locked_immutable_trigger 兜底。';
COMMENT ON COLUMN public.care_medication_records.is_locked IS
    'GLP 簽章鎖。DB 層由 check_care_medication_records_locked_immutable_trigger 兜底；'
    '鎖定後僅 vet_read 可更新。';
COMMENT ON COLUMN public.animal_sacrifices.is_locked IS
    'GLP 簽章鎖。DB 層由 check_animal_sacrifices_locked_immutable_trigger 兜底；'
    '本表無旁側可更新欄位，鎖定後全欄不可動。';
