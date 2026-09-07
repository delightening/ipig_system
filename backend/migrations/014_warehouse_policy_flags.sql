-- 014: 倉庫的盤點與警報政策旗標
--
-- 背景（2026-09-07 使用者裁定）
-- ============================================================================
-- 現場的七個地點性質不同，但 `warehouses` 表只有 code/name/address/is_active，
-- 沒有任何欄位能表達「這個倉庫要不要盤、要不要進警報」。規則因此只存在人腦裡，
-- 程式若要判斷就只能比對倉庫名稱——很脆弱。
--
-- 裁定後的規則：
--
--   地點            例行盤點          低庫存警報
--   ------------    --------------    --------------------------------
--   大倉庫          每月              正常
--   準備室          每月              正常
--   儲藏室          不排（異狀才盤）  排除，改為「領不出來時通知」
--   飼料櫃          每月              正常
--   手術室 A / B    每月              正常
--   廢棄物處理區    排除              排除
--
-- 為什麼是兩個布林而不是一個 warehouse_type 列舉
-- ============================================================================
-- 兩個旗標目前的值剛好一樣（儲藏室與廢棄區都是 true/true），看起來像可以合併，
-- 但它們表達的是**兩件不同的事**：
--
--   儲藏室  —— 它**是**庫存，只是不維護帳面準確度（每天領用，盤了也馬上變）
--   廢棄區  —— 它**根本不是**庫存資產，數量不代表可用存量
--
-- 合成一個旗標，日後出現「要盤但不要警報」的倉庫就回不去了。
-- 用列舉（storage/frontline/point_of_use/disposal）語意更清楚，但當日討論中
-- 「準備室算前線還是使用點」本身還沒定案——分類還沒穩定就定死列舉值，
-- 之後改動比加一個布林貴得多。等分類穩定再歸納。
--
-- 預設值刻意都是 false
-- ============================================================================
-- 也就是「照常盤點、照常警報」——本 migration 跑完**不改變任何現有行為**。
-- 哪些倉庫要設 true 由使用者在倉庫管理頁勾選，不寫死在這裡：各環境
-- （dev/staging/prod）的倉庫資料不同，用名稱或 code 去 UPDATE 會在別的環境
-- 命中錯的列，或一列都沒命中而靜默無事發生。
--
-- ⚠️ 所以部署完這支之後還有一步：去倉庫管理頁把儲藏室與廢棄物處理區的兩個
-- 旗標打開。沒做這步，這支 migration 等於沒生效。

ALTER TABLE warehouses
    ADD COLUMN IF NOT EXISTS exclude_from_alerts boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS skip_routine_stocktake boolean NOT NULL DEFAULT false;

COMMENT ON COLUMN warehouses.exclude_from_alerts IS
    '排除於低庫存警報之外。用於帳面準確度不受維護的倉庫（如每天領用的儲藏室），或本來就不是庫存資產的地點（如廢棄物處理區）。對不準的數字發警報只會製造疲勞，把真正該理的警報埋掉。';

COMMENT ON COLUMN warehouses.skip_routine_stocktake IS
    '不納入例行盤點。命名用 routine 而非 scheduled：本系統沒有盤點排程器，月盤是人工開單，「例行」涵蓋人工與未來可能的自動排程兩種。設為 true 不代表永不盤——儲藏室仍在缺貨或異狀時盤，只是不排固定週期。';

-- 低庫存警報排除掛旗標的倉庫
-- ============================================================================
-- 欄位結構與 002_schema.sql 的定義完全一致，只在 WHERE 多一個條件，
-- 所以 CREATE OR REPLACE 可用（REPLACE 不允許改變欄位清單或型別）。
--
-- ⚠️ 這裡不動 `safety_stock` 的單位問題：本 view 直接比較
-- `on_hand_qty_base < p.safety_stock` 而不看 `safety_stock_uom`，那是既有缺陷，
-- 與本次的政策旗標無關，混在一起改會讓兩件事的驗證糾纏在一起。另案處理。
CREATE OR REPLACE VIEW public.v_low_stock_alerts AS
 SELECT inv.warehouse_id,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku AS product_sku,
    p.name AS product_name,
    p.base_uom,
    inv.on_hand_qty_base AS qty_on_hand,
    p.safety_stock,
    p.reorder_point,
        CASE
            WHEN (inv.on_hand_qty_base <= (0)::numeric) THEN 'out_of_stock'::text
            WHEN ((p.safety_stock IS NOT NULL) AND (inv.on_hand_qty_base < p.safety_stock)) THEN 'below_safety'::text
            WHEN ((p.reorder_point IS NOT NULL) AND (inv.on_hand_qty_base < p.reorder_point)) THEN 'below_reorder'::text
            ELSE 'below_safety'::text
        END AS stock_status
   FROM ((public.inventory_snapshots inv
     JOIN public.products p ON ((inv.product_id = p.id)))
     JOIN public.warehouses w ON ((inv.warehouse_id = w.id)))
  WHERE (p.is_active AND w.is_active AND (NOT w.exclude_from_alerts) AND ((inv.on_hand_qty_base <= (0)::numeric) OR ((p.safety_stock IS NOT NULL) AND (inv.on_hand_qty_base < p.safety_stock)) OR ((p.reorder_point IS NOT NULL) AND (inv.on_hand_qty_base < p.reorder_point))));
