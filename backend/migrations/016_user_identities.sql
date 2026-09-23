-- R121：外部身分提供者（目前只有 Google）與系統帳號的綁定。
--
-- 一列 = 「某個外部帳號（provider + subject）可以登入某個系統帳號」。
-- `subject` 是提供者給的**永久**帳號 ID（Google 的 `sub` claim），不是 email——
-- email 在 Google 端可以改、也可能被回收再發給別人，拿它當鍵會讓綁定悄悄指向錯的人。
--
-- 綁定規則（使用者 2026-09-23 裁定，由 service 層 `services/auth/google_oidc.rs` 把關，
-- 本 migration 只建資料骨架）：
--   * 只開放既有帳號：本表**不會**因為外部登入而多出 users 列。
--   * 首次登入時，若提供者回報 email 已驗證、且與恰好一個在職帳號相同（不分大小寫），
--     就自動建立綁定；之後一律以 (provider, subject) 查找，不再看 email。
--   * 密碼登入保留，本表與 `users.password_hash` 互不影響。
--
-- 回退：本表是純新增、沒有任何既有表參照它，`DROP TABLE public.user_identities;`
-- 即完整還原（綁定資料隨之消失，使用者下次用 Google 登入會依 email 重新綁定）。

CREATE TABLE public.user_identities (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    -- 帳號被硬刪除時綁定跟著消失；軟刪除（deleted_at）與停用（is_active）不刪綁定，
    -- 由登入流程在每次登入時檢查帳號狀態擋下，帳號恢復後綁定照舊可用。
    user_id uuid NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    provider character varying(32) NOT NULL,
    subject character varying(255) NOT NULL,
    -- 建立綁定當下提供者回報的 email，只供稽核追溯「當初憑什麼綁上這個帳號」，
    -- 不參與之後的登入比對。
    email_at_link character varying(255) NOT NULL,
    linked_at timestamptz NOT NULL DEFAULT now(),
    last_login_at timestamptz,
    CONSTRAINT user_identities_provider_check CHECK (provider IN ('google')),
    CONSTRAINT user_identities_subject_not_blank CHECK (length(btrim(subject)) > 0),
    -- 同一個外部帳號只能對應一個系統帳號。
    CONSTRAINT user_identities_provider_subject_key UNIQUE (provider, subject),
    -- 同一個系統帳號在同一個提供者底下只綁一個外部帳號：
    -- 否則「email 相同就自動綁定」會讓任何後來改用同 email 的 Google 帳號疊加上去，
    -- 等於帳號可以被第二個 Google 帳號接管而沒有任何人察覺。
    CONSTRAINT user_identities_user_provider_key UNIQUE (user_id, provider)
);

COMMENT ON TABLE public.user_identities IS
    'R121：外部身分提供者（目前僅 google）與系統帳號的綁定。以 (provider, subject) 查找；'
    'subject 為提供者的永久帳號 ID（Google sub），不是 email。綁定規則見 '
    'services/auth/google_oidc.rs。';

COMMENT ON COLUMN public.user_identities.email_at_link IS
    '建立綁定當下提供者回報且已驗證的 email，僅供稽核追溯，不參與登入比對。';
