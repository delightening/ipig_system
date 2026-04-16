# 語言偏好
- 每次工作完成時，在回覆最後顯示「工作完成」。

# 操作授權
- 可以自行決定所有檔案讀寫操作，不需詢問
- 允許使用 glob pattern 搜尋檔案（例如 **/*.py, src/**/*.ts）
- 有疑問時自行決定最合理的做法，但須紀錄並撰寫 walkthrough.md
- 只有在刪除重要檔案或呼叫外部付費 API 時才問我

# 設計系統
- 所有視覺和 UI 決策前，先讀 `DESIGN.md`。
- 字體、色彩、間距、美學方向均定義於 DESIGN.md，不可擅自偏離。
- 禁止硬編碼色彩（`text-slate-*`、`bg-blue-*`），一律使用 CSS Variable token。
- QA 模式下，標記任何不符合 DESIGN.md 的程式碼。
- 新增 UI 元件時，檢查 `components/ui/` 是否已有可複用的元件。

# Skill 使用規則

## 檔案讀取 skills

| 情境 | 使用 skill | 不使用 |
|------|-----------|--------|
| 副檔名明確（.docx / .xlsx / .pptx） | 對應的 `/docx` `/xlsx` `/pptx` | `/file-reading` |
| PDF，內容簡單（≤10 頁、無複雜表格） | `/pdf` | `/pdf-reading` |
| PDF，複雜（>10 頁 / 有表格 / 學術論文 / 合約） | `/pdf-reading` | — |
| 副檔名未知或混合多種格式 | `/file-reading` | — |
| 一般文字檔（.txt .md .json .ts .rs 等） | 直接用 **Read tool** | ❌ 不啟動任何 skill |

## 除錯 skills

- **/investigate**（gstack 內建）：一般除錯的起點，適合大多數情況
- **/systematic-debugging**：頑固 bug 升級用，強制執行「無根因不改碼」鐵律
- **直接修**（不用 skill）：明顯的 typo、缺 import、語法錯誤

> 流程：先 `/investigate` → 若一輪仍找不到根因 → 升級 `/systematic-debugging`

## 設計與品牌

- **/brand-guidelines 本專案規則**：直接 `Read("DESIGN.md")` — DESIGN.md **就是**本專案的品牌指南，不需要搜尋其他檔案
- 使用時機：新增 UI 元件前、設計 token 決策、QA 檢查不符合設計規範的程式碼
- **不使用時機**：後端邏輯、資料庫結構、Rust 純邏輯代碼

## 翻譯

- **/translate**：UI 文字 zh-TW ↔ en 翻譯、i18n key 命名、文件翻譯
- **不使用時機**：資料庫欄位名稱（保持 snake_case 英文）、程式碼邏輯

## 內部溝通

- **/internal-comms**：事故報告、週報、系統異動公告、會議紀錄
- **不使用時機**：UI 文字（用 i18n `t()` 函數）、API 文件、程式碼注解、commit message

## Skill 建立

- **/skill-creator**：僅在使用者**明確要求**建立新 skill 時使用，不主動建議

# 文件記錄規則

## 核心原則
- **時間排序統一為反向時間序**（新→舊），適用於所有檔案的時間性內容。
- **同一事件只詳細記錄一次**，其他位置用摘要或指標。
- **每個檔案只做一件事**，職責不重疊。

## 各檔案職責

| 檔案 | 唯一職責 | 不該放的內容 |
|------|---------|-------------|
| `docs/TODO.md` | 任務追蹤（待辦 + 已完成項目） | ~~變更日誌~~（已封存） |
| `docs/PROGRESS.md` | 系統完成度概覽（§1-8）+ 唯一變更日誌（§9） | 任務狀態追蹤 |
| `DESIGN.md` | 設計系統規範 + 設計決策日誌 | 任務、進度 |

## 完成工作後的記錄流程

每次完成 TODO.md 中的項目時，依序執行：

1. **TODO.md**：該項目標記 `[x]`，更新「待辦統計」數量
2. **PROGRESS.md §9**：新增一筆變更紀錄（格式見下方）
3. **DESIGN.md Decisions Log**：僅在涉及設計/架構決策時新增

不需要在 TODO.md 變更紀錄區新增條目（該區已封存）。

