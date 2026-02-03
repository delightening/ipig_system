# iPig System 資料庫結構詳細註解

> **版本**: 2.0 | **更新日期**: 2026-01-19

---

## 1. 核心認證模組

### 1.1 users (使用者)

> **用途**: 儲存系統所有使用者的帳號資訊，包含內部員工與外部合作夥伴

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | 使用 UUID 確保分散式系統唯一性 |
| email | VARCHAR(255) | UK, NOT NULL | 電子郵件 | 作為登入帳號，全系統唯一 |
| password_hash | VARCHAR(255) | NOT NULL | 密碼雜湊 | 使用 bcrypt 加密儲存，不可逆 |
| display_name | VARCHAR(100) | NOT NULL | 顯示名稱 | 中文姓名或暱稱，用於介面顯示 |
| phone | VARCHAR(20) | | 電話 | 行動電話，用於緊急聯繫 |
| organization | VARCHAR(200) | | 組織 | 外部使用者所屬公司/機構 |
| is_internal | BOOLEAN | DEFAULT true | 是否內部人員 | true=公司員工, false=外部委託者 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 | false 時無法登入，用於停用帳號 |
| must_change_password | BOOLEAN | DEFAULT true | 需變更密碼 | 首次登入或重設密碼後須強制變更 |
| last_login_at | TIMESTAMPTZ | | 最後登入時間 | 用於安全稽核與閒置帳號偵測 |
| login_attempts | INTEGER | DEFAULT 0 | 登入嘗試次數 | 連續失敗超過 5 次將鎖定帳號 |
| locked_until | TIMESTAMPTZ | | 鎖定至 | 帳號鎖定解除時間，null 表示未鎖定 |
| theme_preference | VARCHAR(20) | DEFAULT 'light' | 主題偏好 | 可選: light, dark, system |
| language_preference | VARCHAR(10) | DEFAULT 'zh-TW' | 語言偏好 | 可選: zh-TW, en-US |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 建立時間 | 帳號建立時自動設定 |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | 更新時間 | 任何欄位變更時自動更新 |

---

### 1.2 roles (角色)

> **用途**: 定義系統角色，用於權限群組管理 (RBAC 模型)

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| code | VARCHAR(50) | UK, NOT NULL | 角色代碼 | 程式內部使用，如 ADMIN, VET, RESEARCHER |
| name | VARCHAR(100) | NOT NULL | 角色名稱 | 前端顯示名稱，如「系統管理員」 |
| description | TEXT | | 說明 | 角色職責描述 |
| is_internal | BOOLEAN | DEFAULT true | 內部角色 | true=僅內部員工可持有 |
| is_system | BOOLEAN | DEFAULT false | 系統角色 | true=系統預設角色，不可刪除 |
| is_deleted | BOOLEAN | DEFAULT false | 已刪除 | 軟刪除標記 |
| is_active | BOOLEAN | DEFAULT true | 是否啟用 | false 時該角色權限暫停生效 |

---

### 1.3 permissions (權限)

> **用途**: 定義細粒度權限項目，如「查看豬隻」、「編輯實驗計畫」

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| code | VARCHAR(100) | UK, NOT NULL | 權限代碼 | 格式: MODULE.ACTION，如 pig.view, protocol.edit |
| name | VARCHAR(200) | NOT NULL | 權限名稱 | 前端顯示名稱 |
| module | VARCHAR(50) | | 模組 | 所屬功能模組，用於分組顯示 |
| description | TEXT | | 說明 | 權限詳細說明 |

---

### 1.4 user_roles (使用者角色關聯)

> **用途**: 多對多關聯表，一個使用者可擁有多個角色

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| user_id | UUID | PK, FK → users | 使用者 ID | |
| role_id | UUID | PK, FK → roles | 角色 ID | |
| assigned_at | TIMESTAMPTZ | | 指派時間 | 用於稽核追蹤 |
| assigned_by | UUID | FK → users | 指派者 | 記錄哪位管理員指派此角色 |

---

### 1.5 role_permissions (角色權限關聯)

> **用途**: 多對多關聯表，定義每個角色擁有哪些權限

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| role_id | UUID | PK, FK → roles | 角色 ID | |
| permission_id | UUID | PK, FK → permissions | 權限 ID | |

---

### 1.6 refresh_tokens (JWT 更新令牌)

> **用途**: 儲存 JWT refresh token，用於延長登入狀態

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK → users | 使用者 ID | |
| token_hash | VARCHAR | | 令牌雜湊 | 僅儲存雜湊值，原始 token 由客戶端保管 |
| expires_at | TIMESTAMPTZ | | 過期時間 | 通常設定為 7 天 |
| revoked_at | TIMESTAMPTZ | | 撤銷時間 | 非 null 表示已被強制登出 |

