# 豬博士動物科技系統規格書

## 系統總覽與整合架構

### 1. 系統概述

豬博士 iPig 系統為一套整合型實驗動物管理平台，採用**統一入口門戶**架構，使用者透過單一登入點存取所有子系統功能。

---

### 2. 子系統組成

| 子系統 | 主要功能 | 目標使用者 |
|-------|---------|-----------|
| **AUP 提交與審查系統** | IACUC 動物試驗計畫書的撰寫、提交、審查、核准流程 | PI、審查委員、IACUC 行政人員、試驗工作人員 |
| **iPig ERP (進銷存管理系統)** | 物資採購、庫存、成本管理（飼料、藥品、器材、耗材） | 倉庫管理員、系統管理員 |
| **實驗動物管理系統** | 豬隻分配、實驗紀錄、健康監控、病歷管理 | PI、獸醫師、試驗工作人員、委託單位 |
| **人員管理系統** | 請假流程、補休累計、特休假管理、審核簽核 | 公司內部員工（不包含審查人員、IACUC 主席等外部角色） |

---

### 3. 系統架構圖

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      豬博士 iPig 統一入口門戶                             │
│                                                                         │
│  ┌─────────────┐      ┌─────────────────┐      ┌─────────────────────┐  │
│  │   登入認證   │ ──── │   角色權限控管   │ ──── │   功能路由分派       │  │
│  └─────────────┘      └─────────────────┘      └─────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐   │
│  │ 1. AUP 審查系統    │  │ 2. iPig ERP 系統    │  │ 3. 實驗動物管理    │   │
│  │                   │  │                   │  │                   │   │
│  │ • 計畫書撰寫       │  │ • 採購            │  │ • 我的計劃        │   │
│  │ • 提交與版本控管   │  │ • 庫存盤點         │  │ • 豬隻管理        │   │
│  │ • IACUC 審查      │  │ • 成本追蹤         │  │ • 實驗紀錄        │   │
│  │ • 核准/修訂/否決   │  │                   │  │ • 健康監控        │   │
│  └─────────┬─────────┘  └─────────┬─────────┘  └─────────┬─────────┘   │
│            │                      │                      │             │
│  ┌─────────┴─────────┐            │                      │             │
│  │ 4. 人員管理系統    │            │                      │             │
│  │                   │            │                      │             │
│  │ • 請假申請        │            │                      │             │
│  │ • 補休累計        │            │                      │             │
│  │ • 特休假管理       │            │                      │             │
│  │ • 審核簽核        │            │                      │             │
│  └─────────┬─────────┘            │                      │             │
│            │                      │                      │             │
│            └──────────────────────┼──────────────────────┘             │
│                                   │                                     │
├───────────────────────────────────┴─────────────────────────────────────┤
│                           共用資料層                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐              │
│  │ users   │ │  pigs   │ │protocols│ │   audit_logs    │              │
│  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 4. 資料流向與系統串接

#### 4.1 重要說明：進銷存與豬隻管理的關係

```
┌─────────────────────────────────────────────────────────────────┐
│                        資料範圍劃分                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   iPig ERP (進銷存管理系統)          實驗動物管理系統              │
│   ================                 ==================           │
│                                                                 │
│   管理對象：                        管理對象：                   │
│   • 飼料                           • 豬隻（pigs 表）             │
│   • 藥品                           • 實驗紀錄                   │
│   • 器材                           • 健康監控                   │
│   • 耗材                           • 病歷管理                   │
│   • 其他物資                                                    │
│                                                                 │
│   ⚠️ 不管理豬隻                     ⚠️ 不管理物資庫存            │
│                                                                 │
│   products 表                       pigs 表                     │
│   └── 物資庫存 (stock_ledger)       └── 豬隻紀錄 (pig_*)        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 4.2 主要資料流

```
┌──────────────────┐
│  AUP 審查系統     │
│  (計畫書核准)     │
└────────┬─────────┘
         │ 產生 IACUC NO.（如 PIG-114017）
         │
         │  ┌─────────────────────────────────────────┐
         │  │  iPig ERP (進銷存管理系統)│
         │  │  • 採購飼料、藥品                        │
         │  │  • 庫存管理                              │
         │  │  • 供應實驗所需物資                      │
         │  └─────────────────────────────────────────┘
         │                    （物資供應）
         ▼                         ↓
┌──────────────────┐               ↓
│  實驗動物管理系統 │ ←─────────────┘
└────────┬─────────┘
         │
         ├─► 豬隻入場登錄（pigs 表，狀態 = 未分配）
         │
         ├─► 分配豬隻至計劃（關聯 IACUC NO.，狀態 → 已分配）
         │
         ├─► 進入實驗（狀態 → 實驗中）
         │
         ├─► 實驗紀錄（觀察、手術、體重、疫苗、用藥）
         │
         ├─► 犧牲/採樣、病理報告
         │
         └─► 完成實驗（狀態 → 實驗完畢）
                 │
                 ▼
         ┌──────────────────┐
         │  匯出病歷總表     │
         │  回饋至計畫結案   │
         └──────────────────┘
```

#### 4.3 系統間資料關聯

| 關聯 | 說明 |
|-----|------|
| protocols.iacuc_no → pigs.iacuc_no | 計畫與豬隻的關聯 |
| protocols → 實驗動物管理.我的計劃 | 計畫主持人查看自己的計畫 |
| products（進銷存）⊥ pigs（動物管理） | **無直接關聯，分開管理** |

---

### 5. 統一角色權限模型

#### 5.1 全系統角色定義

| 角色代碼 | 角色名稱 | 說明 |
|---------|---------|------|
| SYSTEM_ADMIN | 系統管理員 | 全系統最高權限，使用者管理、豬隻設定、系統維運 |
| WAREHOUSE_MANAGER | 倉庫管理員 | 專責 ERP 進銷存系統（採購、庫存、盤點、報表） |
| PROGRAM_ADMIN | 程式管理員 | 系統程式層級管理 |
| PI | 計畫主持人 | 提交計畫、管理自己的計畫與豬隻 |
| VET | 獸醫師 | 審查計畫、豬隻健康管理、提供建議 |
| REVIEWER | 審查委員 | IACUC 計畫審查 |
| CHAIR | IACUC 主席 | 主導審查決策 |
| IACUC_STAFF | 執行秘書 | 行政流程管理、管理所有計劃進度 |
| EXPERIMENT_STAFF | 試驗工作人員 | 執行實驗操作、記錄數據、查詢 ERP 物資現況 |
| CLIENT | 委託人 | 查看委託計畫與豬隻紀錄（同單位可多人多計劃，各自獨立帳號） |

---

#### 5.2 角色與子系統功能對應矩陣

| 角色 | AUP 審查系統 | 進銷存管理 | 實驗動物管理 |
|-----|-------------|-----------|-------------|
| SYSTEM_ADMIN | ✓ 全部 | ✓ 全部 | ✓ 全部 |
| WAREHOUSE_MANAGER | ✗ | ✓ 全部 | ✗ |
| PROGRAM_ADMIN | ✓ 全部 | ✓ 全部 | ✓ 全部 |
| CHAIR | ✓ 審查決策 | ✗ | ○ 檢視 |
| IACUC_STAFF | ✓ 流程管理 | ✗ | ✓ 管理所有計劃 |
| REVIEWER | ✓ 審查意見 | ✗ | ✗ |
| VET | ✓ 審查意見 | ✗ | ✓ 健康管理、建議 |
| PI | ✓ 提交/修訂 | ✗ | ✓ 我的計劃、豬隻紀錄 |
| EXPERIMENT_STAFF | ✗ | ○ 查詢物資 | ✓ 實驗紀錄操作 |
| CLIENT | ○ 檢視自己的 | ✗ | ○ 檢視委託計畫 |

> ✓ 完整存取 ｜ ○ 唯讀/部分存取 ｜ ✗ 無權限

---

### 6. 統一認證機制

#### 6.1 認證流程

```
使用者 ──(帳號/密碼)──> 統一登入 ──(JWT Token)──> 各子系統 API
                              │
                              ├── Access Token（短效，API 認證）
                              └── Refresh Token（長效，Token 更新）
