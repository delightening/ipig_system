-- `protocol_pi_delegates` 把「外部 PI（無系統帳號）尚未開通帳號前，誰能代替
-- 他簽署/核准」從**無聲借位**（`pi_user_id` 借用建立者/匯入者 id，簽署時
-- `signer_id == pi_user_id` 天生成立，稽核鏈上看不出是代簽）變成**顯式、
-- 需計劃負責人（SD）核准、留有核可證據**的正式授權。
--
-- ⚠️ 這不是要取代 `pi_is_external`（migration 009）或改動 `pi_user_id` 語意——
-- 曾評估過改用固定 sentinel 帳號取代借位，但那會直接打斷結案雙簽（PI 那一簽
-- 明文禁止 admin 繞過）與安樂死核准（24 小時時限、無任何 fallback）等一系列
-- 硬性要求「登入者 == pi_user_id」的合規檢查。這裡改採疊加式設計：`pi_user_id`
-- 與借位機制完全不動，只是在「必須是 PI 本人」的檢查上，額外開一條「或是
-- SD 核准的生效中代理人」的路，且這條路必須留下可歸責的證據。
--
-- 一份計畫同時最多一筆生效中（`revoked_at IS NULL`）代理授權；換人須先撤銷
-- 再重新核准，不做隱性覆蓋，保留完整歷史。核准/撤銷的授權規則（誰能核准、
-- SD 想指定自己為代理人時如何避免自簽自證）由 service 層
-- `services/protocol/pi_delegate.rs` 把關，本 migration 只建資料骨架。

CREATE TABLE public.protocol_pi_delegates (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    protocol_id uuid NOT NULL REFERENCES public.protocols(id) ON DELETE CASCADE,
    delegate_user_id uuid NOT NULL REFERENCES public.users(id),
    authorized_by uuid NOT NULL REFERENCES public.users(id),
    authorized_at timestamptz NOT NULL DEFAULT now(),
    reason text,
    revoked_by uuid REFERENCES public.users(id),
    revoked_at timestamptz,
    revoked_reason text,
    CONSTRAINT protocol_pi_delegates_revoke_pair_check
        CHECK (revoked_at IS NULL OR revoked_by IS NOT NULL)
);

-- 一份計畫同時只能有一個生效中代理人（部分唯一索引；`revoked_at IS NULL` = 現行，
-- 沿用本專案既有的 `idx_application_notices_active` 手法）。
CREATE UNIQUE INDEX protocol_pi_delegates_active_idx
    ON public.protocol_pi_delegates (protocol_id) WHERE revoked_at IS NULL;

CREATE INDEX protocol_pi_delegates_delegate_idx
    ON public.protocol_pi_delegates (delegate_user_id) WHERE revoked_at IS NULL;

COMMENT ON TABLE public.protocol_pi_delegates IS
    '外部 PI（pi_is_external=true）尚未開通系統帳號前，由計劃負責人（SD）核准的代簽授權。'
    '一份計畫同時僅一筆生效中（revoked_at IS NULL）。核准/撤銷的授權規則見'
    'services/protocol/pi_delegate.rs（SD 核准他人；SD 指定自己需改由 IACUC_STAFF/admin 核准，'
    '避免自簽自證）。';

-- `electronic_signatures` 加一個 nullable 欄位，把「這張簽章是依哪筆代理授權
-- 簽的」直接綁進簽章紀錄本身，而不是事後用 signer_id 反推（反推不出來——
-- 代理人簽署時 signer_id 就是代理人自己，密碼驗證與 HMAC signature_data 都
-- 綁死對 signer_id 本人，不可能也不應該把 PI 的 id 塞進 signer_id 蒙混）。
--
-- ⚠️ 這裡刻意**不**拆成「先加欄位 → 再 ADD CONSTRAINT ... NOT VALID → 之後 VALIDATE」
-- 的線上安全三段式。Squawk 的 `adding-foreign-key-constraint` 會對這一行示警
-- （CodeRabbit 於 2026-09-02 據此提為 Major），但那條規則是通用規則，套不到
-- 「同一句 ADD COLUMN 順帶帶 FK」這個形狀：新欄位在既有列上一律是 NULL，
-- 而 NULL 天生滿足 FK，PostgreSQL 因此不需要回頭掃描。
--
-- 2026-09-03 於 PostgreSQL 16、200 萬列的表上實測三種寫法（丟棄用測試庫）：
--   A 本寫法 `ADD COLUMN ... REFERENCES`                     18.0 ms
--   B 先 ADD COLUMN、再 ADD CONSTRAINT（這個才真的掃描）      94.7 ms
--   C ADD CONSTRAINT NOT VALID + VALIDATE（linter 建議寫法）  0.5 + 75.9 ms
-- A 不但沒有掃描，還比 C 快約四倍；且 C 也躲不掉 ADD COLUMN 本身的 ACCESS
-- EXCLUSIVE。再者本 migration 的被參照表 `protocol_pi_delegates` 是上面幾行
-- 才建的空表，那把 SHARE ROW EXCLUSIVE 沒有任何意義。
--
-- 結論：三段式在這裡是為不存在的問題付出複雜度。**但這只適用於「新增欄位順帶
-- 建 FK」**；若日後是在既有欄位上補 FK（情境 B），該規則完全成立，請照做。
ALTER TABLE public.electronic_signatures
    ADD COLUMN delegation_id uuid REFERENCES public.protocol_pi_delegates(id);

COMMENT ON COLUMN public.electronic_signatures.delegation_id IS
    '非 NULL 時代表 signer_id 是依此筆 protocol_pi_delegates 授權代簽，而非本人簽署。'
    'signer_id 仍是實際輸入密碼、完成簽署動作的人；delegation_id 只補「代表誰、'
    '依何授權」這一層語意，不影響簽章本身的密碼學完整性。既有簽章一律 NULL'
    '（=本人簽署），不需回填。';
