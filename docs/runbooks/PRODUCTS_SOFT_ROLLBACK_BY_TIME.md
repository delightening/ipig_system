# 產品依時間軟刪除（無備份時還原到某時間點）

> **適用情境**：沒有 DB 備份可還原，但希望產品列表「看起來像」回到某個時間點之前（只保留該時間前建立的產品，之後建立的產品從列表隱藏）。  
> **效果**：僅將指定時間之後建立的產品設為停用（`is_active = false`），採購/庫存等關聯資料仍保留，僅產品在管理介面不再顯示。

---

## 1. 確認資料庫／連線時區

`products.created_at` 為 **TIMESTAMPTZ**，PostgreSQL 內部存 UTC；比對時會依**連線的 session 時區**解讀你寫的時間字面值。

在 psql 或 pgAdmin 執行：

```sql
SHOW timezone;
```

- 若為 `Asia/Taipei`（或 `UTC+8`）：可直接用**台灣時間**寫條件（例如 `'2026-03-05 19:48:00'`）。
- 若為 `UTC`：若你的截止時間是**台灣時間 19:48**，請用 UTC 寫成 `'2026-03-05 11:48:00+00'`（台灣 19:48 = UTC 11:48）。

---

## 2. 先查會影響幾筆（建議）

把底下的 `'2026-03-05 19:48:00'` 換成你要的截止時間（若 DB 是 UTC 且截止是台灣時間，改用上面 UTC 寫法）。

```sql
-- 檢視將被停用的產品
SELECT id, sku, name, created_at
FROM products
WHERE created_at > '2026-03-05 19:48:00'
ORDER BY created_at;
```

確認筆數與內容無誤後再執行更新。

---

## 3. 執行停用（軟刪除）

**若 session 時區為台灣（Asia/Taipei）：**

```sql
UPDATE products
SET is_active = false, updated_at = NOW()
WHERE created_at > '2026-03-05 19:48:00';
```

**若 DB/連線為 UTC，而 19:48 是台灣時間：**

```sql
UPDATE products
SET is_active = false, updated_at = NOW()
WHERE created_at > '2026-03-05 11:48:00+00';
```

執行後，這些產品在產品管理列表就不會再出現；若有採購/庫存紀錄仍會保留，只是產品被隱藏。

---

## 4. 強制刪除（硬刪除）19:48 以後的產品與關聯資料

**警告**：以下會**永久刪除**該時間之後建立的產品，以及其單據明細、庫存、庫存快照、庫存帳流水；`product_uom_conversions`、`storage_location_inventory` 會因 FK CASCADE 一併刪除。`treatment_drug_options.erp_product_id` 會先被清空再刪產品。**執行前請務必先做 DB 備份或確認可接受資料損失。**

時區約定同 §1、§2：若 session 為 **Asia/Taipei** 用 `'2026-03-05 19:48:00'`；若為 **UTC** 且 19:48 為台灣時間用 `'2026-03-05 11:48:00+00'`。底下以 `@CUTOFF` 表示你選用的時間 literal。

**建議先查會影響的產品與筆數：**

```sql
SELECT id, sku, name, created_at FROM products WHERE created_at > @CUTOFF ORDER BY created_at;
```

**依賴順序刪除（請依序執行，或將 @CUTOFF 替換成同一時間後一次執行）：**

```sql
-- 1. 單據明細（採購/入庫等 document_lines）
DELETE FROM document_lines
WHERE product_id IN (SELECT id FROM products WHERE created_at > '2026-03-05 19:48:00');

-- 2. 庫存流水
DELETE FROM stock_ledger
WHERE product_id IN (SELECT id FROM products WHERE created_at > '2026-03-05 19:48:00');

-- 3. 庫存快照
DELETE FROM inventory_snapshots
WHERE product_id IN (SELECT id FROM products WHERE created_at > '2026-03-05 19:48:00');

-- 4. 用藥選項關聯（可為 NULL，先清空再刪產品）
UPDATE treatment_drug_options
SET erp_product_id = NULL, updated_at = NOW()
WHERE erp_product_id IN (SELECT id FROM products WHERE created_at > '2026-03-05 19:48:00');

-- 5. 刪除產品（會 CASCADE 刪除 product_uom_conversions、storage_location_inventory）
DELETE FROM products
WHERE created_at > '2026-03-05 19:48:00';
```

若 DB 為 **UTC** 且 19:48 為台灣時間，將上列所有 `'2026-03-05 19:48:00'` 改為 `'2026-03-05 11:48:00+00'`。

---

## 5. 注意事項

- 本 runbook **不提供**「依時間還原整庫」功能；§3 為軟刪除、§4 為硬刪除。
- 若要精確對應你環境的時區，請先執行 `SHOW timezone;` 後，依結果選擇時間 literal，或將截止時間換算成該 session 時區。
