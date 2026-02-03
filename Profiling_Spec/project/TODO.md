# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-03

## 🚧 今日未完成事項 (2026-02-03)

### AUP 審查系統
- [ ] /api/hr/staff 後端端點更新 - 支援回傳學經歷欄位
- [ ] UUID 遷移完整測試 - 執行資料庫遷移並進行端對端測試 (遺留)

### 其他
- [x] Docker 建置測試 - 確認後端與前端建置成功 ✅ 已完成

### 📝 技術備註
- **權限代碼命名**: `main.rs` 使用 `animal.*` 開頭，但部分 migration 使用 `pig.*`。系統透過 `ensure_required_permissions()` 在啟動時補齊。
- **Admin 全權限**: Migration `027_admin_all_permissions.sql` 確保 admin 擁有所有權限。

---

## ✅ v1.0 MVP - 已完成

### 共用基礎架構
- [x] 認證系統（JWT 登入、登出、Token 刷新）
- [x] 密碼管理（忘記密碼、重設密碼、強制變更）
- [x] 使用者管理（CRUD、角色指派）
- [x] 角色權限管理（RBAC）
- [x] Email 服務（帳號開通、密碼重設）
- [x] 稽核紀錄

### iPig ERP (進銷存管理系統)
- [x] 倉庫、產品、SKU 管理
- [x] 供應商/客戶管理
- [x] 採購流程（PO → GRN → PR）
- [x] 銷售流程（SO → DO → SR）
- [x] 倉儲作業（調撥、盤點、調整）
- [x] 報表（庫存現況、流水、明細、成本）

### AUP 審查系統
- [x] 計畫書 CRUD
- [x] 狀態機（12 種狀態）
- [x] 審查流程基本功能
- [x] 我的計劃（外部使用者）
- [x] 審查意見管理（列出、新增、標記解決）
- [x] 審查人員指派（指派對話框、指派清單）
- [x] 版本內容檢視對話框
- [x] 附件管理前端 UI

### 實驗動物管理系統
- [x] 豬隻管理（列表、新增、編輯、詳情）
- [x] 依狀態篩選、依欄位分組檢視
- [x] 批次分配至計劃、批次進入實驗
- [x] 匯入基本資料（Excel/CSV 支援、資料驗證、批次匯入）
- [x] 匯入體重資料（Excel/CSV 支援、資料驗證、批次匯入）
- [x] 下載匯入範本（Excel 格式）
- [x] 下載匯入範本（CSV 格式）
- [x] 匯入批次記錄與錯誤追蹤
- [x] 觀察試驗紀錄完整表單（含用藥、儀器、照片上傳）
- [x] 手術紀錄完整表單（含麻醉、生理數值、術後給藥）
- [x] 複製紀錄功能
- [x] 版本歷史檢視
- [x] 獸醫師已讀標記、獸醫師建議
- [x] 體重、疫苗、犧牲、病理報告紀錄
- [x] 資料匯出（病歷、各類紀錄）
- [x] 豬隻來源管理

### 通知系統
- [x] 站內通知系統（Header 圖示、下拉選單）
- [x] 未讀數量顯示
- [x] 標記已讀、全部已讀

---

## ✅ v1.1 - 已完成

### 通知系統
- [x] 排程通知任務（低庫存、效期提醒背景排程）
- [x] 計畫狀態變更 Email 通知
- [x] 審查指派 Email 通知
- [x] 獸醫師建議 Email 通知

### 基礎架構
- [x] 檔案上傳後端實作（檔案儲存服務）
- [x] 資料匯入功能（Excel/CSV 支援、資料驗證、批次匯入）
- [x] CSV 範本下載功能
- [x] 豬隻編輯功能 - 新增 entry_date 欄位支援
- [x] 列表頁快速編輯功能 - 在進場日期旁添加編輯按鈕

### GLP 合規功能（2026-01-19 完成）
- [x] 軟刪除 + 刪除原因（前後端）
- [x] 變更原因記錄（change_reasons 表）
- [x] 審計日誌增強（old_value, new_value）
- [x] 電子簽章服務（後端 SignatureService + 前端整合）
- [x] 附註/更正服務（後端 AnnotationService + 前端對話框）
- [x] 記錄鎖定機制（後端 + 前端鎖定狀態顯示）

### HR 人事管理系統（2026-01-19 完成）
- [x] 特休額度管理 - 僅管理員可存取
- [x] 員工到職日期 (hire_date) 欄位支援
- [x] 週年制特休到期邏輯
- [x] 未休補償追蹤機制

### 本機開發
- [x] Rust 端資料庫連線重試機制（啟動時自動重試連線，適用於 Docker Compose 環境）
- [x] Docker 資料庫 healthcheck 優化
- [x] API 啟動失敗時明確的資料庫狀態日誌

---

## ✅ v1.2 - 已完成 (2026-02-02)

### 資料庫架構現代化
- [x] 豬隻 ID 遷移至 UUID - pig 模組主鍵從 INTEGER 改為 UUID，保留 `pig_no` 顯示
- [x] GIN 索引優化 - JSONB 欄位（protocols.working_content、pig_observations.treatments）
- [x] Array Foreign Key 完整性修復 - 透過 `leave_balance_usage` 處理 `comp_time_source_ids`
- [x] 自動 Partition 維護 - 排程器自動建立 `user_activity_logs` 新分區表
- [x] 編譯錯誤修復 - `display_name` 欄位錯誤、`create_weight` 類型不匹配（Uuid vs i32）

