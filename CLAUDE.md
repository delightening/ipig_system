# 語言偏好
- 每次工作完成時，在回覆最後顯示「工作完成」。

# 操作授權
- 可以自行決定所有檔案讀寫操作，不需詢問
- 允許使用 glob pattern 搜尋檔案（例如 **/*.py, src/**/*.ts）
- 有疑問時自行決定最合理的做法，但須紀錄並撰寫 walkthrough.md
- 只有在刪除重要檔案或呼叫外部付費 API 時才問我

# 執行紀律（大型重構計畫期間）
對應計畫檔：`C:/Users/admin/.claude/plans/plan-for-the-critical-validated-pebble.md`

## 測試驗證標準（問題 1：依 PR 性質）
- **PR 類別判斷**：
  - **純 infra / models / services 層**（不改 handler）→ 最小 `cargo test --lib` 綠燈即可
  - **動到 handlers / middleware / routes 層** → 必須 `cargo test --all-targets` 全綠（含整合測試，需本地啟動 Postgres：`docker compose -f docker-compose.test.yml up -d postgres`）
  - **只動 CLAUDE.md / docs / migration SQL** → `cargo check` 綠燈即可，不需跑 test
- **不確定時**：主動問使用者該 PR 屬哪類。

## 停機規則
- **跨 PR 邊界必停**：每個 PR 做完適用的測試命令綠燈 + `git commit` 後必須停下，不得自動 push、不得自動開下一個 PR、不得自動 merge。
- **測試 / 編譯失敗必停**：`cargo check` / `cargo test` / clippy 任一紅燈立即停，回報問題，不得硬闖。
- **pattern 驗證必停**（配合做法 β，問題 4）：
  - PR #3 (Protocol Service-driven 示範) 完成後 **必停一次**，由使用者確認 pattern 可複製
  - PR #4 animals + PR #5 hr/leave 可在**同 session 內合併做完再停**（因 pattern 已驗證）
  - PR #6 起恢復「每 PR 一次停」節奏

## 不可逆操作必經明確同意
- `git push`、`git reset --hard`、`git force-push`
- Migration 跑到 **staging / production** DB（**dev DB 自動 OK**：app 啟動自動跑 `sqlx::migrate!`，不需逐次問，問題 2）
- Merge PR 到 main
- 新增 / 移除 dependency（Cargo.toml / package.json）
- 修改 `.env` / `secrets/`
- 修改 CI 設定檔（`.github/workflows/*`）

## Clippy 過關標準（問題 5）
- **目前階段（PR #1 ~ R26-4 完成前）**：
  ```
  cargo clippy --all-targets -- -D warnings -A deprecated
  ```
  `-A deprecated` 用於容忍舊版 `AuditService::log_activity` 的過渡期警告（由 `#[deprecated]` 產生、預期出現在遷移中的 handler 呼叫點）。
- **R26-4 完成後**（舊版 `log_activity` 移除）：恢復嚴格 `-D warnings`，不含 `-A deprecated`。
- **新 PR 不得引入新的 deprecated 警告或其他 warning**（現有 91 處只減不增）。

## Commit 粒度（問題 3：放寬）
- **一般原則**：每個邏輯單元 + 其測試一次 commit。PR 內至少 3 個 commit、最多 15 個。
- **INFRA 類 PR**（如 PR #1）：按元件切，例如：
  - commit 1: migration + actor.rs (INFRA-1)
  - commit 2: DataDiff + AuditRedact (INFRA-3)
  - commit 3: AuditService::log_activity_tx (INFRA-2)
  - commit 4: CancellationToken + main.rs + jwt_blacklist (INFRA-4 基礎)
  - commit 5: scheduler.rs 各 job 接 token (INFRA-4 scheduler)
- **Service-driven 模組重構 PR**（PR #3+）：每 service fn + 呼叫端 handler 調整 + audit + tests 算一個邏輯單元。
- **避免**：單一 commit 改動 > 500 lines 或跨 > 3 個不相關模組。

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

## 表格設計（強制）

- **/system_table_chats**：**新增或修改任何 React 表格元件前必須執行**
- 涵蓋：RWD 審計 → 欄位優先級討論 → ≤3 個斷點確認 → HTML 預覽 → 實作計畫
- 實作目標：`@tailwindcss/container-queries`，禁止新增 JS ResizeObserver 斷點邏輯
- **不使用時機**：純後端、無 UI 的資料表結構變更

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
- **禁止將 custom hook 回傳的整個物件放入 `useEffect`/`useCallback`/`useMemo` deps 陣列**。Custom hook return value 是 plain object，每次 render 都是新 reference，放入 deps 會導致 effect 在每次渲染觸發（可能造成無限迴圈或覆蓋使用者輸入）。應解構取出具體的穩定值（`useRef` 物件、`useState` setter、`useCallback` 函式）再放入 deps。

### useEffect deps 穩定性速查表

寫 `useEffect`/`useCallback`/`useMemo` 時，放入 deps 前逐一確認：

| 值的來源 | Reference 穩定? | 正確做法 |
|---|---|---|
| `useRef(...)` 回傳值 | ✅ 穩定 | 可直接放入 deps |
| `useState` setter（`setX`） | ✅ 穩定 | 可直接放入 deps |
| `useCallback(fn, [...])` | ✅ 穩定（deps 不變時） | 可直接放入 deps |
| `useMemo(() => val, [...])` | ✅ 穩定（deps 不變時） | 可直接放入 deps |
| **custom hook 回傳的 `{}` 物件** | ❌ 每次 render 都新 | 解構取出上方穩定值再放入 |
| 元件 props 裡的 callback | ⚠️ 視上層是否用 `useCallback` | 確認後再放 |
| 陣列/物件 literal（`[]`, `{}`） | ❌ 每次 render 都新 | 移出 component 或用 `useMemo` |

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

<!-- rtk-instructions v2 -->
# RTK - Rust 指令前綴規則

對所有 Rust / Git / GitHub 指令加 `rtk` 前綴：

```bash
rtk cargo build
rtk cargo check
rtk cargo clippy
rtk cargo test
rtk git status / log / diff / add / commit / push / pull
rtk gh pr view / pr checks / run list
rtk docker ps / images / logs
```
<!-- /rtk-instructions -->