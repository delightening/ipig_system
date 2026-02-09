# 擴展性規格

> **版本**：1.0  
> **最後更新**：2026-01-17  
> **對象**：架構師、資深開發人員

---

## 1. 目的

本文件描述 iPig 的擴展性架構，使系統能夠擴展以支援：

- **新動物物種** - 豬隻以外的動物（兔、小鼠等）
- **新設施** - 額外的研究中心、建築
- **新角色與部門** - 組織成長
- **新模組** - 未來功能

指導原則：**「像 Notion 一樣成長，不干擾使用者」**
- 新增式變更優先於破壞性變更
- 穩定的導覽與工作流程
- 清晰的遷移路徑
- 最小化重新訓練需求

---

## 2. 現況分析

### 2.1 動物（豬隻）

| 現況 | 限制 |
|------|------|
| `pigs` 資料表硬編碼 | 新增物種需建立新資料表 |
| `breed` 使用 ENUM | 新品種需執行遷移 |
| 路由 `/api/pigs/*` | URL 綁定物種 |
| 權限 `pig.*` | 每物種需獨立權限集 |
| `ear_tag` 識別碼 | 並非所有物種使用耳號 |

### 2.2 設施/位置

| 現況 | 限制 |
|------|------|
| `pen_location` 為 VARCHAR | 無結構（如「A01」）|
| 區域顏色寫在 CSS | 硬編碼，不可設定 |
| 畫面上 A/B 棟 | 硬編碼切換按鈕 |

### 2.3 角色/部門

| 現況 | 限制 |
|------|------|
| 扁平角色模型 | 無階層 |
| 無部門概念 | 無法依團隊組織 |
| 無直屬主管 | 無法自動路由核准 |

---

## 3. 新架構

### 3.1 物種抽象化

```sql
-- 通用物種設定
CREATE TABLE species (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE, -- 'pig', 'rabbit', 'mouse'
    name VARCHAR(100),        -- '豬', '兔', '小鼠'
    config JSONB              -- 物種特定設定
);

-- 豬隻設定範例：
{
    "breeds": ["Minipig", "White", "Other"],
    "identifier_label": "耳號",
    "identifier_format": "###",
    "has_pen_assignment": true,
    "pen_prefixes": ["A", "B", "C", "D", "E", "F", "G"]
}
```

#### 遷移策略

1. 建立 `species` 資料表，含豬隻種子資料
2. 在 `pigs` 資料表新增 `species_id`（初始允許 NULL）
3. 回填現有豬隻的物種 ID
4. 將 `species_id` 設為 NOT NULL
5. 建立 `animals` 視圖以提供通用存取

### 3.2 設施階層

```
設施 (Facility)
    └── 棟舍 (Building)
            └── 區域 (Zone)
                    └── 欄位 (Pen)
```

```sql
CREATE TABLE facilities (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    address TEXT
);

CREATE TABLE buildings (
    id UUID PRIMARY KEY,
    facility_id UUID REFERENCES facilities(id),
    code VARCHAR(20),     -- 'A', 'B'
    name VARCHAR(100),    -- 'A棟', 'B棟'
    config JSONB          -- 棟舍特定設定
);

CREATE TABLE zones (
    id UUID PRIMARY KEY,
    building_id UUID REFERENCES buildings(id),
    code VARCHAR(20),     -- 'A', 'B', 'C'...
    color VARCHAR(20),    -- '#4CAF50'（可設定！）
    layout_config JSONB   -- 行列配置
);

CREATE TABLE pens (
    id UUID PRIMARY KEY,
    zone_id UUID REFERENCES zones(id),
    code VARCHAR(20),     -- 'A01', 'A02'
    capacity INTEGER,
    status VARCHAR(20)    -- 'available', 'occupied', 'maintenance'
);
```

#### 遷移策略

1. 建立新資料表，不含對 `pigs` 的外鍵
2. 植入目前 A/B 棟結構
3. 在 `pigs` 新增 `pen_id`（允許 NULL）
4. 建立腳本將 `pen_location` 對應至 `pen_id`
5. 執行完全符合項目的自動遷移
6. 不符合項目需人工審查
7. 保留 `pen_location` 以向後相容（標記棄用）

### 3.3 部門與主管階層