### Amendment 變更申請系統（後端完成）
- [x] 資料庫遷移 `022_amendment_system.sql` - amendments、amendment_versions、amendment_review_assignments、amendment_status_history
- [x] Models - AmendmentType、AmendmentStatus、Amendment、CreateAmendmentRequest 等
- [x] Services - AmendmentService 完整實作（建立、分類、審查、版本快照）
- [x] Handlers - REST API（建立、列表、詳情、更新、提交、分類、審查決定）
- [x] 權限設定 - PI/IACUC_STAFF/REVIEWER/CHAIR 各角色權限
- [x] Coeditor 草稿回覆 - review_comments 新增 draft_content、drafted_by、draft_updated_at

### 動物管理功能增強
- [x] 緊急用藥通知 - 觸發緊急通知警示獸醫師和 PI
- [x] 快速新增動物對話框 - 耳號未找到時手動新增，自動格式化
- [x] 安樂死工作流程 UI - 命令建立、PI 核准/申訴、CHAIR 仲裁

### 審查系統改進
- [x] 審查者匿名化 - PI/CLIENT 視角顯示 "Reviewer A/B/C"
- [x] AUP 表單翻譯 - ProtocolEditPage.tsx 章節標題中翻英

### 權限系統優化
- [x] 權限分類整合 - 合併重複「其他」類別，CRUD 操作分組至父模組
- [x] 階層結構 - 動物使用計畫、動物管理、庫存管理、管理階級、系統管理、開發工具

---

## ✅ v1.3 - 已完成 (2026-02-03)

### 使用者管理功能增強
- [x] 新增「學經歷」欄位 - 在使用者管理介面新增「學經歷」多行文字框，支援建立與編輯
- [x] 資料庫遷移 `023_add_user_experience.sql` - users 表新增 `experience` 欄位
- [x] 端點更新 - `UserService` 與相關 DTO 支援 `experience` 欄位
- [x] 新增「entry_date」入職日期欄位 - 用於計算員工年資 `years_experience`
- [x] 新增「position」、「aup_roles」、「trainings」欄位 - 完善使用者資料結構

### 角色權限系統強化
- [x] 角色權限驗證 - 確保每個權限名稱正確，各角色擁有適當的存取權限
- [x] Admin 完整權限 - 確保系統管理員（admin）擁有所有權限
- [x] 使用者建立錯誤修復 - 解決 422 BusinessRule 錯誤，修正權限檢查與資料驗證邏輯

### AUP 審查系統改進
- [x] 條件式人員對話框 - PI 角色顯示文字輸入框，Co-editor 角色顯示員工下拉選單
- [x] 人員名稱修正 - 更新「許芮蓁」為「芮蓁」
- [x] Protocol 表單錯誤修復 - 修正 personnel、training_certificates、roles 陣列初始化問題

### 獸醫師通知增強
- [x] 緊急旗標功能 - 獸醫師建議新增 `is_urgent` 欄位
- [x] 差異化通知發送 - 緊急建議觸發站內通知 + Email，一般建議僅觸發站內通知
- [x] 後端服務更新 - AnimalService、NotificationService、EmailService 支援緊急旗標邏輯

### 編譯錯誤修復
- [x] UserResponse struct 更新 - 新增 entry_date、experience、position、aup_roles、years_experience、trainings 欄位
- [x] JSX 語法修復 - UsersPage.tsx 標籤正確閉合
- [x] Docker 容器建置成功

---

## 📋 v2.0 規劃功能

### iPig ERP 系統
- [ ] 條碼掃描功能（行動裝置支援）
- [x] Excel/CSV 匯入匯出（實驗動物管理系統已完整實作）
- [ ] 進階成本法（FIFO）
- [ ] 批號效期到期提醒進階設定
- [ ] 作廢已核准單據：沖銷機制（Reversal Document）
- [ ] 庫位管理（Bin Location）
- [ ] 與會計/ERP API 對接

### AUP 審查系統
- [x] 後端 PDF 產生端點（全 9 節完整呈現）
- [x] 協編者權限修正
- [x] 審查意見回覆功能
- [x] 計畫書頁面整合
- [ ] 前端 TypeScript 型別統一
- [ ] 版本比較功能

### 實驗動物管理
- [ ] 疼痛評估紀錄時間軸檢視
- [ ] 獸醫師建議通知機制進階設定
- [ ] 空欄位快速移動功能

### 使用者體驗
- [x] 通知偏好設定
- [ ] 行動端響應式設計

---

## 變更紀錄

| 日期 | 內容 |
|------|------|
| 2026-02-03 | 角色權限驗證、Admin 完整權限、使用者建立錯誤修復、條件式人員對話框、獸醫師通知緊急旗標、entry_date 欄位、編譯錯誤修復、Docker 建置成功 |
| 2026-02-03 | 新增學經歷欄位（後端/使用者管理前端）、規劃 AUP 第 8 節整合 |
| 2026-02-02 | 新增今日未完成事項：UUID 遷移測試、Emergency Medication UI、Euthanasia Workflow、AUP 翻譯驗證、Reviewer Anonymization 測試、權限分類驗證 |
| 2026-01-19 | GLP 前端整合完成、HR 特休管理、協編者權限、PDF 全 9 節完成、審查意見回覆、頁面整合 |
| 2026-01-19 | GLP 合規功能：軟刪除+刪除原因、電子簽章、附註服務（後端完成） |
| 2026-01-11 | 完整實作資料匯入功能（Excel/CSV 支援、資料驗證、批次匯入、CSV 範本下載） |
| 2026-01-09 | AUP 審查系統 UI 完善（審查意見、指派、附件、版本檢視） |
| 2026-01-08 | v1.0 MVP 完成，更新 v1.1、v2.0 規劃 |


