-- 025_field_correction_entry_date.sql
-- 動物欄位修正申請新增可修正欄位：entry_date（進場日期）。
--
-- 使用者 2026-11-12 核准（維護期例外）：進場日期建檔後不可直接改，輸錯時要能走
-- 「申請 → admin 核准」的修正流程，與耳號、出生日期、性別、品種同一條路。
--
-- 為什麼需要 migration：animal_field_correction_requests.field_name 有 CHECK
-- `chk_field_name`（002_schema.sql），只允許 ear_tag / birth_date / gender / breed。
-- 後端放行 entry_date 之後，不放寬這個 CHECK，INSERT 會被資料庫擋下。
--
-- 只放寬、不動既有資料：既有申請列的 field_name 都在新集合內，套用不會因資料不符而失敗。
--
-- 套用不需要停機；舊版 API 仍只會寫入原本四個值，與新 CHECK 相容。

ALTER TABLE public.animal_field_correction_requests
    DROP CONSTRAINT chk_field_name;

ALTER TABLE public.animal_field_correction_requests
    ADD CONSTRAINT chk_field_name
    CHECK (field_name IN ('ear_tag', 'birth_date', 'gender', 'breed', 'entry_date'));

COMMENT ON COLUMN public.animal_field_correction_requests.field_name IS
    '欄位名稱：ear_tag, birth_date, gender, breed, entry_date';

-- Down（手動）：
--   ⚠️ 還原前先確認沒有 entry_date 的申請列，否則加回舊 CHECK 會失敗：
--     SELECT count(*) FROM public.animal_field_correction_requests WHERE field_name = 'entry_date';
--   有的話不要直接刪（那是申請與審核的紀錄）；先由人工決定如何處理再還原。
--
--   ALTER TABLE public.animal_field_correction_requests DROP CONSTRAINT chk_field_name;
--   ALTER TABLE public.animal_field_correction_requests
--       ADD CONSTRAINT chk_field_name
--       CHECK (field_name IN ('ear_tag', 'birth_date', 'gender', 'breed'));
--   COMMENT ON COLUMN public.animal_field_correction_requests.field_name IS
--       '欄位名稱：ear_tag, birth_date, gender, breed';
