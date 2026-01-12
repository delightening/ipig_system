# 豬博士 iPig 系統

豬博士 iPig 系統為一套整合型實驗動物管理平台，採用**統一入口門戶**架構，使用者透過單一登入點存取所有子系統功能。

## 子系統組成

| 子系統 | 主要功能 | 目標使用者 |
|-------|---------|-----------|
| **AUP 提交與審查系統** | IACUC 動物試驗計畫書的撰寫、提交、審查、核准流程 | PI、審查委員、IACUC 行政人員、試驗工作人員 |
| **iPig ERP (進銷存管理系統)** | 物資採購、庫存、成本管理（飼料、藥品、器材、耗材） | 倉庫管理員、系統管理員 |
| **實驗動物管理系統** | 豬隻分配、實驗紀錄、健康監控、病歷管理 | PI、獸醫師、試驗工作人員、委託單位 |

## 技術架構

### 後端
- **語言**: Rust
- **框架**: Axum
- **資料庫**: PostgreSQL
- **ORM**: SQLx
- **認證**: JWT + Argon2

### 前端
- **框架**: React 18 + TypeScript
- **建構工具**: Vite
- **樣式**: Tailwind CSS
- **元件庫**: shadcn/ui (Radix UI)
- **狀態管理**: TanStack Query + Zustand
- **表格**: TanStack Table
- **表單**: React Hook Form + Zod

## 系統架構

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
│  └───────────────────┘  └───────────────────┘  └───────────────────┘   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                           共用資料層                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐              │
│  │ users   │ │  pigs   │ │protocols│ │   audit_logs    │              │
│  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────┘
```

## 功能特色

### AUP 審查系統
- 計畫書草稿撰寫與自動儲存
- 多版本控管（提交快照）
- IACUC 審查流程（預審→審查→核准/修訂/否決）
- 審查意見與回覆
- 附件管理

### iPig ERP (進銷存管理系統)
- 產品管理（SKU、批號、效期追蹤）
- 倉庫管理
- 庫存流水（GLP 合規）
- 低庫存警示

### 實驗動物管理系統
- 豬隻登錄與分配
- 觀察試驗紀錄
- 手術紀錄
- 體重追蹤
- 疫苗/驅蟲紀錄
- 犧牲採樣紀錄
- 病理組織報告
- 獸醫師建議與檢視追蹤

## 角色權限

| 角色代碼 | 角色名稱 | 說明 |
|---------|---------|------|
| SYSTEM_ADMIN | 系統管理員 | 全系統最高權限 |
| WAREHOUSE_MANAGER | 倉庫管理員 | 進銷存系統操作 |
| PROGRAM_ADMIN | 程式管理員 | 系統程式層級管理 |
| PI | 計畫主持人 | 提交計畫、管理豬隻 |
| VET | 獸醫師 | 審查計畫、健康管理 |
| REVIEWER | 審查委員 | IACUC 計畫審查 |
| CHAIR | IACUC 主席 | 主導審查決策 |
| IACUC_STAFF | 執行秘書 | 行政流程管理 |
| EXPERIMENT_STAFF | 試驗工作人員 | 執行實驗、記錄數據 |
| CLIENT | 委託人 | 查看委託計畫與豬隻紀錄 |

## 快速開始

### 使用 Docker Compose（推薦）

#### 1. 設定環境變數

在項目根目錄創建 `.env` 文件，複製 `backend/env.sample` 並填入實際值：

```bash
# 複製範例文件
cp backend/env.sample .env

# 編輯 .env 文件，填入必要的環境變數
```

**重要環境變數說明：**

| 變數 | 說明 |
|-----|------|
| `POSTGRES_PASSWORD` | 資料庫密碼（請使用強密碼） |
| `JWT_SECRET` | JWT 簽名密鑰（必須使用安全的隨機值） |
| `SMTP_HOST` | SMTP 伺服器（可選，郵件通知用） |

**生成安全的 JWT_SECRET（PowerShell）：**

```powershell
$jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
Write-Output $jwt
```

#### 2. 啟動服務

```bash
# 啟動所有服務
docker-compose up -d