---

### 1.7 password_reset_tokens (密碼重設令牌)

> **用途**: 「忘記密碼」功能使用，一次性令牌

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK → users | 使用者 ID | |
| token_hash | VARCHAR | | 令牌雜湊 | 透過 email 發送給使用者 |
| expires_at | TIMESTAMPTZ | | 過期時間 | 通常 1 小時內有效 |
| used_at | TIMESTAMPTZ | | 使用時間 | 非 null 表示已使用，不可重複使用 |

---

## 2. ERP 庫存管理模組

### 2.1 warehouses (倉庫)

> **用途**: 定義實體倉儲位置，支援多倉庫管理

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| code | VARCHAR(50) | UK | 倉庫代碼 | 簡短編號，如 WH01, WH02 |
| name | VARCHAR(200) | | 倉庫名稱 | 如「主倉庫」、「冷藏室」 |
| address | TEXT | | 地址 | 實體位置 |
| is_active | BOOLEAN | | 是否啟用 | false 時不可新增庫存異動 |

---

### 2.2 sku_categories (SKU 分類)

> **用途**: 產品一級分類，用於 SKU 編號產生規則

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| code | CHAR(3) | PK | 分類代碼 | 如 DRG=藥品, CON=耗材, FED=飼料 |
| name | VARCHAR(50) | | 分類名稱 | 前端顯示名稱 |
| sort_order | INTEGER | | 排序順序 | 下拉選單排序依據 |
| is_active | BOOLEAN | | 是否啟用 | |

---

### 2.3 products (產品)

> **用途**: 產品主檔，可追蹤批號與效期

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| sku | VARCHAR(50) | UK, NOT NULL | SKU 編號 | 格式: 分類碼-子分類碼-流水號 |
| name | VARCHAR(200) | NOT NULL | 產品名稱 | |
| spec | TEXT | | 規格 | 如「500mg x 100錠」 |
| category_code | CHAR(3) | FK | 分類代碼 | 關聯 sku_categories |
| base_uom | VARCHAR(20) | DEFAULT 'pcs' | 基本單位 | 如 pcs, box, ml, kg |
| track_batch | BOOLEAN | DEFAULT false | 追蹤批號 | 藥品類通常需要 |
| track_expiry | BOOLEAN | DEFAULT false | 追蹤效期 | 有效期限管理 |
| safety_stock | NUMERIC(18,4) | | 安全庫存 | 低於此數量觸發警告 |

---

### 2.4 partners (夥伴)

> **用途**: 供應商與客戶主檔

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| partner_type | partner_type | NOT NULL | 夥伴類型 | supplier=供應商, customer=客戶 |
| code | VARCHAR(50) | UK, NOT NULL | 夥伴代碼 | 簡短編號 |
| name | VARCHAR(200) | NOT NULL | 夥伴名稱 | 公司全名 |
| supplier_category | supplier_category | | 供應商分類 | 僅供應商需填，如 drug, feed |

---

### 2.5 documents (單據表頭)

> **用途**: 所有庫存單據的表頭資訊 (採購單、進貨單、調撥單等)

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| doc_type | doc_type | NOT NULL | 單據類型 | PO=採購單, GRN=進貨單, TR=調撥單 |
| doc_no | VARCHAR(50) | UK, NOT NULL | 單據編號 | 自動產生，格式: 類型-YYYYMMDD-流水號 |
| status | doc_status | DEFAULT 'draft' | 狀態 | draft→submitted→approved/cancelled |
| warehouse_id | UUID | FK | 倉庫 ID | 入/出庫倉庫 |
| partner_id | UUID | FK | 夥伴 ID | 供應商或客戶 |
| doc_date | DATE | NOT NULL | 單據日期 | 實際交易日期 |
| created_by | UUID | FK, NOT NULL | 建立者 | 開單人員 |
| approved_by | UUID | FK | 核准者 | 核准後才能影響庫存 |

---

### 2.6 document_lines (單據明細)

> **用途**: 單據的產品明細行

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| document_id | UUID | FK, NOT NULL | 單據 ID | 關聯表頭 |
| line_no | INTEGER | NOT NULL | 行號 | 顯示順序 |
| product_id | UUID | FK, NOT NULL | 產品 ID | |
| qty | NUMERIC(18,4) | NOT NULL | 數量 | 支援小數，如 2.5kg |
| uom | VARCHAR(20) | NOT NULL | 單位 | 可與 base_uom 不同 |
| unit_price | NUMERIC(18,4) | | 單價 | 採購/銷售單價 |