## TODO.md 撰寫規範

### 章節排列順序（固定）
```
禁止事項 → P0 → P1 → P2 → P3 → P4 → P5
→ 歷史改善計畫（P2-R3、P0-P2改進、R4-100）
→ R6 → R7 → R8 → R9 → R10 → R11 → R12 → R13 → ...（嚴格遞增）
→ 待辦統計
→ 變更紀錄（封存，不再新增）
```

### 表格欄位（新增 section 時統一使用）
```markdown
| # | 項目 | 說明 | 狀態 |
```
- 「範圍」「建議 AI」「來源」等資訊合併寫入「說明」欄。
- 歷史 section 保留原格式不回溯修改。

### 編號格式
```
{Section}-{序號}    例：R13-1, R14-2, P0-3
```
- 序號從 1 開始連續遞增，不跳號。
- 不嵌套優先級或嚴重度（例如不用 `R7-P0-1`）。

## PROGRESS.md §9 變更紀錄格式

```markdown
### YYYY-MM-DD 簡短標題

- ✅ **粗體摘要**：細節說明（一句話）
- ✅ **粗體摘要**：細節說明
```

- 反向時間序（新條目加在 §9 最上方）。
- 每個 bullet 以 `✅` 開頭 + 粗體摘要。
- 同一天多次工作合併為同一個 `###` 條目。

## DESIGN.md Decisions Log 格式

```markdown
| Date | Decision | Rationale |
```

- 反向時間序（新決策加在表格最上方，header 之後）。
- 僅記錄設計/架構決策，不記錄任務完成狀態。

---

# 代碼規範

> 本專案門檻優先於 common/ 預設值。若有衝突，以下方數值為準。

## 1. 通用原則

