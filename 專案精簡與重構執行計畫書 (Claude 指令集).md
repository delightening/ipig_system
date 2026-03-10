# 專案精簡與重構執行計畫書 (巢狀指令版)

本計畫採用巢狀層級設計，旨在將複雜的重構任務拆解為可控的微小步驟，藉此節省 Token 消耗，並在任務中斷時能快速精確地接續進度。

---

## 全域約束 (Global Constraints)

> **Claude 在執行本計畫時，必須遵守以下規則：**

1. **斷點即停**：遇到 `🔶 斷點` 時，必須停下來回報結果並等待使用者確認，不得自動跳到下一步。
2. **交付物必呈**：每個子任務都有明確的「交付物」，完成後必須呈現給使用者。
3. **進度追蹤**：完成一個子任務後，回到本文件的「進度追蹤」區段，將對應項目勾選為 `[x]`。
4. **禁止越級**：不得跳過任何子任務，即使你認為它是多餘的。
5. **錯誤即停**：如果執行過程中遇到編譯錯誤或測試失敗，立即停下回報，不要嘗試自行修復（除非該步驟指令明確要求修復）。
6. **規範優先**：所有變動必須符合 `CLAUDE.md` 中定義的規範。

---

## 接續機制 (Resume Protocol)

> **如果任務中斷（連線斷開、Token 耗盡等），恢復時請使用以下指令：**

```
請讀取「專案精簡與重構執行計畫書 (Claude 指令集).md」的進度追蹤區段，
告訴我目前完成到哪一步，然後從下一個未完成的任務繼續執行。
```

---

## 進度追蹤 (Progress Tracker)

### Task 01: 建立專案規範

- [ ] 01a-1: 目錄結構掃描
- [ ] 01a-2: 代碼風格採樣
- [ ] 01b-1: 起草結構與拆解標準
- [ ] 01b-2: 定義模組職責與精簡原則
- [ ] 01c-1: 寫入 CLAUDE.md

### Task 02: 邏輯去脂執行

- [ ] 02a-1: 判斷式優化 (Backend)
- [ ] 02a-2: 判斷式優化 (Frontend)
- [ ] 02a-3: 錯誤處理統一化
- [ ] 02b-1: 移除 Dead Code

### Task 03: 工具函式大遷徙

- [ ] 03a-1: 輔助函式全域掃描
- [ ] 03b-1: 模組化遷移
- [ ] 03b-2: 全域引用自動校準

### Task 04: 最終審計與量化

- [ ] 04a-1: 靜態分析與規範校驗
- [ ] 04b-1: 優化數據統計
- [ ] 04b-2: 撰寫重構總結

---

# Task 01: 建立專案規範 (The Architect)

**目標：** 透過掃描與共識，建立 `CLAUDE.md` 作為後續重構的唯一執行憲法。
**前置條件：** 無
**完成條件：** `CLAUDE.md` 已更新並經使用者確認。

## Task 01a: 專案現狀偵查 (The Snapshot)

### Task 01a-1: 目錄結構掃描

- **指令**：列出以下目錄的樹狀結構（排除 `node_modules`, `.git`, `.venv`, `target`, `dist`, `coverage`）：
  - `backend/src/` — Rust 後端核心（handlers, services, models, middleware, utils）
  - `frontend/src/` — React/Vite 前端核心（components, pages, hooks, lib, types, stores）
  - `scripts/` — 輔助腳本
  - `tests/` — 測試檔案
- **交付物**：一份完整的目錄樹狀圖，並附註各目錄的推測職責。
- 🔶 **斷點**：呈現樹狀圖，等待使用者確認目錄權責劃分是否正確。

### Task 01a-2: 代碼風格採樣

- **指令**：讀取以下檔案並分析代碼風格：
  - **Backend**：`backend/src/main.rs`, `backend/src/routes.rs`, 以及 `backend/src/services/` 下任意 2 個 service 檔案
  - **Frontend**：`frontend/src/App.tsx`, 以及 `frontend/src/pages/` 下任意 2 個頁面元件
