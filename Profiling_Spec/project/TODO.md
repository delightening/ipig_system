# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-13

## 🚧 進行中與近期事項

### 優先事項
- [x] **計畫變更（Amendment）整合測試撰寫** - `tests/test_amendment_full.py` 14 步驟（Minor/Major 兩條路線）
- [x] **狀態變更至「審查中」顯示審查委員** - `protocol.rs` change_status 記錄審查委員姓名到 remark 與 extra_data

### 系統管理與報表
- [🔶] PDF 報表生成 - 目前分頁邏輯不盡人意，規劃改為每個 Session 獨立分頁
- [ ] 行動端適配 (響應式設計)

---

## 📋 AUP 系統優化建議 (2026-02-06)

### AUP 表單與內容優化
- [ ] 參考文獻格式參考農業部提供之格式

---

## 📋 實驗動物管理優化

- [ ] 疼痛評估紀錄時間軸檢視
- [ ] 獸醫師建議通知機制進階設定
- [ ] 血液檢查組合後台管理元件 (`BloodTestPanelManager.tsx`) - 管理 Panel 組合的 CRUD 頁面（後端 API 已完成，前端管理頁面尚未建立）
- [ ] **資料分析模組** - 實驗動物管理子系統中的「資料分析」功能
  - [ ] 血液檢查結果統計與趨勢分析（依動物、實驗、日期區間）
  - [ ] 血液數值異常標記與警示
  - [ ] 圖表視覺化（折線圖/盒鬚圖等）
  - [ ] 資料匯出（CSV/Excel）
  - 📌 **注意：** 血液檢查流程定位
    - 動物管理系統：記錄哪隻動物做了什麼檢查（檢查項目與結果）
    - ERP 系統：以專案與日期區間來區分血液檢查的費用（成本管理）✅ `BloodTestCostReportPage`
    - 資料分析模組：對血液檢查結果進行後續分析與視覺化（本項目）

---

## 📋 系統開發與維護 (技術債)

- [x] 表單驗證統一：建立 `validation.ts` Zod schema 統一驗證模組
- [x] 錯誤處理統一：建立 `useApiError.ts` hook + `ErrorBoundary.tsx` 元件
- [x] Loading 狀態統一：建立 `Skeleton.tsx` 骨架屏 + `LoadingOverlay.tsx`
- [x] 前端 TypeScript 型別統一：建立 `types/common.ts` + `types/index.ts` 統一匯出

---

## 📋 下一步優化建議 (預計)

- [ ] 實作稽核日誌匯出 CSV/PDF 功能
- [ ] 增加敏感資料（如密碼修改、權限變更）的二級審計
- [ ] 優化安全警報的郵件/通知即時推送
- [ ] 計畫詳情頁面「活動紀錄」標籤性能優化 (若紀錄過多時的分頁處理)

---

## 📋 v2.0 遠程規劃

### iPig ERP 系統
- [ ] 條碼掃描功能（行動裝置支援）
- [ ] 進階成本法（FIFO）
- [ ] 批號效期到期提醒進階設定
- [ ] 作廢已核准單據：沖銷機制（Reversal Document）
- [ ] 庫位管理（Bin Location）
- [ ] 與會計/ERP API 對接

---

## ✅ 已完成項目紀錄 (最近)

### 2026-02-13
- [x] 新增「血液檢查項目管理」前端頁面（ERP 基礎資料模組）
  - 前端 `BloodTestTemplatesPage.tsx` CRUD 頁面（64 個模板，啟用/停用管理）
  - 已加入 ERP 基礎資料 tab（`ErpPage.tsx`）
- [x] 血液檢查組合 (Panel) 快速勾選功能
  - `026_blood_test_panels.sql` migration（14 組 + 64 筆關聯）
  - 後端 Panel CRUD API（6 個端點）
  - 前端 `BloodTestTab.tsx` Toggle 按鈕列 UI
  - `test_blood_panel.py` 整合測試 28/28 通過
