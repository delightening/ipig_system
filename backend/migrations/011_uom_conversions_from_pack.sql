-- 單位換算：把 products 的包裝關係遷入 product_uom_conversions，並補上 DB 層約束。
--
-- 背景：換算表在此之前是空的，所以既有的單位換算功能一律退回「factor 恆為 1」，
-- 等於沒有生效。品項的包裝關係一直只存在 products.pack_unit / pack_qty，
-- 那兩欄與換算表描述的是同一件事，卻互不校驗。
--
-- 遷移條件（四個同時成立）：
--   is_active                    停用品項不進盤點底稿，範圍越小越好驗
--   pack_unit 非 NULL 且非空白
--   pack_qty >= 2                factor = 1 沒有換算意義，只會讓單位下拉多出等值選項，
--                                並讓依字串相等比對單位的收貨對帳 SQL 依填法產生不同結果
--   正規化後 <> base_uom         🔴 必須「正規化之後」才比
--
-- 最後一條是本檔最容易寫錯的地方：base_uom 是「盒」而 pack_unit 填成 BX（正規化後也是
-- 「盒」）時，原字串不同、實質同名。只比原字串會整批漏掉這一類，而它們正是危險的那一類
-- ——同名列會讓盤點端把貨架現存量除成極小的數字、核准時再以原值去減，開出把整個貨架
-- 清空的盤虧調整單；收貨端則會讓已收量膨脹，把完全正確的收貨判成超收而擋下。
-- 實查確認這一類的 pack_qty 全部 >= 2，靠 pack_qty 門檻濾不掉。

-- ---------------------------------------------------------------------------
-- 正規化函式（本檔唯一的映射來源）
-- ---------------------------------------------------------------------------
-- 為什麼做成函式而不是每段各寫一次 VALUES：本檔原本有六份重複的對照清單，而最後那條
-- CHECK 約束根本沒用到映射、只比原字串——結果是「約束比它宣稱的弱」：
-- `base_uom = 'BX'` 且換算列本來就是 `'BX'` 時，正規化把換算列改成「盒」而
-- 複製過去的 base_uom 仍是 'BX'，`'盒' <> 'BX'` 就這樣通過了，
-- 但它語意上正是要禁止的 base→base 同名列。函式化之後映射只有一份，約束也用得上它。
--
-- IMMUTABLE 是 CHECK 約束的硬性要求（PostgreSQL 只允許約束呼叫 immutable 函式）。
-- 本函式純查表、無 I/O、對同一輸入永遠同一輸出，符合。
--
-- ⚠️ 必須與 `backend/src/services/product/uom.rs` 的 `UOM_CANONICAL`、
-- `frontend/src/lib/utils.ts` 的 `UOM_MAP` 保持同一組值。三處分岔會讓
-- 「畫面同一個單位、DB 存兩個字串」的假重複重新出現。

CREATE OR REPLACE FUNCTION uom_canonical(raw character varying)
RETURNS character varying
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT COALESCE(
        (SELECT m.name
           FROM (VALUES
                    ('EA','個'), ('pcs','個'), ('PC','支'), ('PR','雙'),
                    ('TB','錠'), ('CP','膠囊'), ('BT','瓶'), ('AMP','安瓿'), ('VIA','小瓶'),
                    ('BX','盒'), ('BOX','箱'), ('CTN','箱'), ('PK','包'), ('CASE','件'),
                    ('RL','卷'), ('SET','組'),
                    ('G','g'), ('KG','kg'), ('MG','mg'), ('ML','mL'), ('L','L')
                ) AS m(code, name)
          WHERE m.code = btrim(raw)),
        btrim(raw)
    )::character varying;
$$;

COMMENT ON FUNCTION uom_canonical(character varying) IS
'單位字串的正規形式（英文代碼 → 中文顯示名）。與 services/product/uom.rs 的 UOM_CANONICAL
及 frontend/src/lib/utils.ts 的 UOM_MAP 同一組值；查不到的原樣保留（例：桶、捲）。
被 product_uom_conversions 的 chk_puc_uom_not_base 約束使用，故必須維持 IMMUTABLE。';

