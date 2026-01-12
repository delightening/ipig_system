# iPig ERP (進銷存管理系統) Spec（Rust + PostgreSQL + 前端 UI）

版本：v0.1  
目標：建立一套可支援「採購、入庫、銷售、出庫、盤點、調撥、報表、權限」的 iPig ERP 進銷存系統  
技術：後端 Rust（Axum）+ PostgreSQL；前端 Web UI（React + Vite + Tailwind）  

---

## 1. 專案定位與範圍

### 1.1 使用情境
- 中小型企業或部門使用
- 多倉庫、多品項、多單位換算、批號或效期（可選）
- 需要基本的審核流程與報表匯出

### 1.2 系統目標
- 交易可追溯：每一筆庫存增減都能追溯到單據與明細
- 庫存即時：以「庫存流水」為主（Stock Ledger），避免直接改庫存造成不一致
- 權限清楚：管理層可控「功能、資料範圍、核准權限」
- 可擴充：後續可加條碼、API 對接、會計科目、成本法等

### 1.3 不在 v0.1 的範圍（先排除）
- 完整會計總帳、應收應付帳款（可留接口）
- 複雜製造（BOM/MRP/工單）

---

## 2. 角色與權限（RBAC）

### 2.1 內建角色（可由管理員自訂）
- 系統管理員 Admin：全權、角色權限管理、系統設定
- 倉管 Warehouse：入庫/出庫/盤點/調撥、可檢視庫存
- 採購 Purchasing：供應商、採購單、採購入庫、退貨
- 業務 Sales：客戶、銷售單、銷售出庫、退貨
- 財務 Finance（選用）：成本報表、匯出、核准
- 主管 Approver：可核准/駁回單據、查看報表

### 2.2 權限模型
- 功能權限：例如 `po.read`, `po.create`, `po.approve`, `stock.adjust`
- 資料範圍：倉庫範圍、部門範圍（例如只能看某倉）
- 單據流程權限：送審、核准、反核（可選）

---

## 3. 核心名詞與資料流

### 3.1 單據類型
- 採購單 PO（Purchase Order）
- 採購入庫 GRN（Goods Receipt Note）
- 採購退貨 PR（Purchase Return）
- 銷售單 SO（Sales Order）
- 銷售出庫 DO（Delivery Order）
- 銷售退貨 SR（Sales Return）
- 調撥單 TR（Transfer）
- 盤點單 STK（Stocktake）
- 調整單 ADJ（Stock Adjustment）

### 3.2 庫存計算方式（建議）
- 以「庫存流水 Stock Ledger」為唯一真相  
- 每張會影響庫存的單據核准後產生 ledger 記錄（in/out/transfer/adjust）
- 庫存結存可由 ledger 聚合得出（也可做快取表提升效能）

### 3.3 成本法（v0.1）
- 先支援「移動平均（Moving Average）」或「加權平均」二選一  
- FIFO/LIFO 留到 v0.2

---

## 4. 功能需求（FR）

### 4.1 基礎資料（Master Data）
1. 產品（SKU）
   - SKU 編碼、名稱、規格、類別、單位、條碼（可選）
   - 是否啟用批號/效期
   - 安全庫存、建議補貨點
2. 倉庫
   - 倉庫清單、地址、是否啟用庫位（可選）
3. 供應商 / 客戶
   - 基本資料、聯絡資訊、統編、付款條件（可選）
4. 單位與換算（可選）
   - 例如 箱 = 12 入、包 = 10 入

### 4.2 採購流程
- 建立採購單（草稿）
- 送審 -> 核准 -> 可產生採購入庫
- 採購入庫（可部分入庫）
- 採購退貨（從入庫或庫存退回）
- 報表：採購明細、供應商排行、未交貨明細

### 4.3 銷售流程
- 建立銷售單（草稿）
- 送審 -> 核准 -> 可產生銷售出庫
- 銷售出庫（可部分出庫）
- 銷售退貨（入庫回倉）
- 報表：銷售明細、客戶排行、未出貨明細

### 4.4 倉儲作業
- 庫存查詢（依倉庫、品項、批號、效期）
- 調撥（倉 A -> 倉 B）
- 盤點
  - 盤點單建立 -> 匯入盤點數量 -> 計算差異 -> 核准後產生調整 ledger
