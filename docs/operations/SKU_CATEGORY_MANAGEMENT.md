# 品類管理說明

## 1. 目前是否有「品類管理」規則？

**目前沒有中央的「品類管理」介面**，品類的來源是：

| 層級 | 說明 |
|------|------|
| **資料庫** | 表 `sku_categories`（品類）、`sku_subcategories`（子類）定義於 `backend/migrations/008_audit_erp.sql`，**沒有種子資料**（沒有任何 INSERT）。 |
| **後端 API** | `GET /api/sku/categories`、`GET /api/sku/categories/{code}/subcategories` 僅**讀取** DB，沒有「新增/編輯品類」的 API。 |
| **前端** | 產品列表、編輯、新增頁的「品類」選單是**寫死**在程式裡（例如 `frontend/src/pages/master/ProductsPage.tsx` 的 `CATEGORIES` 常數），不是從 API 讀取。 |

因此：

- **匯入產品時**：品類/子類留空會由後端預設為 **GEN / OTH**，不依賴 `sku_categories` 是否有資料；SKU 仍會正常產生（如 `GEN-OTH-001`）。
- **顯示品類名稱時**：若從 API 取「品類名稱」（例如產品詳情），會查 `sku_categories` / `sku_subcategories`；若表裡沒有對應 code，名稱會是空。前端篩選用的品類清單則是寫死的，與 DB 無關。

---

## 2. 若要有「品類管理」，應該在哪裡新增？

可以擇一或並用：

### 作法 A：用 migration 寫入種子資料（建議先做）

- **位置**：新增一筆 migration，例如 `backend/migrations/XXXX_sku_categories_seed.sql`。
- **內容**：對 `sku_categories`、`sku_subcategories` 做 INSERT，例如：

  - 品類：GEN（通用）、DRG（藥品）、MED（醫材）、CON（耗材）、CHM（化學品）、EQP（設備）等。
  - 各品類下的子類：如 CON 底下 GAU、GLV、OTH 等（可對齊 `ProductsPage.tsx` 裡的 `CATEGORIES`）。

- **優點**：不用改後端 API 與前端，就能讓「依 API 顯示品類名稱」有資料；匯入時若填了品類代碼，也會有對應名稱。

### 作法 B：後台「品類管理」頁面（中長期）

- **位置**：在「主資料」或「設定」下新增「品類管理」頁（例如 `frontend/src/pages/master/SkuCategoriesPage.tsx` 或放在設定裡）。
- **後端**：新增 `POST/PUT/DELETE` 品類、子類的 API，對 `sku_categories`、`sku_subcategories` 做 CRUD。
- **前端**：品類/子類選單改為從 `GET /api/sku/categories` 等 API 讀取，不再寫死，這樣管理員就能在畫面上新增、編輯品類。

---

## 3. 與產品匯入的關係

- 匯入時**不填**品類/子類：系統一律用 **GEN-OTH**，不依賴品類表是否有資料。
- 匯入時**有填**品類/子類：產品會存成該 code；若 DB 沒有對應的 `sku_categories` / `sku_subcategories`，僅「品類名稱」顯示會是空，SKU 與儲存仍正常。

若你希望「由系統判定品類」（例如依名稱/規格自動推斷），需要另在匯入流程或後端邏輯中實作推斷規則（可參考 `docs/operations/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md`）。
