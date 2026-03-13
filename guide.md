# ipig_system 大型專案開發與 Claude 溝通指南

這份文件旨在優化與 Claude 的對話效率，透過結構化的溝通與 Token 節省策略，讓 AI 能精確掌握專案現況。

---

## 🚀 核心溝通策略：XML 標籤定位化

為了避免 Claude 每次對話都重新讀取全部代碼造成 Token 浪費，請遵循以下流程。

### 1. 初始建立：專案地圖與索引
第一次將專案交給 Claude 時，請使用以下 Prompt：

> 「這是一個名為 `ipig_system` 的大型專案。請先讀取下方的 XML 結構化代碼。
> 1. 請將這些內容建立索引，之後我會根據路徑（如 `backend/src/main.rs`）向你提問。
> 2. 讀取完畢後，**請勿摘要全文**，僅需簡單回覆：『專案代碼已建立索引，包含 [X] 個檔案，隨時可以開始討論特定模組。』」

---

### 2. 結構化代碼包裝格式
將檔案貼給 Claude 時，請務必使用這種標籤格式：

```xml
<project_context>
  <file path="backend/src/services/notification/erp.rs">
    // ... 代碼內容 ...
  </file>
  <file path="docs/PROGRESS.md">
    // ... 文件內容 ...
  </file>
</project_context>
```

---

## 🛠 推薦工具：Repomix (原 Packu)

建議在本地端使用 `Repomix` 將整個專案打包成單一優化過的文字檔。

**執行方式：**
```powershell
# 在專案根目錄執行
npx repomix
```
這會產生一個 `repomix-output.xml`。將此檔案內容提供給 Claude，它會自動將所有檔案路徑標記為「已檢索」。

---

## 💡 高效提問範例

建立索引後，請養成「具體點名」的提問習慣：

- **❌ 錯誤示範**：
  > 「幫我改一下發送通知的邏輯。」
  > *(原因：Claude 需要自己去搜尋，容易迷失或混淆不同模組。)*

- **✅ 正確示範**：
  > 「根據你在 `backend/src/services/notification/erp.rs` 中讀取的內容，我需要修改 `send_po_notification` 的處理邏輯。」
  > *(原因：強迫 Claude 僅搜索特定區塊，減少運算量且提高準確度。)*

---

## 📂 專案模組快速索引

如果您想直接引導 Claude 進入特定領域，可以這樣描述：

- **後端核心 (Backend)**: `backend/src/` (Rust)
- **前端介面 (Frontend)**: `frontend/src/` (React/TypeScript)
- **資料庫遷移 (Migrations)**: `migrations/*.sql`
- **現有進度與待辦**: `docs/PROGRESS.md`, `docs/TODO.md`

---

## ⚠️ 注意事項
1. **快取 (Caching)**: 在同一個對話視窗內，Claude 會記住之前的 Context。除非你開啟「新對話」，否則不需要重新貼上舊代碼。
2. **區分環境**: 如果是在 Web 介面提問，建議使用 **Claude Projects** 功過，直接把 `guide.md` 和關鍵代碼上傳到 Project Knowledge。
3. **排除無用資料**: 打包時請務必排除 `target/`, `node_modules/`, `.git/` 等目錄。

---
*上次更新時間: 2026-03-13 20:41*
