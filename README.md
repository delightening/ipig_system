# 豬博士 iPig 系統

> **版本：** v2.0 | **狀態：** 功能完整，上線準備中 | **最後更新：** 2026-03-01

豬博士 iPig 系統為一套整合型實驗動物管理平台，採用**統一入口門戶**架構，使用者透過單一登入點存取所有子系統功能。系統嚴格遵循 GLP (Good Laboratory Practice) 合規要求，涵蓋 AUP 計畫書審查、進銷存、動物醫療紀錄、人事管理與稽核安全。

---

## 子系統組成

| 子系統 | 主要功能 | 目標使用者 |
|-------|---------|-----------|
| **AUP 提交與審查系統** | IACUC 動物試驗計畫書撰寫、提交、審查、核准、變更申請 | PI、審查委員、IACUC 行政人員、試驗工作人員 |
| **iPig ERP (進銷存)** | 產品/SKU、倉庫/儲位、單據流程、庫存追蹤、低庫存警示 | 倉庫管理員、系統管理員 |
| **實驗動物管理系統** | 動物分配、觀察/手術/血檢/體重/疫苗、安樂死、轉讓、病理報告 | PI、獸醫師、試驗工作人員、委託單位 |
| **人事管理 (HR)** | 打卡、請假、加班、年假/補休、Google Calendar 同步 | 全體員工、主管 |
| **通知系統** | 站內通知、Email、可配置路由、排程報表 | 全體使用者 |
| **設施管理** | 物種、建築、區域、欄舍階層管理 | 系統管理員 |

---

## 技術架構

### 後端
- **語言**: Rust
- **框架**: Axum 0.7
- **資料庫**: PostgreSQL 16
- **ORM**: SQLx 0.8
- **認證**: JWT (HttpOnly Cookie) + Refresh Token + TOTP 2FA
- **密碼**: Argon2

### 前端
- **框架**: React 18 + TypeScript
- **建構工具**: Vite
- **樣式**: Tailwind CSS
- **元件庫**: shadcn/ui (Radix UI)
- **狀態管理**: TanStack Query + Zustand
- **表格**: TanStack Table
- **表單**: React Hook Form + Zod
- **i18n**: react-i18next (繁中 / 英文)

### 部署與維運
- **容器**: Docker Compose，三層網路隔離、Docker Secrets
- **監控**: Prometheus、Grafana、Alertmanager
- **WAF**: ModSecurity + OWASP CRS（選用 overlay）
- **備份**: pg_dump + GPG 加密 + rsync

---

