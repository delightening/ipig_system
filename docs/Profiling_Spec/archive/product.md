# 產品管理模組規格 product.md

> **SKU 編碼規則**：本文件的類別與 SKU 生成規則以 `skuSpec.md` 為準。

## 1 模組概述

### 1.1 目的
產品管理模組負責維護系統中所有產品/品項的主檔資料，作為採購、銷售、庫存等模組的基礎資料來源。

### 1.2 核心功能
- 產品列表瀏覽與搜尋
- 新增產品（SKU 自動生成，詳見 `skuSpec.md`）
- 編輯產品資料
- 產品停用/啟用
- 批次匯入/匯出
- 重複產品檢測

### 1.3 關聯模組
| 模組 | 關聯方式 |
|------|----------|
| 採購模組 | 採購單明細引用產品 |
| 銷售模組 | 銷售單明細引用產品 |
| 庫存模組 | 庫存以產品為單位計算 |
| 供應商模組 | 產品可關聯多個供應商 |

---

## 2 資料模型

### 2.1 產品主檔 (products)

| 欄位 | 類型 | 必填 | 說明 |
|------|------|------|------|
| id | UUID | ✓ | 主鍵 |
| sku | VARCHAR(50) | ✓ | 產品識別碼，**系統自動生成，不可修改**，格式：`XXX-XXX-NNN`（11字元） |
| name | VARCHAR(200) | ✓ | 產品名稱 |
| spec | TEXT | ✗ | 規格描述（可選） |
| category_id | UUID | ✗ | 類別 ID FK → product_categories.id（可選，用於層級分類） |
| category_code | CHAR(3) | ✗ | 類別代碼 FK → sku_categories.code（用於 SKU 生成，預設 GEN） |
| subcategory_code | CHAR(3) | ✗ | 子類別代碼 FK → sku_subcategories.code（用於 SKU 生成，預設 OTH） |
| base_uom | VARCHAR(20) | ✓ | 庫存基本單位 |
| pack_unit | VARCHAR(20) | ✗ | 包裝單位（如：盒、箱） |
| pack_qty | INTEGER | ✗ | 包裝量（每包裝含基本單位數，整數） |
| barcode | VARCHAR(50) | ✗ | 原廠條碼（EAN-13, UPC） |
| safety_stock | NUMERIC(18,4) | ✗ | 安全庫存量 |
| safety_stock_uom | VARCHAR(20) | ✗ | 安全庫存單位 |
| reorder_point | NUMERIC(18,4) | ✗ | 補貨點 |
| reorder_point_uom | VARCHAR(20) | ✗ | 補貨點單位 |
| track_batch | BOOLEAN | ✓ | 是否追蹤批號（預設 false） |
| track_expiry | BOOLEAN | ✓ | 是否追蹤效期（預設 false） |
| default_expiry_days | INTEGER | ✗ | 預設有效天數 |
| storage_condition | VARCHAR(50) | ✗ | 保存條件 |
| license_no | VARCHAR(100) | ✗ | 許可證號 |
| status | VARCHAR(20) | ✓ | 狀態：active/inactive/discontinued（預設 active） |
| tags | TEXT[] | ✗ | 搜尋標籤 |
| remark | TEXT | ✗ | 備註 |
| image_url | VARCHAR(500) | ✗ | 產品圖片 URL |
| is_active | BOOLEAN | ✓ | 是否啟用（預設 true） |
| created_at | TIMESTAMPTZ | ✓ | 建立時間 |
| updated_at | TIMESTAMPTZ | ✓ | 更新時間 |
| created_by | UUID | ✗ | 建立者 FK → users.id |

### 2.2 產品類別表 (product_categories)

> 可選的層級分類系統，用於產品管理分類（與 SKU 分類系統獨立）

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| code | VARCHAR(50) | 類別代碼（唯一） |
| name | VARCHAR(100) | 類別名稱 |
| parent_id | UUID | 父類別 ID（層級分類用，可為 NULL） |
| is_active | BOOLEAN | 是否啟用 |
| created_at | TIMESTAMPTZ | 建立時間 |

### 2.3 SKU 類別表 (sku_categories)

> 詳見 `skuSpec.md` 第 6.1 節。用於 SKU 自動生成的分類系統。

