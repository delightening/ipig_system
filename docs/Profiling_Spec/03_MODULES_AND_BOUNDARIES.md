# 模組與邊界

> **版本**：5.0  
> **最後更新**：2026-02-17  
> **對象**：架構師、資深開發人員

---

## 1. 系統拆解

iPig 系統組織成獨立的有界上下文：

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              iPig System                                    │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────────────┐  │
│  │  AUP 審查系統   │  │   iPig ERP     │  │  動物管理系統               │  │
│  │  (計畫書)       │  │  (進銷存)      │  │  (動物)                     │  │
│  └────────┬────────┘  └────────┬───────┘  └────────────┬────────────────┘  │
│           │                     │                        │                   │
│  ┌────────┴────────┐  ┌────────┴───────┐  ┌────────────┴────────────────┐  │
│  │  變更申請系統   │  │  倉庫儲位系統  │  │  血液檢查系統               │  │
│  │  (Amendment)    │  │  (Storage)     │  │  (Blood Test)               │  │
│  └─────────────────┘  └────────────────┘  └─────────────────────────────┘  │
│                                                                              │
│                         橫切關注點                                           │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│   │  認證系統   │  │  人事管理   │  │  通知系統   │  │  安全與稽核      │  │
│   │  (Auth)     │  │  (HR)       │  │  (Notify)   │  │  (Audit)         │  │
│   └─────────────┘  └─────────────┘  └─────────────┘  └──────────────────┘  │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 認證模組

**目的**：使用者認證、授權、工作階段管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 認證處理器 | `handlers/auth.rs` (11KB) | 登入/登出/Token |
| 認證服務 | `services/auth.rs` (23KB) | 認證邏輯 |
| 登入追蹤 | `services/login_tracker.rs` (15KB) | 登入失敗偵測 |
| 工作階段 | `services/session_manager.rs` (6KB) | 工作階段管理 |
| 認證中間件 | `middleware/auth.rs` (3KB) | 請求認證 |
| Rate Limiter | `middleware/rate_limiter.rs` (6KB) | 速率限制 |
| Real IP | `middleware/real_ip.rs` (2KB) | 真實 IP 擷取 |
| GeoIP | `services/geoip.rs` (3KB) | IP 地理定位 |

**API 前綴**：`/api/auth/*`、`/api/me`

**主要端點**：
- `POST /auth/login` - 使用者登入
- `POST /auth/refresh` - Token 刷新
- `POST /auth/logout` - 使用者登出
- `POST /auth/heartbeat` - 心跳回報
- `POST /auth/forgot-password` - 密碼重設請求
- `POST /auth/reset-password` - 使用 Token 重設
- `GET/PUT /me` - 個人資訊
- `PUT /me/password` - 變更密碼
- `GET/PUT/DELETE /me/preferences/:key` - 偏好設定

---

## 3. 使用者管理模組

**目的**：使用者 CRUD、角色管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 使用者處理器 | `handlers/user.rs` (7KB) | 使用者操作 |
| 使用者服務 | `services/user.rs` (10KB) | 管理邏輯 |
| 使用者模型 | `models/user.rs` (11KB) | 資料結構 |
| 角色處理器 | `handlers/role.rs` (4KB) | 角色操作 |
| 角色服務 | `services/role.rs` (8KB) | 角色邏輯 |
| 偏好設定處理器 | `handlers/user_preferences.rs` (4KB) | 使用者偏好 |
| 偏好設定模型 | `models/user_preferences.rs` (6KB) | 偏好結構 |

**主要端點**：
- `GET/POST /users` - 使用者列表/建立
- `GET/PUT/DELETE /users/:id` - 使用者 CRUD
- `PUT /users/:id/password` - 重設密碼
- `POST /users/:id/impersonate` - 模擬登入
- `GET/POST /roles` - 角色列表/建立
- `GET /permissions` - 權限列表

---

## 4. AUP 審查系統

**目的**：IACUC 計畫書管理、多層審查流程。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 計畫書處理器 | `handlers/protocol/crud.rs` (15KB) | 計畫 CRUD |
| 計畫書處理器 | `handlers/protocol/review.rs` (14KB) | 審查操作 |
| 計畫書處理器 | `handlers/protocol/export.rs` (3KB) | PDF 匯出 |
| 計畫書服務 | `services/protocol/core.rs` (11KB) | 核心邏輯 |
| 計畫書服務 | `services/protocol/status.rs` (21KB) | 狀態機 |
| 計畫書服務 | `services/protocol/review.rs` (11KB) | 審查邏輯 |
| 計畫書服務 | `services/protocol/numbering.rs` (5KB) | 編號產生 |
| 計畫書服務 | `services/protocol/history.rs` (7KB) | 版本歷程 |
| 計畫書服務 | `services/protocol/comment.rs` (10KB) | 審查意見 |
| 計畫書服務 | `services/protocol/my_protocols.rs` (7KB) | 我的計畫 |
| 計畫書模型 | `models/protocol.rs` (25KB) | 資料結構 |