---

### 2.7 stock_ledger (庫存分類帳)

> **用途**: 記錄每筆庫存異動，可追溯任何時點的庫存數量

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| warehouse_id | UUID | FK, NOT NULL | 倉庫 ID | |
| product_id | UUID | FK, NOT NULL | 產品 ID | |
| trx_date | TIMESTAMPTZ | NOT NULL | 交易日期 | 精確到秒 |
| doc_type | doc_type | NOT NULL | 單據類型 | 來源單據類型 |
| doc_id | UUID | NOT NULL | 單據 ID | 來源單據 |
| direction | stock_direction | NOT NULL | 異動方向 | in=入庫, out=出庫, adjust_in=調增 |
| qty_base | NUMERIC(18,4) | NOT NULL | 基本單位數量 | 統一換算為基本單位 |

---

### 2.8 inventory_snapshots (即時庫存)

> **用途**: 維護目前在手庫存，避免每次重新計算

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| warehouse_id | UUID | PK, FK | 倉庫 ID | 複合主鍵 |
| product_id | UUID | PK, FK | 產品 ID | 複合主鍵 |
| on_hand_qty_base | NUMERIC(18,4) | DEFAULT 0 | 在手數量 | 由 trigger 自動維護 |

---

## 3. 實驗計畫模組

### 3.1 protocols (實驗計畫)

> **用途**: 儲存 IACUC 動物實驗計畫，需通過審查流程

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| protocol_no | VARCHAR(50) | UK, NOT NULL | 計畫編號 | 內部編號 |
| iacuc_no | VARCHAR(50) | UK | IACUC 編號 | 審查通過後由委員會核發 |
| title | VARCHAR(500) | NOT NULL | 計畫標題 | 實驗計畫名稱 |
| status | protocol_status | DEFAULT 'DRAFT' | 狀態 | 審查流程狀態 |
| pi_user_id | UUID | FK, NOT NULL | 主持人 ID | 計畫主持人 (PI) |
| working_content | JSONB | | 工作內容 | 計畫書內容，JSON 格式儲存 |
| start_date | DATE | | 開始日期 | 計畫核准後的執行期間 |
| end_date | DATE | | 結束日期 | |

---

### 3.2 user_protocols (使用者計畫關聯)

> **用途**: 定義使用者在計畫中的角色 (PI/共同主持人/委託者)

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| user_id | UUID | PK, FK | 使用者 ID | |
| protocol_id | UUID | PK, FK | 計畫 ID | |
| role_in_protocol | protocol_role | NOT NULL | 計畫角色 | PI=主持人, CLIENT=委託者, CO_EDITOR=共編 |

---

## 4. 動物管理模組

### 4.1 pigs (豬隻)

> **用途**: 豬隻基本資料，每隻豬有唯一耳標

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | SERIAL | PK | 主鍵 | 自動遞增 |
| ear_tag | VARCHAR(10) | NOT NULL | 耳標 | 豬隻唯一識別碼，如 M001, F023 |
| status | pig_status | DEFAULT 'unassigned' | 狀態 | 生命週期狀態 |
| breed | pig_breed | NOT NULL | 品種 | miniature=迷你豬, white=白豬, LYD |
| source_id | UUID | FK | 來源 ID | 豬隻供應商/來源農場 |
| gender | pig_gender | NOT NULL | 性別 | male, female |
| birth_date | DATE | | 出生日期 | |
| entry_date | DATE | NOT NULL | 入場日期 | 進入設施的日期 |
| entry_weight | NUMERIC(5,1) | | 入場體重 | 單位: kg |
| pen_location | VARCHAR(10) | | 欄位位置 | 目前所在欄舍，如 A-01, B-03 |
| iacuc_no | VARCHAR(20) | | IACUC 編號 | 分配到的實驗計畫 |
| is_deleted | BOOLEAN | DEFAULT false | 已刪除 | 軟刪除標記 |

---

### 4.2 pig_observations (豬隻觀察紀錄)