-- ---------------------------------------------------------------------------
-- 0. 別名正規化（必須在遷移之前）
-- ---------------------------------------------------------------------------
-- 🔴 沒有這一段，整支 migration 等於白做。
--
-- 盤點底稿取換算率的那一句是
--   `LEFT JOIN product_uom_conversions c ON c.product_id = p.id AND c.uom = p.pack_unit`
-- ——它拿 **`products.pack_unit` 的原值**去比。下面第 1 節寫進換算表的是**正規形式**
-- （`BX` → `盒`），若不同時把 `pack_unit` 也收斂，兩邊永遠比不中：
--
--   products.pack_unit = 'BX'      product_uom_conversions.uom = '盒'   → join 不成立
--
-- 症狀與「這個品項根本沒建換算列」一模一樣：`pack_factor` 為 NULL、底稿退回 base_uom、
-- 沒有任何錯誤訊息。換算表裡多出一堆沒人用得到的資料，而「盤點論盒」仍然不會生效。

-- 0-pre. 記下所有將被改動的原值，讓本 migration 可還原。
--
-- 為什麼需要：下面的去重、改名、以及 `products.pack_unit` 的改寫都是**破壞性**的——
-- `BX=50` 被改成 `盒=50` 之後，光看結果無從得知它原本叫什麼。沒有這三張表，
-- 回退腳本只能把改名後的列當成「本 migration 新增的」刪掉，
-- 等於把使用者原有的資料一併抹掉（丟棄庫實測已重現：4 列回退後只剩 1 列）。
--
-- 這三張表在正式環境預期是**空的**（實查換算表目前 0 列）。
-- 確認本 migration 不再需要回退後可自行 DROP。

CREATE TABLE IF NOT EXISTS mig011_conversion_backup (
    id             uuid PRIMARY KEY,
    product_id     uuid NOT NULL,
    uom            character varying(20) NOT NULL,
    factor_to_base numeric(18,6) NOT NULL
);

CREATE TABLE IF NOT EXISTS mig011_pack_unit_backup (
    product_id uuid PRIMARY KEY,
    pack_unit  character varying(20) NOT NULL
);

CREATE TABLE IF NOT EXISTS mig011_inserted_conversion (
    id uuid PRIMARY KEY
);

-- 備份所有非正規寫法的換算列（下面會被去重刪掉或改名）
INSERT INTO mig011_conversion_backup (id, product_id, uom, factor_to_base)
SELECT c.id, c.product_id, c.uom, c.factor_to_base
FROM product_uom_conversions c
WHERE c.uom <> uom_canonical(c.uom)
ON CONFLICT (id) DO NOTHING;

-- 備份所有非正規寫法的 pack_unit
INSERT INTO mig011_pack_unit_backup (product_id, pack_unit)
SELECT p.id, p.pack_unit
FROM products p
WHERE p.pack_unit IS NOT NULL
  AND p.pack_unit <> uom_canonical(p.pack_unit)
ON CONFLICT (product_id) DO NOTHING;

-- 0a. 同一品項、正規化後同一個單位、但換算率不一致 → 停止，交人裁定。
--
-- ⚠️ 這裡必須用 **(product_id, 正規形式) 分組**，不能拿「別名」去比對「已是正規值的那一列」。
-- 本檔第一版就是後者，漏掉了 alias↔alias 的情形：`BOX=10` 與 `CTN=20` 都正規化成「箱」，
-- 但兩列的 uom 都不等於「箱」，於是檢查放行、去重也不做，最後改名時兩列撞成同一個
-- (product_id, uom) 而以原始 UNIQUE 違規中止——不是設計中的 fail-closed 訊息。
--
-- 兩個值都可能是對的，migration 不該替人選一個（與應用層 `uom::resolve_alias` 一致）。
DO $$
DECLARE
    conflicts text;
BEGIN
    SELECT string_agg(t.msg, '；' ORDER BY t.msg) INTO conflicts
      FROM (
          SELECT format('%s：%s 有 %s 種換算率（%s）',
                        p.sku,
                        uom_canonical(c.uom),
                        count(DISTINCT c.factor_to_base),
                        string_agg(DISTINCT c.uom || '=' || c.factor_to_base::text, ' / ')) AS msg
            FROM product_uom_conversions c
            JOIN products p ON p.id = c.product_id
           GROUP BY p.sku, c.product_id, uom_canonical(c.uom)
          HAVING count(DISTINCT c.factor_to_base) > 1
      ) t;

    IF conflicts IS NOT NULL THEN
        RAISE EXCEPTION
            '單位換算存在無法自動合併的別名衝突，請先人工清理再套用本 migration：%',
            conflicts;
    END IF;