**API 前綴**：`/api/protocols/*`、`/api/reviews/*`、`/api/my-projects`

**主要端點**：
- `GET/POST /protocols` - 計畫列表/建立
- `GET/PUT /protocols/:id` - 計畫詳情/更新
- `POST /protocols/:id/submit` - 送審
- `POST /protocols/:id/status` - 變更狀態
- `GET /protocols/:id/versions` - 版本歷程
- `GET /protocols/:id/activities` - 活動紀錄
- `GET /protocols/:id/animal-stats` - 動物統計
- `GET /protocols/:id/export-pdf` - 匯出 PDF
- `GET/POST /protocols/:id/co-editors` - 共同編輯者
- `GET/POST /reviews/assignments` - 審查指派
- `GET/POST /reviews/comments` - 審查意見
- `POST /reviews/comments/:id/resolve` - 解決意見
- `POST /reviews/comments/reply` - 回覆意見
- `POST/GET /reviews/comments/draft` - 草稿回覆
- `POST /reviews/vet-form` - 獸醫審查表單
- `GET /my-projects` - 我的計畫

---

## 5. 變更申請系統 (Amendment)

**目的**：計畫書核准後的變更管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 變更處理器 | `handlers/amendment.rs` (14KB) | 變更操作 |
| 變更模型 | `models/amendment.rs` (8KB) | 資料結構 |
| 變更通知 | `services/notification/amendment.rs` (9KB) | 通知發送 |

**API 前綴**：`/api/amendments/*`

**主要端點**：
- `GET/POST /amendments` - 變更列表/建立
- `GET/PATCH /amendments/:id` - 變更詳情/更新
- `POST /amendments/:id/submit` - 送審
- `POST /amendments/:id/classify` - 分類
- `POST /amendments/:id/start-review` - 開始審查
- `POST /amendments/:id/decision` - 審查決定
- `POST /amendments/:id/status` - 狀態變更
- `GET /amendments/:id/versions` - 版本歷程
- `GET /amendments/:id/history` - 異動歷程
- `GET /amendments/:id/assignments` - 審查指派
- `GET /amendments/pending-count` - 待處理數量
- `GET /protocols/:id/amendments` - 計畫下的變更

---

## 6. 動物管理系統

**目的**：實驗動物完整生命週期管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 動物處理器 | `handlers/animal/animal.rs` (8KB) | 基本操作 |
| 觀察紀錄處理器 | `handlers/animal/observation.rs` (8KB) | 觀察 CRUD |
| 手術紀錄處理器 | `handlers/animal/surgery.rs` (6KB) | 手術 CRUD |
| 體重疫苗處理器 | `handlers/animal/weight_vaccination.rs` (8KB) | 體重/疫苗 |
| 血液檢查處理器 | `handlers/animal/blood_test.rs` (14KB) | 血檢 CRUD |
| 犧牲病理處理器 | `handlers/animal/sacrifice_pathology.rs` (4KB) | 犧牲/病理 |
| 匯入匯出處理器 | `handlers/animal/import_export.rs` (10KB) | Excel 匯入 |
| 獸醫建議處理器 | `handlers/animal/vet_recommendation.rs` (9KB) | 獸醫建議 |
| 來源管理處理器 | `handlers/animal/source.rs` (2KB) | 動物來源 |
| Dashboard 處理器 | `handlers/animal/dashboard.rs` (2KB) | 動物 Dashboard |
| 核心服務 | `services/animal/core.rs` (18KB) | 動物核心邏輯 |
| 醫療服務 | `services/animal/medical.rs` (18KB) | 醫療紀錄 |
| 觀察服務 | `services/animal/observation.rs` (9KB) | 觀察紀錄 |
| 手術服務 | `services/animal/surgery.rs` (9KB) | 手術紀錄 |
| 血檢服務 | `services/animal/blood_test.rs` (20KB) | 血液檢查 |
| 匯入匯出服務 | `services/animal/import_export.rs` (34KB) | Excel 處理 |

**API 前綴**：`/api/animals/*`、`/api/observations/*`、`/api/surgeries/*`、`/api/weights/*`、`/api/vaccinations/*`、`/api/blood-tests/*`、`/api/blood-test-templates/*`、`/api/blood-test-panels/*`