- **分析項目**：命名慣例（snake_case / camelCase）、函數長度分佈、巢狀深度、錯誤處理模式、import 組織方式。
- **交付物**：一份「代碼風格觀察報告」，以表格呈現各面向的現況與建議。
- 🔶 **斷點**：呈現觀察報告，等待使用者補充或修正觀察結果。

## Task 01b: 擬定規範草案 (The Draft)

### Task 01b-1: 起草結構與拆解標準

- **指令**：根據 01a 的觀察結果，草擬以下標準：
  - 函數長度上限（建議 ≤ 30 行，超過則必須拆分）
  - 巢狀深度上限（建議 ≤ 3 層）
  - Backend (Rust)：`match` 分支數上限、`unwrap()` 使用禁令
  - Frontend (React/TS)：元件 Props 數量上限、Custom Hook 提取時機
- **交付物**：條列式的「拆解標準草案」。
- 🔶 **斷點**：等待使用者確認標準是否合理，或需調整門檻。

### Task 01b-2: 定義模組職責與精簡原則

- **指令**：為以下目錄定義明確的存放規則：
  - **Backend**：`handlers/`（路由處理）、`services/`（商業邏輯）、`models/`（資料模型）、`middleware/`（中間件）、`utils/`（通用工具）
  - **Frontend**：`components/`（可複用元件）、`pages/`（頁面級元件）、`hooks/`（自定義 Hook）、`lib/`（API 呼叫與工具）、`types/`（型別定義）、`stores/`（狀態管理）
- **精簡原則**：重複邏輯合併策略、Import 排序規範、統一錯誤處理模式。
- **交付物**：條列式的「模組職責定義」與「精簡原則草案」。
- 🔶 **斷點**：等待使用者校閱並確認。

## Task 01c: 憲法固化 (The Commit)

### Task 01c-1: 寫入 CLAUDE.md

- **指令**：將 01b 確認的共識，合併至現有的 `CLAUDE.md`（保留原有的「語言偏好」、「操作授權」、「待辦與進度文件」區段）。新增「代碼規範」章節。
- **交付物**：更新後的 `CLAUDE.md` 完整內容。
- 🔶 **斷點**：呈現完整 `CLAUDE.md` 內容，等待使用者最終確認。

---

# Task 02: 邏輯去脂執行 (The Logic Thinning)

**目標：** 針對現有核心邏輯進行「內容精簡」，不改變檔案結構以降低風險。
**前置條件：** Task 01 全部完成。
**完成條件：** 所有核心邏輯符合 `CLAUDE.md` 規範，編譯通過。

## Task 02a: 冗餘邏輯清理

### Task 02a-1: 判斷式優化 — Backend (Rust)

- **指令**：掃描 `backend/src/handlers/` 與 `backend/src/services/`，將深層巢狀的 `if/else` 與 `match` 改為 Early Return 或 `?` 運算子。
- **約束**：每次最多修改 5 個檔案，修改完一批就停下來。
- **交付物**：修改清單（檔案名稱 + 修改前後對比摘要）。
- 🔶 **斷點**：呈現修改清單，執行 `cargo check` 確認編譯通過，等待使用者確認後再處理下一批。

### Task 02a-2: 判斷式優化 — Frontend (React/TS)

- **指令**：掃描 `frontend/src/pages/` 與 `frontend/src/components/`，將冗長的條件渲染改為 Early Return 或獨立元件。
- **約束**：每次最多修改 5 個檔案。
- **交付物**：修改清單（檔案名稱 + 修改前後對比摘要）。
- 🔶 **斷點**：呈現修改清單，執行 `npm run build` 確認編譯通過，等待使用者確認。

### Task 02a-3: 錯誤處理統一化

- **指令**：
  - **Backend**：將散落的 `unwrap()` 替換為 `?` 或 `expect("描述")`，統一使用 `backend/src/error.rs` 定義的錯誤類型。
  - **Frontend**：統一 API 呼叫的錯誤處理模式（如：統一使用 try-catch + toast 通知）。
- **交付物**：錯誤處理統一化報告。
- 🔶 **斷點**：確認編譯通過，等待使用者確認。

## Task 02b: 代碼清理 (Housekeeping)

### Task 02b-1: 移除 Dead Code