```sql
CREATE TABLE departments (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    parent_id UUID REFERENCES departments(id), -- 階層
    manager_id UUID REFERENCES users(id)       -- 部門主管
);

-- 新增至 users
ALTER TABLE users ADD COLUMN department_id UUID;
ALTER TABLE users ADD COLUMN direct_manager_id UUID;
```

此架構可實現：
- 自動核准路由（員工 → 主管 → 部門主管）
- 基於部門的權限
- 團隊檢視功能

### 3.4 角色群組

常見職能的預定義角色組合：

```sql
CREATE TABLE role_groups (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(100),
    description TEXT
);

CREATE TABLE role_group_roles (
    role_group_id UUID REFERENCES role_groups(id),
    role_id UUID REFERENCES roles(id)
);
```

角色群組範例：
- `INTERNAL_STAFF` → 基本 HR 權限
- `EXPERIMENT_TEAM` → 動物管理權限
- `ADMIN_TEAM` → 系統管理權限

---

## 4. 擴展點

### 4.1 擴展 API 設計

#### 現行（物種特定）
```
GET /api/pigs
GET /api/pigs/:id
POST /api/pigs
```

#### 新設計（通用 + 別名）
```
# 通用端點
GET /api/animals?species=pig
GET /api/animals/:id
POST /api/animals { species_id: "...", ... }

# 向後相容別名（棄用警告）
GET /api/pigs → 重導向至 /api/animals?species=pig
```

### 4.2 權限模式

#### 現行
```
pig.read, pig.create, pig.update, pig.delete
```

#### 新設計
```
# 通用
animal.read, animal.create, animal.update, animal.delete

# 物種特定（選用，細粒度控制）
animal.pig.read, animal.rabbit.read
```

### 4.3 UI 設定

不再硬編碼區域顏色：

```typescript
// 從管理員設定或 API 取得
const zoneConfig = await fetchZoneConfig();
// { "A": { color: "#4CAF50", ... }, "B": { color: "#2196F3", ... } }
```

設施選擇器變為動態：
```typescript
const buildings = await fetchBuildings();
// 動態頁籤取代硬編碼的 A棟/B棟
```

---

## 5. 實作階段

### 第一階段：資料庫基礎（第 1-2 週）✅

- [x] 建立 `species` 資料表並植入豬隻
- [x] 建立 `facilities`、`buildings`、`zones`、`pens` 資料表
- [x] 植入現有欄位結構
- [x] 建立 `departments` 資料表
- [x] 在 users 新增 `department_id`、`direct_manager_id`
- [x] 在 pigs 新增 `pen_id`
- [x] 建立 `pen_details` 視圖

### 第二階段：後端抽象（第 3-4 週）

- [ ] 建立基於 `pigs` 的 `animals` 視圖
- [ ] 新增 `/api/animals/*` 路由
- [ ] 保留 `/api/pigs/*` 作為別名，附棄用標頭
- [ ] 建立 `/api/facilities/*` CRUD
- [ ] 建立 `/api/buildings/*` CRUD
- [ ] 建立 `/api/zones/*` CRUD
- [ ] 建立 `/api/pens/*` CRUD
- [ ] 更新權限檢查為通用模式

### 第三階段：前端漸進增強（第 5-8 週）

- [ ] 建立可設定的區域顏色系統
- [ ] 建立動態設施/棟舍選擇器
- [ ] 建立設施管理管理員頁面
- [ ] 逐步替換硬編碼的欄位參照
- [ ] 功能開關控制新元件

### 第四階段：上線與清理（第 9-12 週）

- [ ] 對測試使用者啟用新 UI
- [ ] 收集回饋並迭代
- [ ] 對所有使用者啟用
- [ ] 記錄棄用端點的使用
- [ ] 公告棄用時程
- [ ] 6 個月後移除棄用程式碼

---

## 6. 向後相容性

### 6.1 舊版查詢視圖

```sql
-- 保留 pigs 作為主要資料表名稱
-- 或建立看起來像舊結構的視圖
CREATE VIEW legacy_pigs AS
SELECT 
    p.*,
    pn.code AS pen_location_normalized
FROM pigs p
LEFT JOIN pens pn ON p.pen_id = pn.id;
```

### 6.2 路由別名

