# E2E 動物列表失敗原因探究與修復計畫

## 1. 失敗現象

- **規格**：`frontend/e2e/animals.spec.ts` 的 6 個測試全部在 `beforeEach` 階段失敗。
- **錯誤**：`page.waitForLoadState('networkidle')` 在 30 秒內未達成，導致 Test timeout。

## 2. 根本原因

### 2.1 `networkidle` 的定義與風險

Playwright 的 `waitForLoadState('networkidle')` 代表：

- 至少連續 **500ms** 內，網路連線數 ≤ **0**，導航才視為完成。

在 SPA 且有多個 API、或背景輪詢的頁面，此條件經常**難以在短時間內達成**，甚至永遠不達成。

### 2.2 動物列表頁實際行為

- **`/animals` 對應**：`AnimalsPage.tsx`。
- **載入時會觸發的請求**（含 Layout）：
  - Layout：`/notifications/unread-count`、`/amendments/pending-count`（且設有 `refetchInterval: 60000`）。
  - AnimalsPage：`/animals`（列表）、`/animals`（count）、`/animal-sources`；若為欄舍視圖還有 `/animals/by-pen`。
- **React Query 設定**：多筆查詢為 `refetchOnMount: true`、`refetchOnWindowFocus: true`，`staleTime: 0`，因此一進入頁面會並行發出多個請求。

因此：

- 多個請求並行或依序進行時，很難出現「連續 500ms 完全無請求」的窗口。
- 若後端較慢或偶發延遲，`networkidle` 會一直等不到，直到 30s 逾時。

### 2.3 其他使用 `networkidle` 的 E2E

以下規格也在 `beforeEach` 使用 `waitForLoadState('networkidle')`，有類似風險：

- `e2e/profile.spec.ts`（個人資料）
- `e2e/protocols.spec.ts`（計畫書列表）
- `e2e/dashboard.spec.ts`、`e2e/login.spec.ts`、`e2e/auth.setup.ts`（單次導航，影響較小）

動物列表因請求多、資料量可能較大，最容易先爆出逾時。

## 3. 修復計畫

### 3.1 原則

- **不要依賴 `networkidle`** 作為「頁面可操作」的判斷；改用「**以 DOM 穩定元素為準**」。
- 導航後：先等 `domcontentloaded` 或 `load`，再**等待代表頁面已就緒的單一元素**（例如搜尋框、主要 Tab），並設定合理 timeout（如 15s）。

### 3.2 建議修改（動物列表）

| 項目 | 目前 | 建議 |
|------|------|------|
| `beforeEach` 導航後 | `waitForLoadState('networkidle')` | `waitForLoadState('domcontentloaded')` 或 `'load'` |
| 就緒條件 | 無（僅依 networkidle） | 明確等待「頁面已渲染」：例如 `page.locator('input[placeholder]').first()` 或 Tab 按鈕 `toBeVisible({ timeout: 15_000 })` |
| 個別測試內 | 已有 `expect(...).toBeVisible({ timeout: 15_000 })` | 維持不變，作為雙重保障 |

效果：

- 不再依賴「網路完全靜止」，避免因多請求或後端延遲導致 30s 逾時。
- 以「使用者可見且可操作」的 DOM 為準，更貼近真實使用情境，也較穩定。

### 3.3 建議修改（其他 spec，可選）

- **profile.spec.ts**、**protocols.spec.ts**：同樣將 `beforeEach` 的 `networkidle` 改為 `domcontentloaded`/`load` + 等待該頁面獨有的穩定元素（例如 profile 的 disabled email input、protocols 的搜尋欄或新增按鈕），以預防未來在慢速或高負載下逾時。

### 3.4 不需改動的部分

- **auth.setup.ts**、**login.spec.ts**：登入流程為單一請求後跳轉，`networkidle` 風險較低；若日後仍出現偶發逾時，再改為等待登入後 URL 或首屏元素即可。

## 4. 實作項目摘要

1. **animals.spec.ts**：`beforeEach` 改為 `domcontentloaded`（或 `load`）+ 等待搜尋框或 Tab 可見（15s）。
2. **（可選）profile.spec.ts / protocols.spec.ts**：同上，改為依元素就緒，移除對 `networkidle` 的依賴。
3. 若 CI 或本機仍有偶發逾時，可再考慮對 `animals` 或相關 spec 適度提高 `test.setTimeout`，但優先以「元素等待」為主，timeout 為輔。

---

*文件建立後，依此計畫修改 e2e 並重跑驗證。*