- 調整
  - 正調整/負調整（需權限）
- 低庫存提醒（依安全庫存/補貨點）

### 4.5 報表與匯出
- 庫存現況表（即時）
- 庫存流水帳（按日期、單據、品項）
- 進貨/出貨明細（可篩選）
- 成本報表（移動平均成本、期末存貨）
- 匯出：CSV / Excel（v0.1 可先 CSV，Excel v0.2）

### 4.6 稽核與追蹤
- 單據狀態：Draft -> Submitted -> Approved -> Posted（或 Approved 即 Posted）
- 稽核軌跡：誰在何時做了什麼（create/update/approve/cancel）
- 反核（可選，v0.1 可先提供作廢與沖銷單）

---

## 5. 非功能需求（NFR）

- 安全性：JWT、Refresh Token、密碼 Argon2
- 權限：後端強制檢查（不可只靠前端）
- 一致性：交易寫入必須在 DB transaction 內完成（單據核准 + ledger）
- 效能：庫存查詢需支援 50k+ ledger 仍可用（必要索引、可加物化視圖）
- 可觀測性：結構化 log、request id、慢查詢紀錄
- 部署：Docker Compose（API + DB + Web）

---

## 6. 系統架構

### 6.1 後端（Rust）
- Web Framework：Axum
- DB：SQLx（PostgreSQL）
- Migration：sqlx migrate 或 refinery
- Auth：jsonwebtoken + argon2 + refresh token table
- Validation：validator
- API 文件：OpenAPI（utoipa / aide 擇一）
- 佇列（可選）：後續通知可用 Redis/Queue（v0.2）

### 6.2 前端（Web UI）
- React + Vite
- Tailwind CSS
- State：TanStack Query + Zustand（或 Redux Toolkit）
- Form：React Hook Form + Zod
- Table：TanStack Table
- Route：React Router
- UI Component：shadcn/ui（建議）

---

## 7. 資料庫設計（PostgreSQL）

> 重點：主檔（master）+ 單據頭/身（header/lines）+ 庫存流水（stock_ledger）

### 7.1 基礎表

#### users

> ⚠️ **注意**：users 表採用統一資料模型，完整欄位定義請參考 `role.md` 第 5.1 節。
> 
> 主要欄位包含：
> - is_internal（區分內/外部人員）
> - must_change_password（首次登入強制變更）

基本欄位：
- id (uuid, pk)
- email (unique)
- password_hash
- display_name
- is_internal (bool) ← 參考 role.md
- is_active
- must_change_password (bool) ← 參考 role.md
- created_at, updated_at

#### roles
- id (uuid, pk)
- code (unique) e.g. admin, warehouse
- name

#### permissions
- id (uuid, pk)
- code (unique) e.g. po.create, stock.view
- name

#### role_permissions
- role_id, permission_id (pk)

#### user_roles
- user_id, role_id (pk)

#### warehouses
- id (uuid, pk)
- code (unique)
- name
- address
- is_active

#### products
- id (uuid, pk)
- sku (unique)
- name
- spec (text)
- category_id (uuid, nullable)
- base_uom (text) e.g. pcs
- track_batch (bool)
- track_expiry (bool)
- safety_stock (numeric)
- reorder_point (numeric)
- is_active
- created_at, updated_at

#### product_uom_conversions（可選）
- id (uuid, pk)
- product_id
- uom
- factor_to_base (numeric) 例如 1 箱 = 12 pcs -> factor_to_base=12

#### partners（供應商/客戶共用）
- id (uuid, pk)
- type (enum: supplier, customer)
- code (unique)
- name
- tax_id
- phone, email, address
- is_active

---

### 7.2 單據共用結構

#### documents（通用單據頭，或分表也可）
- id (uuid, pk)
- doc_type (enum: PO, GRN, PR, SO, DO, SR, TR, STK, ADJ)
- doc_no (unique, 可含前綴與年月)
- status (enum: draft, submitted, approved, cancelled)
- warehouse_id (nullable，視單別)
- warehouse_from_id (nullable，調撥用)
- warehouse_to_id (nullable，調撥用)
- partner_id (nullable)
- doc_date (date)
- remark (text)
- created_by, approved_by (uuid)
- created_at, updated_at, approved_at