| 欄位 | 類型 | 說明 |
|------|------|------|
| code | CHAR(3) | 主鍵，類別代碼 |
| name | VARCHAR(50) | 類別名稱 |
| sort_order | INTEGER | 排序（預設 0） |
| is_active | BOOLEAN | 是否啟用（預設 true） |
| created_at | TIMESTAMPTZ | 建立時間 |

### 2.4 SKU 子類別表 (sku_subcategories)

> 詳見 `skuSpec.md` 第 6.2 節

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| category_code | CHAR(3) | FK → sku_categories.code |
| code | CHAR(3) | 子類別代碼 |
| name | VARCHAR(50) | 子類別名稱 |
| sort_order | INTEGER | 排序（預設 0） |
| is_active | BOOLEAN | 是否啟用（預設 true） |
| created_at | TIMESTAMPTZ | 建立時間 |

> 唯一約束：(category_code, code)

### 2.5 產品單位換算表 (product_uom_conversions)

> 用於定義產品的多單位換算關係（如：1 箱 = 12 個）。基本單位（base_uom）的換算係數為 1。

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| product_id | UUID | FK → products.id（ON DELETE CASCADE） |
| uom | VARCHAR(20) | 單位代碼 |
| factor_to_base | NUMERIC(18,6) | 換算至基本單位的倍數 |
| UNIQUE (product_id, uom) | | 每個產品同一單位只能有一個換算率 |

### 2.6 SKU 流水號表 (sku_sequences)

> 詳見 `skuSpec.md` 第 6.3 節

| 欄位 | 類型 | 說明 |
|------|------|------|
| category_code | CHAR(3) | 類別代碼 |
| subcategory_code | CHAR(3) | 子類別代碼 |
| last_sequence | INTEGER | 最後使用的流水號（預設 0） |

> 主鍵：(category_code, subcategory_code)

### 2.7 預設品類與子類

> **SKU 格式**：`[類別代碼]-[子類別代碼]-[流水號]`，總長度 11 字元
> **範例**：`MED-ANE-001`（藥品-麻醉劑-第001號）

#### 主類別

| 代碼 | 類別 | 說明 |
|------|------|------|
| MED | 藥品 | 藥物、疫苗、注射劑 |
| MSP | 醫材 | 醫療器材、醫療耗材 |
| FED | 飼料 | 各類動物飼料 |
| EQP | 器材 | 可重複使用設備 |
| CON | 耗材 | 一次性消耗品 |
| CHM | 化學品 | 試劑、溶劑、標準品 |
| OTH | 其他 | 未分類項目 |

#### 子類別明細

| 主類別 | 子類別代碼 | 子類別名稱 | 說明 |
|--------|------------|------------|------|
| **MED（藥品）** | | | |
| MED | ANE | 麻醉劑 | Zoletil、Isoflurane 等 |
| MED | ANT | 抗生素 | 抗菌藥物 |
| MED | VAC | 疫苗 | 各類疫苗 |
| MED | PAI | 止痛劑 | 鎮痛藥物 |
| MED | DEW | 驅蟲劑 | Ivermectin 等 |
| MED | OPH | 眼科藥 | 眼藥水、眼藥膏 |
| MED | TOP | 外用藥 | 優點軟膏等 |
| MED | INJ | 注射劑 | 其他注射用藥 |
| MED | ORL | 口服藥 | 口服藥物 |
| MED | OTH | 其他藥品 | 未分類藥品 |
| **MSP（醫材）** | | | |
| MSP | SYR | 注射器材 | 針筒、針頭、留置針 |
| MSP | BND | 敷料繃帶 | 紗布、繃帶、OK繃 |
| MSP | TUB | 導管管路 | 導尿管、引流管 |
| MSP | MON | 監測耗材 | 電極片、血氧探頭 |
| MSP | SUR | 手術耗材 | 縫線、手術刀片 |
| MSP | OTH | 其他醫材 | 未分類醫材 |
| **FED（飼料）** | | | |
| FED | PIG | 豬用飼料 | 一般豬飼料 |
| FED | MIN | 迷你豬飼料 | 迷你豬專用 |
| FED | SUP | 營養補充 | 營養添加劑 |
| FED | OTH | 其他飼料 | 未分類飼料 |
| **EQP（器材）** | | | |
| EQP | SUR | 手術器材 | 手術刀、鉗子等 |
| EQP | MON | 監測設備 | 心跳監測、血氧機 |
| EQP | IMG | 影像設備 | C-arm、超音波等 |
| EQP | ANE | 麻醉設備 | 麻醉機、氣體供應 |
| EQP | RES | 保定設備 | 固定架、保定器 |
| EQP | WEI | 量測設備 | 體重計、量杯 |
| EQP | OTH | 其他器材 | 未分類器材 |
| **CON（耗材）** | | | |
| CON | GLV | 手套 | 各類手套 |
| CON | GAU | 紗布敷料 | 紗布、棉球 |
| CON | CLN | 清潔消毒 | 消毒液、酒精棉 |
| CON | TAG | 標示耗材 | 耳標、標籤 |
| CON | LAB | 實驗耗材 | 試管、吸管、培養皿 |
| CON | OTH | 其他耗材 | 未分類耗材 |
| **CHM（化學品）** | | | |
| CHM | RGT | 試劑 | 化學試劑 |
| CHM | SOL | 溶劑 | 有機溶劑、水溶液 |
| CHM | STD | 標準品 | 標準物質 |
| CHM | BUF | 緩衝液 | pH 緩衝液 |
| CHM | DYE | 染劑 | 染色劑 |
| CHM | OTH | 其他化學品 | 未分類化學品 |
| **OTH（其他）** | | | |
| OTH | GEN | 一般 | 通用項目 |