END $$;

-- 0a2. 換算列正規化後與該品項的 base_uom 同名 → 停止。
--
-- 這是最危險的一類（見檔頭），而它不會被 0a 抓到：0a 比的是「同一組正規單位內的換算率」，
-- 這裡比的是「換算列 vs 基本單位」。不先擋掉的話，最後那條 CHECK 會以原始約束違規中止，
-- 訊息看不出是哪個品項。
DO $$
DECLARE
    same_as_base text;
BEGIN
    SELECT string_agg(format('%s：%s（正規化後＝base_uom %s）',
                             p.sku, c.uom, uom_canonical(p.base_uom)), '；' ORDER BY p.sku)
      INTO same_as_base
      FROM product_uom_conversions c
      JOIN products p ON p.id = c.product_id
     WHERE uom_canonical(c.uom) = uom_canonical(p.base_uom);

    IF same_as_base IS NOT NULL THEN
        RAISE EXCEPTION
            '換算表存在與基本單位同名的列（正規化後），請先人工清理再套用本 migration：%',
            same_as_base;
    END IF;
END $$;

-- 0b. 同一組正規單位有多列且換算率一致 → 只留一列。
--     優先保留「已經是正規寫法」的那一列（不必改名、id 不變）；
--     全是別名時保留 id 最小的，讓結果與執行次序無關。
WITH ranked AS (
    SELECT c.id,
           row_number() OVER (
               PARTITION BY c.product_id, uom_canonical(c.uom)
               ORDER BY (c.uom = uom_canonical(c.uom)) DESC, c.id
           ) AS rn
      FROM product_uom_conversions c
)
DELETE FROM product_uom_conversions c
USING ranked r
WHERE c.id = r.id AND r.rn > 1;

-- 0c. 其餘非正規列改名成正規形式（0a/0a2/0b 之後已無撞鍵可能）。
UPDATE product_uom_conversions c
   SET uom = uom_canonical(c.uom)
 WHERE c.uom <> uom_canonical(c.uom);

-- 0d. `products.pack_unit` 一併收斂——這是讓上面那句 join 接得上的另一半。
UPDATE products p
   SET pack_unit = uom_canonical(p.pack_unit)
 WHERE p.pack_unit IS NOT NULL
   AND p.pack_unit <> uom_canonical(p.pack_unit);

-- ⚠️ `products.base_uom` 刻意不動：它是庫存數量與所有既有單據明細的單位，
-- 改它等於重新解釋既有數字（`document_lines.uom` 全部等於各自品項的 base_uom）。
-- 那屬於另一件事，不在本 migration 範圍。下面的 CHECK 因此用 `uom_canonical()` 兩邊都套，
-- 而不是假設 base_uom 已經是正規形式。

-- ---------------------------------------------------------------------------
-- 1. 資料遷移
-- ---------------------------------------------------------------------------

WITH candidate AS (
    SELECT p.id                          AS product_id,
           uom_canonical(p.pack_unit)    AS uom,
           uom_canonical(p.base_uom)     AS base_canonical,
           p.pack_qty::numeric(18,6)     AS factor_to_base
    FROM products p
    WHERE p.is_active
      AND p.pack_unit IS NOT NULL
      AND btrim(p.pack_unit) <> ''
      AND p.pack_qty >= 2
),
ins AS (
    INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
    SELECT gen_random_uuid(), c.product_id, c.uom, c.factor_to_base
    FROM candidate c
    WHERE c.uom <> c.base_canonical
    -- 冪等：已存在的列一律不動。它可能是人工建的、換算率未必等於 pack_qty，不覆寫。
    ON CONFLICT (product_id, uom) DO NOTHING
    RETURNING id
)
-- 記下「本 migration 實際插入的是哪幾列」。回退時只刪這些 id，
-- 不靠條件反推——條件在 0d 改過 pack_unit 之後已經不等價了。
INSERT INTO mig011_inserted_conversion (id)
SELECT id FROM ins
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------------------
-- 2. 約束：factor_to_base 必須為正
-- ---------------------------------------------------------------------------
-- 非正數乘上去會讓入庫變出庫（負數）或整行歸零而靜默漏帳。應用層讀取端目前是「略過」
-- 這種列（等於視為未定義單位），這裡直接讓它寫不進來。