**主要端點**（共 42 個）：
- `GET/POST /animals` - 動物列表/建立
- `GET/PUT/DELETE /animals/:id` - 動物 CRUD
- `GET /animals/by-pen` - 依欄位分組
- `POST /animals/batch/assign` - 批次分配
- `GET /animals/vet-comments` - 獸醫待閱
- `POST /animals/:id/vet-read` - 標記已讀
- `GET/POST /animals/:id/observations` - 觀察紀錄
- `POST /animals/:id/observations/copy` - 複製紀錄
- `GET/POST /animals/:id/surgeries` - 手術紀錄
- `GET/POST /animals/:id/weights` - 體重紀錄
- `GET/POST /animals/:id/vaccinations` - 疫苗紀錄
- `GET/POST /animals/:id/sacrifice` - 犧牲紀錄
- `GET/POST /animals/:id/pathology` - 病理報告
- `GET/POST /animals/:id/blood-tests` - 血液檢查
- `GET/PUT/DELETE /blood-tests/:id` - 血檢 CRUD
- `GET/POST /blood-test-templates` - 模板管理
- `GET/POST /blood-test-panels` - 組合管理
- `POST /animals/:id/export` - 醫療資料匯出
- `POST /projects/:iacuc_no/export` - 計畫醫療匯出
- `GET/POST /animals/import/*` - 批次匯入
- `POST /animals/:id/sudden-death` - 猝死登記
- `GET /animals/:id/data-boundary` - 資料隔離邊界
- `POST /animals/:id/transfers` - 發起轉讓
- `GET /animals/:id/transfers` - 轉讓紀錄
- `GET /transfers/:id` - 轉讓詳情
- `POST /transfers/:id/source-pi-confirm` - 來源 PI 確認
- `POST /transfers/:id/vet-evaluate` - 獸醫評估
- `POST /transfers/:id/target-pi-confirm` - 目標 PI 確認
- `POST /transfers/:id/iacuc-approve` - IACUC 核准
- `POST /transfers/:id/complete` - 執行完成

---

## 7. 安樂死管理系統 (Euthanasia)

**目的**：安樂死申請核准流程。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 安樂死處理器 | `handlers/euthanasia.rs` (5KB) | 操作處理 |
| 安樂死服務 | `services/euthanasia.rs` (16KB) | 商業邏輯 |
| 安樂死模型 | `models/euthanasia.rs` (6KB) | 資料結構 |
| 安樂死通知 | `services/notification/euthanasia.rs` (4KB) | 通知 |

**API 前綴**：`/api/euthanasia/*`

**主要端點**：
- `POST /euthanasia/orders` - 建立申請
- `GET /euthanasia/orders/pending` - 待核准
- `GET /euthanasia/orders/:id` - 看詳情
- `POST /euthanasia/orders/:id/approve` - 核准
- `POST /euthanasia/orders/:id/appeal` - 申訴
- `POST /euthanasia/orders/:id/execute` - 執行
- `POST /euthanasia/appeals/:id/decide` - 申訴裁決

---

## 8. 電子簽章模組 (GLP Compliance)

**目的**：GLP 法規合規的電子簽章與紀錄標註。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 簽章處理器 | `handlers/signature.rs` (10KB) | 簽章操作 |
| 簽章服務 | `services/signature.rs` (12KB) | 簽章邏輯 |

**API 前綴**：`/api/signatures/*`、`/api/annotations/*`

**主要端點**：
- `POST /signatures/sacrifice/:id` - 簽署犧牲紀錄（密碼/手寫）
- `GET /signatures/sacrifice/:id` - 犧牲簽章狀態
- `POST /signatures/observation/:id` - 簽署觀察紀錄
- `POST /signatures/euthanasia/:id` - 簽署安樂死紀錄
- `GET /signatures/euthanasia/:id/status` - 安樂死簽章狀態
- `POST /signatures/transfer/:id` - 簽署轉讓紀錄
- `GET /signatures/transfer/:id/status` - 轉讓簽章狀態
- `POST /signatures/protocol/:id` - 簽署計畫審查
- `GET /signatures/protocol/:id/status` - 計畫簽章狀態
- `GET /annotations/:record_type/:record_id` - 取得標註
- `POST /annotations/:record_type/:record_id` - 新增標註

---

## 9. ERP 進銷存系統