# 查看日誌
docker-compose logs -f

# 停止服務
docker-compose down
```

**服務入口：**
| 服務 | 網址 |
|-----|------|
| 前端 | http://localhost:8080 |
| API | http://localhost:3000 |
| 資料庫 | localhost:5433 |

### 本地開發

#### 後端

```bash
cd backend

# 建立 .env 檔案
cp env.sample .env
# 編輯 .env 設定資料庫連線等

# 安裝 SQLx CLI
cargo install sqlx-cli

# 建立資料庫
sqlx database create

# 執行 migrations
sqlx migrate run

# 啟動開發伺服器
cargo run
```

#### 前端

```bash
cd frontend

# 安裝相依套件
npm install

# 啟動開發伺服器
npm run dev
```

## 預設帳號

| 帳號 | 密碼 |
|-----|------|
| admin@ipig.local | admin123 |

## API 文件

### 認證
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/auth/login` | POST | 使用者登入 |
| `/auth/refresh` | POST | 更新 Access Token |
| `/auth/logout` | POST | 登出 |
| `/auth/forgot-password` | POST | 寄送密碼重設信件 |
| `/auth/reset-password` | POST | 重設密碼 |

### 計畫書（AUP）
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/protocols` | GET/POST | 計畫書列表/建立 |
| `/protocols/{id}` | GET/PATCH | 計畫書操作 |
| `/protocols/{id}/submit` | POST | 提交計畫書 |
| `/protocols/{id}/status` | POST | 變更狀態 |
| `/protocols/{id}/versions` | GET | 取得版本列表 |

### iPig ERP (進銷存)
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/warehouses` | GET/POST | 倉庫列表/建立 |
| `/products` | GET/POST | 產品列表/建立 |
| `/inventory/on-hand` | GET | 庫存現況 |
| `/inventory/ledger` | GET | 庫存流水 |

### 豬隻管理
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/pigs` | GET/POST | 豬隻列表/新增 |
| `/pigs/{id}` | GET/PATCH | 豬隻操作 |
| `/pigs/{id}/observations` | GET/POST | 觀察試驗紀錄 |
| `/pigs/{id}/surgeries` | GET/POST | 手術紀錄 |
| `/pigs/{id}/weights` | GET/POST | 體重紀錄 |
| `/pigs/{id}/vaccinations` | GET/POST | 疫苗/驅蟲紀錄 |
| `/my-projects` | GET | 我的計劃 |

## 專案結構

```
ipig_system/
├── backend/                  # Rust 後端
│   ├── src/
│   │   ├── main.rs          # 程式入口
│   │   ├── config.rs        # 設定
│   │   ├── error.rs         # 錯誤處理
│   │   ├── routes.rs        # 路由
│   │   ├── models/          # 資料模型
│   │   ├── handlers/        # 請求處理器
│   │   ├── services/        # 業務邏輯
│   │   └── middleware/      # 中間件
│   ├── migrations/          # 資料庫遷移
│   └── Cargo.toml
├── frontend/                 # React 前端
│   ├── src/
│   │   ├── main.tsx         # 程式入口
│   │   ├── App.tsx          # 主元件
│   │   ├── components/      # UI 元件
│   │   ├── pages/           # 頁面元件
│   │   ├── layouts/         # 佈局元件
│   │   ├── stores/          # 狀態管理
│   │   └── lib/             # 工具函式
│   └── package.json
├── _Spec.md                  # 系統規格書
├── docker-compose.yml        # Docker 編排
└── README.md
```

## 相關文件

| 文件 | 說明 |
|-----|------|
| [_Spec.md](_Spec.md) | 系統完整規格書 |
| [ERPSpec.md](ERPSpec.md) | 進銷存系統詳細規格 |
| [notificationSpec.md](notificationSpec.md) | 通知系統規格 |
| [skuSpec.md](skuSpec.md) | SKU 編碼規則 |
| [role.md](role.md) | 角色權限詳細說明 |
| [ipigmanager.md](ipigmanager.md) | 使用者操作手冊 |

## 授權

MIT License
