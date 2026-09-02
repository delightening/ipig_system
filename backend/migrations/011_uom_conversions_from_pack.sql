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
-- 1. 資料遷移
-- ---------------------------------------------------------------------------

WITH uom_map(code, name) AS (
    -- 與 backend/src/services/product/uom.rs 的 UOM_CANONICAL、
    -- frontend/src/lib/utils.ts 的 UOM_MAP 同一組值——三處必須一致，
    -- 任一處分岔就會出現「畫面同一個單位、DB 存兩個字串」的假重複。
    VALUES ('EA','個'), ('pcs','個'), ('PC','支'), ('PR','雙'),
           ('TB','錠'), ('CP','膠囊'), ('BT','瓶'), ('AMP','安瓿'), ('VIA','小瓶'),
           ('BX','盒'), ('BOX','箱'), ('CTN','箱'), ('PK','包'), ('CASE','件'),
           ('RL','卷'), ('SET','組'),
           ('G','g'), ('KG','kg'), ('MG','mg'), ('ML','mL'), ('L','L')
),
candidate AS (
    SELECT p.id                                  AS product_id,
           COALESCE(mp.name, btrim(p.pack_unit)) AS uom,
           COALESCE(mb.name, btrim(p.base_uom))  AS base_canonical,
           p.pack_qty::numeric(18,6)             AS factor_to_base
    FROM products p
    LEFT JOIN uom_map mp ON mp.code = btrim(p.pack_unit)
    LEFT JOIN uom_map mb ON mb.code = btrim(p.base_uom)
    WHERE p.is_active
      AND p.pack_unit IS NOT NULL
      AND btrim(p.pack_unit) <> ''
      AND p.pack_qty >= 2
)
INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
SELECT gen_random_uuid(), c.product_id, c.uom, c.factor_to_base
FROM candidate c
WHERE c.uom <> c.base_canonical
-- 冪等：已存在的列一律不動。它可能是人工建的、換算率未必等於 pack_qty，不覆寫。
ON CONFLICT (product_id, uom) DO NOTHING;

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
    ADD CONSTRAINT chk_puc_uom_not_base CHECK (uom <> base_uom);

-- 既有的 product_id 單欄外鍵（ON DELETE CASCADE）與上面的複合外鍵語意重疊，
-- 保留不動：多一條外鍵不影響正確性，移除它反而要處理相依的索引與既有 constraint 名稱。