```

#### 6.2 認證 API

| 端點 | 方法 | 說明 |
|-----|------|------|
| `/auth/login` | POST | 使用者登入，回傳 JWT Token |
| `/auth/refresh` | POST | 更新 Access Token |
| `/auth/logout` | POST | 登出，撤銷 Token |
| `/auth/forgot-password` | POST | 寄送密碼重設信件 |
| `/auth/reset-password` | POST | 重設密碼 |

#### 6.3 請求標頭

所有需認證的 API 請求必須包含：

```
Authorization: Bearer {access_token}
```

---

### 7. 共用資料模型

#### 7.1 核心共用資料表

| 資料表 | 說明 | 關聯系統 |
|-------|------|---------|
| `users` | 使用者帳號（含 organization 欄位） | 全部 |
| `user_roles` | 使用者角色關聯 | 全部 |
| `protocols` | 計畫書主檔 | AUP、實驗動物 |
| `pigs` | 豬隻主檔 | 實驗動物 |
| `warehouses` | 倉庫主檔 | ERP |
| `pig_sources` | 豬隻來源主檔 | 實驗動物 |
| `products` | 產品主檔（SKU） | ERP、實驗動物（查詢） |
| `stock_ledger` | 庫存流水（進出庫紀錄） | ERP |
| `audit_logs` | 稽核紀錄 | 全部 |

> **SKU 編碼規則**：產品 SKU 採互動式生成，詳見 `skuSpec.md`。
> 
> **通知系統**：系統自動通知規格詳見 `notificationSpec.md`。

---

#### 7.2 產品與庫存資料模型（GLP 合規）

##### products 表（產品主檔）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| sku | VARCHAR(50) | 產品編碼（唯一） |
| name | VARCHAR(200) | 產品名稱 |
| category | ENUM | 類別：藥品/飼料/器材/耗材/其他 |
| spec | TEXT | 規格說明（如：10mg/ml 5ml裝） |
| base_unit | VARCHAR(20) | 基本單位（如：瓶） |
| pack_unit | VARCHAR(20) | 包裝單位（如：盒） |
| pack_qty | INTEGER | 每包裝含基本單位數（如：10 瓶/盒） |
| track_batch | BOOLEAN | 是否追蹤批號 |
| track_expiry | BOOLEAN | 是否追蹤效期 |
| safety_stock | DECIMAL | 安全庫存量 |
| reorder_point | DECIMAL | 補貨點 |
| is_active | BOOLEAN | 是否啟用 |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

##### stock_ledger 表（庫存流水 - GLP 追溯）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| product_id | UUID | FK → products.id |
| warehouse_id | UUID | FK → warehouses.id |
| trx_date | TIMESTAMP | 異動日期時間 |
| direction | ENUM | in（入庫）/ out（出庫）/ adjust |
| qty | DECIMAL | 異動數量（基本單位） |
| batch_no | VARCHAR(50) | 批號 |
| expiry_date | DATE | 效期 |
| doc_type | VARCHAR(20) | 來源單據類型 |
| doc_no | VARCHAR(50) | 來源單據編號 |
| remark | TEXT | 備註 |
| created_by | UUID | 操作人員 |
| created_at | TIMESTAMP | |

##### inventory_view（庫存現況視圖）

EXPERIMENT_STAFF 查詢用，顯示：
- 產品名稱、規格
- 批號、效期
- 現有數量（基本單位）
- 現有數量（包裝單位換算）
- 效期狀態（正常/即將到期/已過期）

```sql
-- 範例查詢：查看藥品庫存
SELECT 
  p.name,
  p.spec,
  sl.batch_no,
  sl.expiry_date,
  SUM(CASE WHEN sl.direction = 'in' THEN sl.qty 
           WHEN sl.direction = 'out' THEN -sl.qty 
           ELSE sl.qty END) as on_hand_qty,
  p.base_unit,
  CASE 
    WHEN sl.expiry_date < CURRENT_DATE THEN '已過期'
    WHEN sl.expiry_date < CURRENT_DATE + 30 THEN '即將到期'
    ELSE '正常'
  END as expiry_status
FROM products p
JOIN stock_ledger sl ON p.id = sl.product_id
WHERE p.category = '藥品'
GROUP BY p.id, sl.batch_no, sl.expiry_date;
```

##### GLP 合規要點

| 要求 | 實作方式 |
|-----|---------|
| 可追溯性 | stock_ledger 記錄每筆異動，含操作人員與時間 |
| 批號管理 | batch_no 欄位，藥品/疫苗強制填寫 |
| 效期管理 | expiry_date 欄位，系統提供到期提醒 |
| 說寫做合一 | audit_logs 記錄所有變更，不可刪除 |
| 數據完整性 | stock_ledger 僅新增不修改，調整用 adjust 類型 |

##### warehouses 表（倉庫主檔）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| code | VARCHAR(20) | 倉庫代碼（唯一） |
| name | VARCHAR(100) | 倉庫名稱 |
| address | TEXT | 地址 |
| is_active | BOOLEAN | 是否啟用 |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

##### audit_logs 表（稽核紀錄）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| actor_user_id | UUID | 操作人員 FK → users.id |
| action | VARCHAR(50) | 動作類型（CREATE/UPDATE/DELETE/APPROVE/STATUS_CHANGE） |
| entity_type | VARCHAR(50) | 實體類型（protocol/pig/document/user 等） |
| entity_id | UUID | 實體 ID |
| before | JSONB | 變更前資料（可為 null） |
| after | JSONB | 變更後資料（可為 null） |
| ip_address | VARCHAR(45) | 來源 IP |
| user_agent | TEXT | 瀏覽器資訊 |
| created_at | TIMESTAMP | 建立時間（不可修改） |

> **重要**：audit_logs 為僅新增表，不可更新或刪除記錄。

#### 7.3 跨系統關聯鍵

```
protocols.iacuc_no ←──────────────────→ pigs.iacuc_no
     │                                        │
     │    ┌───────────────────────────┐      │
     └───>│  實驗動物管理系統          │<─────┘
          │  (我的計劃 + 豬隻紀錄)     │
          └───────────────────────────┘
