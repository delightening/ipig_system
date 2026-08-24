-- 審查意見加上「意見類型」，讓「無意見」成為可選項而非自由文字
--
-- 問題：審查委員沒有意見時，現況只能在自由文字欄打「無意見」。實測全庫 270 筆
-- review_comments 中有 12 筆內容正好是「無意見」，全部出自同一位執行祕書、
-- 全部在 PRE_REVIEW 階段、全部零回覆。它在系統裡跟一則「請補充說明」長得一模一樣：
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

ALTER TABLE public.review_comments
    ADD COLUMN comment_type character varying(30) NOT NULL DEFAULT 'COMMENT';

ALTER TABLE public.review_comments
    ADD CONSTRAINT review_comments_comment_type_check
    CHECK (comment_type IN ('COMMENT', 'NO_OBJECTION'));

COMMENT ON COLUMN public.review_comments.comment_type IS
    '意見類型。COMMENT=一般意見（申請人需回覆）；NO_OBJECTION=無意見（不需回覆）。'
    '兩者都算「已發表意見」，核准閘門（status.rs）一視同仁。';

-- 回填既有的 12 筆「無意見」。
-- 精準比對完整內容，不用 LIKE：實測那 12 筆的 content 就是「無意見」三個字，
-- 而其他意見（例如「已補充」「未填寫,請補充」）語意完全不同，模糊比對會誤傷。
UPDATE public.review_comments
SET comment_type = 'NO_OBJECTION'
WHERE content = '無意見';

-- 供「待回覆意見」類查詢使用：實務上永遠會帶 comment_type 過濾。
CREATE INDEX review_comments_comment_type_idx
    ON public.review_comments USING btree (comment_type);
