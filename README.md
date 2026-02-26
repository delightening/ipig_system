# 豬博士 iPig 系統

豬博士 iPig 系統為一套整合型實驗動物管理平台，採用**統一入口門戶**架構，使用者透過單一登入點存取所有子系統功能。

## 子系統組成

| 子系統 | 主要功能 | 目標使用者 |
|-------|---------|-----------|
| **AUP 提交與審查系統** | IACUC 動物試驗計畫書的撰寫、提交、審查、核准流程 | PI、審查委員、IACUC 行政人員、試驗工作人員 |
| **iPig ERP (進銷存管理系統)** | 物資採購、庫存、成本管理（飼料、藥品、器材、耗材） | 倉庫管理員、系統管理員 |
| **實驗動物管理系統** | 動物分配、實驗紀錄、健康監控、病歷管理 | PI、獸醫師、試驗工作人員、委託單位 |

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
│  │ • 提交與版本控管   │  │ • 庫存盤點         │  │ • 動物管理        │   │
│  │ • IACUC 審查      │  │ • 成本追蹤         │  │ • 實驗紀錄        │   │
│  │ • 核准/修訂/否決   │  │                   │  │ • 健康監控        │   │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                           共用資料層                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐              │
│  │ users   │ │ animals │ │protocols│ │   audit_logs    │              │
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
- 動物登錄與分配
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
| PI | 計畫主持人 | 提交計畫、管理動物 |
| VET | 獸醫師 | 審查計畫、健康管理 |
| REVIEWER | 審查委員 | IACUC 計畫審查 |
| CHAIR | IACUC 主席 | 主導審查決策 |
| IACUC_STAFF | 執行秘書 | 行政流程管理 |
| EXPERIMENT_STAFF | 試驗工作人員 | 執行實驗、記錄數據 |
| CLIENT | 委託人 | 查看委託計畫與動物紀錄 |

## 文件導覽

| 文件 | 說明 |
|------|------|
| [QUICK_START.md](docs/QUICK_START.md) | 快速啟動（Docker / 本地開發） |
| [USER_GUIDE.md](docs/USER_GUIDE.md) | 使用者操作手冊（登入、AUP、動物管理、ERP） |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | 部署與維運手冊（系統需求、首次部署、備份、監控、故障排除、GHCR/Watchtower） |
| [security.md](docs/security.md) | 安全性紀錄（已評估之 CVE 與處置；如 CVE-2026-25646 已記錄於此，非漏修） |

## 快速開始

本地或測試環境可依 [QUICK_START.md](docs/QUICK_START.md) 操作；**正式環境或伺服器首次部署**請依 [DEPLOYMENT.md](docs/DEPLOYMENT.md) 進行。

### 使用 Docker Compose（推薦）

```bash
# 1. 設定環境變數（專案根目錄）
cp .env.example .env
# 編輯 .env，至少設定 POSTGRES_PASSWORD、JWT_SECRET、ADMIN_INITIAL_PASSWORD

# 2. 啟動服務
docker compose up -d

# 查看日誌
docker compose logs -f
```

**服務入口：** 前端 http://localhost:8080 | API http://localhost:8000 | 資料庫 localhost:5433

### 本地開發

後端：`cd backend` → `cp env.sample .env` → `sqlx database create` → `sqlx migrate run` → `cargo run`  
前端：`cd frontend` → `npm install` → `npm run dev`

詳見 [QUICK_START.md](docs/QUICK_START.md)。

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

### 動物管理
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/animals` | GET/POST | 動物列表/新增 |
| `/animals/{id}` | GET/PATCH | 動物操作 |
| `/animals/{id}/observations` | GET/POST | 觀察試驗紀錄 |
| `/animals/{id}/surgeries` | GET/POST | 手術紀錄 |
| `/animals/{id}/weights` | GET/POST | 體重紀錄 |
| `/animals/{id}/vaccinations` | GET/POST | 疫苗/驅蟲紀錄 |
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
├── deploy/                 # 部署用設定（cloudflared、Grafana dashboard）
├── docs/                   # 說明文件（快速啟動、部署、使用者手冊等）
├── scripts/                # 腳本（啟動、隧道、備份、k6 等）
└── README.md
```

## 相關文件

| 文件 | 說明 |
|-----|------|
| [QUICK_START.md](docs/QUICK_START.md) | 快速啟動指南 |
| [USER_GUIDE.md](docs/USER_GUIDE.md) | 使用者操作手冊 |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | 部署與維運手冊 |
| [_Spec.md](_Spec.md) | 系統完整規格書 |
| [ERPSpec.md](ERPSpec.md) | 進銷存系統詳細規格 |
| [notificationSpec.md](notificationSpec.md) | 通知系統規格 |
| [skuSpec.md](skuSpec.md) | SKU 編碼規則 |
| [role.md](role.md) | 角色權限詳細說明 |
| [ipigmanager.md](ipigmanager.md) | 使用者操作手冊 |

## 授權

MIT License