## 系統架構

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        豬博士 iPig 統一入口門戶                               │
│                                                                             │
│  ┌─────────────┐    ┌─────────────────┐    ┌─────────────────────────┐    │
│  │ 登入認證     │───│ 角色權限控管     │───│ 功能路由分派             │    │
│  │ JWT + 2FA   │    │ RBAC            │    │ /api/*                   │    │
│  └─────────────┘    └─────────────────┘    └─────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ AUP 審查     │ │ iPig ERP     │ │ 動物管理     │ │ HR 人事      │       │
│  │ 計畫書/審查  │ │ 進銷存/庫存  │ │ 紀錄/血檢    │ │ 打卡/請假    │       │
│  │ 變更申請    │ │ 單據流程     │ │ 安樂死/轉讓  │ │ 行事曆同步   │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
│                                                                             │
│  ┌──────────────┐ ┌──────────────┐                                         │
│  │ 通知系統     │ │ 設施管理     │                                         │
│  │ 站內/Email  │ │ 物種/欄舍    │                                         │
│  └──────────────┘ └──────────────┘                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ 共用資料層：users | roles | permissions | animals | protocols | audit_logs   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 功能特色

### AUP 審查系統
- 計畫書草稿撰寫與自動儲存、多版本控管
- IACUC 審查流程（預審→審查→核准/修訂/否決）
- 審查意見與回覆、共同編輯者、變更申請 (Amendment)
- 附件管理、電子簽章、PDF 匯出

### iPig ERP (進銷存)
- 產品管理（SKU 自動編碼、批號、效期追蹤）
- 倉庫與儲位管理、庫存流水（GLP 合規）
- 單據工作流（建立→提交→核准→取消）
- 低庫存與到期日警示

### 實驗動物管理系統
- 動物登錄、批次分配、匯入匯出
- 觀察試驗、手術、體重、疫苗/驅蟲、血液檢查
- 疼痛評估、獸醫建議（含附件）
- 犧牲/採樣、安樂死審批、轉讓、猝死登記
- 病理組織報告、電子簽章

### 人事管理 (HR)
- 打卡（IP + GPS 雙重驗證）、請假、加班
- 年假/補休餘額、代理人機制
- Google Calendar 雙向同步、行事曆總覽

### 安全與合規
- TOTP 2FA、敏感操作二級認證
- CSRF、Rate Limiting、DOMPurify XSS 防護
- 稽核日誌、HMAC 完整性驗證、資料保留政策

---

## 角色權限

| 角色代碼 | 角色名稱 | 說明 |
|---------|---------|------|
| SYSTEM_ADMIN | 系統管理員 | 全系統最高權限 |
| WAREHOUSE_MANAGER | 倉庫管理員 | 進銷存系統操作 |
| EQUIPMENT_MAINTENANCE | 設備維護人員 | 設備與校準紀錄管理 |
| PROGRAM_ADMIN | 程式管理員 | 系統程式層級管理 |
| PI | 計畫主持人 | 提交計畫、管理動物 |
| VET | 獸醫師 | 審查計畫、健康管理 |
| REVIEWER | 審查委員 | IACUC 計畫審查 |
| CHAIR | IACUC 主席 | 主導審查決策 |
| IACUC_STAFF | 執行秘書 | 行政流程管理 |
| EXPERIMENT_STAFF | 試驗工作人員 | 執行實驗、記錄數據 |
| CLIENT | 委託人 | 查看委託計畫與動物紀錄 |

---

## 資料夾一覽

| 資料夾 | 用途 | 說明 |
|--------|------|------|
| **backend/** | 後端服務 | Rust (Axum) API、migrations、整合測試 |
| **frontend/** | 前端應用 | React + TypeScript、E2E (Playwright) |
| **docs/** | 說明文件 | [文件索引](docs/README.md) 依主題分類（規格、開發、資料庫、安全、維運、runbook） |
| **scripts/** | 腳本工具 | [scripts/README.md](scripts/README.md) 啟動、CI、備份、部署、環境驗證 |
| **tests/** | Python 整合測試 | [tests/README.md](tests/README.md) AUP/ERP/動物/HR 等 8 大模組 |
| **monitoring/** | 監控設定 | [monitoring/README.md](monitoring/README.md) Prometheus、Alertmanager、Promtail |
| **deploy/** | 部署與儀表板 | [deploy/README.md](deploy/README.md) Grafana、Prometheus、cloudflared、WAF 排除規則 |
| **.github/** | CI/CD | GitHub Actions 工作流程 |

**依角色閱讀**：開發者 → docs/README.md → development/、QUICK_START；維運 → operations/OPERATIONS.md、DEPLOYMENT、runbooks/；測試 → tests/README、docs/e2e/。

---

## 文件導覽

| 文件 | 說明 |
|------|------|
| [文件索引 (docs/)](docs/README.md) | 依主題分類之完整文件目錄 |
| [QUICK_START.md](docs/QUICK_START.md) | 快速啟動（Docker / 本地開發 / E2E 測試） |
| [OPERATIONS.md](docs/operations/OPERATIONS.md) | 維運手冊（服務擁有者、on-call、故障排除） |
| [COMPOSE.md](docs/operations/COMPOSE.md) | Docker Compose 總覽（各 compose 檔用途、情境指令、依賴關係） |
| [USER_GUIDE.md](docs/USER_GUIDE.md) | 使用者操作手冊（9 章節完整指南） |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | 部署與維運手冊（系統需求、備份、監控、故障排除） |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | 架構文件（部署圖、資料流、模組、認證流程） |
| [security.md](docs/security-compliance/security.md) | 安全性紀錄（CVE 評估與處置） |
| [Profiling_Spec/](docs/Profiling_Spec/) | 技術規格（架構、API、權限、資料庫） |

---

## 快速開始

本地或測試環境可依 [QUICK_START.md](docs/QUICK_START.md) 操作；**正式環境或伺服器首次部署**請依 [DEPLOYMENT.md](docs/DEPLOYMENT.md) 進行。

### 使用 Docker Compose（推薦）

各 compose 檔用途與生產／WAF／CI 組合指令詳見 [COMPOSE.md](docs/operations/COMPOSE.md)。

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

---

## 預設帳號

| 帳號 | 密碼 |
|-----|------|
| admin@ipig.local | admin123 |

（正式環境請於 .env 設定 `ADMIN_INITIAL_PASSWORD` 並關閉 `SEED_DEV_USERS`。）

---

## API 文件

Swagger UI：`http://localhost:8000/swagger-ui/`（服務啟動後）

### 認證
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/auth/login` | POST | 使用者登入 |
| `/auth/refresh` | POST | 更新 Access Token |
| `/auth/logout` | POST | 登出 |
| `/auth/2fa/setup` | POST | TOTP 2FA 設定 |
| `/auth/2fa/verify` | POST | TOTP 驗證登入 |
| `/auth/forgot-password` | POST | 寄送密碼重設信件 |
| `/auth/reset-password` | POST | 重設密碼 |

### 計畫書（AUP）
| 端點 | 方法 | 說明 |
|-----|------|------|
| `/protocols` | GET/POST | 計畫書列表/建立 |
| `/protocols/{id}` | GET/PATCH | 計畫書操作 |
| `/protocols/{id}/submit` | POST | 提交計畫書 |
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

---

## 專案結構

```
ipig_system/
├── backend/                  # Rust 後端
│   ├── src/
│   │   ├── main.rs           # 程式入口
│   │   ├── config.rs         # 設定 + Docker Secrets
│   │   ├── error.rs          # 錯誤處理
│   │   ├── routes.rs         # 路由
│   │   ├── handlers/         # 請求處理器（按模組分）
│   │   ├── services/         # 業務邏輯
│   │   ├── middleware/       # Auth, CSRF, Rate Limiter
│   │   └── models/           # 資料模型
│   ├── migrations/           # 19 個 SQL 遷移
│   └── Cargo.toml
├── frontend/                 # React 前端
│   ├── src/
│   │   ├── main.tsx          # 程式入口
│   │   ├── App.tsx           # 主元件
│   │   ├── components/       # UI 元件
│   │   ├── pages/            # 頁面元件
│   │   ├── layouts/          # 佈局元件
│   │   ├── stores/           # Zustand 狀態
│   │   ├── lib/              # api, queryKeys, validations
│   │   └── locales/          # i18n (zh-TW, en)
│   └── package.json
├── deploy/                   # 部署設定（Grafana、Prometheus、cloudflared）
├── docs/                     # 說明文件
├── monitoring/               # Prometheus、Alertmanager 設定
├── scripts/                  # 備份、k6、驗證等腳本
├── docker-compose.yml        # 核心服務
├── docker-compose.prod.yml   # 生產環境覆蓋
├── docker-compose.waf.yml    # WAF 覆蓋（選用）
├── docker-compose.logging.yml    # 日誌堆疊 Loki+Promtail（選用）
├── docker-compose.monitoring.yml # 監控堆疊 Prometheus+Grafana（選用）
├── docker-compose.test.yml   # CI 測試用（獨立 stack）
└── README.md
```

---

## 上線準備度

| 面向 | 狀態 |
|------|------|
| 測試 | Rust 119 unit tests、API 整合測試 25+ cases、E2E 7 spec 34 tests |
| 可觀測性 | /health、/metrics、Prometheus、Grafana Dashboard |
| 備份 / DR | GPG 加密備份、DR Runbook |
| 安全性 | 2FA、WAF overlay、容器掃描、Docker Secrets |
| GLP 合規 | 電子簽章、GLP 驗證文件、資料保留政策 |
| 效能基準 | k6 壓力測試、正式基準報告 |

---

## 授權

MIT License © 2026 iPig System Contributors
