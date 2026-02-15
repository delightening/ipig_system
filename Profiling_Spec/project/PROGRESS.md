# 豬博士 iPig 系統專案進度評估表

> **評估日期：** 2026-02-14  
> **規格版本：** v1.4  
> **評估標準：** ✅ 完成 | 🔶 部分完成 | 🔴 未開始 | ⏸️ 暫緩

## 📑 目錄

| # | 章節 | 說明 |
|---|------|------|
| - | [總體進度概覽](#-總體進度概覽) | 各子系統完成度摘要 |
| 1 | [共用基礎架構](#1-共用基礎架構) | 認證授權、使用者管理、角色權限、Email、稽核 |
| 2 | [AUP 提交與審查系統](#2-aup-提交與審查系統) | 計畫書管理、審查流程、附件、我的計劃 |
| 3 | [iPig ERP (進銷存管理系統)](#3-ipig-erp-進銷存管理系統) | 基礎資料、採購、銷售、倉儲、報表 |
| 4 | [實驗動物管理系統](#4-實驗動物管理系統) | 豬隻管理、紀錄、血液檢查、匯出、GLP |
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

**整體專案進度：100% ✅ (持續迭代中)**



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
| JWT 黑名單主動撤銷 | ✅ | - | ✅ | 記憶體黑名單 + auth middleware 整合 (SEC-23, 2026-02-15) |
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
| 豬隻紀錄 Tab | ✅ | ✅ | ✅ | |

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

### 4.4 豬隻紀錄 - 其他紀錄

| 功能 | 後端 | 前端 | 狀態 | 備註 |
|-----|:---:|:---:|:---:|------|
| 體重紀錄 CRUD | ✅ | ✅ | ✅ | |
| 疫苗/驅蟲紀錄 CRUD | ✅ | ✅ | ✅ | |
| 犧牲/採樣紀錄 | ✅ | ✅ | ✅ | |
| 病理組織報告 | ✅ | ✅ | ✅ | 含上傳功能 |

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
| 單一豬隻病歷匯出 | ✅ | ✅ | ✅ | ExportDialog |
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
| 通知路由管理 | ✅ | ✅ | ✅ | `notification_routing` 表、CRUD API、前端管理介面 (2026-02-15) |

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
| 001_initial_schema.sql | ERP 基礎表、使用者權限 | ✅ |
| 002_extend_schema.sql | AUP 系統、動物管理系統表 | ✅ |
| 003_frontend_requirements.sql | 通知系統、排程報表、views | ✅ |
| 004_pig_frontend_requirements.sql | 動物管理前端需求擴充 | ✅ |
| 005_notification_types.sql | 通知類型擴展、日誌表、清理函數 | ✅ |
| 006_attachments.sql | 附件管理表、索引 | ✅ |
| 007_role_soft_delete_and_admin_consolidation.sql | 角色軟刪除、管理員合併 | ✅ |
| 008_add_deleted_status.sql | 計劃書刪除狀態 | ✅ |
| 009_hr_system.sql | HR 特休表、hire_date 欄位 | ✅ |
| 010_glp_compliance.sql | 電子簽章、附註、變更原因表 | ✅ |
| 013_rename_chair_role.sql | 重新命名 CHAIR 角色為 IACUC_CHAIR | ✅ |
| 022_amendment_system.sql | 變更申請系統、review_comments 草稿欄位 | ✅ |
| 024_blood_test.sql | 血液檢查系統（主表、明細、模板） | ✅ |
| 025_seed_blood_templates.sql | 血液檢查模板 Seed 資料（64 項） | ✅ |
| 026_blood_test_panels.sql | 檢驗組合 + 組合項目關聯 + Seed（14 組） | ✅ |
| 011_notification_routing.sql | 通知路由規則表 + 21 筆預設種子資料 | ✅ |

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

- ✅ PDF 報表分頁優化：`generate_project_medical_pdf` 重構為每隻豬獨立分頁、封面摘要、共用 `render_pig_medical_data` helper
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
- ✅ **Profiling_Spec 文件完整重寫**：01-09 全部重寫（含新增 07_SECURITY_AUDIT.md），README 索引更新。API 規格 273 端點、RBAC 84 權限 × 11 角色完整對照
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
- ✅ **豬隻→動物最終掃描修正**：追加修正前端 7 檔案（`api.ts`、`BloodTestTab.tsx`、`QuickEditAnimalDialog.tsx`、`ImportDialog.tsx`、`VetCommentsWidget.tsx`、`AnimalEditPage.tsx`、`types/animal.ts`）+ 後端 7 檔案（`euthanasia.rs`、`import_export.rs`、`upload.rs`、`dashboard.rs`、`blood_test.rs`、`alert.rs`、`numbering.rs`）。`tsc --noEmit` exit code 0。前端 0 殘留、後端僅剩品牌 logo

### 2026-02-06 ~ 2026-02-08

- ✅ AUP 歷程紀錄增強、動物列表體重排序
- ✅ 後端與規格重組、AUP 表單預設值、審查系統與權限優化
- ✅ 角色重命名 CHAIR → IACUC_CHAIR、審查意見互動增強

### 2026-02-02 ~ 2026-02-05

- ✅ 倉庫平面圖編輯器改進、儲位庫存管理
- ✅ Login As 功能、AUP 資歷管理、版本比較改進
- ✅ 使用者管理增強（學經歷欄位）、獸醫師通知增強
- ✅ UUID 遷移、Amendment 後端系統、安樂死工作流程 UI

### 2026-01-19 及更早

- ✅ HR 特休管理系統、GLP 合規、PDF 匯出
- ✅ 協編者權限修正、審查意見回覆、Google Calendar 整合
- ✅ 資料匯入（Excel/CSV）、Bug 修復（豬隻計數、PigBreed enum、體重驗證）
- ✅ v1.0 MVP 完成、檔案上傳服務、通知偏好設定

---

## 10. 結論

豬博士 iPig 系統整體完成度 **100%** ✅，所有子系統均已完成：

1. **共用基礎架構** 完整 100%，認證授權、Email 服務均已就緒
2. **AUP 審查系統** 完整 100%，審查流程 UI 完整，PDF 匯出已完成
3. **iPig ERP (進銷存管理系統)** 完成度 100%，已可投入使用
4. **實驗動物管理系統** 完整 100%，所有功能均已實作，GLP 合規完成
5. **通知系統** 完整 100%，排程任務、Email 通知、偏好設定均已完成
6. **HR 人事管理系統** 完整 100%，特休管理、Google Calendar 整合已完成

---

*本報告由系統自動產生，如有疑問請聯繫開發團隊。*
