# 藥物選單重複項目 — 分析與整合建議

> **情境：** 藥物選單管理頁出現多筆相同藥物（如 Atropine、Stroless、O2 等各出現多次），使用者詢問是否應整合、業界如何處理。

---

## 1. 重複來源分析

目前系統中重複項可能來自：

| 來源 | 說明 |
|------|------|
| **Seed 資料重複執行** | `009_supplementary.sql` 的 `INSERT INTO treatment_drug_options` 若在不同環境或重跑 migration 時執行多次，會產生多筆相同 name/display_name 的資料。 |
| **手動「新增藥物」** | 無「同名稱＋同分類已存在」檢查，可重複新增同一藥物。 |
| **從 ERP 匯入** | 後端僅依 `erp_product_id` 防重：同一 ERP 產品不會匯入兩次；但若同一邏輯藥品在 ERP 有多個產品（不同 SKU/規格），或先手動新增再從 ERP 匯入同品名，會產生多筆。 |
| **缺少唯一約束** | `treatment_drug_options` 表無 `UNIQUE (name, category)` 或類似約束，資料庫層級不阻止重複。 |

因此，「同一邏輯藥物」在表內可能對應多個 `id`，僅在「從 ERP 匯入且同一 product_id」時會跳過。

---

## 2. 業界常見做法

- **單一主檔（Master Data）**  
  一個邏輯藥物 = 一筆主檔。下拉選單、報表、統計皆以主檔為準，避免同一藥品多個代碼造成報表與選單混亂。

- **以業務鍵防重**  
  以「業務鍵」唯一識別一筆藥物，例如：
  - `(normalize(name), category)`，或  
  - 有 ERP 時：`erp_product_id` 唯一（一產品對一藥物選項）。  
  新增/匯入時：若業務鍵已存在則 **upsert（更新既有）** 或 **回傳既有並提示**，不新增第二筆。

- **ERP 同步策略**  
  - 以 `erp_product_id` 為外部鍵：同一 ERP 產品只對應一筆藥物選項；匯入時有則更新、無則新增。  
  - 若同一邏輯藥品在 ERP 有多個產品（如不同包裝），可選擇：  
    - 仍維持「一邏輯藥物一筆」並在該筆上關聯多個 ERP 產品或僅選一個代表產品，或  
    - 明確定義「一 ERP 產品一藥物選項」並在 UI 標示為不同規格，避免使用者誤以為是重複。

- **既有重複的處理（一次性清理）**  
  - 依「名稱＋分類」或「名稱＋顯示名稱」分組，每組保留一筆 canonical（例如最早建立或已連結 ERP 者）。  
  - 將其他重複項的 `id` 對應到 canonical `id`，更新所有引用（如 `animal_observations.treatments` JSONB 內的 `drug_option_id`）。  
  - 再將重複列停用或刪除，並在之後以唯一約束防止再發生。

---

## 3. 建議方向：應整合在一起

建議 **將同一邏輯藥物整合為一筆**，理由：

1. **選單與報表一致**：下拉選單不重複、報表與統計不會因多個 id 而分散。  
2. **符合主檔思維**：藥物選單本質為參考資料，一藥一筆較易維護與稽核。  
3. **與 ERP 對齊**：若未來以 ERP 為主，可採「一 ERP 產品一藥物選項」或「一邏輯藥物一選項＋多產品關聯」，但都應避免無意義的重複列。

實作上可分兩階段：

- **短期（防新重複）**  
  - 新增/匯入前檢查「同 name（及必要時 category）是否已存在」；若存在則更新或回傳既有並提示，不新增。  
  - 可選：在 DB 加 `UNIQUE (lower(trim(name)), category)`（或等同欄位）以強制唯一。

- **中期（清理既有重複）**  
  - 撰寫一次性腳本或後台「合併重複藥物」功能：  
    - 依 name（＋ display_name/category）分組；  
    - 每組選一筆 canonical；  
    - 更新 `animal_observations.treatments` 等處的 `drug_option_id` 指向 canonical；  
    - 停用或刪除其餘重複列。  
  - 執行前備份 DB，並在測試環境驗證。

---

## 4. 與現有程式/資料的關係

- **Observations 的 treatments**：以 JSONB 儲存，內含 `drug_option_id`。合併時需將舊的 option id 對應到保留的 canonical id，不影響「當時用了哪種藥」的語意，僅統一指向同一選項。  
- **DrugCombobox**：目前以 `treatment-drugs` API 列出選項；合併後選項變少、不重複，使用者體驗會更好。  
- **從 ERP 匯入**：維持以 `erp_product_id` 防重；可再加「同 name＋category 已存在則更新該筆的 erp_product_id」等策略，避免再產生同藥多筆。

---

## 5. 小結

| 問題 | 建議 |
|------|------|
| 是否應該整合？ | **是**，同一邏輯藥物建議整合成一筆。 |
| 業界做法？ | 主檔唯一、以業務鍵防重、ERP 以外部鍵 upsert、既有重複做一次性合併並更新引用。 |
| 下一步 | 1）新增/匯入時防重（檢查＋可選 DB UNIQUE）；2）規劃一次性重複合併腳本並更新 `drug_option_id` 引用。 |

---

## 6. 已實作項目（2026-03-05）

- **Migration 015**（`backend/migrations/015_treatment_drug_options_deduplication.sql`）  
  - 依業務鍵分組，每組保留一筆 canonical（優先有 `erp_product_id`、其次 `created_at` 最早）。  
  - 更新 `animal_observations.treatments`、`animal_surgeries` 的 `induction_anesthesia` / `pre_surgery_medication` / `anesthesia_maintenance` / `post_surgery_medication` 中之 `drug_option_id` 指向 canonical。  
  - 將重複列軟刪除（`is_active = false`）。

- **Migration 016**（`backend/migrations/016_treatment_drug_options_unique_business_key.sql`）  
  - 新增部分唯一索引：`(lower(trim(name)), COALESCE(category, '')) WHERE is_active = true`，防止之後再產生同名稱＋分類的啟用重複項。

- **後端**（`backend/src/services/treatment_drug.rs`）  
  - `create`：新增前檢查業務鍵，若已存在啟用項則回傳 `409 Conflict`。  
  - `update`：若名稱或分類變更，檢查新業務鍵是否與他筆衝突，若有則 `409`。  
  - `import_from_erp`：同 `erp_product_id` 跳過；同名稱＋分類若已有啟用項則更新該筆的 `erp_product_id`／單位並列入回傳，不新增列。

- **前端**  
  - 新增/編輯表單下方提示：「同一『藥物名稱＋分類』僅能有一筆啟用項目」。  
  - 409 時以後端回傳訊息顯示（沿用既有 `getApiErrorMessage`）。

**部署時請依序執行：** `sqlx migrate run`（先 015 再 016）。執行前建議備份資料庫。