**目的**：產品、庫存、採購、銷售管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 單據處理器 | `handlers/document.rs` (8KB) | 單據操作 |
| 單據服務 | `services/document/` (5 檔) | 商業邏輯 |
| 產品處理器 | `handlers/product.rs` (5KB) | 產品管理 |
| 產品服務 | `services/product.rs` (14KB) | 產品邏輯 |
| SKU 處理器 | `handlers/sku.rs` (3KB) | SKU 產生 |
| SKU 服務 | `services/sku.rs` (18KB) | SKU 邏輯 |
| 庫存處理器 | `handlers/stock.rs` (1KB) | 庫存查詢 |
| 庫存服務 | `services/stock.rs` (20KB) | 庫存追蹤 |
| 倉庫處理器 | `handlers/warehouse.rs` (4KB) | 倉庫管理 |
| 倉庫服務 | `services/warehouse.rs` (6KB) | 倉庫邏輯 |
| 儲位處理器 | `handlers/storage_location.rs` (6KB) | 儲位操作 |
| 儲位服務 | `services/storage_location.rs` (21KB) | 儲位邏輯 |
| 夥伴處理器 | `handlers/partner.rs` (5KB) | 供應商/客戶 |
| 夥伴服務 | `services/partner.rs` (11KB) | 夥伴邏輯 |

**API 前綴**：`/api/products/*`、`/api/documents/*`、`/api/inventory/*`、`/api/warehouses/*`、`/api/storage-locations/*`、`/api/partners/*`、`/api/sku/*`

---

## 10. 人事管理系統

**目的**：出勤、請假、加班、行事曆同步。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 出勤處理器 | `handlers/hr/attendance.rs` (2KB) | 打卡操作 |
| 請假處理器 | `handlers/hr/leave.rs` (8KB) | 請假操作 |
| 加班處理器 | `handlers/hr/overtime.rs` (6KB) | 加班操作 |
| 餘額處理器 | `handlers/hr/balance.rs` (4KB) | 假期餘額 |
| Dashboard 處理器 | `handlers/hr/dashboard.rs` (7KB) | HR 儀表板 |
| 行事曆處理器 | `handlers/calendar.rs` (5KB) | 行事曆同步 |
| 行事曆服務 | `services/calendar.rs` (26KB) | 同步邏輯 |
| Google Calendar | `services/google_calendar.rs` (17KB) | Google API |
| 餘額到期 | `services/balance_expiration.rs` (6KB) | 餘額管理 |
| HR 模型 | `models/hr.rs` (17KB) | HR 結構 |

**API 前綴**：`/api/hr/*`

---

## 11. 通知系統

**目的**：Email 通知、站內通知、排程任務。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 通知處理器 | `handlers/notification.rs` (10KB) | 通知操作 |
| 通知 CRUD | `services/notification/crud.rs` (7KB) | 基礎 CRUD |
| 通知 Helpers | `services/notification/helpers.rs` (2KB) | 輔助函數 |
| Protocol 通知 | `services/notification/protocol.rs` (17KB) | 計畫相關 |
| Animal 通知 | `services/notification/animal.rs` (8KB) | 動物相關 |
| Amendment 通知 | `services/notification/amendment.rs` (9KB) | 變更相關 |
| Euthanasia 通知 | `services/notification/euthanasia.rs` (4KB) | 安樂死相關 |
| Alert 通知 | `services/notification/alert.rs` (7KB) | 庫存/效期 |
| ERP 通知 | `services/notification/erp.rs` (3KB) | 單據相關 |
| HR 通知 | `services/notification/hr.rs` (4KB) | 人事相關 |
| Report 通知 | `services/notification/report.rs` (5KB) | 報表相關 |
| Email | `services/email/` (4 檔) | SMTP 發送 |
| 排程器 | `services/scheduler.rs` (22KB) | Cron 排程 |

**API 前綴**：`/api/notifications/*`、`/api/alerts/*`、`/api/scheduled-reports/*`、`/api/admin/notification-routing/*`

**新增：通知路由管理**：
- `GET/POST/PUT/DELETE /admin/notification-routing` - 路由 CRUD

---

## 12. 安全與稽核系統

**目的**：GLP 合規稽核、異常偵測、工作階段管理。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 稽核處理器 | `handlers/audit.rs` (7KB) | 稽核操作 |
| 稽核服務 | `services/audit.rs` (22KB) | 稽核邏輯 |
| 稽核模型 | `models/audit.rs` (9KB) | 資料結構 |
| Activity Logger | `middleware/activity_logger.rs` (7KB) | 活動記錄中間件 |

**API 前綴**：`/api/admin/audit/*`

---

## 13. 設施管理模組

