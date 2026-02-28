# 進銷存系統規格 (iPig ERP)

> **模組**：庫存、採購、銷售管理  
> **版本**：7.0  
> **最後更新**：2026-03-01

---

## 1. 系統目的

iPig ERP 負責管理系統中所有物資的進銷存作業：

- 飼料、藥品、器材、耗材的採購與入庫
- 庫存盤點與調撥
- 成本追蹤與報表
- **客戶分類管理**
- **血液檢查費用追蹤報表**

> **重要**：本系統**不管理動物**，動物屬於動物管理系統。

---

## 2. 角色權限

| 角色 | 權限 |
|------|------|
| SYSTEM_ADMIN | 全權管理 |
| WAREHOUSE_MANAGER | 入庫/出庫/盤點/調撥/採購/報表 |
| ADMIN_STAFF | 基礎操作（查詢、建立銷售單） |
| EXPERIMENT_STAFF | 建立銷售單、唯讀查詢庫存 |

> 僅限內部人員（`is_internal = true`）存取

---

## 3. 核心資料模型

### 3.1 產品主檔 (products)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| sku | VARCHAR(50) | 產品編碼（唯一，系統自動生成） |
| name | VARCHAR(200) | 產品名稱 |
| category_code | CHAR(3) | SKU 類別代碼 |
| subcategory_code | CHAR(3) | SKU 子類別代碼 |
| base_uom | VARCHAR(20) | 基本單位 |
| pack_unit | VARCHAR(20) | 包裝單位 |
| pack_qty | INTEGER | 包裝量 |
| track_batch | BOOLEAN | 追蹤批號 |
| track_expiry | BOOLEAN | 追蹤效期 |
| safety_stock | DECIMAL | 安全庫存量 |
| reorder_point | DECIMAL | 補貨點 |

### 3.2 庫存流水 (stock_ledger)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| product_id | UUID | FK → products.id |
| warehouse_id | UUID | FK → warehouses.id |
| direction | ENUM | in / out / adjust |
| qty | DECIMAL | 異動數量 |
| unit_cost | DECIMAL | 單位成本 |
| batch_no | VARCHAR(50) | 批號 |
| expiry_date | DATE | 效期 |
| doc_type | VARCHAR(20) | 來源單據類型 |
| doc_no | VARCHAR(50) | 來源單據編號 |

### 3.3 夥伴 (partners)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| type | partner_type | 供應商/客戶 |
| customer_category | customer_category | 客戶分類（internal/external/research/other） |
| name | VARCHAR(200) | 名稱 |
| contact_name | VARCHAR(100) | 聯絡人 |
| tax_id | VARCHAR(20) | 統一編號 |

---

## 4. SKU 編碼規則

**格式**：`[類別代碼]-[子類別代碼]-[流水號]`（11 字元）

**範例**：`MED-ANT-001`（藥品-抗生素-第 001 號）

### 4.1 主類別

| 代碼 | 類別 |
|------|------|
| MED | 藥品 |
| MSP | 醫材 |
| FED | 飼料 |
| EQP | 器材 |
| CON | 耗材 |
| CHM | 化學品 |
| OTH | 其他 |

---

## 5. 單據類型

| 代碼 | 類型 | 說明 |
|------|------|------|
| PO | 採購單 | Purchase Order |
| GRN | 採購入庫 | Goods Receipt Note |
| PR | 採購退貨 | Purchase Return |
| SO | 銷售單 | Sales Order（成本價從 stock_ledger 平均成本取得） |
| DO | 銷售出庫 | Delivery Order |
| TR | 調撥單 | Transfer |
| STK | 盤點單 | Stock Take |
| ADJ | 調整單 | Adjustment |
| RTN | 退貨單 | Return |

---

## 6. 報表模組

| 報表 | 頁面元件 | 說明 |
|------|----------|------|
| 庫存現況 | `StockOnHandReportPage` | 即時庫存 |
| 庫存流水 | `StockLedgerReportPage` | 異動明細 |
| 採購明細 | `PurchaseLinesReportPage` | 採購分析 |
| 銷售明細 | `SalesLinesReportPage` | 銷售分析 |
| 成本摘要 | `CostSummaryReportPage` | 成本統計 |
| **血液檢查費用** | `BloodTestCostReportPage` | 專案+日期+實驗室篩選 |

所有報表支援 CSV 匯出。

---

## 7. API 端點

### 7.1 產品管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/products` | 產品列表 |
| POST | `/products` | 新增產品（SKU 自動生成） |
| GET | `/products/:id` | 產品詳情 |
| PUT | `/products/:id` | 更新產品 |
| PATCH | `/products/:id/status` | 變更狀態 |

### 7.2 庫存管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/inventory/on-hand` | 庫存現況 |
| GET | `/inventory/expiring` | 即將到期品項 |
| GET | `/stock-ledger` | 庫存流水 |

### 7.3 單據管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/documents` | 單據列表 |
| POST | `/documents` | 建立單據 |
| GET | `/documents/:id` | 單據詳情 |
| PUT | `/documents/:id` | 編輯單據 |
| POST | `/documents/:id/submit` | 送審 |
| POST | `/documents/:id/approve` | 核准 |

### 7.4 報表

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/reports/stock-on-hand` | 庫存現況報表 |
| GET | `/reports/stock-ledger` | 異動報表 |
| GET | `/reports/purchase-lines` | 採購明細 |
| GET | `/reports/sales-lines` | 銷售明細 |
| GET | `/reports/cost-summary` | 成本分析 |
| GET | `/reports/blood-test-cost` | 血檢成本報表 |

---

## 8. GLP 合規要點

| 要求 | 實作方式 |
|------|----------|
| 可追溯性 | stock_ledger 記錄每筆異動 |
| 批號管理 | batch_no 欄位 |
| 效期管理 | expiry_date + 系統提醒 |
| 數據完整性 | 僅新增不修改，調整用 adjust |

---

## 9. 前端路由

| 路由 | 頁面 |
|------|------|
| `/products` | 產品列表 |
| `/products/new` | 新增產品 |
| `/products/:id` | 產品詳情 |
| `/inventory` | 庫存現況 |
| `/stock-ledger` | 庫存流水 |
| `/documents` | 單據列表 |
| `/warehouses` | 倉庫管理 |
| `/partners` | 供應商/客戶管理 |

---

## 10. 相關文件

- [通知系統](./NOTIFICATION_SYSTEM.md) - 低庫存/效期提醒
- [動物管理](./ANIMAL_MANAGEMENT.md) - 血液檢查費用關聯
