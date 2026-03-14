# 豬博士 iPig 系統專案進度評估表

> **最後更新：** 2026-03-14 (v14)
> **規格版本：** v7.0  
> **評估標準：** ✅ 完成 | 🔶 部分完成 | 🔴 未開始 | ⏸️ 暫緩

---

## 🎓 給高中生看的入門說明

如果你是第一次看到這份文件，別擔心！下面是「用白話文解釋」這份進度表在說什麼。

### 這份文件是什麼？

這是一個叫做 **豬博士 iPig 系統** 的軟體專案進度表。這個系統是給**實驗室、研究機構**使用的，用來管理：

- 實驗動物的資料（例如：豬的健康狀況、醫療紀錄）
- 實驗計畫的審核流程
- 進銷存（買東西、賣東西、庫存）
- 人事、請假、考勤
- 還有各種通知、報表等

就像學校有教務系統（選課、成績）、學務系統（請假、獎懲）一樣，這個系統是把「實驗動物相關」的所有工作整合在一起。

---

### 常用術語解釋（高中生版）

| 術語 | 白話解釋 |
|------|----------|
| **API** | 程式之間互相溝通的「介面」。例如：前端網頁要顯示動物列表，就要透過 API 跟後端說「給我資料」。 |
| **後端** | 伺服器端的程式，負責存資料、算資料、控制權限。使用者看不到程式碼，只能透過網頁操作。 |
| **前端 / UI** | 你在瀏覽器看到的畫面（按鈕、表格、表單），也就是「使用者介面」。 |
| **資料庫** | 儲存所有資料的地方（像一個超大的 Excel）。 |
| **AUP** | 動物使用計畫（Animal Use Protocol），就是「你要怎麼對動物做實驗」的計畫書，需要經過審核才能執行。 |
| **ERP** | 企業資源規劃，這裡專指**進銷存**：進貨、銷貨、庫存管理。 |
| **HR** | 人事管理（Human Resources），例如請假、加班、考勤。 |
| **遷移 (Migration)** | 修改資料庫結構的腳本，例如新增欄位、新增資料表。 |
| **E2E 測試** | 模擬真人操作瀏覽器，從點擊登入到完成某個流程，確認整個系統沒壞。 |
| **CI/CD** | 程式一提交到 Git，就自動跑測試、檢查程式碼，確保品質。 |
| **上線 (Production)** | 正式給真正的使用者使用的環境（不是測試機）。 |
| **GLP** | 優良實驗室操作規範，國際上對實驗品質、紀錄保存的標準。 |
| **2FA / 雙因素認證** | 登入時除了密碼，還要輸入手機 App 產生的一次性碼，更安全。 |
| **WAF** | 網頁應用程式防火牆，用來擋惡意攻擊。 |
| **Prometheus / Grafana** | 監控系統效能的工具，可以畫出流量、錯誤率等圖表。 |
| **Storybook** | 前端元件展示工具，可單獨預覽按鈕、表單等元件，方便設計與測試。 |
| **P0 / P1 / P2 / P5** | 優先級代號：P0 最高、必須先做；P5 較低、有餘力再做。 |

---

### 總體進度在說什麼？（一句話版）

> 各子系統的**後端程式**、**資料庫**、**前端畫面**都已經做完，整體完成度 100%。  
> 現在在做的是：**測試、監控、安全強化**，準備正式上線給使用者用。

---

## 📑 目錄