### 2.8 庫存單位代碼

| 代碼 | 名稱 | 說明 |
|------|------|------|
| EA | 個/支 | 最小單位 |
| TB | 錠 | 錠劑 |
| CP | 顆/膠囊 | 膠囊 |
| BT | 瓶 | 瓶裝 |
| BX | 盒 | 盒裝 |
| PK | 包 | 包裝 |
| RL | 卷 | 卷裝 |
| SET | 組 | 套組 |
| ML | 毫升 | 液體 |
| L | 公升 | 液體 |
| G | 公克 | 重量 |
| KG | 公斤 | 重量 |
| PR | 雙 | 手套等成對物品 |
| VL | 瓶（小） | 小瓶/安瓿 |

### 2.9 產品狀態

| 狀態 | 說明 |
|------|------|
| active | 正常使用中 |
| inactive | 暫時停用（可恢復） |
| discontinued | 已停產（不可恢復，僅供歷史查詢） |

### 2.10 保存條件

| 代碼 | 名稱 | 說明 |
|------|------|------|
| RT | 常溫 | 15-25°C |
| RF | 冷藏 | 2-8°C |
| FZ | 冷凍 | -20°C 以下 |
| DK | 避光 | 避免光照 |
| DY | 乾燥 | 避免潮濕 |

---

## 3 頁面規格

### 3.1 產品列表頁 `/products`

#### 3.1.1 頁面結構
```
┌──────────────────────────────────────────────────────────┐
│ 產品管理                                    [+ 新增產品]  │
│ 管理系統中的產品/品項資料                                  │
├──────────────────────────────────────────────────────────┤
│ [搜尋產品...]  [品類 ▼]  [狀態 ▼]  [更多篩選]  [匯入] [匯出] │
├──────────────────────────────────────────────────────────┤
│ ☐ │ SKU         │ 名稱       │ 規格    │ 單位 │ 狀態 │ 操作 │
│───┼─────────────┼────────────┼─────────┼──────┼──────┼──────│
│ ☐ │ MED-ANT-001 │ Amoxic...  │ 500mg錠 │ TB   │ 啟用 │ ⋮    │
│ ☐ │ CON-GLV-001 │ 乳膠手套   │ L號無粉 │ BX   │ 啟用 │ ⋮    │
│ ☐ │ MSP-SYR-001 │ 注射針筒   │ 10ml    │ EA   │ 啟用 │ ⋮    │
│ ☐ │ CHM-RGT-001 │ 生理食鹽水 │ 500ml   │ BT   │ 啟用 │ ⋮    │
│ ☐ │ ...         │ ...        │ ...     │ ...  │ ...  │ ...  │
├──────────────────────────────────────────────────────────┤
│ 顯示 1-20 共 156 筆                    [< 1 2 3 4 5 ... >]│
└──────────────────────────────────────────────────────────┘
```

#### 3.1.2 搜尋與篩選

| 篩選項 | 類型 | 說明 |
|--------|------|------|
| 關鍵字搜尋 | 文字 | 搜尋 SKU、名稱、規格、標籤 |
| 品類 | 下拉單選 | 依品類篩選 |
| 子類 | 下拉單選 | 依子類篩選（連動品類） |
| 狀態 | 下拉單選 | 全部/啟用/停用/停產 |
| 追蹤批號 | 下拉單選 | 全部/是/否 |
| 追蹤效期 | 下拉單選 | 全部/是/否 |
| 保存條件 | 下拉多選 | 常溫/冷藏/冷凍等 |