> **用途**: 每日健康觀察、異常紀錄、用藥處置

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | SERIAL | PK | 主鍵 | |
| pig_id | INTEGER | FK, NOT NULL | 豬隻 ID | |
| event_date | DATE | NOT NULL | 事件日期 | 觀察日期 |
| record_type | record_type | NOT NULL | 紀錄類型 | abnormal=異常, observation=一般觀察 |
| content | TEXT | NOT NULL | 內容 | 觀察內容文字 |
| no_medication_needed | BOOLEAN | DEFAULT false | 無需用藥 | 勾選表示無需治療 |
| stop_medication | BOOLEAN | DEFAULT false | 停止用藥 | 停止先前的用藥 |
| treatments | JSONB | | 治療處置 | 用藥資訊 JSON |
| vet_read | BOOLEAN | DEFAULT false | 獸醫已讀 | 獸醫確認批次閱讀 |

---

### 4.3 pig_surgeries (豬隻手術紀錄)

> **用途**: 實驗手術記錄，包含麻醉與生命徵象

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | SERIAL | PK | 主鍵 | |
| pig_id | INTEGER | FK, NOT NULL | 豬隻 ID | |
| is_first_experiment | BOOLEAN | DEFAULT true | 首次實驗 | 是否為此豬首次手術 |
| surgery_date | DATE | NOT NULL | 手術日期 | |
| surgery_site | VARCHAR(200) | NOT NULL | 手術部位 | 如「左頸動脈」 |
| induction_anesthesia | JSONB | | 誘導麻醉 | 麻醉藥物與劑量 |
| anesthesia_maintenance | JSONB | | 麻醉維持 | 維持麻醉資訊 |
| vital_signs | JSONB | | 生命徵象 | 心率、血壓等監測數據 |
| vet_read | BOOLEAN | DEFAULT false | 獸醫已讀 | |

---

## 5. 人資管理模組

### 5.1 attendance_records (出勤紀錄)

> **用途**: 員工每日出勤打卡記錄

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK, NOT NULL | 使用者 ID | |
| work_date | DATE | NOT NULL | 工作日期 | (user_id, work_date) 唯一約束 |
| clock_in_time | TIMESTAMPTZ | | 上班打卡 | |
| clock_out_time | TIMESTAMPTZ | | 下班打卡 | |
| regular_hours | NUMERIC(5,2) | DEFAULT 0 | 正常工時 | 自動計算 |
| overtime_hours | NUMERIC(5,2) | DEFAULT 0 | 加班工時 | 超過 8 小時部分 |
| status | VARCHAR(20) | DEFAULT 'normal' | 狀態 | normal, late, early_leave, absent |

---

### 5.2 overtime_records (加班紀錄)

> **用途**: 加班申請與補休時數計算

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK, NOT NULL | 使用者 ID | |
| overtime_date | DATE | NOT NULL | 加班日期 | |
| start_time | TIMESTAMPTZ | NOT NULL | 開始時間 | |
| end_time | TIMESTAMPTZ | NOT NULL | 結束時間 | |
| hours | NUMERIC(5,2) | NOT NULL | 時數 | 實際加班時數 |
| overtime_type | VARCHAR(20) | NOT NULL | 類型 | weekday=平日, weekend=假日, holiday=國定 |
| multiplier | NUMERIC(3,2) | DEFAULT 1.0 | 倍率 | 平日1.0, 假日1.34, 國定2.0 |
| comp_time_hours | NUMERIC(5,2) | NOT NULL | 補休時數 | = hours × multiplier |
| comp_time_expires_at | DATE | NOT NULL | 補休到期日 | 加班日起 6 個月內需使用 |
| status | VARCHAR(20) | DEFAULT 'draft' | 狀態 | 需經簽核才能生效 |

---

### 5.3 leave_requests (請假單)

> **用途**: 員工請假申請，需經主管簽核

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK, NOT NULL | 申請人 ID | |
| proxy_user_id | UUID | FK | 代理人 ID | 請假期間的職務代理人 |
| leave_type | leave_type | NOT NULL | 假別 | ANNUAL=特休, SICK=病假, COMPENSATORY=補休 |
| start_date | DATE | NOT NULL | 開始日期 | |
| end_date | DATE | NOT NULL | 結束日期 | |
| total_days | NUMERIC(5,2) | NOT NULL | 總天數 | 支援半天假 (0.5) |
| reason | TEXT | | 原因 | 請假事由 |
| status | leave_status | DEFAULT 'DRAFT' | 狀態 | 簽核流程狀態 |
| current_approver_id | UUID | FK | 目前簽核人 | 等待此人審核 |

---

## 6. 稽核與安全模組

### 6.1 user_activity_logs (使用者活動日誌)