#### document_lines
- id (uuid, pk)
- document_id
- line_no (int)
- product_id
- qty (numeric)
- uom (text)
- unit_price (numeric, nullable)
- batch_no (text, nullable)
- expiry_date (date, nullable)
- remark (text)

索引建議：
- documents(doc_type, doc_no)
- documents(status, doc_date)
- document_lines(document_id)
- document_lines(product_id)

---

### 7.3 庫存流水與成本

#### stock_ledger
- id (uuid, pk)
- warehouse_id
- product_id
- trx_date (timestamp)
- doc_type
- doc_id
- doc_no
- line_id (nullable)
- direction (enum: in, out, transfer_in, transfer_out, adjust_in, adjust_out)
- qty_base (numeric) 以 base_uom 計
- unit_cost (numeric, nullable) v0.1 可由平均成本產生
- batch_no (nullable)
- expiry_date (nullable)
- created_at

索引建議：
- stock_ledger(warehouse_id, product_id, trx_date)
- stock_ledger(doc_id)
- stock_ledger(product_id, trx_date)

#### inventory_snapshot（可選快取）
- warehouse_id
- product_id
- on_hand_qty_base
- avg_cost
- updated_at
主鍵：(warehouse_id, product_id)

> v0.1 可先不做 snapshot，庫存查詢用聚合視圖；若效能不夠再加

---

### 7.4 稽核

#### audit_logs
- id (uuid, pk)
- actor_user_id
- action (text) e.g. DOC_APPROVE
- entity_type (text) e.g. document
- entity_id (uuid)
- before (jsonb, nullable)
- after (jsonb, nullable)
- created_at

---

## 8. API 規格（REST）

### 8.1 認證
- POST `/auth/login`
- POST `/auth/refresh`
- POST `/auth/logout`
- GET `/me`

> **注意**：本系統採私域註冊，無公開註冊 API。帳號由 SYSTEM_ADMIN 建立，詳見 `role.md`。

### 8.2 主檔
- CRUD `/warehouses`
- CRUD `/products`
- CRUD `/partners`
- CRUD `/roles` `/permissions` `/user-roles`

### 8.3 單據（通用）
- POST `/documents`（建立草稿）
- GET `/documents?doc_type=&status=&date_from=&date_to=&keyword=`
- GET `/documents/{id}`
- PUT `/documents/{id}`（限 draft）
- POST `/documents/{id}/submit`
- POST `/documents/{id}/approve`（核准後寫入 stock_ledger）
- POST `/documents/{id}/cancel`

### 8.4 庫存
- GET `/inventory/on-hand?warehouse_id=&product_id=&keyword=&batch_no=`
- GET `/inventory/ledger?warehouse_id=&product_id=&date_from=&date_to=`

### 8.5 報表
- GET `/reports/stock-on-hand`
- GET `/reports/stock-ledger`
- GET `/reports/purchase-lines`
- GET `/reports/sales-lines`
- GET `/reports/cost-summary`

---

## 9. 前端 UI 規格

### 9.1 版面與導覽
- Layout：左側側欄 + 頂部操作列（使用者/登出/語言）
- Sidebar：
  - Dashboard
  - 基礎資料（產品、倉庫、供應商/客戶）
  - 採購（採購單、採購入庫、採購退貨）
  - 銷售（銷售單、銷售出庫、銷售退貨）
  - 倉儲（庫存查詢、調撥、盤點、調整）
  - 報表（庫存、進貨、出貨、成本）
  - 系統管理（使用者、角色權限、設定）

### 9.2 重要頁面

#### Dashboard
- 今日入庫/出庫筆數
- 低庫存列表
- 近 7 天出入庫趨勢（v0.1 可先表格，圖表 v0.2）

#### 產品列表/編輯
- 表格：SKU、名稱、單位、庫存（可選顯示）、安全庫存、狀態
- 編輯：批號/效期開關、換算單位（可選）