#### 3.1.3 表格欄位

| 欄位 | 寬度 | 排序 | 說明 |
|------|------|------|------|
| 勾選框 | 40px | ✗ | 批次操作用 |
| SKU | 180px | ✓ | 點擊複製，hover 顯示完整 |
| 名稱 | auto | ✓ | 點擊進入詳情頁 |
| 規格 | 150px | ✗ | |
| 單位 | 60px | ✗ | |
| 安全庫存 | 100px | ✓ | 顯示「數量 單位」 |
| 批號 | 60px | ✗ | Badge: 啟用/— |
| 效期 | 60px | ✗ | Badge: 啟用/— |
| 狀態 | 80px | ✓ | Badge: 啟用/停用/停產 |
| 操作 | 60px | ✗ | 更多選單 |

#### 3.1.4 操作選單

| 操作 | 權限 | 說明 |
|------|------|------|
| 檢視 | 所有人 | 進入產品詳情頁 |
| 編輯 | Admin, Secretary | 進入編輯頁 |
| 複製 | Admin, Secretary | 複製產品，進入新增頁並帶入資料 |
| 停用 | Admin | 將產品設為停用 |
| 啟用 | Admin | 將產品設為啟用 |
| 標記停產 | Admin | 將產品標記為停產 |

#### 3.1.5 批次操作

當勾選一個以上產品時，顯示批次操作列：

| 操作 | 權限 | 說明 |
|------|------|------|
| 批次停用 | Admin | 停用選中的產品 |
| 批次匯出 | Admin, Secretary | 匯出選中產品為 Excel |
| 批次設定標籤 | Admin, Secretary | 為選中產品添加/移除標籤 |

---

### 3.2 新增產品頁 `/products/new`

詳細規格請參考 `skuSpec.md`

#### 3.2.1 頁面佈局
```
┌─────────────────────────────────────────────────────────────────┐
│ ← 新增產品                                                        │
│   建立新的產品/品項資料，SKU 將由系統自動產生                          │
├───────────────────────────────────┬─────────────────────────────┤
│                                   │ ┌─────────────────────────┐ │
│  ┌─ 基本資訊 ─────────────────┐   │ │ SKU（系統自動產生）      │ │
│  │ 產品名稱 *  [            ] │   │ │ ┌─────────────────────┐ │ │
│  │ 規格描述 *  [            ] │   │ │ │   MED-ANT-007       │ │ │
│  │ 類別 *      [▼ 藥品 MED  ] │   │ │ │              [📋]   │ │ │
│  │ 子類別 *    [▼ 抗生素 ANT] │   │ │ └─────────────────────┘ │ │
│  └────────────────────────────┘   │ │                         │ │
│                                   │ │ SKU 格式說明：           │ │
│  ┌─ 包裝與單位 ───────────────┐   │ │  MED = 藥品            │ │
│  │ 庫存單位 *  [▼ TB 錠    ] │   │ │  ANT = 抗生素          │ │
│  │ 包裝量 *    [10        ] │   │ │  007 = 流水號          │ │
│  └────────────────────────────┘   │ │                         │ │
│                                   │ └─────────────────────────┘ │
│  ┌─ 庫存管理 ─────────────────┐   │                           │
│  │ 安全庫存                    │   │ ┌─────────────────────────┐ │
│  │ [10][50][100][500]         │   │ │      [建立產品]         │ │
│  │ [────●──────────────]      │   │ └─────────────────────────┘ │
│  │   [-] [ 100 ] [+]  [▼ TB]  │   │                           │
│  │                             │   │ SKU 將在建立時由系統自動    │
│  │ 補貨點                      │   │ 產生流水號（001-999）       │
│  │ [────●──────────────]      │   │                           │
│  │   [-] [  50 ] [+]  [▼ TB]  │   │                           │
│  └────────────────────────────┘   │                           │
│                                   │                           │
│  ┌─ 追蹤設定 ─────────────────┐   │                           │
│  │ 追蹤批號  [○ 關]           │   │                           │
│  │ 追蹤效期  [● 開]           │   │                           │
│  │ 預設有效天數 [730] 天      │   │                           │
│  └────────────────────────────┘   │                           │
│                                   │                           │
│  ┌─ 其他資訊 ─────────────────┐   │                           │
│  │ 保存條件  [▼ 常溫        ] │   │                           │
│  │ 原廠條碼  [              ] │   │                           │
│  │ 搜尋標籤  [抗生素][口服][+]│   │                           │
│  │ 備註                        │   │                           │
│  │ [                         ]│   │                           │
│  └────────────────────────────┘   │                           │
└───────────────────────────────────┴─────────────────────────────┘
```

