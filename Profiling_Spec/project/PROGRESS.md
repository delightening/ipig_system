# 豬博士 iPig 系統專案進度評估表

> **評估日期：** 2026-02-23  
> **規格版本：** v7.0  
> **評估標準：** ✅ 完成 | 🔶 部分完成 | 🔴 未開始 | ⏸️ 暫緩

## 📑 目錄

| # | 章節 | 說明 |
|---|------|------|
| - | [總體進度概覽](#-總體進度概覽) | 各子系統完成度摘要 |
| 1 | [共用基礎架構](#1-共用基礎架構) | 認證授權、使用者管理、角色權限、Email、稽核 |
| 2 | [AUP 提交與審查系統](#2-aup-提交與審查系統) | 計畫書管理、審查流程、附件、我的計劃 |
| 3 | [iPig ERP (進銷存管理系統)](#3-ipig-erp-進銷存管理系統) | 基礎資料、採購、銷售、倉儲、報表 |
| 4 | [實驗動物管理系統](#4-實驗動物管理系統) | 動物管理、紀錄、血液檢查、匯出、GLP |
| 5 | [通知系統](#5-通知系統) | Email 通知、站內通知、排程任務 |
| 6 | [HR 人事管理系統](#6-hr-人事管理系統) | 特休、考勤、Google Calendar |
| 7 | [資料庫 Schema 完成度](#7-資料庫-schema-完成度) | Migration 清單 |
| 8 | [版本規劃](#8-版本規劃) | v1.0 / v1.1 里程碑 |
| 9 | [更新紀錄](#9-更新紀錄) | 最新變更與待處理問題 |
| 10 | [結論](#10-結論) | 整體完成度總結 |

---

## 📊 總體進度概覽

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

| 面向 | 現況 | 目標 | 狀態 |
|------|------|------|------|
| **測試覆蓋率** | Rust 119 tests、Python 8 模組、前端 0 | 核心邏輯 ≥ 80%、E2E 關鍵流程 100% | 🔶 |
| **可觀測性** | 無 /health、無 /metrics、無錯誤監控 | 健康檢查 + Prometheus + Sentry | 🔴 |
| **備份 / DR** | pg_dump + rsync ✅、復原未演練 | 復原 SOP + 上傳檔案備份 + 加密 | 🔶 |
| **安全性** | 14 SEC 已完成、cargo audit CI ✅ | npm audit CI + Pentest + Trivy | 🔶 |
| **GLP 合規** | 電子簽章/稽核/HMAC 已實作 | CSV 驗證文件 + 資料保留政策 | 🔴 |
| **效能基準** | Bundle 242KB ✅、Lazy Loading ✅ | 壓力測試 + Brotli + FCP/LCP 達標 | 🔶 |
| **文件** | Swagger 83/293 (28%)、QUICK_START.md | Swagger ≥90%、操作/維運手冊 | 🔴 |
| **UX / 相容性** | 行動端適配 ✅、Skeleton ✅ | 瀏覽器相容性測試 + 錯誤 UX 統一 | 🔶 |

**上線準備度估算：約 60%（功能開發完成，品質/合規/文件尚待補齊）**



---

## 1. 共用基礎架構

### 1.1 認證與授權

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 登入 (JWT) | ✅ | ✅ | ✅ | 完整實作 |
| 登出 | ✅ | ✅ | ✅ | |
| Token 刷新 | ✅ | ✅ | ✅ | |
| 忘記密碼 | ✅ | ✅ | ✅ | 含 Email 發送 |
| 重設密碼 | ✅ | ✅ | ✅ | Token 驗證 |
| 修改自己密碼 | ✅ | ✅ | ✅ | MainLayout 內對話框 |
| 首次登入強制變更密碼 | ✅ | ✅ | ✅ | ForceChangePasswordPage |
| 密碼強度檢查 | ✅ | ✅ | ✅ | 前端即時驗證 |
| 帳號鎖定（暴力破解防護） | ✅ | ✅ | ✅ | 15 分鐘內 5 次失敗即封鎖 (SEC-20, 2026-02-15) |
| JWT Secret 強度驗證 | ✅ | - | ✅ | 啟動時強制 ≥ 32 字元 (SEC-21, 2026-02-15) |
| JWT 黑名單主動撤銷 | ✅ | - | ✅ | 記憶體 + DB 持久化（SEC-33, 2026-02-22 升級），重啟不遺失 |
| JWT 有效期 15 分鐘 | ✅ | ✅ | ✅ | `jwt_expiration_seconds` 預設 900 秒 (SEC-25, 2026-02-15) |
| CSRF 防護 | ✅ | ✅ | ✅ | Double Submit Cookie + `X-CSRF-Token` header (SEC-24, 2026-02-15) |
| Session 併發限制 | ✅ | - | ✅ | 每用戶上限 5 個活躍 session (SEC-28, 2026-02-15) |
| 安全回應標頭 | ✅ | ✅ | ✅ | API 層 + nginx CSP (SEC-27, 2026-02-15) |
| 開發帳號安全防護 | ✅ | - | ✅ | 正式環境拒絕啟動 + 密碼改環境變數 (SEC-26, 2026-02-15) |
| 角色切換 (Login As) | ✅ | ✅ | ✅ | 管理員可切換身分 |


### 1.2 使用者管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 使用者列表 | ✅ | ✅ | ✅ | UsersPage |
| 建立使用者（私域註冊） | ✅ | ✅ | ✅ | 含 Email 通知 |
| 編輯使用者 | ✅ | ✅ | ✅ | 含學經歷欄位 |
| 停用/啟用帳號 | ✅ | ✅ | ✅ | |
| 重設密碼 | ✅ | ✅ | ✅ | 管理員重設 |
| 指派角色 | ✅ | ✅ | ✅ | |
| 個人資歷維護 | ✅ | ✅ | ✅ | 用於 AUP 第 8 節 |


### 1.3 角色權限管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 角色列表 | ✅ | ✅ | ✅ | RolesPage |
| 建立角色 | ✅ | ✅ | ✅ | |
| 編輯角色 | ✅ | ✅ | ✅ | |
| 角色權限指派 | ✅ | ✅ | ✅ | |
| 權限列表 | ✅ | ✅ | ✅ | |
| RBAC 中間件 | ✅ | ✅ | ✅ | 後端強制檢查 |

### 1.4 Email 服務

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| Gmail SMTP 整合 | ✅ | - | ✅ | lettre 套件 |
| 帳號開通通知信 | ✅ | - | ✅ | HTML + 純文字 |
| 密碼重設通知信 | ✅ | - | ✅ | 含重設連結 |
| 密碼變更成功通知 | ✅ | - | ✅ | |

### 1.5 稽核紀錄

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 稽核紀錄寫入 | ✅ | - | ✅ | 自動記錄 |
| 稽核紀錄查詢 | ✅ | ✅ | ✅ | AuditLogsPage |
| 安全警報系統 | ✅ | ✅ | ✅ | 自動偵測異常登入（GeoIP 國家層級）與暴力破解 (2026-02-15 GeoIP 升級) |
| 安全警報排序 | - | ✅ | ✅ | 支援按時間/類型/嚴重程度/狀態排序 (2026-02-16) |
| 篩選功能 | ✅ | ✅ | ✅ | |

---

## 2. AUP 提交與審查系統

### 2.1 計畫書管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 計畫書列表 | ✅ | ✅ | ✅ | ProtocolsPage |
| 列表排序功能 | - | ✅ | ✅ | 支援多欄位排序 |
| 建立計畫書（草稿） | ✅ | ✅ | ✅ | ProtocolEditPage |
| 編輯計畫書 | ✅ | ✅ | ✅ | |
| 檢視計畫書 | ✅ | ✅ | ✅ | ProtocolDetailPage |
| 提交計畫書 | ✅ | ✅ | ✅ | |
| 計畫書狀態機 | ✅ | ✅ | ✅ | 12 種狀態 |
| 狀態變更 | ✅ | ✅ | ✅ | 依角色權限 |
| 版本控管 | ✅ | ✅ | ✅ | 含版本內容檢視對話框 |
| 狀態歷程 | ✅ | ✅ | ✅ | 增強版：詳細活動日誌 (誰/何時/何事) |
| 版本比較 | ✅ | ✅ | ✅ | 支援巢狀欄位差異顯示 |

| 草稿刪除（軟刪除） | ✅ | ✅ | ✅ | DELETED 狀態，刪除後從 UI 完全消失（不顯示"已刪除"狀態） |

### 2.2 審查流程

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 指派審查人員 | ✅ | ✅ | ✅ | 審查人員 Tab + 指派對話框 |
| 審查意見列表 | ✅ | ✅ | ✅ | 含審查者資訊 |
| 新增審查意見 | ✅ | ✅ | ✅ | 新增意見對話框 |
| 解決審查意見 | ✅ | ✅ | ✅ | 標記已解決按鈕 |
| 核准/否決 | ✅ | ✅ | ✅ | |

### 2.3 附件管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 上傳附件 | ✅ | ✅ | ✅ | FileService + 多類型支援 |
| 下載附件 | ✅ | ✅ | ✅ | 安全路徑檢查 |
| 附件列表 | ✅ | ✅ | ✅ | 附件 Tab 完整 |
| 刪除附件 | ✅ | ✅ | ✅ | 權限控制 |

### 2.4 我的計劃（外部使用者）

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 我的計劃列表 | ✅ | ✅ | ✅ | MyProjectsPage |
| 計劃詳情 | ✅ | ✅ | ✅ | MyProjectDetailPage |
| 申請表 Tab | ✅ | ✅ | ✅ | 完整顯示 |
| 動物紀錄 Tab | ✅ | ✅ | ✅ | |

---

## 3. iPig ERP (進銷存管理系統)

### 3.1 基礎資料

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 倉庫管理 | ✅ | ✅ | ✅ | WarehousesPage |
| 產品管理 | ✅ | ✅ | ✅ | ProductsPage |
| 產品新增 | ✅ | ✅ | ✅ | CreateProductPage |
| 產品詳情 | ✅ | ✅ | ✅ | ProductDetailPage |
| SKU 生成 | ✅ | ✅ | ✅ | 互動式生成 |
| 供應商/客戶管理 | ✅ | ✅ | ✅ | PartnersPage |
| 客戶分類 | ✅ | ✅ | ✅ | customer_category 欄位 (2026-02-14) |
| 產品類別 | ✅ | ✅ | ✅ | |

### 3.2 採購流程

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 採購單（PO） | ✅ | ✅ | ✅ | DocumentsPage |
| 採購入庫（GRN） | ✅ | ✅ | ✅ | |
| 採購退貨（PR） | ✅ | ✅ | ✅ | |
| 單據編輯 | ✅ | ✅ | ✅ | DocumentEditPage |
| 單據詳情 | ✅ | ✅ | ✅ | DocumentDetailPage |
| 送審/核准 | ✅ | ✅ | ✅ | |

### 3.3 銷售流程

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 銷售單（SO） | ✅ | ✅ | ✅ | |
| 銷售出庫（DO） | ✅ | ✅ | ✅ | |

### 3.4 倉儲作業

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 庫存查詢 | ✅ | ✅ | ✅ | InventoryPage |
| 庫存流水 | ✅ | ✅ | ✅ | StockLedgerPage |
| 調撥單（TR） | ✅ | ✅ | ✅ | |
| 盤點單（STK） | ✅ | ✅ | ✅ | |
| 調整單（ADJ） | ✅ | ✅ | ✅ | |
| 低庫存提醒 | ✅ | ✅ | ✅ | |

### 3.5 報表

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 庫存現況報表 | ✅ | ✅ | ✅ | StockOnHandReportPage |
| 庫存流水報表 | ✅ | ✅ | ✅ | StockLedgerReportPage |
| 採購明細報表 | ✅ | ✅ | ✅ | PurchaseLinesReportPage |
| 銷售明細報表 | ✅ | ✅ | ✅ | SalesLinesReportPage |
| 成本摘要報表 | ✅ | ✅ | ✅ | CostSummaryReportPage |
| 血液檢查費用報表 | ✅ | ✅ | ✅ | BloodTestCostReportPage（專案+日期+實驗室篩選） |
| CSV 匯出 | ✅ | ✅ | ✅ | |

---

## 4. 實驗動物管理系統

### 4.1 動物管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 動物列表 | ✅ | ✅ | ✅ | AnimalsPage |
| 依狀態篩選 | ✅ | ✅ | ✅ | Tab 切換 |
| 依欄位分組檢視 | ✅ | ✅ | ✅ | 分組卡片視圖 |
| 排序功能 | - | ✅ | ✅ | 支援耳號、進場日期、體重排序 |
| 新增動物 | ✅ | ✅ | ✅ | |
| 編輯動物 | ✅ | ✅ | ✅ | AnimalEditPage |
| 動物詳情（7 Tab） | ✅ | ✅ | ✅ | AnimalDetailPage |
| 批次分配至計劃 | ✅ | ✅ | ✅ | |
| 批次進入實驗 | ✅ | ✅ | ✅ | |
| 匯入基本資料 | ✅ | ✅ | ✅ | ImportDialog，支援 Excel/CSV |
| 匯入體重 | ✅ | ✅ | ✅ | ImportDialog，支援 Excel/CSV |
| 下載匯入範本（Excel） | ✅ | ✅ | ✅ | 基本資料/體重範本 |
| 下載匯入範本（CSV） | ✅ | ✅ | ✅ | 基本資料/體重 CSV 範本 |

### 4.2 動物紀錄 - 觀察試驗紀錄

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 紀錄列表 | ✅ | ✅ | ✅ | |
| 新增紀錄 | ✅ | ✅ | ✅ | ObservationFormDialog 完整表單 |
| 編輯紀錄 | ✅ | ✅ | ✅ | |
| 刪除紀錄（軟刪除） | ✅ | ✅ | ✅ | |
| 複製紀錄 | ✅ | ✅ | ✅ | 含確認對話框 |
| 版本歷史 | ✅ | ✅ | ✅ | VersionHistoryDialog |
| 獸醫師已讀標記 | ✅ | ✅ | ✅ | |
| 獸醫師建議 | ✅ | ✅ | ✅ | VetRecommendationDialog |

### 4.3 動物紀錄 - 手術紀錄

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 紀錄列表 | ✅ | ✅ | ✅ | |
| 新增紀錄 | ✅ | ✅ | ✅ | SurgeryFormDialog 完整表單 |
| 編輯紀錄 | ✅ | ✅ | ✅ | |
| 刪除紀錄 | ✅ | ✅ | ✅ | |
| 複製紀錄 | ✅ | ✅ | ✅ | 含確認對話框 |
| 版本歷史 | ✅ | ✅ | ✅ | VersionHistoryDialog |
| 生理數值記錄 | ✅ | ✅ | ✅ | Repeater 多筆輸入 |
| 獸醫師建議 | ✅ | ✅ | ✅ | VetRecommendationDialog |

### 4.4 動物紀錄 - 其他紀錄

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 體重紀錄 CRUD | ✅ | ✅ | ✅ | |
| 疫苗/驅蟲紀錄 CRUD | ✅ | ✅ | ✅ | |
| 犧牲/採樣紀錄 | ✅ | ✅ | ✅ | |
| 病理組織報告 | ✅ | ✅ | ✅ | 含上傳功能 |
| 安樂死/猝死狀態管理 | ✅ | ✅ | ✅ | 6 狀態 + 轉換驗證 + 猝死登記 API (2026-02-16) |
| 轉讓流程 6 步 API | ✅ | ✅ | ✅ | 後端 8 路由 + DB + 前端 TransferTab + 資料隔離 (2026-02-16) |

### 4.5 血液檢查

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 血液檢查 CRUD | ✅ | ✅ | ✅ | BloodTestTab |
| 檢驗模板管理 | ✅ | ✅ | ✅ | 64 個預設模板 |
| 檢驗組合管理 (Panel) | ✅ | ✅ | ✅ | 14 組預設組合 + CRUD API (2026-02-13) |
| 組合快速勾選 Toggle UI | - | ✅ | ✅ | BloodTestTab Toggle 按鈕列 (2026-02-13) |
| 組合停用/恢復 | ✅ | - | ✅ | 軟刪除 (2026-02-13) |
| 整合測試 | ✅ | - | ✅ | test_blood_panel.py 28/28 通過 (2026-02-13) |

### 4.6 資料匯出

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 單一動物病歷匯出 | ✅ | ✅ | ✅ | ExportDialog |
| 計劃病歷批次匯出 | ✅ | ✅ | ✅ | ExportDialog |
| 觀察試驗紀錄匯出 | ✅ | ✅ | ✅ | |
| 手術紀錄匯出 | ✅ | ✅ | ✅ | |

### 4.7 動物來源管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 來源列表 | ✅ | ✅ | ✅ | AnimalSourcesPage |
| 新增來源 | ✅ | ✅ | ✅ | |
| 編輯來源 | ✅ | ✅ | ✅ | |
| 刪除來源 | ✅ | ✅ | ✅ | |

### 4.8 GLP 合規功能（更新 2026-01-19）

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 軟刪除 + 刪除原因 | ✅ | ✅ | ✅ | 刪除時需提供原因 |
| 變更原因記錄 | ✅ | ✅ | ✅ | change_reasons 表 |
| 審計日誌增強 | ✅ | ✅ | ✅ | old_value, new_value |
| 電子簽章 | ✅ | ✅ | ✅ | SignatureService 完整實作 |
| 記錄鎖定機制 | ✅ | ✅ | ✅ | 簽章後自動鎖定 |
| 附註/更正功能 | ✅ | ✅ | ✅ | AnnotationService 完成 |

### 4.9 資料匯出功能

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| AUP 計畫書 PDF 匯出 | ✅ | ✅ | ✅ | 全 9 節完整呈現 |
| 動物病歷 PDF 匯出 | ✅ | ✅ | ✅ | 含觀察/手術紀錄 |

### 4.10 資料分析

> 📌 **血液檢查流程定位說明：**
> - **動物管理系統**：記錄哪隻動物做了什麼檢查（檢查項目與結果數值）
> - **ERP 系統**：以專案與日期區間來區分血液檢查的費用（成本管控，不涉及結果分析）
> - **資料分析模組**：對血液檢查結果進行後續統計分析與視覺化（本區塊）

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 血液檢查結果統計 | ✅ | ✅ | ✅ | BloodTestAnalysisPage 依動物/專案/日期區間分析 (2026-02-15) |
| 異常值標記與警示 | ✅ | ✅ | ✅ | 異常值醒目標記 + 警示區塊 (2026-02-15) |
| 趨勢圖表視覺化 | ✅ | ✅ | ✅ | Recharts 折線圖 + 自訂盒鬚圖 (2026-02-15) |
| 分析結果匯出 | ✅ | ✅ | ✅ | CSV + Excel (xlsx) 匯出 (2026-02-15) |

---

## 5. 通知系統

### 5.1 Email 通知

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 帳號開通通知 | ✅ | - | ✅ | HTML + 純文字格式 |
| 密碼重設通知 | ✅ | - | ✅ | 含重設連結 |
| 密碼變更通知 | ✅ | - | ✅ | |
| 計畫提交通知 | ✅ | - | ✅ | 通知 IACUC_STAFF |
| 計畫狀態變更通知 | ✅ | - | ✅ | 通知 PI 及相關人員 |
| 審查指派通知 | ✅ | - | ✅ | 通知被指派的審查人員 |
| 獸醫師建議通知 | ✅ | - | ✅ | 通知 EXPERIMENT_STAFF |
| 低庫存提醒 | ✅ | - | ✅ | 排程 + 手動觸發 |
| 效期提醒 | ✅ | - | ✅ | 排程 + 手動觸發 |

### 5.2 站內通知

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 通知資料表 | ✅ | - | ✅ | migration 005 新增欄位 |
| 通知列表 API | ✅ | - | ✅ | handler 完整 |
| 未讀數量 | ✅ | ✅ | ✅ | Header 顯示 |
| 標記已讀 | ✅ | ✅ | ✅ | |
| 標記全部已讀 | ✅ | ✅ | ✅ | |
| Header 通知圖示 | - | ✅ | ✅ | MainLayout |
| 通知下拉選單 | - | ✅ | ✅ | 含點擊導航 |
| 通知設定 | ✅ | ✅ | ✅ | SettingsPage 整合 |
| 通知路由管理 | ✅ | ✅ | ✅ | `notification_routing` 表、CRUD API、前端獨立管理頁面 `NotificationRoutingPage.tsx`（Table + 分類下拉 + 啟停 Switch） |

### 5.3 排程任務

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 低庫存每日檢查 | ✅ | - | ✅ | 每日 08:00 執行 |
| 效期每日檢查 | ✅ | - | ✅ | 每日 08:00 執行 |
| 過期通知清理 | ✅ | - | ✅ | 每週日 03:00 執行 |
| 手動觸發檢查 | ✅ | - | ✅ | Admin API 可手動觸發 |

---

## 6. HR 人事管理系統

### 6.1 特休管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 特休額度管理 | ✅ | ✅ | ✅ | 僅管理員可存取 |
| 員工到職日期 | ✅ | ✅ | ✅ | hire_date 欄位 |
| 週年制特休計算 | ✅ | ✅ | ✅ | 到職日 + 2 年到期 |
| 未休補償追蹤 | ✅ | ✅ | ✅ | compensation_status |
| 手動授予特休 | ✅ | ✅ | ✅ | 可指定天數 |

### 6.2 考勤管理

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 考勤統計報表 | ✅ | ✅ | ✅ | 排除管理員帳號 |
| 請假餘額顯示 | ✅ | ✅ | ✅ | Dashboard Widget |
| 打卡 IP 限制 | ✅ | ✅ | ✅ | CIDR 白名單 + 403 友善提示 (2026-02-17) |

### 6.3 Google Calendar 整合

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| Google OAuth 連接 | ✅ | ✅ | ✅ | 持久化連接狀態 |
| 行事曆事件同步 | ✅ | ✅ | ✅ | 自動同步 |
| 錯誤處理優化 | ✅ | ✅ | ✅ | 未連接時顯示友善訊息 |


---

## 7. 資料庫 Schema 完成度

| Migration | 內容 | 狀態 |
|-----------|------|:---:|
| 001_core_schema.sql | 自訂型別（enum）、使用者/角色/權限/通知/附件基礎表、使用者偏好設定 | ✅ |
| 002_core_permissions.sql | 權限定義 + 角色預設權限指派 + 動物擴展權限 | ✅ |
| 003_animal_management.sql | 動物管理全系統（sources/animals/records/blood test/sudden death/transfer） | ✅ |
| 004_aup_system.sql | AUP 計畫書/審查/附件/變更申請/系統設定/獸醫審查/活動歷程/往返歷史 | ✅ |
| 005_hr_system.sql | HR 考勤/特休/Google Calendar 整合 | ✅ |
| 006_audit_system.sql | 稽核日誌（分區）/登入事件/安全警報 | ✅ |
| 007_erp_warehouse.sql | ERP 倉庫/產品/單據/庫存 | ✅ |
| 008_supplementary.sql | 通知路由規則 + 電子簽章 + 記錄附註 | ✅ |

> 📌 **2026-02-18 整合**：遷移從 10 個整合為 8 個。009（GPS 欄位）併入 005 CREATE TABLE、010（通知 seed）已存在於 008。`cargo build` 通過。

---


## 8. 版本規劃

### v1.1（已完成 2026-01-19）

- [x] 排程任務
- [x] 完整 Email 通知
- [x] 檔案上傳後端服務
- [x] 通知偏好設定前端
- [x] HR 特休管理系統（週年制、hire_date、未休補償）
- [x] GLP 合規功能（電子簽章、附註、記錄鎖定）
- [x] AUP 計畫書 PDF 匯出（全 9 節）
- [x] 協編者權限修正
- [x] 審查意見回覆功能
- [x] 計畫書頁面整合
- [x] Google Calendar 整合
- [x] 權限系統優化（審查員、實驗人員）
- [x] 資料庫 ERD 文件

### v1.0 MVP（完成度：100% ✅ 2026-01-08）

- [x] 共用基礎架構
- [x] 進銷存核心功能
- [x] 實驗動物管理完整表單
- [x] 資料匯出功能
- [x] 站內通知系統
- [x] 複製紀錄功能
- [x] 版本歷史檢視
- [x] 獸醫師建議功能
- [x] AUP 審查流程完整 UI（審查意見、指派、附件）
- [x] 檔案上傳後端服務
- [x] 通知偏好設定前端

---

## 9. 更新紀錄

### 待處理問題

✅ **全部已解決**（截至 2026-02-14）

1. ~~🔶 **狀態變更至「審查中」時未顯示審查委員**~~ → ✅ 已於 2026-02-13 修復，審查委員名單記錄至 remark + extra_data
2. ~~🔶 **計畫變更（Amendment）整合測試**~~ → ✅ 已於 2026-02-13 完成，`test_amendment_full.py` 14 步驟

### 最新更新 (2026-02-14)

- ✅ **安全性強化**
  - 審計日誌記錄客戶端真實 IP（解析 `X-Forwarded-For` 標頭，不再顯示 Proxy/Docker 內部 IP）
  - 移除系統預設管理員帳號密碼
  - 修復安全警報解決時的 422 錯誤
- ✅ **基礎設施升級**
  - Docker 映像升級：Node.js v20→v22、Rust 與 Nginx 版本固定
  - npm / Rust 套件 patch 更新
  - Docker 容器重建與重新部署
- ✅ **Session 管理強化** - 實作 heartbeat 機制，`last_activity_at` 即時反映使用者最近一次頁面活動與 IP
- ✅ **ERP 功能增強**
  - 客戶分類功能（`customer_category` 欄位、前後端 CRUD + 篩選/顯示）
  - 銷售單金額顯示成本價（從 `stock_ledger` 取得平均成本）
  - UOM 單位中文翻譯統一（重構 `formatUom` 至 `lib/utils.ts` 共用）
  - ERP 權限測試修復（`EXPERIMENT_STAFF` 建立銷售單）
- ✅ **AUP / Amendment 修復**
  - AUP Section 8 人員資料解析錯誤修正
  - Amendment 行变更項目翻譯鍵大小寫不一致修復
  - Amendment 列表管理員無法檢視全部記錄修復
- ✅ **Email 修復** - 文字顏色過淡修正、破圖修復、系統網址更新為 Cloudflare Tunnel 網址
- ✅ **帳號管理** - 建立帳號移除入職日期必填驗證

### 2026-02-13 以前

- ✅ ERP 血液檢查費用報表完成（`BloodTestCostReportPage.tsx`），支援專案/日期/實驗室篩選、摘要統計、CSV 匯出
- 📋 釐清血液檢查流程定位：動物管理記錄檢查 → ERP 管理費用 → 資料分析模組進行結果分析；新增「4.10 資料分析」規劃區塊
- ✅ Amendment 整合測試（`test_amendment_full.py`，Minor/Major 兩條路線 14 步驟）
- ✅ UNDER_REVIEW 狀態轉換記錄審查委員姓名（`protocol.rs` remark + extra_data）
- ✅ 前端技術債基礎建設：Zod 驗證（`validation.ts`）、API 錯誤 Hook（`useApiError.ts`）、ErrorBoundary、Skeleton 骨架屏、LoadingOverlay、TypeScript 型別統一匯出（`types/index.ts`）
- ✅ 血液檢查項目管理前端頁面（`BloodTestTemplatesPage.tsx`，64 個模板）
- ✅ Blood Test Panel 快速勾選：DB migration + 後端 CRUD 6 端點 + 前端 Toggle UI + 測試 28/28 通過
- ✅ ERP 站內通知系統整合
- ✅ 血液檢查移除審核步驟（自動 completed）
- ✅ 資料庫 migration 整合（008-029 合併）
- ✅ HR 測試修復
- ✅ 修復 API 404 錯誤、後端容器重啟問題
- ✅ 確認「創建」活動類型已正確使用 `ProtocolActivityType::Created`（經調查程式碼已修正）
- ✅ 確認獸醫審查節點（VET_REVIEW）已完整實作（狀態驗證 + 自動指派 + 審查表）
- ✅ 確認執秘（IACUC_STAFF）審查意見回覆功能已完成（含草稿回覆流程）
- ✅ 確認 Amendment 前端頁面已完成（`AmendmentsTab.tsx` + `MyAmendmentsPage.tsx`）

### 2026-02-12

- ✅ 修復 `reply_comment` UTF-8 字元切割 panic、Animal 測試 status、AUP 測試 JSON 路徑
- ✅ 動物狀態簡化（6 種 → 3 種）、時間軸增強、動物記錄 ID 從 UUID 遷移至 SERIAL
- ✅ 測試資料清理腳本（SQL + PowerShell + pytest 整合）
- ✅ 全部測試通過：AUP 14/14 ・ ERP 9/9 ・ Animal 21/21

### 2026-02-11

- ✅ 安全審計強化：登入紀錄修復、AUP 活動紀錄補全、非上班時間警報優化
- ✅ 整合測試腳本建立（AUP/ERP/Animal 三套 + 統一入口）
- ✅ 實驗動物管理 23 個寫入操作接入審計系統
- ✅ 角色權限修復（WAREHOUSE_MANAGER / ADMIN_STAFF / EXPERIMENT_STAFF）

### 2026-02-15 (下午)

- ✅ **P2 豬隻→動物命名全面重構**：
  - DB migration `012_rename_pig_to_animal.sql`：6 個 enum、10 個表、18 個索引、所有 `pig_id`→`animal_id` 欄位重命名
  - 後端 42+ Rust 檔案：models/handlers/services/routes 全面更新，路由 `/pigs`→`/animals`
  - 前端 47+ TypeScript 檔案：types/pages/components 重命名與內容更新
  - 測試 2 檔案更新（`test_animal_full.py`、`test_blood_panel.py`）
  - 前端 `npx tsc --noEmit` 編譯通過（0 錯誤）

### 2026-02-22

- ✅ 🔒 **安全性全面強化（8 項修復）**：
  - SEC-29 CSRF Cookie 加入 `Secure` flag（`csrf.rs` 依 `cookie_secure` 動態設定、`routes.rs` 改用 `from_fn_with_state`）
  - SEC-30 IP Header 信任策略（`real_ip.rs` 新增 `extract_real_ip_with_trust`、`config.rs` 新增 `trust_proxy_headers`）
  - SEC-33 JWT 黑名單 DB 持久化（`010_jwt_blacklist.sql` migration、`jwt_blacklist.rs` 記憶體+DB 雙層架構、`main.rs` 啟動載入+背景清理）
  - SEC-31 CORS Origin 動態化（`config.rs` 新增 `cors_allowed_origins`、`main.rs` 從環境變數讀取）
  - Docker 預設值翻轉（`COOKIE_SECURE` 預設 true、`SEED_DEV_USERS` 預設 false）
  - SEC-34 Refresh Token CSPRNG（`services/auth.rs` 從 UUID v4 改為 OsRng 256-bit）
  - SEC-32 JWT 過期統一（`jwt_expiration_hours` → `JWT_EXPIRATION_MINUTES`，預設 15 分鐘）
  - Mutex 中毒 fail-closed（`jwt_blacklist.rs` + `rate_limiter.rs` 中毒時拒絕請求而非放行）
  - `cargo build` 零錯誤零警告、`cargo test` 87 測試全通過
  - trust_proxy_headers 全面整合：12 處 `extract_real_ip` 呼叫全部改為 `extract_real_ip_with_trust`（`auth.rs`×4、`user.rs`×3、`attendance.rs`×2、`activity_logger.rs`×1、`rate_limiter.rs`×2）；rate limiter 加入 `State<AppState>`、`routes.rs` 改用 `from_fn_with_state`
  - 移除殘留 `JWT_EXPIRATION_HOURS`（`docker-compose.yml`、`backend/.env`）

### 2026-02-23

- ✅ 🔧 **CI 前端修復（tsc + vitest）**：
  - 新增 `src/vite-env.d.ts`（`/// <reference types="vite/client" />`），解決 `import.meta.env` 型別錯誤
  - `tsconfig.json` 加入 `"types": ["vitest/globals"]`，解決 vitest/globals 型別定義缺失
  - 執行 `npm install` 安裝缺少的 139 個 devDependencies（含 vitest、jsdom、@testing-library 等）
  - 驗證：`tsc --noEmit` exit code 0（0 錯誤）、`npx vitest run` 2 tests 全通過

- ✅ 🔒 **安全強化（SEC-34/35/36）**：
  - SEC-36 輸入長度限制：`nginx.conf` 加入 `client_max_body_size 30m`；`main.rs` 加入 `DefaultBodyLimit::max(30MB)`；`file.rs` ProtocolAttachment 50→30MB；`user.rs` 9 處 / `protocol.rs` 3 處 / `euthanasia.rs` 4 處加入 `validate(length(max=...))` 限制
  - SEC-35 上傳目錄隔離：`nginx.conf` 加入 `location /uploads { deny all; }`；`file.rs` upload 函式加入檔名與 entity_id 路徑穿越檢查
  - SEC-34 稽核日誌防篡改：`011_audit_integrity.sql` migration（`integrity_hash` + `previous_hash` 欄位）；`config.rs` 加入 `audit_hmac_key`；`audit.rs` 加入 HMAC-SHA256 雜湊鏈（graceful degradation）；`Cargo.toml` 加入 `hmac = "0.12"`
  - `.env.example` 完整範本建立（含所有環境變數分類說明與密鑰產生指引）
  - `.env` 加入 `AUDIT_HMAC_KEY`
  - `cargo test` 87 tests 全通過（含 FileCategory max_size 30MB 斷言更新）

- ✅ 🧪 **測試覆蓋率擴充（87→119, +32 tests）**：
  - `real_ip.rs` 新增 9 個測試：trust_proxy true/false、CF-Connecting-IP > X-Real-IP > X-Forwarded-For 優先級、多 IP 逗號分隔、空 header 跳過、空白修剪、fallback
  - `csrf.rs` 新增 15 個測試：requires_csrf_check（POST/PUT/DELETE/PATCH 需驗證、GET/HEAD/OPTIONS 免驗）、is_exempt_path（login/refresh/forgot/reset 豁免、一般路徑不豁免）、extract_csrf_cookie（存在/不存在/無 header/單一 cookie）
  - `config.rs` 新增 7 個測試：is_email_enabled 有無 SMTP host、GPS 半徑預設值、JWT Secret 最小長度、HMAC key 預設 None、CORS origins、cookie_secure 預設 false
  - `cargo test` 119 tests 全通過

- ✅ 📘 **OpenAPI Phase 1：Auth + Users + Roles**：
  - 新增 `openapi.rs`（23 handler paths + 18 ToSchema schemas）
  - `main.rs` 掛載 SwaggerUI 於 `/swagger-ui`
  - `models/user.rs` 13 struct/enum 加 ToSchema
  - `models/role.rs` 5 struct 加 ToSchema
  - `models/mod.rs` PaginationQuery/PaginatedResponse 加 ToSchema
  - `error.rs` 新增 ErrorResponse/ErrorDetail（ToSchema）
  - `handlers/auth.rs` 10 handler、`handlers/user.rs` 7 handler、`handlers/role.rs` 6 handler 加 `#[utoipa::path]`
  - `handlers/mod.rs` user/role 改 `pub(crate)`
  - `cargo test` 119 tests 全通過

### 2026-02-21

- ✅ 📱 **手機端 Dialog 滾動修復**：
  - `dialog.tsx`：`DialogContent` 改為 flexbox 置中（`fixed inset-0 flex items-center justify-center`）+ 內部可滾動 div（`max-h-[100dvh] sm:max-h-[90vh] overflow-y-auto`）
  - `index.html`：viewport 加入 `maximum-scale=1.0, user-scalable=no` 防 iOS 輸入框自動縮放
  - `index.css`：新增手機端 Dialog 全域 CSS（input font-size ≥16px、觸控平滑滾動、overscroll-behavior、body scroll lock）
  - 清理 14 處 DialogContent 的重複 `max-h/overflow-y` class（BloodTestTab ×2、ProtocolEditPage、WarehouseLayoutPage、DashboardPage、RolesPage ×2、AuditLogsPage、AdminAuditPage ×2、ObservationFormDialog、SurgeryFormDialog、SacrificeFormDialog、QuickEditAnimalDialog）
  - `npm run build`（tsc + vite）編譯通過

- ✅ 💊 **治療方式藥物選單改造**：
  - `ObservationFormDialog.tsx`：治療方式 Repeater 的 drug/dosage Input 改為 DrugCombobox（可搜尋、自動帶入單位）
  - `SurgeryFormDialog.tsx`：4 處「其他」Repeater（誘導、術前、維持、術後）全部改用 DrugCombobox
  - `EmergencyMedicationDialog.tsx`：藥品+劑量 Input 改為單一 DrugCombobox 元件
  - `SacrificeFormDialog.tsx`：保留簡單 Input（使用者決定）
  - TreatmentItem / MedicationItem 型別擴充 `drug_option_id` + `dosage_unit` 欄位
  - `npm run build`（tsc + vite）編譯通過（exit code 0）

### 2026-02-17

- ✅ 🔔 **啟動配置警告 Dialog**：
  - 後端新增 `GET /admin/config-warnings` API（`config_check.rs`），回傳三項配置檢查狀態 JSON
  - 前端 `MainLayout.tsx` 新增 Dialog 元件，管理員登入後自動彈出
  - 顯示三項狀態：⚠️ 地理圍籬警告 / ✅ 密碼設定正確 / ✅ 開發帳號正確（或 ℹ️ 未啟用）
  - 使用 `sessionStorage` 防止同一 session 重複彈出
  - 防止點擊外部或按 ESC 關閉（必須按確認按鈕）
  - API 驗證通過：`warn_count: 1`（地理圍籬 ⚠️ + 其他兩項 ✅）

- ✅ 🔧 **啟動配置匯總框改進**：
  - `main.rs` 啟動配置完整性檢查改為永遠顯示三項狀態匯總框（不再僅有警告時才顯示）
  - 設定正確：✅ 圖示、有問題：⚠️ 圖示、未啟用：ℹ️ 圖示
  - 地理圍籬：IP/GPS 全未設定 → ⚠️、部分設定 → ⚠️（建議補齊）、全部設定 → ✅
  - ADMIN_INITIAL_PASSWORD：未設定/過弱 → ⚠️、設定正確 → ✅
  - SEED_DEV_USERS：啟用 + TEST_USER_PASSWORD 未設定 → ⚠️、啟用 + 密碼設定正確 → ✅、未啟用 → ℹ️ 提醒
  - 有警告時使用 `tracing::warn!`，全通過時使用 `tracing::info!`
  - `cargo check` 編譯通過，Docker 容器重建後日誌確認正確輸出

- ✅ 🧪 **CI cargo test 修復**：
  - `enums.rs`：`AnimalStatus::Completed` display_name 從 `"存活完成"` 改回 `"實驗完成"`（與前端/測試一致）
  - `enums.rs`：`AnimalBreed::White` display_name 從 `"白"` 改回 `"白豬"`（與前端/測試一致）
  - `services/animal/mod.rs`：移除未使用的 `pub use core::IacucChangeInfo` re-export
  - `services/partition_maintenance.rs`：移除測試中未使用的 `use super::*`
  - 87 個測試全部通過，0 個編譯警告

- ✅ 🔐 **密碼更新不登出**：
  - 後端：`AuthService::change_own_password` 改為回傳 `LoginResponse`，密碼更新後重新簽發 access/refresh tokens（而非撤銷所有 tokens）
  - 後端：`change_own_password` handler 改用 `login_response_with_cookies` 回傳新 cookies
  - 前端：`MainLayout.tsx` 密碼更新成功後改呼叫 `checkAuth()` 取代 `logout()`
  - `cargo check` 編譯通過

- ✅ 🔒 **打卡 IP 限制**：
  - 後端：`Config` 新增 `ALLOWED_CLOCK_IP_RANGES`（CIDR 格式，如 `10.0.4.0/24,125.231.147.132`）
  - 後端：`attendance handler` 加入 `extract_real_ip` + `validate_clock_ip` 白名單驗證，IP 寫入 DB
  - 後端：`HrService::is_ip_in_ranges` CIDR 比對函式（支援 `/24` 網段 + 單一 IP）
  - 前端：`HrAttendancePage.tsx` 403 狀態碼辨識 + 「請連接辦公室 WiFi」友善提示
  - `docker-compose.yml` 新增 `ALLOWED_CLOCK_IP_RANGES` 環境變數

- ✅ 📍 **GPS 定位打卡**：
  - DB：`009_attendance_gps.sql` 新增 4 個欄位（`clock_in_latitude`/`clock_in_longitude`/`clock_out_latitude`/`clock_out_longitude`）
  - 後端：`Config` 新增 `CLOCK_OFFICE_LATITUDE`/`CLOCK_OFFICE_LONGITUDE`/`CLOCK_GPS_RADIUS_METERS` 設定
  - 後端：`attendance handler` 新增 Haversine 公式距離計算 + `validate_clock_location`（IP 或 GPS 任一通過）
  - 後端：`ClockInRequest`/`ClockOutRequest` 新增 `latitude`/`longitude` 欄位，`AttendanceRecord` 新增 GPS 欄位
  - 後端：`HrService::clock_in`/`clock_out` GPS 座標寫入 DB
  - 前端：`HrAttendancePage.tsx` 使用 `navigator.geolocation.getCurrentPosition()` 取得座標後送出
  - `.env` 設定辦公室座標（24.654053, 120.784923 苗栗後龍）
  - `docker-compose.yml` 新增 GPS 環境變數傳遞

- ✅ 🐷 **動物詳情頁新增「登記猝死」前端操作入口**：
  - `AnimalDetailPage.tsx`：新增 `showSuddenDeathDialog` 狀態、`suddenDeathForm` 表單狀態、`createSuddenDeathMutation`
  - Dialog 表單欄位：發現時間（datetime-local）、發現地點、可能原因、備註、需要病理檢查勾選
  - 按鈕顯示條件：`in_experiment` 和 `completed` 狀態，與緊急給藥/安樂死按鈕同列
  - 確認提示避免誤操作，提交後自動更新動物狀態
  - `tsc --noEmit` 編譯通過

- ✅ 📄 **AUP 參考文獻格式（農業部表單）**：
  - `protocol.ts`：`guidelines` 型別新增 `databases` 陣列（code/checked/keywords/note）
  - `constants.ts`：預設初始化 A-L 共 12 個資料庫項目
  - `SectionGuidelines.tsx`：重寫為 5.1 法源依據 + 5.2 資料庫搜尋紀錄（A-L 勾選 + 關鍵字/備註）+ 5.3 引用文獻列表
  - `ProtocolContentView.tsx`：唯讀顯示已勾選資料庫及關鍵字/備註
  - `zh-TW.json` / `en.json`：新增 databases A-L 翻譯、databasesTitle、keywordsLabel、noteLabel
  - `tsc --noEmit` 編譯通過

- ✅ 🧹 **測試套件整合與清理**：
  - 執行 `cleanup_test_data.ps1` 清理所有測試資料（保留審計記錄）
  - 建立 `tests/_archive/` 歸檔 4 個過時腳本（`aup_test_standalone.py`、`audit_check_deep.py`、`audit_check_quick.py`、`debug_csrf.py`）
  - `tests/README.md` 全面改寫：目錄結構、8 大測試模組詳細說明、共用基底架構（BaseApiTester/SharedTestContext/test_fixtures）、CLI 指令、清理方式、環境變數
  - 驗證 `run_all_tests.py --help` 正常運作，8 個測試模組全部涵蓋

- ✅ 🔒 **IACUC No. 變更保護與時間軸記錄**：
  - 後端：`AnimalService::update` 禁止 `in_experiment` 動物更改 IACUC No.（回傳 BadRequest）
  - 後端：偵測 IACUC No. 變更，回傳 `IacucChangeInfo`，`update_animal` handler 寫入 `IACUC_CHANGE` 審計事件（含 before/after data）
  - 後端：新增 `GET /animals/:id/events` API，從 `user_activity_logs` 查詢 IACUC 變更事件
  - 前端：`AnimalEditPage.tsx` 實驗中動物 IACUC 下拉禁用 + 提示文字
  - 前端：`AnimalTimelineView.tsx` 新增 `iacuc_change` 時間軸項目（Edit2 圖示 + amber 配色）
  - 前端：`AnimalDetailPage.tsx` 新增 events query + 傳遞 `iacucEvents` prop
  - `cargo check` + `tsc --noEmit` 編譯通過

- ✅ 📝 **Profiling_Spec v5.0 全面重寫**：
  - 重寫 01-09 主文件（11 個檔案）：架構概覽、核心領域模型、模組與邊界、資料庫綱要、API 規格、權限 RBAC、安全稽核、出勤模組、擴展性
  - 重寫 5 個模組文件：ANIMAL_MANAGEMENT、AUP_SYSTEM、ERP_SYSTEM、HR_SYSTEM、NOTIFICATION_SYSTEM
  - 更新 README.md、database_erd.md
  - 新增內容：動物轉讓流程、6 種動物狀態、猝死登記、手寫電子簽章、4 場景、資料隔離機制、通知路由可配置化、migration 14→08 整合
  - 更新統計：~293 API 端點、~73 資料表、8 migration

### 2026-02-16

- ✅ **第三波前端轉讓 UI 完成**：
  - 新增 `TransferTab.tsx`（Stepper 進度條 + 6 步角色表單 + 歷史紀錄）
  - `animal.ts`：`AnimalTransferStatus` / `AnimalTransfer` / `TransferVetEvaluation` 型別 + 4 DTO
  - `api.ts`：`transferApi` 9 端點 + `transferStatusNames` re-export
  - `AnimalTimelineView.tsx`：`transferred` 事件（indigo 配色 / ArrowRightLeft 圖示）
  - `AnimalDetailPage.tsx`：transfer Tab（completed/transferred 條件顯示）+ transfers 查詢
  - `AnimalEditPage.tsx`：狀態下拉排除 `transferred` / `sudden_death`
  - `tsc --noEmit` 編譯通過
- ✅ **動物狀態生命週期重構（第二波後端完成）**：
  - DB：`animal_status` enum 新增 `transferred`、migration 014（`animal_transfers` + `transfer_vet_evaluations`）
  - 後端：`AnimalTransferStatus` 6 狀態 enum、轉讓 service 完整 6 步 + handler 7 端點 + 8 路由
  - 前端：`transferred` 狀態擴充 + Timeline 安樂死/猝死事件 + 多語系
  - `cargo check` + `tsc --noEmit` 編譯通過
- ✅ **動物狀態生命週期重構（第一波完成）**：
  - DB：`animal_status` enum 新增 `euthanized`/`sudden_death`、`animal_sudden_deaths` 表（migration 013）
  - 後端：5 狀態 `can_transition_to()`/`is_terminal()`/`display_name()`、安樂死→自動設 `euthanized` + 建空犧牲紀錄、猝死 handler+service GET/POST、protocol stats 統計修正
  - 前端：`AnimalStatus` 擴充、`AnimalsPage`/`AnimalDetailPage` 顏色和 Tab、`zh-TW.json`/`en.json` 多語系
  - `cargo check` + `tsc --noEmit` 編譯通過
- ✅ **安全警報排序功能**：`AdminAuditPage.tsx` 安全警報表格新增前端排序功能，支援按時間/類型/嚴重程度/狀態排序（`useMemo` + 自訂優先級映射 critical>warning>info、open>acknowledged>resolved）
- ✅ **IACUC No. 大小寫修正**：前端 5 個檔案 6 處 + 後端 3 個檔案 7 處 `IACUC NO.` → `IACUC No.`（`AnimalDetailPage.tsx`、`EuthanasiaChairArbitrationPanel.tsx`、`EuthanasiaOrderDialog.tsx`、`EuthanasiaPendingPanel.tsx`、`ExportDialog.tsx`、`animal.rs`、`euthanasia.rs`、`alert.rs`）
- ✅ **修復時間軸未顯示實驗完成事件**：`AnimalTimelineView.tsx` 新增犧牲紀錄（`sacrificed`）和實驗完成（`completed`）兩種事件類型到時間軸，`AnimalDetailPage.tsx` 犧牲資料查詢 enabled 條件擴展至 timeline Tab
- ✅ **修復欄位表顯示錯誤**：`AnimalsPage.tsx` 中 `renderPenCell` 函式誤用外層作用域的 `animals`（全部動物列表），改為 `penAnimals`（該欄位的動物），修復每個欄位格子都重複渲染所有動物的問題
- ✅ **測試檔案 pig → animal 重構**：
  - `test_animal_full.py`：`PIG_CONFIGS` → `ANIMAL_CONFIGS`、`pig`/`pig_data`/`pig0` → `animal`/`animal_data`/`animal0`、`pig_index` → `animal_index`、中文註解「豬」→「動物」
  - `test_blood_panel.py`：修正 `test_pig` → `test_animal` bug（原始碼會導致 `NameError`）、註解更新
  - `cleanup_test_data.ps1`：SQL 表名 `pigs` → `animals`、`pig_sources` → `animal_sources`
  - `audit_check_deep.py`：entity_type `pig` → `animal`
  - 4 個檔案路徑更新（`test_erp_permissions.py`、`test_amendment_full.py`、`test_hr_full.py`、`run_all_tests.py`）

- ✅ **測試套件重寫**：
  - 3 個編碼損壞測試從零重建：`test_hr_full.py`（6 階段 HR 流程）、`test_amendment_full.py`（14 步驟 Amendment 流程）、`test_erp_permissions.py`（WM/AS/ES 權限驗證）
  - 所有 `.py` 檔案硬編碼帳密清除：`test_blood_panel.py`、`aup_test_standalone.py`、`audit_check_quick.py`、`audit_check_deep.py` 改用 `ADMIN_CREDENTIALS` / 環境變數
  - `run_all_tests.py` 從 5 個模組擴充為 7 個（新增 `--amendment`、`--erp-perm`）
  - `test_base.py` 新增 `save_results()` 自動儲存測試報告至 `tests/results/`
  - 建立測試規格文件 `tests/spec/test_spec.md`（7 模組測試目標與驗證方式）
  - 12 個 Python 檔案全部通過 `py_compile` 語法驗證

- ✅ **測試帳密環境變數化**：
  - `test_base.py` 新增 `TEST_USER_PASSWORD = os.getenv("TEST_USER_PASSWORD", "password123")` 共用常數
  - 8 個 `test_*` 檔案：匯入 `TEST_USER_PASSWORD` 取代硬編碼密碼（共 34 處）
  - 2 個獨立檔案（`aup_test_standalone.py`、`audit_verify.py`）：自行定義 `TEST_USER_PASSWORD` 環境變數
  - `test_base.py` DB fallback：`ipig_password_123` → `postgres`（移除硬編碼資料庫密碼）
  - 靜態驗證通過：`tests/` 中僅剩 `os.getenv` fallback 定義處包含 `password123`

- ✅ **資料庫遷移重寫（pig → animal 消除）**：
  - 重寫 `001_core_schema.sql`：5 個 enum 從 `pig_*` → `animal_*`，`import_type` 值更新，`protocol_activity_type` 中 `PIG_ASSIGNED`/`PIG_UNASSIGNED` → `ANIMAL_ASSIGNED`/`ANIMAL_UNASSIGNED`
  - 重寫 `003_animal_management.sql`：全面使用 `animal_*` 表名/索引名/欄位名
  - 重寫 `009_blood_test_system.sql`：`pig_blood_tests` → `animal_blood_tests`
  - 刪除 `012_rename_pig_to_animal.sql`（不再需要）
  - 遷移從 12 個整合為 11 個
  - `cargo check` + `cargo test` 87 個測試全數通過
- ✅ **Profiling_Spec 全面重寫（pig → animal 文件更新）**：
  - 重寫 12 個文件：01-09、database_erd.md、README.md、modules/ANIMAL_MANAGEMENT.md
  - 所有 pig_*/pigs/豬隻 用語替換為 animal_*/animals/動物
  - API 路由 `/pigs` → `/animals`、表名 `pigs` → `animals`、ENUM `pig_*` → `animal_*`
  - 版本升至 4.0，日期更新為 2026-02-16
  - PROGRESS.md、walkthrough.md 殘留 pig 用語修復
- ✅ **NAMING_CONVENTIONS.md 重寫（pig → animal）**：
  - Rust 模組/處理器/服務/模型範例全面更新（`pig.rs` → `animal.rs`）
  - 資料庫表名/欄位/索引/列舉範例更新（`pigs` → `animals`、`pig_id` → `animal_id`）
  - React 元件/頁面/Hook/型別範例更新（`PigDetail` → `AnimalDetail`、`usePigs` → `useAnimals`）
  - API 路由範例更新（`/pigs` → `/animals`、`/pig-sources` → `/animal-sources`）
  - CSS 類別/Git 分支/提交訊息範例同步更新
  - 版本升至 3.0
- ✅ **UI_UX_GUIDELINES.md 重寫（pig → animal）**：
  - Lucide 圖示 `Pig` → `PawPrint`
  - 版本升至 3.0
- ✅ **前端 AuditLogsPage pig→animal 修正**：
  - `AuditLogsPage.tsx`：`PIG_CREATE`/`PIG_UPDATE`/`PIG_DELETE`/`PIG_BATCH_ASSIGN` → `ANIMAL_*` 事件類型
  - `AuditLogsPage.tsx`：`pig_observation`/`pig_surgery`/`pig_weight` 等 7 個實體類型 → `animal_*`
  - `AuditLogsPage.tsx`：`categoryEntityMap` 篩選器值同步更新（ALL + ANIMAL 兩處）
  - 後端 `protocol.rs`：`PigAssigned`/`PigUnassigned` → `AnimalAssigned`/`AnimalUnassigned` enum 變體
  - 前端 `aup.ts`：`ProtocolActivityType` 類型定義同步
  - `tsc --noEmit` 編譯通過（exit code 0）

### 2026-02-15 (晚間)

- ✅ **後端編譯修復（pig → animal 殘留清理）**：
  - 修正 `Pig` → `Animal` 匯入和型別引用（`animal_core.rs`、`core.rs`、`medical.rs`）
  - `upload_pig_photo` → `upload_animal_photo`、`pig_breed` → `animal_breed` SQL 型別
  - `pig_ear_tag`/`pig_iacuc_no` → `animal_ear_tag`/`animal_iacuc_no`（euthanasia model + service + 前端 2 檔）
  - `pigEarTag` → `animalEarTag`（`VetRecommendationDialog` + `AnimalDetailPage`）
  - Migration 012 修復（移除不存在的 `pig_export_records` 表）並成功套用
  - 刪除全部 `.sqlx/` 快取、`cargo sqlx prepare` 重建 50 個快取檔案
  - `cargo build --release` ✅ 通過

### 2026-02-15

- ✅ **P1 資料分析模組**：血液檢查結果分析頁面完整實作
  - 後端 `GET /reports/blood-test-analysis` API（扁平化數據，支援專案/動物/項目/日期篩選）
  - 前端 `BloodTestAnalysisPage.tsx`（篩選區、摘要統計、異常警示、折線圖、盒鬚圖、資料明細、CSV/Excel 匯出）
  - 安裝 `recharts` + `xlsx` 依賴
- ✅ **P3 安全改善 SEC-14**：檔案上傳 Magic Number 驗證（`validate_magic_number` 函數，支援 7 種檔案格式，14 個測試通過）
- ✅ **技術債清理 T2**：`services/document.rs`（984 行）拆分為 4 個子模組（crud/workflow/grn/stocktake）
- ✅ **技術債清理 T3**：`models/animal.rs`（1250 行）拆分為 4 個子模組（enums/entities/requests/mod），12 個測試通過
- ✅ `cargo check` 零錯誤、`cargo test animal::tests` 12/12 通過
- ✅ **TODO 清單整理**：按 P1-P5 優先級重新排序，清理已完成項目，表格化呈現 24 項待辦

### 2026-02-14 (下午)

- ✅ PDF 報表分頁優化：`generate_project_medical_pdf` 重構為每隻動物獨立分頁、封面摘要、共用 `render_animal_medical_data` helper
- ✅ 血液檢查組合管理頁面 `BloodTestPanelsPage.tsx`：Panel CRUD + 管理包含項目（搜尋、篩選、排序）
- ✅ 路由整合（`App.tsx`）與 `BloodTestTemplatesPage.tsx` 按鈕改為「管理分類」導向
- ✅ 所有測試帳密統一更新（7 個測試檔案）
- ✅ `test_blood_panel.py` 58 項全數通過

### 2026-02-14 (上午)

- ✅ 記錄真實 IP 位址、Docker 依賴升級、Session 活動追蹤修正
- ✅ 移除預設管理員帳密、Email 破圖修正
- ✅ 多帳號腳本登入偵測 + 同時大量登入偵測 + 安全警報修復

### 2026-02-09 ~ 2026-02-10

- ✅ 多帳號腳本登入偵測 + 同時大量登入偵測 + 安全警報修復
- ✅ 審查委員強制意見、多輪往返功能、權限放寬
- ✅ AUP 狀態歷程修復、PI 刪除草稿權限修復
- ✅ 翻譯標準化（手術計畫標籤、Placeholder）

### 2026-02-15

- ✅ 後端 Service 重構：`amendment.rs`（803 行）拆分為 `crud.rs` + `workflow.rs` + `query.rs`
- ✅ 後端 Service 重構：`pdf.rs`（732 行）拆分為 `context.rs` + `service.rs`
- ✅ 單元測試擴充：為 `protocol.rs`、`hr.rs`、`facility.rs` 新增 20 個測試（79→87）
- ✅ **前端重構 T6**：`ProtocolEditPage.tsx`（4240 行）拆分為 10 個 Section 元件 + 4 個工具模組，tsc --noEmit 零錯誤
- ✅ **修復 Login 頁面 401 無限迴圈**：axios interceptor 移除 `window.location.href` 硬跳轉，改用 zustand `clearAuth()` 清除前端狀態 + `isLoggingOut` 鎖防重複觸發，讓 React Router 自然導向 `/login`
- ✅ **修復 IP 異常偵測 bug**：`check_unusual_location` SQL 比對缺少 `::INET` 轉型，導致每次都誤判為新 IP
- ✅ **GeoIP 地理位置異常偵測**：整合 MaxMind GeoLite2-City（`maxminddb` crate），`check_unusual_location` 改為國家層級比對（30 天內未見過的國家才觸發），登入記錄寫入 `geo_country`/`geo_city`/`geo_timezone`，Docker volume 掛載 `.mmdb` 檔案
- ✅ **稽核日誌匯出 CSV/PDF**：後端新增 `export_activities` API（`/admin/audit/activities/export`，LIMIT 10000），前端 `AuditLogsPage.tsx` 加入 CSV（BOM + Blob 下載）及 PDF（可列印 HTML 表格 + `window.print()`）匯出按鈕
- ✅ **活動紀錄分頁優化**：`ProtocolDetailPage.tsx` 歷程 Tab 加入前端分頁（每頁 15 筆 + 上/下一頁控制列 + 總筆數顯示）
- ✅ **行動端適配（響應式設計）**：`MainLayout.tsx` overlay sidebar + 漢堡選單 + 背景遮罩、`PigsPage` / `DashboardPage` / `AuditLogsPage` / `ProtocolDetailPage` 表格/篩選/標題響應式、`index.css` 全域工具 class（`.page-title`、`.table-responsive`、`.filter-row`）
- ✅ **Profiling_Spec 文件完整重寫**：01-09 全部重寫（含新增 07_SECURITY_AUDIT.md），README 索引更新。API 規格 293 端點、RBAC 84 權限 × 11 角色完整對照
- ✅ **通知路由可配置化**：`notification_routing` 資料表 + 後端 CRUD API（`/admin/notification-routing`）+ `get_recipients_by_event()` 動態查詢 + 6 個通知模組硬編碼遷移（hr/erp/alert/protocol/amendment）+ 前端管理介面（`NotificationRoutingSection.tsx`）
- ✅ **SEC-20 帳號鎖定機制**：`AuthService::login()` 加入 15 分鐘內 5 次失敗登入即暫時封鎖，阻止暴力破解攻擊
- ✅ **SEC-21 JWT Secret 強度驗證**：`Config::from_env()` 啟動時強制檢查金鑰長度 ≥ 32 字元，附建議產生指令
- ✅ **SEC-22 啟動安全配置警告**：`main.rs` 啟動時檢查 `COOKIE_SECURE`/`SEED_DEV_USERS`，不安全配置醒目警告
- ✅ **修復強制登出自己後未跳轉登入頁**：`AdminAuditPage.tsx` `forceLogoutMutation.onSuccess` 新增 user_id 比對，強制登出自己的 Session 時呼叫 `logout()` 跳轉登入頁
- ✅ **安全審計手機端排版修正**：`AdminAuditPage.tsx` TabsList 從 `grid-cols-5` 改為 `flex overflow-x-auto sm:grid sm:grid-cols-5`，手機端隱藏圖示、縮小文字、Badge 響應式調整
- ✅ **SEC-23~28 安全加強 9 項完整實作**：
  - SEC-23 JWT 黑名單（`jwt_blacklist.rs` 記憶體 HashMap + 背景清理 + auth middleware 整合）
  - SEC-24 CSRF 防護（`csrf.rs` Double Submit Cookie + 前端 `api.ts` request interceptor 自動附加）
  - SEC-25 JWT 有效期縮短至 15 分鐘（`config.rs` `jwt_expiration_seconds` + 前端 refresh queue 防競態）
  - SEC-26 正式環境禁止開發帳號（`main.rs` 啟動拒絕 + `seed.rs` 密碼改環境變數）
  - SEC-27 安全回應標頭（`main.rs` API 層 + `nginx.conf` CSP）
  - SEC-28 Session 併發限制（`handlers/auth.rs` 登入後自動裁減超額 session）
  - 前端增強：`App.tsx` ProtectedRoute isInitialized loading、`auth.ts` isInitialized flag
- ✅ **豬隻→動物命名全端重構**：後端 19 檔案（seed/permissions/handlers/services/file/middleware）、前端 17 檔案（翻譯 key/queryKey/entity type/local variable/comments）、翻譯 JSON 2 檔案，共計 ~180 處修改。`cargo build` 和 `tsc --noEmit` 均通過
- ✅ **分頁功能實作**：`AdminAuditPage.tsx` 4 個 Tab（活動記錄、登入事件、Sessions、安全警報）加入伺服器端分頁（`page`/`per_page=50`）；`UsersPage.tsx` 加入前端分頁（每頁 50 筆）；`AuditLogsPage.tsx` 已有分頁功能不需修改
- ✅ **豬隻→動物最終掃描修正**：追加修正前端 7 檔案（`api.ts`、`BloodTestTab.tsx`、`QuickEditAnimalDialog.tsx`、`ImportDialog.tsx`、`VetCommentsWidget.tsx`、`AnimalEditPage.tsx`、`types/animal.ts`）+ 後端 7 檔案（`euthanasia.rs`、`import_export.rs`、`upload.rs`、`dashboard.rs`、`blood_test.rs`、`alert.rs`、`numbering.rs`）。`tsc --noEmit` exit code 0。前端 0 殘留、後端僅剩品牌 logo
- ✅ **豬隻→動物第二輪掃描修正**：Bug Fix `MyProjectDetailPage.tsx`（`species === 'animal'` → `'pig'`、`white_animal` → `white_pig` DB 常數還原）、`en.json` 翻譯 key 6 處、後端 6 檔案中文註釋。前後端 grep 0 殘留確認
- ✅ **測試套件 CSRF/Cookie 修正**：`test_base.py` 加入 CSRF Double Submit Cookie 支援、修正 session cookie 覆蓋 Bearer token 問題、`test_aup_full.py` 修正 Step 14 reviewer 意見邏輯、`test_hr_full.py` 改用統一請求方法。全部 7/7 模組測試通過（AUP, ERP, Animal, Blood Panel, HR, Amendment, ERP Permissions）
- ✅ **SEC：auth_middleware Token 優先順序**：從 Cookie > Bearer 改為 **Bearer > Cookie**（`auth.rs`），降低 Cookie 注入攻擊風險，避免 session cookie 殘留覆蓋正確的 Authorization header。7/7 測試通過
- ✅ **動物轉讓資料隔離**：後端 `GET /animals/:id/data-boundary` API + 5 個 service/handler 加入 `after` 時間過濾（觀察、手術、體重、疫苗、血檢）；前端 `AnimalDetailPage.tsx` 查詢 data-boundary 並自動傳入 `?after=` 參數。特權角色（ADMIN/VET/IACUC_STAFF/IACUC_CHAIR）可繞過隔離看全部資料。`cargo check` + `tsc --noEmit` 均通過

### 2026-02-06 ~ 2026-02-08

- ✅ AUP 歷程紀錄增強、動物列表體重排序
- ✅ 後端與規格重組、AUP 表單預設值、審查系統與權限優化
- ✅ 角色重命名 CHAIR → IACUC_CHAIR、審查意見互動增強

### 2026-02-02 ~ 2026-02-05

- ✅ 倉庫平面圖編輯器改進、儲位庫存管理
- ✅ Login As 功能、AUP 資歷管理、版本比較改進
- ✅ 使用者管理增強（學經歷欄位）、獸醫師通知增強
- ✅ UUID 遷移、Amendment 後端系統、安樂死工作流程 UI

### 2026-02-16（晚間）

- ✅ 📱 **手寫電子簽章 Phase 1 完成**：
  - 後端：DB migration 015（`handwriting_svg`、`stroke_data`、`signature_method` 欄位）、`SignatureService` 擴展（`sign_with_handwriting`）、handlers 修改
  - 前端：`signature_pad` 套件安裝、`HandwrittenSignaturePad.tsx` 元件、`signatureApi` API 層、`SacrificeFormDialog.tsx` 整合手寫簽名區塊、zh-TW/en 多語系翻譯
  - 驗證：`cargo check` + `tsc --noEmit` 編譯通過

- ✅ 🔧 **測試套件整合重構**：
  - 新增 `test_fixtures.py`（24 角色統一帳號註冊表）+ `test_context.py`（SharedTestContext 共享 token/session）
  - 修改 8 個測試模組接受 `ctx` 參數注入（`test_aup_full`、`test_amendment_full`、`test_animal_full`、`test_blood_panel`、`test_erp_full`、`test_erp_permissions`、`test_hr_full`、`test_aup_integration`）
  - 重寫 `run_all_tests.py`：一次性初始化共享 Context + 傳遞 ctx + AUP→Amendment protocol_id 複用 + 新增 `--aup-integ`/`--no-shared` 參數
  - 資料量精簡：動物建立 20→5、AUP 動物 5→2、Amendment 複用 AUP 結果
  - 驗證：11 個 Python 檔案 `py_compile` 語法編譯通過、24 角色 import 測試通過

- ✅ 🧪 **Animal + AUP Integration 測試修復（共 11 個 bug）**：
  - **Animal（27/27 通過）**：Phase 7 犧牲動物 `PUT` 改為 `GET` 驗證狀態、新增 `euthanized` 為有效犧牲狀態
  - **AUP Integration（41/41 通過）**：
    - co-editor 角色從 PI 改為 IACUC_STAFF（需 `aup.review.assign` 權限）
    - `approve_protocol()` 新增 `skip_pre_review` 參數避免重複 PRE_REVIEW
    - ear_tag 格式改為三位數字（API 驗證規則）
    - `vaccine_date` → `administered_date`（符合 API schema）
    - 動物狀態機轉換：`unassigned → in_experiment(+iacuc_no) → completed` 兩步驟
    - transfer 端點統一使用 EXP_STAFF 角色（`animal.record.create` 權限）

- ✅ 📱 **手寫電子簽章 Phase 2–4 完成**：
  - **Phase 2（安樂死）**：後端 `sign_euthanasia_order` / `get_euthanasia_signature_status` handler + 路由；前端 `EuthanasiaPendingPanel.tsx` PI 同意流程整合手寫簽名
  - **Phase 3（轉讓）**：後端 `sign_transfer_record` / `get_transfer_signature_status` handler + 路由；前端 `TransferTab.tsx` PI 同意 + 完成轉讓整合手寫簽名
  - **Phase 4（計劃審查）**：後端 `sign_protocol_review` / `get_protocol_signature_status` handler + 路由；前端 `SectionSignature.tsx` 新增手寫簽名模式（與上傳並存）；`ProtocolWorkingContent` 型別擴展
  - `api.ts` 新增 6 個 API 函式（`signEuthanasia`、`getEuthanasiaStatus`、`signTransfer`、`getTransferStatus`、`signProtocol`、`getProtocolStatus`）
  - 驗證：`cargo check` + `tsc --noEmit` 編譯通過

### 2026-01-19 及更早

- ✅ HR 特休管理系統、GLP 合規、PDF 匯出
- ✅ 協編者權限修正、審查意見回覆、Google Calendar 整合
- ✅ 資料匯入（Excel/CSV）、Bug 修復（動物計數、PigBreed enum、體重驗證）
- ✅ v1.0 MVP 完成、檔案上傳服務、通知偏好設定

---

## 已完成待辦項目（從 TODO.md 移入）

> 以下項目原列於 TODO.md，已完成後移入此處。

| 原編號 | 項目 | 完成日期 | 摘要 |
|--------|------|----------|------|
| P2-1 | **豬隻→動物命名重構** | 2026-02-15 | 後端 42+ 檔案、前端 47+ 檔案、DB migration 全面 pig→animal 重命名 |
| P2-1a | **動物狀態生命週期（第一波）** | 2026-02-16 | euthanized/sudden_death 狀態 + 狀態機 + 猝死登記 API |
| P2-1b | **動物狀態生命週期（第二波）** | 2026-02-16 | transferred 狀態 + 轉讓 6 步 API（8 路由）+ 前端 TransferTab + 資料隔離 |
| P2-2 | **AUP 參考文獻格式** | 2026-02-17 | A-L 資料庫勾選與關鍵字，符合農業部表單格式 |
| P2-10 | **📱 行動裝置電子簽章** | 2026-02-16 | Phase 1–4 完成（犧牲紀錄/安樂死/轉讓/計劃審查手寫簽名） |
| P4-21 | **整合測試登入優化** | 2026-02-16 | SharedTestContext 共享 token + protocol_id 複用 + 動物數 20→5 |

---

## 10. 結論

豬博士 iPig 系統功能開發完成度 **100%** ✅，所有子系統均已完成：

1. **共用基礎架構** 完整 100%，認證授權、Email 服務均已就緒
2. **AUP 審查系統** 完整 100%，審查流程 UI 完整，PDF 匯出已完成
3. **iPig ERP (進銷存管理系統)** 完成度 100%，已可投入使用
4. **實驗動物管理系統** 完整 100%，所有功能均已實作，GLP 合規完成
5. **通知系統** 完整 100%，排程任務、Email 通知、偏好設定均已完成
6. **HR 人事管理系統** 完整 100%，特休管理、Google Calendar 整合已完成

### 正式上線準備度：~60%

功能開發雖已全數完成，但距離「正式上線」尚需補齊以下面向：

| 面向準備度 | 待補齊項目 |
|-----------|----------|
| ✅ 已就緒 | 安全強化 14 SEC、備份機制、行動端適配、Bundle 優化、CI/CD Pipeline |
| 🔶 需加強 | 測試覆蓋率、效能基準、安全掃描 (npm audit/Trivy)、前端錯誤 UX |
| 🔴 未開始 | 可觀測性 (/health /metrics Sentry)、GLP 合規文件、使用者/維運手冊、滲透測試 |


### 2026-02-17：修復 cleanup_test_data.sql 清理腳本

**修復 3 處 bug：**
1. `google_calendar_config` 用 `id` 引用 `users.id`（無 `user_id` 欄位）
2. `system_settings` 用 `updated_by`（無 `created_by` 欄位）
3. 89 個 FK 約束阻擋 `DELETE FROM users` → 用 `DISABLE TRIGGER ALL` 暫時停用

**其他修正：**
- 舊表名 `pigs`/`pig_*` → `animals`/`animal_*`
- 新增 10+ 個遺漏的表（`animal_transfers`、`protocol_status_history`、`review_round_history` 等）
- 保留 admin 帳號與角色、種子資料、審計日誌

**驗證結果：** 使用者=1（admin）、動物/計畫/倉庫/單據=0、角色=11（保留）、admin 角色「系統管理員」完整

---

### 2026-02-17：通知路由進階設定

**後端變更：**
- `routing.rs` 新增 `list_available_event_types()`（5 大類 26 種事件）、`list_available_roles()`（DB 查詢）
- `notification.rs` 新增 `EventTypeCategory`、`EventTypeInfo`、`RoleInfo` 三個 struct
- `notification_routing.rs` 新增兩個 handler（event-types, roles）
- `routes.rs` 新增兩條路由

**DB Seed：**
- `008_supplementary.sql` 新增 5 條預設路由：`all_reviews_completed`、`all_comments_resolved`、`animal_abnormal_record`、`animal_sudden_death`、`low_stock_alert`→PURCHASING

**前端變更：**
- `notification.ts` 新增 3 個介面 + 9 個新事件類型名稱
- `api.ts` 新增 `getEventTypes()`、`getRoles()` API
- `NotificationRoutingSection.tsx` 全面重寫：文字輸入→下拉選單、規則列表→分類卡片、新增分類圖示

**驗證：** 後端 `cargo check` + 前端 `tsc --noEmit` 通過

**通知觸發邏輯實作（2026-02-17 續）：**
- `protocol.rs`：新增 `notify_all_reviews_completed()` + `notify_all_comments_resolved()`
- `animal.rs`：新增 `notify_abnormal_record()`
- `review.rs`：`create_review_comment` handler 非同步檢查全員意見完成 → 觸發通知
- `review.rs`：`resolve_review_comment` handler 非同步檢查全部意見已解決 → 觸發通知（修復 `Option<Uuid>` protocol_id 型別）
- `observation.rs`：異常觀察紀錄建立時觸發通知（修復 `general_appearance`/`clinical_signs` 欄位不存在問題，改用 `content`）
- `010_notification_routing_seed.sql`：5 條新 seed data

---

### 2026-02-17：SEC-31 資料庫自動備份

**新增檔案：**
- `scripts/backup/pg_backup.sh`：pg_dump + gzip + 30 天清理 + rsync 異地備份
- `scripts/backup/Dockerfile.backup`：postgres:16-alpine + dcron + rsync
- `scripts/backup/entrypoint.sh`：環境變數注入 cron + 前景執行 crond
- `scripts/backup/BACKUP.md`：備份指南（快速開始/手動備份/還原/rsync 設定）

**修改檔案：**
- `docker-compose.yml`：新增 `db-backup` 服務 + `db_backups` volume

**環境變數：** `BACKUP_SCHEDULE`、`BACKUP_RETENTION_DAYS`、`RSYNC_TARGET`、`BACKUP_ON_START`

---

### 2026-02-18：治療方式藥物選單 + 後台管理（Phase 1）

**後端新增檔案：**
- `migrations/009_treatment_drug_options.sql`：`treatment_drug_options` 表 + 14 筆 seed data
- `src/models/treatment_drug.rs`：資料模型
- `src/services/treatment_drug.rs`：CRUD + ERP 匯入服務
- `src/handlers/treatment_drug.rs`：HTTP handler（CurrentUser + require_permission!）

**後端修改檔案：**
- `src/routes.rs`：新增 `/treatment-drugs` + `/admin/treatment-drugs/*` 路由
- `src/models/mod.rs`、`src/services/mod.rs`、`src/handlers/mod.rs`：模組註冊

**前端新增檔案：**
- `src/types/treatment-drug.ts`：型別定義 + 常數
- `src/components/animal/DrugCombobox.tsx`：可搜尋藥物下拉選擇元件
- `src/pages/admin/TreatmentDrugOptionsPage.tsx`：後台 CRUD 管理頁面

**前端修改檔案：**
- `src/lib/api.ts`：新增 `treatmentDrugApi`
- `src/types/index.ts`：匯出 treatment-drug 型別
- `src/App.tsx`：新增 `/admin/treatment-drugs` 路由
- `src/layouts/MainLayout.tsx`：側邊欄「系統管理 > 藥物選單」

**驗證：** `cargo build`（SQLX_OFFLINE=true）✅ · `npx tsc --noEmit` ✅

---

### 2026-02-19：前端 Bundle 優化

**問題：** Vite build 主 JS chunk 達 3,267 KB（gzip 923 KB），遠超 500 KB 建議值；`auth.ts` 同時被靜態與動態 import 導致 code-splitting 失效。

**修改檔案：**
- `src/lib/api.ts`：`auth.ts` 動態 import → 靜態 import（消除 Vite 混合引入警告）
- `src/App.tsx`：50 個頁面元件從靜態 import 改為 `React.lazy()` + `Suspense`（路由級 code-splitting）
- `vite.config.ts`：新增 `manualChunks`（7 組 vendor chunk：react、data、radix、charts、office、calendar、i18n）

**效果：**
- 主 `index-*.js`：3,267 KB → **242 KB**（下降 92.6%）
- 產出 50+ 個頁面 chunk + 7 個 vendor chunk
- auth.ts 混合引入警告已消除

---

### 2026-02-21：Rust 安全強化（六項）

**CI Pipeline 擴充：**
- `ci.yml` 新增 4 個安全掃描 job：`security-audit`（cargo audit）、`cargo-deny`（供應鏈授權/漏洞）、`sql-injection-guard`（grep 防護 format! SQL）、`unsafe-guard`（unsafe 偵測）
- 升級 `backend-lint` job：`-D warnings -W clippy::unwrap_used`（unwrap 產生警告，漸進式修復）

**新增檔案：**
- `backend/deny.toml`：供應鏈安全策略（漏洞 deny / 授權白名單 MIT+Apache+BSD / 來源限 crates.io）
- `backend/clippy.toml`：Clippy 閾值設定（行數 200 / 參數 10 / 複雜度 30）
- `backend/.cargo/config.toml`：rustflags 啟用 `clippy::unwrap_used` warn
- `backend/fuzz/Cargo.toml` + 2 個 fuzz 目標：`fuzz_ear_tag.rs`（耳號解析）、`fuzz_sku_parse.rs`（SKU 編碼解析）

**修改檔案：**
- `backend/Cargo.toml`：新增 `[profile.release] overflow-checks = true`

**審計結果：**
- SQL 注入：✅ 全部使用 sqlx 巨集參數綁定，零手動拼接
- unsafe：✅ 專案本身零 unsafe 程式碼
- unwrap：✅ 61 處生產程式碼 unwrap() 已全部修復為 expect()（2026-02-22 完成）

---

### 2026-02-21：P2 四項安全與功能強化

**1. SEC-37：HSTS 標頭**
- `nginx.conf` 加入 `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`

**2. 敏感資料二級審計**
- `handlers/auth.rs`、`handlers/user.rs`、`handlers/audit.rs` 加入 `log_activity` 記錄密碼變更/重設/Login As/角色變更/帳號啟停/強制登出
- `AdminAuditPage.tsx` SECURITY 類別紅色 Badge 高亮

**3. 疼痛評估紀錄時間軸**
- 後端新增：`services/animal/care_record.rs`（Model + CRUD）、`handlers/animal/care_record.rs`（4 路由）
- 前端新增：`PainAssessmentTab.tsx`（CRUD 表單 + Badge 嚴重度標記 + Recharts 五維度折線圖）
- `AnimalDetailPage.tsx` 整合新 Tab

**4. 安全警報即時推送（SSE）**
- 後端新增：`handlers/sse.rs`（`AlertBroadcaster` + SSE handler）
- `AppState` 加入 `alert_broadcaster` 欄位
- `login_tracker.rs` 三處 alert 建立處加入 `broadcaster.send()`
- `auth.rs` 登入成功/失敗事件傳遞 broadcaster
- 前端新增：`hooks/useSecurityAlerts.ts`（EventSource + toast）
- `MainLayout.tsx` 整合 hook

**驗證：** `cargo check` ✅ · `npm run build` ✅

---

### OpenAPI 文件整合 — Phase 2（2026-02-23）

**Phase 2A — Facility & Warehouse**
- `models/facility.rs` 27 struct + `models/warehouse.rs` 4 struct → `ToSchema`
- `handlers/facility.rs` 30 handler + `handlers/warehouse.rs` 5 handler → `#[utoipa::path]`

**Phase 2B — Protocol & Review**
- `models/protocol.rs` 31+ struct/enum → `ToSchema`
- `handlers/protocol/crud.rs` 14 handler + `review.rs` 10 handler + `export.rs` 1 handler → `#[utoipa::path]`
- `openapi.rs` 新增 paths / schemas / tags（計畫書管理、審查管理）
- `handlers/mod.rs` + `protocol/mod.rs` → `pub(crate)`

**累計 OpenAPI 文件化 endpoint：83**（Phase 1: 23 + Phase 2A: 35 + Phase 2B: 25）

**驗證：** `cargo check` ✅

---

*本報告由系統自動產生，如有疑問請聯繫開發團隊。*