> **用途**: GLP 合規稽核，記錄所有資料異動

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| actor_user_id | UUID | FK | 操作者 ID | 執行操作的使用者 |
| event_category | VARCHAR(50) | NOT NULL | 事件分類 | AUTH, DATA, CONFIG 等 |
| event_type | VARCHAR(100) | NOT NULL | 事件類型 | 如 pig.create, protocol.approve |
| entity_type | VARCHAR(50) | | 實體類型 | 被操作的資料表名稱 |
| entity_id | UUID | | 實體 ID | 被操作的記錄 ID |
| before_data | JSONB | | 變更前資料 | UPDATE/DELETE 時記錄原值 |
| after_data | JSONB | | 變更後資料 | INSERT/UPDATE 時記錄新值 |
| changed_fields | TEXT[] | | 變更欄位 | 列出哪些欄位被修改 |
| ip_address | INET | | IP 位址 | 操作者 IP |
| partition_date | DATE | NOT NULL | 分區日期 | 按季度分區，加速查詢 |

> ⚠️ **注意**: 此表按季度分區，每季自動建立新分區

---

### 6.2 login_events (登入事件)

> **用途**: 記錄登入登出，偵測異常登入行為

| 欄位名稱 | 資料類型 | 約束 | 說明 | 業務邏輯註解 |
|----------|----------|------|------|--------------|
| id | UUID | PK | 主鍵 | |
| user_id | UUID | FK | 使用者 ID | 登入成功時有值 |
| email | VARCHAR(255) | NOT NULL | 電子郵件 | 嘗試登入的帳號 |
| event_type | VARCHAR(20) | NOT NULL | 事件類型 | login_success, login_failure, logout |
| ip_address | INET | | IP 位址 | |
| is_unusual_time | BOOLEAN | DEFAULT false | 異常時間 | 非正常上班時間登入 |
| is_unusual_location | BOOLEAN | DEFAULT false | 異常位置 | 非常用 IP 區段 |
| is_new_device | BOOLEAN | DEFAULT false | 新裝置 | 首次使用的裝置 |
| failure_reason | VARCHAR(100) | | 失敗原因 | 如 invalid_password, account_locked |

---

## 7. 列舉類型說明

### 7.1 單據類型 (doc_type)

| 代碼 | 中文 | 說明 |
|------|------|------|
| PO | 採購單 | Purchase Order - 向供應商下訂 |
| GRN | 進貨單 | Goods Received Note - 確認收貨入庫 |
| PR | 採購申請 | Purchase Requisition - 內部申請採購 |
| SO | 銷貨單 | Sales Order - 客戶訂單 |
| DO | 出貨單 | Delivery Order - 出庫交貨 |
| SR | 銷貨退回 | Sales Return - 客戶退貨 |
| TR | 調撥單 | Transfer - 倉庫間調撥 |
| STK | 盤點單 | Stock Take - 實物盤點 |
| ADJ | 調整單 | Adjustment - 帳面調整 |
| RTN | 退貨單 | Return - 向供應商退貨 |

### 7.2 請假類型 (leave_type)

| 代碼 | 中文 | 說明 |
|------|------|------|
| ANNUAL | 特休 | 年度特別休假 |
| PERSONAL | 事假 | 私人事務請假 |
| SICK | 病假 | 因病無法上班 |
| COMPENSATORY | 補休 | 使用加班換得的補休時數 |
| MARRIAGE | 婚假 | 結婚假 |
| BEREAVEMENT | 喪假 | 親屬過世 |
| MATERNITY | 產假 | 生產假 (女性) |
| PATERNITY | 陪產假 | 配偶生產 (男性) |
| MENSTRUAL | 生理假 | 女性生理期 |
| OFFICIAL | 公假 | 因公外出 |
| UNPAID | 無薪假 | 留職停薪 |

### 7.3 豬隻狀態 (pig_status)

| 代碼 | 中文 | 說明 |
|------|------|------|
| unassigned | 未指派 | 尚未分配到實驗計畫 |
| assigned | 已指派 | 已分配實驗計畫，等待實驗 |
| in_experiment | 實驗中 | 正在進行實驗 |
| completed | 完成 | 實驗結束 |
| transferred | 轉出 | 轉移至其他單位 |
| deceased | 死亡 | 已死亡/犧牲 |

---

## 約束說明對照表

| 縮寫 | 英文 | 中文 | 說明 |
|------|------|------|------|
| PK | Primary Key | 主鍵 | 唯一識別記錄 |
| FK | Foreign Key | 外鍵 | 關聯其他表 |
| UK | Unique | 唯一約束 | 不可重複 |
| NOT NULL | Not Null | 必填 | 不可為空 |
| DEFAULT | Default | 預設值 | 未填時自動設定 |