---

### 3.3 產品詳情頁 `/products/:id`

#### 3.3.1 頁面結構
```
┌──────────────────────────────────────────────────────────────────┐
│ ← 產品詳情                                        [編輯] [更多 ▼] │
├──────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐                                                   │
│ │   [圖片]    │  Amoxicillin 500mg Tablet                        │
│ │             │  SKU: MED-ANT-001 [📋]                            │
│ └─────────────┘  類別: 藥品 > 抗生素  │  狀態: ● 啟用             │
├──────────────────────────────────────────────────────────────────┤
│ [基本資訊] [庫存設定] [相關單據] [異動紀錄]                        │
├──────────────────────────────────────────────────────────────────┤
│  基本資訊                                                         │
│  ─────────────────────────────────────────────                   │
│  產品名稱    Amoxicillin                                          │
│  規格描述    500mg Tablet                                         │
│  類別        MED（藥品）                                          │
│  子類別      ANT（抗生素）                                        │
│  庫存單位    TB（錠）                                              │
│  包裝量      10                                                    │
│  原廠條碼    4710088123456                                         │
│  保存條件    常溫                                                  │
│  許可證號    衛署藥製字第012345號                                  │
│  搜尋標籤    [抗生素] [口服] [處方藥]                              │
│  備註        避光保存，開封後請於30天內使用完畢                     │
│                                                                   │
│  追蹤設定                                                         │
│  ─────────────────────────────────────────────                   │
│  追蹤批號    否                                                    │
│  追蹤效期    是（預設有效天數：730 天）                            │
│                                                                   │
│  系統資訊                                                         │
│  ─────────────────────────────────────────────                   │
│  建立時間    2026-01-07 14:30:25                                  │
│  建立者      王小明                                                │
│  更新時間    2026-01-07 15:20:10                                  │
└──────────────────────────────────────────────────────────────────┘
```

#### 3.3.2 分頁內容

| 分頁 | 內容 |
|------|------|
| 基本資訊 | 產品主檔資料 |
| 庫存設定 | 安全庫存、補貨點、各倉庫庫存快照 |
| 相關單據 | 最近的採購單、銷售單、庫存異動 |
| 異動紀錄 | 產品資料的變更歷史（Audit Log） |

---

### 3.4 編輯產品頁 `/products/:id/edit`

#### 3.4.1 與新增頁的差異

| 項目 | 新增頁 | 編輯頁 |
|------|--------|--------|
| SKU | 預覽狀態，建立後確定 | 唯讀，不可修改 |
| 產品名稱 | 可輸入 | 可修改 |
| 規格描述 | 可輸入 | 可修改 |
| 品類/子類 | 可選擇 | 可修改（但不影響 SKU） |
| 狀態 | 預設 active | 可修改 |

#### 3.4.2 SKU 不可修改提示
```
┌─────────────────────────────────────┐
│ SKU                                  │
│ ┌─────────────────────────────────┐ │
│ │ MED-ANT-001                 🔒  │ │
│ └─────────────────────────────────┘ │
│ ⓘ SKU 在產品建立後不可修改          │
└─────────────────────────────────────┘
```

---

## 4 批次匯入/匯出

### 4.1 匯出功能

#### 4.1.1 匯出格式
- Excel (.xlsx)
- CSV (.csv)

#### 4.1.2 匯出欄位
| 欄位 | 是否包含 |
|------|----------|
| SKU | ✓ |
| 名稱 | ✓ |
| 規格 | ✓ |
| 品類 | ✓ |
| 子類 | ✓ |
| 單位 | ✓ |
| 安全庫存 | ✓ |
| 補貨點 | ✓ |
| 追蹤批號 | ✓ |
| 追蹤效期 | ✓ |
| 狀態 | ✓ |
| 標籤 | ✓ |
| 備註 | ✓ |

### 4.2 匯入功能