1. **DRY (Don't Repeat Yourself)**：重複邏輯超過 2 處必須抽出共用函式，放入對應層級的目錄。
2. **最小依賴**：未使用的 import、變數、函式一律移除。
3. **扁平優於巢狀**：避免超過 3 層的條件巢狀，優先使用 early return。
4. **單一職責**：每個檔案只做一件事。超過 300 行應考慮拆分。
5. **命名一致性**：Rust 用 snake_case，TypeScript 用 camelCase，React 元件用 PascalCase。

## 2. 量化門檻

| 指標 | 上限 | 說明 |
|------|------|------|
| **函數長度** | ≤ 50 行 | Rust 語法特性下的平衡值（嚴格 40 / 寬鬆 60 之間）。超過即拆分 |
| **圈複雜度** | ≤ 10 | 符合業界標準 |
| **參數數量** | ≤ 5 個 | 超過封裝為 struct（Rust）或 config object（TS） |
| **巢狀深度** | ≤ 3 層 | 使用 early return、`?` 運算子減少巢狀 |
| **React 元件檔案** | ≤ 300 行 | 超過即拆分為子元件或自定義 hook |
| **React JSX return** | ≤ 80 行 | 超過即將區塊提取為獨立元件 |
| **單一 useEffect / handler** | ≤ 30 行 | 超過即提取邏輯為函數或 hook |
| **Rust match 分支數** | ≤ 7 支 | 超過考慮提取為 enum method 或查表 |
| **React 元件 Props** | ≤ 6 個 | 超過考慮合併為 config object 或拆分元件 |

## 3. 架構分層與依賴方向

```
Handler → Service → Repository → Model
   ↓         ↑
Middleware ──┘

Utils（純函式，任何層皆可呼叫，但 Utils 本身不依賴任何層）
```

**規則：**
- 依賴只能**向下**，禁止反向依賴
- `utils/` 不依賴任何業務模組，不依賴 `AppState`
- `models/` 不依賴任何其他層
- Middleware 可呼叫 Service（例如認證中間件驗證 token）

## 4. Backend 模組職責 (Rust / Axum)

### 目錄職責表

| 目錄 | 職責 | 禁止事項 |
|------|------|----------|
| **`handlers/`** | HTTP 請求解析與回應組裝 | ❌ 業務邏輯、SQL、複雜條件判斷 |
| **`services/`** | 核心業務邏輯、權限檢查 | ❌ 直接建構 HTTP response、直接寫 SQL |
| **`repositories/`** | 封裝所有 SQL 查詢（sqlx::query） | ❌ 業務邏輯判斷 |
| **`models/`** | DB entity（`FromRow`）+ API request/response DTO | ❌ 業務邏輯、SQL、依賴其他層 |
| **`middleware/`** | 橫切關注點（認證、CSRF、限流、ETag） | ❌ 業務邏輯 |
| **`utils/`** | 純函式工具（日期、字串、加密） | ❌ 依賴 AppState 或業務型別 |
| **`startup/`** | 應用程式啟動初始化（DB、migration、seed） | ❌ runtime 呼叫 |
| **`bin/`** | CLI 維運工具 | 過時工具定期清理 |

### 基礎設施檔案

| 檔案 | 職責 | 禁止事項 |
|------|------|----------|
| **`error.rs`** | 統一錯誤型別 `AppError` enum + `IntoResponse` 實作 | — |
| **`config.rs`** | 環境變數讀取，`Config` struct | ❌ 散落讀取 `std::env::var` |
| **`constants.rs`** | 全域常數（狀態字串、魔術數字、預設值） | — |

### Backend 專項規則

- 錯誤處理統一使用 `AppError`，禁止裸用 `unwrap()` 於非測試碼。
- `expect()` 僅允許於程式啟動初始化階段（如 `main.rs`），須附帶描述訊息。
- 禁止 `#[allow(dead_code)]`、`#[allow(unused)]`，未使用的程式碼直接刪除。
- 測試碼禁止使用 `unwrap_err()`，一律改用 `expect_err("描述訊息")`（clippy `unwrap_used` 規則）。
- 資料庫查詢使用 SQLx，盡量用具名參數，禁止字串拼接 SQL。
- 相同 SQL SELECT 出現 ≥2 次，必須提取至 `repositories/` 層。
- 魔術字串（狀態字串、設定值）必須定義為 `const` 或 `enum`。
- Service 層應呼叫 Repository 取得資料，而非直接寫 SQL。
- 權限檢查邏輯放 `services/access.rs`（非 `utils/`）。

### Repository 函式命名慣例

| 操作 | 命名慣例 | 範例 |
|------|----------|------|
| 查詢單筆 | `find_{entity}_by_{field}` | `find_animal_by_id` |
| 查詢多筆 | `list_{entities}` 或 `find_{entities}_by_{field}` | `list_animals`、`find_animals_by_status` |
| 新增 | `insert_{entity}` | `insert_animal` |
| 更新 | `update_{entity}` | `update_animal` |
| 刪除 | `delete_{entity}` | `delete_animal` |
| 檢查存在 | `exists_{entity}_by_{field}` | `exists_animal_by_id` |

## 5. Frontend 模組職責 (TypeScript / React)

### 目錄職責表

| 目錄 | 職責 | 禁止事項 |
|------|------|----------|
| **`pages/`** | 頁面級元件（對應路由）。可包含同層 `components/`、`hooks/`、`constants.ts` | ❌ 定義跨頁面復用的元件或 hook |
| **`components/`** | 可復用元件（≥2 頁面使用）。按業務域分子目錄。`ui/` 為基礎 UI 元件庫 | ❌ 頁面級路由邏輯 |
| **`hooks/`** | 全域共用 Custom Hooks（≥2 頁面使用）。檔名以 `use` 開頭 | ❌ 包含 JSX |
| **`lib/`** | 核心工具（utils, validation, queryKeys, logger 等） | ❌ React 相關邏輯 |
| **`lib/api/`** | API 層：按業務域拆分 API 函式 | — |
| **`lib/constants/`** | 跨頁面的設定常數、狀態對應表（≥10 行的常數物件） | — |
| **`types/`** | 共用型別定義，每個業務域一個檔案 | ❌ 在頁面中重複定義相同型別 |
| **`stores/`** | Zustand 全域狀態（auth、UI 偏好） | ❌ 頁面級狀態 |

### Frontend API 細分規範

```
lib/
  api/
    client.ts       # Axios 實例設定、interceptors
    animal.ts       # 動物相關 API 函式
    hr.ts           # 人資相關 API 函式
    protocol.ts     # 實驗計畫相關 API 函式
    ...             # 按業務域拆分
    index.ts        # 統一匯出
```

### Zustand Store 決策流程

1. 該狀態是否需要在**不相鄰的元件**間共用？→ 否 → props / context
2. 該狀態是否需要**跨路由存活**？→ 否 → React state 或 URL search params
3. 該狀態是否需要**頁面重新整理後存活**？→ 是 → Zustand + persist middleware
4. 以上皆是 → Zustand Store

### Frontend 專項規則

- API 呼叫統一使用 TanStack Query，禁止裸用 `fetch` 或 `axios`。
- 驗證邏輯統一使用 Zod schema（`lib/validation.ts`），不重複手寫驗證。
- Custom Hook 提取時機：state + effect 邏輯超過 15 行，或在 ≥2 個元件重複時。
- 內聯常數（≥10 行）禁止放在頁面內，移至 `lib/constants/` 或同層 `constants.ts`。
- 未使用的 import / 變數禁止殘留，ESLint 零警告。

## 6. 檔案命名規範

| 層級 | Rust | TypeScript |
|------|------|------------|
| 檔案名 | `snake_case.rs` | `camelCase.ts` 或 `PascalCase.tsx`（元件） |
| Handler 函式 | `get_animal_list` | — |
| Service 函式 | `create_animal` | — |
| Repository 函式 | `find_animal_by_id` | — |
| React 元件 | — | `AnimalCard.tsx`（PascalCase） |
| Custom Hook | — | `useAnimalList.ts`（camelCase + `use` 前綴） |
| 常數檔 | `constants.rs`（UPPER_SNAKE_CASE） | `constants.ts`（camelCase） |
| 型別檔 | — | `animal.ts`（camelCase，在 `types/` 下） |

## 7. 重複邏輯合併策略

| 場景 | 合併目標位置 |
|------|-------------|
| 同一 SQL SELECT ≥2 次 | `repositories/` |
| 同一權限檢查邏輯 ≥2 處 | `services/access.rs` |
| 同一格式化/轉換函式 ≥2 處 | Backend: `utils/`、Frontend: `lib/utils.ts` |
| 同一驗證邏輯 ≥2 處 | Frontend: `lib/validation.ts`、Backend: `utils/validation.rs` |
| 同一 React state + effect ≥2 處 | `hooks/` 或頁面同層 `hooks/` |
| 同一 UI pattern ≥2 處 | `components/` |

## 8. Import 排序規範

### Rust
```
1. std 標準庫
2. 第三方 crate（按字母序）
3. （空行）
4. crate:: 內部模組
5. super:: / self:: 當前模組
```

**自動化**：`rustfmt` 搭配 `imports_granularity = "Module"` + `group_imports = "StdExternalCrate"`

### TypeScript
```
1. React 核心 (react, react-dom, react-router-dom)
2. 第三方庫 (TanStack Query, Zod, lucide-react 等)
3. （空行）
4. 內部模組 (@/lib/, @/hooks/, @/stores/)
5. 元件 (@/components/)
6. 型別 (@/types/)
7. 同層相對路徑 (./components/, ./hooks/)
```

**自動化**：ESLint `eslint-plugin-import` 的 `import/order` 規則

## 9. 統一錯誤處理

### Backend
- 所有 handler 回傳 `Result<impl IntoResponse, AppError>`。
- Service 層回傳 `Result<T, AppError>`。
- 使用 `?` 運算子逐層傳播，不在 handler 中 catch 後手動建構 response。
- 禁止 `unwrap()`（測試除外）。

### Frontend
- **全域 QueryClient onError**：在 `QueryClient` 預設配置統一處理 401/403/500。queries 401 不重試，其他 retry ≤3 次；mutations `onError` 統一呼叫 `toast.error(getApiErrorMessage(error))`。
- **React Error Boundary**：捕捉渲染階段錯誤，顯示 fallback UI（使用 `PageErrorBoundary`）。
- **頁面層級**：TanStack Query `onError` 處理特定業務錯誤。
- ❌ 禁止裸 try-catch + `console.error`。

## 10. 清理規則

- 移除所有 `#[allow(dead_code)]`、`#[allow(unused)]` 及對應的 dead code。
- 移除前端中被註解掉的程式碼區塊（超過 5 行的）。
- 移除未使用的 npm/cargo 依賴。
