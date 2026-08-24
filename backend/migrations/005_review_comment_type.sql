-- 審查意見加上「意見類型」，讓「無意見」成為可選項而非自由文字
--
-- 問題：審查委員沒有意見時，現況只能在自由文字欄打「無意見」。它在系統裡跟一則
-- 「請補充說明」長得一模一樣：
--   * 申請人看到會以為要回覆
--   * 任何「還有幾則意見待處理」的統計都會把它算進去
--   * 它甚至讓計畫刪不掉（review_comments 的 FK 是 NO ACTION）
--
-- ⚠️ 但「無意見」不能單純不寫——`services/protocol/status.rs` 的核准閘門要求
-- **每個被指派的正式審查委員都必須發表過至少一則 top-level 意見**才能核准
-- （`AND parent_comment_id IS NULL`）。所以「無意見」是審查委員在沒有意見時
-- 滿足該閘門的唯一方法，必須保留為一筆真實的列，只是要標記出它的性質。
--
-- 因此本 migration 只加一個型別欄位，不動既有的閘門邏輯：
--   COMMENT       一般意見，申請人需要回覆（預設值，行為與現況完全相同）
--   NO_OBJECTION  無意見／無異議，申請人不需要回覆
--
-- 用 text + CHECK 而非 PG enum：這組值未來可能增加（例如「已了解」「僅供參考」），
-- enum 加值要 migration 且無法回退，CHECK 改起來單純得多。本專案 `protocols.status`
-- 那種需要嚴格型別的才用 enum，這裡是輕量標記。
--
-- 實測依據（正式庫）：
--   2026-08-24  270 筆 review_comments，其中 12 筆內容正好是「無意見」
--   2026-08-25  269 筆，其中 11 筆——少的那一筆隨計畫 PIG-110005 一起被刪除，
--               它正是當初擋住該計畫刪不掉的那筆「無意見」
-- ⚠️ 這兩個數字會隨營運資料變動，不要拿來當驗收條件。不變的是它們的性質，
-- 這才是本 migration 的立論基礎：全部出自同一位執行祕書、全部在 PRE_REVIEW 階段、
-- 全部是 top-level、全部零回覆——沒有任何一筆被當成「需要申請人回覆」在使用。
-- 實際回填幾筆以下方 RAISE NOTICE 的輸出為準。

ALTER TABLE public.review_comments
    ADD COLUMN comment_type character varying(30) NOT NULL DEFAULT 'COMMENT';

ALTER TABLE public.review_comments
    ADD CONSTRAINT review_comments_comment_type_check
    CHECK (comment_type IN ('COMMENT', 'NO_OBJECTION'));

COMMENT ON COLUMN public.review_comments.comment_type IS
    '意見類型。COMMENT=一般意見（申請人需回覆）；NO_OBJECTION=無意見（不需回覆）。'
    '兩者都算「已發表意見」，核准閘門（status.rs）一視同仁。';

-- 回填既有的「無意見」。
--
-- 精準比對完整內容，不用 LIKE 也不用 TRIM。2026-08-25 在正式庫上量過三種放寬方式，
-- 一筆都不會多抓：
--   btrim(content) = '無意見' 但 content <> '無意見'（前後有空白）        → 0 筆
--   去掉所有空白（含全形　）後 = '無意見' 但 btrim 後不是（中間有空白）    → 0 筆
--   content LIKE '%無意見%' 但去空白後不等於（夾在長句裡）                 → 0 筆
-- 放寬比對換不到任何一筆，卻讓「已補充」「未填寫,請補充」這類語意完全不同的意見
-- 暴露在誤傷風險下。這裡嚴格比較好。
DO $$
DECLARE
    filled integer;
BEGIN
    UPDATE public.review_comments
    SET comment_type = 'NO_OBJECTION'
    WHERE content = '無意見';

    GET DIAGNOSTICS filled = ROW_COUNT;

    -- 留稽核軌跡：部署當下實際回填幾筆，不必回頭比對註解裡的歷史數字。
    RAISE NOTICE '005: 回填 comment_type = NO_OBJECTION 共 % 筆', filled;
END $$;

-- 供「哪些計畫收到過無意見」這類查詢使用。
--
-- 刻意做成部分索引而非整欄索引：comment_type 只有兩個值，NO_OBJECTION 是極少數
-- （2026-08-25 實測 11/269，約 4%）。整欄 btree 對佔 96% 的 COMMENT 沒有選擇性，
-- planner 查那一側本來就會走 seq scan，索引只是白付寫入成本。
CREATE INDEX review_comments_no_objection_idx
    ON public.review_comments USING btree (protocol_id)
    WHERE comment_type = 'NO_OBJECTION';