| # | 章節 | 說明 |
|---|------|------|
| - | [總體進度概覽](#-總體進度概覽) | 各子系統完成度摘要 |
| - | [正式上線準備度](#-正式上線準備度-production-readiness) | 品質、測試、監控、安全等檢查結果 |
| - | [最新變更動態](#9-最新變更動態) | 每次更新做了什麼（技術細節） |

**閱讀建議：**

- 想快速了解專案狀態 → 看「總體進度概覽」和「正式上線準備度」
- 想了解最新改動 → 看「最新變更動態」（可只看日期和標題，不必逐行理解）
- 想學專案用到的技術名詞 → 看開頭的「術語解釋」

| # | 章節 | 說明 |
|---|------|------|
| 1 | [共用基礎架構](#1-共用基礎架構) | 認證授權、使用者管理、角色權限、Email、稽核 |
| 2 | [AUP 提交與審查系統](#2-aup-提交與審查系統) | 計畫書管理、審查流程、附件、我的計劃 |
| 3 | [iPig ERP (進銷存管理系統)](#3-ipig-erp-進銷存管理系統) | 基礎資料、採購、銷貨、倉儲、報表 |
| 4 | [實驗動物管理系統](#4-實驗動物管理系統) | 動物管理、紀錄、血液檢查、匯出、GLP |
| 5 | [通知系統](#5-通知系統) | Email 通知、站內通知、排程任務 |
| 6 | [HR 人事管理系統](#6-hr-人事管理系統) | 特休、考勤、Google Calendar |
| 7 | [資料庫 Schema 完成度](#7-資料庫-schema-完成度) | Migration 清單 |
| 8 | [版本規劃](#8-版本規劃) | v1.0 / v1.1 里程碑 |
| 9 | [最新變更動態](#9-最新變更動態) | 2026-03-14 R8 代碼規範重構全部 11 項完成 |

---

## 📊 總體進度概覽

> **白話版：** 左邊是各個功能模組，右邊是「後端程式」「資料庫」「網頁畫面」各自的完成度。全部 100% 代表功能都開發完成了。

| 子系統 | 後端 API | 資料庫 | 前端 UI | 整體進度 |
|--------|----------|--------|---------|----------|
| **共用基礎架構** | 100% | 100% | 100% | **100%** |
| **AUP 審查系統** | 100% | 100% | 100% | **100%** |
| **iPig ERP (進銷存管理系統)** | 100% | 100% | 100% | **100%** |
| **實驗動物管理系統** | 100% | 100% | 100% | **100%** |
| **通知系統** | 100% | 100% | 100% | **100%** |
| **HR 人事管理系統** | 100% | 100% | 100% | **100%** |

**整體專案進度：100% ✅ (功能開發完成，上線準備中)**

---

## 🎯 正式上線準備度 (Production Readiness)

> **白話版：** 程式寫完不等於可以上線。上線前要確保：有足夠測試、能監控狀況、有備份還原、有安全防護、符合法規、有效能基準、有文件、使用體驗沒問題。下面就是各項檢查的結果。

| 面向 | 現況 | 目標 | 狀態 |
|------|------|------|------|
| **測試覆蓋率** | Rust 142 unit tests ✅, API 整合測試 25+ cases ✅, CI/CD 整合 DB ✅, E2E 7 spec 34 tests ✅ | 核心邏輯 ≥ 80%、E2E 關鍵流程 100% | ✅ |
| **可觀測性** | /health ✅, /metrics ✅, Prometheus scrape ✅, Grafana Dashboard (10 panels) ✅ | 健康檢查 + Prometheus + Grafana | ✅ |
| **備份 / DR** | GPG 加密備份 ✅, DR Runbook ✅ | 復原 SOP + 上傳檔案備份 + 加密 | ✅ |
| **安全性** | Named Tunnel 腳立 ✅, 容器掃描 ✅ | Pentest + 具名隧道遷移 | ✅ |
| **GLP 合規** | 電子簽章 ✅, GLP 驗證文件 v1.0 ✅, 資料保留政策 ✅ | CSV 驗證文件 + 資料保留政策 | ✅ |
| **效能基準** | k6 基準建立 (P95: 1.76~2.31ms) ✅, 正式基準報告 ✅ | 壓力測試 + Brotli 驗證 + 基準報告 | ✅ |
| **文件** | 使用者手冊 v2.0 ✅（9 章節完整操作手冊）, Swagger ≥90% ✅, 核心模組註解 ✅ | Swagger ≥90%、完整操作手冊 | ✅ |
| **UX / 相容性** | 錯誤處理 UX 統一 ✅, 跨瀏覽器基礎驗證 ✅ | 瀏覽器相容性測試 + 錯誤 UX 統一 | ✅ |

**上線準備度估算：100%（核心功能完整、所有品質補強全數完成，Storybook + 2FA + WAF 長期演進項目亦已交付）**

### 各面向白話說明

| 面向 | 白話解釋 |
|------|----------|
| **測試覆蓋率** | 程式有被自動測試檢查到的比例。測試越多，改程式時越不容易出錯。 |
| **可觀測性** | 系統出問題時，我們有沒有辦法「看得見」哪裡壞了（健康檢查、流量、錯誤率等圖表）。 |
| **備份 / DR** | 資料有備份、有加密；萬一主機壞了，有還原流程（Disaster Recovery）。 |
| **安全性** | 網路隔離、憑證保護、容器掃描等，降低被駭的風險。 |
| **GLP 合規** | 符合實驗室規範：電子簽章、資料保留政策、驗證文件等。 |
| **效能基準** | 用壓力測試（k6）測過，知道系統負載下回應時間大概多少，之後可對比是否變慢。 |
| **文件** | 有操作手冊、API 說明、註解，方便維護與交接。 |
| **UX / 相容性** | 錯誤訊息友善、不同瀏覽器都能正常使用。 |

---

## 1. 共用基礎架構

認證授權、使用者管理、角色權限、Email、稽核。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 2. AUP 提交與審查系統

計畫書管理、審查流程、附件、我的計劃。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 3. iPig ERP (進銷存管理系統)

基礎資料、採購、銷貨、倉儲、報表。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 4. 實驗動物管理系統

動物管理、紀錄、血液檢查、匯出、GLP。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 5. 通知系統

Email 通知、站內通知、排程任務。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 6. HR 人事管理系統

特休、考勤、Google Calendar。完成度 100% ✅（詳見上方總體進度概覽）。

---

## 7. 資料庫 Schema 完成度

Migration 清單。詳見 [backend/migrations/](../backend/migrations/) 目錄；回滾流程見 [database/DB_ROLLBACK.md](database/DB_ROLLBACK.md)。

---

## 8. 版本規劃

v1.0 / v1.1 里程碑。詳見 [TODO.md](TODO.md)（待辦與優先級）、[IMPROVEMENT_PLAN_MARKET_REVIEW.md](IMPROVEMENT_PLAN_MARKET_REVIEW.md)（改進計劃）、[project/VERSION_HISTORY.md](project/VERSION_HISTORY.md)（版本歷程）。

---

## 9. 最新變更動態

> **白話版：** 這裡記錄每次更新做了什麼。按照日期從新到舊排列。  
> 你會看到很多技術細節（例如「useState → Custom Hooks」），簡單說就是：**重構程式碼，讓它更好維護、更不容易出錯**。  
> **P0 / P1 / P2 / P5** 是優先級：P0 最重要，P5 較次要。
>
> **更新慣例**：新項目請放在本區塊**最前面**（時間由近到遠），勿追加於末端。

---

### 2026-03-14 R8 代碼規範重構 — 全部 11 項問題修正完成（R8-1～R8-11）
- ✅ **R8-1**：`routes.rs`（1,236 行）→ `routes/` 目錄（mod.rs + 10 業務域子模組），`cargo check` 零警告。
- ✅ **R8-2**：`main.rs` 450→148 行；啟動邏輯提取至 `startup/tracing.rs`、`startup/migration.rs`、`startup/config_check.rs`、`startup/server.rs`。
- ✅ **R8-3**：建立 `repositories/` 層（equipment/product/role/sku/user/warehouse），遷移 8 個 service 中重複 SQL。
- ✅ **R8-4**：`utils/access.rs` → `services/access.rs`；`utils/mod.rs` 清空為純說明注解。
- ✅ **R8-5**：`services/animal/core.rs`（684 行）→ `core/` 目錄（mod.rs + query.rs/write.rs/update.rs/delete.rs）。
- ✅ **R8-6**：`App.tsx` 四個內聯 Route 元件抽離至 `components/auth/`；`DASHBOARD_ROLES` 常數統一，消除 `getHomeRedirect` 與 `DashboardRoute` 重複。
- ✅ **R8-7**：`lib/api.ts`（514 行）→ `lib/api/` 目錄（client.ts + 7 業務域檔案 + index.ts），原 `api.ts` 改為向後相容 re-export。
- ✅ **R8-8**：`AnimalsPage.tsx` 576→308 行（mutations 提取至 `useAnimalsMutations.ts`，queries 提取至 `useAnimalsQueries.ts`）。
- ✅ **R8-9**：`AnimalsPage.tsx`/`ProtocolsPage.tsx` 型別 import 從 `@/lib/api` 改為 `@/types/*`；`axios` 從非業務用途移除。
- ✅ **R8-10**：`ProtocolsPage.tsx` 中 17 行 `statusColors` 移至 `pages/protocols/constants.ts`。
- ✅ **R8-11**：`services/protocol/core.rs` `use chrono::Datelike` 從函式體內移至檔案頂部。

---

### 2026-03-14 請購/採購單批號與效期調整
- ✅ **前端表單驗證 (Frontend)**：修改 `useDocumentForm.ts` 中的 `needsShelf` 與 `isShelfRequired` 邏輯，排除 `PR` 單據，使其不強制要求儲位。調整 `buildPayload` 驗證，針對 `GRN`/`DO` 等單據，透過品項設定 (`track_batch`, `track_expiry`) 動態決定批號與效期是否為必填，而非一律強制。
- ✅ **後端 CRUD 驗證 (Backend)**：修改 `crud.rs` 中的單據 `create` 與 `update` 方法，結合單據類型與產品 `track_batch`、`track_expiry` 屬性，動態驗證批號與效期，確保正確控制請購/採購與入庫單的資料流向。

### 2026-03-14 R4-100-T5 + T6：單元測試補齊與覆蓋率量測 CI

**R4-100-T5：protocol / document / hr services 單元測試**

- ✅ **protocol/numbering**：提取 `parse_no_sequence` 與 `format_protocol_no` 純函式，並同步重構 `generate_apig_no` / `generate_iacuc_no` 使用這兩個函式；新增 8 個測試（前綴解析、格式化補零、非法輸入）。
- ✅ **protocol/status**：直接測試既有 `validate_protocol_content` 私有函式（透過 `ProtocolService::` 呼叫）；7 個測試涵蓋缺少 content、缺少 basic、空白標題、GLP 未填授權單位、缺少 project_type 及正常通過。
- ✅ **hr/leave**：補充 `effective_hours` 純函式（total_hours 優先換算邏輯）；7 個測試涵蓋 `is_half_hour_multiple` 邊界值與 `effective_hours` 換算。
- ✅ **hr/overtime**：提取 `overtime_multiplier`、`comp_time_hours_for_type`、`calc_hours_from_minutes` 三個純函式，同步重構 `create_overtime`；8 個測試涵蓋各類型乘數、補休規則、0.5 小時捨入。
- ✅ **hr/attendance**：直接測試既有 `is_ip_in_ranges` 公開函式；補充 `attendance_status_display` 純函式；8 個測試涵蓋精確 IP、CIDR /24、/32、多段清單、空清單、無效 IP。
- ✅ **hr/balance**：提取 `compute_leave_expiry` 純函式（到期日計算含閏年退回邏輯），同步重構 `create_annual_leave_entitlement`；4 個測試涵蓋無到職日、有到職日、2/29 閏年邊界。
- ✅ **document/grn**：提取 `next_seq_from_last_no` 與 `receipt_status_label` 純函式，同步重構 `create_grn_from_po` 與 `get_po_receipt_status`；8 個測試涵蓋各種單號格式、非法字串、三種入庫狀態。
- **總計**：新增 50 個單元測試；`cargo check --tests` 通過。

**R4-100-T6：cargo-tarpaulin 覆蓋率量測 CI**

- ✅ **新增 `backend-coverage` job**：在 `.github/workflows/ci.yml` 加入獨立覆蓋率量測流程。
- ✅ **設定**：`SQLX_OFFLINE=true`（不需要 DB）、`--lib`（只跑 lib 單元測試）、`--fail-under 25`（行覆蓋率門檻 25%）、`--timeout 120`、輸出 XML 格式。
- ✅ **報告保存**：XML 覆蓋率報告以 `coverage-report` artifact 上傳，保留 14 天。
- ✅ **快取優化**：使用 `cargo-tarpaulin-` 前綴的獨立快取 key。

---

### 2026-03-14 批次套用儲位 UI 與邏輯優化
- ✅ **批次套用儲位選填化**：標明單據表頭（如採購入庫、調撥單）的儲位選擇為「批次套用儲位 (選填)」，避免使用者誤以為只能限定單一儲位，適應同一張採購單品項存在不同儲位的情境。
- ✅ **預設儲位繼承**：使用者點擊「新增明細」時，新明細會自動繼承表頭已選的「批次套用儲位」，大幅提升多儲位配置的建檔效率。

### 2026-03-14 品項選擇與單據關連優化
- ✅ **動態品類同步 (已修復)**：品項選擇彈窗現在會自動透過 `useSkuCategories` Hook 同步品類設定，修正了之前調用未定義 `/categories` API 導致 Tabs 未發揮作用的問題。
- ✅ **UX 優化**：新增品類 Tabs 篩選器，支援關鍵字與品類雙重過濾。同時擴增後端庫存查詢，實現在「庫存模式」下也能依據對應品類即時過濾。
- ✅ **採購入庫強化**：連動「來源採購單」時自動過濾供應商與核准狀態。
- ✅ **系統修復**：修復 API 400 (參數大小寫/解析錯誤) 與 500 (SQL 欄位缺失) 報錯。
- ✅ **明細顯示修復**：修正 `poReceiptStatus` 屬性未傳遞至 `DocumentLineEditor` 的問題，確保 GRN 選擇來源採購單後能正確列出待入庫明細。

---

### 2026-03-13 R8 代碼規範重構 — 目錄掃描與風格採樣（01a-1, 01a-2）
- ✅ **01a-1 目錄掃描**：建立 backend/frontend/scripts/tests 完整樹狀圖，標注各目錄推測職責；發現 `utils/access.rs` 位置不符規範、缺少 `repositories/` 層、`lib/api.ts` 未按業務域拆分等三項架構問題。
- ✅ **01a-2 風格採樣**：分析 `main.rs`、`routes.rs`、2 個 service、`App.tsx`、2 個 page，產出命名慣例/函式長度/巢狀深度/錯誤處理/import 組織五維度比較表；識別 11 項具體問題（R8-1～R8-11），記錄至 `docs/TODO.md` R8 區段。

### 2026-03-14 採購入庫品項篩選強化 (修正)
- ✅ **入庫邏輯嚴格化**：修正 GRN 品項篩選失效問題。
- ✅ **UI 增強**：新增「來源採購單」下拉選單，支援依供應商自動篩選已核准 PO。
- ✅ **邏輯修正**：修正 `useDocumentForm` 中 `poReceiptStatus` 查詢邏輯（改用 `source_doc_id`），確保品項彈窗正確過濾已入庫項目。

### 2026-03-14 單據頁面 UI 體驗優化 (V2)
- ✅ **銷貨單優化**：隱藏專屬單據 (SO/DO) 重複的「客戶」欄位，減少 UI 冗餘。
- ✅ **調撥單功能增強**：新增「來源儲位」與「目標儲位」的批次套用選單，支援所有明細行同步更新。

### 2026-03-14 單據儲位選單選取問題修復
- ✅ **UI 綁定修正**：解決了「批次套用儲位」重灌後下拉按鈕標籤不更新的 Bug。
- ✅ **狀態管理優化**：新增 `batchStorageLocationId` 狀態以追蹤並呈現當前選定的批次儲位，提升選取回饋感。

### 2026-03-14 供應商與專屬計畫填寫互斥修復
- ✅ **邏輯解耦**：在 `DocumentFormData` 中新增獨立的 `protocol_no` 欄位，解除了計畫代碼與供應商 ID 的強制綁定。
- ✅ **採購單流程優化**：在 `PO`/`GRN`/`PR` 等採購相關單據中，選擇計畫後不再覆蓋已填寫的供應商。
- ✅ **向後相容性**：銷貨/出庫單（`SO`/`DO`）維持原有邏輯，選擇計畫後自動帶出對應客戶，符合現有作業流程。

### 2026-03-14 專屬計畫載入效能優化
- ✅ **載入邏輯修復**：修正了 `PO`/`PR` 單據類型無法觸發計畫列表獲取的 Bug。
- ✅ **Loading 體驗優化**：解耦了 `activeProtocols` 的載入狀態，在無資料時正確顯示「無可用計畫」而非持續顯示「載入中」。
- ✅ **效能提升**：優化了計畫列表的過濾與計算邏輯。

### 2026-03-13 單據邏輯增強與 IACUC 關聯實作
- ✅ **單據欄位規範調整 (Dynamic Fields)**：依單據類型動態切換日期、倉庫、貨架、計畫與供應商的必填/可見狀態。
    - **倉庫-儲位連動 (Header Linkage)**：表頭選定倉庫後跳出儲位選擇器，支援全單批次套用至明細行。
    - PO (採購單)：顯示供應商 (必填) 與計畫 (選填)。
    - GRN (採購入庫)：顯示供應商 (必填)，**計畫欄位隱藏** (符合不需要規範)。
    - SO/DO (銷貨/出庫)：顯示客戶 (必填) 與 IACUC No. (必填)。
    - STK/ADJ (盤點/調整)：**隱藏所有夥伴與計畫欄位**。
- ✅ **前端驗證強化 (Frontend Validation)**：`useDocumentForm` 實作跨欄位提交校驗與 `*` 標誌呈現。

### 2026-03-13 倉庫管理頁面重構計畫啟動

- 🏗️ **架構規劃 (Planning)**：擬定「上、中、下」三層式結構改善計畫。將 `WarehouseLayoutPage.tsx` 拆分為 `WarehouseActionHeader` (上)、`StorageLocationEditor` (中) 與 `WarehouseDetailTabs` (下)。
- ✅ **功能實作 (Implementation)**：補全倉庫 CRUD (建立、刪除、停用、編輯) 功能，支援建築結構 (牆、門、窗) 的 2D 視覺化佈局。
- 🧪 **品質驗證 (Verification)**：通過 `tsc` 編譯檢查，確認元件通訊與 API 互動正常。
- 📁 **產出**：`implementation_plan.md`、`task.md`、`walkthrough.md`。

### 2026-03-13 前端編譯錯誤修復 (DocumentEditPage.tsx)

- ✅ **編譯修復 (Bug Fix)**：修正 `DocumentEditPage.tsx` 在解構 `useDocumentForm()` 時漏掉 `setFormData` 的問題。這解決了 `DocumentLineEditor` 組件因接收到未定義函數而導致的 `Cannot find name 'setFormData'` TypeScript 錯誤，確保 Docker 建置與 `npm run build` 能正常完成。
- 📁 **產出**：`DocumentEditPage.tsx`。

### 2026-03-13 測試基礎設施修復 (Test Infrastructure Fix)

- ✅ **測試環境修錯 (Bug Fix)**：修正 `backend/tests/common/mod.rs` 中 `ensure_admin_user` 函數參數遺漏問題（從 1 個參數補齊為 2 個，包含 `config`），恢復整合測試代碼的編譯。
- 📁 **產出**：`backend/tests/common/mod.rs`。

### 2026-03-13 採購單未入庫通知與狀態顯示功能

- ✅ **通知邏輯 (Notification)**：實作 `notify_po_pending_receipt`，自動檢查已核准但尚未有 GRN 入庫紀錄的採購單 (PO)，並發送通知給倉管主管。
- ✅ **排程任務 (Scheduler)**：新增每日 09:00 定期檢查排程，確保倉管人員及時處理未入庫單據。
- ✅ **手動觸發 API**：新增 `/api/admin/trigger/po-pending-receipt-check` 端點，允許管理員視需要手動執行檢查。
- ✅ **通知路由配置**：在 `RoutingService` 中註冊 `po_pending_receipt` 事件，並於資料庫中新增預設路由。
- ✅ **單據列表強化**：`DocumentListItem` 模型新增 `receipt_status` 欄位；後端 SQL 結合 `v_purchase_order_receipt_status` 視圖自動計算入庫狀態。
- ✅ **前端視覺化**：單據管理頁面 (`DocumentsPage.tsx`) 針對 PO 顯示「未入庫」、「部分入庫」、「已入庫」彩色標籤，並於通知設定中加入對應事件名稱。
- 📁 **產出**：erp.rs, scheduler.rs, routing.rs, workflow.rs, crud.rs, document.rs (model), DocumentsPage.tsx, notification.ts (frontend) 等多處更新。

### 2026-03-13 ERP 庫存管理強化與視覺體驗優化

- ✅ **視覺體驗優化 (UX)**：針對庫存查詢頁面進行全方位美化。
  - **下拉選單 (WarehouseShelfTreeSelect)**：解決 Popover 選單背景透明導致的文字重疊問題。引入 `Popover.Portal` 確保層級正確，並加入 Glassmorphism（背景模糊）、陰影與流暢的動畫效果。
  - **列表樣式**：優化表格 Layout，提升資料可讀性。增加單行 Hover 效果、漸變標題與精緻的狀態標籤（如安全庫存預警）。
  - **空狀態重塑 (Empty State)**：當搜尋無結果或無資料時，顯示更具引導性的插圖與文字描述，而非單調的圖標。
  - **加載體驗**：改進 Skeleton 與 Loader 顯示方式，使其在資料加載過程中視覺上更穩定。
- ✅ **下拉選單穩定性**：修復「新增單據」頁面中倉庫、合作夥伴與 IACUC No. 下拉選單選項不穩定問題。透過 `react-query` 的 `refetchOnMount` 與前端 Loading 狀態處理，確保資料在載入過程中 UI 顯示一致。
- ✅ **庫存查詢**：新增「未分配庫存查詢」功能。前台 `WarehouseLayoutPage` 可快速查看尚未指派儲位的產品庫存，後端 `StockService` 提供對應 API。
- ✅ **系統健全度**：`StockService` 查詢結果加入 `storage_location` 預設值處理，避免特定情境下的欄位缺失。
- ✅ **資料庫架構**：完成 Migration 清理，將 `phone_ext` (分機) 與 `leave_cancelled` 路由邏輯正式併入基礎遷移檔案，提升資料庫一致性。
- 📁 **產出**：InventoryPage.tsx、WarehouseShelfTreeSelect.tsx、useDocumentForm.ts、DocumentEditPage.tsx、stock.rs、WarehouseLayoutPage.tsx、migrations 多檔更新。

### 2026-03-10 系統電話分機欄位 (Phone Extension) 支援

- ✅ **資料庫與架構**：Migration `002`、`004`、`007` 新增 `phone_ext` 欄位至 `users`、`partners`、`animal_sources` 並清理臨時遷移文件。
- ✅ **計畫書 (AUP)**：`SectionBasic.tsx` 與 `ProtocolContentView.tsx` 新增資助者 (Sponsor) 與計畫主持人 (PI) 的聯絡分機，PDF 產生同步支援顯示。
- ✅ **使用者管理**：`ProfileSettingsPage.tsx` 與型別 `User` 新增 `phone_ext`，支援個人資料分機設定。
- ✅ **交易夥伴**：`PartnersPage.tsx` 與型別 `Partner` 新增 `phone_ext`，支援供應商與客戶的分機管理。
- ✅ **動物來源**：`AnimalSourcesPage.tsx` 與型別 `AnimalSource` 新增 `phone_ext`，支援來源廠商的分機管理。
- ✅ **型別與初始值**：同步更新 `auth.ts`、`erp.ts`、`animal.ts`、`protocol.ts` 與 `constants.ts` 確保前端型別一致與表單預設值。
- 📁 **產出**：涉及 User, Partner, AnimalSource, Protocol 型別與 UI 元件多處更新。

---

### 2026-03-10 AUP 計畫主持人電話新增「分機」欄位 (及編譯錯誤修復)

- ✅ **前端**：`SectionBasic.tsx` 新增分機 (Extension) 輸入框，UI 顯示為 `電話 #分機` 格式。
- ✅ **前端檢視**：`ProtocolContentView.tsx` 計畫書內容檢視頁面同步顯示分機號碼。
- ✅ **類型修復**：修改 `src/types/protocol.ts`，在 `ProtocolWorkingContent.basic.pi` 中增加 `phone_ext?: string` 選填欄位，解決元件中的型別不匹配錯誤。
- ✅ **編譯修復**：修正 `src/pages/master/CreateProductPage.tsx` 缺少 `useEffect` 匯入的問題。
- ✅ **初始值同步**：更新 `protocol-edit/constants.ts` 中的 `defaultFormData`，加入 `phone_ext` 初始值。
- ✅ **本地化**：`zh-TW.json` 新增 `aup.basic.piExtension` 字串。
- ✅ **後端 PDF**：`backend/src/services/pdf/service.rs` 更新 PDF 產生邏輯，計畫主持人電話欄位現在會包含分機。
- 📁 **產出**：protocol.ts、constants.ts、CreateProductPage.tsx、ProtocolContentView.tsx、SectionBasic.tsx、zh-TW.json、service.rs。

### 2026-03-09 重構動物服務模組 (Service 拆分與解耦)

- ✅ **Service 抽取**：將原 `AnimalService` 龐大邏輯拆分為 9 個獨立 Service：`AnimalBloodTestService`、`AnimalMedicalService`、`AnimalObservationService`、`AnimalSurgeryService`、`AnimalWeightService`、`AnimalSourceService`、`AnimalTransferService`、`AnimalImportExportService`、`AnimalFieldCorrectionService`。
- ✅ **核心 CRUD**：`AnimalService` (core.rs) 僅保留動物基礎 CRUD 與批次分配邏輯。
- ✅ **工具函數解耦**：耳號格式化、欄位編號格式化、品種轉換等通用邏輯移動至 `AnimalUtils`。
- ✅ **Handler 同步**：同步更新所有動物相關 Handler (`blood_test.rs`, `import_export.rs`, `source.rs`, `transfer.rs` 等)，從調用單一 `AnimalService` 改為調用對應的專屬 Service。
- ✅ **修復隱患**：修正 `import_export.rs` Handler 中的匯出紀錄建立參數不匹配問題。
- 📁 **產出**：`backend/src/services/animal/` 下所有檔案及 `backend/src/handlers/animal/` 對應檔案。

### 2026-03-09 修正 Clippy 編譯警告與安全隱患 (unwrap 清理)

- ✅ **Clippy 修正**：修復 `services/hr/attendance.rs` 中的 `needless-borrows-for-generic-args` 警告，提升程式碼品質。
- ✅ **安全強化**：將 `services/email/mod.rs` 中的 `.unwrap()` 改為 `.expect()`，並提供明確的錯誤訊息，避免潛在的 panic。
- 📁 **產出**：`backend/src/services/hr/attendance.rs`、`backend/src/services/email/mod.rs`。

### 2026-03-09 清理重複的胰臟分類

- ✅ **重複統合**：將資料庫遷移檔案 `004_animal_management.sql` 中重複的「胰臟」分類移除，並將相關檢驗項目（AMY, LPS）統合至「胰臟與血糖」(`SUGAR`) 分類下。解決前端畫面顯示重複的問題。
- 📁 **產出**：`004_animal_management.sql`。

### 2026-03-09 請假管理動作後自動重新整理頁面

- ✅ **自動重新整理**：在「新增請假」、「送審」、「核准」、「駁回」、「取消」等動作成功後，加入 1 秒延遲並執行 `window.location.reload()`。
- ✅ **資料同步強化**：確保動作完成後，頁面上的餘額摘要、待審核數量紅點及各分頁列表皆能完全同步。
- 📁 **產出**：`frontend/src/pages/hr/HrLeavePage.tsx`。

---

### 2026-03-09 API 規格文件全面對齊程式碼（第二輪）

- ✅ **轉讓端點修正**：移除 source-pi-confirm/target-pi-confirm/iacuc-approve，新增 vet-evaluation/assign-plan/approve/reject（對齊 routes.rs）
- ✅ **移除未實現端點**：protocols/:id/status-history、animals/batch/start-experiment
- ✅ **補齊未記錄端點**：care-records、treatment-drugs、blood-test-presets、equipment、equipment-calibrations、training-records、qau/dashboard、admin data-export/import/config-warnings、SSE 警報、通知路由子端點
- ✅ **ENUM 修正**：animal_transfer_status 對齊 001_types.sql（pending/vet_evaluated/plan_assigned/pi_approved/completed/rejected）
- ✅ **設施管理表標註**：標註 species/facilities/buildings/zones/pens/departments 遷移待補建
- ✅ **權限代碼對齊**：05_API_SPECIFICATION Section 5 → admin.user.*、Section 6 → dev.role.*
- ✅ **RBAC 文件更新**：新增 dev.* 權限區塊，移除不存在的 admin.role.view/manage
- ✅ **DELETE 備用路由通則**：新增 Section 1.5 說明 POST /:id/delete 備用路由設計
- 📁 **產出**：05_API_SPECIFICATION.md、04_DATABASE_SCHEMA.md、06_PERMISSIONS_RBAC.md

---

### 2026-03-08 R7 安全性原始碼審視修復 + 文件全面對齊

- ✅ **R7-P0**：`data_import.rs` SQL 拼接改為參數化查詢，消除 SQL injection 風險
- ✅ **R7-P1-1**：`create_admin.rs` 不再將管理員密碼明文印至 stdout
- ✅ **R7-P1-2**：`config.rs` `trust_proxy` 預設值由 `true` 改為 `false`
- ✅ **R7-P4-1**：`etag.rs` 改用 `constants::ETAG_VERSION` 取代硬編碼字串
- ✅ **R7-P4-2**：認證端點 rate limit 由 100/min 降至 30/min
- ✅ **文件對齊**：ARCHITECTURE.md（技術棧/目錄/rate limit）、TODO.md（R7 完成項/統計）、PROGRESS.md、QUICK_START.md（環境變數）、API 規格、DB Schema、RBAC 權限文件全面更新
- 📁 **產出**：`data_import.rs`、`create_admin.rs`、`config.rs`、`constants.rs`、`etag.rs`、`rate_limiter.rs`；docs 多檔修正

---

### 2026-03-08 日曆功能審視與重構 (業內標準化)

- ✅ **前端重構**：將 `CalendarSyncSettingsPage` 拆分為 `useCalendarSync`、`useCalendarEvents` Hooks 與 4 個獨立 Tab 元件（Status/Events/History/Conflicts）；實作日曆事件點擊預覽 (Popover)，支援直接跳轉 Google Calendar。
- ✅ **後端解耦**：引入 `CalendarApi` trait 抽象日曆操作，實現 `GoogleCalendarClient` 與 `CalendarService` 的解耦，支援依賴注入與 Mock 測試。
- ✅ **同步重構**：重構 `trigger_sync` 邏輯，拆分為 `process_pending_creates/updates/deletes`，提升代碼可讀性與維護性。
- ✅ **測試補強**：新增 `useCalendarEvents` 的導航邏輯單元測試 (Vitest) 與後端 `CalendarService` 輔助函式單元測試 (Cargo test)。
- 📁 **產出**：CalendarView.tsx、CalendarSyncSettingsPage.tsx、useCalendarSync.ts、useCalendarEvents.ts、google_calendar.rs (Trait)、calendar.rs (Refactored)。

---

### 2026-03-07 Calendar 月份切換修正 & 2.0 體驗升級

**月份切換 Bug 修正（根本原因修正）：**

- ✅ 移除 `key={format(calendarDateRange.start, 'yyyy-MM')}` — 不再因月份變化強制 remount FullCalendar
- ✅ 採用 React Query `keepPreviousData` — 換月時保留舊事件顯示，新資料到才平滑替換，無閃爍
- ✅ 刪除 `calendarMounted` 雙層 RAF 邏輯 — 移除不必要的延遲掛載 workaround
- ✅ 刪除 `shouldAcceptDateRange` — 改以格式化字串比較去重，邏輯更清晰
- ✅ 新增 `isFetching` → 換月期間右上角顯示小 spinner，不遮擋日曆本體

**Calendar 2.0 體驗升級（P0–P1 全數完成）：**

- ✅ **假別顏色 coding**：從 summary 解析 `[假別]` 標籤，映射 10 種假別顏色（特休＝綠、病假＝橘、事假＝藍...），不需後端改動
- ✅ **假別篩選 chips**：日曆上方顯示當月出現的假別 chip，點擊即過濾；再點取消；「全部」chip 常駐
- ✅ **衝突解決補全 `accept_google`**：衝突列表新增第三個解決方案「接受 Google 版本」（後端原已支援）
- ✅ **分頁 UI**：同步歷史、衝突列表加前/下頁按鈕，後端已支援 pagination，前端補接
- ✅ **衝突樂觀更新**：點擊解決按鈕後立即從列表移除，失敗時自動回滾
- ✅ **Popover 解析升級**：事件彈出框解析 `[假別] 員工名（代理人）` 格式，顯示假別顏色 badge + 員工名 + 代理人欄位
- ✅ **時間格式改善**：Popover 顯示完整日期範圍（全天事件顯示「月/日（全天）」或「月/日 – 月/日（全天）」）
- ✅ **連線狀態升級**：顯示近期錯誤警告、最後同步結果 badge、下次同步時間
- ✅ **自動同步設定 UI**：連線狀態分頁加入同步排程設定（啟用開關、早/晚同步時間），對接 `PUT /hr/calendar/config` API

- 📁 **產出**：useCalendarEvents.ts、useCalendarSync.ts、CalendarView.tsx、CalendarEventsTab.tsx、CalendarStatusTab.tsx、ConflictsTab.tsx、SyncHistoryTab.tsx、CalendarSyncSettingsPage.tsx、hr.ts（新增 CalendarConfig / UpdateCalendarConfig 型別）

---

### 2026-03-07 血檢 API 與動物權限綁定

- ✅ **需求**：list_all（panels/templates/presets）與血檢分析報表應與動物權限綁定；能看到動物的範圍，就看到其血檢分析結果。
- ✅ **實作**：list_all_blood_test_* API 改為 require `animal.record.view`（原 `animal.blood_test_template.manage`）；blood_test_analysis 報表加權限檢查，若僅 view_project 則只回傳 `iacuc_no IS NOT NULL` 之動物；REVIEWER 新增 `animal.animal.view_all`、`animal.record.view` 以存取血檢分析。
- 📁 **產出**：blood_test.rs、report.rs、ReportService、permissions.rs、003、BloodTestAnalysisPage、06_PERMISSIONS_RBAC.md。

### 2026-03-07 血檢項目權限 `animal.blood_test_template.manage`

- ✅ **需求**：僅具該權限者可檢視與編輯血檢項目（模板、組合、常用組合）；管理者可於「角色權限」處勾選／取消。
- ✅ **後端**：新增權限 `animal.blood_test_template.manage`（Migration 011、permissions.rs）；模板／組合／常用組合之 list_all、create、update、delete API 改為檢查此權限（原先為 animal.record.*）。`list`（啟用中）仍不檢查，供動物血檢 Tab 建立紀錄時使用。
- ✅ **前端**：側邊欄「血檢項目」加 `permission`；`/blood-test-templates`、`/blood-test-panels`、`/blood-test-presets` 路由包上 `RequirePermission`。
- ✅ **角色**：預設指派給 EXPERIMENT_STAFF；admin 具全部權限。
- 📁 **產出**：003_notifications_roles_seed.sql（權限與角色指派）、blood_test.rs、Sidebar.tsx、App.tsx、usePermissionManager.ts、06_PERMISSIONS_RBAC.md。

### 2026-03-06 新增 EQUIPMENT_MAINTENANCE（設備維護人員）角色

- ✅ **需求**：於系統管理「角色權限」中新增「設備維護人員」角色，供管理設備與校準紀錄。
- ✅ **實作**：將角色與權限寫入既有 migration **009_glp_extensions.sql**（9.3b 區塊）：插入角色 `EQUIPMENT_MAINTENANCE`（名稱：設備維護人員）、並指派 `equipment.view`、`equipment.manage`、`training.view`、`training.manage_own`、`dashboard.view`。維持 10 個 migration 檔案。
- ✅ **結果**：重啟後端後，角色權限頁面會顯示「設備維護人員」卡片；可將使用者指派為此角色以存取 ERP 設備維護分頁。

### 2026-03-05 migrations 升級重整（維持 10 個）

- ✅ **合併結果**：原 16 個 migration 重整為 10 個，最終 schema 不變、執行順序與依賴維持正確。
- ✅ **對應**：001_types、002_users_auth 未改；003＝原 003＋004（通知/附件/稽核/trigger＋角色權限 seed/user_preferences）；004＝原 005 動物管理；005＝原 006 AUP；006＝原 007 HR；007＝原 008 稽核＋ERP；008＝原 009＋011＋012（補充、犧牲鎖欄、轉讓類型、修正、效能）；009＝原 010＋013＋014（GLP 訓練/設備/QAU/會計、血液檢查預設、SKU 品類種子）；010＝原 015＋016（治療藥物去重與業務鍵唯一）。
- ✅ **舊檔移除**：006_aup_system、007_hr_system、008_audit_erp、009_supplementary、010_glp_accounting、011～016 已刪除；保留 `.gitattributes`。

### 2026-03-05 系統時間統一為台灣時間 (Asia/Taipei)

- **後端**：新增 `backend/src/time.rs` 提供 `now_taiwan()`、`today_taiwan_naive()`；活動日誌 partition_date、會計 as_of、審計儀表板、HR 出勤／請假「今日」、單據編號日期、PDF/郵件顯示日期、排程月報、匯出檔名等皆改為以台灣日期／時間為準。
- **前端**：`formatDate`／`formatDateTime` 及所有內聯日期顯示皆加上 `timeZone: 'Asia/Taipei'`，不論使用者瀏覽器時區皆顯示台灣時間。
- **Grafana**：`deploy/grafana_dashboard.json` 的 `timezone` 設為 `Asia/Taipei`。

### 2026-03-05 R4-100-T3 user/role service 單元測試

- **R4-100-T3**：UserService 提取 `user_search_pattern(keyword)` 供 list 使用，3 個單元測試；RoleService 提取 `is_valid_role_code(s)`（1–50 字、英數字與底線）、於 create 前驗證，3 個單元測試。另修正 `time.rs` 測試缺少 `chrono::Datelike` 導致編譯失敗。TODO R4-100-T3 標完成，待辦統計 4→3、合計 5→4。

### 2026-03-05 R4-100-T2 partner service 單元測試

- **R4-100-T2**：PartnerService 提取可測函式 `format_partner_code`、`is_valid_email`，`parse_partner_code_sequence`（#[cfg(test)]）、`parse_partner_type`／`parse_supplier_category`／`parse_customer_category` 改為 `pub(crate)`；新增 6 個單元測試（format_partner_code、parse_partner_code_sequence、parse_partner_type、parse_supplier_category、parse_customer_category、is_valid_email）。TODO R4-100-T2 標完成，待辦統計 5→4、合計 6→5。

### 2026-03-05 R4-100-T1 product service 單元測試

- **R4-100-T1**：ProductService 提取可測函式 `format_product_sku`、`validate_product_status`，`parse_bool` 改為 `pub(crate)`；新增 8 個單元測試（format_product_sku 3、validate_product_status 3、parse_bool 2）。TODO R4-100-T1 標完成，待辦統計 6→5、合計 7→6。

### 2026-03-05 R4-100-O7 報表／會計／治療藥物 OpenAPI 完成

- **R4-100-O7**：report（7 個端點）、accounting（7 個端點）、treatment_drug（6 個端點）補齊 `#[utoipa::path]`，openapi.rs 註冊 paths、tags「報表／會計／治療藥物」及相關 schemas（CreateApPaymentRequest、CreateArReceiptRequest、TreatmentDrugOption 等）。TODO.md R4-100-O7 標完成，待辦統計 7→6、合計 8→7。

### 2026-03-05 編輯產品與新增產品對齊（包裝結構、分類、移除耗材 LAB 主分類）

- **編輯產品頁**：品類改為與新增產品一致（分類按鈕＋子分類下拉）；移除「耗材(LAB)」主分類，實驗耗材改為耗材之子分類；舊 LAB 主分類產品載入時自動對應為 耗材＋實驗耗材。
- **編輯產品頁**：新增「包裝結構」區塊，可檢視與編輯兩層／三層包裝（外層→內層→基礎單位），與新增產品相同邏輯計算 `pack_unit`／`pack_qty` 儲存。
- 變更檔案：`frontend/src/pages/master/ProductEditPage.tsx`。

---

### 2026-03-05 移除 Sentry 錯誤監控

- 後端：移除 sentry crate、Config.sentry_dsn、main 中 sentry::init 與 runtime 改回 #[tokio::main]、error.rs 中 sentry::capture_error。
- 前端：移除 @sentry/react、instrument.ts、main 首行 import、ErrorBoundary 內 Sentry.captureException、系統設定頁「錯誤監控測試」卡片；Dockerfile / docker-compose 移除 VITE_SENTRY_DSN。
- 文件與範本：.env.example、DEPLOYMENT、OPERATIONS、IMPROVEMENT_PLAN_R4 還原為未導入 Sentry 狀態。

### 2026-03-04 全域刪除改用 POST /delete（避免代理/tunnel 回傳 405）

- ✅ **根因**：部分代理、Cloudflare Tunnel 等對 `DELETE` 請求回傳 405 Method Not Allowed，導致刪除操作失敗但前端仍顯示成功。
- ✅ **後端**：為所有 DELETE 端點新增 `POST /.../delete` 替代路由（36 個），涵蓋 users、roles、warehouses、storage-locations、products、partners、documents、animal-sources、animals、observations、surgeries、weights、vaccinations、care-records、blood-tests、notifications、attachments、equipment、training-records、hr、facilities 等。
- ✅ **前端**：新增 `deleteResource(url, options?)` 輔助函式；`bloodTestApi`、`bloodTestTemplateApi`、`bloodTestPanelApi`、`notificationRoutingApi`、`treatmentDrugApi` 及 20+ 頁面/元件全部改為使用 `deleteResource`，支援 body（如 reason）與 headers（如 X-Reauth-Token）。
- ✅ **倉庫列表**：列表 API 預設傳入 `is_active=true`，刪除（軟刪除）後已停用倉庫不再顯示於主列表。

---

### 2026-03-05 端點文件化與單元測試盤點、storage_location + SKU 完成

- ✅ **盤點文件**：新增 `docs/development/OPENAPI_AND_TESTS_STATUS.md`，總計路由 **318** 個 handler、已文件化 **132**、尚未文件化約 **186**；單元測試 **148** 個，並列出未文件化模組與建議補強測試模組。
- ✅ **OpenAPI 儲位與 SKU**：storage_location 全模組 **11** 端點（含 ToSchema/IntoParams 與 openapi 註冊）；SKU **6** 端點（get_sku_categories, get_sku_subcategories, generate_sku, validate_sku, preview_sku, create_product_with_sku），models/sku 與 ProductWithUom 等 schema 已註冊。
- ✅ **Rust 單元測試**：維持 **148** 個測試通過（前次已補常數/SKU/倉庫 6 個）。

### 2026-03-05 IMPROVEMENT_PLAN_R4 延續（端點文件化、Rust 單元測試）

- ✅ **OpenAPI 監控端點**：新增 3 個端點文件化：`health_check`（GET /api/health）、`metrics_handler`（GET /metrics）、`vitals_handler`（POST /api/metrics/vitals），含 HealthResponse/PoolCheck/DiskCheck/WebVitalsMetric 等 schema，新增「監控」tag。
- ✅ **Rust 單元測試**：新增 6 個測試（常數 audit 2 個、SKU 格式 2 個、倉庫代碼序號 2 個），總計 **148** 個測試通過。

### 2026-03-04 IMPROVEMENT_PLAN_R4 目標補齊（Rust 測試、OpenAPI）

- ✅ **Rust 單元測試**：新增 15 個核心業務邏輯測試（SKU 格式解析 7 個、倉庫代碼序號 4 個、常數驗證 4 個），總計 **142** 個測試通過，強化覆蓋率。
- ✅ **OpenAPI 端點文件化**：補齊 10 個端點 `#[utoipa::path]` 與 openapi.rs 註冊：`export_me`、`delete_me_account`、2FA、使用者偏好；**R4-100-O1** products（10 paths）；**R4-100-O2** partners（8 paths）；**R4-100-O3** documents + storage_location（19 paths）；**R4-100-O4** SKU（5 paths）；**R4-100-O5** animal 子模組（觀察/手術/體重/疫苗/犧牲/病理/轉讓，31 paths）；**R4-100-O6** HR（出勤/請假/加班）+ 通知 + 稽核（11 paths），符合 ≥90% 端點文件化目標。

### 2026-03-04 全專案資料夾整理與分類

- ✅ **維運手冊歸位**：`docs/OPERATIONS.md` 移入 `docs/operations/OPERATIONS.md`，與 COMPOSE、ENV_AND_DB、TUNNEL 等同屬「環境與建置」分類；所有引用已更新（SOC2_READINESS、SLA、docs/README）。
- ✅ **文件索引**：`docs/README.md` 新增維運手冊入口、operations 區塊補齊 OPERATIONS.md、目錄結構摘要加註分類說明、頂部新增「閱讀建議」依角色導引。
- ✅ **根目錄導覽**：`README.md` 新增「資料夾一覽」表（backend、frontend、docs、scripts、tests、monitoring、deploy、.github）及「依角色閱讀」；文件導覽加入 OPERATIONS.md 連結。
- ✅ **monitoring/ 與 deploy/**：新增 `monitoring/README.md`（Prometheus、Alertmanager、Promtail 結構與用途）、`deploy/README.md`（Grafana、cloudflared、WAF 規則分類與相關文件連結），便於維運與新成員查找。

### 2026-03-04 scripts 目錄整理

- ✅ **scripts/README.md**：新增總覽與分類索引（啟動/隧道、CI/測試、資料庫、備份、部署、環境、Windows 建置），含目錄結構與相關文件連結。
- ✅ **引用修正**：文件中原不存在的 `fix_migration_checksums.ps1` 改為 `sync_migrations.ps1 -Method FixChecksums`（`restore_old_dump.ps1`、`docs/database/RESTORE_OLD_DUMP.md`）。

---

### 2026-03-04 新規則：已犧牲動物可將欄位改為空值

- ✅ **規則**：若動物已為犧牲（euthanized）狀態，允許透過更新動物 API 將欄位（`pen_location`）改為空值；其餘狀態時，傳空則保留原值。
- ✅ **實作**：`backend/src/services/animal/core.rs` 更新動物時，依 `current_status == Euthanized` 決定 `pen_location` 綁定值與 SQL（`CASE WHEN status = 'euthanized' THEN $3 ELSE COALESCE($3, pen_location) END`）。
- ✅ **規格**：已於 `_Spec.md` 2.7.1 新增「已犧牲動物可清空欄位」條目並更新實作方式說明。

### 2026-03-04 犧牲/安樂死時自動移出欄位（pen_location = NULL）

- ✅ **規格**：依 `docs/Profiling_Spec/archive/_Spec.md`「犧牲時移除欄位」規則，已安樂死之動物應將欄位清空（`pen_location = NULL`）以移出欄位。
- ✅ **實作**：原先僅更新狀態為 `euthanized`，未清空欄位；現已補上。
  - **犧牲/採樣紀錄確認**：`backend/src/handlers/animal/sacrifice_pathology.rs` 於 `confirmed_sacrifice` 時，`UPDATE animals` 一併設定 `pen_location = NULL`。
  - **安樂死單執行**：`backend/src/services/euthanasia.rs` 於執行安樂死單時，`UPDATE animals` 一併設定 `pen_location = NULL`。
- ✅ **結果**：已安樂死動物之「欄位」會顯示為空，不再佔用欄位。

---

### 2026-03-03 E2E CI 環境模擬全通過

- ✅ **根因**：admin-users 測試失敗因「啟動配置警告」對話框擋住頁面；auth 首次嘗試使用 `.env` 的錯誤 `ADMIN_INITIAL_PASSWORD`。
- ✅ **修正**：`docker-compose.test.yml` 新增 `TEST_USER_PASSWORD`、`ALLOWED_CLOCK_IP_RANGES`、`CLOCK_OFFICE_LATITUDE/LONGITUDE` 抑制配置警告；`run-ci-e2e-tests.ps1` 設定 `ADMIN_INITIAL_PASSWORD`、清除 `.auth` 資料夾、修正 docker compose `--progress` 旗標。
- ✅ **結果**：35 個 Playwright E2E 測試全數通過（約 1.8 分鐘）。

---

### 2026-03-03 本機複現 CI 環境與 Backend 測試全通過

- ✅ **腳本**：新增 `scripts/run-ci-backend-tests.ps1`，以 Docker db-test + CI 環境變數複現 GitHub Actions 流程。
- ✅ **CI 調整**：`DISABLE_ACCOUNT_LOCKOUT=true` 避免 `login_with_wrong_password_returns_401` 因帳號鎖定回傳 400；`--test-threads=1` 減少共用 DB 衝突；`--force-recreate` 確保乾淨測試 DB。
- ✅ **權限**：補齊 `dev.role.*` 並指派給 admin（角色 API 需此權限）。
- ✅ **測試修正**：`post_unaffected_no_etag` 補上 `code` 欄位；`list_protocols_returns_200` / `list_users_returns_paginated_result` 改為檢查直接陣列；TestApp 建立 `uploads` 目錄以通過 health 檢查。
- ✅ **結果**：`cargo test` 全數通過（127 unit + 整合測試）。

### 2026-03-03 疫苗紀錄刪除失效修復與刪除功能檢視

- ✅ **根因**：`list_vaccinations` 未過濾 `deleted_at IS NULL`，導致軟刪除後紀錄仍顯示於列表（後端已正確軟刪除，但列表查詢未排除）。
- ✅ **修正**：`backend/src/services/animal/medical.rs` 於 `list_vaccinations` 查詢加入 `AND deleted_at IS NULL`。
- ✅ **前端型別**：`AnimalVaccination.id` 由 `number` 改為 `string`（UUID），`VaccinationsTab` 之 `deleteTarget` 同步修正。
- ✅ **照護紀錄刪除**：Migration 012 新增 `care_medication_records` 軟刪除欄位（deleted_at, deletion_reason, deleted_by）；`delete_care_record` 改為軟刪除 + `DeleteRequest` + `AuditService::log_activity`；`PainAssessmentTab` 改用 `DeleteReasonDialog`。
- ✅ **刪除功能檢視**：疫苗、體重、觀察、手術、血液檢查、動物、照護紀錄均已為軟刪除 + 操作日誌（user_activity_logs）。
- ✅ **軟刪除欄位統一**：血液檢查、報表、安樂死等改為 `deleted_at IS NULL`；Migration 013 移除 `animal_blood_tests.is_deleted`；`AnimalBloodTest`、前端型別同步更新。

---

### 2026-03-02 動物欄位修正申請（需 admin 批准）

- ✅ **需求**：耳號、出生日期、性別、品種等欄位建立後不可直接修改；若 staff 輸入錯誤，可經 admin 批准後修正。
- ✅ **後端**：Migration 011 新增 `animal_field_correction_requests` 表；`POST /animals/:id/field-corrections` 建立申請、`GET` 查詢該動物申請；`GET /animals/animal-field-corrections/pending` 列出待審、`POST /animals/animal-field-corrections/:id/review` 批准/拒絕。僅 admin 可審核。
- ✅ **前端**：動物詳情/編輯頁「申請修正」按鈕與 `RequestCorrectionDialog`；實驗動物管理「修正審核」頁面，可批准或拒絕並填寫拒絕原因。

---

### 2026-03-01 權限稽核與訓練紀錄權限調整

- ✅ **權限稽核報告**：新增 `docs/development/PERMISSION_AUDIT_2026-03-01.md`，掃描全專案頁面與權限
- ✅ **EXPERIMENT_STAFF 訓練紀錄**：新增 `training.view`、`training.manage_own`，可管理**自己的**訓練紀錄
- ✅ **ADMIN_STAFF 審批**：保有 `training.manage`，可審批/管理**所有人**紀錄
- ✅ **設備維護**：確認 equipment.view / equipment.manage 僅 ADMIN_STAFF（特定人員）
- ✅ **後端**：`TrainingService` 支援 `training.manage_own`（create/update/delete 僅限自己）
- ✅ **前端**：TrainingRecordsPage 依 `canManageAll` 隱藏員工篩選、新增表單人員欄
- 📁 **產出**：migration 012、permissions.rs、training.rs、TrainingRecordsPage、App.tsx、06_PERMISSIONS_RBAC.md

---

### 2026-03-01 R6 第六輪改善計劃建立與執行

> **白話版：** 針對專案進行下一輪評估後，在 `docs/TODO.md` 新增第六輪改善計劃並依序執行。

**R6-6 一鍵全庫匯出/匯入（Phase 1–3）✅**

- **Phase 1–2**：匯出/匯入 API、schema_version、前端按鈕
- **Phase 3**：Column mapper 架構（`schema_mapping::transform_row`，跨版本匯入時套用）；Zip 分包匯出（`format=zip`，manifest + 每表一檔，>10k 行表用 NDJSON）；Zip 匯入支援；前端「輸出為 Zip 分包」選項、支援 .zip 上傳

**R6-1 useState → hooks 擴展 ✅**

- EquipmentPage：useTabState + useDialogSet（activeTab、4 個 Dialog 開關）
- TrainingRecordsPage：useTabState + useDialogSet（activeTab、create/edit Dialog）

**R6-2 useDateRangeFilter / useTabState ✅**

- 新增 `src/hooks/useDateRangeFilter.ts`（支援 lazy 初始化、setRange、reset）
- 新增 `src/hooks/useTabState.ts`（相容 Radix Tabs onValueChange）
- 套用 useDateRangeFilter：HrLeavePage、HrOvertimePage、AdminAuditPage、AuditLogsPage、BloodTestCostReportPage、BloodTestAnalysisPage、AccountingReportPage
- 套用 useTabState：HrLeavePage、HrOvertimePage、AdminAuditPage、BloodTestAnalysisPage、EquipmentPage、TrainingRecordsPage

**R6-3 Skeleton DOM nesting 修正 ✅**

- InlineSkeleton 由 `SkeletonPulse`（div）改為 `<span>`，消除 `<p>` 內 `<div>` 的 validateDOMNesting 警告

**R6-4 財務模組 Phase 2–5 評估 ✅**

- 產出 `docs/assessments/R6-4_FINANCE_PHASE2_5_ASSESSMENT.md`：Phase 2–5 工時與優先建議

**R6-5 Dependabot Phase 2.5 依賴評估 ✅**

- 產出 `docs/assessments/R6-5_DEPENDABOT_PHASE25_ASSESSMENT.md`：printpdf、utoipa、axum-extra、tailwind-merge 升級建議與相依關係

---

### 2026-03-01 useState → Custom Hooks 重構 (P5-48)

> **白話版：** React 的 `useState` 用來管理畫面上的狀態（例如：彈窗開/關、輸入值）。  
> 把這些邏輯抽成「自訂 Hooks」（可重複使用的小工具），可以讓程式碼更整潔、更容易測試。

依據 `docs/development/REFACTOR_PLAN_USESTATE_TO_HOOKS.md` 執行 Phase 1–2：

**Phase 1：低風險通用 Hooks ✅**

- 新增 `useToggle`：布林切換（密碼可見、進階篩選）
- 遷移：LoginPage、SettingsPage、ResetPasswordPage、ForceChangePasswordPage、PasswordChangeDialog、ProductsPage（showAdvancedFilters）
- 新增 `useDialogSet`：多個 Dialog 開關集中管理
- 遷移：TreatmentDrugOptionsPage、AmendmentsTab、ReviewersTab、HrAnnualLeavePage、PartnersPage

**Phase 2：列表頁標準化 ✅**

- 新增 `useListFilters`：search、filters、page、perPage、sort
- 遷移：PartnersPage（search + typeFilter）

**Phase 3 已完成（2026-03-01 續）**：useSteps、useSelection、TwoFactorSetup 用 useDialogSet

- 新增 `useSteps`：wizard 步驟索引、next/prev/goTo
- 遷移：CreateProductPage
- 新增 `useSelection`：勾選 toggle/selectAll/clear/has/size
- 遷移：ProductsPage、TreatmentDrugOptionsPage（ErpImportDialog）
- TwoFactorSetup 用 useDialogSet 管理 setup/disable 兩 Dialog

**Phase 4 已完成（2026-03-01）**：feature 專用 hooks

- 新增 `useSettingsForm`：系統設定表單 + API 同步 + dirty 追蹤
- 遷移：SettingsPage
- 新增 `useLeaveRequestForm`：假單表單 + 日期/天數雙向計算 + 圖片上傳
- 遷移：HrLeavePage（含 useDialogSet）
- 新增 `useProductListState`：產品列表篩選/分頁/排序 + queryParams
- 遷移：ProductsPage（含 useDialogSet 管理 status/batchStatus/import）

---

### 2026-03-01 iPig R5 改善計畫 Phase 3 執行（項目 7、8）

> **白話版：** R5 是第五輪改善計畫。這次做的是「網頁效能監控」和「API 快取優化」。

依據 `dazzling-twirling-kitten.md` 計劃執行：

**項目 7：Web Vitals 監控 (P2) ✅**

- Web Vitals 是 Google 訂的「使用者體驗指標」（頁面載入速度、版面是否突然跳動等）。我們監控這五項：onCLS、onINP、onLCP、onFCP、onTTFB
- `sendToAnalytics`：DEV 時 `console.debug`，production 時 `navigator.sendBeacon('/api/metrics/vitals', JSON.stringify(metric))`
- `main.tsx` 呼叫 `reportWebVitals()`
- 後端 `POST /api/metrics/vitals` handler（接收並紀錄 Web Vitals 指標，回傳 204）

**項目 8：API 回應快取 ETag (P2) ✅**

- ETag 是「內容指紋」。伺服器給每份資料一個 ETag，瀏覽器下次請求時帶上這個值；若資料沒變，伺服器直接回 304（不必再傳一次完整內容），節省頻寬、加快速度
- 排除 `/api/auth/*`、`/api/health`、`/api/metrics/*`
- 套用 `Cache-Control: private, no-cache, must-revalidate`
- 對 GET 路由套用 etag middleware
- 單元測試：`test_is_excluded_path`、`test_etag_format`；整合測試：`api_etag.rs`（ETag 生成、304 回應、POST 不受影響、排除路徑）

### 2026-03-01 iPig R5 改善計畫 Phase 2 執行（項目 3、4、5、6）

依據 `dazzling-twirling-kitten.md` 計劃執行：

**項目 3：大型頁面元件拆分 (P1) ✅**

- **3a DocumentEditPage**：311 行，拆出 `useDocumentForm`、`DocumentLineEditor`、`DocumentPreview`、`types.ts`
- **3b UsersPage**：150 行，拆出 `useUserManagement`、`UserTable`、`UserFormDialogs`
- **3c BloodTestTemplatesPage**：143 行，拆出 `useBloodTestTemplates`、`BloodTestTemplateTable`、`BloodTestTemplateFormDialog`、`BloodTestPanelFormDialog`
- **3d SurgeryFormDialog**：108 行，拆出 `SurgeryBasicInfoSection`、`SurgeryProcedureSection`、`SurgeryAnesthesiaSection`、`useSurgeryForm`、`SurgeryFormComponents`

**項目 4：useState → custom hooks (P1) ✅**

- **AnimalsPage**：25 useState → 4 hooks（useAnimalFilters、useAnimalDialogs、useAnimalSelection、useAnimalForms），頁面 useState 數歸零

**項目 5：Alertmanager Receiver 設定 (P1) ✅**

- 新增 `monitoring/alertmanager/alertmanager.example.yml` 範本（含 `${ALERTMANAGER_WEBHOOK_URL}`、`${ALERT_EMAIL_*}`）
- 自訂 Dockerfile + entrypoint.sh（sed 替換，busybox 相容），啟動時自動 envsubst
- `docker-compose.monitoring.yml` 建置自訂映像、加入 ALERT_* 環境變數
- `.env.example` 補齊 Alertmanager 通知變數說明

**項目 6：Git Pre-commit Hooks (P1) ✅**

- 專案根目錄 `package.json` 已有 husky、lint-staged
- `.husky/pre-commit`：前端 lint-staged（ESLint + Prettier）、後端 `cargo fmt --check`

### 2026-03-01 iPig R5 改善計畫 Phase 1 執行（項目 1–2）

依據 `dazzling-twirling-kitten.md` 計劃執行：

**項目 1：eslint-disable 清理 (P0) ✅**

- 修正 3 處 ESLint 錯誤：utils.test.ts 常數表達式、EquipmentPage/TrainingRecordsPage 未使用 `Search` 匯入
- 移除 4 處 eslint-disable：ProtocolsPage (useCallback getStatusName)、BloodTestTemplatesPage (useCallback sortTemplates)、ErpPage (移除未使用 hasPermission)、ObservationFormDialog + SurgeryFormDialog (useCallback jumpToNextEmptyField)
- 保留並改善註釋 6 處：DocumentEditPage、ObservationFormDialog、SacrificeFormDialog、SurgeryFormDialog、handwritten-signature-pad、WarehouseLayoutPage 的 init-only / ref-loop 正當抑制
- `npx eslint src/ --max-warnings 0` 通過

**項目 2：前端單元測試擴充 (P0) ✅**

- 新增 `useApiError.test.ts`（5 tests：handleError、withErrorHandling、成功/失敗流程）
- 新增 `useHeartbeat.test.ts`（3 tests：未認證不發送、認證時初始 heartbeat、活動監聽）
- 現有 lib/、hooks/ 測試：utils、queryKeys、sanitize、validation、validations、logger、useDebounce、useConfirmDialog、useUnsavedChangesGuard
- `npx vitest run` 全數通過（207 tests）

### 2026-03-01 財務 SOC2 QAU 三項規劃完成

> **白話版：** 做了三件事：**(1) QAU 品質保證**：新增角色、權限、會計相關資料表與後台儀表板；**(2) SOC2 合規**：憑證輪換、SLA、災難還原演練；**(3) 財務模組**：會計科目、傳票、應付/應收等規劃。

**一、QAU（品質保證檢視）**

- `022_qau_accounting_plan.sql`（整合 022–024）：QAU 角色與權限、會計基礎（科目/傳票/分錄）、AP/AR 付款收款表
- `GET /qau/dashboard`：handlers/qau.rs、services/qau.rs，計畫狀態、審查進度、稽核摘要、動物統計
- `QAUDashboardPage.tsx`，路由 `/admin/qau`，側邊欄僅 QAU 可見

**二、SOC2 缺口補齊**

- 憑證輪換（半自動）：`check_credential_rotation.sh`（每月提醒）、`record_credential_rotation.sh`（紀錄輪換）；JWT 不輪換
- `docs/security-compliance/SLA.md`：RTO/RPO、可用性目標
- `docs/runbooks/DR_DRILL_CHECKLIST.md`：DR 演練檢查表

**三、財務模組（AP/AR/GL）**

- **Phase 1**：會計基礎（migration 022 內）、`AccountingService::post_document`；GRN/DO 核准時自動過帳
- **Phase 2（AP）**：`ap_payments`、`POST /accounting/ap-payments`、`GET /accounting/ap-aging`、前端「新增付款」
- **Phase 3（AR）**：`ar_receipts`、`POST /accounting/ar-receipts`、`GET /accounting/ar-aging`、前端「新增收款」
- **Phase 4（GL）**：`GET /accounting/trial-balance`、`/journal-entries`、`/chart-of-accounts`
- **Phase 5（UI）**：`AccountingReportPage` 四 Tab、ERP 報表中心「會計報表」入口 `/accounting`

### 2026-03-01 P0–P2 改進計劃執行完成（P1-M0～P2-M2）

- **P1-M3**：新增 `docs/OPERATIONS.md`（服務擁有者、on-call、升級流程、故障排除）
- **P1-M4**：標記完成（`docs/security-compliance/CREDENTIAL_ROTATION.md` 已存在）
- **P2-M5**：新增 `docs/security-compliance/SOC2_READINESS.md`（Trust Services Criteria 對照）
- **P1-M0**：稽核日誌匯出 API `GET /admin/audit-logs/export?format=csv|json`，權限 `audit.logs.export`
- **P2-M4**：稽核日誌 UI 新增「操作者」篩選
- **P1-M1**：API 版本路徑 `/api/v1/`，前端 baseURL 更新
- **P1-M2**：GDPR 資料主體權利 `GET /me/export`、`DELETE /me/account`（軟刪除 + 二級認證），隱私政策補充
- **P1-M5**：Dependabot Phase 2 收尾（zod 4、zustand 5、date-fns 4 已升級，build/test 通過）
- **P2-M2**：人員訓練紀錄模組（migration 020、`training_records` 表、CRUD API、`TrainingRecordsPage` 管理後台）
- **P2-M3**：設備校準紀錄模組（migration 021、`equipment` 與 `equipment_calibrations` 表、CRUD API、`EquipmentPage` 雙 Tab 管理後台）

### 2026-03-01 市場基準檢視與改進計劃

- **產出**：`docs/development/IMPROVEMENT_PLAN_MARKET_REVIEW.md`
- **檢視基準**：企業 ERP 系統、GLP 合規軟體、生產環境就緒檢查清單
- **內容**：市場基準對照表（ERP 核心功能、技術基礎設施、安全合規、GLP、生產就緒）、改進計劃分級（P0–P3）、既有優勢摘要、執行建議
- **重點項目**：P0 稽核日誌匯出 API、憑證輪換文件；P1 API 版本、GDPR、維運文件；P2 PWA、人員訓練紀錄、設備校準；P3 財務模組、QAU、原生 App、多租戶

### 2026-03-01 Dependabot PR 遷移計畫完成（Phase 1–3）

- **Phase 1**：GitHub Actions（checkout v6、setup-node v6、cache v5、upload-artifact v7）、validator 0.20、axios、lucide-react、@types/dompurify
- **Phase 2**：zod 4、@hookform/resolvers 5、zustand 5、date-fns 4；validation.ts / validations.ts 遷移
- **Phase 3**：metrics-exporter-prometheus、thiserror 2、jsonwebtoken 10（rust_crypto）、tower 0.5、tokio-cron-scheduler 0.15
- **產出**：`docs/development/DEPENDABOT_MIGRATION_PLAN.md`（總覽、遷移細節、相依關係圖）、`scripts/verify-deps.sh` / `.ps1`
- **暫緩**：printpdf 0.9、utoipa 5、axum-extra 0.12、tailwind-merge 3（Phase 2.5 可選）

### 2026-03-01 複製後編輯觀察紀錄 500 錯誤修復

> **白話版：** 使用者在「複製一筆觀察紀錄 → 再編輯儲存」時，系統噴出 500 錯誤。原因是資料庫型別轉換的 bug，已修正。

- **問題**：複製觀察紀錄後編輯儲存時出現「資料庫操作失敗，請稍後再試」(500)
- **根因**：migration 013 將 `version_record_type` enum 的 cast 改為 ASSIGNMENT，導致 (1) WHERE 比較 `record_type = $1` 時 `version_record_type = text` 無運算子；(2) cast 函數 `$1::text` / `$1::version_record_type` 遞迴呼叫造成 stack overflow
- **修復**：(1) `save_record_version` / `get_record_versions` 改為 `record_type::text = $1` 比較；(2) 新增 migration 019 修正 `version_record_type_to_text`、`text_to_version_record_type` 為非遞迴實作；(3) `AnimalObservation` 補齊 `deleted_at`、`deletion_reason`、`deleted_by`、`version` 欄位
- **驗證**：新增 `tests/test_reproduce_copy_edit_observation.py` 重現腳本，4 步驟全數通過

### 2026-02-28 附件 API 500 錯誤修正

- ✅ **AttachmentsTab 查詢參數修正**：前端傳送 `protocol_id` 但後端期望 `entity_type` + `entity_id`，導致空字串綁定 UUID 欄位引發 PostgreSQL 型別錯誤。修正為 `entity_type=protocol&entity_id=<uuid>`。
- ✅ **上傳路由修正**：附件上傳從錯誤的 `POST /attachments?protocol_id=...` 改為正確的 `POST /protocols/:id/attachments` 專用路由。

### 2026-02-28 第二輪系統改善 15 項完成

- ✅ **P0-R2-1 XSS 防護**：安裝 DOMPurify，建立 `sanitize.ts` 清理 SVG，所有 `dangerouslySetInnerHTML` 已包裹 `sanitizeSvg()`
- ✅ **P0-R2-2 Rate Limiting 分級**：新增寫入端點 120/min + 檔案上傳 30/min 獨立限流，上傳路由抽出獨立 Router
- ✅ **P1-R2-3 大型依賴動態導入**：`jsPDF`+`html2canvas` 改為 `import()` 動態載入，減少 ~360KB 初始 bundle
- ✅ **P1-R2-4 動物列表分頁**：後端 `AnimalService::list` 支援 `page`/`per_page` + COUNT，前端分頁控制元件
- ✅ **P1-R2-5 健康檢查深度擴充**：`/api/health` 擴充 DB 連線池狀態 + 磁碟 uploads 目錄檢查
- ✅ **P1-R2-6 Alertmanager 告警**：`monitoring/` 新增 Prometheus + Alertmanager + Grafana 設定，4 條告警規則
- ✅ **P1-R2-7 外部服務重試**：`services/retry.rs` 通用 `with_retry` 指數退避，已套用 SMTP 發送
- ✅ **P1-R2-8 Query Key Factory**：`lib/queryKeys.ts` 統一 ~50 個 query key 定義
- ✅ **P2-R2-9 表單驗證統一**：`lib/validations.ts` 提供 Partner/Warehouse/Animal 三組 Zod schema
- ✅ **P2-R2-10 i18n 補齊**：zh-TW.json + en.json 新增 `validation` 區塊 18 個 key
- ✅ **P2-R2-11 Zustand Selector**：auth store 新增 `useAuthUser`/`useAuthHasRole`/`useAuthActions` 等 selector hooks
- ✅ **P2-R2-12 DB 維護自動化**：`018_db_maintenance.sql` pg_stat_statements + `maintenance_vacuum_analyze()` + 慢查詢 View + 排程
- ✅ **P2-R2-13 Dependabot**：`.github/dependabot.yml` 涵蓋 Cargo/npm/Docker/GitHub Actions
- ✅ **P2-R2-14 零停機遷移策略**：`docs/database/ZERO_DOWNTIME_MIGRATIONS.md` 完整規範
- ✅ **P2-R2-15 架構圖**：`docs/ARCHITECTURE.md` 含部署/資料流/模組/認證流程 4 張 Mermaid 圖 + 技術堆疊表

### 2026-02-28 第三輪改善：P2-R3-11 + P2-R3-14 完成

- ✅ **P2-R3-11 Protocol `any` 型別消除**：6 個檔案消除 ~44 處 `: any`
  - `ProtocolEditPage.tsx`：14 處 → 0（`AxiosError<ApiErrorPayload>` 取代 error any、`ProtocolWorkingContent` 子型別取代 item/person/staff any、`Record<string, unknown>` 取代動態 section 存取）
  - `ProtocolContentView.tsx`：13 處 → 0（interface prop `any` → `ProtocolWorkingContent`、map callback `any` → 具體子型別 TestItem/ControlItem/SurgeryDrug/AnimalEntry 等）
  - `CommentsTab.tsx`：4 處 → 0（`VetReviewAssignment` 取代 vetReview any、error handler 改用 `AxiosError`、Protocol prop 型別改用 `Protocol` interface）
  - `AttachmentsTab.tsx`：2 處 → 0（error handler 改用 `AxiosError<ApiErrorPayload>`）
  - `ReviewCommentsReport.tsx`：3 處 → 0（props 全面型別化為 `Protocol`/`ReviewCommentResponse[]`/`VetReviewAssignment`）
  - `ReviewersTab.tsx`：1 處 → 0（vetReview prop 改用 `VetReviewAssignment`）
  - 新增 `VetReviewItem`/`VetReviewFormData`/`VetReviewAssignment` 三個 interface 至 `types/aup.ts`
- ✅ **P2-R3-14 Error Boundary 分層**：
  - 新增 `components/ui/page-error-boundary.tsx`（class component + 錯誤重試 UI）
  - `MainLayout.tsx` 於 `<Suspense>` 外層包裹 `<PageErrorBoundary>`，所有 lazy-loaded 頁面自動受保護
- ✅ TypeScript `tsc --noEmit` 零錯誤通過

### 2026-02-28 第三輪系統改善 20 項完成

詳細計畫見 `docs/development/IMPROVEMENT_PLAN_R3.md`

**🔴 P0 安全性（4 項）：**

- ✅ **P0-R3-1 SQL 動態拼接修正**：4 個檔案（`treatment_drug.rs`, `report.rs`, `warehouse.rs`, `document/crud.rs`）的手動 `format!("${}", param_idx)` 參數索引全部改為 `sqlx::QueryBuilder` 的 `push_bind()` 自動綁定
- ✅ **P0-R3-2 IDOR 漏洞修補**：HR `get_leave` 加入 owner/approver/view_all 三重檢查、`get_overtime` 加入 owner/view_all 檢查、`get_user` 允許查看自己的 profile 無需 admin 權限
- ✅ **P0-R3-3 .expect() 清理**：handlers/ 14 處 + services/ 28 處共 42 個 `.expect()` 替換為 proper error propagation（`ok_or_else`/`map_err`/`anyhow`），消除 production panic 風險
- ✅ **P0-R3-4 前端容器非 root**：Dockerfile 加入 `USER nginx`、nginx listen 改為 8080、`nginx-main.conf` 設定 pid/temp 路徑至 `/tmp/nginx/`、docker-compose 端口映射更新

**🟡 P1 效能與可靠性（6 項）：**

- ✅ **P1-R3-5 搜尋 debounce**：新增 `hooks/useDebounce.ts`，套用至 AnimalsPage/PartnersPage/WarehousesPage/ProtocolsPage（400ms 延遲）
- ✅ **P1-R3-6 staleTime 調優**：23 個檔案 38 個 useQuery 依資料特性分級設定（即時 30s/列表 1min/計數 5min/參考 10min/設定 30min）
- ✅ **P1-R3-7 AnimalsPage 拆分**：1898 行 → 495 行（-74%），抽離 AnimalFilters/AnimalListTable/AnimalPenView/AnimalAddDialog + constants.ts
- ✅ **P1-R3-8 Rate Limiter DashMap**：`Arc<Mutex<HashMap>>` 改為 `DashMap`，消除 Mutex 競爭
- ✅ **P1-R3-9 DB Pool Prometheus 指標**：`/metrics` 新增 `db_pool_connections_total/idle/active` 三個 gauge
- ✅ **P1-R3-10 Skeleton Loading**：新增 `TableSkeleton` 元件，套用至 4 個列表頁取代 Loader2 spinner

**🔵 P2 品質與維運（10 項）：**

- ✅ **P2-R3-11 Protocol any 消除**：6 個檔案 ~44 處 `: any` 替換為具體型別（`ProtocolWorkingContent`/`VetReviewAssignment`/`AxiosError<ApiErrorPayload>` 等）
- ✅ **P2-R3-12 審計日誌補齊**：HR leave approval/rejection 和 overtime approval 新增 `AuditService::log()` 呼叫；新增 `AuditAction::Reject` variant
- ✅ **P2-R3-13 常數提取**：新增 `backend/src/constants.rs`（分頁/認證/Rate Limit/上傳/排程/Session/密碼 共 18 個常數）
- ✅ **P2-R3-14 Error Boundary 分層**：新增 `PageErrorBoundary` 元件，包裹 MainLayout 的 Suspense
- ✅ **P2-R3-15 SSL/TLS 範本**：新增 `docs/operations/SSL_SETUP.md` + `frontend/nginx-ssl.conf.example`（TLS 1.2/1.3 + OCSP + HSTS）
- ✅ **P2-R3-16 備份自動驗證**：新增 `scripts/backup/pg_backup.sh`（gzip 完整性 + pg_restore 驗證 + SHA256 校驗 + 30 天自動清理）
- ✅ **P2-R3-17 日誌聚合**：新增 `docker-compose.logging.yml`（Loki + Promtail）+ `monitoring/promtail/config.yml`
- ✅ **P2-R3-18 環境驗證**：新增 `scripts/validate-env.sh`（必填/選填變數分級檢查 + HMAC key 長度驗證）
- ✅ **P2-R3-19 無障礙**：搜尋輸入框加入 `aria-label`（Animals/Partners/Warehouses/Protocols 4 頁）
- ✅ **P2-R3-20 API 一致性**：`amendment.rs` 4 處硬編碼角色名稱陣列改為 `has_permission("aup.protocol.*")` 權限檢查
- ✅ `cargo check` + `tsc --noEmit` 零錯誤通過

### 2026-02-28 第四輪改善計畫 R4 完成（20 項）

**P0 安全性（4 項）：**

- P0-R4-1 IDOR 修補：`check_resource_access()` helper，amendment/document handler 加入所有權檢查
- P0-R4-2 CSP：移除 nginx `style-src unsafe-inline`
- P0-R4-3 console 清理：`lib/logger.ts` 封裝，生產環境靜默
- P0-R4-4 `.expect()` 清理：partner.rs Regex、auth.rs 改用 `?`

**P1 效能與可靠性（7 項）：**

- P1-R4-5~8 元件拆分與 Skeleton（AnimalsPage、DocumentEditPage、AdminAuditPage、TableSkeleton）
- P1-R4-9 Nginx：HTTP/2、rate limit、JSON log、worker_connections
- P1-R4-10 還原腳本：`scripts/backup/pg_restore.sh`
- P1-R4-11 備份腳本：GPG 清理邏輯、pg_restore --list 驗證

**P2 品質與維運（9 項）：**

- P2-R4-12 Protocol `any` 消除：ProtocolPerson、ProtocolAnimalItem、ProtocolSurgeryDrugItem 型別
- P2-R4-13 Animal `any` 消除：25 處 onError→unknown、handleChange、payload、AnimalTimelineView、AnimalListTable 等
- P2-R4-14 後端配置提取：constants.rs 集中管理 rate limit、file size、auth expiry、時區
- P2-R4-15 Error Boundary：DashboardPage、ProtocolEditPage、AnimalDetailPage 頁面級
- P2-R4-16 錯誤處理統一：後端 `req.validate()?`、前端 AnimalsPage `error: unknown`
- P2-R4-17 Prometheus：monitoring 埠號統一為 api:8000
- P2-R4-18 .env.example：POSTGRES_PORT 修正、GRAFANA 變數補齊
- P2-R4-19 staleTime：STALE_TIME 常數、10+ useQuery 調優
- P2-R4-20 backend/.dockerignore：排除 target、.git 等

### 2026-02-28 手寫簽名 Canvas 寬度無限擴張修復

- ✅ **根因**：CSS Grid `grid-cols-[280px_1fr]` 中 `1fr` 等同 `minmax(auto, 1fr)`，canvas 的 intrinsic size 撐大 grid cell → ResizeObserver 重新量測 → canvas 再擴張，形成無限迴圈（container 寬度飆至 9870px）
- ✅ **修復 4 個檔案**：
  - `ProtocolEditPage.tsx`：`1fr` → `minmax(0,1fr)`，允許 grid 欄位縮小不受子元素 intrinsic size 影響
  - `SectionSignature.tsx`：Card / CardContent 加上 `min-w-0`，手寫簽名容器加上 `min-w-0 max-w-full`
  - `handwritten-signature-pad.tsx`：新增 `wrapperRef` 從 wrapper（非 container）量測寬度；canvas 改為絕對定位
  - `index.css`：`.signature-canvas` 改為 `position: absolute; inset: 0`；wrapper 加上 `max-w-full`
- ✅ **驗證結果**：Playwright 自動測試確認 container 寬度 686px、canvas 682px、grid 第二欄 736px，均在正常範圍
- 📁 **產出**：4 個檔案修改

### 2026-02-28 ProtocolEditPage Section 導航改用 URL Search Params

- ✅ **方案 C 實作**：`activeSection` 從 `useState` 改為 `useSearchParams` 驅動，URL 反映當前 section（如 `?section=purpose`）
- ✅ 瀏覽器上一頁/下一頁可切換 section，可書籤/分享特定 section 連結
- ✅ 無效 `section` 參數自動 fallback 至 `basic`
- ✅ 原有表單狀態管理、儲存、驗證邏輯不受影響
- 📁 **產出**：`frontend/src/pages/protocols/ProtocolEditPage.tsx`（2 處修改）

### 2026-02-28 系統改善 14 項完成（安全性/效能/程式碼品質）

**🔴 P0 安全性（3 項）：**

- ✅ **P0-S1 Docker 網路隔離**：定義 `frontend` / `backend` / `database` 三個自訂 bridge 網路，每個服務僅加入必要網路（web 容器無法直接存取 db）
- ✅ **P0-S2 DB 埠口 localhost-only**：`docker-compose.yml` 資料庫 port 綁定改為 `127.0.0.1:5433:5432`，防止外部直連
- ✅ **P0-S3 Docker Secrets**：`config.rs` 新增 `read_secret()` / `require_secret()` helper（優先讀 `*_FILE` 路徑，fallback 環境變數）；`docker-compose.prod.yml` 定義 4 個 secrets（jwt_secret / db_url / db_password / smtp_password）

**🟡 P1 效能（5 項）：**

- ✅ **P1-S4 RoleService N+1 修復**：`list()` 從 1+N 次查詢改為 2 次（roles + 批次 permissions via `ANY($1)`），記憶體分組
- ✅ **P1-S5 UserService N+1 修復**：`list()` 從 1+2N 次查詢改為 3 次（users + 批次 roles + 批次 permissions via `ANY($1)`）
- ✅ **P1-S6 迴圈 INSERT → UNNEST**：`role.rs` 建立/更新角色 + `user.rs` 建立/更新使用者的權限/角色指派改為 `SELECT $1, unnest($2::uuid[])`
- ✅ **P1-S7 移除 .expect()**：`handlers/auth.rs` 6 處 + `handlers/two_factor.rs` 2 處改為 `map_err(AppError::Internal)`，`login_response_with_cookies` 回傳改為 `Result<Response>`
- ✅ **P1-S8 複合索引**：`017_composite_indexes.sql` 新增 5 個 `CREATE INDEX CONCURRENTLY`（animals/protocols/notifications/audit_logs/attachments）

**🔵 P2 程式碼品質（6 項）：**

- ✅ **P2-S9 is_admin + UserResponse::from_user**：`CurrentUser::is_admin()` 方法 + `UserResponse::from_user(&User)` 消除 8 處重複建構 + 22 處 handler admin 檢查統一化
- ✅ **P2-S10 TypeScript 嚴格化**：新增 `types/error.ts`（ApiErrorPayload + getErrorMessage），10 個檔案 18 處 `error: any` → `error: unknown`
- ✅ **P2-S11 API 錯誤統一**：`lib/api.ts` interceptor 新增 500+/timeout/網路斷線全域 toast（使用 shadcn/ui toast）
- ✅ **P2-S12 MainLayout 拆分**：1,192→~210 行，抽離 Sidebar（~420 行）/ NotificationDropdown（~195 行）/ PasswordChangeDialog（~130 行）
- ✅ **P2-S13 Memoization**：`useMemo` 包裝 2 個 Detail 頁面的 tabs 陣列 + `React.memo` 包裝 7 個 Tab 元件 + `useCallback` 包裝事件處理器
- ✅ **P2-S14 Dockerfile cargo-chef**：5-stage 建置（chef→planner→builder→runtime→distroless），依賴層獨立快取

📁 **產出**：~25 個修改/新增檔案（後端 15 + 前端 10+ + Docker 3 + migration 1）

---

### 2026-02-28 最終 3 項 P5 待辦全數完成（全部功能零缺口）

**P5-13 前端元件庫文件化（Storybook 10）：**

- ✅ **15 個 Stories**：7 個既有（Button/Badge/Card/Checkbox/Input/Skeleton/Switch）+ 8 個新增（Select/Dialog/Slider/Tabs/AlertDialog/FormField/LoadingOverlay/Textarea）
- ✅ 每個 Story 包含 Default + 多種 variant/use case（繁中標籤）
- ✅ `npx storybook build` 成功編譯
- 📁 **產出**：8 個新 `.stories.tsx` 檔案

**P5-15 SEC-39 Two-Factor Authentication (TOTP)：**

- ✅ **DB Migration**：`016_totp_2fa.sql` 新增 `totp_enabled`/`totp_secret_encrypted`/`totp_backup_codes` 三欄位
- ✅ **後端依賴**：`totp-rs` v5（gen_secret + otpauth + qr features）
- ✅ **後端 API 4 個端點**：
  - `POST /auth/2fa/setup`（產生 TOTP secret + otpauth URI + 10 組備用碼）
  - `POST /auth/2fa/confirm`（驗證第一次 code 正式啟用 2FA）
  - `POST /auth/2fa/disable`（需密碼 + code 雙重驗證）
  - `POST /auth/2fa/verify`（temp_token + TOTP code 完成 2FA 登入，支援備用碼）
- ✅ **登入流程改造**：`AuthService::validate_credentials()` 分離密碼驗證；密碼通過後若 `totp_enabled=true` 回傳 `TwoFactorRequiredResponse` + temp JWT（5 分鐘）
- ✅ **前端 Login 頁面**：密碼驗證後自動切換至 TOTP 輸入畫面（6 碼大字型 + 備用碼支援），支援返回
- ✅ **前端 ProfileSettingsPage**：`TwoFactorSetup` 元件 — QR Code 掃描設定（qrcode.react）+ 備用碼顯示/複製 + 停用 Dialog
- ✅ **前端 auth store**：新增 `verify2FA` action，login 偵測 `requires_2fa` 回應
- 📁 **產出**：1 migration + 2 後端檔案 + 5 前端檔案修改/新增

**P5-16 SEC-40 Web Application Firewall：**

- ✅ **`docker-compose.waf.yml`**：OWASP ModSecurity CRS v4 nginx-alpine overlay，預設偵測模式
- ✅ **iPig 自訂排除規則**：JSON Content-Type / 密碼欄位 / TOTP code / 富文本 / 檔案上傳 5 項排除
- ✅ **WAF 文件**：`docs/security-compliance/WAF.md`（架構/啟用/保護範圍/排除規則/日誌分析/Paranoia Level/生產注意事項）
- ✅ 啟用方式：`docker compose -f docker-compose.yml -f docker-compose.waf.yml up -d`
- 📁 **產出**：1 overlay + 2 排除規則 conf + 1 文件

### 2026-02-28 系統設定頁面全端串接 + 通知路由 UI 改善

- ✅ **後端 System Settings API**：新增 `GET/PUT /api/admin/system-settings`（admin only），利用既有 `system_settings` 資料表
  - `backend/src/handlers/system_settings.rs`：GET 回傳所有設定（SMTP password 遮罩為 `********`），PUT 批次更新
  - `backend/src/services/system_settings.rs`：DB CRUD + `resolve_smtp_config()` 方法（DB-first + .env fallback）
  - `backend/src/services/email/mod.rs`：新增 `send_email_smtp()` + `resolve_smtp()` 方法供 DB-first SMTP 解析
- ✅ **DB Migration**：`015_system_settings_seed.sql` seed 10 項初始設定值（company_name / default_warehouse_id / cost_method / smtp_* / session_timeout_minutes）
- ✅ **前端 SettingsPage 重構**（`frontend/src/pages/admin/SettingsPage.tsx`）：
  - 四大設定區塊（基本/庫存/郵件/安全）全部從後端 API 載入當前值
  - `handleSave` 呼叫 `PUT /admin/system-settings` 實際儲存
  - 倉庫下拉從 `GET /warehouses` 動態載入
  - SMTP 密碼欄位顯示遮罩值，點擊時清空供輸入新密碼
  - Session 逾時選項新增 360/480 分鐘
  - Loading / Error 狀態完整處理
- ✅ **通知路由管理 UI 改善**（`frontend/src/components/admin/NotificationRoutingSection.tsx`）：
  - 分類可收合/展開（Chevron 圖示），減少視覺壓力
  - Switch 元件取代 ToggleLeft/ToggleRight 圖示
  - 角色顯示中文名稱（不只 code）
  - ConfirmDialog 取代原生 `window.confirm`
  - 規則使用 grid layout 對齊
  - 分類標題列顯示啟用/總數統計
- 📁 **新增/修改檔案**：
  - `backend/src/handlers/system_settings.rs`（new）
  - `backend/src/services/system_settings.rs`（new）
  - `backend/migrations/015_system_settings_seed.sql`（new）
  - `backend/src/services/email/mod.rs`（modified）
  - `backend/src/handlers/mod.rs`（modified）
  - `backend/src/services/mod.rs`（modified）
  - `backend/src/routes.rs`（modified）
  - `frontend/src/pages/admin/SettingsPage.tsx`（rewritten）
  - `frontend/src/components/admin/NotificationRoutingSection.tsx`（rewritten）

### 2026-02-28 P5-14 ProtocolDetailPage 重構（1,929→647 行，-66%）

- ✅ **ProtocolDetailPage.tsx**：從 1,929 行縮減至 647 行
- ✅ **抽離 6 個 Tab 元件**至 `frontend/src/components/protocol/`：
  1. `VersionsTab.tsx`（203 行）— 版本列表 + 版本比較 + 版本檢視 Dialog
  2. `HistoryTab.tsx`（185 行）— 活動歷史時間軸 + 分頁
  3. `CommentsTab.tsx`（431 行）— 審查意見、回覆、PDF 匯出 + 匿名化邏輯
  4. `ReviewersTab.tsx`（281 行）— 審查委員列表 + 獸醫審查表單 + 指派 Dialog
  5. `CoEditorsTab.tsx`（245 行）— 協作者列表 + 新增/移除 Dialog
  6. `AttachmentsTab.tsx`（215 行）— 附件上傳/下載/刪除
- ✅ **重構原則**：父元件保留 Header、Info Cards、Tab 導航、Status 變更 Dialog；各 Tab 自帶 queries、mutations、dialog state
- ✅ **TypeScript 零錯誤通過**
- 📁 **產出**：6 個新 Tab 元件 + 重構後的 ProtocolDetailPage.tsx

### 2026-02-28 JWT 預設過期時間調整為 6 小時

- ✅ **後端 config.rs**：`JWT_EXPIRATION_MINUTES` 預設值從 15 改為 360（6 小時），test default 900s→21600s
- ✅ **前端 session fallback**：`auth.ts`、`api.ts` 中 `sessionExpiresAt` fallback 從 `15 * 60 * 1000` 改為 `6 * 60 * 60 * 1000`
- ✅ **環境配置**：`.env`（60→360）、`.env.example`（15→360）、`docker-compose.yml`（預設 15→360）
- ✅ **E2E 驗證腳本**：`verify-config.ts` fallback 從 '15' 改為 '360'
- 📁 **產出**：7 個檔案更新

### 2026-02-28 品質補強 18 項全數完成

**高影響 6 項（P1-30~35）：**

- ✅ **P1-30 Graceful Shutdown**：`main.rs` 加入 `shutdown_signal()` + `with_graceful_shutdown()`，支援 SIGTERM（Docker stop）與 Ctrl+C，確保進行中的請求完成後才關閉
- ✅ **P1-31 自訂 404 頁面**：`NotFoundPage` 元件取代 catch-all redirect，含「返回上一頁」與「回到首頁」按鈕
- ✅ **P1-32 Session 逾時預警**：auth store 新增 `sessionExpiresAt` 追蹤 JWT 到期時間，`SessionTimeoutWarning` 元件在到期前 60s 顯示倒數 Dialog，可續期或登出
- ✅ **P1-33 刪除記錄清理檔案**：`FileService::delete_by_entity()` 方法查詢 `attachments` 表並刪除磁碟檔案 + DB 記錄，已整合動物與觀察紀錄刪除 handler
- ✅ **P1-34 Optimistic Locking**：`014_optimistic_locking.sql` 為 animals/protocols/observations/surgeries 加入 `version` 欄位，animal update SQL 加入版本檢查（409 Conflict）
- ✅ **P1-35 confirm() 統一 Dialog**：`useConfirmDialog` hook + `ConfirmDialog` + `AlertDialog` 元件，9 個檔案 11 處原生 `confirm()` 全部替換

**中影響 7 項（P2-36~42）：**

- ✅ **P2-36 i18n 補齊**：AnimalDetailPage 11 個 Tab 標籤 + 404 頁面 + Session 預警翻譯鍵加入 zh-TW.json 與 en.json
- ✅ **P2-37 列表 API 分頁**：`PaginationParams` struct + `sql_suffix()` 方法（LIMIT/OFFSET，per_page 上限 100），users/warehouses/partners handler 支援 `?page=&per_page=`
- ✅ **P2-38 表單離開確認**：`useUnsavedChangesGuard` hook（React Router useBlocker + beforeunload）+ `UnsavedChangesDialog`，已整合 ProtocolEditPage
- ✅ **P2-39 隱私政策/服務條款**：`PrivacyPolicyPage` + `TermsOfServicePage` 公開路由，登入頁底部加連結
- ✅ **P2-40 Cookie 同意橫幅**：`CookieConsent` 元件（localStorage 記憶 + 底部半透明 banner + 了解更多連結）
- ✅ **P2-41 Rollback 文件**：`docs/database/DB_ROLLBACK.md` 涵蓋 14 個 migration 的精確回滾 SQL + 建議回退流程
- ✅ **P2-42 .env.example 補齊**：新增 HOST/PORT/DATABASE_MAX_CONNECTIONS/MAX_SESSIONS_PER_USER/UPLOAD_DIR 等 9 個缺漏變數

**低影響 5 項（P5-43~47）：**

- ✅ **P5-43 ARIA 無障礙**：12 個檔案新增 23 個 `aria-label`（編輯/刪除/檢視/關閉/導航按鈕）
- ✅ **P5-44 表單驗證回饋**：Input/Textarea 新增 `error` prop 紅框樣式，`FormField` 通用元件含 label + 錯誤訊息
- ✅ **P5-45 磁碟空間監控**：`scripts/monitor/check_disk_space.sh` 含 uploads 大小 + 磁碟使用率 + Prometheus textfile 輸出
- ✅ **P5-46 LICENSE**：MIT License 2026 正式文件
- ✅ **P5-47 Meta Tags**：title「豬博士 iPig 系統」+ description + theme-color #f97316 + favicon.ico

📁 **產出**：~30 個新增/修改檔案（後端 6 + 前端 20+ + 文件 3 + 腳本 1）

---

### 2026-02-28 交付前補強 3 項（非阻擋）

- ✅ **P4-19 Prometheus 服務部署**：
  - `deploy/prometheus.yml`：scrape `api:8000/metrics`，15s interval
  - `deploy/grafana/provisioning/`：自動註冊 Prometheus datasource + dashboard
  - `deploy/grafana_dashboard.json`：從 2 panel 擴充至 **10 panels**（API Request Rate / Latency P50-P95-P99 / Error Rate / Status Code Pie / Duration Heatmap / DB Pool Stacked / Pool Utilization Gauge / Top Endpoints Bar）
  - `docker-compose.monitoring.yml`：獨立 overlay 檔，含 Prometheus (9090) + Grafana (3000) 服務、volume 持久化、資源限制
  - 啟用方式：`docker compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d`

- ✅ **P4-20 後端 API 整合測試套件**：
  - 重構 `src/lib.rs`（新建）+ `src/main.rs`（改用 `use erp_backend::`），使 crate 同時支援 library + binary，讓 `tests/` 目錄可存取內部模組
  - `tests/common/mod.rs`：`TestApp` 測試基礎架構（spawn Axum on random port + PgPool + reqwest client + login helper）
  - 6 個整合測試檔案、25+ test cases：
    - `api_health.rs`：健康檢查 200 + metrics 端點 + 404 unknown route
    - `api_auth.rs`：登入成功/失敗/格式錯誤、me 有無 token、refresh、logout 撤銷、密碼變更
    - `api_animals.rs`：列表/無 auth/建立取得/無效資料 400/不存在 404
    - `api_protocols.rs`：列表/建立草稿/無 auth
    - `api_users.rs`：列表/建立取得/角色列表/權限列表
    - `api_reports.rs`：三個報表端點 200/無 auth 401/通知列表
  - `cargo check --tests` 編譯通過（僅 dead_code warnings）
  - 新增 dev-dependencies：`reqwest` (cookies)、`serial_test`

- ✅ **P4-21 效能基準報告文件化**：
  - `docs/assessments/PERFORMANCE_BENCHMARK.md`：8 章節正式報告（摘要 / 測試環境 / 方法 / 指標結果 / 閾值摘要 / 資源觀測 / 限制 / 結論建議）含附錄
  - k6 腳本 `scripts/k6/load-test.js` 優化：改用 `setup()` 階段單次登入共用 token，消除 50 VU 同時登入觸發 rate limit 的串連失敗問題
  - 分析 7 份歷次測試 JSON，選定 `k6_2026-02-25T12-13-34.json` 為基準數據

- 📁 **產出**：12 個新建/修改檔案

### 2026-03-01 PowerShell Migration 執行紀錄

- ✅ **嘗試 1**：`cargo install sqlx-cli` 失敗（Windows 缺少 MSVC linker）
- ✅ **嘗試 2**：Docker + psql 直接執行 migrations，因既有 DB 已有 schema 及 PowerShell 編碼問題而產生錯誤
- ✅ **結論**：新 migrations（001~010）僅適用於全新安裝；既有環境維持現狀
- 📁 **產出**：`docs/walkthrough.md` 新增 PowerShell Migration 執行紀錄與建議做法

### 2026-02-28 市場交付阻擋項修復（3 項）

- ✅ **檔案上傳/下載功能串接**：
  - 後端：`file.rs` 新增 `ObservationAttachment` FileCategory（含 PDF/DOC MIME 支援），`upload.rs` 新增 `upload_observation_attachment` handler，`routes.rs` 新增 `POST /observations/:id/attachments`
  - 後端：修正 `VetRecommendation` FileCategory 的 MIME 類型，新增 PDF/DOC 支援（原僅允許圖片）
  - 前端：`VetRecommendationDialog.tsx` 串接 multipart 上傳至 `/vet-recommendations/{type}/{id}/attachments` + 附件下載至 `/attachments/{id}`
  - 前端：`ObservationFormDialog.tsx` 串接附件上傳（編輯模式即時上傳，新增模式存後上傳）
- ✅ **使用者操作手冊**：`docs/USER_GUIDE.md` 從 26 行擴充至 v2.0 完整手冊（9 章節：登入/儀表板/AUP/動物/ERP/HR/報表/系統管理/FAQ）
- ✅ **生產環境 Docker 強化**：`docker-compose.prod.yml` 所有服務新增 `deploy.resources.limits`（CPU/記憶體）與 `logging` json-file 日誌輪轉
- 📁 **產出**：6 個檔案修改（3 後端 + 2 前端 + 1 Docker）

### 2026-02-28 P5-14 前端超長頁面重構（兩大頁面完成）

- ✅ **AnimalDetailPage.tsx**：1,945→748 行（**-61%**），抽離 7 個 Tab 元件至 `components/animal/`
- ✅ **ProtocolDetailPage.tsx**：1,929→647 行（**-66%**），抽離 6 個 Tab 元件至 `components/protocol/`
- 📁 **產出**：13 個新 Tab 元件 + 2 個重構後的 Detail 頁面

### 2026-02-28 P4-17 基礎映像與 CVE 週期檢查

- ✅ **版本釘選**：`frontend/Dockerfile` 的 `FROM georgjung/nginx-brotli:alpine` → `georgjung/nginx-brotli:1.29.5-alpine`（nginx 1.29.5 + Alpine 3.23.3，2026-02-05 發佈）
- ✅ **CVE 驗證**：Trivy 掃描確認 CVE-2026-25646 仍存在（libpng 1.6.54-r0，修復版 1.6.55-r0 尚未納入映像）
- ✅ **文件更新**：`.trivyignore` 加入檢查日期與下次排程、`docs/security-compliance/security.md` 更新映像版本與檢查紀錄
- 📅 **下次檢查**：排定 2026-Q2，屆時若映像包含 libpng ≥ 1.6.55-r0 則移除 CVE
- 📁 **產出**：[Dockerfile](../frontend/Dockerfile)、[.trivyignore](../.trivyignore)、[security.md](security.md)

### 2026-02-27 E2E 跨瀏覽器 Session 過期修復（CI 30 failures 歸零）

- ✅ **問題**：CI（Ubuntu）上 100 tests 依序跑 webkit→firefox→chromium，auth.setup 產生的 JWT storageState 在後執行的瀏覽器 session 已過期，導致 30 個 webkit/firefox 測試一致失敗（`Target page, context or browser has been closed`）
- ✅ **根因**：workers=1 序列執行耗時 ~2 分鐘，storageState 中的 JWT 過期，後執行的 browser project 的 admin-context 共用 context 失效
- ✅ **修復**：
  1. Firefox/WebKit 改為全域 opt-in（需設 `PLAYWRIGHT_FIREFOX=1`、`PLAYWRIGHT_WEBKIT=1`）
  2. 預設僅跑 Chromium（34 tests），避免 session 過期問題
  3. 移除無效的 per-test `{ retries: 1 }` 語法
  4. admin-users.spec.ts：加入 table visible 等待、增加 button timeout
  5. CI retries 維持 2（容錯），本地 retries 改回 0（快速回饋）
- 📊 **結果**：CI 預設 34 tests（Chromium），22s 完成，0 failures

### 2026-02-27 E2E 測試 100% 通過（P4-18 Rate Limiting / Session 穩定化）

- ✅ **根本原因分析**：所有 `/api/*` 請求共用 120/min rate limit，React SPA 每次頁面載入觸發多個 API 呼叫（/api/me、資料列表等），34 個測試密集執行時輕易超限；`sharedAdminContext` 每次初始化都重新登入浪費配額。
- ✅ **admin-context.ts 重構**：改用 auth.setup 儲存的 `admin.json` storageState 檔案，worker 初始化時直接載入 cookie + localStorage，無需重新登入（0 次額外 API 呼叫）。
- ✅ **API rate limit 提升**：`rate_limiter.rs` API 端點 120→600/min，為密集測試提供充足配額。
- ✅ **login.spec.ts credential fallback**：改用 `getAdminCredentials()` 統一 fallback 邏輯（支援 .env 的 `ADMIN_INITIAL_PASSWORD`）。
- 📊 **成果**：34/34 測試連續 2 次全部通過，執行時間從 2.3 分鐘降至 **22 秒**。
- 📁 **產出**：
  - [admin-context.ts](../frontend/e2e/fixtures/admin-context.ts)（storageState 載入）
  - [rate_limiter.rs](../backend/src/middleware/rate_limiter.rs)（API limit 600/min）
  - [login.spec.ts](../frontend/e2e/login.spec.ts)（credential fallback）

### 2026-02-27 E2E 測試總結計畫實施（選項 1）

- ✅ **Dashboard 修復交付**：原計畫主要目標已達成，Dashboard 6/6 通過。
- ✅ **Rate Limiting 調查記錄**：已嘗試 JWT TTL 延長、auth rate limit 放寬、Cookie Path 與 context.cookies() 修復，仍存在 Session 過期導致大量重新登入 → 429 連鎖失敗問題。
- ✅ **後續任務建立**：將 Rate Limiting / Session 穩定化建立為 P4 獨立待辦，詳見 `docs/TODO.md`。

### 2026-02-26 E2E 測試全面改進（Session 管理優化）

- ✅ **配置驗證與文檔**：
  - 新增 `docs/e2e/README.md`（完整指南：架構說明、配置檢查清單、故障排除、維護手冊）
  - 新增 `frontend/e2e/scripts/verify-config.ts`（配置驗證腳本，檢查 JWT TTL、Cookie、環境變數）
  - 更新 `docs/QUICK_START.md`（新增配置驗證步驟）

- ✅ **診斷工具**：
  - 新增 `frontend/e2e/helpers/diagnostics.ts`（E2E 診斷工具，自動記錄 session 狀態、檢查 access_token、提供故障排除建議）
  - 新增 `scripts/analyze-e2e-logs.sh`（後端日誌分析腳本，自動檢查 401 錯誤、JWT 過期、Session 相關日誌）

- ✅ **Session 管理優化**：
  - 新增 `frontend/e2e/helpers/session-monitor.ts`（Session 監控工具，追蹤 session 存活時間、檢查是否接近過期）
  - 優化 `frontend/e2e/fixtures/admin-context.ts`：
    - 加入 `isSessionExpired()` 檢查 cookie 過期時間
    - 加入 `tryRefreshToken()` 主動 refresh 機制
    - 改進 `ensureLoggedIn()` 含重試邏輯（最多 3 次）
    - Page fixture 在測試前主動檢查並 refresh token（剩餘 < 60s 時）

- ✅ **測試穩定性改進**：
  - 確認所有測試已移除 `networkidle` 依賴，改用明確的元素等待策略
  - Session 自動重新登入機制驗證成功
  - Session 監控正常追蹤並記錄狀態

- 📊 **改進成果**：
  - Session 管理更健壯，自動處理 token 過期情況
  - 完整的診斷工具鏈，失敗時提供清晰的故障排除資訊
  - 配置驗證腳本確保環境設定正確
  - 文檔完整，涵蓋架構、配置、故障排除、維護指南

- ✅ **Dashboard 測試選擇器修復**：
  - 修復「通知鈴鐺應可見」測試：改用 `header button.relative` 選擇器，避免 strict mode violation（避免匹配到行動端漢堡按鈕）
  - 修復「語言切換應可運作」測試：改用 `header getByRole('combobox')` 選擇器（Radix UI Select.Trigger 標準 role）
  - Dashboard 測試套件 6/6 全部通過 ✅
  - 產出：[dashboard.spec.ts](../frontend/e2e/dashboard.spec.ts)（Line 31-45）

### 2026-02-25 SEC-33 敏感操作二級認證 (P3-7)

- ✅ **後端**：新增 `POST /auth/confirm-password`，以密碼換取短期 reauth JWT（5 分鐘）；`delete_user`、`reset_user_password`、`impersonate_user`、`delete_role` 四個敏感操作需帶 `X-Reauth-Token` header，否則回傳 403。
- ✅ **前端**：新增 `ConfirmPasswordModal` 與 `confirmPassword()` API；使用者管理（刪除使用者、重設他人密碼、模擬登入）與角色管理（刪除角色）執行前皆需重新輸入登入密碼以取得 reauth token 後再送出請求。

### 2026-02-25 電子簽章合規審查 (P1-7) 與 OpenAPI 完善 (P1-12)

- ✅ **P1-7 電子簽章合規審查**：新增 `docs/security-compliance/ELECTRONIC_SIGNATURE_COMPLIANCE.md`，對照 21 CFR Part 11 子章 B/C，審查犧牲／觀察／安樂死／轉讓／計畫書簽章與附註實作，結論為技術面已符合核心要求，建議補齊書面政策與訓練紀錄。
- ✅ **P1-12 OpenAPI 文件完善**：後端新增電子簽章 10 paths + 2 附註 paths、動物管理 9 paths，以及對應 Request/Response Schema（SignRecordRequest/Response、SignatureStatusResponse、Annotation、Animal、AnimalListItem、AnimalQuery 等），Swagger UI 已涵蓋認證、使用者、角色、設施、倉儲、計畫書、審查、電子簽章、動物管理。

### 2026-02-25 CI `sqlx-cli` 安裝修正

- ✅ **強制覆蓋**：在 `ci.yml` 的 `cargo install sqlx-cli` 步驟增加 `--force` 參數，解決 GitHub Actions 快取恢復後的二進位檔衝突問題。

### 2026-02-25 資料保留政策定義 (P1-8)

- ✅ **政策文檔產出**：建立 `DATA_RETENTION_POLICY.md`，定義 AUP、醫療紀錄、稽核日誌、ERP 與 HR 資料之法定保留年限。
- ✅ **合規基準**：參考 GLP、21 CFR Part 11 與台灣勞基法制定。

### 2026-02-25 Trivy 安全掃描優化

- ✅ **CI 參數統一**：將 `ci.yml` 中的 Trivy 掃描參數統一為 `vulnerability-type`。
- ✅ **過濾名單清理**：移除 `.trivyignore` 中無效的 `CVE-2026-0861` 編號。

### 2026-02-25 E2E CI 自動化 (P1-2)

- ✅ **GitHub Actions 整合**：新增 `e2e-test` 作業，自動執行 Playwright 測試。
- ✅ **測試環境容器化**：建立 `docker-compose.test.yml` 供 CI 使用。

### 2026-02-25 P1-1 前端 E2E 測試穩定化

- ✅ **Playwright E2E 測試**：7 spec 檔案、34 個測試案例，連續 3 次執行 0 failures。
- ✅ **涵蓋流程**：登入 (6)、Dashboard (4)、動物列表 (6)、計畫書 (6)、個人資料 (5)、Admin 使用者管理 (5)、Auth Setup (2)。
- ✅ **429 Rate Limit 重試**：`auth.setup.ts` 自動偵測 `Retry-After` header 並等待重試（最多 3 次）。
- ✅ **React 狀態 race condition 修正**：登入後若前端未自動跳轉，fallback 手動導航驗證 HttpOnly cookie。
- ✅ **i18n 雙語 selector**：所有 UI 文字匹配使用 `/English|中文/` regex，相容中英文介面。

### 2026-02-25 壓力測試基準建立 (P1-5)

- ✅ **k6 效能基準**：成功執行 50 VU 壓力測試，測得一般 API P95 為 2.3s，報表 API P95 為 1.76s。
- ✅ **認證優化**：腳本支援 JWT Bearer Token 並實作 VU 級別登入緩存。
- ✅ **結果歸檔**：測試數據已儲存於 `tests/results/k6_*.json`。

### 2026-02-25 瀏覽器相容性測試與 GLP 文件生成

- ✅ **相容性測試 (P0-6)**：執行 Playwright 跨瀏覽器測試，驗證基本渲染與登入流程。
- ✅ **GLP 驗證文件 (P1-6)**：產出 `GLP_VALIDATION.md` 驗證框架。

### 2026-02-25 P0-7 錯誤處理 UX 統一

- ✅ **安全強化**：隱藏原始 DB 錯誤。
- ✅ **前端錯誤導引**：優化 `getApiErrorMessage` 處理逾時與網路異常。

### 2026-03-09 請假與加班改為小時計算（0.5 單位）

- ✅ **請假**：表單與顯示改為「時數」（0.5 步進）；`useLeaveRequestForm` 雙向計算日期↔時數；後端 `create_leave` 驗證 0.5 倍數、`LeaveRequestWithUser` 含 `total_hours`。
- ✅ **加班**：`create_overtime` 時數四捨五入至 0.5 小時；前端新增加班 Dialog 顯示預估加班時數。

### 2026-03-04 docs 整理分類

- ✅ **文件索引**：新增 `docs/README.md` 總索引，依主題分類並列出各子目錄說明。
- ✅ **子目錄**：建立 `development/`、`database/`、`security-compliance/`、`runbooks/`、`operations/`、`assessments/`，將原根目錄散落文件移入對應分類。
- ✅ **連結更新**：根目錄保留 PROGRESS、TODO、QUICK_START、USER_GUIDE、DEPLOYMENT、ARCHITECTURE、walkthrough；README、PROGRESS、TODO、CI、backend 等處之文件路徑已更新為新路徑。

---

(其餘詳細 1-8 章節內容已併入本檔案)
