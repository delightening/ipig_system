-- IACUC No. 變更事件的「補登實際日期」。
--
-- 背景：時間軸的「IACUC No. 變更」事件存在 user_activity_logs（稽核表，有防改觸發器與雜湊鏈，
-- 且依 partition_date 分區），較早寫入的事件沒有存實際變更日期，畫面只能顯示登錄時間。
-- 稽核表不能更新，所以實際日期另存在這張表；時間軸讀取時取每個事件「最新一筆」補登。
--
-- 設計重點：
--   * 只新增、不覆蓋：更正已補登的日期＝再新增一筆，讀取取最新一筆；舊筆永遠保留。
--   * event_id 不建外鍵：user_activity_logs 是分割表，主鍵含 partition_date，無法被單欄外鍵參照。
--     事件是否存在、是否屬於該動物、類型是否為 IACUC_CHANGE 由服務層在寫入前驗證。
--   * 每次補登另由服務層在同一交易寫一筆稽核事件（IACUC_EVENT_DATE_BACKFILL）。
--
-- 新增資料表，不改任何既有資料表或資料。

CREATE TABLE animal_event_actual_dates (
    id          uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id   uuid        NOT NULL REFERENCES animals (id),
    event_id    uuid        NOT NULL,
    actual_date date        NOT NULL,
    reason      text        NOT NULL,
    created_by  uuid        NOT NULL REFERENCES users (id),
    created_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT animal_event_actual_dates_reason_not_blank
        CHECK (length(btrim(reason)) > 0 AND length(reason) <= 500)
);

CREATE INDEX animal_event_actual_dates_event_idx
    ON animal_event_actual_dates (event_id, created_at DESC);
CREATE INDEX animal_event_actual_dates_animal_idx
    ON animal_event_actual_dates (animal_id);

COMMENT ON TABLE animal_event_actual_dates IS
    'IACUC No. 變更事件的補登實際日期；只新增不覆蓋，讀取取同一 event_id 最新一筆';
COMMENT ON COLUMN animal_event_actual_dates.event_id IS
    'user_activity_logs.id（IACUC_CHANGE 事件）；分割表無法建外鍵，由服務層驗證';
COMMENT ON COLUMN animal_event_actual_dates.reason IS '補登原因（必填，去空白後不可空，最多 500 字）';

-- 防改：UPDATE／DELETE 一律拒絕（比照既有的 check_*_immutable 慣例）。
--
-- 刻意不加 TRUNCATE 擋板：IDXF 全庫匯入前的 `TRUNCATE ... CASCADE`
-- （services/data_import.rs::cleanup_partial_unique_tables）會沿
-- pens ← animals ← 本表 的外鍵鏈遞移波及本表，裸的擋板會讓整句 TRUNCATE 失敗、
-- 全庫匯入直接壞掉（013 的 animal_blood_test_items 就是這個原因才需要 escape hatch）。
-- 本表不是稽核鏈本身：真正的稽核軌跡在 user_activity_logs（已有擋板），
-- 每次補登都會在那裡留下一筆事件。
CREATE FUNCTION public.check_animal_event_actual_dates_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'animal_event_actual_dates 只可新增，不可 %。需更正日期請再補登一筆（讀取取最新一筆）。', TG_OP
        USING ERRCODE = 'P0001';
END;
$$;

COMMENT ON FUNCTION public.check_animal_event_actual_dates_immutable() IS
    '補登實際日期為只新增的紀錄：UPDATE／DELETE 一律拒絕；更正請再新增一筆。';

CREATE TRIGGER check_animal_event_actual_dates_immutable_trigger
    BEFORE UPDATE OR DELETE ON animal_event_actual_dates
    FOR EACH ROW
    EXECUTE FUNCTION public.check_animal_event_actual_dates_immutable();