- [x] ERP 站內通知系統整合
- [x] 血液檢查移除審核步驟（自動標記 completed）
- [x] 資料庫 migration 整合（008-029 合併）
- [x] HR 測試修復
- [x] 修復 API 404 錯誤（路由註冊問題）
- [x] 修復後端容器重啟問題
- [x] **狀態歷程「創建」活動類型修正** - `protocol.rs` 建立計畫時已使用 `ProtocolActivityType::Created`，`record_status_change` 中 `Draft => Created` 映射正確
- [x] **審查流程調整：先獸醫審查，再委員審查** - `VET_REVIEW` 狀態完整實作（狀態驗證、自動指派獸醫、獸醫審查表）
- [x] **執秘審查加入審查意見回覆功能** - `IACUC_STAFF` 已有 `aup.review.reply` 權限，`reply_review_comment` handler 完整實作含草稿回覆流程
- [x] **Amendment 前端頁面開發** - `AmendmentsTab.tsx`（建立/提交/列表）+ `MyAmendmentsPage.tsx`（篩選/列表）

### 2026-02-12
- [x] 修復後端 `reply_comment` UTF-8 中文字元切割 panic
- [x] 修復 Animal 測試 `deceased` → `completed` status
- [x] 修復 AUP 測試 `get_status()` JSON 路徑錯誤
- [x] 動物狀態簡化（6 種 → 3 種：Unassigned / InExperiment / Completed）
- [x] 動物時間軸增強（建立日期、體重紀錄、犧牲標記）
- [x] 動物記錄 ID 從 UUID 遷移至 SERIAL INTEGER

### 2026-02-11
- [x] 修復登入成功紀錄失效問題 (SQL INET)
- [x] 補全 AUP 計畫書編輯、評論、指派與核准活動紀錄
- [x] 優化非上班時間登入警報判斷點 (18:00-08:00)
- [x] 整合實驗動物管理活動紀錄（23 個寫入操作）至安全審計系統
- [x] AUP 完整流程測試 (`tests/test_aup_full.py`) 14/14
- [x] ERP 完整流程測試 (`tests/test_erp_full.py`) 9/9
- [x] 動物管理系統完整測試 (`tests/test_animal_full.py`) 21/21
- [x] 修復角色權限（WAREHOUSE_MANAGER / ADMIN_STAFF / EXPERIMENT_STAFF）
- [x] 修復空 Email 地址發送問題

### 2026-02-09 ~ 2026-02-10
- [x] 實作「多帳號腳本登入偵測」與全域 Critical 警報
- [x] 新增「同時大量登入」偵測機制與 UI 標籤
- [x] 安全警報偵測與顯示修復
- [x] 計畫書管理表格排序功能
- [x] 修復審查人員列表無法顯示的問題
- [x] AUP 狀態歷程顯示修復與優化
- [x] 修復 PI 無法刪除草稿權限問題
- [x] AUP 手術計畫書標籤翻譯同步
- [x] AUP Placeholder 翻譯標準化
- [x] 審查委員強制發表意見檢查
- [x] AUP 審查流程多輪往返功能（PRE_REVIEW_REVISION_REQUIRED / VET_REVISION_REQUIRED）

---

## 變更紀錄

| 日期 | 內容 |
|------|------|
| 2026-02-13 | 釐清血液檢查流程定位（動物記錄/ERP費用/資料分析），新增「資料分析模組」待辦 |
| 2026-02-13 | Amendment 整合測試、UNDER_REVIEW 審查委員記錄、前端技術債基礎建設（Zod 驗證、錯誤處理、骨架屏、型別統一） |
| 2026-02-13 | 調查並標記已完成項目：狀態歷程 Created 修正、獸醫審查流程、執秘回覆功能、Amendment 前端；新增 Amendment 測試為優先事項 |
| 2026-02-13 | 血液檢查項目管理頁面、Panel 快速勾選、ERP 通知整合、移除血液檢查審核、migration 合併、HR 測試修復、API 404 修復、容器重啟修復 |
| 2026-02-12 | 整合測試 Bug 修復、動物狀態簡化、時間軸增強、動物記錄 ID 遷移 |
| 2026-02-11 | 安全審計強化、整合測試腳本建立、角色權限修復、空 Email 修復 |
| 2026-02-10 | 多帳號/大量登入偵測、安全警報修復 |
| 2026-02-09 | 審查人員列表修復、狀態歷程修復、審查委員強制意見、多輪往返功能、翻譯標準化 |
| 2026-02-08 | AUP 歷程紀錄增強、動物列表體重排序 |
| 2026-02-03 | Login As 功能、版本比較強化、Dashboard 錯誤處理、AUP 國際化 |
| 2026-02-02 | UUID 遷移、Amendment 後端、安樂死工作流程 UI |