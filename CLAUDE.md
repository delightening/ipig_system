# 語言偏好
- 一律使用繁體中文（台灣）回覆。技術術語保留英文。

# 操作授權
- 可以自行決定所有檔案讀寫操作，不需詢問
- 允許使用 glob pattern 搜尋檔案（例如 **/*.py, src/**/*.ts）
- 有疑問時自行決定最合理的做法，但須紀錄並撰寫 walkthrough.md
- 只有在刪除重要檔案或呼叫外部付費 API 時才問我

# 待辦與進度文件（請記住）
- 當 `docs/TODO.md` 中某項工作標為完成 [x] 時，務必同步：(1) 在 `docs/PROGRESS.md` 的「9. 最新變更動態」新增該完成項的簡短紀錄（日期、項目編號、產出摘要）；(2) 更新 TODO.md 的「待辦統計」未完成數量。已完成項目保留在 TODO 表中並標 [x]，不刪除。

# 代碼簡化與模組化規範

## 原則
1. **DRY (Don't Repeat Yourself)**：重複邏輯超過 2 處必須抽出共用函式，放入對應的 `utils/` 目錄。
2. **最小依賴**：未使用的 import、變數、函式一律移除。
3. **扁平優於巢狀**：避免超過 3 層的條件巢狀，優先使用 early return。
4. **單一職責**：每個檔案只做一件事。超過 300 行應考慮拆分。
5. **命名一致性**：Rust 用 snake_case，TypeScript 用 camelCase，React 元件用 PascalCase。

## 前端規範 (TypeScript / React)
- 共用工具函式統一放在 `frontend/src/lib/` 下。
- 共用 React hooks 統一放在 `frontend/src/hooks/` 下。
- 頁面級元件放 `pages/`，可復用元件放 `components/`。
- API 呼叫統一使用 TanStack Query，禁止裸用 `fetch` 或 `axios`。
- 驗證邏輯統一使用 Zod schema，不重複手寫驗證。
- 型別定義統一放在 `frontend/src/types/` 下，禁止在頁面中重複定義相同型別。

## 後端規範 (Rust / Axum)
- 共用工具函式統一放在 `backend/src/utils/` 下。
- Service 層處理業務邏輯，Handler 層只做請求解析與回應組裝。
- 錯誤處理統一使用 `AppError`，禁止裸用 `unwrap()` 於非測試碼。
- 資料庫查詢使用 SQLx，盡量用具名參數，禁止字串拼接 SQL。

## 清理規則
- 移除所有 `#[allow(dead_code)]`、`#[allow(unused)]` 及對應的 dead code。
- 移除前端中被註解掉的程式碼區塊（超過 5 行的）。
- 移除未使用的 npm/cargo 依賴。