#### 4.2.1 匯入流程
```
1. 下載範本 → 2. 填寫資料 → 3. 上傳檔案 → 4. 預覽驗證 → 5. 確認匯入
```

#### 4.2.2 匯入範本欄位
| 欄位 | 必填 | 說明 |
|------|------|------|
| 名稱 | ✓ | 產品名稱 |
| 規格 | ✗ | 規格描述（可選） |
| 類別代碼 | ✗ | MED/MSP/FED/EQP/CON/CHM/OTH（預設 GEN） |
| 子類別代碼 | ✗ | 如 ANT, SYR, GLV 等（預設 OTH，見 2.7 子類別明細） |
| 基本單位代碼 | ✓ | 如 TB, BX, EA |
| 包裝單位 | ✗ | 包裝單位代碼（如：BX） |
| 包裝量 | ✗ | 每包裝含基本單位數（整數） |
| 安全庫存 | ✗ | 數值 |
| 安全庫存單位 | ✗ | 單位代碼 |
| 補貨點 | ✗ | 數值 |
| 補貨點單位 | ✗ | 單位代碼 |
| 追蹤批號 | ✗ | Y/N（預設 N） |
| 追蹤效期 | ✗ | Y/N（預設 N） |
| 預設有效天數 | ✗ | 整數（天數） |
| 保存條件 | ✗ | RT/RF/FZ/DK/DY |
| 原廠條碼 | ✗ | |
| 許可證號 | ✗ | |
| 標籤 | ✗ | 逗號分隔 |
| 備註 | ✗ | |

**注意：SKU 由系統自動產生（格式：`XXX-XXX-NNN`），匯入時不需填寫**

#### 4.2.3 匯入驗證
| 驗證項目 | 錯誤訊息 |
|----------|----------|
| 必填欄位 | 第 X 列：名稱為必填 |
| 品類代碼不存在 | 第 X 列：品類代碼 'XXX' 不存在 |
| 單位代碼不存在 | 第 X 列：單位代碼 'XXX' 不存在 |
| 重複產品 | 第 X 列：與現有產品可能重複（名稱+規格+單位相同） |

