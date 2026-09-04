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
    -- 授權自動失效時點。NULL = 不設期限（沿用原本行為：一直有效到被撤銷為止）。
    --
    -- 為什麼需要這一欄：代理授權的典型情境是「PI 出國兩週」，但撤銷是純手動的，
    -- 沒有任何機制提醒。少了期限，一筆為兩週開的授權會安靜地活到有人想起來為止——
    -- 而且它同時握有結案簽署、安樂死核准/暫緩、修正案寫入、須知簽署全部五項權限。
    -- 這是**會自己惡化**的風險（沒有人在看著它），與「範圍給太寬」那種 SD 知情下
    -- 做的決定不同，所以先補這一項。
    --
    -- ⚠️ 期限**不追溯**：判斷「這筆授權現在還能不能用來做新的事」時檢查它，
    -- 但**不**用它去否定過去已經做成的行為。與 `revoked_at` 同一原則——
    -- `closure::dual_signature_ready` 條件 6 刻意不看 `revoked_at`，也同樣不看本欄，
    -- 因為簽署是時點行為，事後過期不該讓已簽的簽章失真。
    --
    -- ⚠️ 不寫進下面的部分唯一索引：索引述詞不能用 `now()`（非 immutable）。
    -- 因此「一份計畫同時只有一位生效代理人」這條約束仍以 `revoked_at IS NULL` 為準，
    -- 已過期但未撤銷的列**仍然佔著那個位置**。`authorize_pi_delegate` 會在核准新代理人時
    -- 自動撤銷已過期的舊列（比照 SD 變更的自動撤銷），SD 不需要先手動清理。
    expires_at timestamptz,
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

-- `euthanasia_appeals`（暫緩申請）同樣需要代簽證據，但它漏掉的方式與上面那欄不同、
-- 也更隱蔽：暫緩申請**不建立簽章**（`pi_appeal` 從頭到尾沒有呼叫任何 `sign_record_*`），
-- 所以它借不到 `electronic_signatures.delegation_id` 那條證據鏈。
--
-- 而本 migration 讓 `lock_order_for_pi` 開始接受代理人之後，`euthanasia_appeals.pi_user_id`
-- 的語意實質上變了：在此之前該欄必然等於計畫的 PI（授權檢查寫死 `pi_user_id = $2`），
-- 之後它可能是代理人。**欄位名沒變、值的意義卻變了**——這種變化最容易在事後稽核時
-- 被誤讀成「PI 本人親自申請了暫緩」。
--
-- 本欄補的就是這一層：非 NULL = `pi_user_id` 是依此筆授權代為申請的代理人。
-- 沿用與 `electronic_signatures.delegation_id` 相同的形狀（保留原本的行為人欄位、
-- 另外加一欄授權引用），而不是把 `pi_user_id` 改名——改名會波及既有消費端，
-- 也與上面那欄的慣例不一致。
--
-- FK 寫法與上面同一句型（ADD COLUMN 順帶帶 FK），適用同一份實測依據，不再重述。
ALTER TABLE public.euthanasia_appeals
    ADD COLUMN delegation_id uuid REFERENCES public.protocol_pi_delegates(id);

COMMENT ON COLUMN public.euthanasia_appeals.delegation_id IS
    '非 NULL 時代表 pi_user_id 是依此筆 protocol_pi_delegates 授權代為申請暫緩的代理人，'
    '而非計畫的 PI 本人。pi_user_id 一律是實際送出申請的人；本欄只補「依何授權代為申請」'
    '這一層可歸責性。既有紀錄一律 NULL（=本人申請），不需回填。';

-- `amendments`（變更申請）同理。`access::can_write_amendment` 的第三個分支放行生效中
-- 代理人，所以 `created_by` / `submitted_by` 這兩欄與上面那些欄位踩到同一個坑：
-- 本 migration 之前它們必然是計畫 PI（授權判準只認 admin 與 PI），之後可能是代理人。
--
-- 兩個行為人欄位各配一欄授權引用，而不是只加一欄：建立與送審是兩個獨立時點的動作，
-- 可能由不同的人做（PI 起草、代理人送審，或反過來），共用一欄會把兩件事混為一談。
-- `classified_by` 不配：分類是 IACUC 執秘的動作，不在代理範圍內。
--
-- 變更申請的簽章（`approved_signature_id` / `rejected_signature_id`）是**審查方的決定簽**，
-- 不是提交方的簽章——所以這條路徑跟暫緩一樣借不到 `electronic_signatures.delegation_id`，
-- 必須自己留證據。
ALTER TABLE public.amendments
    ADD COLUMN created_delegation_id uuid REFERENCES public.protocol_pi_delegates(id),
    ADD COLUMN submitted_delegation_id uuid REFERENCES public.protocol_pi_delegates(id);

COMMENT ON COLUMN public.amendments.created_delegation_id IS
    '非 NULL 時代表 created_by 是依此筆 protocol_pi_delegates 授權代為建立變更申請的代理人。';

COMMENT ON COLUMN public.amendments.submitted_delegation_id IS
    '非 NULL 時代表 submitted_by 是依此筆 protocol_pi_delegates 授權代為送審變更申請的代理人。';