```rust
// 在 routes.rs
.route("/pigs", get(handlers::list_pigs_deprecated))  // 記錄棄用
.route("/animals", get(handlers::list_animals))        // 新端點
```

### 6.3 棄用標頭

```rust
fn add_deprecation_header(response: &mut Response) {
    response.headers_mut().insert(
        "Deprecation", 
        HeaderValue::from_static("true")
    );
    response.headers_mut().insert(
        "Sunset",
        HeaderValue::from_static("2027-01-01")
    );
}
```

---

## 7. 新增物種範例

### 7.1 資料庫變更

```sql
-- 新增物種
INSERT INTO species (code, name, config) VALUES (
    'rabbit', 
    '兔',
    '{"breeds": ["NZW", "JW"], "identifier_label": "耳號"}'
);

-- 建立物種特定資料表（選用，若綱要差異較大）
-- 或若綱要足夠相似，直接使用 animals 資料表
```

### 7.2 後端變更

若使用現有 `pigs` 綱要：
- 只需在查詢中加入 species_id 篩選

若物種需要自訂欄位：
- 建立新資料表，包含共用與自訂欄位
- 擴展動物處理器以路由至正確資料表

### 7.3 前端變更

```typescript
// 若使用設定導向方式則已是動態
const species = await fetchSpecies();
// 自動渲染物種選擇器
```

---

## 8. 新增設施範例

### 8.1 管理員 UI

1. 前往 **系統管理 → 設施管理**
2. 點擊 **新增設施**
3. 輸入名稱、地址
4. 在設施下新增棟舍
5. 在棟舍下新增區域
6. 在區域下新增欄位

### 8.2 資料輸入

```sql
INSERT INTO facilities (code, name) VALUES ('BRANCH2', '二廠');
INSERT INTO buildings (facility_id, code, name) VALUES (?, 'A', 'A棟');
INSERT INTO zones (building_id, code, color) VALUES (?, 'A', '#4CAF50');
-- ... 欄位
```

### 8.3 UI 更新

若使用動態設施載入，無需程式碼變更：
- 設施選擇器自動顯示新設施
- 棟舍頁籤自動填入
- 欄位格線自動依設定產生

---

## 9. 導覽演進

### 現行結構
```
📊 儀表板
📋 動物使用計畫 (AUP)
🐷 動物管理
  └─ 豬隻
  └─ 我的計劃
📦 ERP
⚙️ 系統管理
```

### 未來結構（新增式）
```
📊 儀表板
📋 動物使用計畫 (AUP)
🐷 動物管理
  ├─ 所有動物
  ├─ 依物種瀏覽 ▶
  │   ├─ 豬隻
  │   ├─ 兔（未來）
  │   └─ 小鼠（未來）
  └─ 我的計劃
📦 ERP
🏢 設施管理 ▶（僅限管理員）
  ├─ 設施/棟舍
  └─ 欄位配置
👥 人員管理 ▶（HR）
  ├─ 出勤打卡
  └─ 請假管理
⚙️ 系統管理
  ├─ ...現有...
  └─ 安全審計 ▶（新增）
```

**關鍵原則**：現有導覽項目保持原位。新項目新增於末端或作為子項目。

---

## 10. 風險緩解

| 風險 | 緩解措施 |
|------|----------|
| 資料遷移錯誤 | 先於測試環境執行遷移；邊界案例人工審查 |
| 效能退化 | 測試查詢效能；主動新增索引 |
| 使用者困惑 | 分階段上線；收集回饋；提供培訓 |
| 破壞整合 | 舊端點設定 6 個月棄用期 |
| 遷移不完整 | 追蹤未遷移紀錄；提供管理員工具 |

---

## 11. 成功指標

| 指標 | 目標 |
|------|------|
| 舊端點使用率 | 3 個月後 <10% |
| 新 UI 採用率 | 1 個月後 >90% |
| 客服工單量 | 上線期間無增加 |
| 查詢效能 | 與現行相同或更佳 |
| 使用者滿意度 | 導覽變更無抱怨 |

---

## 12. 相關文件

- [資料庫綱要](./04_DATABASE_SCHEMA.md) - 資料表定義
- [API 規格](./05_API_SPECIFICATION.md) - 端點詳情
- [UI/UX 指南](./10_UI_UX_GUIDELINES.md) - 導覽原則

---

*最後更新：2026-01-17*
