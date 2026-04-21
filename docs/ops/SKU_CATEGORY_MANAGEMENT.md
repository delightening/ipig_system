# 品類管理說明

## 0. 品類代碼／子類代碼／分類要在哪裡改？

| 要改的內容 | 目前編輯位置 | 說明 |
|------------|--------------|------|
| **品類／子類的「代碼與名稱」**（如 DRG 藥品、CON 耗材、GLV 手套） | **DB**：`sku_categories`、`sku_subcategories` 表。種子見 `backend/migrations/014_sku_categories_seed.sql`；日後可加「品類管理」頁做 CRUD。 | 前端**已改為從 API 讀取**（`useSkuCategories` hook），新增／編輯／列表／匯入／詳情頁皆同一來源。 |
| **CSV「分類」欄中文對應到代碼**（如 耗材→CON、藥品→DRG） | **後端**：`backend/src/services/product.rs` 的 `map_category_display_to_code` 函數 | 匯入 CSV 時，若欄位名為「分類」或「品類代碼」，會用此對應轉成三碼代碼。 |

---

## 1. 目前是否有「品類管理」規則？

**目前沒有中央的「品類管理」介面**，品類的來源是：

| 層級 | 說明 |
|------|------|
| **資料庫** | 表定義於 `008_audit_erp.sql`；**種子資料**見 `014_sku_categories_seed.sql`（GEN/DRG/MED/CON/CHM/EQP 及子類）。 |
| **後端 API** | `GET /api/sku/categories`、`GET /api/sku/categories/{code}/subcategories` 讀取 DB；尚無「新增/編輯品類」的 CRUD API。 |
| **前端** | 品類／子類**一律由 API 讀取**（`frontend/src/hooks/useSkuCategories.ts`），新增產品、編輯、列表篩選、匯入、詳情頁共用，單一來源。 |

因此：

- **匯入產品時**：品類/子類留空會由後端預設為 **GEN / OTH**，不依賴 `sku_categories` 是否有資料；SKU 仍會正常產生（如 `GEN-OTH-001`）。
- **顯示品類名稱時**：若從 API 取「品類名稱」（例如產品詳情），會查 `sku_categories` / `sku_subcategories`；若表裡沒有對應 code，名稱會是空。前端篩選用的品類清單則是寫死的，與 DB 無關。

---

## 2. 若要有「品類管理」，應該在哪裡新增？

可以擇一或並用：

### 作法 A：用 migration 寫入種子資料（**已完成**）

- **位置**：`backend/migrations/014_sku_categories_seed.sql`。
- **內容**：已寫入品類 GEN/DRG/MED/CON/CHM/EQP 及各子類（與前端原 CATEGORIES 對齊）；MED 以 MED-MED 一筆表示無子類。
- **部署**：執行 migration 後，GET /api/sku/categories 即有資料；前端已改為從 API 讀取，無需再改。

### 作法 B：後台「品類管理」頁面（中長期）

- **位置**：在「主資料」或「設定」下新增「品類管理」頁（例如 `frontend/src/pages/master/SkuCategoriesPage.tsx` 或放在設定裡）。
- **後端**：新增 `POST/PUT/DELETE` 品類、子類的 API，對 `sku_categories`、`sku_subcategories` 做 CRUD。
- **前端**：品類/子類選單改為從 `GET /api/sku/categories` 等 API 讀取，不再寫死，這樣管理員就能在畫面上新增、編輯品類。

---

## 3. 與產品匯入的關係

- 匯入時**不填**品類/子類：系統一律用 **GEN-OTH**，不依賴品類表是否有資料。
- 匯入時**有填**品類/子類：產品會存成該 code；若 DB 沒有對應的 `sku_categories` / `sku_subcategories`，僅「品類名稱」顯示會是空，SKU 與儲存仍正常。

若你希望「由系統判定品類」（例如依名稱/規格自動推斷），需要另在匯入流程或後端邏輯中實作推斷規則（可參考 `docs/ops/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md`）。

---

## 4. 業界常見做法（品類／子類怎麼管）

| 做法 | 說明 | 本專案對應 |
|------|------|------------|
| **主資料集中管理** | 品類／子類視為「主資料」，在後台有獨立「品類管理」或「主資料／分類設定」頁，由管理員新增／編輯／停用，代碼穩定、名稱可改。 | 尚未實作；目前為 DB 表 + 唯讀 API，無 CRUD 頁。 |
| **種子 + 少改** | 用 migration 或 seed 寫入一組標準品類／子類，之後很少改；要改就改 DB 或加 migration。 | 可做：新增 `XXXX_sku_categories_seed.sql` 對齊前端 `CATEGORIES`。 |
| **代碼與顯示分離** | 代碼（如 CON、DRG）固定、供 SKU 與整合用；顯示名稱（如「耗材」「藥品」）可多語或依客戶自訂。 | 表結構已支援（code + name）；名稱目前單一。 |
| **匯入時對應** | 外部檔案常是「分類名稱」或自家代碼；系統在匯入時對應到內部代碼（對照表或規則）。 | 已做：`map_category_display_to_code` 將「耗材」等對應到 CON 等。 |
| **與前端選單一致** | 下拉選單與匯入可用代碼應來自同一來源（API 或共用設定），避免前後端不一致。 | 目前新增產品用寫死常數、匯入用 API；建議長期改為都從 API。 |

**實務建議**：短期用 **migration 種子** 把 DB 品類／子類補齊（與 CreateProductPage 的 CATEGORIES 一致），讓匯入與顯示都有正確名稱；中長期再補 **品類管理頁 + CRUD API**，前端改為全從 API 讀取，即可在畫面上編輯品類代碼／子類代碼／顯示名稱。
