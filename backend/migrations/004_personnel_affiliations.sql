-- R106-1：人員隸屬身分模型 —— affiliation_types（類型主檔）＋ personnel_affiliations（每人每段身分）
--
-- 背景：特休／請假／加班／計畫書人員四個功能各自靠副作用判定「這個人算不算數」，
-- 系統裡沒有任何一個地方明講「這個人是我們的正職／兼職／實習／外聘」。
-- 本 migration 只建表與六種類型的種子資料，不動任何既有表、不搬既有資料
-- （既有資料轉換是 R106-8，另案且需使用者逐一裁定 35 位無到職日者的分類）。
--
-- 類型不寫死在程式碼／enum 裡（2026-08-23 使用者裁定）：affiliation_types 是主檔，
-- 由 HR 在 UI 增修（R106-14），故用 FK 而非 PG enum —— enum 加值要 migration，
-- 做不到「HR 自己在 UI 上新增一種類型」。
--
-- ⚠️ default_* 五個旗標只是「新增身分列時的預設勾選」，不是判定依據；
-- 真正決定資格的一律是 personnel_affiliations 自己的五個旗標（可個別覆寫類型預設），
-- 這樣調整類型預設值時，既有人員的資格不會跟著跳動。
--
-- ⚠️ 不加「同一人期間不得重疊」的約束——一人可同時具多種身分（實測有
-- REVIEWER+VET、IACUC_CHAIR+REVIEWER 的兼任），資格採聯集（R106-2）。

CREATE TABLE public.affiliation_types (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    is_internal boolean NOT NULL,
    default_eligible_annual_leave boolean DEFAULT false NOT NULL,
    default_eligible_leave boolean DEFAULT false NOT NULL,
    default_eligible_overtime boolean DEFAULT false NOT NULL,
    default_assignable_to_protocol boolean DEFAULT false NOT NULL,
    default_requires_leave_proxy boolean DEFAULT false NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT affiliation_types_pkey PRIMARY KEY (id),
    -- code 建立後不可改（應用層強制，UI 需明講；DB 層只保證唯一）
    CONSTRAINT affiliation_types_code_key UNIQUE (code)
);

COMMENT ON TABLE public.affiliation_types IS
    'R106-1：人員身分類型主檔，HR 可在 /hr/affiliations 增修（R106-14）。code 建立後不可改。';
COMMENT ON COLUMN public.affiliation_types.default_eligible_annual_leave IS
    '新增身分列時的預設勾選，不是判定依據——判定一律看 personnel_affiliations 自己的旗標。';

CREATE TABLE public.personnel_affiliations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    affiliation_type_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    eligible_annual_leave boolean DEFAULT false NOT NULL,
    eligible_leave boolean DEFAULT false NOT NULL,
    eligible_overtime boolean DEFAULT false NOT NULL,
    assignable_to_protocol boolean DEFAULT false NOT NULL,
    -- 只在 eligible_leave 為真時有意義；判定送審是否需代理人只看這裡（見
    -- backend/src/handlers/hr/leave.rs 既有的 proxy_user_id / PENDING_PROXY 機制）
    requires_leave_proxy boolean DEFAULT false NOT NULL,
    department_id uuid,
    note text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT personnel_affiliations_pkey PRIMARY KEY (id),
    CONSTRAINT personnel_affiliations_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES public.users(id),
    CONSTRAINT personnel_affiliations_affiliation_type_id_fkey
        FOREIGN KEY (affiliation_type_id) REFERENCES public.affiliation_types(id),
    CONSTRAINT personnel_affiliations_department_id_fkey
        FOREIGN KEY (department_id) REFERENCES public.departments(id),
    -- 同一人同一類型同一起始日只能有一列（防重複建立，不是防兼任）
    CONSTRAINT personnel_affiliations_user_type_from_key
        UNIQUE (user_id, affiliation_type_id, effective_from),
    CONSTRAINT personnel_affiliations_effective_range_check
        CHECK (effective_to IS NULL OR effective_to >= effective_from)
);

COMMENT ON TABLE public.personnel_affiliations IS
    'R106-1：每人每段身分。effective_to IS NULL = 現行。一人可同時多列（兼任），資格解析一律取聯集（R106-2），故刻意不加期間重疊約束。';
COMMENT ON COLUMN public.personnel_affiliations.requires_leave_proxy IS
    '只在 eligible_leave 為真時有意義。判定是否需代理人只看這裡，不看類型的 default_*。';

-- 供 R106-2 的資格解析函式／view 使用：依人查、依類型查、依生效區間查。
CREATE INDEX personnel_affiliations_user_id_idx
    ON public.personnel_affiliations USING btree (user_id);
CREATE INDEX personnel_affiliations_affiliation_type_id_idx
    ON public.personnel_affiliations USING btree (affiliation_type_id);
CREATE INDEX personnel_affiliations_effective_idx
    ON public.personnel_affiliations USING btree (effective_from, effective_to);

-- 六種類型的種子資料（2026-08-23 使用者填定的矩陣，見 TODO.md R106-1；
-- 完整矩陣、兼任聯集實例、離職/復職表達方式見 docs/table-preview-PersonnelAffiliations.html）。
INSERT INTO public.affiliation_types
    (code, name, is_internal,
     default_eligible_annual_leave, default_eligible_leave,
     default_eligible_overtime, default_assignable_to_protocol,
     default_requires_leave_proxy, sort_order)
VALUES
    ('FULL_TIME', '正職', true,
     true, true, true, true, true, 10),
    ('PART_TIME', '內部兼職・特約', true,
     false, false, false, true, false, 20),
    ('INTERN', '實習', true,
     false, true, false, false, false, 30),
    ('EXTERNAL_IACUC_REVIEWER', '外聘審查委員', false,
     false, false, false, false, false, 40),
    ('EXTERNAL_PI', '外部主持人', false,
     false, false, false, false, false, 50),
    ('CLIENT', '委託人', false,
     false, false, false, false, false, 60);
