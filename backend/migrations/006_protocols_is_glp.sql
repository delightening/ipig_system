-- `is_glp` 從 JSONB 拉成 protocols 的獨立欄位（裁定 14）
--
-- 問題：「GLP 計畫不可更換 Study Director」這條規則，判定來源是
-- `protocols.working_content -> 'basic' ->> 'is_glp'`——而 `working_content`
-- 是**設計上就可編輯的工作中內容**。
--
-- ⚠️ 先講清楚既有防線，免得誤以為是從零開始：
-- `services/protocol/core.rs` 已經有 GLP 鎖，而且已經防了單一 request 的繞過
--（`is_glp` 取自 `req.working_content.or(before.working_content)`，也就是
-- 「更新後生效」的值，同一個 request 裡同時翻 is_glp 又改 SD 會被擋）。
--
-- 擋不住的是**跨 request**：
--   request 1：只改 is_glp = false，不帶 study_director_user_id
--              → `if let Some(sd_id)` 整段不執行，鎖根本沒被檢查
--   request 2：改 SD（此時 is_glp 已是 false，鎖不觸發）
--   request 3：把 is_glp 改回 true
-- 全程不需要特殊權限，也不留下「GLP 屬性被改過」的痕跡。
--
-- 規則寫得再嚴，判定來源可被任意編輯的話，整組規則都只是裝飾。
--
-- 本 migration 只做「搬家」——把判定來源搬到欄位。鎖定邏輯在應用層
--（core.rs），因為它要分辨狀態與補登中，那些條件在 DB 層表達不了。
--
-- 實測依據（正式庫）：`protocols` 全欄位清單無任何 GLP 相關欄位，
-- `working_content.basic.is_glp` 為 true 的一筆都沒有。因此回填是一次性且
-- 無爭議的（全部落在 false），趁還沒有第一筆 GLP 案時做，成本最低。
-- ⚠️ 刻意不寫筆數：營運計數會隨時間漂移，寫死只會讓日後的人拿過期數字對帳。
-- 實際回填幾筆以下方 RAISE NOTICE 的輸出為準。

ALTER TABLE public.protocols
    ADD COLUMN is_glp boolean NOT NULL DEFAULT false;

COMMENT ON COLUMN public.protocols.is_glp IS
    'GLP 計畫。**這是判定的權威來源**，不要改用 working_content->basic->>is_glp——'
    '後者是可編輯的工作中內容，拿它當規則判定來源等於沒有規則。'
    '核准且完成補登後鎖定，見 services/protocol/core.rs 的 is_glp_locked。';

-- 回填：以 working_content 現值為準。
--
-- `->>` 回傳 text，所以比對字串 'true' 而不是布林——JSONB 裡這個欄位歷史上
-- 可能被寫成布林 true 或字串 "true"，兩種在 `->>` 之下都會得到 'true'。
-- 其餘情況（false／null／欄位不存在／working_content 為 NULL）一律落在
-- 欄位預設的 false，語意正確：沒有明確標記為 GLP 的就不是 GLP。
DO $$
DECLARE
    filled integer;
BEGIN
    UPDATE public.protocols
    SET is_glp = true
    WHERE working_content -> 'basic' ->> 'is_glp' = 'true';

    GET DIAGNOSTICS filled = ROW_COUNT;

    RAISE NOTICE '006: 回填 is_glp = true 共 % 筆', filled;
END $$;

-- 供裁定 13（GLP 案 SD 不得停用帳號）查詢用：
--   「這個人是不是某個未結案 GLP 計畫的 SD」
-- 部分索引而非整欄：is_glp 只有兩個值，且 true 是極少數（實測目前為零），
-- 整欄 btree 對佔絕大多數的 false 沒有選擇性。
CREATE INDEX protocols_glp_sd_idx
    ON public.protocols USING btree (study_director_user_id)
    WHERE is_glp;