- **指令**：
  - **Backend**：執行 `cargo clippy` 並修復所有 `dead_code`、`unused_imports`、`unused_variables` 警告。
  - **Frontend**：執行 ESLint 並修復所有 `no-unused-vars`、`no-unused-imports` 警告。
  - 移除過時的註解（如 `// TODO: 暫時先這樣`、被註解掉的代碼區塊）。
- **交付物**：清理報告（移除了哪些項目、各幾處）。
- 🔶 **斷點**：確認編譯通過，等待使用者確認。

---

# Task 03: 工具函式大遷徙 (The Batch Migration)

**目標：** 合併重複功能，建立統一的 Utility 模組。
**前置條件：** Task 02 全部完成。
**完成條件：** 無重複的輔助函式，所有引用路徑正確。

## Task 03a: 重複功能辨識

### Task 03a-1: 輔助函式全域掃描

- **指令**：掃描以下範圍，找出功能相似或重複的輔助函式：
  - **Backend**：`backend/src/utils/`、`backend/src/services/` 中散落的 helper functions
  - **Frontend**：`frontend/src/lib/`、`frontend/src/hooks/`、各元件中內嵌的通用工具
- **分類**：日期/時間處理、字串格式化、驗證邏輯、API 工具、型別轉換。
- **交付物**：一份「重複函式清點表」，標註每個函式的所在位置與建議合併目標。
- 🔶 **斷點**：呈現清點表，等待使用者確認合併方案。

## Task 03b: 執行遷移與更新

### Task 03b-1: 模組化遷移

- **指令**：根據使用者確認的合併方案，將工具函式遷移至：
  - **Backend**：`backend/src/utils/` 下的獨立模組（如 `date.rs`, `validation.rs`）
  - **Frontend**：`frontend/src/lib/` 下的獨立模組（如 `date-utils.ts`, `format-utils.ts`）
- **約束**：單一檔案僅負責單一職責，函式需有完整的 Docstring。
- **交付物**：遷移對照表（原始位置 → 新位置）。
- 🔶 **斷點**：確認新模組的組織結構，等待使用者確認。

### Task 03b-2: 全域引用自動校準

- **指令**：更新專案中所有相關檔案的 Import / use 路徑，確保指向新的模組位置。
- **驗證**：
  - Backend：執行 `cargo check` 確認編譯通過
  - Frontend：執行 `npm run build` 確認編譯通過
- **交付物**：引用更新清單 + 編譯結果截圖/日誌。
- 🔶 **斷點**：確認所有路徑**有效**，等待使用者確認。

---

# Task 04: 最終審計與量化 (The Audit)

**目標：** 確認重構品質，並提供具體的優化數據。
**前置條件：** Task 03 全部完成。
**完成條件：** 所有檢查通過，重構總結已交付。

## Task 04a: 重構品質檢查

### Task 04a-1: 靜態分析與規範校驗

- **指令**：
  - Backend：執行 `cargo clippy -- -W warnings` 確認零警告
  - Frontend：執行 `npx eslint src/` 確認零警告
  - 抽查 5 個修改過的檔案，逐條比對 `CLAUDE.md` 規範
- **交付物**：靜態分析結果 + 規範符合度報告。
- 🔶 **斷點**：如有殘留問題，修復後再次回報。

## Task 04b: 成果量化回報

### Task 04b-1: 優化數據統計

- **指令**：統計以下指標（使用 `git diff --stat` 或 `cloc`）：
  - 總代碼行數變化（Before / After）
  - 移除的重複函式數量
  - 合併後的模組數量
  - 平均函數長度變化
  - 最大巢狀深度變化
- **交付物**：一份表格化的「重構成果數據報告」。
- 🔶 **斷點**：呈現數據，等待使用者確認。

### Task 04b-2: 撰寫重構總結

- **指令**：產出一份 `docs/REFACTORING_SUMMARY.md`，包含：
  - 重構目標回顧
  - 各階段執行摘要
  - 量化成果
  - 後續維護建議（如：建議定期執行 Clippy / ESLint 檢查）
- **交付物**：`docs/REFACTORING_SUMMARY.md` 完整內容。
- 🔶 **斷點**：**任務結束**，呈現總結，等待使用者最終審閱。