```

---

### 8. 統一前端路由規劃

| 路由 | 所屬系統 | 頁面說明 |
|-----|---------|---------|
| `/login` | 共用 | 統一登入頁 |
| `/forgot-password` | 共用 | 忘記密碼 |
| `/change-password` | 共用 | 變更密碼 |
| `/dashboard` | 共用 | 首頁儀表板（依角色顯示不同內容） |
| `/protocols/*` | AUP 系統 | 計畫書相關頁面 |
| `/inventory/*` | 進銷存系統 | 庫存管理相關頁面 |
| `/my-projects` | 實驗動物管理 | 我的計劃 |
| `/my-projects/:id` | 實驗動物管理 | 計劃詳情（申請表 + 豬隻紀錄） |
| `/pigs/*` | 實驗動物管理 | 豬隻管理相關頁面 |
| `/admin/*` | 共用 | 系統管理（使用者、角色、稽核） |

---

### 9. 整合效益

| 效益 | 說明 |
|-----|------|
| **單一登入 (SSO)** | 使用者只需一組帳密，跨系統無縫切換 |
| **資料一致性** | IACUC 核准 → 豬隻分配 → 實驗紀錄，資料自動串接 |
| **完整稽核軌跡** | 從申請到結案，全程可追溯 |
| **角色統一管理** | 一處設定，全系統生效 |
| **降低維運成本** | 單一部署、單一監控、統一更新 |
| **使用者體驗一致** | 統一 UI/UX 風格，降低學習成本 |

---

## 子系統詳細規格

### 1. 豬博士動物試驗申請書(AUP)提交與審查系統
IACUC 動物試驗計畫書（AUP）提交與審查系統規格書

#### 1. 系統目的與範圍
本系統用於管理 IACUC（Institutional Animal Care and Use Committee）動物試驗計畫書（Animal Use Protocol, AUP）的完整生命週期，包含草稿撰寫、提交、審查、修訂、核准、暫停與結案等流程  

系統支援角色權限控管、完整審查流程、版本控管、附件管理與稽核軌跡，適用於研究機構、CRO、動物試驗單位與其 IACUC 組織  

---

#### 2. 使用者角色與權限模型
##### 2.1 使用者角色
系統支援以下角色（每位使用者可同時擁有多個角色）：

- PI（計畫主持人）
- REVIEWER（審查委員）
- VET（獸醫師）
- CHAIR（IACUC 主席）
- IACUC_STAFF（執行秘書／行政人員，管理所有計劃進度）
- WAREHOUSE_MANAGER（倉庫管理員，專責 ERP）
- SYSTEM_ADMIN（系統管理員）

---
##### 2.2 角色能力概述

| 角色 | 核心能力 |
|----|----|
| PI | 建立與編輯草稿、提交計畫、回應審查意見 |
| REVIEWER / VET | 審查指派案件、新增審查意見 |
| CHAIR | 主導審查決策、核准或否決計畫 |
| IACUC_STAFF | 指派審查人員、管理流程狀態、管理所有計劃進度 |
| WAREHOUSE_MANAGER | 專責 ERP 進銷存管理 |
| SYSTEM_ADMIN | 全系統管理、使用者管理、系統維運 |

---

#### 3. 核心資料模型（資料庫實體）
##### 3.1 資料表一覽
- `users`  
  系統使用者（含 organization 欄位記錄所屬單位）  

- `user_roles`  
  使用者角色關係  

- `protocols`  
  計畫書主檔，包含目前狀態與草稿內容  

- `protocol_versions`  
  每次提交或重送所產生的不可變版本快照  

- `protocol_status_history`  
  狀態轉移歷程紀錄  

- `review_assignments`  
  審查人員與計畫書指派關係  

- `comments`  
  審查意見（綁定特定版本）  

- `attachments`  
  附件檔案與其中繼資料  

- `audit_logs`  
  系統稽核紀錄（所有變動操作）

---

#### 4. 計畫書狀態機（State Machine）

##### 4.1 狀態定義

- DRAFT（草稿）
- SUBMITTED（已提交）
- PRE_REVIEW（行政預審）
- UNDER_REVIEW（審查中）
- REVISION_REQUIRED（需修訂）
- RESUBMITTED（已重送）
- APPROVED（核准）
- APPROVED_WITH_CONDITIONS（附條件核准）
- DEFERRED（延後審議）
- REJECTED（否決）
- SUSPENDED（暫停）
- CLOSED（結案）

---

##### 4.2 允許的狀態轉移與角色

- DRAFT → SUBMITTED  
  角色：PI  

- SUBMITTED → PRE_REVIEW  
  角色：IACUC_STAFF、SYSTEM_ADMIN  

- PRE_REVIEW → UNDER_REVIEW  
  角色：IACUC_STAFF、CHAIR、SYSTEM_ADMIN  

- UNDER_REVIEW → REVISION_REQUIRED  
  角色：REVIEWER、VET、CHAIR、IACUC_STAFF  

- UNDER_REVIEW → APPROVED / APPROVED_WITH_CONDITIONS / REJECTED  
  角色：CHAIR、SYSTEM_ADMIN  

- UNDER_REVIEW → DEFERRED  
  角色：CHAIR、IACUC_STAFF  

- REVISION_REQUIRED → RESUBMITTED  
  角色：PI  

- RESUBMITTED → PRE_REVIEW / UNDER_REVIEW  
  角色：IACUC_STAFF、CHAIR  

- APPROVED / APPROVED_WITH_CONDITIONS → SUSPENDED  
  角色：CHAIR、IACUC_STAFF、SYSTEM_ADMIN  

- DEFERRED / SUSPENDED → UNDER_REVIEW  
  角色：CHAIR、IACUC_STAFF、SYSTEM_ADMIN  

- 任一狀態 → CLOSED  
  依狀態機規則與權限限制執行  

---

#### 5. 後端 API 規格

##### 5.1 驗證與身分

- POST `/auth/login`
- POST `/auth/refresh`

---
##### 5.3 使用者

- GET `/users/me`
- GET `/users`
- GET `/users/{user_id}`

---
##### 5.4 計畫書（Protocols）

- POST `/protocols`
- GET `/protocols`
- GET `/protocols/{protocol_id}`
- PATCH `/protocols/{protocol_id}`
- POST `/protocols/{protocol_id}/submit`
- POST `/protocols/{protocol_id}/status`
- GET `/protocols/{protocol_id}/versions`
- GET `/protocols/{protocol_id}/versions/{version_id}`
- GET `/protocols/{protocol_id}/status-history`

---
##### 5.5 審查流程

- POST `/reviews/assignments`
- GET `/reviews/assignments`
- POST `/reviews/comments`
- GET `/reviews/comments`
- GET `/reviews/comments/{comment_id}`
- POST `/reviews/comments/{comment_id}/resolve`

---
##### 5.6 附件管理

- POST `/attachments?protocol_version_id=...`
- GET `/attachments?protocol_version_id=...`
- GET `/attachments?protocol_id=...`
- GET `/attachments/{attachment_id}/download`

---
##### 5.7 稽核紀錄

- GET `/audit-logs`
- GET `/audit-logs/{log_id}`

---

#### 6. 前端頁面與路由

- `/login`  
  使用者登入  

- `/dashboard`  
  計畫書清單  

- `/protocol/new`  
  建立新計畫書（草稿）  

- `/protocol/:protocolId`  
  編輯、提交與查看計畫書  

---

#### 7. AUP 表單資料結構（working_content）

草稿內容儲存於 `protocols.working_content` 欄位，格式為 JSONB，並需具備版本化 schema  

建議章節結構：

- 1. 基本資料（GLP、PI、計畫名稱、期間）
- 2. 3Rs 原則說明
- 3. 試驗物質與對照組
- 4. 試驗流程與麻醉止痛
- 5. 參考文獻
- 6. 手術計畫
- 7. 動物資訊（種類、性別、數量、年齡、體重、來源、飼養）
- 8. 人員名單與職責
- 9. 流程圖與附件

---

#### 8. 檔案儲存機制

- 所有附件儲存於伺服器檔案系統（`UPLOAD_DIR`）
- 附件中繼資料存於 `attachments` 資料表
- 檔案存取必須符合計畫版本權限

---

#### 9. 安全與存取控制

- 使用 JWT Access Token 與 Refresh Token
- 所有資料變動必須寫入 `audit_logs`

---

#### 10. 非功能性需求

- 所有狀態變更具備可追蹤性
- 計畫書編號唯一，格式：`PROTO-YYYY-NNN`
- 審查流程必須嚴格遵守狀態機規則

---

#### 11. 已知缺口與後續改善方向

- 後端尚未提供 PDF 產生端點（目前由前端產生）
- 前端 TypeScript 型別與實際表單欄位不一致，需統一 schema
- 尚未實作審查、稽核與使用者管理 UI
- 建議導入 OpenAPI 規格並進行前後端自動生成

### 2. 豬博士 iPig ERP (進銷存管理系統)

進銷存系統 Spec（Rust + PostgreSQL + 前端 UI）

詳細規格請參考 `ERPSpec.md`

#### 2.1 ERP 角色與統一角色對應

本系統採用統一角色模型，ERPSpec.md 中定義的 ERP 角色對應如下：

| ERPSpec 角色 | 統一角色代碼 | 說明 |
|-------------|-------------|------|
| Admin（系統管理員） | SYSTEM_ADMIN | 全權管理 |
| Warehouse（倉管） | WAREHOUSE_MANAGER | 入庫/出庫/盤點/調撥 |
| Purchasing（採購） | WAREHOUSE_MANAGER | 供應商、採購單管理 |
| Sales（業務） | — | 本系統不使用（無銷售業務） |
| Finance（財務） | WAREHOUSE_MANAGER | 成本報表、匯出 |
| Approver（主管） | WAREHOUSE_MANAGER | 單據核准 |

> **說明**：進銷存系統僅限內部人員（`is_internal = true`）存取，外部人員完全不可見此模組。主要操作由 WAREHOUSE_MANAGER 角色負責，SYSTEM_ADMIN 具備全部權限。EXPERIMENT_STAFF 可查詢物資現況（唯讀）。

#### 2.2 庫存查詢 API（EXPERIMENT_STAFF 可用）

| 端點 | 方法 | 說明 | 權限 |
|-----|------|------|------|
| `/inventory/products` | GET | 查詢產品清單 | 內部人員 |
| `/inventory/on-hand` | GET | 查詢庫存現況（含批號、效期） | 內部人員 |
| `/inventory/on-hand?category=藥品` | GET | 依類別篩選 | 內部人員 |
| `/inventory/expiring?days=30` | GET | 查詢即將到期物資 | 內部人員 |

**回傳範例（庫存現況）：**
```json
{
  "items": [
    {
      "product_id": "uuid",
      "sku": "MED-001",
      "name": "XX牌藥物",
      "spec": "10mg/ml 5ml裝",
      "category": "藥品",
      "batch_no": "LOT2024001",
      "expiry_date": "2025-06-30",
      "on_hand_qty": 50,
      "base_unit": "瓶",
      "pack_qty": 10,
      "pack_unit": "盒",
      "on_hand_packs": 5,
      "expiry_status": "正常"
    }
  ]
}
```

---

### 3. 實驗動物管理系統

#### 1. 角色：
程式管理員
系統管理員
一般使用者
委託單位
試驗工作人員
獸醫師
計劃管理人

#### 2. sections
##### 1. 我的計劃

###### 1.1 計劃列表頁

**頁面說明：** 列出登入帳號所擁有/參與的計劃清單

**列表欄位：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| 申請案號 | String | 內部申請編號 |
| IACUC number | String | IACUC 核准編號（如 PIG-114017） |
| 委託人 | String | 計劃主持人姓名 |
| 委託單位 | String | 所屬機構 |
| 審查狀態 | Badge | 對應 AUP 系統狀態 |
| 計劃名稱 | String | 計劃標題 |
| 起迄執行日期 | Date Range | 計劃執行期間 |
| 詳細內容 | Button | 「檢視」按鈕，進入詳細頁 |

**審查狀態 Badge 對應：**
| 狀態 | 顏色 |
|-----|------|
| 草稿 | 灰色 |
| 審查中 | 藍色 |
| 需修訂 | 橘色 |
| 已核准 | 綠色 |
| 已結案 | 深灰 |

---

###### 1.2 計劃詳細頁（⚠️ 需討論）

**頁面結構：** 2 個 Tab 切換

| Tab | 內容來源 |
|-----|---------|
| 申請表 | AUP 系統的計畫書內容（唯讀） |
| 豬隻紀錄 | 該計劃下所有已分配豬隻清單 |

---

**Tab 1：申請表**

顯示來自 AUP 系統的計畫書內容：

- 1. 基本資料（GLP、PI、計畫名稱、期間）
- 2. 3Rs 原則說明
- 3. 試驗物質與對照組
- 4. 試驗流程與麻醉止痛
- 5. 參考文獻
- 6. 手術計畫
- 7. 動物資訊（種類、性別、數量、年齡、體重、來源、飼養）
- 8. 人員名單與職責
- 9. 流程圖與附件

**操作按鈕（依權限顯示）：**
- 下載 PDF
- 下載附件

---

**Tab 2：豬隻紀錄**

**列表欄位：**
| 欄位 | 類型 |
|-----|------|
| 系統號 | Integer |
| 耳號 | String |
| 欄位 | String |
| 豬隻狀態 | Badge |
| 品種 | String |
| 性別 | String |
| 進場日期 | Date |
| 動作 | Button |

**動作按鈕：**
- 檢視：進入豬隻詳細頁（7 個 Tab）

**下載功能（依權限）：**
- 下載病歷總表（選定豬隻）
- 下載觀察試驗紀錄
- 下載手術紀錄

---

###### 1.3 業務規則

| 項目 | 決議 |
|-----|------|
| 委託單位查看權限 | ✅ 可查看完整詳細頁（僅限自己的計畫） |
| 豬隻紀錄下載 | ✅ 支援批次下載（同一計劃內可勾選多筆） |
| 計劃結案後狀態 | ✅ 唯讀（不可新增或編輯紀錄） |
| 申請表同步 | ✅ 即時同步（AUP 系統修改後自動反映） |

###### 1.4 委託單位（CLIENT）權限細則

| 項目 | 決議 |
|-----|------|
| 下載病歷 | ✅ 可下載（病歷總表、觀察紀錄、手術紀錄） |
| 查看獸醫師建議 | ✅ 可見 |
| 結案後存取期限 | ✅ 永久（帳號有效期間皆可查閱歷史資料） |
| 帳號管理 | ✅ 同一委託單位可建立多人帳號，各自獨立 |

---

###### 1.4 相關 API 端點

- GET `/my-projects` - 取得我的計劃列表
- GET `/my-projects/{protocol_id}` - 取得計劃詳情
- GET `/my-projects/{protocol_id}/application` - 取得申請表內容
- GET `/my-projects/{protocol_id}/pigs` - 取得計劃下的豬隻列表
- GET `/my-projects/{protocol_id}/export/medical-records` - 匯出病歷總表
- GET `/my-projects/{protocol_id}/export/observations` - 匯出觀察試驗紀錄
- GET `/my-projects/{protocol_id}/export/surgeries` - 匯出手術紀錄

##### 2. 豬隻管理

###### 2.1 頁面概述
本頁面用於管理所有豬隻的基本資料、分配狀態、實驗狀態、用藥紀錄等，支援依豬隻狀態篩選與多種檢視模式。

---

###### 2.2 頁面標題與說明
- 標題：豬隻
- 副標題說明：包含狀態為「未分配」、「已分配」、「實驗中」的豬隻
- 右上角顯示：管理員 > 豬隻 > 清單（麵包屑導航）

---

###### 2.3 頁籤（Tabs）
| 頁籤名稱 | 篩選條件 | 可用功能按鈕 |
|---------|---------|-------------|
| 所有豬隻-欄位 | 顯示所有豬隻，依欄位分組 | +新增豬隻、匯入豬隻基本資料、匯入豬隻體重、下載試驗紀錄 |
| 已分配 | 豬隻狀態 = 已分配 | 進入實驗（批次操作） |
| 實驗中 | 豬隻狀態 = 實驗中 | |
| 完成實驗 | 豬隻狀態 = 實驗完畢 | |
| 所有豬隻 | 顯示所有豬隻（不分組） | |

---

###### 2.4 功能按鈕說明
| 按鈕名稱 | 樣式 | 出現頁籤 | 功能說明 |
|---------|------|---------|---------|
| ＋新增 豬隻 | 紫色實心 | 所有豬隻-欄位 | 開啟表單新增單筆豬隻資料 |
| 匯入豬隻基本資料 | 綠色實心 | 所有豬隻-欄位 | 批次匯入豬隻基本資訊（CSV/Excel） |
| 匯入豬隻體重 | 藍色實心 | 所有豬隻-欄位 | 批次匯入豬隻體重紀錄（CSV/Excel） |
| 下載試驗紀錄 | 白色邊框 | 所有豬隻-欄位 | 匯出選定豬隻之試驗紀錄 |
| 進入實驗 | 橘色實心 | 已分配 | 將勾選的已分配豬隻批次變更為「實驗中」狀態 |

---

###### 2.5 「所有豬隻-欄位」頁籤（依欄位分組檢視）

**區域分組：**
- A、C、D區（左側表格）
- B、E、F、G區（右側表格）

**表格欄位：**
| 欄位名稱 | 資料類型 | 說明 |
|---------|---------|------|
| 欄號 | String | 豬欄編號（如 A01、C05、D13） |
| 耳號 | String | 豬隻耳標號碼，橘色文字，可顯示多筆 |
| 獸醫師最後檢視 | Date | 獸醫師最近檢視日期 |
| 最新異常紀錄時間 | Date | 最近異常紀錄日期 |
| 獸醫師建議 | Date | 獸醫師建議日期（可為空） |
| 動作 | Button | 「內容」下拉按鈕 |

**狀態標示：**
- 淺藍色「未選」標籤：該豬隻尚未分配至任何計劃
- 黃色/橘色背景列：需特別關注的豬隻（如有近期異常紀錄）
- 空白欄位：該欄位目前無豬隻

---

###### 2.6 列表頁籤通用欄位（已分配/實驗中/完成實驗/所有豬隻）

**表格欄位定義：**
| 欄位名稱 | 資料類型 | 說明 | 可排序 |
|---------|---------|------|--------|
| （勾選框） | Checkbox | 批次選取用（僅「已分配」頁籤） | 否 |
| 系統號 | Integer | 系統內部唯一識別碼 | 是 |
| 耳號 | String | 豬隻耳標號碼（3位數），橘色文字 | 是 |
| 欄位 | String | 豬欄位置編號（如 A05、B10、D27） | 是 |
| IACUC NO. | String | 關聯的 IACUC 計劃編號（如 PIG-114017），未分配時為空 | 是 |
| 豬隻狀態 | Enum | 未分配／已分配／實驗中／實驗完畢，橘色文字 | 是 |
| 品種 | Enum | 迷你豬／白豬／其他 | 是 |
| 性別 | Enum | 公／母 | 是 |
| 用藥中 | Boolean | 是／否 | 是 |
| 最後用藥時間 | Date | 最近一次用藥日期（YYYY-MM-DD） | 是 |
| 獸醫師建議 | Date | 獸醫師建議日期（可為空） | 是 |
| 進場日期 | Date | 豬隻進入場區日期（YYYY-MM-DD） | 是 |
| 動作 | Dropdown | 「內容」下拉選單按鈕 | 否 |

---

###### 2.7 豬隻狀態流程

```
未分配 ──(分配至計劃)──> 已分配 ──(進入實驗)──> 實驗中 ──(完成實驗)──> 實驗完畢
```

| 狀態值 | 說明 |
|-------|------|
| 未分配 | 豬隻尚未指派至任何 IACUC 計劃 |
| 已分配 | 豬隻已指派至某 IACUC 計劃，但實驗尚未開始 |
| 實驗中 | 豬隻正在進行實驗 |
| 實驗完畢 | 豬隻已完成實驗 |

---

###### 2.8 品種與性別枚舉值

**品種（Breed）：**
| 值 | 說明 |
|---|------|
| 迷你豬 | Miniature Pig |
| 白豬 | White Pig |
| 其他 | Other |

**性別（Gender）：**
| 值 | 說明 |
|---|------|
| 公 | 雄性 |
| 母 | 雌性 |

---

###### 2.9 分頁與搜尋功能

| 功能 | 說明 |
|-----|------|
| 每頁筆數 | 下拉選單，預設 25 筆，選項：10/25/50/100 |
| 搜尋框 | 即時搜尋，可搜尋耳號、欄位、IACUC NO. 等欄位 |
| 分頁導覽 | 「上一頁」、頁碼按鈕、「下一頁」 |
| 紀錄統計 | 顯示「正在顯示 X 個紀錄中的 Y 至 Z 項」 |

---

###### 2.10 豬隻詳細頁面（7 個 Tab）

豬隻詳細頁面採用 Tab 導覽，共 7 個子模組：

| Tab 名稱 | 說明 | 資料類型 |
|---------|------|---------|
| 觀察試驗紀錄 | 日常觀察、異常紀錄、試驗操作 | 多筆列表 |
| 手術紀錄 | 手術過程、麻醉、術後照護 | 多筆列表 |
| 體重紀錄 | 體重測量歷程 | 多筆列表 |
| 疫苗/驅蟲紀錄 | 疫苗接種與驅蟲紀錄 | 多筆列表 |
| 犧牲/採樣紀錄 | 實驗結束後的處置 | 單筆 |
| 豬隻資料 | 豬隻基本資料 | 單筆 |
| 病理組織報告 | 病理報告檔案 | 單筆 + 附件 |

---

###### 2.10.1 觀察試驗紀錄

**列表欄位：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| （展開按鈕） | Button | 展開查看詳細內容 |
| 事件發生日期 | Date | 紀錄日期 |
| 內容 | Text | 紀錄摘要 |
| 不需用藥/停止用藥 | Boolean | 勾選表示已無需用藥 |
| 獸醫師建議 | Badge | 已讀/未讀狀態 |
| 記錄者 | String | 記錄人員姓名 |
| 獸醫師讀取 | Badge | 已讀（綠色）/未讀 |
| 動作 | Buttons | 檢視、編輯、修訂版本、刪除 |

**新增/編輯表單欄位：**
| 欄位 | 類型 | 必填 | 說明 |
|-----|------|-----|------|
| 耳號 | Display | - | 唯讀顯示 |
| 事件發生日期 | Date | ✓ | 日期選擇器 |
| 紀錄性質 | Radio | ✓ | 異常紀錄 / 試驗紀錄 / 觀察紀錄 |
| 使用儀器 | Checkbox | | C-arm、超音波、CT、MRI、X光、其他 |
| 麻醉開始時間 | DateTime | | 時間選擇器 |
| 麻醉結束時間 | DateTime | | 時間選擇器 |
| 內容 | Textarea | ✓ | 詳細描述 |
| 不需用藥/停止用藥 | Checkbox | | 勾選表示無需繼續用藥 |
| 治療方式 | Repeater | | 可新增多筆：預計用藥、其他用藥名稱、預計劑量、預計最後用藥日期 |
| 備註 | Textarea | | 其他備註 |
| 相片 | File[] | | 多檔上傳 |
| 附件 | File[] | | 多檔上傳 |

**子表格 - 獸醫師建議：**
| 欄位 | 類型 |
|-----|------|
| 系統號 | Integer |
| 建立日期 | DateTime |
| 內容 | Text |

**子表格 - 照護觀察給藥紀錄（已停用，改用疼痛評估）：**
| 欄位 | 類型 |
|-----|------|
| 術後天數 | Integer |
| 時段 | String |
| 精神 | String |
| 食慾 | String |
| 活動力-站立 | String |
| 活動力-行走 | String |

**子表格 - 疼痛評估照護給藥紀錄：**
| 欄位 | 類型 |
|-----|------|
| 術後天數 | Integer |
| 時段 | String |
| 活動力-站立 | String |
| 態度/行為 | String |
| 食慾 | String |
| 獸醫師讀取 | Badge |

---

###### 2.10.2 手術紀錄

**列表欄位：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| （展開按鈕） | Button | 展開查看詳細 |
| 是否為第一次實驗 | Boolean | 是/否 |
| 手術日期 | Date | |
| 手術部位 | String | 如：雙眼眼底鏡觀察及ERG |
| 不需用藥/停止用藥 | Boolean | |
| 獸醫師建議 | Badge | |
| 記錄者 | String | |
| 獸醫師讀取 | Badge | 已讀/未讀 |
| 動作 | Buttons | 檢視、編輯、修訂版本、刪除 |

**新增/編輯表單欄位：**

*基本資訊：*
| 欄位 | 類型 | 必填 |
|-----|------|-----|
| 耳號 | Display | - |
| 是否為第一次實驗 | Radio | ✓ |
| 手術日期 | Date | ✓ |
| 手術部位 | Text | ✓ |

*誘導麻醉區塊：*
| 欄位 | 類型 | 說明 |
|-----|------|------|
| Atropine | Checkbox + Input | 勾選後輸入劑量 |
| Stroless | Checkbox + Input | 勾選後輸入劑量 |
| Zoletil-50 | Checkbox + Input | 勾選後輸入劑量 |
| 誘導麻醉-其他藥劑 | Repeater | 藥劑名稱、藥劑劑量 |

*術前給藥區塊：*
| 欄位 | 類型 |
|-----|------|
| 術前給藥 | Repeater | 藥品名稱、劑量 |
| 術前給藥-其他藥劑 | Repeater | 藥劑名稱、藥劑劑量 |

*固定姿勢區塊：*
| 欄位 | 類型 |
|-----|------|
| 固定姿勢 | Text |
| 其他固定姿勢 | Repeater | 名稱 |

*麻醉維持區塊：*
| 欄位 | 類型 | 說明 |
|-----|------|------|
| O2 | Checkbox + Input | 勾選後輸入劑量 |
| N2O | Checkbox + Input | 勾選後輸入劑量 |
| Isoflurane | Checkbox + Input | 勾選後輸入劑量 |
| 麻醉維持-其他藥劑 | Repeater | 藥劑名稱、藥劑劑量 |

*監測與恢復區塊：*
| 欄位 | 類型 |
|-----|------|
| 麻醉觀察過程 | Textarea |
| 生理數值 | Repeater | 呼吸方式、時間、心跳次數/分鐘、呼吸次數/分鐘、體溫(攝氏)、血氧SPO2 |
| 反射恢復觀察 | Textarea |
| 呼吸頻率觀察-自主呼吸：呼吸次數/分鐘 | Number |

*術後區塊：*
| 欄位 | 類型 |
|-----|------|
| 術後給藥-優點軟膏 | Checkbox |
| 術後給藥-其他 | Repeater | 名稱 |
| 備註 | Textarea |
| 不需用藥/停止用藥 | Checkbox |
| 相片 | File[] |
| 附件 | File[] |

*子表格（同觀察試驗紀錄）：*
- 獸醫師建議
- 照護觀察給藥紀錄
- 疼痛評估照護給藥紀錄
- 獸醫師讀取

---

###### 2.10.3 體重紀錄

**列表欄位：**
| 欄位 | 類型 | 可排序 |
|-----|------|--------|
| 系統號 | Integer | ✓ |
| 測量日期 | Date | ✓ |
| 體重 | Decimal | ✓ |
| 記錄者 | String | ✓ |
| 建立時間 | DateTime | ✓ |
| 動作 | Buttons | - |

**新增/編輯表單欄位：**
| 欄位 | 類型 | 必填 | 驗證 |
|-----|------|-----|------|
| 耳號 | Display | - | 唯讀 |
| 測量日期 | Date | ✓ | |
| 體重 | Decimal | ✓ | 大於0的正數，小數點後至多一位 |

---

###### 2.10.4 疫苗/驅蟲紀錄

**列表欄位：**
| 欄位 | 類型 | 可排序 |
|-----|------|--------|
| 施打日期 | Date | ✓ |
| 疫苗 | String | ✓ |
| 驅蟲劑量 | String | ✓ |
| 記錄者 | String | ✓ |
| 建立時間 | DateTime | ✓ |
| 動作 | Buttons | - |

**新增/編輯表單欄位：**
| 欄位 | 類型 | 必填 |
|-----|------|-----|
| 耳號 | Display | - |
| 施打日期 | Date | ✓ |
| 疫苗 | Text | | 如：SEP、IRON |
| 驅蟲劑量 | Text | | 如：Ivermectin 2mL |

---

###### 2.10.5 犧牲/採樣紀錄

**檢視模式（單筆資料）：**
| 欄位 | 內容 |
|-----|------|
| 犧牲日期 | Date |
| 犧牲方式：Zoletil-50(ml) | Text |
| 犧牲方式：200V電擊 | Boolean |
| 犧牲方式：放血 | Boolean |
| 犧牲方式：其他 | Text |
| 採樣 | Text |
| 採樣：其他說明 | Text |
| 採樣：血液(ml) | Decimal |
| 確定犧牲 | Boolean |
| 相片 | Image |
| 系統號 | Integer |
| 記錄者 | String |
| 建立時間 | DateTime |

**編輯表單欄位：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| 犧牲日期 | Date | |
| 犧牲方式：Zoletil-50(ml) | Text | 輸入劑量 |
| 犧牲方式：200V電擊 | Checkbox | |
| 犧牲方式：放血 | Checkbox | |
| 犧牲方式：其他 | Text | |
| 採樣 | Text | |
| 採樣：其他說明 | Text | |
| 採樣：血液(ml) | Decimal | |
| 確定犧牲 | Checkbox | 勾選後豬隻狀態變更為「實驗完畢」 |
| 相片 | File[] | |

---

###### 2.10.6 豬隻資料

**檢視模式欄位：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| 耳號 | String | |
| 豬隻狀態 | Enum | 未分配/已分配/實驗中/實驗完畢 |
| 進場日期 | Date | |
| 品種 | Enum | 迷你豬/白豬/其他 |
| 來源 | Dropdown | 選擇來源（由 pig_sources 表管理） |
| 進場體重 | Decimal | |
| 性別 | Enum | 公/母 |
| 出生日期 | Date | |
| 實驗前代號 | String | 如：PIG-110000 |
| 實驗代號 | String | 如：PIG-113002（IACUC NO.） |
| 實驗日期 | Date | |
| 欄位 | String | 如：A03 |
| 備註 | Text | |
| 獸醫師體重紀錄檢視時間 | DateTime | 系統自動記錄 |
| 獸醫師疫苗/驅蟲紀錄檢視時間 | DateTime | 系統自動記錄 |
| 獸醫師犧牲/採樣紀錄檢視時間 | DateTime | 系統自動記錄 |
| 獸醫師最近檢視資料時間 | DateTime | 系統自動記錄 |
| 記錄者 | String | |
| 系統號 | Integer | |
| 建立時間 | DateTime | |

**編輯表單欄位：**
| 欄位 | 類型 | 必填 |
|-----|------|-----|
| 耳號 | Text | ✓ |
| 豬隻狀態 | Dropdown | ✓ |
| 進場日期 | Date | ✓ |
| 品種 | Radio | ✓ |
| 來源 | Radio | ✓ |
| 進場體重 | Decimal | |
| 性別 | Radio | ✓ |
| 出生日期 | Date | ✓ |
| 實驗前代號 | Text | ✓ |
| 實驗代號 | Dropdown | | 選擇已核准的 IACUC 計劃 |
| 欄位 | Dropdown | ✓ | 選擇豬欄位置 |
| 備註 | Textarea | | |

---

###### 2.10.7 病理組織報告

**檢視模式：**
| 欄位 | 類型 |
|-----|------|
| 報告 | File Link | 可下載 |
| 系統號 | Integer |
| 記錄者 | String |
| 建立時間 | DateTime |

**編輯模式：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| 報告 | File Upload | +新增 檔案，支援多檔 |

---

###### 2.10.8 通用操作按鈕

每個紀錄列表的動作欄位包含：
| 按鈕 | 功能 | 權限 |
|-----|------|------|
| 檢視 | 唯讀查看詳細資料 | 所有人 |
| 編輯 | 修改紀錄內容 | 記錄者、管理員 |
| 修訂版本 | 查看歷史版本 | 所有人 |
| 刪除 | 刪除紀錄（需確認） | 記錄者、管理員 |

---

###### 2.11 資料模型

**pig_sources 表（豬隻來源）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| code | VARCHAR(20) | 來源代碼（唯一） |
| name | VARCHAR(100) | 來源名稱 |
| address | TEXT | 地址 |
| contact | VARCHAR(100) | 聯絡人 |
| phone | VARCHAR(20) | 聯絡電話 |
| is_active | BOOLEAN | 是否啟用 |
| sort_order | INTEGER | 排序 |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

**預設資料：**
| code | name |
|-----|------|
| TAITUNG | 台東種畜繁殖場 |
| QINGXIN | 青欣牧場 |
| PIGMODEL | 豬博士畜牧場 |
| PINGSHUN | 屏順牧場 |

---

**pigs 表（豬隻主檔）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵（系統號） |
| ear_tag | VARCHAR(10) | 耳號（唯一） |
| status | ENUM | 未分配/已分配/實驗中/實驗完畢 |
| breed | ENUM | 迷你豬/白豬/其他 |
| source_id | UUID | FK → pig_sources.id |
| gender | ENUM | 公/母 |
| birth_date | DATE | 出生日期 |
| entry_date | DATE | 進場日期 |
| entry_weight | DECIMAL(5,1) | 進場體重 |
| pen_location | VARCHAR(10) | 欄位編號 |
| pre_experiment_code | VARCHAR(20) | 實驗前代號 |
| iacuc_no | VARCHAR(20) | 實驗代號（IACUC NO.，FK） |
| experiment_date | DATE | 實驗開始日期 |
| remark | TEXT | 備註 |
| vet_weight_viewed_at | TIMESTAMP | 獸醫師體重紀錄檢視時間 |
| vet_vaccine_viewed_at | TIMESTAMP | 獸醫師疫苗紀錄檢視時間 |
| vet_sacrifice_viewed_at | TIMESTAMP | 獸醫師犧牲紀錄檢視時間 |
| vet_last_viewed_at | TIMESTAMP | 獸醫師最近檢視時間 |
| created_by | UUID | 記錄者 |
| created_at | TIMESTAMP | 建立時間 |
| updated_at | TIMESTAMP | 更新時間 |

---

**pig_observations 表（觀察試驗紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id |
| event_date | DATE | 事件發生日期 |
| record_type | ENUM | 異常紀錄/試驗紀錄/觀察紀錄 |
| equipment_used | JSONB | 使用儀器（多選） |
| anesthesia_start | TIMESTAMP | 麻醉開始時間 |
| anesthesia_end | TIMESTAMP | 麻醉結束時間 |
| content | TEXT | 內容 |
| no_medication_needed | BOOLEAN | 不需用藥/停止用藥 |
| treatments | JSONB | 治療方式（多筆） |
| remark | TEXT | 備註 |
| vet_read | BOOLEAN | 獸醫師是否已讀 |
| vet_read_at | TIMESTAMP | 獸醫師讀取時間 |
| created_by | UUID | 記錄者 |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

---

**pig_surgeries 表（手術紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id |
| is_first_experiment | BOOLEAN | 是否為第一次實驗 |
| surgery_date | DATE | 手術日期 |
| surgery_site | VARCHAR(200) | 手術部位 |
| induction_anesthesia | JSONB | 誘導麻醉藥物及劑量 |
| pre_surgery_medication | JSONB | 術前給藥 |
| positioning | VARCHAR(100) | 固定姿勢 |
| anesthesia_maintenance | JSONB | 麻醉維持藥物及劑量 |
| anesthesia_observation | TEXT | 麻醉觀察過程 |
| vital_signs | JSONB | 生理數值（多筆） |
| reflex_recovery | TEXT | 反射恢復觀察 |
| respiration_rate | INTEGER | 呼吸次數/分鐘 |
| post_surgery_medication | JSONB | 術後給藥 |
| remark | TEXT | 備註 |
| no_medication_needed | BOOLEAN | 不需用藥/停止用藥 |
| vet_read | BOOLEAN | |
| vet_read_at | TIMESTAMP | |
| created_by | UUID | |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

---

**pig_weights 表（體重紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id |
| measure_date | DATE | 測量日期 |
| weight | DECIMAL(5,1) | 體重（kg） |
| created_by | UUID | 記錄者 |
| created_at | TIMESTAMP | |

---

**pig_vaccinations 表（疫苗/驅蟲紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id |
| administered_date | DATE | 施打日期 |
| vaccine | VARCHAR(100) | 疫苗名稱 |
| deworming_dose | VARCHAR(100) | 驅蟲劑量 |
| created_by | UUID | 記錄者 |
| created_at | TIMESTAMP | |

---

**pig_sacrifices 表（犧牲/採樣紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id（唯一） |
| sacrifice_date | DATE | 犧牲日期 |
| zoletil_dose | VARCHAR(50) | Zoletil-50 劑量(ml) |
| method_electrocution | BOOLEAN | 200V電擊 |
| method_bloodletting | BOOLEAN | 放血 |
| method_other | TEXT | 其他方式 |
| sampling | TEXT | 採樣 |
| sampling_other | TEXT | 採樣其他說明 |
| blood_volume_ml | DECIMAL(6,1) | 採樣血液(ml) |
| confirmed_sacrifice | BOOLEAN | 確定犧牲 |
| created_by | UUID | |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

---

**pig_pathology_reports 表（病理組織報告）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| pig_id | INTEGER | FK → pigs.id（唯一） |
| created_by | UUID | |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

---

**pig_record_attachments 表（紀錄附件通用表）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| record_type | ENUM | observation/surgery/sacrifice/pathology |
| record_id | INTEGER | 對應紀錄 ID |
| file_type | ENUM | photo/attachment/report |
| file_name | VARCHAR(255) | 原始檔名 |
| file_path | VARCHAR(500) | 儲存路徑 |
| file_size | INTEGER | 檔案大小 |
| mime_type | VARCHAR(100) | MIME 類型 |
| created_at | TIMESTAMP | |

---

**vet_recommendations 表（獸醫師建議）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| record_type | ENUM | observation/surgery |
| record_id | INTEGER | 對應紀錄 ID |
| content | TEXT | 建議內容 |
| created_by | UUID | 獸醫師 ID |
| created_at | TIMESTAMP | |

---

**care_medication_records 表（照護給藥紀錄）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| record_type | ENUM | observation/surgery |
| record_id | INTEGER | 對應紀錄 ID |
| record_mode | ENUM | legacy（照護觀察）/ pain_assessment（疼痛評估） |
| post_op_days | INTEGER | 術後天數 |
| time_period | VARCHAR(20) | 時段 |
| spirit | VARCHAR(50) | 精神（legacy） |
| appetite | VARCHAR(50) | 食慾 |
| mobility_standing | VARCHAR(50) | 活動力-站立 |
| mobility_walking | VARCHAR(50) | 活動力-行走（legacy） |
| attitude_behavior | VARCHAR(50) | 態度/行為（pain_assessment） |
| vet_read | BOOLEAN | 獸醫師讀取 |
| created_at | TIMESTAMP | |

---

**record_versions 表（紀錄版本歷史）：**
| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | SERIAL | 主鍵 |
| record_type | ENUM | observation/surgery/weight/vaccination/sacrifice/pathology |
| record_id | INTEGER | 對應紀錄 ID |
| version_no | INTEGER | 版本號 |
| snapshot | JSONB | 當時的完整資料快照 |
| changed_by | UUID | 修改者 |
| changed_at | TIMESTAMP | |

---

###### 2.12 相關 API 端點

**豬隻來源管理（SYSTEM_ADMIN）：**
- GET `/pig-sources` - 取得來源清單
- POST `/pig-sources` - 新增來源
- PATCH `/pig-sources/{id}` - 更新來源
- DELETE `/pig-sources/{id}` - 停用來源（軟刪除）

**豬隻 CRUD：**
- GET `/pigs` - 取得豬隻清單（支援 ?status=&breed=&gender=&search= 篩選）
- GET `/pigs/by-pen` - 取得依欄位分組的豬隻清單
- GET `/pigs/{pig_id}` - 取得單一豬隻詳細資料
- POST `/pigs` - 新增豬隻
- PATCH `/pigs/{pig_id}` - 更新豬隻資料
- DELETE `/pigs/{pig_id}` - 刪除豬隻

**批次操作：**
- POST `/pigs/import/basic` - 批次匯入基本資料
- POST `/pigs/import/weight` - 批次匯入體重紀錄
- GET `/pigs/export/experiment-records` - 匯出試驗紀錄
- POST `/pigs/batch/start-experiment` - 批次變更為實驗中
- POST `/pigs/batch/assign` - 批次分配至計劃

**觀察試驗紀錄：**
- GET `/pigs/{pig_id}/observations` - 取得觀察試驗紀錄列表
- GET `/pigs/{pig_id}/observations/{id}` - 取得單筆紀錄
- POST `/pigs/{pig_id}/observations` - 新增紀錄
- PATCH `/pigs/{pig_id}/observations/{id}` - 更新紀錄
- DELETE `/pigs/{pig_id}/observations/{id}` - 刪除紀錄
- GET `/pigs/{pig_id}/observations/{id}/versions` - 取得版本歷史

**手術紀錄：**
- GET `/pigs/{pig_id}/surgeries` - 取得手術紀錄列表
- GET `/pigs/{pig_id}/surgeries/{id}` - 取得單筆紀錄
- POST `/pigs/{pig_id}/surgeries` - 新增紀錄
- PATCH `/pigs/{pig_id}/surgeries/{id}` - 更新紀錄
- DELETE `/pigs/{pig_id}/surgeries/{id}` - 刪除紀錄

**體重紀錄：**
- GET `/pigs/{pig_id}/weights` - 取得體重紀錄列表
- POST `/pigs/{pig_id}/weights` - 新增紀錄
- PATCH `/pigs/{pig_id}/weights/{id}` - 更新紀錄
- DELETE `/pigs/{pig_id}/weights/{id}` - 刪除紀錄

**疫苗/驅蟲紀錄：**
- GET `/pigs/{pig_id}/vaccinations` - 取得疫苗/驅蟲紀錄列表
- POST `/pigs/{pig_id}/vaccinations` - 新增紀錄
- PATCH `/pigs/{pig_id}/vaccinations/{id}` - 更新紀錄
- DELETE `/pigs/{pig_id}/vaccinations/{id}` - 刪除紀錄

**犧牲/採樣紀錄：**
- GET `/pigs/{pig_id}/sacrifice` - 取得犧牲/採樣紀錄
- POST `/pigs/{pig_id}/sacrifice` - 建立/更新紀錄
- PATCH `/pigs/{pig_id}/sacrifice` - 更新紀錄

**病理組織報告：**
- GET `/pigs/{pig_id}/pathology` - 取得病理報告
- POST `/pigs/{pig_id}/pathology` - 上傳報告檔案
- DELETE `/pigs/{pig_id}/pathology/files/{file_id}` - 刪除檔案

**獸醫師操作：**
- POST `/pigs/{pig_id}/vet-read` - 標記獸醫師已讀
- POST `/observations/{id}/recommendations` - 新增獸醫師建議
- POST `/surgeries/{id}/recommendations` - 新增獸醫師建議

**照護給藥紀錄：**
- POST `/observations/{id}/care-records` - 新增照護給藥紀錄
- POST `/surgeries/{id}/care-records` - 新增照護給藥紀錄

**附件管理：**
- POST `/records/{type}/{id}/attachments` - 上傳附件
- GET `/attachments/{id}/download` - 下載附件
- DELETE `/attachments/{id}` - 刪除附件

##### 3. 系統操作說明

下載本系統操作說明，內容如 `ipigmanager.md`

**使用者手冊章節：**
| 章節 | 內容 |
|-----|------|
| 登入系統 | 使用工作人員提供的 E-mail 及密碼登入 |
| 忘記密碼 | 點擊「忘記密碼」，透過信箱重設 |
| 變更密碼 | 初次登入後自行設定新密碼 |
| 我的計畫 | 檢視帳號下的計畫清單，進入計畫詳情 |
| 計畫內容-申請表 | 檢視申請表內容、下載相關文件 |
| 計畫內容-豬隻紀錄 | 檢視「實驗中」和「實驗完成」的豬隻清單 |
| 豬隻資料 | 豬隻基本資料 |
| 觀察試驗紀錄 | 該豬隻所有觀察試驗紀錄 |
| 手術紀錄 | 該豬隻所有手術紀錄 |
| 體重紀錄 | 該豬隻體重變化歷程 |
| 疫苗/驅蟲紀錄 | 該豬隻疫苗與驅蟲紀錄 |
| 犧牲/採樣紀錄 | 實驗完成後的犧牲與採樣紀錄 |
| 病理組織報告 | 下載病理組織報告檔案 |
| 下載豬隻病歷 | 下載病歷總表、觀察試驗紀錄、手術紀錄 |

---

### 4. 人員管理系統

#### 4.1 系統目的與範圍

本系統用於管理公司內部員工的請假、補休假及特休假之申請、審核、核銷等作業流程。

**適用範圍：**
- ✅ 全職員工
- ✅ 兼職員工（依比例計算）
- ✅ 約聘人員（依合約規定）
- ❌ 不包含審查人員、IACUC 主席等外部角色

**法規依據：**
- 勞動基準法
- 性別工作平等法
- 勞工請假規則
- 公司內部管理規章

詳細規格請參考 `humanResourceManager.md`

#### 4.2 核心功能

##### 4.2.1 請假流程
- 請假申請（含緊急請假、事後補請）
- 多層級審核（主管簽核 → 行政簽核 → 負責人核假）
- 請假狀態追蹤
- 請假記錄查詢與統計

##### 4.2.2 特休假管理
- 特休假額度自動計算（依年資）
- 特休假使用記錄
- 特休假遞延管理
- 未休完特休假折發工資計算

##### 4.2.3 補休假管理
- 補休假累計（依加班時數）
- 補休假使用記錄
- 補休假到期提醒
- 補休假到期轉換為加班費

##### 4.2.4 審核流程
- 一級審核：直屬主管（24小時內）
- 二級審核：部門主管（48小時內，如需要）
- 行政審核：人資部門（72小時內，特定假別）
- 負責人核假：總經理（5個工作天內，特定假別）

#### 4.3 資料庫設計

主要資料表：
- `leave_requests` - 請假申請表
- `annual_leave_balances` - 特休假額度表
- `compensatory_leave_balances` - 補休假額度表
- `leave_records` - 請假記錄表
- `leave_approvals` - 請假審核記錄表
- `google_calendar_tokens` - Google 行事曆授權記錄（選用）
- `leave_calendar_events` - 請假與行事曆事件對應表（選用）

詳細資料庫設計請參考 `humanResourceManager.md` 第 6.1 節。

#### 4.4 API 端點規格

##### 4.4.1 請假申請相關
```
POST   /api/hr/leaves                    # 建立請假申請
GET    /api/hr/leaves                    # 查詢請假申請列表（支援篩選）
GET    /api/hr/leaves/:id                # 查詢單一請假申請
PUT    /api/hr/leaves/:id                # 更新請假申請（僅草稿狀態）
DELETE /api/hr/leaves/:id                # 刪除請假申請（僅草稿狀態）
POST   /api/hr/leaves/:id/submit         # 送審請假申請
POST   /api/hr/leaves/:id/withdraw       # 撤回請假申請
POST   /api/hr/leaves/:id/cancel         # 取消請假申請
POST   /api/hr/leaves/:id/revoke         # 銷假
```

##### 4.4.2 請假審核相關
```
GET    /api/hr/leaves/pending            # 查詢待審核請假列表
POST   /api/hr/leaves/:id/approve        # 核准請假申請
POST   /api/hr/leaves/:id/reject         # 駁回請假申請
POST   /api/hr/leaves/:id/request-revision # 要求補件
```

##### 4.4.3 請假額度查詢
```
GET    /api/hr/leaves/balances           # 查詢個人請假額度
GET    /api/hr/leaves/balances/annual    # 查詢特休假額度
GET    /api/hr/leaves/balances/compensatory # 查詢補休假額度
```

##### 4.4.4 報表相關
```
GET    /api/hr/leaves/reports/summary    # 請假統計報表
GET    /api/hr/leaves/reports/calendar   # 請假行事曆
GET    /api/hr/leaves/reports/export     # 匯出請假記錄（CSV/Excel）
```

詳細 API 規格請參考 `humanResourceManager.md` 第 6.2 節。

#### 4.5 權限設定

| 功能 | 權限代碼 | 說明 |
|------|---------|------|
| 查看個人請假記錄 | `hr.leave.view.own` | 所有員工 |
| 申請請假 | `hr.leave.create` | 所有員工 |
| 編輯個人請假申請 | `hr.leave.edit.own` | 僅草稿狀態 |
| 刪除個人請假申請 | `hr.leave.delete.own` | 僅草稿狀態 |
| 查看部門請假記錄 | `hr.leave.view.department` | 部門主管 |
| 審核一級請假 | `hr.leave.approve.l1` | 直屬主管 |
| 審核二級請假 | `hr.leave.approve.l2` | 部門主管 |
| 審核人資請假 | `hr.leave.approve.hr` | 人資人員 |
| 審核總經理請假 | `hr.leave.approve.gm` | 總經理 |
| 查看所有請假記錄 | `hr.leave.view.all` | 人資、系統管理員 |
| 管理請假額度 | `hr.leave.balance.manage` | 人資、系統管理員 |
| 匯出請假報表 | `hr.leave.export` | 人資、系統管理員 |

#### 4.6 角色與子系統功能對應

| 角色 | 人員管理系統 |
|-----|------------|
| SYSTEM_ADMIN | ✓ 全部 |
| WAREHOUSE_MANAGER | ✓ 全部（內部員工） |
| PROGRAM_ADMIN | ✓ 全部（內部員工） |
| VET | ✓ 全部（內部員工） |
| EXPERIMENT_STAFF | ✓ 全部（內部員工） |
| IACUC_STAFF | ✓ 全部（內部員工） |
| CHAIR | ✗ 無權限（外部角色） |
| REVIEWER | ✗ 無權限（外部角色） |
| PI | ✗ 無權限（外部角色） |
| CLIENT | ✗ 無權限（外部角色） |

> ✓ 完整存取 ｜ ✗ 無權限

**重要說明：** 人員管理系統僅限**公司內部員工**（`is_internal = true`）使用，外部角色完全不可見此模組。

#### 4.7 前端路由規劃

| 路由 | 頁面說明 |
|-----|---------|
| `/hr/leaves` | 請假申請列表 |
| `/hr/leaves/new` | 新增請假申請 |
| `/hr/leaves/:id` | 請假申請詳情 |
| `/hr/leaves/pending` | 待審核請假列表 |
| `/hr/leaves/balances` | 請假額度查詢 |
| `/hr/leaves/reports` | 請假統計報表 |
| `/hr/leaves/calendar` | 請假行事曆 |

#### 4.8 系統整合

##### 4.8.1 與出勤系統整合
- 請假記錄自動同步至出勤系統
- 請假期間自動標記為「請假」，不計入缺勤

##### 4.8.2 與薪資系統整合
- 給薪假別自動計算薪資
- 未休完特休假自動計算折發工資
- 補休假到期轉換為加班費

##### 4.8.3 與通知系統整合
- 請假申請、審核、到期提醒等通知
- 支援 Email、系統內通知

##### 4.8.4 與 Google 行事曆整合（選用）
- 自動將已核准的請假記錄同步至員工的 Google 行事曆
- 提供個人/部門/公司層級的請假行事曆訂閱

詳細整合規格請參考 `humanResourceManager.md` 第 10 節。

---

## 附錄

### A. 版本紀錄

| 版本 | 日期 | 說明 |
|-----|------|------|
| 1.0 | 2026-01-07 | 初版，整合三大子系統規格 |
| 1.1 | 2026-01-XX | 新增人員管理系統規格 |

### B. 參考資料

- 豬博士 iPig 客戶使用說明書
- IACUC 動物試驗相關法規

### C. 名詞對照表

| 英文 | 中文 | 說明 |
|-----|------|------|
| AUP | Animal Use Protocol | 動物使用計畫書 |
| IACUC | Institutional Animal Care and Use Committee | 實驗動物照護及使用委員會 |
| PI | Principal Investigator | 計畫主持人 |
| JWT | JSON Web Token | 認證令牌 |
| SSO | Single Sign-On | 單一登入 |