**目的**：管理物種、設施、棟舍、區域、欄位及部門。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 設施處理器 | `handlers/facility.rs` (10KB) | 設施操作 |
| 設施服務 | `services/facility.rs` (18KB) | 設施邏輯 |
| 設施模型 | `models/facility.rs` (11KB) | 資料結構 |

**API 前綴**：`/api/facilities/*`

**主要端點**：
- `GET/POST /facilities/species` - 物種管理
- `GET/POST /facilities` - 設施 CRUD
- `GET/POST /facilities/buildings` - 棟舍 CRUD
- `GET/POST /facilities/zones` - 區域 CRUD
- `GET/POST /facilities/pens` - 欄位 CRUD
- `GET/POST /facilities/departments` - 部門 CRUD

---

## 14. 報表系統

**目的**：庫存、採購、銷售、成本、血檢報表。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 報表處理器 | `handlers/report.rs` (3KB) | 報表端點 |
| 報表服務 | `services/report.rs` (20KB) | 報表邏輯 |

**API 前綴**：`/api/reports/*`、`/api/scheduled-reports/*`

**主要端點**：
- `GET /reports/stock-on-hand` - 現有庫存
- `GET /reports/stock-ledger` - 異動歷程
- `GET /reports/purchase-lines` - 採購明細
- `GET /reports/sales-lines` - 銷售明細
- `GET /reports/cost-summary` - 成本分析
- `GET /reports/blood-test-cost` - 血檢成本
- `GET /reports/blood-test-analysis` - 血檢分析
- `GET/POST /scheduled-reports` - 排程報表
- `GET /report-history` - 報表歷程

---

## 15. 檔案上傳系統

**目的**：管理各類附件上傳與下載。

| 元件 | 檔案路徑 | 說明 |
|------|----------|------|
| 上傳處理器 | `handlers/upload.rs` (16KB) | 上傳操作 |
| 檔案服務 | `services/file.rs` (20KB) | 檔案管理 |
| PDF 服務 | `services/pdf/` (3 檔) | PDF 產生 |

**支援的上傳類型**：
- 計畫書附件 (`/protocols/:id/attachments`)
- 動物照片 (`/animals/:id/photos`)
- 病理報告附件 (`/animals/:id/pathology/attachments`)
- 犧牲照片 (`/animals/:id/sacrifice/photos`)
- 獸醫建議附件 (`/vet-recommendations/:record_type/:record_id/attachments`)
- 請假附件 (`/hr/leaves/attachments`)

---

## 16. 前端路由總覽

```
/login                          → 登入頁
/forgot-password                → 忘記密碼
/reset-password                 → 重設密碼
/force-change-password          → 強制變更密碼

/dashboard                      → 儀表板
/profile/settings               → 個人設定

/erp                            → ERP 首頁
/products                       → 產品列表
/products/new                   → 新增產品
/products/:id                   → 產品詳情
/warehouses                     → 倉庫管理
/partners                       → 夥伴管理
/blood-test-templates           → 血檢模板
/blood-test-panels              → 血檢組合

/documents                      → 單據列表
/documents/new                  → 新增單據
/documents/:id                  → 單據詳情

/inventory                      → 庫存查詢
/inventory/ledger               → 庫存異動
/inventory/layout               → 倉庫儲位

/reports                        → 報表中心
/reports/stock-on-hand           → 庫存報表
/reports/stock-ledger            → 異動報表
/reports/purchase-lines          → 採購報表
/reports/sales-lines             → 銷售報表
/reports/cost-summary            → 成本報表
/reports/blood-test-cost         → 血檢成本
/reports/blood-test-analysis     → 血檢分析

/protocols                      → 計畫列表
/protocols/new                  → 新增計畫
/protocols/:id                  → 計畫詳情
/protocols/:id/edit             → 編輯計畫

/my-projects                    → 我的計劃
/my-projects/:id                → 計劃詳情
/my-amendments                  → 我的變更

/animals                        → 動物列表
/animals/:id                    → 動物詳情（多 Tab）
/animals/:id/edit               → 編輯動物
/animal-sources                 → 動物來源

/hr/attendance                  → 出勤打卡
/hr/leaves                      → 請假管理
/hr/overtime                    → 加班管理
/hr/annual-leave                → 特休管理
/hr/calendar                    → 行事曆設定

/admin/users                    → 使用者管理
/admin/roles                    → 角色管理
/admin/settings                 → 系統設定
/admin/audit-logs               → 稽核日誌
/admin/audit                    → 安全審計
```

---

*下一章：[資料庫綱要](./04_DATABASE_SCHEMA.md)*