#### 單據列表（通用）
- 篩選：單別、狀態、日期、關鍵字
- 操作：新建、檢視、編輯（draft）、送審、核准、作廢
- 欄位：單號、日期、狀態、對象、倉庫、建立人、核准人

#### 單據編輯（Header + Lines）
- Header：日期、倉庫、供應商/客戶、備註
- Lines：品項搜尋選取、數量、單位、單價（採購/銷售）、批號/效期（依產品設定顯示）
- 即時檢核：
  - 出庫不得超過可用庫存（v0.1 可先提示，核准時後端強制）
  - 必填欄位提醒
- 儲存草稿、送審按鈕

#### 庫存查詢
- 篩選：倉庫、品項、關鍵字、批號
- 列表：品項、現有量、平均成本、最後異動時間

#### 盤點
- 建立盤點單（選倉庫）
- 匯入盤點結果（v0.1 可先貼上表格或 CSV 上傳）
- 顯示差異，核准後生成調整 ledger

### 9.3 UI 元件標準
- 表格支援：分頁、排序、欄位顯示切換、匯出 CSV
- 所有表單都有：
  - Dirty state 提示（離開未儲存提醒）
  - 錯誤訊息一致化
- Loading / Empty / Error 狀態一律一致

---

## 10. 狀態機與規則

### 10.1 單據狀態
- draft：可編輯、可刪除（可選）
- submitted：不可編輯，等待核准
- approved：寫入 stock_ledger，庫存生效
- cancelled：不生效（若已 approved，需走沖銷或反核流程，v0.1 可先禁止取消 approved）

### 10.2 庫存規則（核准時）
- 出庫/調撥出庫：檢查 on hand >= qty
- 批號/效期產品：要求 batch_no/expiry_date（依設定）
- 所有 ledger 寫入必須與單據核准在同一個 DB transaction

---

## 11. 專案結構（建議）

### 11.1 後端
backend/
    src/
        main.rs
        app.rs
        config.rs
    db/
        mod.rs
        pool.rs
    migrations/
    modules/
    auth/
    master/
    products.rs
    warehouses.rs
    partners.rs
    docs/
        documents.rs
        approvals.rs
    inventory/
        on_hand.rs
        ledger.rs
    reports/
    middleware/
        auth.rs
        rbac.rs
    types/
        errors.rs
        pagination.rs

### 11.2 前端
frontend/
    src/
    app/
    pages/
    dashboard/
    master/
    docs/
    inventory/
    reports/
    admin/
    components/
    layout/
    forms/
    tables/
    api/
    hooks/
    routes/
    styles/

---

## 12. 測試與驗收（Acceptance Criteria）

### 12.1 核心驗收
1. 可建立產品/倉庫/供應商/客戶，並在 UI 中 CRUD
2. 採購單 -> 核准 -> 採購入庫核准後庫存增加
3. 銷售單 -> 核准 -> 銷售出庫核准後庫存減少，且不可超賣
4. 調撥核准後：來源倉減少、目的倉增加（同一張單兩筆 ledger）
5. 盤點核准後：依差異自動產生調整 ledger
6. 庫存現況與庫存流水可查詢且能追溯到單據

### 12.2 權限驗收
- 倉管不能管理角色權限
- 業務不能做庫存調整
- 主管能核准但不能編輯非自己建立的草稿（可依規則調整）

### 12.3 稽核驗收
- 任一單據核准都會留下 audit_logs
- audit_logs 可查：時間、使用者、動作、單號

---

## 13. v0.2 Roadmap（可選）
- 進階成本法（FIFO）
- 批號效期到期提醒
- 條碼掃描、行動端倉管模式
- Excel 匯入匯出完整化
- 作廢已核准單據：沖銷機制（Reversal Document）
- 庫位管理（Bin Location）
- 與會計/ERP API 對接

---

## 14. 開發優先順序（建議）
1) Auth + RBAC + Master Data  
2) Documents 通用框架（狀態機 + header/lines）  
3) 入庫/出庫（GRN/DO）先跑通 + stock_ledger  
4) 庫存查詢（on-hand + ledger）  
5) 盤點/調整/調撥  
6) 報表匯出 + audit_logs  

---