ALTER TABLE product_uom_conversions
    ADD CONSTRAINT chk_puc_factor_positive CHECK (factor_to_base > 0);

-- ---------------------------------------------------------------------------
-- 3. 約束：uom 不得等於該品項的 base_uom（宣告式，非 trigger）
-- ---------------------------------------------------------------------------
-- 跨表條件寫不成單表 CHECK，故把 base_uom 去正規化進本表，再用複合外鍵綁回 products。
--
-- 為什麼不用 trigger：trigger 要在兩張表各掛一支才等效（換算表的寫入、以及 products
-- 改 base_uom），漏掉後者就會留下一條「當初合法、改完 base_uom 之後變同名」的壞列。
-- 宣告式版本由 ON UPDATE CASCADE 負責：base_uom 一改，本表的副本跟著改，
-- 若因此與既有 uom 撞名，CHECK 會讓那次 UPDATE 直接失敗，而不是靜默留下壞資料。
--
-- 🔴 CHECK 兩邊都套 `uom_canonical()`：base_uom 本身可能仍是非正規寫法（本檔刻意不動它），
-- 只比原字串的話，`uom='盒'` 配 `base_uom='BX'` 會通過——而那正是要禁止的同名列。

-- 下面這條 UNIQUE 是靜態掃描（Squawk）每一輪都會報的兩條警告，刻意保留內嵌寫法。
-- 它不會回應說明、下一輪還會再報一次，所以理由寫在這裡，不要再重審一次：
--
--   `constraint-missing-not-valid`：對 UNIQUE 不成立。實測 PostgreSQL 16 直接回
--   `ERROR: UNIQUE constraints cannot be marked NOT VALID`——NOT VALID 只支援
--   CHECK 與外鍵。這條警告本身就套錯約束型別。
--
--   `disallowed-unique-constraint`（改用 CREATE UNIQUE INDEX CONCURRENTLY）：
--   三個理由不採用。
--   (a) 順序上不可行。下面那條複合外鍵 REFERENCES products (id, base_uom)，
--       要求這個唯一鍵**先存在**，故它只能在本檔之內或更早，不能拆成後面的 migration。
--   (b) 拆了也擋不住寫入。整支 migration 跑在同一個交易裡（`sqlx::migrate!().run()`
--       預設如此），鎖一律持有到 COMMIT。實測開著交易查 pg_locks：即使不算這條 UNIQUE，
--       外鍵那句自己就在 products 上押 ShareRowExclusiveLock 到交易結束。拆走 UNIQUE
--       只是把「連讀也擋」降成「擋寫」，而那個差別的長度實測是 8 ms（合成表 2 萬列）。
--   (c) 代價是實的。CONCURRENTLY 不能在交易內跑（實測
--       `ERROR: CREATE INDEX CONCURRENTLY cannot run inside a transaction block`），
--       要拆就得給那支 migration 加 `-- no-transaction`；失敗時會留下 INVALID 索引
--       與半套用狀態，得人工善後。本檔的資料改寫需要的正是「壞了就整個 rollback」。
--
-- 前提上也不需要：migration 在 app 啟動時跑（main.rs 綁 HTTP 埠之前），
-- 部署是手動 build + up -d，這段期間本來就沒有應用流量，沒有 online DDL 的需求。

ALTER TABLE products
    ADD CONSTRAINT products_id_base_uom_key UNIQUE (id, base_uom);

ALTER TABLE product_uom_conversions
    ADD COLUMN base_uom character varying(20);

UPDATE product_uom_conversions c
   SET base_uom = p.base_uom
  FROM products p
 WHERE p.id = c.product_id;

ALTER TABLE product_uom_conversions
    ALTER COLUMN base_uom SET NOT NULL;

ALTER TABLE product_uom_conversions
    ADD CONSTRAINT product_uom_conversions_product_base_fkey
    FOREIGN KEY (product_id, base_uom) REFERENCES products (id, base_uom)
    ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE product_uom_conversions
    ADD CONSTRAINT chk_puc_uom_not_base
    CHECK (uom_canonical(uom) <> uom_canonical(base_uom));

-- 既有的 product_id 單欄外鍵（ON DELETE CASCADE）與上面的複合外鍵語意重疊，
-- 保留不動：多一條外鍵不影響正確性，移除它反而要處理相依的索引與既有 constraint 名稱。