#### 4.2.4 匯入預覽畫面
```
┌──────────────────────────────────────────────────────────────────┐
│ 匯入預覽                                                         │
├──────────────────────────────────────────────────────────────────┤
│ 檔案：products_import_20260107.xlsx                              │
│ 總筆數：50  │  成功：48  │  警告：1  │  錯誤：1                  │
├──────────────────────────────────────────────────────────────────┤
│ 列 │ 名稱        │ 規格      │ 狀態     │ 訊息                   │
│────┼─────────────┼───────────┼──────────┼────────────────────────│
│ 2  │ Amoxicil... │ 500mg...  │ ✓ 成功   │                        │
│ 3  │ 手套        │ L號       │ ⚠ 警告   │ 與現有產品可能重複      │
│ 4  │ 生理食鹽水  │           │ ✗ 錯誤   │ 規格為必填              │
│ ...│ ...         │ ...       │ ...      │ ...                    │
├──────────────────────────────────────────────────────────────────┤
│                                        [取消]  [略過錯誤並匯入]    │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5 API 規格

### 5.1 產品列表
`GET /api/products`

Query Parameters:
| 參數 | 類型 | 說明 |
|------|------|------|
| keyword | string | 搜尋關鍵字（SKU、名稱） |
| category_id | uuid | 品類 ID（product_categories.id） |
| category_code | string | 類別代碼（sku_categories.code，如 MED, MSP） |
| subcategory_code | string | 子類別代碼（sku_subcategories.code，如 ANT, SYR） |
| status | string | active/inactive/discontinued |
| track_batch | boolean | 是否追蹤批號 |
| track_expiry | boolean | 是否追蹤效期 |
| storage_condition | string | 保存條件（RT/RF/FZ/DK/DY） |
| is_active | boolean | 是否啟用 |
| sort_by | string | 排序欄位 |
| sort_order | string | asc/desc |

Response:
```json
{
  "data": [
    {
      "id": "uuid",
      "sku": "MED-ANT-001",
      "name": "Amoxicillin",
      "spec": "500mg Tablet",
      "category_code": "MED",
      "category_name": "藥品",
      "subcategory_code": "ANT",
      "subcategory_name": "抗生素",
      "base_uom": "TB",
      "pack_unit": "BX",
      "pack_qty": 10,
      "safety_stock": 100,
      "track_batch": false,
      "track_expiry": true,
      "status": "active",
      "is_active": true,
      "uom_conversions": [
        {
          "id": "uuid",
          "uom": "BX",
          "factor_to_base": 10
        }
      ]
    }
  ],
  "total": 156,
  "page": 1,
  "per_page": 20,
  "total_pages": 8
}
```

### 5.2 取得單一產品
`GET /api/products/:id`

### 5.3 新增產品
`POST /api/products`

Request:
```json
{
  "name": "Amoxicillin",
  "spec": "500mg Tablet",
  "category_code": "MED",
  "subcategory_code": "ANT",
  "base_uom": "TB",
  "pack_unit": "BX",
  "pack_qty": 10,
  "track_batch": false,
  "track_expiry": true,
  "default_expiry_days": 730,
  "safety_stock": 100,
  "safety_stock_uom": "TB",
  "reorder_point": 50,
  "reorder_point_uom": "TB",
  "storage_condition": "RT",
  "barcode": "4710088123456",
  "license_no": "衛署藥製字第012345號",
  "tags": ["抗生素", "口服"],
  "remark": "避光保存",
  "uom_conversions": [
    {
      "uom": "BX",
      "factor_to_base": 10
    }
  ]
}
```

**注意：**
- SKU 由系統自動生成，格式：`{category_code}-{subcategory_code}-{sequence}`
- 如未提供 `category_code` 或 `subcategory_code`，系統會使用預設值（GEN, OTH）
- `uom_conversions` 為可選，用於定義多單位換算關係

詳見 @skuSpec.md 第 10.2 節

### 5.4 更新產品
`PUT /api/products/:id`

Request:
```json
{
  "name": "Amoxicillin",
  "spec": "500mg Tablet",
  "category_code": "MED",
  "subcategory_code": "ANT",
  "base_uom": "TB",
  "pack_unit": "BX",
  "pack_qty": 10,
  "safety_stock": 100,
  "safety_stock_uom": "TB",
  "reorder_point": 50,
  "reorder_point_uom": "TB",
  "track_batch": false,
  "track_expiry": true,
  "default_expiry_days": 730,
  "storage_condition": "RT",
  "barcode": "4710088123456",
  "license_no": "衛署藥製字第012345號",
  "tags": ["抗生素", "口服"],
  "remark": "避光保存",
  "uom_conversions": [
    {
      "uom": "BX",
      "factor_to_base": 10
    },
    {
      "uom": "CT",
      "factor_to_base": 120
    }
  ]
}
```

**注意：**
- SKU 欄位不可更新（變更類別/子類別不會影響已產生的 SKU）
- `uom_conversions` 為可選欄位，用於定義多重單位換算（例如：1 盒(BX) = 10 錠(TB)、1 箱(CT) = 120 錠(TB)）

### 5.5 變更產品狀態
`PATCH /api/products/:id/status`

Request:
```json
{
  "status": "inactive"
}
```

### 5.6 批次匯入
`POST /api/products/import`

Content-Type: multipart/form-data

| 欄位 | 類型 | 說明 |
|------|------|------|
| file | File | Excel 或 CSV 檔案 |
| skip_errors | boolean | 是否略過錯誤繼續匯入 |

### 5.7 批次匯出
`GET /api/products/export`

Query Parameters:
| 參數 | 類型 | 說明 |
|------|------|------|
| ids | string | 逗號分隔的 ID（選擇性匯出） |
| format | string | xlsx/csv |
| ...其他篩選參數 | | 同列表 API |

### 5.8 檢查重複產品
`POST /api/products/check-duplicate`

Request:
```json
{
  "name": "Amoxicillin",
  "spec": "500mg Tablet",
  "category_code": "MED",
  "subcategory_code": "ANT"
}
```

Response:
```json
{
  "is_duplicate": true,
  "similar_products": [
    {
      "id": "uuid",
      "sku": "MED-ANT-001",
      "name": "Amoxicillin",
      "spec": "500mg Tablet",
      "similarity": 0.95
    }
  ]
}
```

---

## 6 權限控制

### 6.1 權限代碼

| 代碼 | 說明 |
|------|------|
| product.create | 建立產品 |
| product.read | 檢視產品 |
| product.update | 更新產品 |
| product.delete | 刪除產品 |
| product.import | 匯入產品 |
| product.export | 匯出產品 |
| product.status | 變更狀態 |

### 6.2 角色權限對照

| 權限 | Admin | Secretary | Warehouse | Researcher |
|------|-------|-----------|-----------|------------|
| product.create | ✓ | ✓ | ✗ | ✗ |
| product.read | ✓ | ✓ | ✓ | ✓ |
| product.update | ✓ | ✓ | ✗ | ✗ |
| product.delete | ✓ | ✗ | ✗ | ✗ |
| product.import | ✓ | ✓ | ✗ | ✗ |
| product.export | ✓ | ✓ | ✓ | ✗ |
| product.status | ✓ | ✗ | ✗ | ✗ |

---

## 7 稽核日誌

### 7.1 記錄事件

| 事件 | 說明 |
|------|------|
| product.created | 產品建立 |
| product.updated | 產品更新 |
| product.status_changed | 狀態變更 |
| product.deleted | 產品刪除 |
| product.imported | 批次匯入 |

### 7.2 記錄內容

```json
{
  "id": "uuid",
  "actor_user_id": "uuid",
  "action": "product.updated",
  "entity_type": "product",
  "entity_id": "uuid",
  "before_data": {
    "safety_stock": 50
  },
  "after_data": {
    "safety_stock": 100
  },
  "created_at": "2026-01-07T15:20:10Z"
}
```

---

## 8 SKU 範例參考

> 完整 SKU 規則請參考 `skuSpec.md`

| SKU | 產品名稱 | 類別 | 子類別 | 說明 |
|-----|---------|------|--------|------|
| MED-ANE-001 | Zoletil-50 | 藥品 | 麻醉劑 | 麻醉劑 |
| MED-ANT-001 | Amoxicillin | 藥品 | 抗生素 | 抗生素 |
| MED-VAC-001 | 豬瘟疫苗 | 藥品 | 疫苗 | 疫苗 |
| MED-DEW-001 | Ivermectin | 藥品 | 驅蟲劑 | 驅蟲劑 |
| MSP-SYR-001 | 10ml 注射針筒 | 醫材 | 注射器材 | 醫療器材 |
| MSP-BND-001 | 無菌紗布 | 醫材 | 敷料繃帶 | 敷料 |
| FED-PIG-001 | 大白豬飼料 20kg | 飼料 | 豬用飼料 | 飼料 |
| FED-MIN-001 | 迷你豬飼料 10kg | 飼料 | 迷你豬飼料 | 迷你豬專用 |
| EQP-MON-001 | 血氧監測儀 | 器材 | 監測設備 | 設備 |
| EQP-SUR-001 | 手術刀組 | 器材 | 手術器材 | 手術器材 |
| CON-GLV-001 | 乳膠手套 M | 耗材 | 手套 | 一次性耗材 |
| CON-LAB-001 | 10ml 試管 | 耗材 | 實驗耗材 | 實驗室耗材 |
| CHM-RGT-001 | 生理食鹽水 | 化學品 | 試劑 | 試劑 |
| CHM-SOL-001 | 75% 酒精 | 化學品 | 溶劑 | 溶劑 |
| CHM-STD-001 | pH 標準液 | 化學品 | 標準品 | 標準品 |

---

## 9 變更紀錄

| 版本 | 日期 | 變更內容 |
|------|------|----------|
| v1.0 | 2026-01-07 | 初始版本 |
| v1.1 | 2026-01-08 | 依 skuSpec.md 統一 SKU 格式，新增醫材(MSP)、化學品(CHM)類別 |
| v1.2 | 2026-01-10 | 更新資料模型以符合實際實作：<br>- `spec` 改為可選欄位（TEXT 型別）<br>- `category_code` 和 `subcategory_code` 改為可選（預設值 GEN/OTH）<br>- `pack_qty` 型別改為 INTEGER<br>- `sku` 欄位長度改為 VARCHAR(50)<br>- `safety_stock` 和 `reorder_point` 型別改為 NUMERIC(18,4)<br>- 新增 `category_id` 和 `pack_unit` 欄位說明<br>- 新增 `product_uom_conversions` 表說明<br>- 新增 `product_categories` 表說明（層級分類系統）<br>- 更新 API 規格以包含 `uom_conversions` 和 `pack_unit`<br>- 更新查詢參數以包含 `category_code`、`subcategory_code`、`storage_condition`、`is_active`<br>- 修正匯入範本欄位說明以反映實際必填/可選狀態 |
